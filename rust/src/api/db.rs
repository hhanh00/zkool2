use std::fs;

use crate::{coin::Coin, get_coin};
use anyhow::Result;
use flutter_rust_bridge::frb;

pub async fn open_database(db_filepath: &str, password: Option<String>) -> Result<()> {
    let (server_type, lwd, use_tor) = {
        let c = get_coin!();
        (c.server_type.clone(), c.url.clone(), c.use_tor)
    };
    let coin = Coin::new(server_type, &lwd, use_tor, db_filepath, password).await?;
    tracing::info!("Open DB");

    let mut c = crate::coin::COIN.lock().unwrap();
    *c = coin;
    assert!(c.pool.is_some());

    tracing::info!("/Open DB");

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
    let mut connection = coin.get_connection().await?;
    crate::db::get_prop(&mut connection, key).await
}

#[frb]
pub async fn put_prop(key: &str, value: &str) -> Result<()> {
    let coin = get_coin!();
    let mut connection = coin.get_connection().await?;
    crate::db::put_prop(&mut connection, key, value).await
}

#[frb]
pub async fn list_db_names(dir: &str) -> Result<Vec<String>> {
    let entries = fs::read_dir(dir)?;
    let mut db_names = vec![];

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                let name = path.file_stem().unwrap().display().to_string();
                if ext == "db" {
                    db_names.push(name);
                }
            }
        }
    }

    Ok(db_names)
}
