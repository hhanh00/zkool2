use std::sync::Mutex;

use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use tonic::transport::ClientTlsConfig;
use zcash_protocol::consensus::Network;

use crate::db::create_schema;
use crate::lwd::compact_tx_streamer_client::CompactTxStreamerClient;
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
    pub lwd: String,
}

impl Coin {
    pub async fn new(lwd: &str, db_filepath: &str, password: Option<String>) -> Result<Coin> {
        // Create a connection pool
        let options = get_connect_options(db_filepath, password);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .idle_timeout(std::time::Duration::from_secs(30))
            .max_lifetime(std::time::Duration::from_secs(60 * 60))
            .connect_with(options)
            .await?;

        if sqlx::query("SELECT 1 FROM sqlite_master WHERE type='table' AND name='props'")
            .fetch_optional(&pool)
            .await?
            .is_none()
        {
            create_schema(&pool).await?;
            let testnet = db_filepath.contains("testnet");
            let coin_value = if testnet { "1" } else { "0" };
            crate::db::put_prop(&pool, "coin", coin_value).await?;
        }

        let (coin,): (String,) = sqlx::query_as("SELECT value FROM props WHERE key = 'coin'")
            .fetch_one(&pool)
            .await?;
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
            lwd: lwd.to_string(),
        })
    }

    pub fn get_pool(&self) -> &SqlitePool {
        self.pool.as_ref().unwrap()
    }

    pub fn set_lwd(&mut self, lwd: &str) {
        self.lwd = lwd.to_string();
    }

    pub async fn client(&self) -> Result<Client> {
        let mut channel = tonic::transport::Channel::from_shared(self.lwd.clone())?;
        if self.lwd.starts_with("https") {
            let tls = ClientTlsConfig::new().with_enabled_roots();
            channel = channel.tls_config(tls)?;
        }
        let client = CompactTxStreamerClient::connect(channel).await?;
        Ok(client)
    }
}

impl Default for Coin {
    fn default() -> Self {
        Coin {
            coin: 0,
            account: 0,
            network: Network::MainNetwork,
            db_filepath: String::new(),
            pool: None,
            lwd: String::new(),
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
