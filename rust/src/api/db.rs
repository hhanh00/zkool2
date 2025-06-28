use crate::{coin::Coin, get_coin};
use anyhow::Result;
use flutter_rust_bridge::frb;

pub async fn open_database(db_filepath: &str, password: Option<String>) -> Result<()> {
    let (server_type, lwd) = {
        let c = crate::coin::COIN.lock().unwrap();
        (c.server_type.clone(), c.url.clone())
    };
    let coin = Coin::new(server_type, &lwd, db_filepath, password).await?;
    let mut c = crate::coin::COIN.lock().unwrap();
    *c = coin;

    Ok(())
}

#[frb]
pub async fn change_db_password(
    db_filepath: &str,
    tmp_dir: &str,
    old_password: &str,
    new_password: &str,
) -> Result<()> {
    crate::db::change_db_password(db_filepath, tmp_dir, old_password, new_password).await
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
