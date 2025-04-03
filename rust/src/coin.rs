use std::sync::Mutex;

use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use zcash_protocol::consensus::Network;

#[macro_export]
macro_rules! setup {
    ($account: expr) => {
        let mut coin = crate::coin::COIN.lock().unwrap();
        coin.account = $account;
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
        })
    }

    pub fn connect(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        let connection = self.pool.get()?;
        Ok(connection)
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
        }
    }
}

lazy_static::lazy_static! {
    pub static ref COIN: Mutex<Coin> = Mutex::new(Coin::default());
}
