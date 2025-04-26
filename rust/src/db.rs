use anyhow::{anyhow, Result};
use futures::TryStreamExt;
use orchard::keys::{FullViewingKey, SpendingKey};
use sqlx::Row as _;
use sqlx::{sqlite::SqliteRow, SqlitePool};
use tracing::info;
use zcash_keys::keys::sapling::{DiversifiableFullViewingKey, ExtendedSpendingKey};
use zcash_transparent::keys::{AccountPrivKey, AccountPubKey};

use crate::account::TxNote;
use crate::api::account::{Account, Memo, Tx};
use crate::api::sync::PoolBalance;

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
        passphrase TEXT NOT NULL DEFAULT '',
        seed_fingerprint BLOB,
        aindex INTEGER NOT NULL,
        dindex INTEGER NOT NULL,
        def_dindex INTEGER NOT NULL,
        icon BLOB,
        birth INTEGER NOT NULL,
        position INTEGER NOT NULL,
        use_internal BOOL NOT NULL,
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
        id_taddress INTEGER PRIMARY KEY,
        account INTEGER NOT NULL,
        scope INTEGER NOT NULL,
        dindex INTEGER NOT NULL,
        sk BLOB,
        pk BLOB NOT NULL,
        address TEXT NOT NULL,
        UNIQUE (account, scope, dindex))",
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
        "CREATE TABLE IF NOT EXISTS headers(
        height INTEGER PRIMARY KEY,
        hash BLOB NOT NULL,
        time INTEGER NOT NULL)",
    )
    .execute(connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notes(
        id_note INTEGER PRIMARY KEY,
        height INTEGER NOT NULL,
        account INTEGER NOT NULL,
        pool INTEGER NOT NULL,
        scope INTEGER,
        nullifier BLOB NOT NULL,
        tx INTEGER NOT NULL,
        value INTEGER NOT NULL,
        cmx BLOB,
        taddress INTEGER,
        position INTEGER,
        diversifier BLOB,
        rcm BLOB,
        rho BLOB,
        locked BOOL NOT NULL DEFAULT FALSE,
        UNIQUE(account, nullifier))",
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
        value INTEGER NOT NULL)",
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
        details BOOL NOT NULL DEFAULT FALSE,
        UNIQUE (account, txid))",
    )
    .execute(connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS witnesses(
        id_witness INTEGER PRIMARY KEY,
        account INTEGER NOT NULL,
        note INTEGER NOT NULL,
        height INTEGER NOT NULL,
        witness BLOB NOT NULL,
        UNIQUE (note, height))",
    )
    .execute(connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS memos(
        id_memo INTEGER PRIMARY KEY,
        account INTEGER NOT NULL,
        height INTEGER NOT NULL,
        tx INTEGER NOT NULL,
        pool INTEGER NOT NULL,
        vout INTEGER NOT NULL,
        note INTEGER,
        memo_text TEXT,
        memo_bytes BLOB NOT NULL,
        UNIQUE (tx, pool, vout))",
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
    fingerprint: &Option<Vec<u8>>,
    birth: u32,
    use_internal: bool,
) -> Result<u32> {
    let (last_position,): (u32,) = sqlx::query_as("SELECT MAX(position) FROM accounts")
        .fetch_optional(connection)
        .await?
        .unwrap_or_default();

    let (id,): (u32,) = sqlx::query_as(
        "INSERT INTO accounts(name, icon, seed_fingerprint, birth,
        aindex, dindex, def_dindex, position, use_internal, saved, hidden)
        VALUES (?, ?, ?, ?, 0, 0, 0, ?, ?, FALSE, FALSE)
        ON CONFLICT(id_account) DO UPDATE SET
            name = excluded.name
        RETURNING id_account",
    )
    .bind(name)
    .bind(icon)
    .bind(fingerprint)
    .bind(birth)
    .bind(last_position + 1)
    .bind(use_internal)
    .fetch_one(connection)
    .await?;

    Ok(id)
}

