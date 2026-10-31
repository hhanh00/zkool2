use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use anyhow::Result;
use arti_client::config::TorClientConfigBuilder;
use arti_client::TorClient;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;
use hyper_util::rt::TokioIo;
use sqlx::pool::PoolConnection;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Sqlite, SqlitePool};
use tokio::sync::{Mutex, OnceCell};
use tonic::transport::{Channel, ClientTlsConfig, Endpoint, Uri};
use tor_rtcompat::PreferredRuntime;
use tower::service_fn;
use zcash_protocol::consensus::{BlockHeight, OrchardMode};
use zcash_protocol::local_consensus::LocalNetwork;

use crate::db::{backfill_diversifier_index, create_schema, migrate_sapling_addresses};
use crate::lwd::compact_tx_streamer_client::CompactTxStreamerClient;
use crate::net::zebra::ZebraClient;
use crate::sync::driver::SyncDriver;
use crate::sync::session::SyncSession;
use crate::{Client, IntoAnyhow};

/// Wallet handle: which network, which database, and how to reach the
/// server. Immutable; a settings change builds a new handle through the
/// `set_*` methods and the caller replaces the one it holds. Opaque to Dart.
///
/// The handle deliberately carries no selected account: every account-scoped
/// operation takes the account explicitly.
#[cfg_attr(feature = "flutter", frb(opaque))]
#[derive(Clone)]
pub struct Coin {
    pub(crate) coin: u8,
    pub(crate) db_filepath: String,
    pub(crate) pool: Option<SqlitePool>,
    /// One sync at a time for this database; shared by every clone of the
    /// handle that serves it.
    pub(crate) sync: Arc<SyncSession>,
    /// The task every write of the chain log outside a sync goes through;
    /// shared like `sync`. None until a database is open.
    pub(crate) driver: Option<SyncDriver>,
    pub(crate) url: String,
    pub(crate) server_type: u8,
    /// Transport: 0 = direct, 1 = Tor (arti), 2 = Nym mixnet,
    /// 3 = external proxy (uses `proxy`).
    pub(crate) transport: u8,
    /// Optional external proxy URL: socks5://, socks5h://, http://, https://.
    /// Empty string means a direct connection.
    pub(crate) proxy: String,
}

impl Coin {
    /// Open the wallet at `db_filepath`. The wallet's network is the one
    /// stored in it; this handle's coin only names the network of a wallet
    /// being created (the app passes the current wallet's when the user
    /// creates one, the server its `--coin` flag).
    pub async fn open_database(
        &self,
        db_filepath: String,
        password: Option<String>,
    ) -> Result<Coin> {
        let hint_coin = if self.coin != 0 {
            Some(self.coin)
        } else {
            None
        };
        let pool = try_open(&db_filepath, &password, hint_coin).await?;

        let mut connection = pool.acquire().await?;

        // Written by `try_open` when it was missing, so always there.
        let coin = crate::db::get_prop(&mut connection, "coin")
            .await?
            .ok_or_else(|| anyhow::anyhow!("the wallet has no network"))?
            .parse::<u8>()?;
        // The migrations encode addresses: for the wallet's network, not
        // the caller's, which is the previous wallet's after a switch.
        let network = Self {
            coin,
            ..self.clone()
        }
        .network();

        migrate_sapling_addresses(&network, &mut connection).await?;
        backfill_diversifier_index(&mut connection).await?;
        crate::account::backfill_transparent_change_addresses(&network, &mut connection).await?;

        // Voting state lives in a sidecar database beside the wallet file, not
        // in this pool, so its schema is created here rather than by
        // `create_schema`.
        crate::voting::sidecar::create_voting_db(PathBuf::from(&db_filepath)).await?;

        drop(connection);
        Ok(Coin {
            coin,
            db_filepath,
            pool: Some(pool),
            sync: Arc::new(SyncSession::new()),
            driver: Some(SyncDriver::start()),
            ..self.clone()
        })
    }

    /// The sync driver, for a write of the chain log outside a sync.
    pub(crate) fn driver(&self) -> Result<&SyncDriver> {
        self.driver
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no wallet is open"))
    }

