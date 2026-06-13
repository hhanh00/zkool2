use std::fs::File;

use anyhow::{anyhow, Result};
use csv_async::AsyncWriter;
use futures::TryStreamExt;
use orchard::keys::{FullViewingKey, SpendingKey};
use sapling_crypto::PaymentAddress;
use crate::keys::{SaplingDiversifiedAddress, ScopeExt};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteRow},
    Column, Connection, Row, SqliteConnection, TypeInfo,
};
use tracing::info;
use zcash_keys::{
    encoding::AddressCodec,
    keys::sapling::{DiversifiableFullViewingKey, ExtendedSpendingKey},
};
use zcash_protocol::consensus::NetworkUpgrade;
use zcash_protocol::consensus::Parameters;
use zcash_transparent::keys::{AccountPrivKey, AccountPubKey};

use crate::api::account::Folder;
use crate::api::account::TAddressTxCount;
use crate::api::account::{Account, Memo, Tx};
use crate::api::coin::Network;
use crate::api::sync::PoolBalance;
use crate::sync::BlockHeader;
use crate::{api::account::TxNote, tiu};

pub const DB_VERSION: u16 = 8;

pub async fn create_schema(connection: &mut SqliteConnection) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS props(
        key TEXT PRIMARY KEY,
        VALUE TEXT NOT NULL)",
    )
    .execute(&mut *connection)
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
        enabled BOOL NOT NULL DEFAULT TRUE,
        internal BOOL NOT NULL DEFAULT FALSE
        )",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS transparent_accounts(
        account INTEGER PRIMARY KEY,
        xsk BLOB,
        xvk BLOB)",
    )
    .execute(&mut *connection)
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
    .execute(&mut *connection)
    .await?;

    let _ =
        sqlx::query("ALTER TABLE transparent_address_accounts ADD COLUMN uncompressed BOOL NOT NULL DEFAULT FALSE")
            .execute(&mut *connection)
            .await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sapling_accounts(
        account INTEGER PRIMARY KEY,
        xsk BLOB,
        xvk BLOB NOT NULL)",
    )
    .execute(&mut *connection)
    .await?;

    let _ =
        sqlx::query("ALTER TABLE sapling_accounts ADD COLUMN address BLOB NOT NULL DEFAULT('')")
            .execute(&mut *connection)
            .await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS orchard_accounts(
        account INTEGER PRIMARY KEY,
        xsk BLOB,
        xvk BLOB NOT NULL)",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sync_heights(
        account INTEGER,
        pool INTEGER NOT NULL,
        height INTEGER NOT NULL,
        PRIMARY KEY (account, pool))",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS headers(
        height INTEGER PRIMARY KEY,
        hash BLOB NOT NULL,
        time INTEGER NOT NULL)",
    )
    .execute(&mut *connection)
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
        id_asset INTEGER,
        diversifier_index INTEGER,
        UNIQUE(account, nullifier))",
    )
    .execute(&mut *connection)
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
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS transactions(
        id_tx INTEGER PRIMARY KEY,
        txid BLOB NOT NULL,
        height INTEGER NOT NULL,
        account INTEGER NOT NULL,
        time INTEGER,
        details BOOL NOT NULL DEFAULT FALSE,
        tpe INTEGER,
        value INTEGER NOT NULL DEFAULT 0,
        fee INTEGER NOT NULL DEFAULT 0,
        UNIQUE (account, txid))",
    )
    .execute(&mut *connection)
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
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS outputs (
        id_output INTEGER PRIMARY KEY,
        account INTEGER NOT NULL,
        height INTEGER NOT NULL,
        tx INTEGER NOT NULL,
        pool INTEGER NOT NULL,
        vout INTEGER NOT NULL,
        value INTEGER NOT NULL,
        address TEXT NOT NULL,
        UNIQUE (tx, pool, vout))",
    )
    .execute(&mut *connection)
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
        output INTEGER,
        memo_text TEXT,
        memo_bytes BLOB NOT NULL,
        UNIQUE (tx, pool, vout))",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS assets(
        id_asset INTEGER PRIMARY KEY,
        asset_desc_hash BLOB NOT NULL,
        ik BLOB NOT NULL,
        asset_base BLOB NOT NULL,
        finalized BOOL NOT NULL DEFAULT FALSE,
        first_seen_height INTEGER NOT NULL,
        UNIQUE (asset_desc_hash, ik))",
    )
    .execute(&mut *connection)
    .await?;

    // Migration: add id_asset to notes for ZSA note→asset linking
    let _ = sqlx::query(
        "ALTER TABLE notes ADD COLUMN id_asset INTEGER REFERENCES assets(id_asset)",
    )
    .execute(&mut *connection)
    .await;

    // Migration: add diversifier_index to notes for per-address shielded tx counts
    let _ = sqlx::query(
        "ALTER TABLE notes ADD COLUMN diversifier_index INTEGER",
    )
    .execute(&mut *connection)
    .await;

    // Migration: add asset_name to assets for human-readable naming
    let _ = sqlx::query(
        "ALTER TABLE assets ADD COLUMN asset_name TEXT",
    )
    .execute(&mut *connection)
    .await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS dkg_params (
        account INTEGER PRIMARY KEY,
        id INTEGER NOT NULL,
        n INTEGER NOT NULL,
        t INTEGER NOT NULL,
        seed TEXT NOT NULL,
        birth_height INTEGER NOT NULL,
        name TEXT NOT NULL DEFAULT('')
    )",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS dkg_packages (
        id_dkg_package INTEGER PRIMARY KEY,
        account INTEGER NOT NULL,
        public BOOL NOT NULL,
        round INTEGER NOT NULL,
        from_id INTEGER NOT NULL,
        data BLOB NOT NULL,
        UNIQUE (account, public, round, from_id)
    )",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS dkg_addresses (
        account INTEGER NOT NULL,
        from_id INTEGER NOT NULL,
        address TEXT NOT NULL,
        PRIMARY KEY (account, from_id)
    )",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS dkg_state (
        account INTEGER PRIMARY KEY,
        spkg1 BLOB,
        spkg2 BLOB,
        key_pkg BLOB
    )",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS dkg_peers (
        account INTEGER NOT NULL,
        round INTEGER NOT NULL,
        from_id INTEGER NOT NULL,
        data BLOB NOT NULL,
        PRIMARY KEY (account, round, from_id)
    )",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS frost_signatures (
        id_signature INTEGER PRIMARY KEY,
        account INTEGER NOT NULL,
        sighash BLOB NOT NULL,
        idx INTEGER NOT NULL,
        nonce BLOB NOT NULL,
        sigpackage BLOB,
        randomizer BLOB,
        sigshare BLOB,
        signature BLOB,
        UNIQUE (account, sighash, idx))",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS frost_commitments (
        id_nonce INTEGER PRIMARY KEY,
        account INTEGER NOT NULL,
        sighash BLOB NOT NULL,
        idx INTEGER NOT NULL,
        from_id INTEGER NOT NULL,
        commitment BLOB NOT NULL,
        sigshare BLOB,
        UNIQUE (account, sighash, idx, from_id))",
    )
    .execute(&mut *connection)
    .await?;

    // V5
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN folder INTEGER")
        .execute(&mut *connection)
        .await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS folders (
        id_folder INTEGER PRIMARY KEY,
        name TEXT NOT NULL)",
    )
    .execute(&mut *connection)
    .await?;

    let _ = sqlx::query("ALTER TABLE transactions ADD COLUMN category INTEGER")
        .execute(&mut *connection)
        .await;
    let _ = sqlx::query("ALTER TABLE transactions ADD COLUMN price REAL")
        .execute(&mut *connection)
        .await;
    let _ = sqlx::query("ALTER TABLE transactions ADD COLUMN zsa_value INTEGER NOT NULL DEFAULT 0")
        .execute(&mut *connection)
        .await;
    let _ = sqlx::query("ALTER TABLE transactions ADD COLUMN asset_id INTEGER REFERENCES assets(id_asset)")
        .execute(&mut *connection)
        .await;
    if sqlx::query("SELECT 1 FROM sqlite_master WHERE type='table' AND name='categories'")
        .fetch_optional(&mut *connection)
        .await?
        .is_none()
    {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS categories (
                id_category INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                income BOOL NOT NULL,
                UNIQUE (name))",
        )
        .execute(&mut *connection)
        .await?;

        for (c, i) in vec![
            ("Salary", true),
            ("Investment Income/Mining", true),
            ("Rental/Property Income", true),
            ("Other Income", true),
            ("Housing & Utilities", false),
            ("Food & Groceries", false),
            ("Restaurants & Coffee", false),
            ("Transportation & Hotels", false),
            ("Health & Insurance", false),
            ("Debt & Financial Obligations", false),
            ("Education & Training", false),
            ("Entertainment & Lifestyle", false),
            ("Personal & Family Care", false),
            ("Savings & Investments", false),
            ("Other Expenses", false),
        ] {
            sqlx::query(
                "INSERT OR REPLACE INTO categories(name, income)
            VALUES (?, ?)",
            )
            .bind(c)
            .bind(i)
            .execute(&mut *connection)
            .await?;
        }
    }

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS pending_txs (
        id_pending_tx INTEGER PRIMARY KEY,
        account INTEGER NOT NULL,
        txid BLOB NOT NULL,
        height INTEGER NOT NULL,
        price REAL,
        category INTEGER,
        UNIQUE (account, txid))",
    )
    .execute(&mut *connection)
    .await?;

    // V7
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN hw INTEGER NOT NULL DEFAULT(0)")
        .execute(&mut *connection)
        .await;
    let _ = sqlx::query("ALTER TABLE dkg_params ADD COLUMN name TEXT NOT NULL DEFAULT('')")
        .execute(&mut *connection)
        .await;

    // V8 — signing key for FROST message authentication
    let _ = sqlx::query("ALTER TABLE dkg_state ADD COLUMN signing_keypair BLOB")
        .execute(&mut *connection)
        .await;

    // V9 — migrate dkg_packages into dkg_addresses / dkg_state / dkg_peers
    // dkg_packages round=0, public=1  → dkg_addresses
    sqlx::query(
        "INSERT OR IGNORE INTO dkg_addresses (account, from_id, address)
        SELECT account, from_id, CAST(data AS TEXT)
        FROM dkg_packages WHERE round = 0 AND public = 1",
    )
    .execute(&mut *connection)
    .await?;

    // dkg_packages round=1/2/3, public=0 → dkg_state columns
    // Insert a stub row for any account that has secrets, then fill each column.
    sqlx::query(
        "INSERT OR IGNORE INTO dkg_state (account)
        SELECT DISTINCT account FROM dkg_packages WHERE public = 0 AND round IN (1, 2, 3)",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "UPDATE dkg_state SET spkg1 = (
            SELECT data FROM dkg_packages
            WHERE dkg_packages.account = dkg_state.account AND round = 1 AND public = 0
        ) WHERE spkg1 IS NULL",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "UPDATE dkg_state SET spkg2 = (
            SELECT data FROM dkg_packages
            WHERE dkg_packages.account = dkg_state.account AND round = 2 AND public = 0
        ) WHERE spkg2 IS NULL",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "UPDATE dkg_state SET key_pkg = (
            SELECT data FROM dkg_packages
            WHERE dkg_packages.account = dkg_state.account AND round = 3 AND public = 0
        ) WHERE key_pkg IS NULL",
    )
    .execute(&mut *connection)
    .await?;

    // dkg_packages round=1/2/3, public=1 → dkg_peers
    sqlx::query(
        "INSERT OR IGNORE INTO dkg_peers (account, round, from_id, data)
        SELECT account, round, from_id, data
        FROM dkg_packages WHERE public = 1 AND round IN (1, 2, 3)",
    )
    .execute(&mut *connection)
    .await?;

    let version = get_prop(connection, "version").await?;
    match version {
        Some(version) if version.parse::<u16>()? > DB_VERSION => {
            anyhow::bail!("This app version only supports up to db version {DB_VERSION}");
        }
        _ => {
            put_prop(connection, "version", &DB_VERSION.to_string()).await?;
        }
    }

    Ok(())
}

