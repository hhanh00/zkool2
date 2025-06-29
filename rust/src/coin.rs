use std::sync::Mutex;

use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use tonic::transport::ClientTlsConfig;
use zcash_protocol::consensus::Network;

use crate::db::create_schema;
use crate::lwd::compact_tx_streamer_client::CompactTxStreamerClient;
use crate::zebra::ZebraClient;
use crate::Client;

#[macro_export]
macro_rules! setup {
    ($account: expr) => {{
        let mut coin = crate::coin::COIN.lock().unwrap();
        coin.account = $account;
    }};
}

#[macro_export]
macro_rules! get_coin {
    () => {{
        let c = crate::coin::COIN.lock().unwrap();
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
}

impl Coin {
    pub async fn new(
        server_type: ServerType,
        url: &str,
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
            create_schema(&mut *connection).await?;
            let testnet = db_filepath.contains("testnet");
            let coin_value = if testnet { "1" } else { "0" };
            crate::db::put_prop(&mut *connection, "coin", coin_value).await?;
        }

        let coin = crate::db::get_prop(&mut *connection, "coin")
            .await?
            .unwrap_or("0".to_string());
        let coin = coin.parse::<u8>()?;

        let network = match coin {
            0 => Network::MainNetwork,
            1 => Network::TestNetwork,
            _ => Network::MainNetwork,
        };

        Ok(Coin {
            coin,
            account: 0,
            network,
            db_filepath: db_filepath.to_string(),
            pool: Some(pool),
            server_type,
            url: url.to_string(),
        })
    }

    pub fn get_pool(&self) -> &SqlitePool {
        self.pool.as_ref().unwrap()
    }

    pub fn set_url(&mut self, server_type: ServerType, url: &str) {
        self.url = url.to_string();
        self.server_type = server_type;
    }

    pub async fn client(&self) -> Result<Client> {
        match self.server_type {
            ServerType::Lwd => {
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
        }
    }
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
            network: Network::MainNetwork,
            db_filepath: String::new(),
            pool: None,
            server_type: ServerType::Lwd,
            url: String::new(),
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

lazy_static::lazy_static! {
    pub static ref COIN: Mutex<Coin> = Mutex::new(Coin::default());
}