    /// Network id: 0 mainnet, 1 testnet, 2 regtest, 3 ZSA regtest.
    #[cfg_attr(feature = "flutter", frb(sync, getter))]
    pub fn coin(&self) -> u8 {
        self.coin
    }

    pub fn get_name(&self) -> &'static str {
        match self.coin {
            0 => "mainnet",
            1 => "testnet",
            2 => "regnet",
            3 => "zsa",
            _ => unimplemented!(),
        }
    }

    pub(crate) fn network(&self) -> Network {
        match self.coin {
            0 => Network::Main,
            1 => Network::Test,
            2 => Network::Regtest(LocalNetwork {
                overwinter: Some(BlockHeight::from_u32(1)),
                sapling: Some(BlockHeight::from_u32(1)),
                blossom: Some(BlockHeight::from_u32(1)),
                heartwood: Some(BlockHeight::from_u32(1)),
                canopy: Some(BlockHeight::from_u32(1)),
                nu5: Some(BlockHeight::from_u32(1)),
                nu6: Some(BlockHeight::from_u32(1)),
                nu6_1: Some(BlockHeight::from_u32(1)),
                nu6_2: Some(BlockHeight::from_u32(1)),
                nu6_3: Some(BlockHeight::from_u32(250)),
                nu7: None,
                zsa: None,
                orchard_mode: OrchardMode::Normal,
            }),
            3 => {
                // Deployed ZSA regtest uses its own branch ID, not official NU7.
                // Orchard protocol V2 with cross-address transfers enabled.
                Network::ZsaRegtest(LocalNetwork {
                    overwinter: Some(BlockHeight::from_u32(1)),
                    sapling: Some(BlockHeight::from_u32(1)),
                    blossom: Some(BlockHeight::from_u32(1)),
                    heartwood: Some(BlockHeight::from_u32(1)),
                    canopy: Some(BlockHeight::from_u32(1)),
                    nu5: Some(BlockHeight::from_u32(1)),
                    nu6: Some(BlockHeight::from_u32(1)),
                    nu6_1: Some(BlockHeight::from_u32(1)),
                    nu6_2: Some(BlockHeight::from_u32(1)),
                    nu6_3: None,
                    nu7: None,
                    zsa: Some(BlockHeight::from_u32(1)),
                    orchard_mode: OrchardMode::Zsa,
                })
            }
            _ => Network::Main,
        }
    }

    pub(crate) fn get_pool(&self) -> Result<SqlitePool> {
        self.pool
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Database not opened"))
    }

    /// Closes this handle's database pool when it serves `db_filepath`, so
    /// the file can be deleted, renamed or re-keyed. Every clone of the
    /// handle shares the pool, so they all see it closed.
    pub(crate) async fn close_pool_for(&self, db_filepath: &str) {
        if self.db_filepath == db_filepath {
            if let Some(pool) = &self.pool {
                pool.close().await;
            }
        }
    }

    pub(crate) async fn get_connection(&self) -> Result<PoolConnection<Sqlite>> {
        let pool = self.get_pool()?;
        let start = std::time::Instant::now();
        let result = pool.acquire().await.anyhow();
        let elapsed = start.elapsed();
        if elapsed > std::time::Duration::from_secs(2) {
            tracing::warn!(
                "slow pool acquire took {:?} (size={}, idle={})",
                elapsed,
                pool.size(),
                pool.num_idle()
            );
        }
        result
    }

    #[cfg_attr(feature = "flutter", frb(sync))]
    pub fn set_transport(&self, transport: u8) -> Result<Coin> {
        Ok(Coin {
            transport,
            ..self.clone()
        })
    }

    #[cfg_attr(feature = "flutter", frb(sync))]
    pub fn set_lwd(&self, server_type: u8, url: String) -> Result<Self> {
        Ok(Coin {
            url,
            server_type,
            ..self.clone()
        })
    }

    #[cfg_attr(feature = "flutter", frb(sync))]
    pub fn set_proxy(&self, proxy: String) -> Result<Self> {
        Ok(Coin {
            proxy,
            ..self.clone()
        })
    }

    /// True when traffic to the server goes through the Nym mixnet, either
    /// via a mixnet-native nym:// endpoint or the Nym transport. Always
    /// false when this build has no `nym` feature.
    pub(crate) fn is_mixnet(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "nym")] {
                crate::net::nym_service::parse_nym_url(&self.url).is_some() || self.transport == 2
            } else {
                false
            }
        }
    }

    pub(crate) async fn client(&self) -> Result<Client> {
        #[cfg(feature = "nym")]
        {
            // Mixnet-native endpoint (nym:// URL, a nym-rpc service): bypasses
            // the transport enum entirely — the mixnet IS the transport.
            if let Some(recipient) = crate::net::nym_service::parse_nym_url(&self.url) {
                if self.server_type != 0 {
                    anyhow::bail!("Nym service addresses only support lightwalletd (gRPC) servers");
                }
                let channel = crate::net::nym_service::grpc_channel(recipient).await?;
                let client = CompactTxStreamerClient::new(channel);
                return Ok(Box::new(client) as Client);
            }
        }
        #[cfg(not(feature = "nym"))]
        if self.url.starts_with("nym://") {
            anyhow::bail!("Nym mixnet support is not enabled in this build");
        }

        match self.server_type {
            // lightwalletd (gRPC): transport chosen explicitly by the enum.
            0 => {
                let channel = match self.transport {
                    1 => connect_over_tor(&self.url).await?,
                    #[cfg(feature = "nym")]
                    2 => connect_over_nym(&self.url).await?,
                    #[cfg(not(feature = "nym"))]
                    2 => anyhow::bail!("Nym transport is not enabled in this build"),
                    3 if !self.proxy.is_empty() => {
                        connect_over_proxy(&self.url, &self.proxy).await?
                    }
                    _ => {
                        let mut endpoint =
                            tonic::transport::Channel::from_shared(self.url.clone())?;
                        if self.url.starts_with("https") {
                            let tls = ClientTlsConfig::new().with_enabled_roots();
                            endpoint = endpoint.tls_config(tls)?;
                        }
                        endpoint.connect().await?
                    }
                };
                let client = CompactTxStreamerClient::new(channel);
                Ok(Box::new(client) as Client)
            }

            1 => {
                let client =
                    ZebraClient::new(&self.network(), &self.url, self.transport, &self.proxy)?;
                Ok(Box::new(client) as Client)
            }

            _ => unreachable!(),
        }
    }
}

