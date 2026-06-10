use std::collections::HashMap;
use std::sync::{LazyLock, OnceLock};

use anyhow::Result;
use arti_client::config::TorClientConfigBuilder;
use arti_client::TorClient;
#[cfg(feature="flutter")]
use flutter_rust_bridge::frb;
use hyper_util::rt::TokioIo;
use sqlx::pool::PoolConnection;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Sqlite, SqlitePool};
use tokio::sync::{Mutex, OnceCell};
use tonic::transport::{Channel, ClientTlsConfig, Endpoint, Uri};
use tor_rtcompat::PreferredRuntime;
use tower::service_fn;
use zcash_protocol::consensus::BlockHeight;
use zcash_protocol::local_consensus::LocalNetwork;

use crate::db::{backfill_diversifier_index, create_schema, migrate_sapling_addresses, put_prop};
use crate::lwd::compact_tx_streamer_client::CompactTxStreamerClient;
use crate::net::zebra::ZebraClient;
use crate::{Client, IntoAnyhow};


#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone)]
pub struct Coin {
    pub coin: u8,
    pub account: u32,
    pub db_filepath: String,
    pub url: String,
    pub server_type: u8,
    pub use_tor: bool,
    /// Optional external proxy URL: socks5://, socks5h://, http://, https://.
    /// Empty string means a direct connection.
    pub proxy: String,
}

impl Coin {
    pub async fn open_database(
        self,
        db_filepath: String,
        password: Option<String>,
    ) -> Result<Coin> {
        let network = self.network();

        let pool = try_open(&db_filepath, &password).await?;
        {
            let mut pools = POOLS.lock().unwrap();
            pools.insert(db_filepath.clone(), pool.clone());
        }

        let mut connection = pool.acquire().await?;

        let coin = crate::db::get_prop(&mut connection, "coin")
            .await?
            .unwrap_or("0".to_string());
        let coin = coin.parse::<u8>()?;
        let account = crate::db::get_prop(&mut connection, "account")
            .await?
            .unwrap_or("0".to_string());
        let account = account.parse::<u32>()?;

        migrate_sapling_addresses(&network, &mut connection).await?;
        backfill_diversifier_index(&mut connection).await?;

        Ok(Coin {
            coin,
            db_filepath,
            account,
            ..self
        })
    }

    pub fn get_name(&self) -> &'static str {
        match self.coin {
            0 => "mainnet",
            1 => "testnet",
            2 => "regnet",
            _ => unimplemented!(),
        }
    }

    pub(crate) fn network(&self) -> Network {
        match self.coin {
            0 => Network::Main,
            1 => Network::Test,
            2 => {
                #[cfg(zcash_unstable = "nu7")]
                let nu7 = if self.db_filepath.to_lowercase().contains("zsa") {
                    Some(BlockHeight::from_u32(1))
                } else {
                    None
                };
                Network::Regtest(LocalNetwork {
                    overwinter: Some(BlockHeight::from_u32(1)),
                    sapling: Some(BlockHeight::from_u32(1)),
                    blossom: Some(BlockHeight::from_u32(1)),
                    heartwood: Some(BlockHeight::from_u32(1)),
                    canopy: Some(BlockHeight::from_u32(1)),
                    nu5: Some(BlockHeight::from_u32(1)),
                    nu6: Some(BlockHeight::from_u32(1)),
                    nu6_1: Some(BlockHeight::from_u32(1)),
                    nu6_2: Some(BlockHeight::from_u32(1)),
                    #[cfg(zcash_unstable = "nu7")]
                    nu7,
                })
            }
            _ => Network::Main,
        }
    }

    pub(crate) fn get_pool(&self) -> Result<SqlitePool> {
        let pools = POOLS.lock().unwrap();
        let pool = pools.get(&self.db_filepath).expect("Database not opened");
        Ok(pool.clone())
    }

    pub(crate) async fn get_connection(&self) -> Result<PoolConnection<Sqlite>> {
        let pool = self.get_pool()?;
        pool.acquire().await.anyhow()
    }

    #[cfg_attr(feature = "flutter", frb)]
    pub async fn set_account(self, account: u32) -> Result<Self> {
        let mut conn = self.get_connection().await?;
        put_prop(&mut *conn, "account", &account.to_string()).await?;
        Ok(Coin {
            account,
            ..self
        })
    }

    #[cfg_attr(feature = "flutter", frb)]
    pub fn set_use_tor(self, use_tor: bool) -> Result<Coin> {
        Ok(Coin {
            use_tor,
            ..self
        })
    }

    #[cfg_attr(feature = "flutter", frb(sync))]
    pub fn set_lwd(self, server_type: u8, url: String) -> Result<Self> {
        Ok(Coin {
            url,
            server_type,
            ..self
        })
    }

    #[cfg_attr(feature = "flutter", frb(sync))]
    pub fn set_proxy(self, proxy: String) -> Result<Self> {
        Ok(Coin { proxy, ..self })
    }

    pub(crate) async fn client(&self) -> Result<Client> {
        match self.server_type {
            // lightwalletd (gRPC). Precedence: Tor (arti) > external proxy > direct.
            0 if self.use_tor => {
                let channel = connect_over_tor(&self.url).await?;
                let client = CompactTxStreamerClient::new(channel);
                Ok(Box::new(client) as Client)
            }

            0 if !self.proxy.is_empty() => {
                let channel = connect_over_proxy(&self.url, &self.proxy).await?;
                let client = CompactTxStreamerClient::new(channel);
                Ok(Box::new(client) as Client)
            }

            0 => {
                let mut channel = tonic::transport::Channel::from_shared(self.url.clone())?;
                if self.url.starts_with("https") {
                    let tls = ClientTlsConfig::new().with_enabled_roots();
                    channel = channel.tls_config(tls)?;
                }
                let client = CompactTxStreamerClient::connect(channel).await?;
                Ok(Box::new(client) as Client)
            }

            1 => {
                let client = ZebraClient::new(&self.network(), &self.url, &self.proxy)?;
                Ok(Box::new(client) as Client)
            }

            _ => unreachable!(),
        }
    }
}

