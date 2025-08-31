use std::sync::Mutex;

use anyhow::Result;
use arti_client::config::TorClientConfigBuilder;
use arti_client::TorClient;
use hyper_util::rt::TokioIo;
use sqlx::pool::PoolConnection;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Sqlite, SqlitePool};
use tonic::transport::{Channel, ClientTlsConfig, Endpoint, Uri};
use tor_rtcompat::PreferredRuntime;
use tower::service_fn;
use zcash_protocol::consensus::{
    BlockHeight, MainNetwork, NetworkType, NetworkUpgrade, Parameters, TestNetwork,
};
use zcash_protocol::local_consensus::LocalNetwork;

use crate::db::create_schema;
use crate::lwd::compact_tx_streamer_client::CompactTxStreamerClient;
use crate::zebra::ZebraClient;
use crate::Client;

#[macro_export]
macro_rules! setup {
    ($account: expr) => {{
        let mut coin = $crate::coin::COIN.lock().unwrap();
        coin.account = $account;
    }};
}

#[macro_export]
macro_rules! get_coin {
    () => {{
        let c = $crate::coin::COIN.lock().unwrap();
        c.clone()
    }};
}

#[derive(Clone)]
pub struct Coin {
    pub coin: u8,
    pub account: u32,
    pub network: Network,
    pub db_filepath: String,
    pub pool: Option<SqlitePool>,
    pub url: String,
    pub server_type: ServerType,
    pub use_tor: bool,
}

impl Coin {
    pub async fn new(
        server_type: ServerType,
        url: &str,
        use_tor: bool,
        db_filepath: &str,
        password: Option<String>,
    ) -> Result<Coin> {
        // Create a connection pool
        let options = get_connect_options(db_filepath, password);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .idle_timeout(std::time::Duration::from_secs(30))
            .max_lifetime(std::time::Duration::from_secs(60 * 60))
            .connect_with(options)
            .await?;

        let mut connection = pool.acquire().await?;
        if sqlx::query("SELECT 1 FROM sqlite_master WHERE type='table' AND name='props'")
            .fetch_optional(&mut *connection)
            .await?
            .is_none()
        {
            create_schema(&mut connection).await?;
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

        let coin = crate::db::get_prop(&mut connection, "coin")
            .await?
            .unwrap_or("0".to_string());
        let coin = coin.parse::<u8>()?;

        let network = match coin {
            0 => Network::Main,
            1 => Network::Test,
            2 => *REGTEST,
            _ => Network::Main,
        };

        Ok(Coin {
            coin,
            account: 0,
            network,
            db_filepath: db_filepath.to_string(),
            pool: Some(pool),
            server_type,
            url: url.to_string(),
            use_tor,
        })
    }

    pub fn get_pool(&self) -> &SqlitePool {
        let pool = self.pool.as_ref().expect("Connection pool not initialized");
        pool
    }

    pub async fn get_connection(&self) -> Result<PoolConnection<Sqlite>, sqlx::Error> {
        let pool = self.pool.as_ref().expect("Connection pool not initialized");
        pool.acquire().await
    }

    pub fn set_url(&mut self, server_type: ServerType, url: &str) {
        self.url = url.to_string();
        self.server_type = server_type;
    }

    pub fn set_use_tor(&mut self, use_tor: bool) {
        self.use_tor = use_tor;
    }

    pub async fn client(&self) -> Result<Client> {
        match self.server_type {
            ServerType::Lwd if self.use_tor => {
                let channel = connect_over_tor(&self.url).await?;

                let client = CompactTxStreamerClient::new(channel);
                Ok(Box::new(client))
            }

            ServerType::Lwd if !self.use_tor => {
                let mut channel = tonic::transport::Channel::from_shared(self.url.clone())?;
                if self.url.starts_with("https") {
                    let tls = ClientTlsConfig::new().with_enabled_roots();
                    channel = channel.tls_config(tls)?;
                }
                let client = CompactTxStreamerClient::connect(channel).await?;
                Ok(Box::new(client))
            }

            ServerType::Zebra => {
                let client = ZebraClient::new(&self.network, &self.url);
                Ok(Box::new(client))
            }

            _ => {
                todo!()
            }
        }
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
            let tor_client = TOR.lock().await;
            let tor_client = tor_client
                .as_ref()
                .ok_or(anyhow::anyhow!("TOR Client not started. App needs restart"))?;
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

#[derive(Clone)]
pub enum ServerType {
    Lwd = 0,
    Zebra = 1,
}

impl Default for Coin {
    fn default() -> Self {
        Coin {
            coin: 0,
            account: 0,
            network: Network::Main,
            db_filepath: String::new(),
            pool: None,
            server_type: ServerType::Lwd,
            url: String::new(),
            use_tor: false,
        }
    }
}

fn get_connect_options(db_filepath: &str, password: Option<String>) -> SqliteConnectOptions {
    let options = SqliteConnectOptions::new()
        .filename(db_filepath)
        .create_if_missing(true);
    let options = match password.as_ref() {
        Some(password) => options.pragma("key", password.clone()),
        None => options,
    };
    options
}

#[derive(Copy, Clone, Debug)]
pub enum Network {
    Main,
    Test,
    Regtest(LocalNetwork),
}

impl Parameters for Network {
    fn network_type(&self) -> NetworkType {
        match self {
            Network::Main => MainNetwork.network_type(),
            Network::Test => TestNetwork.network_type(),
            Network::Regtest(n) => n.network_type(),
        }
    }

    fn activation_height(
        &self,
        nu: NetworkUpgrade,
    ) -> Option<zcash_protocol::consensus::BlockHeight> {
        match self {
            Network::Main => MainNetwork.activation_height(nu),
            Network::Test => TestNetwork.activation_height(nu),
            Network::Regtest(n) => n.activation_height(nu),
        }
    }
}

pub fn _regtest() -> LocalNetwork {
    LocalNetwork {
        overwinter: Some(BlockHeight::from_u32(1)),
        sapling: Some(BlockHeight::from_u32(1)),
        blossom: Some(BlockHeight::from_u32(1)),
        heartwood: Some(BlockHeight::from_u32(1)),
        canopy: Some(BlockHeight::from_u32(1)),
        nu5: Some(BlockHeight::from_u32(1)),
        nu6: Some(BlockHeight::from_u32(1)),
        nu6_1: Some(BlockHeight::from_u32(1)),
    }
}

pub async fn init_tor(directory: &str) -> Result<()> {
    let mut t = TOR.lock().await;
    if t.is_none() {
        let tor_client = build_tor(directory).await?;
        *t = Some(tor_client);
    }
    Ok(())
}

lazy_static::lazy_static! {
    pub static ref COIN: Mutex<Coin> = Mutex::new(Coin::default());
    pub static ref TOR: tokio::sync::Mutex<Option<TorClient<PreferredRuntime>>> = tokio::sync::Mutex::new(None);

    pub static ref REGTEST: Network = Network::Regtest(_regtest());
}
