use std::sync::{Arc, Mutex};

use anyhow::Result;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use zcash_protocol::consensus::Network;

#[derive(Clone)]
pub struct Coin {
    pub coin: u8,
    pub account: u32,
    pub network: Network,
    pub db_filepath: Option<String>,
    pub pool: Pool<SqliteConnectionManager>,
    pub connection: Arc<Mutex<Option<PooledConnection<SqliteConnectionManager>>>>,
}

impl Coin {
    pub fn new(coin: u8, network: Network) -> Result<Coin> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder().build(manager)?;
        Ok(Coin {
            coin,
            account: 0,
            network,
            db_filepath: None,
            pool,
            connection: Arc::new(Mutex::new(None)),
        })
    }

    pub fn set_db_filepath(&mut self, db_filepath: String) -> Result<()> {
        let manager = SqliteConnectionManager::file(&db_filepath);
        let pool = Pool::builder().build(manager)?;
        self.db_filepath = Some(db_filepath);
        self.pool = pool;

        Ok(())
    }

    pub fn connect(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        let connection = self.pool.get()?;
        Ok(connection)
    }

    pub fn connection(&mut self) -> Result<Arc<Mutex<Option<PooledConnection<SqliteConnectionManager>>>>> {
        {
            let mut connection = self.connection.lock().unwrap();
            if (*connection).is_none() {
                let c = self.connect()?;
                *connection = Some(c);
            }
        }

        Ok(self.connection.clone())
    }
}

lazy_static::lazy_static! {
    pub static ref COINS: Mutex<[Coin; 2]> = Mutex::new([
        Coin::new(0, Network::MainNetwork).unwrap(),
        Coin::new(1, Network::TestNetwork).unwrap()]);
}
