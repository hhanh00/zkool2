use crate::{
    coin::Coin,
    db::{create_schema, put_prop},
};
use anyhow::Result;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

pub(crate) fn get_connect_options(db_filepath: &str, password: Option<String>) -> SqliteConnectOptions {
    let options = SqliteConnectOptions::new()
        .filename(db_filepath)
        .create_if_missing(true);
    let options = match password.as_ref() {
        Some(password) => options.pragma("key", password.clone()),
        None => options,
    };
    options
}

pub async fn create_database(coin: u8, db_filepath: &str, password: Option<String>) -> Result<()> {
    let options = get_connect_options(db_filepath, password);
    let pool = SqlitePool::connect_with(options).await?;
    create_schema(&pool).await?;
    put_prop(&pool, "coin", &coin.to_string()).await?;

    Ok(())
}

pub async fn open_database(db_filepath: &str, password: Option<String>) -> Result<()> {
    let lwd = {
        let c = crate::coin::COIN.lock().unwrap();
        c.lwd.clone()
    };
    let coin = Coin::new(&lwd, db_filepath, password).await?;
    let mut c = crate::coin::COIN.lock().unwrap();
    *c = coin;

    Ok(())
}