async fn try_open(
    db_filepath: &str,
    password: &Option<String>,
    hint_coin: Option<u8>,
) -> Result<SqlitePool> {
    // Create a connection pool
    let options = get_connect_options(db_filepath, password);
    let pool = SqlitePoolOptions::new()
        // Voting proofs hold a connection for minutes at a time while the
        // synchronizer, share tracker, and session reads run concurrently;
        // keep headroom so a long proof cannot starve the pool (sqlx's
        // default 30 s acquire timeout would surface as "pool timed out").
        .max_connections(10)
        .idle_timeout(std::time::Duration::from_secs(30))
        .max_lifetime(std::time::Duration::from_secs(60 * 60))
        .connect_with(options)
        .await?;

    let mut connection = pool.acquire().await?;
    create_schema(&mut connection).await?;

    // The stored network is the wallet's for good. It is set once, when
    // the wallet is new: from the hint, else from the file name. A hint
    // that disagrees later is the caller's, not the wallet's (the app
    // passes the previous wallet's coin when switching databases).
    match crate::db::get_prop(&mut connection, "coin").await? {
        Some(stored) => {
            if let Some(hint) = hint_coin {
                if hint.to_string() != stored {
                    tracing::warn!(
                        "{db_filepath}: opened with coin {hint}, but the wallet is coin {stored}; keeping {stored}"
                    );
                }
            }
        }
        None => {
            let coin_value = if let Some(coin) = hint_coin {
                coin.to_string()
            } else {
                let testnet = db_filepath.contains("testnet");
                let zsa = db_filepath.contains("zsa");
                let regtest = db_filepath.contains("regtest");
                if testnet {
                    "1"
                } else if zsa {
                    "3"
                } else if regtest {
                    "2"
                } else {
                    "0"
                }
                .to_string()
            };
            crate::db::put_prop(&mut connection, "coin", &coin_value).await?;
        }
    }

    Ok(pool)
}

