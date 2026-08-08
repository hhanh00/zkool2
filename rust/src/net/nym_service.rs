//! Mixnet-native RPC endpoints (`nym://` URLs).
//!
//! Speaks the [nym-rpc](https://github.com/rachyandco/nym-rpc) raw-tunnel
//! protocol: gRPC bytes are framed into bincode-serialized `ProxiedMessage`s
//! (nym-sdk `tcp_proxy` wire types) and exchanged with a nym-rpc server
//! addressed by its Nym recipient key — no IPR exit, no DNS, no clearnet hop
//! on the client side. The first data message of each session carries an
//! `UPSTREAM:host:port\n` hint; public nym-rpc servers pin their upstream
//! and ignore it.
//!
//! Integration mirrors nym-rpc's `TcpProxyClient`: a localhost forwarder
//! accepts plain TCP (tonic connects to it with h2c — Sphinx already
//! provides end-to-end encryption) and each accepted connection becomes an
//! ordered mixnet session.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use nym_sdk::client_pool::ClientPool;
use nym_sdk::mixnet::{
    IncludedSurbs, MixnetClient, MixnetClientBuilder, MixnetMessageSender, NymNetworkDetails,
    Recipient,
};
use nym_sdk::tcp_proxy::utils::{MessageBuffer, Payload, ProxiedMessage};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, Mutex, OnceCell};
use tokio_stream::StreamExt;
use tokio_util::codec::{BytesCodec, FramedRead};
use tonic::transport::{Channel, Endpoint};

pub const NYM_URL_SCHEME: &str = "nym://";
/// Upstream hint sent in the first message of a session (nym-rpc `--zcash`
/// preset value). Public nym-rpc servers pin their upstream and ignore it.
const DEFAULT_UPSTREAM: &str = "127.0.0.1:8137";
/// Idle time after local EOF before the session's mixnet client is released.
const CLOSE_TIMEOUT: Duration = Duration::from_secs(60);
/// Give up building a mixnet client after this long.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(90);
/// Pre-warmed mixnet clients (same as nym-rpc's default).
const POOL_SIZE: usize = 2;
/// Kill a session when a request has gone unanswered this long: bytes went
/// out after the last incoming message and nothing has come back since.
/// Long-lived quiet streams (e.g. the mempool long-poll, which receives
/// last) are not affected. This turns a lost RPC response — e.g. a
/// broadcast whose reply the mixnet dropped — into an error the app can
/// surface instead of hanging forever.
const STALL_TIMEOUT: Duration = Duration::from_secs(120);

/// Parse a `nym://<identity>.<encryption>@<gateway>` URL into a Recipient.
pub fn parse_nym_url(url: &str) -> Option<Recipient> {
    let rest = url.trim().strip_prefix(NYM_URL_SCHEME)?;
    Recipient::try_from_base58_string(rest.trim_end_matches('/')).ok()
}

/// One localhost forwarder per recipient, started on first use.
static FORWARDERS: Mutex<Option<HashMap<String, u16>>> = Mutex::const_new(None);
/// One shared gRPC channel per recipient: every RPC (sync, mempool,
/// block-height poll, broadcast) multiplexes over one warm mixnet session
/// instead of paying a client bootstrap per connection.
static CHANNELS: Mutex<Option<HashMap<String, Channel>>> = Mutex::const_new(None);
static POOL: OnceCell<Arc<ClientPool>> = OnceCell::const_new();

/// Shared gRPC channel to `recipient`. Lazy: tonic (re)connects through the
/// local forwarder on demand, so a dead session heals on the next RPC.
pub async fn grpc_channel(recipient: Recipient) -> Result<Channel> {
    let mut channels = CHANNELS.lock().await;
    let channels = channels.get_or_insert_with(HashMap::new);
    let key = recipient.to_string();
    if let Some(channel) = channels.get(&key) {
        return Ok(channel.clone());
    }
    let port = local_forwarder_port(recipient).await?;
    let endpoint = Endpoint::from_shared(format!("http://127.0.0.1:{port}"))?
        .connect_timeout(Duration::from_secs(30));
    let channel = endpoint.connect_lazy();
    channels.insert(key, channel.clone());
    Ok(channel)
}

/// Local TCP port forwarding to `recipient` through the mixnet, binding a
/// listener for it on first use.
pub async fn local_forwarder_port(recipient: Recipient) -> Result<u16> {
    let mut forwarders = FORWARDERS.lock().await;
    let forwarders = forwarders.get_or_insert_with(HashMap::new);
    let key = recipient.to_string();
    if let Some(port) = forwarders.get(&key) {
        return Ok(*port);
    }
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let port = listener.local_addr()?.port();
    tracing::info!("nym-rpc forwarder for {key} listening on 127.0.0.1:{port}");
    tokio::spawn(accept_loop(listener, recipient));
    forwarders.insert(key, port);
    Ok(port)
}

async fn accept_loop(listener: TcpListener, recipient: Recipient) {
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    if let Err(e) = run_session(stream, recipient).await {
                        tracing::warn!("nym-rpc session failed: {e:#}");
                    }
                });
            }
            Err(e) => {
                tracing::warn!("nym-rpc forwarder accept failed: {e}");
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
}