async fn try_open(db_filepath: &str, password: &Option<String>) -> Result<SqlitePool> {
    // Create a connection pool
    let options = get_connect_options(db_filepath, password);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .idle_timeout(std::time::Duration::from_secs(30))
        .max_lifetime(std::time::Duration::from_secs(60 * 60))
        .connect_with(options)
        .await?;

    let mut connection = pool.acquire().await?;
    create_schema(&mut connection).await?;
    if sqlx::query("SELECT 1 FROM sqlite_master WHERE type='table' AND name='props'")
        .fetch_optional(&mut *connection)
        .await?
        .is_some()
    {
        let testnet = db_filepath.contains("testnet");
        let regtest = db_filepath.contains("regtest");
        let coin_value = if testnet {
            "1"
        } else if regtest {
            "2"
        } else {
            "0"
        };
        crate::db::put_prop(&mut connection, "coin", coin_value).await?;
    }

    Ok(pool)
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

    match scheme.as_str() {
        // socks5h => resolve the target hostname *at the proxy* (remote DNS).
        // This is what allows .onion addresses to work and prevents DNS leaks,
        // so it is the recommended scheme for Tor.
        "socks5h" => {
            let stream = tokio_socks::tcp::Socks5Stream::connect(
                (phost, pport),
                // Passing a &str target makes tokio-socks send the hostname to
                // the proxy as a SOCKS5 DOMAINNAME request (proxy-side DNS).
                (target_host, target_port),
            )
            .await?;
            Ok(stream.into_inner())
        }
        // socks5 => resolve the target hostname locally and send the IP to the
        // proxy. We resolve here explicitly so the distinction from socks5h is
        // honoured even though tokio-socks would otherwise defer to the proxy.
        "socks5" => {
            let mut addrs =
                tokio::net::lookup_host((target_host, target_port)).await?;
            let target_addr = addrs
                .next()
                .ok_or_else(|| anyhow::anyhow!("could not resolve {target_host}"))?;
            let stream = tokio_socks::tcp::Socks5Stream::connect(
                (phost, pport),
                target_addr,
            )
            .await?;
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
    #[cfg_attr(feature = "flutter", frb(sync))]
    pub fn new() -> Self {
        Coin {
            coin: 0,
            account: 0,
            db_filepath: String::new(),
            server_type: 0,
            url: String::new(),
            use_tor: false,
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
pub static POOLS: LazyLock<std::sync::Mutex<HashMap<String, SqlitePool>>> =
    LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
