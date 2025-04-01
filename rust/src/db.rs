use anyhow::Result;
use rusqlite::{params, Connection};
use zcash_keys::keys::sapling::{DiversifiableFullViewingKey, ExtendedSpendingKey};
use zcash_transparent::keys::{AccountPrivKey, AccountPubKey};

use crate::get_coin;

pub fn drop_schema(connection: &Connection) -> Result<()> {
    connection.execute("DROP TABLE IF EXISTS accounts", [])?;
    connection.execute("DROP TABLE IF EXISTS transparent_accounts", [])?;
    connection.execute("DROP TABLE IF EXISTS transparent_address_accounts", [])?;
    connection.execute("DROP TABLE IF EXISTS sapling_accounts", [])?;
    connection.execute("DROP TABLE IF EXISTS orchard_accounts", [])?;

    Ok(())
}

pub fn create_schema(connection: &Connection) -> Result<()> {
    drop_schema(connection)?;
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
        saved BOOL NOT NULL
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
    let id = connection.query_row(
        "INSERT INTO accounts(name, icon, birth, height,
        aindex, dindex, def_dindex, position, saved, hidden)
        VALUES (?, ?, ?, ?, 0, 0, 0, 0, FALSE, FALSE)
        ON CONFLICT(id_account) DO UPDATE SET
            name = excluded.name
        RETURNING id_account",
        params![name, icon, birth, height],
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

    connection.execute("UPDATE accounts SET dindex = ? WHERE id_account = ?", 
        params![dindex, c.account])?;
    if update_default {
        connection.execute("UPDATE accounts SET def_dindex = ? WHERE id_account = ?", 
            params![dindex, c.account])?;
    }

    Ok(())
}