pub async fn migrate_sapling_addresses(
    network: &Network,
    connection: &mut SqliteConnection,
) -> Result<()> {
    let accounts: Vec<(u32, u32, Vec<u8>)> = sqlx::query_as(
        "SELECT id_account, dindex, xvk FROM accounts a
            JOIN sapling_accounts s ON a.id_account = s.account
            WHERE address = ''",
    )
    .fetch_all(&mut *connection)
    .await?;

    for (account, dindex, xvk) in accounts {
        let fvk: [u8; 128] = tiu!(xvk);
        let fvk = DiversifiableFullViewingKey::from_bytes(&fvk).unwrap();
        let address = fvk.address((dindex as u64).into()).unwrap();
        let address = address.encode(network);
        sqlx::query("UPDATE sapling_accounts SET address = ?2 WHERE account = ?1")
            .bind(account)
            .bind(&address)
            .execute(&mut *connection)
            .await?;
    }
    Ok(())
}

/// Resolve a Sapling diversifier index from raw diversifier bytes and the DFVK.
/// Returns None if the diversifier was not derived from this viewing key.
pub fn resolve_sapling_diversifier_index(
    dfvk: &DiversifiableFullViewingKey,
    scope: u8,
    diversifier: &[u8],
) -> Option<i64> {
    let d = sapling_crypto::keys::Diversifier(diversifier.try_into().ok()?);
    let address = dfvk.diversified_address_for_scope(scope, d)?;
    dfvk.decrypt_diversifier(&address)
        .and_then(|(di, _)| di.try_into().ok())
        .map(|d: u64| d as i64)
}

