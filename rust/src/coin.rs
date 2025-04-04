use std::sync::Mutex;

use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
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
    pub pool: Pool<SqliteConnectionManager>,
    pub lwd: String,
}

impl Coin {
    pub fn new(db_filepath: &str) -> Result<Coin> {
        let manager = SqliteConnectionManager::file(db_filepath);
        let pool = Pool::builder().build(manager)?;
        let connection = pool.get()?;
        let coin = crate::db::get_prop(&connection, "coin")?
            .map(|c| c.parse::<u8>().unwrap()).unwrap_or_default();

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
            pool,
            lwd: String::new(),
        })
    }

    pub fn connect(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        let connection = self.pool.get()?;
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
        let pool = Pool::builder().build(SqliteConnectionManager::memory()).unwrap();
        
        Coin {
            coin: 0,
            account: 0,
            network: Network::MainNetwork,
            db_filepath: String::new(),
            pool,
            lwd: String::new(),
        }
    }
}

lazy_static::lazy_static! {
    pub static ref COIN: Mutex<Coin> = Mutex::new(Coin::default());
}
