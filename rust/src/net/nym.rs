//! Nym mixnet transport.
//!
//! Provides TCP streams through the Nym mixnet via a shared
//! `nym_smolmix::Tunnel`. Hostname resolution also goes over the tunnel
//! (DNS to 1.1.1.1 via the mixnet UDP socket) so server names are never
//! resolved locally.
//!
//! Mixnet connections can stall or drop (gateway churn, bridge shutdown),
//! so the tunnel is not cached unconditionally: any transport-level failure
//! invalidates the cached tunnel and the next call bootstraps a fresh one.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Mutex as StdMutex;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use hickory_proto::op::{Message, Query};
use hickory_proto::rr::{Name, RData, RecordType};
use nym_smolmix::{TcpStream, Tunnel};
use tokio::sync::Mutex;

/// Per-attempt DNS query timeout (a fresh socket is used for the retry).
const DNS_TIMEOUT: Duration = Duration::from_secs(10);
const DNS_ATTEMPTS: usize = 2;
/// How long a resolved address may be reused without a new lookup.
const DNS_TTL: Duration = Duration::from_secs(300);
/// Give up on a TCP connect through the mixnet after this long.
const TCP_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
/// Give up on tunnel bootstrap after this long. The bootstrap holds the
/// tunnel lock, so a hang here would otherwise block every caller forever.
const BOOTSTRAP_TIMEOUT: Duration = Duration::from_secs(90);

struct TunnelSlot {
    tunnel: Option<(u64, Tunnel)>,
    next_gen: u64,
}

static TUNNEL: Mutex<TunnelSlot> = Mutex::const_new(TunnelSlot {
    tunnel: None,
    next_gen: 0,
});

static DNS_CACHE: StdMutex<Option<HashMap<String, (IpAddr, Instant)>>> = StdMutex::new(None);

/// Get the shared mixnet tunnel, bootstrapping one if none is cached.
/// The lock is held during bootstrap so concurrent callers wait for the
/// same tunnel instead of racing to build their own.
async fn get_tunnel() -> Result<(u64, Tunnel)> {
    let mut slot = TUNNEL.lock().await;
    if let Some((gen, tunnel)) = &slot.tunnel {
        return Ok((*gen, tunnel.clone()));
    }
    tracing::info!("bootstrapping Nym mixnet tunnel");
    let start = Instant::now();
    let tunnel = tokio::time::timeout(BOOTSTRAP_TIMEOUT, Tunnel::builder().build())
        .await
        .map_err(|_| anyhow::anyhow!("Nym mixnet tunnel bootstrap timed out"))?
        .context("failed to bootstrap Nym mixnet tunnel")?;
    tracing::info!("Nym mixnet tunnel ready in {:?}", start.elapsed());
    let gen = slot.next_gen;
    slot.next_gen += 1;
    slot.tunnel = Some((gen, tunnel.clone()));
    Ok((gen, tunnel))
}

/// Discard the cached tunnel if it is still generation `gen`, so the next
/// call bootstraps a fresh one. The stale tunnel is shut down in the
/// background.
async fn invalidate_tunnel(gen: u64) {
    let mut slot = TUNNEL.lock().await;
    if let Some((g, tunnel)) = &slot.tunnel {
        if *g == gen {
            tracing::warn!("Nym tunnel failed; discarding it and clearing the DNS cache");
            let tunnel = tunnel.clone();
            slot.tunnel = None;
            if let Ok(mut cache) = DNS_CACHE.lock() {
                *cache = None;
            }
            tokio::spawn(async move { tunnel.shutdown().await });
        }
    }
}

/// Open a TCP stream to (host, port) through the Nym mixnet.
/// Transport-level failures invalidate the shared tunnel so the next
/// attempt (e.g. the mempool retry loop) recovers with a fresh one.
pub async fn nym_connect(host: &str, port: u16) -> Result<TcpStream> {
    let (gen, tunnel) = get_tunnel().await?;

    let ip = match resolve_over_nym(&tunnel, host).await {
        Ok(Some(ip)) => ip,
        // The DNS server answered but had no A record: the tunnel works,
        // the hostname is just unresolvable.
        Ok(None) => anyhow::bail!("no A record for {host} from mixnet DNS"),
        Err(e) => {
            invalidate_tunnel(gen).await;
            return Err(e);
        }
    };

    match tokio::time::timeout(TCP_CONNECT_TIMEOUT, tunnel.tcp_connect(SocketAddr::new(ip, port)))
        .await
    {
        Ok(Ok(stream)) => Ok(stream),
        Ok(Err(e)) => {
            invalidate_tunnel(gen).await;
            Err(e).with_context(|| format!("mixnet TCP connect to {host}:{port} failed"))
        }
        Err(_) => {
            invalidate_tunnel(gen).await;
            anyhow::bail!("mixnet TCP connect to {host}:{port} timed out")
        }
    }
}

