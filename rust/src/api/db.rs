use crate::{
    coin::Coin,
    db::{create_schema, put_prop},
};
use anyhow::Result;
use sqlx::
    sqlite::SqlitePoolOptions
;

pub async fn create_database(coin: u8, db_filepath: &str) -> Result<()> {
    let pool = SqlitePoolOptions::new().connect(db_filepath).await?;
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
