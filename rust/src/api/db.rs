use crate::{coin::Coin, get_coin};
use anyhow::Result;
use flutter_rust_bridge::frb;

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

#[frb]
pub async fn get_prop(key: &str) -> Result<Option<String>> {
    let coin = get_coin!();
    crate::db::get_prop(coin.get_pool(), key).await
}

#[frb]
pub async fn put_prop(key: &str, value: &str) -> Result<()> {
    let coin = get_coin!();
    crate::db::put_prop(coin.get_pool(), key, value).await
}