/// Resolve an Orchard diversifier index from raw diversifier bytes and the FVK.
/// Returns None if the diversifier was not derived from this viewing key.
pub fn resolve_orchard_diversifier_index(
    fvk: &FullViewingKey,
    scope: u8,
    diversifier: &[u8],
) -> Option<i64> {
    let d = orchard::keys::Diversifier::from_bytes(diversifier.try_into().ok()?);
    let scope = scope.orchard_scope();
    let address = fvk.address(d, scope);
    fvk.to_ivk(scope)
        .diversifier_index(&address)
        .and_then(|di| di.try_into().ok())
        .map(|d: u64| d as i64)
}

pub async fn backfill_diversifier_index(connection: &mut SqliteConnection) -> Result<()> {
    // Skip if backfill was already completed
    if get_prop(connection, "backfilled_diversifier_index")
        .await?
        .is_some()
    {
        return Ok(());
    }

    // Check if any notes need backfilling
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notes WHERE pool IN (1, 2) AND diversifier IS NOT NULL AND diversifier_index IS NULL",
    )
    .fetch_one(&mut *connection)
    .await?;

    if count.0 == 0 {
        // Nothing to backfill, mark as done
        put_prop(connection, "backfilled_diversifier_index", "1").await?;
        return Ok(());
    }

    info!("Backfilling diversifier_index for {} notes", count.0);

    // Fetch distinct accounts with unbackfilled notes
    let accounts: Vec<(u32,)> = sqlx::query_as(
        "SELECT DISTINCT account FROM notes WHERE pool IN (1, 2) AND diversifier IS NOT NULL AND diversifier_index IS NULL",
    )
    .fetch_all(&mut *connection)
    .await?;

    for (account,) in accounts {
        // Load Sapling DFVK for this account
        let sapling_dfvk: Option<DiversifiableFullViewingKey> = sqlx::query_as(
            "SELECT xvk FROM sapling_accounts WHERE account = ?",
        )
        .bind(account)
        .fetch_optional(&mut *connection)
        .await?
        .map(|(xvk,): (Vec<u8>,)| {
            DiversifiableFullViewingKey::from_bytes(&xvk.try_into().unwrap()).unwrap()
        });

        // Load Orchard FVK for this account
        let orchard_fvk: Option<FullViewingKey> = sqlx::query_as(
            "SELECT xvk FROM orchard_accounts WHERE account = ?",
        )
        .bind(account)
        .fetch_optional(&mut *connection)
        .await?
        .map(|(xvk,): (Vec<u8>,)| {
            FullViewingKey::from_bytes(&xvk.try_into().unwrap()).unwrap()
        });

        // Fetch unbackfilled notes for this account
        let notes: Vec<(u32, u8, u8, Vec<u8>)> = sqlx::query_as(
            "SELECT id_note, pool, scope, diversifier FROM notes WHERE account = ? AND pool IN (1, 2) AND diversifier IS NOT NULL AND diversifier_index IS NULL",
        )
        .bind(account)
        .fetch_all(&mut *connection)
        .await?;

        for (id_note, pool, scope, diversifier) in notes {
            let di: Option<i64> = match pool {
                1 => sapling_dfvk
                    .as_ref()
                    .and_then(|dfvk| resolve_sapling_diversifier_index(dfvk, scope, &diversifier)),
                2 => orchard_fvk
                    .as_ref()
                    .and_then(|fvk| resolve_orchard_diversifier_index(fvk, scope, &diversifier)),
                _ => None,
            };

            if let Some(di) = di {
                sqlx::query("UPDATE notes SET diversifier_index = ? WHERE id_note = ?")
                    .bind(di)
                    .bind(id_note)
                    .execute(&mut *connection)
                    .await?;
            }
        }
    }

    info!("Backfill diversifier_index complete");
    put_prop(connection, "backfilled_diversifier_index", "1").await?;
    Ok(())
}

pub async fn put_prop(connection: &mut SqliteConnection, key: &str, value: &str) -> Result<()> {
    sqlx::query("INSERT OR REPLACE INTO props(key, value) VALUES (?, ?)")
        .bind(key)
        .bind(value)
        .execute(&mut *connection)
        .await?;

    Ok(())
}

pub async fn get_prop(connection: &mut SqliteConnection, key: &str) -> Result<Option<String>> {
    let value: Option<(String,)> = sqlx::query_as("SELECT value FROM props WHERE key = ?")
        .bind(key)
        .fetch_optional(&mut *connection)
        .await?;

    Ok(value.map(|v| v.0))
}

pub async fn store_account_metadata(
    connection: &mut SqliteConnection,
    name: &str,
    icon: &Option<Vec<u8>>,
    fingerprint: &Option<Vec<u8>>,
    birth: u32,
    use_internal: bool,
    internal: bool,
) -> Result<u32> {
    let (last_position,): (u32,) = sqlx::query_as("SELECT MAX(position) FROM accounts")
        .fetch_optional(&mut *connection)
        .await?
        .unwrap_or_default();

    let (id,): (u32,) = sqlx::query_as(
        "INSERT INTO accounts(name, icon, seed_fingerprint, birth,
        aindex, dindex, def_dindex, position, use_internal, saved, hidden, internal)
        VALUES (?, ?, ?, ?, 0, 0, 0, ?, ?, FALSE, FALSE, ?)
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
    .bind(internal)
    .fetch_one(&mut *connection)
    .await?;

    Ok(id)
}

