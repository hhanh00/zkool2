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
    old_password: &str,
    new_password: &str,
    c: &Coin,
) -> Result<()> {
    c.close_pool_for(db_filepath).await;
    crate::db::change_db_password(db_filepath, old_password, new_password).await
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
/// wallet itself runs in rollback-journal mode and never has WAL companions;
/// the sidecar runs in WAL mode, so its `-wal`/`-shm` files exist after an
/// unclean shutdown and are removed with it.
#[cfg_attr(feature = "flutter", frb)]
pub async fn delete_db(db_filepath: &str, c: &Coin) -> Result<()> {
    // Closing the pool first: on Windows an open handle blocks the delete,
    // and elsewhere it would leave the pool serving a file that is gone.
    c.close_pool_for(db_filepath).await;

    let wallet = std::path::Path::new(db_filepath);
    fs::remove_file(wallet)?;
    remove_companions(wallet);

    Ok(())
}

/// Replaces a wallet database with `data`, the bytes of a saved database.
///
/// The voting sidecar describes the wallet it was created beside, so it goes
/// with the old contents: keeping it would pair the restored wallet with
/// ballots and share records that are not its own. The bytes land in a
/// temporary file next to the wallet and are renamed over it, so a failed
/// write leaves the old wallet in place rather than a truncated one.
#[cfg_attr(feature = "flutter", frb)]
pub async fn restore_db(db_filepath: &str, data: Vec<u8>, c: &Coin) -> Result<()> {
    c.close_pool_for(db_filepath).await;

    let wallet = std::path::Path::new(db_filepath);
    let mut staged = wallet.as_os_str().to_os_string();
    staged.push(".restore");
    let staged = std::path::PathBuf::from(staged);

    fs::write(&staged, &data)?;
    remove_companions(wallet);
    fs::rename(&staged, wallet)?;

    Ok(())
}

/// Removes the files that belong to a wallet besides the wallet itself: the
/// voting sidecar and its WAL companions. Missing files are fine.
fn remove_companions(wallet: &std::path::Path) {
    let sidecar = crate::voting::sidecar::voting_db_path(wallet);
    let _ = fs::remove_file(&sidecar);
    for suffix in ["-wal", "-shm"] {
        let mut companion = sidecar.as_os_str().to_os_string();
        companion.push(suffix);
        let _ = fs::remove_file(std::path::PathBuf::from(companion));
    }
}

/// Renames a wallet database and every file that belongs to it.
///
/// Refuses an existing destination: `fs::rename` overwrites silently, and the
/// destination here is another wallet. The voting sidecar moves with the
/// wallet so its state stays attached to the wallet it describes, and the WAL
/// companions move too -- one left behind holds commits the renamed database
/// would no longer see.
#[cfg_attr(feature = "flutter", frb)]
pub async fn rename_db(db_filepath: &str, new_db_filepath: &str, c: &Coin) -> Result<()> {
    c.close_pool_for(db_filepath).await;

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

#[cfg(test)]
mod tests {
    use super::{remove_companions, restore_db};
    use crate::api::coin::Coin;

    /// A fresh directory under the system temp dir, removed on drop.
    struct Scratch(std::path::PathBuf);

    impl Scratch {
        fn new(tag: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!(
                "zkool-{tag}-{}-{nanos}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[tokio::test]
    async fn restore_replaces_the_wallet_and_drops_its_sidecar() {
        let scratch = Scratch::new("restore");
        let wallet = scratch.0.join("wallet.db");
        std::fs::write(&wallet, b"old wallet").unwrap();
        let sidecar = crate::voting::sidecar::voting_db_path(&wallet);
        std::fs::write(&sidecar, b"old sidecar").unwrap();
        let mut sidecar_wal = sidecar.as_os_str().to_os_string();
        sidecar_wal.push("-wal");
        let sidecar_wal = std::path::PathBuf::from(sidecar_wal);
        std::fs::write(&sidecar_wal, b"frames").unwrap();

        let coin = Coin::new(None);
        restore_db(wallet.to_str().unwrap(), b"new wallet".to_vec(), &coin)
            .await
            .unwrap();

        assert_eq!(std::fs::read(&wallet).unwrap(), b"new wallet");
        assert!(!sidecar.exists());
        assert!(!sidecar_wal.exists());
        let leftovers: Vec<_> = std::fs::read_dir(&scratch.0)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(leftovers, vec![std::ffi::OsString::from("wallet.db")]);
    }

    #[test]
    fn removing_companions_tolerates_their_absence() {
        let scratch = Scratch::new("companions");
        remove_companions(&scratch.0.join("missing.db"));
    }
}