#[cfg(test)]
mod network_label_tests {
    use super::Coin;

    fn temp_db(name: &str) -> String {
        let path = std::env::temp_dir().join(format!("{name}_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        path.to_string_lossy().into_owned()
    }

    #[tokio::test]
    async fn the_stored_network_survives_reopening_with_another_hint() {
        let path = temp_db("zkool_label");
        let first = Coin::new(Some(1))
            .open_database(path.clone(), None)
            .await
            .unwrap();
        assert_eq!(first.coin, 1, "a new wallet takes the hint");

        let no_hint = Coin::new(None).open_database(path.clone(), None).await.unwrap();
        assert_eq!(no_hint.coin, 1, "no hint: the stored network, not the file name");

        let other_hint = Coin::new(Some(2)).open_database(path.clone(), None).await.unwrap();
        assert_eq!(other_hint.coin, 1, "a differing hint does not relabel the wallet");
    }

    #[tokio::test]
    async fn a_new_wallet_without_a_hint_is_named_by_its_file() {
        let path = temp_db("zkool_regtest_label");
        let coin = Coin::new(None).open_database(path, None).await.unwrap();
        assert_eq!(coin.coin, 2);
    }
}

async fn build_tor(directory: &str) -> anyhow::Result<TorClient<PreferredRuntime>> {
    let config = TorClientConfigBuilder::from_directories(directory, directory).build()?;
    let tor_client = TorClient::create_bootstrapped(config).await?;
    Ok(tor_client)
}

async fn connect_over_tor(url: &str) -> anyhow::Result<Channel> {
    let uri = url.parse::<Uri>()?;

    let host = uri
        .host()
        .ok_or_else(|| anyhow::anyhow!("no host"))?
        .to_string();
    let port = uri.port_u16().unwrap_or_else(|| {
        if uri.scheme_str() == Some("https") {
            443
        } else {
            80
        }
    });

    let connector = service_fn(move |_dst| {
        let host = host.clone();
        async move {
            let tor_client = get_tor_client().await.lock().await;

            let stream = tor_client
                .connect((host.as_str(), port))
                .await
                .map_err(std::io::Error::other)?;
            // Convert to a type that implements hyper::rt::Read + Write
            let compat_stream = TokioIo::new(stream);
            Ok::<_, anyhow::Error>(compat_stream)
        }
    });

    let mut endpoint = Endpoint::from_shared(url.to_string())?;
    if url.starts_with("https") {
        let tls = ClientTlsConfig::new().with_enabled_roots();
        endpoint = endpoint.tls_config(tls)?;
    }

    Ok(endpoint.connect_with_connector(connector).await?)
}

#[cfg(feature = "nym")]
async fn connect_over_nym(url: &str) -> anyhow::Result<Channel> {
    let uri = url.parse::<Uri>()?;

    let host = uri
        .host()
        .ok_or_else(|| anyhow::anyhow!("no host"))?
        .to_string();
    let port = uri.port_u16().unwrap_or_else(|| {
        if uri.scheme_str() == Some("https") {
            443
        } else {
            80
        }
    });

    let connector = service_fn(move |_dst| {
        let host = host.clone();
        async move {
            // DNS + TCP both go through the mixnet; TLS (with SNI/cert
            // checks against the hostname) runs on top via the endpoint.
            let stream = crate::net::nym::nym_connect(&host, port).await?;
            Ok::<_, anyhow::Error>(TokioIo::new(stream))
        }
    });

    let mut endpoint = Endpoint::from_shared(url.to_string())?;
    if url.starts_with("https") {
        let tls = ClientTlsConfig::new().with_enabled_roots();
        endpoint = endpoint.tls_config(tls)?;
    }

    Ok(endpoint.connect_with_connector(connector).await?)
}

/// Build a tonic Channel to `url` whose TCP connection is established through an
/// external proxy. Supports socks5://, socks5h://, http:// and https:// proxies.
async fn connect_over_proxy(url: &str, proxy: &str) -> anyhow::Result<Channel> {
    let uri = url.parse::<Uri>()?;
    let host = uri
        .host()
        .ok_or_else(|| anyhow::anyhow!("no host"))?
        .to_string();
    let port = uri.port_u16().unwrap_or_else(|| {
        if uri.scheme_str() == Some("https") {
            443
        } else {
            80
        }
    });

    let proxy = proxy.to_string();
    let connector = service_fn(move |_dst| {
        let host = host.clone();
        let proxy = proxy.clone();
        async move {
            let stream = open_proxied_stream(&proxy, &host, port).await?;
            let compat_stream = TokioIo::new(stream);
            Ok::<_, anyhow::Error>(compat_stream)
        }
    });

    let mut endpoint = Endpoint::from_shared(url.to_string())?;
    if url.starts_with("https") {
        let tls = ClientTlsConfig::new().with_enabled_roots();
        endpoint = endpoint.tls_config(tls)?;
    }

    Ok(endpoint.connect_with_connector(connector).await?)
}

/// Open a TCP stream to (`target_host`, `target_port`) through `proxy`.
/// Returns a tokio stream usable as the transport for a single connection.
pub(crate) async fn open_proxied_stream(
    proxy: &str,
    target_host: &str,
    target_port: u16,
) -> anyhow::Result<tokio::net::TcpStream> {
    let puri = proxy.parse::<Uri>()?;
    let scheme = puri.scheme_str().unwrap_or("").to_lowercase();
    let phost = puri
        .host()
        .ok_or_else(|| anyhow::anyhow!("proxy has no host"))?;
    let pport = puri.port_u16().unwrap_or(match scheme.as_str() {
        "socks5" | "socks5h" => 1080,
        "https" => 443,
        _ => 8080,
    });

    // Extract credentials from the authority (user:pass@host:port).
    // http::Uri::host() already strips the userinfo, so phost/pport are fine;
    // we only need the raw authority string to pull out the userinfo prefix.
    let creds: Option<(String, String)> = puri
        .authority()
        .and_then(|a| a.as_str().rfind('@').map(|at| &a.as_str()[..at]))
        .map(|userinfo| {
            let (u, p) = userinfo.split_once(':').unwrap_or((userinfo, ""));
            (u.to_string(), p.to_string())
        });

    match scheme.as_str() {
        // socks5h => resolve the target hostname *at the proxy* (remote DNS).
        // This is what allows .onion addresses to work and prevents DNS leaks,
        // so it is the recommended scheme for Tor.
        "socks5h" => {
            let stream = match &creds {
                Some((u, p)) => {
                    tokio_socks::tcp::Socks5Stream::connect_with_password(
                        (phost, pport),
                        // Passing a &str target makes tokio-socks send the hostname to
                        // the proxy as a SOCKS5 DOMAINNAME request (proxy-side DNS).
                        (target_host, target_port),
                        u.as_str(),
                        p.as_str(),
                    )
                    .await?
                }
                None => {
                    tokio_socks::tcp::Socks5Stream::connect(
                        (phost, pport),
                        (target_host, target_port),
                    )
                    .await?
                }
            };
            Ok(stream.into_inner())
        }
        // socks5 => resolve the target hostname locally and send the IP to the
        // proxy. We resolve here explicitly so the distinction from socks5h is
        // honoured even though tokio-socks would otherwise defer to the proxy.
        "socks5" => {
            let mut addrs = tokio::net::lookup_host((target_host, target_port)).await?;
            let target_addr = addrs
                .next()
                .ok_or_else(|| anyhow::anyhow!("could not resolve {target_host}"))?;
            let stream = match &creds {
                Some((u, p)) => {
                    tokio_socks::tcp::Socks5Stream::connect_with_password(
                        (phost, pport),
                        target_addr,
                        u.as_str(),
                        p.as_str(),
                    )
                    .await?
                }
                None => {
                    tokio_socks::tcp::Socks5Stream::connect((phost, pport), target_addr).await?
                }
            };
            Ok(stream.into_inner())
        }
        "http" | "https" => http_connect_tunnel(phost, pport, target_host, target_port).await,
        other => anyhow::bail!("unsupported proxy scheme: {other}"),
    }
}

/// Establish an HTTP CONNECT tunnel through an http(s) proxy and return the
/// raw TCP stream (post-handshake) ready for TLS/HTTP2 to run over.
async fn http_connect_tunnel(
    proxy_host: &str,
    proxy_port: u16,
    target_host: &str,
    target_port: u16,
) -> anyhow::Result<tokio::net::TcpStream> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut stream = tokio::net::TcpStream::connect((proxy_host, proxy_port)).await?;
    let req = format!(
        "CONNECT {host}:{port} HTTP/1.1\r\nHost: {host}:{port}\r\n\r\n",
        host = target_host,
        port = target_port
    );
    stream.write_all(req.as_bytes()).await?;
    stream.flush().await?;

    // Read until we have the full status line + headers (terminated by \r\n\r\n).
    let mut buf = Vec::with_capacity(256);
    let mut tmp = [0u8; 256];
    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            anyhow::bail!("proxy closed connection during CONNECT");
        }
        buf.extend_from_slice(&tmp[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if buf.len() > 8192 {
            anyhow::bail!("CONNECT response headers too large");
        }
    }

    let head = String::from_utf8_lossy(&buf);
    let status_line = head.lines().next().unwrap_or("");
    // Expect "HTTP/1.1 200 ..." (any 2xx is acceptable).
    let ok = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse::<u16>().ok())
        .map(|c| (200..300).contains(&c))
        .unwrap_or(false);
    if !ok {
        anyhow::bail!("proxy CONNECT failed: {status_line}");
    }
    Ok(stream)
}

