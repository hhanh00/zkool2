use anyhow::{anyhow, Result};
use futures::TryStreamExt;
use orchard::keys::{FullViewingKey, SpendingKey};
use sqlx::Row as _;
use sqlx::{sqlite::SqliteRow, SqlitePool};
use zcash_keys::keys::sapling::{DiversifiableFullViewingKey, ExtendedSpendingKey};
use zcash_transparent::keys::{AccountPrivKey, AccountPubKey};

use crate::api::account::Account;

pub async fn drop_schema(connection: &SqlitePool) -> Result<()> {
    sqlx::query("DROP TABLE IF EXISTS accounts")
        .execute(connection)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS transparent_accounts")
        .execute(connection)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS transparent_address_accounts")
        .execute(connection)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS sapling_accounts")
        .execute(connection)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS orchard_accounts")
        .execute(connection)
        .await?;

    Ok(())
}

pub async fn create_schema(connection: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS props(
        key TEXT PRIMARY KEY,
        VALUE TEXT NOT NULL)",
    )
    .execute(connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS accounts(
        id_account INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        seed TEXT,
        aindex INTEGER NOT NULL,
        dindex INTEGER NOT NULL,
        def_dindex INTEGER NOT NULL,
        icon BLOB,
        birth INTEGER NOT NULL,
        position INTEGER NOT NULL,
        hidden BOOL NOT NULL,
        saved BOOL NOT NULL,
        enabled BOOL NOT NULL DEFAULT TRUE
        )",
    )
    .execute(connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS transparent_accounts(
        account INTEGER PRIMARY KEY,
        xsk BLOB,
        xvk BLOB)",
    )
    .execute(connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS transparent_address_accounts(
        account INTEGER NOT NULL,
        scope INTEGER NOT NULL,
        dindex INTEGER NOT NULL,
        sk BLOB,
        address TEXT,
        PRIMARY KEY (account, scope, dindex))",
    )
    .execute(connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sapling_accounts(
        account INTEGER PRIMARY KEY,
        xsk BLOB,
        xvk BLOB NOT NULL)",
    )
    .execute(connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS orchard_accounts(
        account INTEGER PRIMARY KEY,
        xsk BLOB,
        xvk BLOB NOT NULL)",
    )
    .execute(connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sync_heights(
        account INTEGER,
        pool INTEGER NOT NULL,
        height INTEGER NOT NULL,
        PRIMARY KEY (account, pool))",
    )
    .execute(connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notes(
        id_note INTEGER PRIMARY KEY,
        height INTEGER NOT NULL,
        account INTEGER NOT NULL,
        pool INTEGER NOT NULL,
        nullifier BLOB NOT NULL,
        tx INTEGER NOT NULL,
        value INTEGER NOT NULL,
        cmx BLOB NOT NULL,
        position INTEGER,
        diversifier BLOB,
        rcm BLOB,
        rho BLOB,
        locked BOOL NOT NULL DEFAULT FALSE,
        UNIQUE(account, nullifier))"
    )
    .execute(connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS spends(
        id_note INTEGER PRIMARY KEY,
        height INTEGER NOT NULL,
        account INTEGER NOT NULL,
        pool INTEGER NOT NULL,
        tx INTEGER NOT NULL,
        value INTEGER NOT NULL)"
    )
    .execute(connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS transactions(
        id_tx INTEGER PRIMARY KEY,
        txid BLOB NOT NULL,
        height INTEGER NOT NULL,
        account INTEGER NOT NULL,
        time INTEGER,
        value INTEGER NOT NULL)"
    )
    .execute(connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS witnesses(
        id_witness INTEGER PRIMARY KEY,
        note INTEGER NOT NULL,
        height INTEGER NOT NULL,
        witness BLOB NOT NULL)"
    )
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn put_prop(connection: &SqlitePool, key: &str, value: &str) -> Result<()> {
    sqlx::query("INSERT OR REPLACE INTO props(key, value) VALUES (?, ?)")
        .bind(key)
        .bind(value)
        .execute(connection)
        .await?;

    Ok(())
}

pub async fn get_prop(connection: &SqlitePool, key: &str) -> Result<Option<String>> {
    let value: Option<(String,)> = sqlx::query_as("SELECT value FROM props WHERE key = ?")
        .bind(key)
        .fetch_optional(connection)
        .await?;

    Ok(value.map(|v| v.0))
}

