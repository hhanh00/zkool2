use std::fs;

use crate::api::coin::Coin;
use anyhow::Result;
#[cfg(feature="flutter")]
use flutter_rust_bridge::frb;

#[cfg_attr(feature = "flutter", frb)]
pub async fn change_db_password(
    db_filepath: &str,
    tmp_dir: &str,
    old_password: &str,
    new_password: &str,
) -> Result<()> {
    crate::db::change_db_password(db_filepath, tmp_dir, old_password, new_password).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_prop(key: &str, c: &Coin) -> Result<Option<String>> {
    let mut connection = c.get_connection().await?;
    crate::db::get_prop(&mut connection, key).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn put_prop(key: &str, value: &str, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::put_prop(&mut connection, key, value).await
}

#[cfg_attr(feature = "flutter", frb)]
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