impl Coin {
    /// The HTTP route for the selected transport, `timeout` per request.
    pub(crate) fn route(&self, timeout: std::time::Duration) -> Result<crate::net::http::Route> {
        crate::net::http::Route::new(self.transport, &self.proxy, timeout)
    }

    #[cfg_attr(feature = "flutter", frb(sync))]
    pub fn new(default_coin: Option<u8>) -> Self {
        Coin {
            coin: default_coin.unwrap_or(0),
            db_filepath: String::new(),
            pool: None,
            sync: Arc::new(SyncSession::new()),
            driver: None,
            server_type: 0,
            url: String::new(),
            transport: 0,
            proxy: String::new(),
        }
    }
}

fn get_connect_options(db_filepath: &str, password: &Option<String>) -> SqliteConnectOptions {
    let options = SqliteConnectOptions::new()
        .filename(db_filepath)
        .create_if_missing(true);
    let options = match password.as_ref() {
        Some(password) => {
            let escaped_password = format!("'{}'", password.replace('\'', "''"));
            options.pragma("key", escaped_password)
        }
        None => options,
    };
    options
}

pub(crate) use zcash_trees::network::Network;

pub async fn init_datadir(directory: &str) -> Result<()> {
    let _ = DATADIR.set(directory.to_string());

    // Set the Sapling parameters directory relative to the data directory.
    // On mobile platforms (Android, iOS) the HOME env var is not set,
    // which breaks zcash_proofs::default_params_folder().  Doing it
    // unconditionally keeps the path consistent across all platforms.
    {
        let sapling_dir = std::path::PathBuf::from(directory).join(".zcash-params");
        crate::api::sapling::set_sapling_params_dir(sapling_dir);
    }

    Ok(())
}

pub async fn get_tor_client() -> &'static Mutex<TorClient<PreferredRuntime>> {
    let data_dir = {
        let data_dir = DATADIR.get().expect("Data dir should have been set");
        data_dir.clone()
    };
    let tor = TOR
        .get_or_init(|| async {
            let tor_client = build_tor(&data_dir).await.unwrap();
            Mutex::new(tor_client)
        })
        .await;
    tor
}

pub static TOR: OnceCell<Mutex<TorClient<PreferredRuntime>>> = OnceCell::const_new();
pub static DATADIR: OnceLock<String> = OnceLock::new();