pub async fn store_account_metadata(
    connection: &SqlitePool,
    name: &str,
    icon: &Option<Vec<u8>>,
    birth: u32,
) -> Result<u32> {
    let (last_position,): (u32,) = sqlx::query_as("SELECT MAX(position) FROM accounts")
        .fetch_optional(connection)
        .await?
        .unwrap_or_default();

    let (id,): (u32,) = sqlx::query_as(
        "INSERT INTO accounts(name, icon, birth,
        aindex, dindex, def_dindex, position, saved, hidden)
        VALUES (?, ?, ?, 0, 0, 0, ?, FALSE, FALSE)
        ON CONFLICT(id_account) DO UPDATE SET
            name = excluded.name
        RETURNING id_account",
    )
    .bind(name)
    .bind(icon)
    .bind(birth)
    .bind(last_position + 1)
    .fetch_one(connection)
    .await?;

    for pool in 0..3 {
        sqlx::query(
            "INSERT OR REPLACE INTO sync_heights(account, pool, height)
            VALUES (?, ?, ?)",
        )
        .bind(id)
        .bind(pool)
        .bind(birth - 1)
        .execute(connection)
        .await?;
    }

    Ok(id)
}

pub async fn store_account_seed(
    connection: &SqlitePool,
    account: u32,
    phrase: &str,
    aindex: u32,
) -> Result<()> {
    sqlx::query(
        "UPDATE accounts
         SET seed = ?,
             aindex = ?
         WHERE id_account = ?",
    )
    .bind(phrase)
    .bind(aindex)
    .bind(account)
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn init_account_transparent(connection: &SqlitePool, account: u32) -> Result<()> {
    sqlx::query("INSERT INTO transparent_accounts(account) VALUES (?)")
        .bind(account)
        .execute(connection)
        .await?;

    Ok(())
}

pub async fn store_account_transparent_sk(
    connection: &SqlitePool,
    account: u32,
    xsk: &AccountPrivKey,
) -> Result<()> {
    sqlx::query(
        "UPDATE transparent_accounts
        SET xsk = ? WHERE account = ?",
    )
    .bind(xsk.to_bytes())
    .bind(account)
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn store_account_transparent_vk(
    connection: &SqlitePool,
    account: u32,
    xvk: &AccountPubKey,
) -> Result<()> {
    sqlx::query(
        "UPDATE transparent_accounts
        SET xvk = ? WHERE account = ?",
    )
    .bind(xvk.serialize())
    .bind(account)
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn init_account_sapling(connection: &SqlitePool, account: u32) -> Result<()> {
    sqlx::query("INSERT INTO sapling_accounts(account, xvk) VALUES (?, '')")
        .bind(account)
        .execute(connection)
        .await?;

    Ok(())
}

pub async fn store_account_transparent_addr(
    connection: &SqlitePool,
    account: u32,
    scope: u32,
    dindex: u32,
    sk: &[u8],
    address: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO transparent_address_accounts(account, scope, dindex, sk, address)
        VALUES (?, ?, ?, ?, ?)",
    )
    .bind(account)
    .bind(scope)
    .bind(dindex)
    .bind(sk)
    .bind(address)
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn store_account_sapling_sk(
    connection: &SqlitePool,
    account: u32,
    xsk: &ExtendedSpendingKey,
) -> Result<()> {
    sqlx::query(
        "UPDATE sapling_accounts
        SET xsk = ? WHERE account = ?",
    )
    .bind(xsk.to_bytes().as_slice())
    .bind(account)
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn store_account_sapling_vk(
    connection: &SqlitePool,
    account: u32,
    xvk: &DiversifiableFullViewingKey,
) -> Result<()> {
    sqlx::query(
        "UPDATE sapling_accounts
        SET xvk = ? WHERE account = ?",
    )
    .bind(xvk.to_bytes().as_slice())
    .bind(account)
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn init_account_orchard(connection: &SqlitePool, account: u32) -> Result<()> {
    sqlx::query("INSERT INTO orchard_accounts(account, xvk) VALUES (?, '')")
        .bind(account)
        .execute(connection)
        .await?;

    Ok(())
}

pub async fn store_account_orchard_sk(
    connection: &SqlitePool,
    account: u32,
    xsk: &orchard::keys::SpendingKey,
) -> Result<()> {
    sqlx::query(
        "UPDATE orchard_accounts
        SET xsk = ? WHERE account = ?",
    )
    .bind(xsk.to_bytes().as_slice())
    .bind(account)
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn store_account_orchard_vk(
    connection: &SqlitePool,
    account: u32,
    xvk: &orchard::keys::FullViewingKey,
) -> Result<()> {
    sqlx::query(
        "UPDATE orchard_accounts
        SET xvk = ? WHERE account = ?",
    )
    .bind(xvk.to_bytes().as_slice())
    .bind(account)
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn update_dindex(
    connection: &SqlitePool,
    account: u32,
    dindex: u32,
    update_default: bool,
) -> Result<()> {
    sqlx::query("UPDATE accounts SET dindex = ? WHERE id_account = ?")
        .bind(dindex)
        .bind(account)
        .execute(connection)
        .await?;
    if update_default {
        sqlx::query("UPDATE accounts SET def_dindex = ? WHERE id_account = ?")
            .bind(dindex)
            .bind(account)
            .execute(connection)
            .await?;
    }

    Ok(())
}

pub async fn select_account_transparent(
    connection: &SqlitePool,
    account: u32,
) -> Result<TransparentKeys> {
    let r: Option<(Option<Vec<u8>>, Vec<u8>)> =
        sqlx::query_as("SELECT xsk, xvk FROM transparent_accounts WHERE account = ?")
            .bind(account)
            .fetch_optional(connection)
            .await?;

    let (xsk, xvk) = match r {
        Some((xsk, xvk)) => (xsk, Some(xvk)),
        None => (None, None),
    };

    let keys = TransparentKeys {
        xsk: xsk.map(|xsk| AccountPrivKey::from_bytes(&xsk).unwrap()),
        xvk: xvk.map(|xvk| AccountPubKey::deserialize(&xvk.try_into().unwrap()).unwrap()),
    };

    Ok(keys)
}

pub async fn select_account_sapling(connection: &SqlitePool, account: u32) -> Result<SaplingKeys> {
    let r: Option<(Option<Vec<u8>>, Vec<u8>)> =
        sqlx::query_as("SELECT xsk, xvk FROM sapling_accounts WHERE account = ?")
            .bind(account)
            .fetch_optional(connection)
            .await?;

    let (xsk, xvk) = match r {
        Some((xsk, xvk)) => (xsk, Some(xvk)),
        None => (None, None),
    };

    let keys = SaplingKeys {
        xsk: xsk.map(|xsk| {
            ExtendedSpendingKey::from_bytes(&xsk)
                .map_err(|_| anyhow!("Invalid sdk"))
                .unwrap()
        }),
        xvk: xvk
            .map(|xvk| DiversifiableFullViewingKey::from_bytes(&xvk.try_into().unwrap()).unwrap()),
    };

    Ok(keys)
}

pub async fn select_account_orchard(connection: &SqlitePool, account: u32) -> Result<OrchardKeys> {
    let r: Option<(Option<Vec<u8>>, Vec<u8>)> =
        sqlx::query_as("SELECT xsk, xvk FROM orchard_accounts WHERE account = ?")
            .bind(account)
            .fetch_optional(connection)
            .await?;

    let (xsk, xvk) = match r {
        Some((xsk, xvk)) => (xsk, Some(xvk)),
        None => (None, None),
    };

    let keys = OrchardKeys {
        xsk: xsk.map(|xsk| SpendingKey::from_bytes(xsk.try_into().unwrap()).unwrap()),
        xvk: xvk.map(|xvk| FullViewingKey::from_bytes(&xvk.try_into().unwrap()).unwrap()),
    };

    Ok(keys)
}

pub struct TransparentKeys {
    pub xsk: Option<AccountPrivKey>,
    pub xvk: Option<AccountPubKey>,
}

pub struct SaplingKeys {
    pub xsk: Option<ExtendedSpendingKey>,
    pub xvk: Option<DiversifiableFullViewingKey>,
}

pub struct OrchardKeys {
    pub xsk: Option<SpendingKey>,
    pub xvk: Option<FullViewingKey>,
}

pub async fn list_accounts(connection: &SqlitePool, coin: u8) -> Result<Vec<Account>> {
    let mut rows = sqlx::query(
        "SELECT id_account, name, seed, aindex,
        icon, birth, position, hidden, saved, enabled
        FROM accounts ORDER by position",
    )
    .map(|row: SqliteRow| Account {
        coin,
        id: row.get(0),
        name: row.get(1),
        seed: row.get(2),
        aindex: row.get(3),
        icon: row.get(4),
        birth: row.get(5),
        position: row.get(6),
        hidden: row.get(7),
        saved: row.get(8),
        enabled: row.get(9),
    })
    .fetch(connection);

    let mut accounts = vec![];
    while let Some(row) = rows.try_next().await? {
        accounts.push(row);
    }

    Ok(accounts)
}

pub async fn delete_account(connection: &SqlitePool, account: u32) -> Result<()> {
    let mut tx = connection.begin().await?;
    sqlx::query("DELETE FROM transparent_address_accounts WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM transparent_accounts WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM sapling_accounts WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM orchard_accounts WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM accounts WHERE id_account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(())
}

pub async fn reorder_account(
    connection: &SqlitePool,
    old_position: u32,
    new_position: u32,
) -> Result<()> {
    let mut tx = connection.begin().await?;
    let (id, ): (u32, ) = sqlx::query_as("SELECT id_account FROM accounts WHERE position = ?")
        .bind(old_position)
        .fetch_one(&mut *tx)
        .await?;
    if old_position < new_position {
        sqlx::query(
            "UPDATE accounts
            SET position = position - 1
            WHERE position > ? AND position <= ?",
        )
        .bind(old_position)
        .bind(new_position)
        .execute(&mut *tx)
        .await?;
    }
    if old_position > new_position {
        sqlx::query(
            "UPDATE accounts
            SET position = position + 1
            WHERE position >= ? AND position < ?",
        )
        .bind(new_position)
        .bind(old_position)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(
        "UPDATE accounts
        SET position = ?
        WHERE id_account = ?",
    )
    .bind(new_position)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
