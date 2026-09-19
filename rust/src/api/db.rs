use std::fs;

use sqlx::Row;

use crate::api::coin::Coin;
use anyhow::{ensure, Result};
#[cfg(feature = "flutter")]
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

    let rows = match sqlx::query("SELECT id_account, name FROM accounts ORDER BY position")
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

/// Deletes a wallet database and every file that belongs to it.
///
/// The wallet file must exist; everything beside it is optional. The voting
/// sidecar is one of those files, and leaving it behind would strand voting
/// state that a wallet later recreated under the same name would inherit. The
/// WAL companions of both only exist after an unclean shutdown.
#[cfg_attr(feature = "flutter", frb)]
pub async fn delete_db(db_filepath: &str) -> Result<()> {
    // Dropping the pool first: on Windows an open handle blocks the delete,
    // and elsewhere it would leave the pool serving a file that is gone.
    crate::api::coin::close_pool(db_filepath);

    let wallet = std::path::Path::new(db_filepath);
    let sidecar = crate::voting::sidecar::voting_db_path(wallet);

    fs::remove_file(wallet)?;
    let _ = fs::remove_file(&sidecar);
    for base in [wallet, sidecar.as_path()] {
        for suffix in ["-wal", "-shm"] {
            let mut companion = base.as_os_str().to_os_string();
            companion.push(suffix);
            let _ = fs::remove_file(std::path::PathBuf::from(companion));
        }
    }

    Ok(())
}

/// Renames a wallet database and every file that belongs to it.
///
/// Refuses an existing destination: `fs::rename` overwrites silently, and the
/// destination here is another wallet. The voting sidecar moves with the
/// wallet so its state stays attached to the wallet it describes, and the WAL
/// companions move too -- one left behind holds commits the renamed database
/// would no longer see.
#[cfg_attr(feature = "flutter", frb)]
pub async fn rename_db(db_filepath: &str, new_db_filepath: &str) -> Result<()> {
    crate::api::coin::close_pool(db_filepath);

    let old = std::path::Path::new(db_filepath);
    let new = std::path::Path::new(new_db_filepath);
    ensure!(!new.exists(), "a database already exists at {new_db_filepath}");

    fs::rename(old, new)?;

    let old_sidecar = crate::voting::sidecar::voting_db_path(old);
    let new_sidecar = crate::voting::sidecar::voting_db_path(new);
    if old_sidecar.exists() {
        fs::rename(&old_sidecar, &new_sidecar)?;
    }
    for (from, to) in [
        (old.to_path_buf(), new.to_path_buf()),
        (old_sidecar, new_sidecar),
    ] {
        for suffix in ["-wal", "-shm"] {
            let mut from_companion = from.clone().into_os_string();
            from_companion.push(suffix);
            let from_companion = std::path::PathBuf::from(from_companion);
            if from_companion.exists() {
                let mut to_companion = to.clone().into_os_string();
                to_companion.push(suffix);
                let _ = fs::rename(from_companion, std::path::PathBuf::from(to_companion));
            }
        }
    }

    Ok(())
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