pub async fn store_synced_height(connection: &SqlitePool, account: u32, pool: u8, height: u32) -> Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO sync_heights(account, pool, height)
        VALUES (?, ?, ?)",
    )
    .bind(account)
    .bind(pool)
    .bind(height)
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn store_account_seed(
    connection: &SqlitePool,
    account: u32,
    phrase: &str,
    passphrase: &str,
    fingerprint: &[u8],
    aindex: u32,
) -> Result<()> {
    sqlx::query(
        "UPDATE accounts
         SET seed = ?,
             passphrase = ?,
             seed_fingerprint = ?,
             aindex = ?
         WHERE id_account = ?",
    )
    .bind(phrase)
    .bind(passphrase)
    .bind(fingerprint)
    .bind(aindex)
    .bind(account)
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn init_account_transparent(connection: &SqlitePool, account: u32, birth: u32) -> Result<()> {
    sqlx::query("INSERT INTO transparent_accounts(account) VALUES (?)")
        .bind(account)
        .execute(connection)
        .await?;
    store_synced_height(connection, account, 0, birth - 1)
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

pub async fn store_account_transparent_addr(
    connection: &SqlitePool,
    account: u32,
    scope: u32,
    dindex: u32,
    sk: Option<Vec<u8>>,
    pk: &[u8],
    address: &str,
) -> Result<bool> {
    let r = sqlx::query(
        "INSERT INTO transparent_address_accounts(account, scope, dindex, sk, pk, address)
        VALUES (?, ?, ?, ?, ?, ?) ON CONFLICT DO NOTHING",
    )
    .bind(account)
    .bind(scope)
    .bind(dindex)
    .bind(sk)
    .bind(pk)
    .bind(address)
    .execute(connection)
    .await?;

    Ok(r.rows_affected() > 0)
}

pub async fn init_account_sapling(connection: &SqlitePool, account: u32, birth: u32) -> Result<()> {
    sqlx::query("INSERT INTO sapling_accounts(account, xvk) VALUES (?, '')")
        .bind(account)
        .execute(connection)
        .await?;
    store_synced_height(connection, account, 1, birth - 1)
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

pub async fn init_account_orchard(connection: &SqlitePool, account: u32, birth: u32) -> Result<()> {
    sqlx::query("INSERT INTO orchard_accounts(account, xvk) VALUES (?, '')")
        .bind(account)
        .execute(connection)
        .await?;
    store_synced_height(connection, account, 2, birth - 1)
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
    let r: Option<(Option<Vec<u8>>, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT xsk, xvk FROM transparent_accounts WHERE account = ?")
            .bind(account)
            .fetch_optional(connection)
            .await?;

    let (xsk, xvk, taddress) = match r {
        Some((None, None)) => {
            // no xprv, no xpub => get the address imported as bip38
            let taddress =
                sqlx::query("SELECT address FROM transparent_address_accounts WHERE account = ?")
                    .bind(account)
                    .map(|row: SqliteRow| row.get::<String, _>(0))
                    .fetch_one(connection)
                    .await?;
            (None, None, Some(taddress))
        }
        Some((xsk, xvk)) => (xsk, xvk, None),
        None => (None, None, None),
    };

    let keys = TransparentKeys {
        xsk: xsk.map(|xsk| AccountPrivKey::from_bytes(&xsk).unwrap()),
        xvk: xvk.map(|xvk| AccountPubKey::deserialize(&xvk.try_into().unwrap()).unwrap()),
        address: taddress,
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
    pub address: Option<String>,
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
        "WITH sh AS (SELECT account, MIN(height) AS height FROM sync_heights GROUP BY account),
        unspent AS (SELECT a.*
                FROM notes a
                LEFT JOIN spends b ON a.id_note = b.id_note
                WHERE b.id_note IS NULL)
        SELECT id_account, name, seed, aindex,
        icon, birth, a.position, hidden, saved, enabled,
        sh.height, COALESCE(SUM(unspent.value), 0) AS balance
        FROM accounts a
        JOIN sh ON a.id_account = sh.account
        LEFT JOIN unspent ON a.id_account = unspent.account
        GROUP BY id_account
        ORDER by a.position",
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
        height: row.get(10),
        balance: row.get::<i64, _>(11) as u64,
    })
    .fetch(connection);

    let mut accounts = vec![];
    while let Some(row) = rows.try_next().await? {
        accounts.push(row);
    }

    Ok(accounts)
}


pub async fn get_account_fingerprint(connection: &SqlitePool, account: u32) -> Result<Option<Vec<u8>>> {
    let (fingerprint, ): (Option<Vec<u8>>, ) = sqlx::query_as(
        "SELECT seed_fingerprint FROM accounts WHERE id_account = ?",
    ).bind(account).fetch_one(connection).await?;

    Ok(fingerprint)
}

pub async fn delete_account(connection: &SqlitePool, account: u32) -> Result<()> {
    let mut tx = connection.begin().await?;
    sqlx::query("DELETE FROM memos WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM witnesses WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM notes WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM spends WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM transactions WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM sync_heights WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM transparent_accounts WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM transparent_address_accounts WHERE account = ?")
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
    info!(
        "Reordering account from {} to {}",
        old_position, new_position
    );
    let mut tx = connection.begin().await?;
    let (id,): (u32,) = sqlx::query_as("SELECT id_account FROM accounts WHERE position = ?")
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

pub async fn calculate_balance(pool: &SqlitePool, account: u32) -> Result<PoolBalance> {
    let mut balance = PoolBalance(vec![0, 0, 0]);

    let mut rows = sqlx::query("
    WITH N AS (SELECT value, pool FROM notes WHERE account = ?1 UNION ALL SELECT value, pool FROM spends WHERE account = ?1)
    SELECT pool, SUM(value) FROM N GROUP BY pool")
        .bind(account)
        .map(|row: SqliteRow| (row.get::<u8, _>(0), row.get::<i64, _>(1)))
        .fetch(pool);
    while let Some((pool, value)) = rows.try_next().await? {
        balance.0[pool as usize] += value as u64;
    }

    Ok(balance)
}

pub async fn fetch_txs(connection: &SqlitePool, account: u32) -> Result<Vec<Tx>> {
    // union notes and spends, then sum value by tx into v to get tx value
    // join transactions with v by id_tx and filter by account
    // order by height desc to get latest transactions first
    let transactions = sqlx::query(
        "WITH v AS (WITH n AS (SELECT value, tx FROM notes UNION ALL SELECT value, tx FROM spends)
            SELECT tx, SUM(value) AS value FROM n
            GROUP BY tx)
            SELECT id_tx, txid, height, time, v.value FROM transactions t
            JOIN v ON t.id_tx = v.tx
            WHERE account = ?
            ORDER BY height DESC",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let id: u32 = row.get(0);
        let txid: Vec<u8> = row.get(1);
        let height: u32 = row.get(2);
        let time: u32 = row.get(3);
        let value: i64 = row.get(4);
        Tx {
            id,
            txid,
            height,
            time,
            value,
        }
    })
    .fetch_all(connection)
    .await?;
    Ok(transactions)
}

pub async fn fetch_memos(pool: &SqlitePool, account: u32) -> Result<Vec<Memo>> {
    let memos = sqlx::query(
        "SELECT id_memo, m.height, tx, pool, vout, note, t.time, memo_text, memo_bytes
        FROM memos m JOIN transactions t ON m.tx = t.id_tx
        WHERE m.account = ? ORDER BY m.height DESC",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let id: u32 = row.get(0);
        let height: u32 = row.get(1);
        let tx: u32 = row.get(2);
        let pool: u8 = row.get(3);
        let vout: u32 = row.get(4);
        let note: Option<u32> = row.get(5);
        let time: u32 = row.get(6);
        let memo_text: Option<String> = row.get(7);
        let memo_bytes: Vec<u8> = row.get(8);
        Memo {
            id,
            id_tx: tx,
            id_note: note,
            height,
            pool,
            vout,
            time,
            memo: memo_text,
            memo_bytes,
        }
    })
    .fetch_all(pool)
    .await?;

    Ok(memos)
}

pub async fn get_account_dindex(connection: &SqlitePool, account: u32) -> Result<u32> {
    let (dindex,): (u32,) = sqlx::query_as("SELECT dindex FROM accounts WHERE id_account = ?")
        .bind(account)
        .fetch_one(connection)
        .await?;
    Ok(dindex)
}

pub async fn get_notes(connection: &SqlitePool, account: u32) -> Result<Vec<TxNote>> {
    let notes = sqlx::query(
        "SELECT n.id_note, n.height, n.pool, n.value, n.locked
       FROM notes n LEFT JOIN spends s
	   ON n.id_note = s.id_note
	   WHERE n.account = ? AND s.id_note IS NULL ORDER BY n.height DESC",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let id_note: u32 = row.get(0);
        let height: u32 = row.get(1);
        let pool: u8 = row.get(2);
        let value: u64 = row.get(3);
        let locked: bool = row.get(4);

        TxNote {
            id: id_note,
            height,
            pool,
            value,
            locked,
        }
    })
    .fetch_all(connection)
    .await?;

    Ok(notes)
}

pub async fn lock_note(connection: &SqlitePool, account: u32, id: u32, locked: bool) -> Result<()> {
    sqlx::query("UPDATE notes SET locked = ? WHERE account = ? AND id_note = ?")
        .bind(locked)
        .bind(account)
        .bind(id)
        .execute(connection)
        .await?;
    Ok(())
}
