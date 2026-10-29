use std::fs::File;

use crate::keys::{
    AccountViewingKeys, OrchardViewingKeys, SaplingDiversifiedAddress, SaplingViewingKeys, ScopeExt,
};
use anyhow::{anyhow, Result};
use csv_async::AsyncWriter;
use futures::TryStreamExt;
use orchard::keys::{FullViewingKey, IncomingViewingKey as OrchardIvk, SpendingKey};
use sapling_crypto::{
    keys::{NullifierDerivingKey, SaplingIvk},
    zip32::IncomingViewingKey as SaplingZip32Ivk,
    PaymentAddress,
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteRow},
    Column, Connection, Row, SqliteConnection, TypeInfo,
};
use tracing::debug;
use zcash_keys::{
    encoding::AddressCodec,
    keys::sapling::{DiversifiableFullViewingKey, ExtendedSpendingKey},
};
use zcash_protocol::consensus::NetworkUpgrade;
use zcash_protocol::consensus::Parameters;
use zcash_transparent::keys::{
    AccountPrivKey, AccountPubKey, ExternalIvk, IncomingViewingKey as _,
};

use crate::api::account::Folder;
use crate::api::account::TAddressTxCount;
use crate::api::account::TxNote;
use crate::api::account::{Account, Memo, Tx};
use crate::api::coin::Network;
use crate::api::sync::PoolBalance;

/// Schema version. Bump only when the export format changes (IOAccount or any
/// embedded struct gains/removes/changes a field). Do NOT bump for runtime-only
/// changes (queries, PoolBalance, solve mode, etc.).
pub const DB_VERSION: u16 = 9;