/// A mixnet client from the shared pool, or an ephemeral one if the pool
/// has none ready yet.
async fn get_client() -> Result<MixnetClient> {
    let pool = POOL
        .get_or_init(|| async {
            // Populate the nym network env (mainnet defaults) before any
            // client is built — both the pool's internal builder and the
            // ephemeral fallback read it (mirrors nym-rpc's setup_env call).
            nym_network_defaults::setup_env(None::<&str>);
            let pool = Arc::new(ClientPool::new(POOL_SIZE));
            let filler = Arc::clone(&pool);
            tokio::spawn(async move {
                if let Err(e) = filler.start().await {
                    tracing::warn!("nym-rpc client pool stopped: {e:#}");
                }
            });
            pool
        })
        .await;
    if let Some(client) = pool.get_mixnet_client().await {
        return Ok(client);
    }
    tracing::info!("nym-rpc client pool empty; building an ephemeral mixnet client");
    let net = NymNetworkDetails::new_from_env();
    let client = tokio::time::timeout(CLIENT_TIMEOUT, async {
        MixnetClientBuilder::new_ephemeral()
            .network_details(net)
            .build()?
            .connect_to_mixnet()
            .await
            .map_err(anyhow::Error::from)
    })
    .await
    .map_err(|_| anyhow!("mixnet client bootstrap timed out"))??;
    Ok(client)
}

/// One ordered mixnet session per local TCP connection, mirroring nym-rpc's
/// `TcpProxyClient::handle_incoming`.
async fn run_session(stream: TcpStream, recipient: Recipient) -> Result<()> {
    let session_id = uuid::Uuid::new_v4();
    let mut client = get_client().await?;
    tracing::debug!("nym-rpc session {session_id} started");

    let (tx, mut rx) = oneshot::channel();
    let (read, mut write) = stream.into_split();
    let mut framed_read = FramedRead::new(read, BytesCodec::new());
    let sender = client.split_sender();

    // Seconds-since-session-start of the last outgoing send / incoming
    // message, for stall detection.
    let started = Instant::now();
    let last_out = Arc::new(AtomicU64::new(0));
    let last_out_writer = Arc::clone(&last_out);
    let mut last_in: u64 = 0;

    // Outgoing: local bytes -> ordered ProxiedMessages through the mixnet,
    // UPSTREAM hint on the first message, Close on local EOF.
    let out_started = started;
    tokio::spawn(async move {
        let mut message_id: u16 = 0;
        while let Some(Ok(bytes)) = framed_read.next().await {
            message_id += 1;
            let data = if message_id == 1 {
                let mut framed = format!("UPSTREAM:{DEFAULT_UPSTREAM}\n").into_bytes();
                framed.extend_from_slice(&bytes);
                framed
            } else {
                bytes.to_vec()
            };
            let message = ProxiedMessage::new(Payload::Data(data), session_id, message_id);
            sender
                .send_message(recipient, &bincode1::serialize(&message)?, IncludedSurbs::Amount(100))
                .await?;
            last_out_writer.store(out_started.elapsed().as_secs().max(1), Ordering::Relaxed);
        }
        message_id += 1;
        let message = ProxiedMessage::new(Payload::Close, session_id, message_id);
        sender
            .send_message(recipient, &bincode1::serialize(&message)?, IncludedSurbs::Amount(100))
            .await?;
        tracing::debug!("nym-rpc session {session_id}: local EOF, Close sent");
        let _ = tx.send(true);
        Ok::<_, anyhow::Error>(())
    });

    // Incoming: reorder mixnet messages and write them to the local socket;
    // after local EOF keep draining for CLOSE_TIMEOUT.
    let mut msg_buffer = MessageBuffer::new();
    loop {
        tokio::select! {
            _ = &mut rx => break,
            Some(message) = client.next() => {
                let message = bincode1::deserialize::<ProxiedMessage>(&message.message)?;
                msg_buffer.push(message);
                msg_buffer.tick(&mut write).await?;
                last_in = started.elapsed().as_secs().max(1);
            },
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                msg_buffer.tick(&mut write).await?;
                // Stall: we sent a request after the last reply and the
                // mixnet has returned nothing since. Drop the session so
                // the caller gets an error instead of waiting forever.
                let out = last_out.load(Ordering::Relaxed);
                if out > last_in
                    && started.elapsed().as_secs().saturating_sub(last_in.max(out))
                        > STALL_TIMEOUT.as_secs()
                {
                    tracing::warn!(
                        "nym-rpc session {session_id} stalled (no reply for {}s); closing",
                        STALL_TIMEOUT.as_secs()
                    );
                    client.disconnect().await;
                    return Ok(());
                }
            }
        }
    }
    loop {
        tokio::select! {
            Some(message) = client.next() => {
                let message = bincode1::deserialize::<ProxiedMessage>(&message.message)?;
                msg_buffer.push(message);
                msg_buffer.tick(&mut write).await?;
            },
            _ = tokio::time::sleep(CLOSE_TIMEOUT) => {
                tracing::debug!("nym-rpc session {session_id} closed");
                client.disconnect().await;
                return Ok(());
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADDR: &str = "BbTPrU1gNTsPiieXdC58xkp5QFSHhUUM98BP1Rm2adf9.GKiGLNQB116YszFwbuweeL2GsrfpHpuUzq6JuqFQ8EEE@ZXSDhRTKU5HgMpH8ma78FftvLiKyZ6jWL1e2U7GD7gQ";

    #[test]
    fn parses_nym_scheme_url() {
        assert!(parse_nym_url(&format!("nym://{ADDR}")).is_some());
        assert!(parse_nym_url(&format!("nym://{ADDR}/")).is_some());
        assert!(parse_nym_url(&format!("  nym://{ADDR}  ")).is_some());
    }

    #[test]
    fn rejects_non_nym_urls() {
        assert!(parse_nym_url(ADDR).is_none()); // scheme required
        assert!(parse_nym_url("https://zec.rocks").is_none());
        assert!(parse_nym_url("nym://not-a-recipient").is_none());
        assert!(parse_nym_url("").is_none());
    }
}