pub async fn store_block_header(
    connection: &mut SqliteConnection,
    block_header: &BlockHeader,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO headers (height, hash, time)
                    VALUES (?, ?, ?) ON CONFLICT DO NOTHING",
    )
    .bind(block_header.height)
    .bind(&block_header.hash)
    .bind(block_header.time)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

pub async fn store_synced_height(
    connection: &mut SqliteConnection,
    account: u32,
    pool: u8,
    height: u32,
) -> Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO sync_heights(account, pool, height)
        VALUES (?, ?, ?)",
    )
    .bind(account)
    .bind(pool)
    .bind(height)
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn store_account_seed(
    connection: &mut SqliteConnection,
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
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn init_account_transparent(
    connection: &mut SqliteConnection,
    account: u32,
    birth: u32,
) -> Result<()> {
    sqlx::query("INSERT INTO transparent_accounts(account) VALUES (?)")
        .bind(account)
        .execute(&mut *connection)
        .await?;
    store_synced_height(connection, account, 0, birth).await?;

    Ok(())
}

pub const LEDGER_CODE: u32 = 1;

pub async fn store_account_hw(
    connection: &mut SqliteConnection,
    account: u32,
    hw_code: u32,
    aindex: u32,
) -> Result<()> {
    sqlx::query("UPDATE accounts SET hw = ?2, aindex = ?3 WHERE id_account = ?1")
        .bind(account)
        .bind(hw_code)
        .bind(aindex)
        .execute(connection)
        .await?;
    Ok(())
}

pub async fn store_account_transparent_sk(
    connection: &mut SqliteConnection,
    account: u32,
    xsk: &AccountPrivKey,
) -> Result<()> {
    sqlx::query(
        "UPDATE transparent_accounts
        SET xsk = ? WHERE account = ?",
    )
    .bind(xsk.to_bytes())
    .bind(account)
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn store_account_transparent_vk(
    connection: &mut SqliteConnection,
    account: u32,
    xvk: &AccountPubKey,
) -> Result<()> {
    sqlx::query(
        "UPDATE transparent_accounts
        SET xvk = ? WHERE account = ?",
    )
    .bind(xvk.serialize())
    .bind(account)
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn store_account_transparent_addr(
    connection: &mut SqliteConnection,
    account: u32,
    scope: u32,
    dindex: u32,
    sk: Option<Vec<u8>>,
    pk: &[u8],
    address: &str,
    uncompressed: bool,
) -> Result<bool> {
    let r = sqlx::query(
        "INSERT INTO transparent_address_accounts(account, scope, dindex, sk, pk, address, uncompressed)
        VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT DO NOTHING",
    )
    .bind(account)
    .bind(scope)
    .bind(dindex)
    .bind(sk)
    .bind(pk)
    .bind(address)
    .bind(uncompressed)
    .execute(&mut *connection)
    .await?;

    Ok(r.rows_affected() > 0)
}

pub async fn init_account_sapling(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    birth: u32,
) -> Result<()> {
    sqlx::query("INSERT INTO sapling_accounts(account, xvk) VALUES (?, '')")
        .bind(account)
        .execute(&mut *connection)
        .await?;
    let activation_height: u32 = network
        .activation_height(NetworkUpgrade::Sapling)
        .unwrap()
        .into();
    store_synced_height(connection, account, 1, birth.max(activation_height)).await?;

    Ok(())
}

pub async fn store_account_sapling_sk(
    connection: &mut SqliteConnection,
    account: u32,
    xsk: &ExtendedSpendingKey,
) -> Result<()> {
    sqlx::query(
        "UPDATE sapling_accounts
        SET xsk = ? WHERE account = ?",
    )
    .bind(xsk.to_bytes().as_slice())
    .bind(account)
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn store_account_sapling_vk(
    connection: &mut SqliteConnection,
    account: u32,
    xvk: &DiversifiableFullViewingKey,
    address: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE sapling_accounts
        SET xvk = ?2, address = ?3 WHERE account = ?1",
    )
    .bind(account)
    .bind(xvk.to_bytes().as_slice())
    .bind(address)
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn init_account_orchard(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    birth: u32,
) -> Result<()> {
    sqlx::query("INSERT INTO orchard_accounts(account, xvk) VALUES (?, '')")
        .bind(account)
        .execute(&mut *connection)
        .await?;
    let activation_height = network
        .activation_height(NetworkUpgrade::Nu5)
        .unwrap()
        .into();
    store_synced_height(connection, account, 2, birth.max(activation_height)).await?;

    Ok(())
}

pub async fn store_account_orchard_sk(
    connection: &mut SqliteConnection,
    account: u32,
    xsk: &orchard::keys::SpendingKey,
) -> Result<()> {
    sqlx::query(
        "UPDATE orchard_accounts
        SET xsk = ? WHERE account = ?",
    )
    .bind(xsk.to_bytes().as_slice())
    .bind(account)
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn store_account_orchard_vk(
    connection: &mut SqliteConnection,
    account: u32,
    xvk: &orchard::keys::FullViewingKey,
) -> Result<()> {
    sqlx::query(
        "UPDATE orchard_accounts
        SET xvk = ? WHERE account = ?",
    )
    .bind(xvk.to_bytes().as_slice())
    .bind(account)
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn update_dindex(
    connection: &mut SqliteConnection,
    account: u32,
    dindex: u32,
    update_default: bool,
) -> Result<()> {
    sqlx::query("UPDATE accounts SET dindex = ? WHERE id_account = ?")
        .bind(dindex)
        .bind(account)
        .execute(&mut *connection)
        .await?;
    if update_default {
        sqlx::query("UPDATE accounts SET def_dindex = ? WHERE id_account = ?")
            .bind(dindex)
            .bind(account)
            .execute(&mut *connection)
            .await?;
    }

    Ok(())
}

pub async fn select_account_transparent(
    connection: &mut SqliteConnection,
    account: u32,
    dindex: u32,
) -> Result<TransparentKeys> {
    #[allow(clippy::type_complexity)]
    let r: Option<(Option<Vec<u8>>, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT xsk, xvk FROM transparent_accounts WHERE account = ?")
            .bind(account)
            .fetch_optional(&mut *connection)
            .await?;

    let (xsk, xvk, taddress) = match r {
        Some((None, None)) => {
            // no xprv, no xpub => get the address imported as bip38
            let taddress =
                sqlx::query("SELECT address FROM transparent_address_accounts WHERE account = ?1 AND dindex = ?2 AND scope = 0")
                    .bind(account)
                    .bind(dindex)
                    .map(|row: SqliteRow| row.get::<String, _>(0))
                    .fetch_optional(&mut *connection)
                    .await?;
            (None, None, taddress)
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

pub async fn select_account_sapling(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<SaplingKeys> {
    let r: Option<(Option<Vec<u8>>, Vec<u8>, String)> =
        sqlx::query_as("SELECT xsk, xvk, address FROM sapling_accounts WHERE account = ?")
            .bind(account)
            .fetch_optional(&mut *connection)
            .await?;

    let (xsk, xvk, address) = match r {
        Some((xsk, xvk, address)) => (xsk, Some(xvk), Some(address)),
        None => (None, None, None),
    };

    let keys = SaplingKeys {
        xsk: xsk.map(|xsk| {
            ExtendedSpendingKey::from_bytes(&xsk)
                .map_err(|_| anyhow!("Invalid sdk"))
                .unwrap()
        }),
        xvk: xvk
            .map(|xvk| DiversifiableFullViewingKey::from_bytes(&xvk.try_into().unwrap()).unwrap()),
        address: address.map(|a| PaymentAddress::decode(network, &a).unwrap()),
    };

    Ok(keys)
}

pub async fn select_account_orchard(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<OrchardKeys> {
    let r: Option<(Option<Vec<u8>>, Vec<u8>)> =
        sqlx::query_as("SELECT xsk, xvk FROM orchard_accounts WHERE account = ?")
            .bind(account)
            .fetch_optional(&mut *connection)
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
    pub address: Option<PaymentAddress>,
}

pub struct OrchardKeys {
    pub xsk: Option<SpendingKey>,
    pub xvk: Option<FullViewingKey>,
}

pub async fn list_accounts(connection: &mut SqliteConnection, coin: u8) -> Result<Vec<Account>> {
    let mut rows = sqlx::query(
        "WITH sh AS (SELECT account, MIN(height) AS height FROM sync_heights GROUP BY account),
        unspent AS (SELECT a.*
                FROM notes a
                LEFT JOIN spends b ON a.id_note = b.id_note
                WHERE b.id_note IS NULL AND a.id_asset IS NULL)
        SELECT id_account, a.name, seed, passphrase, aindex, dindex,
        icon, birth, use_internal, a.position, hidden, saved, enabled, internal,
        sh.height, COALESCE(hdr.time, 0), COALESCE(SUM(unspent.value), 0) AS balance,
        COALESCE(f.id_folder, 0), COALESCE(f.name, '') AS folder_name,
        hw
        FROM accounts a
        JOIN sh ON a.id_account = sh.account
        LEFT JOIN headers hdr ON sh.height = hdr.height
        LEFT JOIN unspent ON a.id_account = unspent.account
        LEFT JOIN folders f ON a.folder = f.id_folder
        GROUP BY id_account
        ORDER by a.position",
    )
    .map(|row: SqliteRow| {
        let folder = Folder {
            id: row.get(17),
            name: row.get(18),
        };
        Account {
            coin,
            id: row.get(0),
            name: row.get(1),
            seed: row.get(2),
            passphrase: row.get(3),
            aindex: row.get(4),
            dindex: row.get(5),
            icon: row.get(6),
            birth: row.get(7),
            use_internal: row.get(8),
            position: row.get(9),
            hidden: row.get(10),
            saved: row.get(11),
            enabled: row.get(12),
            internal: row.get(13),
            height: row.get(14),
            time: row.get(15),
            balance: row.get::<i64, _>(16) as u64,
            folder,
            hw: row.get::<u8, _>(19),
        }
    })
    .fetch(&mut *connection);

    let mut accounts = vec![];
    while let Some(row) = rows.try_next().await? {
        accounts.push(row);
    }

    Ok(accounts)
}

pub async fn get_account_fingerprint(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<Vec<u8>>> {
    let (fingerprint,): (Option<Vec<u8>>,) =
        sqlx::query_as("SELECT seed_fingerprint FROM accounts WHERE id_account = ?")
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;

    Ok(fingerprint)
}

pub async fn delete_account(connection: &mut SqliteConnection, account: u32) -> Result<()> {
    let mut tx = connection.begin().await?;

    sqlx::query("DELETE FROM dkg_params WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM dkg_packages WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM dkg_addresses WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM dkg_state WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM dkg_peers WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM frost_signatures WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM frost_commitments WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM outputs WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
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
    connection: &mut SqliteConnection,
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

pub async fn get_sync_height(conn: &mut SqliteConnection, account: u32) -> Result<Option<u32>> {
    let (h,): (Option<u32>,) = sqlx::query_as(
        "SELECT MIN(height) FROM sync_heights
    WHERE account = ?1",
    )
    .bind(account)
    .fetch_one(conn)
    .await?;
    Ok(h)
}

pub async fn calculate_balance(
    pool: &mut SqliteConnection,
    account: u32,
    height: Option<u32>,
) -> Result<PoolBalance> {
    let mut balance = PoolBalance(vec![0, 0, 0]);
    let height = height.unwrap_or(u32::MAX);

    let mut rows = sqlx::query("
    WITH N AS (SELECT value, pool, height FROM notes WHERE account = ?1 AND id_asset IS NULL UNION ALL SELECT s.value, s.pool, s.height FROM spends s JOIN notes n ON s.id_note = n.id_note WHERE s.account = ?1 AND n.id_asset IS NULL)
    SELECT pool, SUM(value) FROM N WHERE height <= ?2 GROUP BY pool")
        .bind(account)
        .bind(height)
        .map(|row: SqliteRow| (row.get::<u8, _>(0), row.get::<i64, _>(1)))
        .fetch(pool);
    while let Some((pool, value)) = rows.try_next().await? {
        balance.0[pool as usize] += value as u64;
    }

    Ok(balance)
}

pub async fn fetch_txs(connection: &mut SqliteConnection, account: u32) -> Result<Vec<Tx>> {
    // union notes and spends, then sum value by tx into v to get tx value
    // join transactions with v by id_tx and filter by account
    // order by height desc to get latest transactions first
    let transactions = sqlx::query(
        "SELECT id_tx, txid, height, time, value, tpe, c.name, t.zsa_value, t.price, t.asset_id,
            a.asset_name, a.asset_desc_hash
            FROM transactions t
            LEFT JOIN categories c ON c.id_category = t.category
            LEFT JOIN assets a ON t.asset_id = a.id_asset
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
        let tpe: Option<u8> = row.get(5);
        let category: Option<String> = row.get(6);
        let zsa_value: i64 = row.get(7);
        let price: Option<f64> = row.get(8);
        let asset_id: Option<i32> = row.get(9);
        let asset_name: Option<String> = row.get(10);
        let asset_desc_hash: Option<Vec<u8>> = row.get(11);
        Tx {
            id,
            txid,
            height,
            time,
            value,
            tpe,
            category,
            zsa_value,
            asset_id,
            asset_display: crate::account::asset_display(
                asset_id,
                asset_name,
                asset_desc_hash,
            ),
            price,
        }
    })
    .fetch_all(&mut *connection)
    .await?;
    Ok(transactions)
}

pub async fn get_memos(pool: &mut SqliteConnection, account: u32) -> Result<Vec<Memo>> {
    let memos = sqlx::query(
        "SELECT id_memo, m.height, tx, pool, vout, note, t.time, memo_text, memo_bytes
        FROM memos m JOIN transactions t ON m.tx = t.id_tx
        WHERE m.account = ? ORDER BY m.height DESC",
    )
    .bind(account)
    .map(row_to_memo)
    .fetch_all(pool)
    .await?;

    Ok(memos)
}

pub async fn get_memos_txid(
    pool: &mut SqliteConnection,
    account: u32,
    txid: &[u8],
) -> Result<Vec<Memo>> {
    let memos = sqlx::query(
        "SELECT id_memo, m.height, tx, pool, vout, note, t.time, memo_text, memo_bytes
        FROM memos m JOIN transactions t ON m.tx = t.id_tx
        WHERE m.account = ?1 AND t.txid = ?2",
    )
    .bind(account)
    .bind(txid)
    .map(row_to_memo)
    .fetch_all(pool)
    .await?;

    Ok(memos)
}

fn row_to_memo(row: SqliteRow) -> Memo {
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
}

pub async fn get_account_aindex(connection: &mut SqliteConnection, account: u32) -> Result<u32> {
    let (dindex,): (u32,) = sqlx::query_as("SELECT aindex FROM accounts WHERE id_account = ?")
        .bind(account)
        .fetch_one(&mut *connection)
        .await?;
    Ok(dindex)
}

pub async fn get_account_dindex(connection: &mut SqliteConnection, account: u32) -> Result<u32> {
    let (dindex,): (u32,) = sqlx::query_as("SELECT dindex FROM accounts WHERE id_account = ?")
        .bind(account)
        .fetch_one(&mut *connection)
        .await?;
    Ok(dindex)
}

pub async fn get_account_hw(connection: &mut SqliteConnection, account: u32) -> Result<u8> {
    let (hw,): (u8,) = sqlx::query_as("SELECT hw FROM accounts WHERE id_account = ?")
        .bind(account)
        .fetch_one(&mut *connection)
        .await?;
    Ok(hw)
}

pub async fn get_notes(connection: &mut SqliteConnection, account: u32) -> Result<Vec<TxNote>> {
    let notes = sqlx::query(
        "SELECT n.id_note, n.height, n.pool, n.tx, n.scope, n.diversifier, n.diversifier_index, n.value, n.locked,
        m.memo_text, n.id_asset, a.asset_name, a.asset_desc_hash
        FROM notes n LEFT JOIN spends s
	    ON n.id_note = s.id_note
        LEFT JOIN memos m ON n.id_note = m.note
        LEFT JOIN assets a ON n.id_asset = a.id_asset
	    WHERE n.account = ? AND s.id_note IS NULL ORDER BY n.height DESC",
    )
    .bind(account)
    .map(row_to_note)
    .fetch_all(&mut *connection)
    .await?;

    Ok(notes)
}

pub async fn get_notes_txid(
    connection: &mut SqliteConnection,
    account: u32,
    txid: &[u8],
) -> Result<Vec<TxNote>> {
    // Return all notes for a given transaction
    // including the ones that may be spent
    let notes = sqlx::query(
        "SELECT n.id_note, n.height, n.pool, n.tx, n.scope, n.diversifier, n.diversifier_index, n.value, n.locked,
        m.memo_text, n.id_asset, a.asset_name, a.asset_desc_hash
       FROM notes n
       JOIN transactions t ON n.tx = t.id_tx
       LEFT JOIN memos m ON n.id_note = m.note
       LEFT JOIN assets a ON n.id_asset = a.id_asset
	   WHERE n.account = ?1
       AND t.txid = ?2",
    )
    .bind(account)
    .bind(txid)
    .map(row_to_note)
    .fetch_all(&mut *connection)
    .await?;

    Ok(notes)
}

fn row_to_note(row: SqliteRow) -> TxNote {
    let id_note: u32 = row.get(0);
    let height: u32 = row.get(1);
    let pool: u8 = row.get(2);
    let tx: u32 = row.get(3);
    let scope: u8 = row.get(4);
    let diversifier: Option<Vec<u8>> = row.get(5);
    let diversifier_index: Option<i64> = row.get(6);
    let value: u64 = row.get(7);
    let locked: bool = row.get(8);
    let memo: Option<String> = row.get(9);
    let id_asset: Option<i64> = row.get(10);
    let asset_name: Option<String> = row.get(11);
    let asset_desc_hash: Option<Vec<u8>> = row.get(12);

    TxNote {
        id: id_note,
        height,
        pool,
        tx,
        scope,
        diversifier,
        diversifier_index,
        value,
        locked,
        id_asset: id_asset.map(|v| v as u32),
        memo,
        asset_display: crate::account::asset_display(
            id_asset.map(|v| v as i32),
            asset_name,
            asset_desc_hash,
        ),
    }
}

pub async fn lock_note(
    connection: &mut SqliteConnection,
    account: u32,
    id: u32,
    locked: bool,
) -> Result<()> {
    sqlx::query("UPDATE notes SET locked = ? WHERE account = ? AND id_note = ?")
        .bind(locked)
        .bind(account)
        .bind(id)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

/// Raw row from the assets table joined with unspent note balances.
#[derive(Clone, Debug)]
pub struct ZsaAssetRow {
    pub id_asset: i64,
    pub asset_desc_hash: Vec<u8>,
    pub asset_name: Option<String>,
    pub ik: Vec<u8>,
    pub asset_base: Vec<u8>,
    pub finalized: bool,
    pub first_seen_height: i32,
    pub balance: i64,
}

pub async fn get_zsa_holdings(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Vec<ZsaAssetRow>> {
    let holdings = sqlx::query(
        "SELECT a.id_asset, a.asset_desc_hash, a.asset_name, a.ik, a.asset_base,
                a.finalized, a.first_seen_height,
                COALESCE(SUM(n.value), 0) AS balance
         FROM assets a
         LEFT JOIN notes n ON n.id_asset = a.id_asset
           AND n.account = ?1
           AND n.id_note NOT IN (SELECT id_note FROM spends)
           AND n.locked = 0
         GROUP BY a.id_asset
         HAVING balance > 0
         ORDER BY a.asset_name, a.asset_desc_hash",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let id_asset: i64 = row.get(0);
        let asset_desc_hash: Vec<u8> = row.get(1);
        let asset_name: Option<String> = row.get(2);
        let ik: Vec<u8> = row.get(3);
        let asset_base: Vec<u8> = row.get(4);
        let finalized: bool = row.get(5);
        let first_seen_height: i32 = row.get(6);
        let balance: i64 = row.get(7);
        ZsaAssetRow {
            id_asset,
            asset_desc_hash,
            asset_name,
            ik,
            asset_base,
            finalized,
            first_seen_height,
            balance,
        }
    })
    .fetch_all(&mut *connection)
    .await?;

    Ok(holdings)
}

pub async fn fetch_transparent_address_tx_count(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Vec<TAddressTxCount>> {
    let rows = sqlx::query(
        "WITH n AS (
        SELECT account, tx, value, taddress FROM notes n WHERE n.pool = 0 UNION ALL
        SELECT n.account, s.tx, s.value, n.taddress FROM spends s JOIN notes n ON s.id_note = n.id_note AND s.account = n.account WHERE s.pool = 0)
        SELECT address, scope, dindex, SUM(n.value), COUNT(tx), MAX(t.time) FROM n
        JOIN transparent_address_accounts ta ON ta.id_taddress = taddress
        JOIN transactions t ON t.id_tx = n.tx
        WHERE n.account = ?
        GROUP BY taddress
        ORDER BY ta.scope, ta.dindex",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let address: String = row.get(0);
        let scope: u8 = row.get(1);
        let dindex: u32 = row.get(2);
        let amount: u64 = row.get(3);
        let tx_count: u32 = row.get(4);
        let time: u32 = row.get(5);
        TAddressTxCount {
            pool: 0,
            address,
            scope,
            dindex,
            amount,
            tx_count,
            time,
        }
    })
    .fetch_all(&mut *connection)
    .await?;

    Ok(rows)
}

/// Raw tx stats for a transparent address slot, keyed by (scope, dindex).
pub struct TransparentSlotStats {
    pub scope: u8,
    pub dindex: u32,
    pub amount: u64,
    pub tx_count: u32,
    pub time: u32,
}

/// Batch query: tx stats for ALL transparent address slots for an account.
/// Returns rows keyed by (scope, dindex), including zero-tx slots (LEFT JOIN).
pub async fn fetch_transparent_slot_stats(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Vec<TransparentSlotStats>> {
    let rows = sqlx::query(
        "WITH n AS (
        SELECT tx, value, taddress FROM notes WHERE pool = 0 AND account = ?1
        UNION ALL
        SELECT s.tx, s.value, n2.taddress FROM spends s
        JOIN notes n2 ON s.id_note = n2.id_note AND s.account = n2.account
        WHERE s.pool = 0 AND n2.account = ?1)
        SELECT ta.scope, ta.dindex,
               COALESCE(SUM(n.value), 0), COUNT(n.tx), COALESCE(MAX(t.time), 0)
        FROM transparent_address_accounts ta
        LEFT JOIN n ON ta.id_taddress = n.taddress
        LEFT JOIN transactions t ON t.id_tx = n.tx AND t.account = ?1
        WHERE ta.account = ?1
        GROUP BY ta.scope, ta.dindex
        ORDER BY ta.scope, ta.dindex",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let scope: u8 = row.get::<i64, _>(0) as u8;
        let dindex: u32 = row.get::<i64, _>(1) as u32;
        let amount: u64 = row.get::<i64, _>(2) as u64;
        let tx_count: u32 = row.get(3);
        let time: u32 = row.get::<Option<u32>, _>(4).unwrap_or(0);
        TransparentSlotStats {
            scope,
            dindex,
            amount,
            tx_count,
            time,
        }
    })
    .fetch_all(&mut *connection)
    .await?;

    Ok(rows)
}

/// Raw tx stats for a shielded address slot, keyed by (pool, scope, dindex).
pub struct ShieldedSlotStats {
    pub pool: u8,
    pub scope: u8,
    pub dindex: u32,
    pub amount: u64,
    pub tx_count: u32,
    pub time: u32,
}

/// Batch query: tx stats for ALL shielded address slots (Sapling + Orchard).
/// Groups notes+spends by (pool, scope, diversifier_index).
pub async fn fetch_shielded_slot_stats(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Vec<ShieldedSlotStats>> {
    let rows = sqlx::query(
        "SELECT sub.pool, sub.scope, sub.diversifier_index,
                SUM(sub.value), COUNT(sub.tx), COALESCE(MAX(t.time), 0)
        FROM (
            SELECT pool, scope, diversifier_index, tx, value
            FROM notes WHERE account = ?1 AND pool IN (1, 2)
            UNION ALL
            SELECT n.pool, n.scope, n.diversifier_index, s.tx, s.value
            FROM spends s
            JOIN notes n ON s.id_note = n.id_note AND s.account = n.account
            WHERE s.pool IN (1, 2) AND n.account = ?1
        ) sub
        JOIN transactions t ON t.id_tx = sub.tx AND t.account = ?1
        GROUP BY sub.pool, sub.scope, sub.diversifier_index
        ORDER BY sub.pool, sub.scope, sub.diversifier_index",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let pool: u8 = row.get::<i64, _>(0) as u8;
        let scope: u8 = row.get::<i64, _>(1) as u8;
        let dindex: u32 = row.get::<i64, _>(2) as u32;
        let amount: u64 = row.get::<i64, _>(3) as u64;
        let tx_count: u32 = row.get(4);
        let time: u32 = row.get::<Option<u32>, _>(5).unwrap_or(0);
        ShieldedSlotStats {
            pool,
            scope,
            dindex,
            amount,
            tx_count,
            time,
        }
    })
    .fetch_all(&mut *connection)
    .await?;

    Ok(rows)
}

pub async fn change_db_password(
    db_filepath: &str,
    tmp_dir: &str,
    old_password: &str,
    new_password: &str,
) -> Result<()> {
    let mut options = SqliteConnectOptions::new().filename(db_filepath);
    if !old_password.is_empty() {
        let escaped_old_password = old_password.replace('\'', "''");
        options = options.pragma("key", format!("'{escaped_old_password}'"));
    }

    let tmp_db_filepath = format!("{tmp_dir}/__tmp.db");
    File::create(&tmp_db_filepath)?;

    {
        let mut connection = SqliteConnection::connect_with(&options).await?;
        let escaped_password = new_password.replace('\'', "''");
        sqlx::query(&format!(
            "ATTACH DATABASE '{}' AS new_db KEY '{}'",
            tmp_db_filepath, escaped_password
        ))
        .execute(&mut connection)
        .await?;
        sqlx::query("SELECT sqlcipher_export('new_db')")
            .execute(&mut connection)
            .await?;
        sqlx::query("DETACH DATABASE new_db")
            .execute(&mut connection)
            .await?;
    }

    std::fs::remove_file(db_filepath)?;
    std::fs::rename(tmp_db_filepath, db_filepath)?;

    Ok(())
}

pub async fn store_pending_tx(
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
    txid: &[u8],
    price: Option<f64>,
    category: Option<u32>,
) -> Result<()> {
    let mut txid = txid.to_vec();
    txid.reverse();
    sqlx::query(
        "INSERT OR REPLACE INTO pending_txs(account, height, txid, price, category)
    VALUES (?, ?, ?, ?, ?)",
    )
    .bind(account)
    .bind(height)
    .bind(&txid)
    .bind(price)
    .bind(category)
    .execute(connection)
    .await?;
    Ok(())
}

pub async fn set_tx_category(
    connection: &mut SqliteConnection,
    id: u32,
    category: Option<u32>,
) -> Result<()> {
    sqlx::query("UPDATE transactions SET category = ?2 WHERE id_tx = ?1")
        .bind(id)
        .bind(category)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

pub async fn set_tx_price(
    connection: &mut SqliteConnection,
    id: u32,
    price: Option<f64>,
) -> Result<()> {
    sqlx::query("UPDATE transactions SET price = ?2 WHERE id_tx = ?1")
        .bind(id)
        .bind(price)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

pub async fn export_data(
    connection: &mut SqliteConnection,
    account: u32,
    tpe: u8,
    writer: &mut AsyncWriter<Vec<u8>>,
) -> Result<()> {
    let sql = match tpe {
        0 => "SELECT t.*, c.name FROM transactions t LEFT JOIN categories c ON c.id_category = t.category WHERE account = ?1 ORDER BY height",
        1 => "SELECT * FROM memos WHERE account = ?1 ORDER BY height",
        2 => "SELECT n.* FROM notes n LEFT JOIN spends s ON n.id_note = s.id_note WHERE n.account = ?1 AND s.id_note IS NULL ORDER BY height",
        3 => "WITH N AS (SELECT id_asset, value FROM notes WHERE account = ?1 AND id_asset IS NOT NULL UNION ALL SELECT n.id_asset, s.value FROM spends s JOIN notes n ON s.id_note = n.id_note WHERE s.account = ?1 AND n.id_asset IS NOT NULL) SELECT a.id_asset, hex(a.asset_desc_hash), a.asset_name, a.finalized, a.first_seen_height, COALESCE(SUM(N.value), 0) AS balance FROM assets a LEFT JOIN N ON N.id_asset = a.id_asset GROUP BY a.id_asset HAVING balance > 0 ORDER BY a.asset_name, a.asset_desc_hash",
        _ => anyhow::bail!("Invalid exported data type")
    };

    let mut rows = sqlx::query(sql)
        .bind(account)
        .map(|r: SqliteRow| {
            r.columns()
                .iter()
                .enumerate()
                .map(|(i, _)| get_sqlite_column_value(&r, i))
                .collect::<Result<Vec<_>>>()
        })
        .fetch(connection);

    while let Some(Ok(row)) = rows.try_next().await? {
        writer.write_record(row).await?;
    }
    Ok(())
}

fn get_sqlite_column_value(row: &SqliteRow, index: usize) -> Result<String> {
    let c = row.column(index);
    let t = c.type_info();
    let v = if let Ok(v) = row.try_get::<i64, _>(index) {
        v.to_string()
    } else if let Ok(v) = row.try_get::<f64, _>(index) {
        v.to_string()
    } else if let Ok(v) = row.try_get::<String, _>(index) {
        v
    } else if let Ok(mut v) = row.try_get::<Vec<u8>, _>(index) {
        if c.name() == "txid" {
            v.reverse();
        }
        hex::encode(&v)
    } else {
        unreachable!("{}", t.name())
    };

    Ok(v)
}

pub async fn lock_recent_notes(
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
    threshold: u32,
) -> Result<()> {
    let max_height = height.saturating_sub(threshold);
    sqlx::query("UPDATE notes SET locked = TRUE WHERE account = ?1 AND height > ?2")
        .bind(account)
        .bind(max_height)
        .execute(connection)
        .await?;
    Ok(())
}

pub async fn unlock_all_notes(connection: &mut SqliteConnection, account: u32) -> Result<()> {
    sqlx::query("UPDATE notes SET locked = FALSE WHERE account = ?1")
        .bind(account)
        .execute(connection)
        .await?;
    Ok(())
}

// TODO: Include pool filter
// Unfortunately, the current UI flow asks for the amount before
// the source pool selection. Therefore we don't know what the user
// wants to use yet
pub async fn max_spendable(connection: &mut SqliteConnection, account: u32) -> Result<u64> {
    let (amount,): (Option<u64>,) = sqlx::query_as(
        "SELECT SUM(n.value) FROM notes n LEFT JOIN spends s ON n.id_note = s.id_note
    WHERE s.id_note IS NULL AND n.account = ?1 AND n.id_asset IS NULL AND NOT(locked)",
    )
    .bind(account)
    .fetch_one(connection)
    .await?;
    Ok(amount.unwrap_or_default())
}
