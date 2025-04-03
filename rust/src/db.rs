use anyhow::{anyhow, Result};
use orchard::keys::{FullViewingKey, SpendingKey};
use rusqlite::{params, Connection, OptionalExtension};
use zcash_keys::keys::sapling::{DiversifiableFullViewingKey, ExtendedSpendingKey};
use zcash_transparent::keys::{AccountPrivKey, AccountPubKey};

use crate::{api::account::Account, get_coin};

pub fn drop_schema(connection: &Connection) -> Result<()> {
    connection.execute("DROP TABLE IF EXISTS accounts", [])?;
    connection.execute("DROP TABLE IF EXISTS transparent_accounts", [])?;
    connection.execute("DROP TABLE IF EXISTS transparent_address_accounts", [])?;
    connection.execute("DROP TABLE IF EXISTS sapling_accounts", [])?;
    connection.execute("DROP TABLE IF EXISTS orchard_accounts", [])?;

    Ok(())
}

pub fn create_schema(connection: &Connection) -> Result<()> {
    // drop_schema(connection)?;
    connection.execute(
        "CREATE TABLE IF NOT EXISTS accounts(
        id_account INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        seed TEXT,
        aindex INTEGER NOT NULL,
        dindex INTEGER NOT NULL,
        def_dindex INTEGER NOT NULL,
        icon BLOB,
        birth INTEGER NOT NULL,
        height INTEGER NOT NULL,
        position INTEGER NOT NULL,
        hidden BOOL NOT NULL,
        saved BOOL NOT NULL,
        enabled BOOL NOT NULL DEFAULT TRUE
        )",
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS transparent_accounts(
        account INTEGER PRIMARY KEY,
        xsk BLOB,
        xvk BLOB)",
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS transparent_address_accounts(
        account INTEGER NOT NULL,
        scope INTEGER NOT NULL,
        dindex INTEGER NOT NULL,
        pubkey BLOB,
        address TEXT,
        PRIMARY KEY (account, scope, dindex))",
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS sapling_accounts(
        account INTEGER PRIMARY KEY,
        xsk BLOB,
        xvk BLOB NOT NULL)",
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS orchard_accounts(
        account INTEGER PRIMARY KEY,
        xsk BLOB,
        xvk BLOB NOT NULL)",
        [],
    )?;

    Ok(())
}

pub fn store_account_metadata(
    connection: &Connection,
    name: &str,
    icon: Option<Vec<u8>>,
    birth: u32,
    height: u32,
) -> Result<u32> {
    let last_position = connection
        .query_row("SELECT MAX(position) FROM accounts", [], |r| {
            r.get::<_, Option<u32>>(0)
        })?
        .unwrap_or_default();

    let id = connection.query_row(
        "INSERT INTO accounts(name, icon, birth, height,
        aindex, dindex, def_dindex, position, saved, hidden)
        VALUES (?, ?, ?, ?, 0, 0, 0, ?, FALSE, FALSE)
        ON CONFLICT(id_account) DO UPDATE SET
            name = excluded.name
        RETURNING id_account",
        params![name, icon, birth, height, last_position + 1],
        |r| r.get::<_, u32>(0),
    )?;

    Ok(id)
}

macro_rules! get_connection {
    ($c: ident, $connection: ident) => {
        let $connection = $c.connection()?;
        let $connection = $connection.lock().unwrap();
        let $connection = $connection.as_ref().unwrap();
    };
}

macro_rules! get_mut_connection {
    ($c: ident, $connection: ident) => {
        let $connection = $c.connection()?;
        let mut $connection = $connection.lock().unwrap();
        let $connection = $connection.as_mut().unwrap();
    };
}

pub fn store_account_seed(phrase: &str, aindex: u32) -> Result<()> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    connection.execute(
        "UPDATE accounts
         SET seed = ?,
             aindex = ?
         WHERE id_account = ?",
        params![&phrase, aindex, c.account],
    )?;

    Ok(())
}

pub fn init_account_transparent() -> Result<()> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    connection.execute(
        "INSERT INTO transparent_accounts(account) VALUES (?)",
        [c.account],
    )?;

    Ok(())
}

pub fn store_account_transparent_sk(xsk: &AccountPrivKey) -> Result<()> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    connection.execute(
        "UPDATE transparent_accounts
        SET xsk = ? WHERE account = ?",
        params![xsk.to_bytes(), c.account],
    )?;

    Ok(())
}

pub fn store_account_transparent_vk(xvk: &AccountPubKey) -> Result<()> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    connection.execute(
        "UPDATE transparent_accounts
        SET xvk = ? WHERE account = ?",
        params![xvk.serialize(), c.account],
    )?;

    Ok(())
}

pub fn init_account_sapling() -> Result<()> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    connection.execute(
        "INSERT INTO sapling_accounts(account, xvk) VALUES (?, '')",
        [c.account],
    )?;

    Ok(())
}

pub fn store_account_transparent_addr(
    scope: u32,
    dindex: u32,
    pk: &[u8],
    address: &str,
) -> Result<()> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    connection.execute(
        "INSERT INTO transparent_address_accounts(account, scope, dindex, pubkey, address)
        VALUES (?, ?, ?, ?, ?)",
        params![c.account, scope, dindex, pk, address],
    )?;

    Ok(())
}

