use std::collections::HashMap;
use std::sync::{LazyLock, OnceLock};

use anyhow::Result;
use arti_client::config::TorClientConfigBuilder;
use arti_client::TorClient;
use flutter_rust_bridge::frb;
use hyper_util::rt::TokioIo;
use sqlx::pool::PoolConnection;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Sqlite, SqlitePool};
use tokio::sync::{Mutex, OnceCell};
use tonic::transport::{Channel, ClientTlsConfig, Endpoint, Uri};
use tor_rtcompat::PreferredRuntime;
use tower::service_fn;
use zcash_protocol::consensus::{
    BlockHeight, MainNetwork, NetworkType, NetworkUpgrade, Parameters, TestNetwork,
};
use zcash_protocol::local_consensus::LocalNetwork;

use crate::db::{create_schema, migrate_sapling_addresses};
use crate::lwd::compact_tx_streamer_client::CompactTxStreamerClient;
use crate::net::zebra::ZebraClient;
use crate::{Client, IntoAnyhow};


#[frb(dart_metadata = ("freezed"))]
#[derive(Clone)]
pub struct Coin {
    pub coin: u8,
    pub account: u32,
    pub db_filepath: String,
    pub url: String,
    pub server_type: u8,
    pub use_tor: bool,
}

impl Coin {
    pub async fn open_database(
        self,
        db_filepath: String,
        password: Option<String>,
    ) -> Result<Coin> {
        let network = self.network();
        let Coin {
            account,
            url,
            server_type,
            use_tor,
            ..
        } = self;

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

        migrate_sapling_addresses(&network, &mut connection).await?;

        Ok(Coin {
            coin,
            account,
            db_filepath,
            server_type,
            url,
            use_tor,
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

    pub fn network(&self) -> Network {
        match self.coin {
            0 => Network::Main,
            1 => Network::Test,
            2 => REGTEST,
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

    #[frb]
    pub fn set_account(self, account: u32) -> Result<Self> {
        let Coin {
            coin,
            db_filepath,
            url,
            server_type,
            use_tor,
            ..
        } = self;
        Ok(Coin {
            coin,
            account,
            db_filepath,
            url,
            server_type,
            use_tor,
        })
    }

    pub fn set_url(self, server_type: u8, url: String) -> Result<Self> {
        let Coin {
            coin,
            account,
            db_filepath,
            use_tor,
            ..
        } = self;
        Ok(Coin {
            coin,
            account,
            db_filepath,
            url,
            server_type,
            use_tor,
        })
    }

    pub fn set_use_tor(self, use_tor: bool) -> Result<Coin> {
        let Coin {
            coin,
            account,
            db_filepath,
            url,
            server_type,
            ..
        } = self;
        Ok(Coin {
            coin,
            account,
            db_filepath,
            url,
            server_type,
            use_tor,
        })
    }

    #[frb(sync)]
    pub fn set_lwd(self, server_type: u8, url: String) -> Result<Self> {
        let Coin {
            coin,
            account,
            db_filepath,
            use_tor,
            ..
        } = self;
        Ok(Coin {
            coin,
            account,
            db_filepath,
            url,
            server_type,
            use_tor,
        })
    }

    pub(crate) async fn client(&self) -> Result<Client> {
        match self.server_type {
            0 if self.use_tor => {
                let channel = connect_over_tor(&self.url).await?;

                let client = CompactTxStreamerClient::new(channel);
                Ok(Box::new(client))
            }

            0 if !self.use_tor => {
                let mut channel = tonic::transport::Channel::from_shared(self.url.clone())?;
                if self.url.starts_with("https") {
                    let tls = ClientTlsConfig::new().with_enabled_roots();
                    channel = channel.tls_config(tls)?;
                }
                let client = CompactTxStreamerClient::connect(channel).await?;
                Ok(Box::new(client))
            }

            1 => {
                let client = ZebraClient::new(&self.network(), &self.url)?;
                Ok(Box::new(client))
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

impl Coin {
    #[frb(sync)]
    pub fn new() -> Self {
        Coin {
            coin: 0,
            account: 0,
            db_filepath: String::new(),
            server_type: 0,
            url: String::new(),
            use_tor: false,
        }
    }
}

fn get_connect_options(db_filepath: &str, password: &Option<String>) -> SqliteConnectOptions {
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

pub(crate) const fn _regtest() -> LocalNetwork {
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
pub static REGTEST: Network = Network::Regtest(_regtest());
pub static POOLS: LazyLock<std::sync::Mutex<HashMap<String, SqlitePool>>> =
    LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