/// Resolve `host` through the mixnet unless it is an IP literal or freshly
/// cached. Returns `Ok(None)` when the DNS server responded without an A
/// record (tunnel healthy, name unresolvable); `Err` on transport failure.
async fn resolve_over_nym(tunnel: &Tunnel, host: &str) -> Result<Option<IpAddr>> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(Some(ip));
    }
    if let Some(ip) = dns_cache_get(host) {
        return Ok(Some(ip));
    }

    let mut last_err = None;
    for attempt in 0..DNS_ATTEMPTS {
        // The timeout covers the whole query: socket creation and send can
        // also stall when the tunnel's bridge is unhealthy, not just recv.
        let query = tokio::time::timeout(DNS_TIMEOUT, dns_query(tunnel, host))
            .await
            .map_err(|_| anyhow::anyhow!("mixnet DNS lookup for {host} timed out"))
            .and_then(|r| r);
        match query {
            Ok(answer) => {
                if let Some(ip) = answer {
                    dns_cache_put(host, ip);
                }
                return Ok(answer);
            }
            Err(e) => {
                tracing::warn!("mixnet DNS lookup for {host} failed (attempt {attempt}): {e:#}");
                last_err = Some(e);
            }
        }
    }
    Err(last_err.expect("DNS_ATTEMPTS > 0"))
}

/// One DNS A query over a fresh tunnel UDP socket.
/// `Ok(None)` means the server responded without a usable A record.
async fn dns_query(tunnel: &Tunnel, host: &str) -> Result<Option<IpAddr>> {
    let udp = tunnel.udp_socket().await?;
    let mut query = Message::new();
    query.set_recursion_desired(true);
    query.add_query(Query::query(Name::from_ascii(host)?, RecordType::A));
    udp.send_to(&query.to_vec()?, "1.1.1.1:53".parse()?).await?;
    let mut buf = [0u8; 1500];
    let (len, _src) = udp.recv_from(&mut buf).await?;
    let response = Message::from_vec(&buf[..len])?;
    Ok(first_a_record(&response))
}

fn dns_cache_get(host: &str) -> Option<IpAddr> {
    let cache = DNS_CACHE.lock().ok()?;
    let (ip, at) = cache.as_ref()?.get(host)?;
    (at.elapsed() < DNS_TTL).then_some(*ip)
}

fn dns_cache_put(host: &str, ip: IpAddr) {
    if let Ok(mut cache) = DNS_CACHE.lock() {
        cache
            .get_or_insert_with(HashMap::new)
            .insert(host.to_string(), (ip, Instant::now()));
    }
}

/// First A record in a DNS response, if any.
fn first_a_record(msg: &Message) -> Option<IpAddr> {
    msg.answers().iter().find_map(|record| match record.data() {
        Some(RData::A(a)) => Some(IpAddr::V4(a.0)),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    use hickory_proto::op::{Message, MessageType, Query};
    use hickory_proto::rr::{rdata, Name, RData, Record, RecordType};

    use super::*;

    #[test]
    fn first_a_record_skips_cname_and_picks_a() {
        let name = Name::from_str("example.com.").unwrap();
        let mut msg = Message::new();
        msg.set_message_type(MessageType::Response);
        msg.add_query(Query::query(name.clone(), RecordType::A));
        msg.add_answer(Record::from_rdata(
            name.clone(),
            300,
            RData::CNAME(rdata::CNAME(Name::from_str("alias.example.com.").unwrap())),
        ));
        msg.add_answer(Record::from_rdata(
            name,
            300,
            RData::A(rdata::A(Ipv4Addr::new(93, 184, 216, 34))),
        ));
        assert_eq!(
            first_a_record(&msg),
            Some(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)))
        );
    }

    #[test]
    fn first_a_record_none_when_no_a_answer() {
        let msg = Message::new();
        assert_eq!(first_a_record(&msg), None);
    }

    #[test]
    fn dns_cache_round_trip_and_expiry_check() {
        let ip = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
        dns_cache_put("cache-test.example", ip);
        assert_eq!(dns_cache_get("cache-test.example"), Some(ip));
        assert_eq!(dns_cache_get("cache-miss.example"), None);
    }
}