pub fn store_account_sapling_sk(xsk: &ExtendedSpendingKey) -> Result<()> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    connection.execute(
        "UPDATE sapling_accounts
        SET xsk = ? WHERE account = ?",
        params![xsk.to_bytes(), c.account],
    )?;

    Ok(())
}

pub fn store_account_sapling_vk(xvk: &DiversifiableFullViewingKey) -> Result<()> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    connection.execute(
        "UPDATE sapling_accounts
        SET xvk = ? WHERE account = ?",
        params![xvk.to_bytes(), c.account],
    )?;

    Ok(())
}

pub fn init_account_orchard() -> Result<()> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    connection.execute(
        "INSERT INTO orchard_accounts(account, xvk) VALUES (?, '')",
        [c.account],
    )?;

    Ok(())
}

pub fn store_account_orchard_sk(xsk: &orchard::keys::SpendingKey) -> Result<()> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    connection.execute(
        "UPDATE orchard_accounts
        SET xsk = ? WHERE account = ?",
        params![xsk.to_bytes(), c.account],
    )?;

    Ok(())
}

pub fn store_account_orchard_vk(xvk: &orchard::keys::FullViewingKey) -> Result<()> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    connection.execute(
        "UPDATE orchard_accounts
        SET xvk = ? WHERE account = ?",
        params![xvk.to_bytes(), c.account],
    )?;

    Ok(())
}

pub fn update_dindex(dindex: u32, update_default: bool) -> Result<()> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    connection.execute(
        "UPDATE accounts SET dindex = ? WHERE id_account = ?",
        params![dindex, c.account],
    )?;
    if update_default {
        connection.execute(
            "UPDATE accounts SET def_dindex = ? WHERE id_account = ?",
            params![dindex, c.account],
        )?;
    }

    Ok(())
}

pub fn select_account_transparent() -> Result<TransparentKeys> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    let r = connection
        .query_row(
            "SELECT xsk, xvk FROM transparent_accounts WHERE account = ?",
            [c.account],
            |r| Ok((r.get::<_, Option<Vec<u8>>>(0)?, r.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?;
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

pub fn select_account_sapling() -> Result<SaplingKeys> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    let r = connection
        .query_row(
            "SELECT xsk, xvk FROM sapling_accounts WHERE account = ?",
            [c.account],
            |r| Ok((r.get::<_, Option<Vec<u8>>>(0)?, r.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?;
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

pub fn select_account_orchard() -> Result<OrchardKeys> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    let r = connection
        .query_row(
            "SELECT xsk, xvk FROM orchard_accounts WHERE account = ?",
            [c.account],
            |r| Ok((r.get::<_, Option<Vec<u8>>>(0)?, r.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?;
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

pub fn list_accounts() -> Result<Vec<Account>> {
    let mut c = get_coin!();
    get_connection!(c, connection);

    let mut stmt = connection.prepare(
        "SELECT id_account, name, seed, aindex,
        icon, birth, height, position, hidden, saved, enabled
        FROM accounts ORDER by position",
    )?;
    let accounts = stmt
        .query_map([], |r| {
            Ok(Account {
                coin: c.coin,
                id: r.get(0)?,
                name: r.get(1)?,
                seed: r.get(2)?,
                aindex: r.get(3)?,
                icon: r.get(4)?,
                birth: r.get(5)?,
                height: r.get(6)?,
                position: r.get(7)?,
                hidden: r.get(8)?,
                saved: r.get(9)?,
                enabled: r.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(accounts)
}

pub fn delete_account() -> Result<()> {
    let mut c = get_coin!();
    get_mut_connection!(c, connection);

    let tx = connection.transaction()?;
    tx.execute(
        "DELETE FROM transparent_address_accounts WHERE account = ?",
        [c.account],
    )?;
    tx.execute(
        "DELETE FROM transparent_address_accounts WHERE account = ?",
        [c.account],
    )?;
    tx.execute(
        "DELETE FROM transparent_accounts WHERE account = ?",
        [c.account],
    )?;
    tx.execute(
        "DELETE FROM sapling_accounts WHERE account = ?",
        [c.account],
    )?;
    tx.execute(
        "DELETE FROM orchard_accounts WHERE account = ?",
        [c.account],
    )?;
    tx.execute("DELETE FROM accounts WHERE id_account = ?", [c.account])?;
    tx.commit()?;

    Ok(())
}

pub fn reorder_account(old_position: u32, new_position: u32) -> Result<()> {
    let mut c = get_coin!();
    get_mut_connection!(c, connection);

    let tx = connection.transaction()?;
    let id = tx.query_row("
        SELECT id_account FROM accounts WHERE position = ?", [old_position], 
    |r| r.get::<_, u32>(0))?;
    if old_position < new_position {
        // moving down the list
        // elements between [old, new) lose 1 position
        tx.execute("
            UPDATE accounts
            SET position = position - 1
            WHERE position > ? AND position <= ?",
            params![old_position, new_position],
        )?;
        // update the old item position
    }
    if old_position > new_position {
        // elements between [new, old) gain 1 position
        tx.execute("
            UPDATE accounts
            SET position = position + 1
            WHERE position >= ? AND position < ?",
            params![new_position, old_position],
        )?;
    }
    tx.execute("
        UPDATE accounts
        SET position = ?
        WHERE id_account = ?",
        params![new_position, id],
    )?;

    tx.commit()?;
    Ok(())
}