pub async fn create_schema(connection: &mut SqliteConnection) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS props(
        key TEXT PRIMARY KEY,
        VALUE TEXT NOT NULL)",
    )
    .execute(&mut *connection)
    .await?;

    // Remove the obsolete vote node setting from existing wallets.
    sqlx::query("DELETE FROM props WHERE key = 'vote_node_url'")
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
    // An account restored from a backup by earlier versions lost the flag
    // (the import did not write it), so its uncompressed keys were hashed
    // as compressed and it could not spend. The stored key is in its
    // original serialization, 65 bytes only when uncompressed.
    sqlx::query(
        "UPDATE transparent_address_accounts SET uncompressed = TRUE
        WHERE uncompressed = FALSE AND length(pk) = 65",
    )
    .execute(&mut *connection)
    .await?;

    // Incoming-only accounts (imported from an incoming viewing key) hold a
    // pool's incoming key and no full viewing key. For transparent that is
    // the external chain's key: chain code and public key.
    let _ = sqlx::query("ALTER TABLE transparent_accounts ADD COLUMN ivk BLOB")
        .execute(&mut *connection)
        .await;

    let _ =
        sqlx::query("ALTER TABLE sapling_accounts ADD COLUMN address BLOB NOT NULL DEFAULT('')")
            .execute(&mut *connection)
            .await;
    migrate_sapling_accounts(connection).await?;
    migrate_orchard_accounts(connection).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sync_heights(
        account INTEGER,
        pool INTEGER NOT NULL,
        height INTEGER NOT NULL,
        PRIMARY KEY (account, pool))",
    )
    .execute(&mut *connection)
    .await?;

    // Additive wallet-local history; does not change the IOAccount export format.
    sqlx::raw_sql(include_str!("db/swaps.sql"))
        .execute(&mut *connection)
        .await?;

    migrate_headers(connection).await?;

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
        "CREATE TABLE IF NOT EXISTS user_memos(
        account INTEGER NOT NULL,
        id_tx INTEGER NOT NULL,
        height INTEGER NOT NULL,
        user_memo TEXT NOT NULL,
        UNIQUE(account, id_tx))",
    )
    .execute(&mut *connection)
    .await?;
    migrate_user_memos(connection).await?;
    tracing::debug!("create_schema: user_memos table ready");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS assets(
        id_asset INTEGER PRIMARY KEY,
        asset_desc_hash BLOB NOT NULL,
        ik BLOB NOT NULL,
        asset_base BLOB NOT NULL,
        finalized BOOL NOT NULL DEFAULT FALSE,
        first_seen_height INTEGER NOT NULL,
        UNIQUE (asset_desc_hash, ik),
        UNIQUE (asset_base))",
    )
    .execute(&mut *connection)
    .await?;

    // Migration: add id_asset to notes for ZSA note→asset linking
    let _ =
        sqlx::query("ALTER TABLE notes ADD COLUMN id_asset INTEGER REFERENCES assets(id_asset)")
            .execute(&mut *connection)
            .await;

    // Migration: add diversifier_index to notes for per-address shielded tx counts
    let _ = sqlx::query("ALTER TABLE notes ADD COLUMN diversifier_index INTEGER")
        .execute(&mut *connection)
        .await;

    // Migration: add asset_name to assets for human-readable naming
    let _ = sqlx::query("ALTER TABLE assets ADD COLUMN asset_name TEXT")
        .execute(&mut *connection)
        .await;

    // Migration: ensure asset_base is unique to prevent duplicate note inserts
    // caused by duplicate issuances with the same asset_base.
    let _ = sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_assets_asset_base ON assets(asset_base)",
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
    let _ = sqlx::query(
        "ALTER TABLE transactions ADD COLUMN asset_id INTEGER REFERENCES assets(id_asset)",
    )
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

    // V9 — contacts
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS contacts (
        id_contact INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        notes TEXT NOT NULL DEFAULT '')",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS contact_addresses (
        id_address INTEGER PRIMARY KEY,
        contact_id INTEGER NOT NULL REFERENCES contacts(id_contact) ON DELETE CASCADE,
        address TEXT NOT NULL,
        receiver TEXT NOT NULL,
        pool INTEGER NOT NULL,
        ordinal INTEGER NOT NULL DEFAULT 0)",
    )
    .execute(&mut *connection)
    .await?;

    let _ = sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_contact_receiver ON contact_addresses(receiver, pool)",
    )
    .execute(&mut *connection)
    .await;

    // Plugins
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS plugins(
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            author TEXT,
            description TEXT,
            min_app_version TEXT NOT NULL,
            types TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            install_dir TEXT NOT NULL,
            script TEXT NOT NULL,
            manifest_json TEXT NOT NULL)",
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

    // DKG publish intent: the outgoing package(s) of the round being
    // published, staged before the broadcast and cleared after it. NULL means
    // nothing pending (or already sent under an older build). Added without a
    // DB_VERSION bump: nullable, only read by new code, older builds ignore it.
    let _ = sqlx::query("ALTER TABLE dkg_state ADD COLUMN pending_publish BLOB")
        .execute(&mut *connection)
        .await;

    // DKG generation of a frost account (ZIP 2005, #1334): 0 before message
    // signing, 1 with a random ed25519 key and the legacy rivk, 2 with the
    // ZIP 2005 derivation. Added without a DB_VERSION bump: defaulted,
    // only read by the FVK builder, older builds ignore it. The ALTER
    // succeeds exactly once, which is when the one-time backfill runs:
    // a completed DKG with no round-0 peer row predates message signing.
    let added = sqlx::query("ALTER TABLE dkg_params ADD COLUMN dkg_version INTEGER NOT NULL DEFAULT 1")
        .execute(&mut *connection)
        .await
        .is_ok();
    if added {
        sqlx::query(
            "UPDATE dkg_params SET dkg_version = 0
            WHERE account IN (SELECT account FROM dkg_state WHERE key_pkg IS NOT NULL)
            AND NOT EXISTS (SELECT 1 FROM dkg_peers
                WHERE dkg_peers.account = dkg_params.account AND round = 0)",
        )
        .execute(&mut *connection)
        .await?;
    }
    // The broadcast account's Orchard spending key, kept for dkg-2 accounts
    // whose rivk is derived from it; NULL for legacy accounts, whose sk is
    // rederivable from the stored addresses.
    let _ = sqlx::query("ALTER TABLE dkg_params ADD COLUMN sk BLOB")
        .execute(&mut *connection)
        .await;

    // Stale voting schema. These tables belong to the election implementation
    // that predates ZIP 262; nothing in the tree reads them any more, and the
    // redesign on zcash_voting 5.1.0 keeps its state in a sidecar database
    // (`<wallet>.voting`) rather than in the wallet pool. Dropping them here
    // is idempotent and needs no DB_VERSION bump: the format exported by
    // IOAccount never carried them.
    for table in [
        "v_actions",
        "v_ballots",
        "v_elections",
        "v_final_results",
        "v_notes",
        "v_questions",
        "v_results",
        "v_spends",
        "v_state",
        "v_witnesses",
        "vc_cmxs",
        "vc_nfs",
        "vs_cmxs",
    ] {
        // Audited for sqlx 0.9's SqlSafeStr: DROP TABLE takes no bind
        // parameters and every name comes from the literal list above.
        sqlx::query(sqlx::AssertSqlSafe(format!("DROP TABLE IF EXISTS {table}")))
            .execute(&mut *connection)
            .await?;
    }

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
    // Cover both empty and NULL addresses: older schemas left the column
    // NULL, newer ones default it to ''.
    let accounts: Vec<(u32, u32, Vec<u8>)> = sqlx::query_as(
        "SELECT id_account, dindex, xvk FROM accounts a
            JOIN sapling_accounts s ON a.id_account = s.account
            WHERE address IS NULL OR address = ''",
    )
    .fetch_all(&mut *connection)
    .await?;

    for (account, dindex, xvk) in accounts {
        // A row that cannot be backfilled must not abort the migration (or
        // panic the runtime): skip it, and the lenient address readers will
        // report it as missing.
        let Ok(fvk) = <[u8; 128]>::try_from(xvk.as_slice()) else {
            tracing::warn!("sapling xvk for account {account} is not 128 bytes; skipping backfill");
            continue;
        };
        let Some(fvk) = DiversifiableFullViewingKey::from_bytes(&fvk) else {
            tracing::warn!("invalid sapling xvk for account {account}; skipping backfill");
            continue;
        };
        let Some(address) = fvk.address((dindex as u64).into()) else {
            tracing::warn!("invalid diversifier {dindex} for account {account}; skipping backfill");
            continue;
        };
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
        "SELECT COUNT(*) FROM notes WHERE diversifier IS NOT NULL AND diversifier_index IS NULL",
    )
    .fetch_one(&mut *connection)
    .await?;

    if count.0 == 0 {
        // Nothing to backfill, mark as done
        put_prop(connection, "backfilled_diversifier_index", "1").await?;
        return Ok(());
    }

    debug!("Backfilling diversifier_index for {} notes", count.0);

    // Fetch distinct accounts with unbackfilled notes
    let accounts: Vec<(u32,)> = sqlx::query_as(
        "SELECT DISTINCT account FROM notes WHERE diversifier IS NOT NULL AND diversifier_index IS NULL",
    )
    .fetch_all(&mut *connection)
    .await?;

    for (account,) in accounts {
        // Load Sapling DFVK for this account
        let sapling_dfvk: Option<DiversifiableFullViewingKey> =
            sqlx::query_as("SELECT xvk FROM sapling_accounts WHERE account = ?")
                .bind(account)
                .fetch_optional(&mut *connection)
                .await?
                .map(|(xvk,): (Vec<u8>,)| {
                    DiversifiableFullViewingKey::from_bytes(&xvk.try_into().unwrap()).unwrap()
                });

        // Load Orchard FVK for this account
        let orchard_fvk: Option<FullViewingKey> =
            sqlx::query_as("SELECT xvk FROM orchard_accounts WHERE account = ?")
                .bind(account)
                .fetch_optional(&mut *connection)
                .await?
                .map(|(xvk,): (Vec<u8>,)| {
                    FullViewingKey::from_bytes(&xvk.try_into().unwrap()).unwrap()
                });

        // Fetch unbackfilled notes for this account
        let notes: Vec<(u32, u8, u8, Vec<u8>)> = sqlx::query_as(
            "SELECT id_note, pool, scope, diversifier FROM notes WHERE account = ? AND diversifier IS NOT NULL AND diversifier_index IS NULL",
        )
        .bind(account)
        .fetch_all(&mut *connection)
        .await?;

        for (id_note, pool, scope, diversifier) in notes {
            let di: Option<i64> = match pool {
                1 => sapling_dfvk
                    .as_ref()
                    .and_then(|dfvk| resolve_sapling_diversifier_index(dfvk, scope, &diversifier)),
                2 | 3 => orchard_fvk
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

    debug!("Backfill diversifier_index complete");
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

pub async fn delete_prop_prefix(connection: &mut SqliteConnection, prefix: &str) -> Result<()> {
    sqlx::query("DELETE FROM props WHERE key LIKE ?")
        .bind(format!("{prefix}%"))
        .execute(&mut *connection)
        .await?;
    Ok(())
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

// Runtime schema migration; the account backup format is unchanged.
/// Give `user_memos` the height of its transaction, so a rewind trims it
/// with the chain log. Before, a note stayed on an `id_tx` that a rewind
/// could delete and a later sync hand to another transaction; notes whose
/// transaction is already gone are dropped here.
async fn migrate_user_memos(connection: &mut SqliteConnection) -> Result<()> {
    let has_height: Option<(String,)> =
        sqlx::query_as("SELECT name FROM pragma_table_info('user_memos') WHERE name = 'height'")
            .fetch_optional(&mut *connection)
            .await?;
    if has_height.is_some() {
        return Ok(());
    }
    let mut tx = connection.begin().await?;
    sqlx::query("ALTER TABLE user_memos ADD COLUMN height INTEGER NOT NULL DEFAULT 0")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "DELETE FROM user_memos WHERE NOT EXISTS (SELECT 1 FROM transactions t
        WHERE t.account = user_memos.account AND t.id_tx = user_memos.id_tx)",
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE user_memos SET height = (SELECT t.height FROM transactions t
        WHERE t.account = user_memos.account AND t.id_tx = user_memos.id_tx)",
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

async fn migrate_headers(connection: &mut SqliteConnection) -> Result<()> {
    let mut tx = connection.begin().await?;
    let columns = sqlx::query("PRAGMA table_info(headers)")
        .fetch_all(&mut *tx)
        .await?;
    let legacy = !columns.is_empty()
        && !columns
            .iter()
            .any(|row| row.get::<String, _>("name") == "account");
    if legacy {
        sqlx::query("ALTER TABLE headers RENAME TO legacy_headers")
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS headers (
        account INTEGER NOT NULL,
        height INTEGER NOT NULL,
        hash BLOB NOT NULL,
        time INTEGER NOT NULL,
        PRIMARY KEY (account, height))",
    )
    .execute(&mut *tx)
    .await?;
    if legacy {
        sqlx::query(
            "INSERT INTO headers (account, height, hash, time)
            SELECT a.id_account, h.height, h.hash, h.time
            FROM accounts a CROSS JOIN legacy_headers h",
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query("DROP TABLE legacy_headers")
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Whether `table` still has its full viewing key column NOT NULL: the
/// shape from before incoming-only accounts, which hold the pool's
/// incoming key and no full key. SQLite cannot drop NOT NULL in place.
/// A table that does not exist yet (a new wallet) has nothing to migrate.
async fn full_key_not_null(tx: &mut SqliteConnection, table: &str) -> Result<bool> {
    let notnull: Option<(bool,)> =
        sqlx::query_as("SELECT \"notnull\" FROM pragma_table_info(?1) WHERE name = 'xvk'")
            .bind(table)
            .fetch_optional(&mut *tx)
            .await?;
    Ok(notnull.is_some_and(|(notnull,)| notnull))
}

/// Sapling keys of an account: `xvk` for a full account, `ivk` (the ZIP 316
/// item, `dk || ivk`) for an incoming-only one. A wallet from before
/// incoming-only accounts is rebuilt once, the way `migrate_headers` does
/// it; a later run finds the column nullable and only creates if missing.
async fn migrate_sapling_accounts(connection: &mut SqliteConnection) -> Result<()> {
    let mut tx = connection.begin().await?;
    let legacy = full_key_not_null(&mut tx, "sapling_accounts").await?;
    if legacy {
        sqlx::query("ALTER TABLE sapling_accounts RENAME TO legacy_sapling_accounts")
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sapling_accounts(
        account INTEGER PRIMARY KEY,
        xsk BLOB,
        xvk BLOB,
        address BLOB NOT NULL DEFAULT(''),
        ivk BLOB)",
    )
    .execute(&mut *tx)
    .await?;
    if legacy {
        sqlx::query(
            "INSERT INTO sapling_accounts (account, xsk, xvk, address)
            SELECT account, xsk, xvk, address FROM legacy_sapling_accounts",
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query("DROP TABLE legacy_sapling_accounts")
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Orchard keys of an account, as [`migrate_sapling_accounts`] for Sapling;
/// the Orchard row also serves Ironwood.
async fn migrate_orchard_accounts(connection: &mut SqliteConnection) -> Result<()> {
    let mut tx = connection.begin().await?;
    let legacy = full_key_not_null(&mut tx, "orchard_accounts").await?;
    if legacy {
        sqlx::query("ALTER TABLE orchard_accounts RENAME TO legacy_orchard_accounts")
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS orchard_accounts(
        account INTEGER PRIMARY KEY,
        xsk BLOB,
        xvk BLOB,
        ivk BLOB)",
    )
    .execute(&mut *tx)
    .await?;
    if legacy {
        sqlx::query(
            "INSERT INTO orchard_accounts (account, xsk, xvk)
            SELECT account, xsk, xvk FROM legacy_orchard_accounts",
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query("DROP TABLE legacy_orchard_accounts")
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
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
    crate::sync::chain_log::set_checkpoint(connection, account, 0, birth).await?;

    Ok(())
}

pub const LEDGER_CODE: u32 = 1;

pub async fn store_account_hw(
    connection: &mut SqliteConnection,
    account: u32,
    hw_code: u8,
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

/// The external chain's incoming key of an incoming-only account: it
/// derives the receiving addresses and nothing else.
pub async fn store_account_transparent_ivk(
    connection: &mut SqliteConnection,
    account: u32,
    ivk: &ExternalIvk,
) -> Result<()> {
    sqlx::query("UPDATE transparent_accounts SET ivk = ? WHERE account = ?")
        .bind(ivk.serialize())
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
    sqlx::query("INSERT INTO sapling_accounts(account) VALUES (?)")
        .bind(account)
        .execute(&mut *connection)
        .await?;
    let activation_height: u32 = network
        .activation_height(NetworkUpgrade::Sapling)
        .unwrap()
        .into();
    crate::sync::chain_log::set_checkpoint(connection, account, 1, birth.max(activation_height)).await?;

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

/// The incoming key of an incoming-only Sapling account, with its default
/// address; the full viewing key stays NULL.
pub async fn store_account_sapling_ivk(
    connection: &mut SqliteConnection,
    account: u32,
    ivk: &SaplingZip32Ivk,
    address: &str,
) -> Result<()> {
    sqlx::query("UPDATE sapling_accounts SET ivk = ?2, address = ?3 WHERE account = ?1")
        .bind(account)
        .bind(ivk.to_bytes().as_slice())
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
    sqlx::query("INSERT INTO orchard_accounts(account) VALUES (?)")
        .bind(account)
        .execute(&mut *connection)
        .await?;
    let activation_height = network
        .activation_height(NetworkUpgrade::Nu5)
        .unwrap()
        .into();
    crate::sync::chain_log::set_checkpoint(connection, account, 2, birth.max(activation_height)).await?;

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

/// The incoming key of an incoming-only Orchard account (serving Ironwood
/// too); the full viewing key stays NULL.
pub async fn store_account_orchard_ivk(
    connection: &mut SqliteConnection,
    account: u32,
    ivk: &OrchardIvk,
) -> Result<()> {
    sqlx::query("UPDATE orchard_accounts SET ivk = ? WHERE account = ?")
        .bind(ivk.to_bytes().as_slice())
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
    let r: Option<(Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT xsk, xvk, ivk FROM transparent_accounts WHERE account = ?")
            .bind(account)
            .fetch_optional(&mut *connection)
            .await?;

    let ivk = r.as_ref().and_then(|(_, _, ivk)| ivk.clone());
    let (xsk, xvk, taddress) = match r {
        Some((None, None, None)) => {
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
        Some((xsk, xvk, _)) => (xsk, xvk, None),
        None => (None, None, None),
    };

    let keys = TransparentKeys {
        xsk: xsk.map(|xsk| AccountPrivKey::from_bytes(&xsk).unwrap()),
        xvk: xvk.map(|xvk| AccountPubKey::deserialize(&xvk.try_into().unwrap()).unwrap()),
        ivk: ivk.map(|ivk| ExternalIvk::deserialize(&ivk.try_into().unwrap()).unwrap()),
        address: taddress,
    };

    Ok(keys)
}

pub async fn select_account_sapling(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<SaplingKeys> {
    #[allow(clippy::type_complexity)]
    let r: Option<(Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>, String)> =
        sqlx::query_as("SELECT xsk, xvk, ivk, address FROM sapling_accounts WHERE account = ?")
            .bind(account)
            .fetch_optional(&mut *connection)
            .await?;

    let (xsk, xvk, ivk, address) = match r {
        Some((xsk, xvk, ivk, address)) => (xsk, xvk, ivk, Some(address)),
        None => (None, None, None, None),
    };

    let keys = SaplingKeys {
        xsk: xsk.map(|xsk| {
            ExtendedSpendingKey::from_bytes(&xsk)
                .map_err(|_| anyhow!("Invalid sdk"))
                .unwrap()
        }),
        xvk: xvk
            .map(|xvk| DiversifiableFullViewingKey::from_bytes(&xvk.try_into().unwrap()).unwrap()),
        ivk: ivk.map(|ivk| SaplingZip32Ivk::from_bytes(&ivk.try_into().unwrap()).unwrap()),
        // A corrupt or empty stored address (e.g. rows imported before the
        // address was backfilled) must not panic the runtime: surface a
        // clear error instead.
        address: match address {
            Some(a) => Some(PaymentAddress::decode(network, &a).map_err(|e| {
                anyhow!("invalid stored sapling address for account {account}: {e}")
            })?),
            None => None,
        },
    };

    Ok(keys)
}

pub async fn select_account_orchard(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<OrchardKeys> {
    let r: Option<(Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT xsk, xvk, ivk FROM orchard_accounts WHERE account = ?")
            .bind(account)
            .fetch_optional(&mut *connection)
            .await?;

    let (xsk, xvk, ivk) = match r {
        Some((xsk, xvk, ivk)) => (xsk, xvk, ivk),
        None => (None, None, None),
    };

    let keys = OrchardKeys {
        xsk: xsk.map(|xsk| SpendingKey::from_bytes(xsk.try_into().unwrap()).unwrap()),
        xvk: xvk.map(|xvk| FullViewingKey::from_bytes(&xvk.try_into().unwrap()).unwrap()),
        ivk: ivk.map(|ivk| OrchardIvk::from_bytes(&ivk.try_into().unwrap()).unwrap()),
    };

    Ok(keys)
}

/// The keys that find an account's incoming Sapling notes at `scope`: the
/// incoming key to trial-decrypt with and, for an account with its full
/// key, the nullifier key. An incoming-only account has the external
/// incoming key alone: it cannot derive nullifiers, so its notes are never
/// seen spent, and it has no internal scope. None when the account has no
/// Sapling key for that scope.
pub async fn sapling_receiving_keys(
    connection: &mut SqliteConnection,
    account: u32,
    scope: u8,
) -> Result<Option<(SaplingIvk, Option<NullifierDerivingKey>)>> {
    Ok(match sapling_viewing_keys(connection, account).await? {
        Some(SaplingViewingKeys::Full(dfvk)) => {
            let (ivk, nk) = crate::keys::sapling_ivk_nk_for_scope(scope, &dfvk);
            Some((ivk, Some(nk)))
        }
        Some(SaplingViewingKeys::Incoming(ivk)) if scope.is_external() => {
            let ivk = SaplingIvk::from_bytes(ivk.to_bytes()[32..].try_into().unwrap()).unwrap();
            Some((ivk, None))
        }
        _ => None,
    })
}

/// An account's Sapling viewing keys, full or incoming-only; None when the
/// account has no Sapling key.
pub async fn sapling_viewing_keys(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<SaplingViewingKeys>> {
    let row: Option<(Option<Vec<u8>>, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT xvk, ivk FROM sapling_accounts WHERE account = ?")
            .bind(account)
            .fetch_optional(&mut *connection)
            .await?;
    Ok(row.and_then(|(xvk, ivk)| SaplingViewingKeys::from_stored(xvk, ivk)))
}

/// An account's shielded viewing keys, full or incoming-only per pool.
pub async fn account_viewing_keys(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<AccountViewingKeys> {
    Ok(AccountViewingKeys {
        sapling: sapling_viewing_keys(&mut *connection, account).await?,
        orchard: orchard_viewing_keys(&mut *connection, account).await?,
    })
}

/// An account's Orchard viewing keys, full or incoming-only (they serve
/// Ironwood too); None when the account has no Orchard key.
pub async fn orchard_viewing_keys(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<OrchardViewingKeys>> {
    let row: Option<(Option<Vec<u8>>, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT xvk, ivk FROM orchard_accounts WHERE account = ?")
            .bind(account)
            .fetch_optional(&mut *connection)
            .await?;
    Ok(row.and_then(|(xvk, ivk)| OrchardViewingKeys::from_stored(xvk, ivk)))
}

/// The keys that find an account's incoming Orchard (and Ironwood) notes at
/// `scope`, as [`sapling_receiving_keys`]: the full viewing key stands as
/// the nullifier key, and is None for an incoming-only account.
pub async fn orchard_receiving_keys(
    connection: &mut SqliteConnection,
    account: u32,
    scope: u8,
) -> Result<Option<(OrchardIvk, Option<FullViewingKey>)>> {
    Ok(match orchard_viewing_keys(connection, account).await? {
        Some(OrchardViewingKeys::Full(fvk)) => {
            Some((fvk.to_ivk(scope.orchard_scope()), Some(fvk)))
        }
        Some(OrchardViewingKeys::Incoming(ivk)) if scope.is_external() => Some((ivk, None)),
        _ => None,
    })
}

/// An account's stored keys for one pool. A full account has `xvk` (and
/// `xsk` when it can spend); an incoming-only account has `ivk` alone.
pub struct TransparentKeys {
    pub xsk: Option<AccountPrivKey>,
    pub xvk: Option<AccountPubKey>,
    pub ivk: Option<ExternalIvk>,
    pub address: Option<String>,
}

pub struct SaplingKeys {
    pub xsk: Option<ExtendedSpendingKey>,
    pub xvk: Option<DiversifiableFullViewingKey>,
    pub ivk: Option<SaplingZip32Ivk>,
    pub address: Option<PaymentAddress>,
}

pub struct OrchardKeys {
    pub xsk: Option<SpendingKey>,
    pub xvk: Option<FullViewingKey>,
    pub ivk: Option<OrchardIvk>,
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
        hw,
        EXISTS(SELECT 1 FROM transparent_accounts k
            WHERE k.account = a.id_account AND k.xvk IS NULL AND k.ivk IS NOT NULL)
        OR EXISTS(SELECT 1 FROM sapling_accounts k
            WHERE k.account = a.id_account AND k.xvk IS NULL AND k.ivk IS NOT NULL)
        OR EXISTS(SELECT 1 FROM orchard_accounts k
            WHERE k.account = a.id_account AND k.xvk IS NULL AND k.ivk IS NOT NULL)
        AS incoming_only,
        EXISTS(SELECT 1 FROM dkg_params d WHERE d.account = a.id_account) AS frost
        FROM accounts a
        JOIN sh ON a.id_account = sh.account
        LEFT JOIN headers hdr ON sh.height = hdr.height AND sh.account = hdr.account
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
            incoming_only: row.get::<bool, _>(20),
            frost: row.get::<bool, _>(21),
        }
    })
    .fetch(&mut *connection);

    let mut accounts = vec![];
    while let Some(row) = rows.try_next().await? {
        accounts.push(row);
    }

    Ok(accounts)
}

/// Whether the account was created from an incoming viewing key: some pool
/// holds an incoming key and no full key. Such an account sees payments
/// received and nothing else, and cannot spend.
pub async fn is_incoming_only(connection: &mut SqliteConnection, account: u32) -> Result<bool> {
    let (incoming_only,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM transparent_accounts
            WHERE account = ?1 AND xvk IS NULL AND ivk IS NOT NULL)
        OR EXISTS(SELECT 1 FROM sapling_accounts
            WHERE account = ?1 AND xvk IS NULL AND ivk IS NOT NULL)
        OR EXISTS(SELECT 1 FROM orchard_accounts
            WHERE account = ?1 AND xvk IS NULL AND ivk IS NOT NULL)",
    )
    .bind(account)
    .fetch_one(&mut *connection)
    .await?;
    Ok(incoming_only)
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

    // Also clean up when the connection does not enforce foreign keys.
    sqlx::query("DELETE FROM swaps WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;

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

    crate::sync::chain_log::delete_account_data(&mut tx, account).await?;
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
    debug!(
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
    let mut balance = PoolBalance(vec![0, 0, 0, 0]);
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

/// The transaction row with its display joins (category, asset, user memo,
/// first matching contact), followed by the given `WHERE` clause. Every
/// transaction query selects these columns and is mapped by `row_to_tx`.
macro_rules! tx_select {
    ($where:literal) => {
        concat!(
            "SELECT t.id_tx, t.txid, t.height, t.time, t.value, t.tpe, c.name, t.zsa_value, t.price, t.asset_id,
            a.asset_name, a.asset_desc_hash,
            um.user_memo as memo,
            (um.user_memo IS NOT NULL AND um.user_memo != '') as is_user_memo,
            oc.contact_name, t.account, t.fee
            FROM transactions t
            LEFT JOIN categories c ON c.id_category = t.category
            LEFT JOIN assets a ON t.asset_id = a.id_asset
            LEFT JOIN user_memos um ON um.id_tx = t.id_tx AND um.account = t.account
            LEFT JOIN (
                SELECT DISTINCT o.tx, ct.name as contact_name
                FROM outputs o
                JOIN contact_addresses ca ON o.address = ca.receiver AND o.pool = ca.pool
                JOIN contacts ct ON ca.contact_id = ct.id_contact
            ) oc ON oc.tx = t.id_tx ",
            $where
        )
    };
}

fn row_to_tx(row: SqliteRow) -> Tx {
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
    let memo: Option<String> = row.get(12);
    let is_user_memo: bool = row.get(13);
    let contact_name: Option<String> = row.get(14);
    let account: u32 = row.get(15);
    let fee: i64 = row.get(16);
    Tx {
        id,
        account,
        txid,
        height,
        time,
        value,
        fee,
        tpe,
        category,
        zsa_value,
        asset_id,
        asset_display: crate::account::asset_display(asset_id, asset_name, asset_desc_hash),
        price,
        memo,
        is_user_memo,
        contact_name,
    }
}

/// An account's transactions, latest first.
pub async fn fetch_txs(connection: &mut SqliteConnection, account: u32) -> Result<Vec<Tx>> {
    fetch_txs_from(connection, account, 0).await
}

/// An account's transactions at or above `min_height`, latest first.
pub async fn fetch_txs_from(
    connection: &mut SqliteConnection,
    account: u32,
    min_height: u32,
) -> Result<Vec<Tx>> {
    let transactions = sqlx::query(tx_select!(
        "WHERE t.account = ?1 AND t.height >= ?2 ORDER BY t.height DESC"
    ))
    .bind(account)
    .bind(min_height)
    .map(row_to_tx)
    .fetch_all(&mut *connection)
    .await?;
    Ok(transactions)
}

/// One of an account's transactions by its txid, in stored (little-endian)
/// byte order.
pub async fn fetch_tx_by_txid(
    connection: &mut SqliteConnection,
    account: u32,
    txid: &[u8],
) -> Result<Option<Tx>> {
    let transaction = sqlx::query(tx_select!("WHERE t.account = ?1 AND t.txid = ?2"))
        .bind(account)
    .bind(txid)
    .map(row_to_tx)
    .fetch_optional(&mut *connection)
    .await?;
    Ok(transaction)
}

/// Transactions by row id, across accounts, in no particular order.
pub async fn fetch_txs_by_ids(connection: &mut SqliteConnection, ids: &[u32]) -> Result<Vec<Tx>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    // Only the placeholder count is dynamic; the ids themselves are bound.
    let placeholders = vec!["?"; ids.len()].join(", ");
    let sql = format!("{} WHERE t.id_tx IN ({placeholders})", tx_select!(""));
    let mut query = sqlx::query(sqlx::AssertSqlSafe(sql));
    for id in ids {
        query = query.bind(*id);
    }
    Ok(query
        .map(row_to_tx)
        .fetch_all(&mut *connection)
        .await?)
}

pub async fn get_memos(pool: &mut SqliteConnection, account: u32) -> Result<Vec<Memo>> {
    let memos = sqlx::query(
        "SELECT COALESCE(m.id_memo, 0) as id_memo,
            t.height,
            t.id_tx as tx,
            COALESCE(m.pool, 0) as pool,
            COALESCE(m.vout, 0) as vout,
            COALESCE(m.note, CASE WHEN t.value + t.fee >= 0 THEN 0 ELSE NULL END) as note,
            t.time,
            COALESCE(NULLIF(um.user_memo, ''), m.memo_text) as memo_text,
            COALESCE(m.memo_bytes, X'') as memo_bytes,
            (um.user_memo IS NOT NULL AND um.user_memo != '') as is_user_memo,
            t.txid
        FROM transactions t
        LEFT JOIN memos m ON m.tx = t.id_tx AND m.account = t.account
        LEFT JOIN user_memos um ON um.id_tx = t.id_tx AND um.account = t.account
        WHERE t.account = ?
          AND (m.id_memo IS NOT NULL OR (um.user_memo IS NOT NULL AND um.user_memo != ''))
        ORDER BY t.height DESC",
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
        "SELECT COALESCE(m.id_memo, 0) as id_memo,
            t.height,
            t.id_tx as tx,
            COALESCE(m.pool, 0) as pool,
            COALESCE(m.vout, 0) as vout,
            COALESCE(m.note, CASE WHEN t.value + t.fee >= 0 THEN 0 ELSE NULL END) as note,
            t.time,
            COALESCE(NULLIF(um.user_memo, ''), m.memo_text) as memo_text,
            COALESCE(m.memo_bytes, X'') as memo_bytes,
            (um.user_memo IS NOT NULL AND um.user_memo != '') as is_user_memo,
            t.txid
        FROM transactions t
        LEFT JOIN memos m ON m.tx = t.id_tx AND m.account = t.account
        LEFT JOIN user_memos um ON um.id_tx = t.id_tx AND um.account = t.account
        WHERE t.account = ?1 AND t.txid = ?2",
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
    let is_user_memo: bool = row.get(9);
    let txid: Vec<u8> = row.get(10);
    Memo {
        id,
        id_tx: tx,
        txid,
        id_note: note,
        height,
        pool,
        vout,
        time,
        memo: memo_text,
        memo_bytes,
        is_user_memo,
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

/// The notes a transaction created for an account, spent or not.
pub async fn get_tx_notes(
    connection: &mut SqliteConnection,
    account: u32,
    id_tx: u32,
) -> Result<Vec<TxNote>> {
    let notes = sqlx::query(
        "SELECT n.id_note, n.height, n.pool, n.tx, n.scope, n.diversifier, n.diversifier_index, n.value, n.locked,
        m.memo_text, n.id_asset, a.asset_name, a.asset_desc_hash
        FROM notes n
        LEFT JOIN memos m ON n.id_note = m.note
        LEFT JOIN assets a ON n.id_asset = a.id_asset
        WHERE n.account = ?1 AND n.tx = ?2",
    )
    .bind(account)
    .bind(id_tx)
    .map(row_to_note)
    .fetch_all(&mut *connection)
    .await?;
    Ok(notes)
}

/// The notes a transaction spent for an account.
pub async fn get_spent_notes(
    connection: &mut SqliteConnection,
    account: u32,
    id_tx: u32,
) -> Result<Vec<TxNote>> {
    let notes = sqlx::query(
        "SELECT n.id_note, n.height, n.pool, n.tx, n.scope, n.diversifier, n.diversifier_index, n.value, n.locked,
        m.memo_text, n.id_asset, a.asset_name, a.asset_desc_hash
        FROM spends s
        JOIN notes n ON s.id_note = n.id_note
        LEFT JOIN memos m ON n.id_note = m.note
        LEFT JOIN assets a ON n.id_asset = a.id_asset
        WHERE s.account = ?1 AND s.tx = ?2
        ORDER BY s.id_note",
    )
    .bind(account)
    .bind(id_tx)
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
            FROM notes WHERE account = ?1 AND pool IN (1, 2) AND id_asset IS NULL
            UNION ALL
            SELECT n.pool, n.scope, n.diversifier_index, s.tx, s.value
            FROM spends s
            JOIN notes n ON s.id_note = n.id_note AND s.account = n.account
            WHERE s.pool IN (1, 2) AND n.account = ?1 AND n.id_asset IS NULL
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

/// Re-encrypts the database at `db_filepath` under `new_password` (empty for
/// no encryption). The copy is exported to `<db_filepath>.tmp` beside the
/// database, then renamed over it: the same directory, so the rename never
/// crosses a filesystem and no other directory has to exist (#1370). A
/// stale copy from an earlier failure is truncated; the copy is removed if
/// the export fails.
pub async fn change_db_password(
    db_filepath: &str,
    old_password: &str,
    new_password: &str,
) -> Result<()> {
    let tmp_db_filepath = format!("{db_filepath}.tmp");
    File::create(&tmp_db_filepath)?;
    let result = export_db(db_filepath, &tmp_db_filepath, old_password, new_password).await;
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp_db_filepath);
    }
    result
}

async fn export_db(
    db_filepath: &str,
    tmp_db_filepath: &str,
    old_password: &str,
    new_password: &str,
) -> Result<()> {
    let mut options = SqliteConnectOptions::new().filename(db_filepath);
    if !old_password.is_empty() {
        let escaped_old_password = old_password.replace('\'', "''");
        options = options.pragma("key", format!("'{escaped_old_password}'"));
    }

    let mut connection = SqliteConnection::connect_with(&options).await?;
    let escaped_password = new_password.replace('\'', "''");
    // Audited for sqlx 0.9's SqlSafeStr: ATTACH ... KEY takes no bind
    // parameters, the path is constructed here, and the password has its
    // quotes doubled directly above.
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "ATTACH DATABASE '{}' AS new_db KEY '{}'",
        tmp_db_filepath, escaped_password
    )))
    .execute(&mut connection)
    .await?;
    sqlx::query("SELECT sqlcipher_export('new_db')")
        .execute(&mut connection)
        .await?;
    sqlx::query("DETACH DATABASE new_db")
        .execute(&mut connection)
        .await?;

    // Explicitly close the connection before file operations to ensure Windows
    // file handles are fully released. Relying on Drop (sqlite3_close_v2) may not
    // be synchronous enough on Windows, causing remove_file/rename to fail.
    connection.close().await?;

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

/// Set or clear the user's note on the account's transaction `txid`. The
/// transaction is named by txid, not row id: a rewind since the caller saw
/// it may have deleted the row and a later sync given its id to another
/// transaction. The note takes the transaction's height, so a rewind past
/// it trims the note too; without the transaction there is nothing to note
/// and the call does nothing.
pub async fn set_user_memo(
    connection: &mut crate::sync::session::Held<'_>,
    account: u32,
    txid: &[u8],
    memo: Option<String>,
) -> Result<()> {
    match memo {
        Some(memo) if !memo.is_empty() => {
            sqlx::query(
                "INSERT INTO user_memos(account, id_tx, height, user_memo)
                SELECT account, id_tx, height, ?3 FROM transactions
                WHERE account = ?1 AND txid = ?2
                ON CONFLICT(account, id_tx) DO UPDATE SET user_memo = excluded.user_memo",
            )
            .bind(account)
            .bind(txid)
            .bind(&memo)
            .execute(&mut **connection)
            .await?;
        }
        _ => {
            sqlx::query(
                "DELETE FROM user_memos WHERE account = ?1 AND id_tx IN
                (SELECT id_tx FROM transactions WHERE account = ?1 AND txid = ?2)",
            )
            .bind(account)
            .bind(txid)
            .execute(&mut **connection)
            .await?;
        }
    }
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

#[cfg(test)]
mod change_db_password_tests {
    use super::*;

    /// A fresh database path under the system temp dir, with no leftovers.
    fn db_path(name: &str) -> String {
        let path = std::env::temp_dir().join(format!("{name}_{}.db", std::process::id()));
        let path = path.to_str().unwrap().to_string();
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{path}.tmp"));
        path
    }

    async fn open(path: &str, password: &str) -> Result<SqliteConnection> {
        let mut options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        if !password.is_empty() {
            options = options.pragma("key", format!("'{password}'"));
        }
        Ok(SqliteConnection::connect_with(&options).await?)
    }

    async fn marker(connection: &mut SqliteConnection) -> Result<String> {
        let (v,): (String,) = sqlx::query_as("SELECT v FROM t")
            .fetch_one(connection)
            .await?;
        Ok(v)
    }

    fn copy_exists(path: &str) -> bool {
        std::path::Path::new(&format!("{path}.tmp")).exists()
    }

    /// The copy goes beside the database, so the change needs no other
    /// directory and leaves nothing behind (#1370). The new password opens
    /// the result; the old one does not.
    #[tokio::test]
    async fn re_encrypts_in_place() -> Result<()> {
        let path = db_path("change_db_password");
        let mut db = open(&path, "old").await?;
        sqlx::raw_sql("CREATE TABLE t(v TEXT); INSERT INTO t VALUES ('kept');")
            .execute(&mut db)
            .await?;
        db.close().await?;

        change_db_password(&path, "old", "new").await?;

        assert!(!copy_exists(&path), "the copy is renamed over the database");
        let mut db = open(&path, "new").await?;
        assert_eq!(marker(&mut db).await?, "kept");
        db.close().await?;
        let mut db = open(&path, "old").await?;
        assert!(
            marker(&mut db).await.is_err(),
            "the old password no longer opens the database"
        );
        let _ = std::fs::remove_file(&path);
        Ok(())
    }

    /// A wrong old password fails the export; the database is untouched and
    /// the half-written copy is removed.
    #[tokio::test]
    async fn failed_export_leaves_no_copy() -> Result<()> {
        let path = db_path("change_db_password_fail");
        let mut db = open(&path, "old").await?;
        sqlx::raw_sql("CREATE TABLE t(v TEXT); INSERT INTO t VALUES ('kept');")
            .execute(&mut db)
            .await?;
        db.close().await?;

        assert!(change_db_password(&path, "wrong", "new").await.is_err());

        assert!(!copy_exists(&path), "the copy is removed on failure");
        let mut db = open(&path, "old").await?;
        assert_eq!(marker(&mut db).await?, "kept", "the database is untouched");
        let _ = std::fs::remove_file(&path);
        Ok(())
    }
}

#[cfg(test)]
mod incoming_key_schema_tests {
    use super::*;

    type SaplingRow = (u32, Option<Vec<u8>>, Option<Vec<u8>>, Vec<u8>, Option<Vec<u8>>);
    type OrchardRow = (u32, Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>);

    async fn rows(db: &mut SqliteConnection) -> (Vec<SaplingRow>, Vec<OrchardRow>) {
        let s = sqlx::query_as("SELECT account, xsk, xvk, address, ivk FROM sapling_accounts")
            .fetch_all(&mut *db)
            .await
            .unwrap();
        let o = sqlx::query_as("SELECT account, xsk, xvk, ivk FROM orchard_accounts")
            .fetch_all(&mut *db)
            .await
            .unwrap();
        (s, o)
    }

    /// A wallet from before incoming-only accounts: full viewing keys NOT
    /// NULL and no incoming-key column. Upgrading keeps every row and lets
    /// an incoming-only row in; upgrading again changes nothing.
    #[tokio::test]
    async fn upgrade_allows_incoming_only_rows_and_keeps_existing_ones() -> Result<()> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        sqlx::raw_sql(
            "CREATE TABLE transparent_accounts(account INTEGER PRIMARY KEY, xsk BLOB, xvk BLOB);
            CREATE TABLE sapling_accounts(account INTEGER PRIMARY KEY, xsk BLOB, xvk BLOB NOT NULL,
                address BLOB NOT NULL DEFAULT(''));
            CREATE TABLE orchard_accounts(account INTEGER PRIMARY KEY, xsk BLOB, xvk BLOB NOT NULL);
            INSERT INTO transparent_accounts VALUES (1, x'01', x'02');
            INSERT INTO sapling_accounts VALUES (1, x'03', x'04', x'05');
            INSERT INTO orchard_accounts VALUES (1, x'06', x'07');",
        )
        .execute(&mut db)
        .await?;

        create_schema(&mut db).await?;
        let upgraded = rows(&mut db).await;
        assert_eq!(
            upgraded,
            (
                vec![(1, Some(vec![3]), Some(vec![4]), vec![5], None)],
                vec![(1, Some(vec![6]), Some(vec![7]), None)]
            )
        );

        sqlx::raw_sql(
            "INSERT INTO transparent_accounts(account, ivk) VALUES (2, x'08');
            INSERT INTO sapling_accounts(account, ivk) VALUES (2, x'09');
            INSERT INTO orchard_accounts(account, ivk) VALUES (2, x'0a');",
        )
        .execute(&mut db)
        .await?;

        create_schema(&mut db).await?;
        let (s, o) = rows(&mut db).await;
        assert_eq!(s.len(), 2);
        assert_eq!(o.len(), 2);
        assert_eq!(s[0], upgraded.0[0]);
        assert_eq!(o[1], (2, None, None, Some(vec![10])));
        Ok(())
    }
}

#[cfg(test)]
mod swap_schema_tests {
    use super::*;

    async fn insert_swap(
        db: &mut SqliteConnection,
        account: i64,
        address: &str,
        amount: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO swaps(account, origin_asset, destination_asset, swap_type,
            amount, slippage_tolerance, recipient, refund_to, deadline, amount_in,
            amount_out, deposit_address, quote_response)
            VALUES (?, 'nep141:zec.omft.near', 'eth', 'EXACT_OUTPUT', ?, 100,
            'recipient', 'refund', '2026-10-10T12:00:00Z', '100000000', ?, ?, '{}')",
        )
        .bind(account)
        .bind(amount)
        .bind(amount)
        .bind(address)
        .execute(db)
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn upgrade_preserves_history_and_account_deletion_is_scoped() -> Result<()> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        sqlx::query(
            "INSERT INTO accounts(id_account, name, aindex, dindex, def_dindex,
            birth, position, use_internal, hidden, saved) VALUES
            (1, 'one', 0, 0, 0, 0, 0, 0, 0, 0), (2, 'two', 1, 0, 0, 0, 1, 0, 0, 0)",
        )
        .execute(&mut db)
        .await?;
        let amount = "123456789012345678901234567890";
        insert_swap(&mut db, 1, "deposit-one", amount).await?;
        insert_swap(&mut db, 2, "deposit-two", "1").await?;
        sqlx::query("UPDATE swaps SET status = 'FUTURE_PROVIDER_STATE' WHERE account = 1")
            .execute(&mut db)
            .await?;
        create_schema(&mut db).await?;
        let row: (String, String) =
            sqlx::query_as("SELECT amount_out, status FROM swaps WHERE account = 1")
                .fetch_one(&mut db)
                .await?;
        assert_eq!(row, (amount.into(), "FUTURE_PROVIDER_STATE".into()));
        assert!(insert_swap(&mut db, 1, "deposit-one", "1").await.is_err());
        for invalid in ["", "0", "000", "-1", "1.2", "1e8"] {
            assert!(insert_swap(&mut db, 1, "invalid", invalid).await.is_err());
        }
        delete_account(&mut db, 1).await?;
        let remaining: Vec<(i64,)> = sqlx::query_as("SELECT account FROM swaps")
            .fetch_all(&mut db)
            .await?;
        assert_eq!(remaining, vec![(2,)]);
        Ok(())
    }
}

#[cfg(test)]
mod header_tests {
    use super::*;

    #[tokio::test]
    async fn legacy_headers_fan_out_and_migration_is_idempotent() -> Result<()> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        sqlx::raw_sql("CREATE TABLE accounts(id_account INTEGER PRIMARY KEY);
            INSERT INTO accounts VALUES (1), (2);
            CREATE TABLE headers(height INTEGER PRIMARY KEY, hash BLOB NOT NULL, time INTEGER NOT NULL);
            INSERT INTO headers VALUES (100, X'01', 123), (200, X'02', 456);")
            .execute(&mut db).await?;
        migrate_headers(&mut db).await?;
        migrate_headers(&mut db).await?;
        let rows: Vec<(u32, u32, Vec<u8>, u32)> = sqlx::query_as(
            "SELECT account, height, hash, time FROM headers ORDER BY account, height",
        )
        .fetch_all(&mut db)
        .await?;
        assert_eq!(
            rows,
            vec![
                (1, 100, vec![1], 123),
                (1, 200, vec![2], 456),
                (2, 100, vec![1], 123),
                (2, 200, vec![2], 456)
            ]
        );
        Ok(())
    }
}
