use anyhow::Result;
use rusqlite::Connection;
use crate::{coin::Coin, db::{create_schema, put_prop}};

pub fn create_database(coin: u8, db_filepath: &str) -> Result<()> {
    let connection = Connection::open(db_filepath)?;
    create_schema(&connection)?;
    put_prop(&connection, "coin", &coin.to_string())?;

    Ok(())
}

pub fn open_database(db_filepath: &str) -> Result<()> {
    let mut c = crate::coin::COIN.lock().unwrap();
    *c = Coin::new(db_filepath)?;

    Ok(())
}