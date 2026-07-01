use std::fs;

use sqlx::Row;

use crate::api::coin::Coin;
use anyhow::Result;
#[cfg(feature="flutter")]
use flutter_rust_bridge::frb;

#[cfg_attr(feature = "flutter", frb)]
pub struct DbAccountPreview {
    pub id: u32,
    pub name: String,
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn list_db_accounts(db_filepath: &str) -> Result<Vec<DbAccountPreview>> {
    if !std::path::Path::new(db_filepath).exists() {
        return Ok(vec![]);
    }

    // Try to connect without a password — encrypted DBs will fail here
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(db_filepath)
        .create_if_missing(false);

    let pool = match sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
    {
        Ok(pool) => pool,
        Err(_) => return Ok(vec![]),
    };

    let mut connection = match pool.acquire().await {
        Ok(c) => c,
        Err(_) => return Ok(vec![]),
    };

    let rows = match sqlx::query(
        "SELECT id_account, name FROM accounts ORDER BY position",
    )
    .fetch_all(&mut *connection)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return Ok(vec![]),
    };

    let accounts = rows
        .iter()
        .map(|row| {
            let id: i64 = row.get(0);
            let name: String = row.get(1);
            DbAccountPreview {
                id: id as u32,
                name,
            }
        })
        .collect();

    Ok(accounts)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn change_db_password(
    db_filepath: &str,
    tmp_dir: &str,
    old_password: &str,
    new_password: &str,
) -> Result<()> {
    crate::api::coin::close_pool(db_filepath);
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
