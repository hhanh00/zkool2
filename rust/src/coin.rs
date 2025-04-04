use std::sync::Mutex;

use anyhow::Result;
use sqlx::{pool::PoolConnection, sqlite::SqlitePoolOptions, Pool, Sqlite};
use tonic::transport::{Certificate, ClientTlsConfig};
use zcash_protocol::consensus::Network;

use crate::Client;
use crate::lwd::compact_tx_streamer_client::CompactTxStreamerClient;

#[macro_export]
macro_rules! setup {
    ($account: expr) => {
        {
        let mut coin = crate::coin::COIN.lock().unwrap();
        coin.account = $account;
        }
    };
}

#[macro_export]
macro_rules! get_coin {
    () => {
        {
            let c = crate::coin::COIN.lock().unwrap();
            c.clone()
        }
    };
}

#[derive(Clone)]
pub struct Coin {
    pub coin: u8,
    pub account: u32,
    pub network: Network,
    pub db_filepath: String,
    pub pool: Option<Pool<Sqlite>>,
    pub lwd: String,
}

impl Coin {
    pub async fn new(db_filepath: &str) -> Result<Coin> {
        // Create a connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .idle_timeout(std::time::Duration::from_secs(30))
            .max_lifetime(std::time::Duration::from_secs(60 * 60))
            .connect(db_filepath).await?;

        let (coin, ): (String, ) = sqlx::query_as("SELECT value FROM props WHERE key = 'coin'")
        .fetch_one(&pool).await?;
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
            lwd: String::new(),
        })
    }

    pub fn get_pool(&self) -> &Pool<Sqlite> {
        self.pool.as_ref().unwrap()
    }

    pub async fn connect(&self) -> Result<PoolConnection<Sqlite>> {
        let connection = self.pool.as_ref().unwrap().acquire().await?;
        Ok(connection)
    }

    pub fn set_lwd(&mut self, lwd: &str) {
        self.lwd = lwd.to_string();
    }

    pub async fn client(&self) -> Result<Client> {
        let mut channel = tonic::transport::Channel::from_shared(self.lwd.clone())?;
        if self.lwd.starts_with("https") {
            let pem = include_bytes!("ca.pem");
            let ca = Certificate::from_pem(pem);
            let tls = ClientTlsConfig::new().ca_certificate(ca);
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

lazy_static::lazy_static! {
    pub static ref COIN: Mutex<Coin> = Mutex::new(Coin::default());
}
