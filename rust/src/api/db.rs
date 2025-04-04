use crate::{
    coin::Coin,
    db::{create_schema, put_prop},
};
use anyhow::Result;
use sqlx::{
    sqlite::SqliteConnectOptions, SqlitePool}
;

pub async fn create_database(coin: u8, db_filepath: &str) -> Result<()> {
    let options = SqliteConnectOptions::new()
        .filename(db_filepath)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;
    create_schema(&pool).await?;
    put_prop(&pool, "coin", &coin.to_string()).await?;

    Ok(())
}

pub async fn open_database(db_filepath: &str) -> Result<()> {
    let coin = Coin::new(db_filepath).await?;
    let mut c = crate::coin::COIN.lock().unwrap();
    *c = coin;

    Ok(())
}
