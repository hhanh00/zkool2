//! Chain log: the single writer of the tables that record what the chain
//! says about an account. Transactions, notes, spends, witnesses, headers,
//! sync checkpoints, outputs and memos are only ever written through this
//! module, plus the chain-derived `assets` rows that notes refer to.
//!
//! The functions speak in chain events (`record_note`, `record_spend`,
//! `checkpoint`, ...) and hide the table layout, the join from a txid to its
//! row, the lock released when a note is spent, the header relevance rule and
//! the per-pool checkpoint. The transparent scanner and the shielded message
//! handler are two adapters producing the same events; the details pass, the
//! backup import and the reorg rewind are plain callers.
//!
//! # Transactions
//!
//! No function here opens or commits a database transaction. Every function
//! takes a connection and works equally inside a caller-held transaction
//! through `&mut *db_tx`. The caller owns the boundary, because that boundary
//! is what keeps the database consistent when a sync is aborted: the shielded
//! writer commits once per block batch, the transparent scanner once per
//! address. The one function that must be atomic across every table,
//! [`trim_to`], takes the caller's transaction as a parameter so it cannot be
//! run half-way.
//!
//! Row types are full table rows with the wallet metadata optional, so the
//! backup import goes through the same functions as the sync paths.

use anyhow::{anyhow, ensure, Result};
use sqlx::{sqlite::SqliteRow, Row, Sqlite, SqliteConnection, Transaction};
use tracing::debug;
use zcash_protocol::consensus::{NetworkUpgrade, Parameters};
use zcash_trees::types::{BlockHeader, Issuance};

use crate::account::has_pool;
use crate::api::coin::Network;

/// A transaction row. The sync paths fill the chain fields and leave the
/// wallet metadata at its defaults; the backup import fills everything.
#[derive(Clone, Debug, Default)]
pub struct TransactionRow<'a> {
    pub txid: &'a [u8],
    pub height: u32,
    /// 0 when the block time is not known yet; see [`set_block_time`].
    pub time: u32,
    pub details: bool,
    pub tpe: Option<u8>,
    pub value: i64,
    pub fee: i64,
    pub price: Option<f64>,
    pub category: Option<u32>,
}

impl<'a> TransactionRow<'a> {
    /// A transaction as the chain reports it, before the details pass.
    pub fn chain(txid: &'a [u8], height: u32, time: u32) -> Self {
        TransactionRow {
            txid,
            height,
            time,
            ..Default::default()
        }
    }
}

/// A note row. A transparent note is keyed by its outpoint (stored in
/// `nullifier`) and refers to the receiving address; a shielded note carries
/// its commitment and witness position.
#[derive(Clone, Debug, Default)]
pub struct NoteRow<'a> {
    pub id_tx: u32,
    pub height: u32,
    pub pool: u8,
    pub scope: Option<u8>,
    /// Shielded nullifier, or the serialized outpoint for a transparent note.
    pub nullifier: &'a [u8],
    pub value: i64,
    pub cmx: Option<&'a [u8]>,
    pub taddress: Option<u32>,
    pub position: Option<u32>,
    pub diversifier: Option<&'a [u8]>,
    pub rcm: Option<&'a [u8]>,
    pub rho: Option<&'a [u8]>,
    pub locked: bool,
    pub id_asset: Option<u32>,
    pub diversifier_index: Option<i64>,
}

/// How a caller names an existing note.
#[derive(Clone, Copy, Debug)]
pub enum NoteKey<'a> {
    Id(u32),
    /// Shielded nullifier or transparent outpoint.
    Nullifier(&'a [u8]),
    Cmx(&'a [u8]),
}

#[derive(Clone, Debug)]
pub struct SpendRow<'a> {
    pub note: NoteKey<'a>,
    pub id_tx: u32,
    pub height: u32,
    pub pool: u8,
    /// Negative: the note's value leaving the account.
    pub value: i64,
}

#[derive(Clone, Debug)]
pub struct OutputRow<'a> {
    pub id_tx: u32,
    pub height: u32,
    pub pool: u8,
    pub vout: u32,
    pub value: i64,
    pub address: &'a str,
}

#[derive(Clone, Debug)]
pub struct MemoRow<'a> {
    pub id_tx: u32,
    pub height: u32,
    pub pool: u8,
    pub vout: u32,
    pub note: Option<u32>,
    pub output: Option<u32>,
    pub text: Option<&'a str>,
    pub bytes: &'a [u8],
}

/// Summary written by the details pass once a transaction is decrypted.
#[derive(Clone, Debug)]
pub struct TransactionDetails {
    pub tpe: u8,
    pub value: i64,
    pub zsa_value: i64,
    pub asset_id: Option<i32>,
}

/// An unspent shielded note that has no witness at its pool's checkpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingWitness {
    pub account: u32,
    pub pool: u8,
    pub height: u32,
    pub value: u64,
    pub checkpoint: u32,
}

// ---------------------------------------------------------------------------
// Transactions

/// Record a transaction. Returns its row id, or `None` when the account
/// already has it (the transparent and shielded scans can both see a
/// transaction, and an address can be scanned again).
pub async fn record_transaction(
    conn: &mut SqliteConnection,
    account: u32,
    tx: &TransactionRow<'_>,
) -> Result<Option<u32>> {
    let id = sqlx::query_scalar(
        "INSERT INTO transactions
        (account, txid, height, time, details, tpe, value, fee, price, category)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT DO NOTHING RETURNING id_tx",
    )
    .bind(account)
    .bind(tx.txid)
    .bind(tx.height)
    .bind(tx.time)
    .bind(tx.details)
    .bind(tx.tpe)
    .bind(tx.value)
    .bind(tx.fee)
    .bind(tx.price)
    .bind(tx.category)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(id)
}

/// Row id and height of the account's transaction with this txid.
pub async fn transaction_ref(
    conn: &mut SqliteConnection,
    account: u32,
    txid: &[u8],
) -> Result<(u32, u32)> {
    find_transaction(conn, account, txid)
        .await?
        .ok_or_else(|| anyhow!("account {account} has no transaction {}", hex::encode(txid)))
}

/// [`transaction_ref`] for a caller to whom a missing transaction is a
/// state, not an error: one rewound since the caller saw it.
pub async fn find_transaction(
    conn: &mut SqliteConnection,
    account: u32,
    txid: &[u8],
) -> Result<Option<(u32, u32)>> {
    Ok(
        sqlx::query_as("SELECT id_tx, height FROM transactions WHERE account = ? AND txid = ?")
            .bind(account)
            .bind(txid)
            .fetch_optional(&mut *conn)
            .await?,
    )
}

/// Set the block time of the account's transactions at `height`. With
/// `only_missing`, transactions that already have a time are left alone.
pub async fn set_block_time(
    conn: &mut SqliteConnection,
    account: u32,
    height: u32,
    time: u32,
    only_missing: bool,
) -> Result<()> {
    let sql = if only_missing {
        "UPDATE transactions SET time = ? WHERE height = ? AND account = ? AND time = 0"
    } else {
        "UPDATE transactions SET time = ? WHERE height = ? AND account = ?"
    };
    sqlx::query(sql)
        .bind(time)
        .bind(height)
        .bind(account)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Store the decrypted summary of a transaction and mark its details done.
pub async fn set_transaction_details(
    conn: &mut SqliteConnection,
    id_tx: u32,
    details: &TransactionDetails,
) -> Result<()> {
    sqlx::query(
        "UPDATE transactions SET details = TRUE, tpe = ?, value = ?, zsa_value = ?, asset_id = ?
        WHERE id_tx = ?",
    )
    .bind(details.tpe)
    .bind(details.value)
    .bind(details.zsa_value)
    .bind(details.asset_id)
    .bind(id_tx)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

pub async fn set_transaction_fee(conn: &mut SqliteConnection, id_tx: u32, fee: u64) -> Result<()> {
    sqlx::query("UPDATE transactions SET fee = ? WHERE id_tx = ?")
        .bind(fee as i64)
        .bind(id_tx)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Set or clear the budget category of a transaction.
pub async fn set_transaction_category(
    conn: &mut SqliteConnection,
    id_tx: u32,
    category: Option<u32>,
) -> Result<()> {
    sqlx::query("UPDATE transactions SET category = ?2 WHERE id_tx = ?1")
        .bind(id_tx)
        .bind(category)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Detach every transaction from a category that is being deleted.
pub async fn clear_category(conn: &mut SqliteConnection, category: u32) -> Result<()> {
    sqlx::query("UPDATE transactions SET category = NULL WHERE category = ?1")
        .bind(category)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Set or clear the fiat price recorded for a transaction.
pub async fn set_transaction_price(
    conn: &mut SqliteConnection,
    id_tx: u32,
    price: Option<f64>,
) -> Result<()> {
    sqlx::query("UPDATE transactions SET price = ?2 WHERE id_tx = ?1")
        .bind(id_tx)
        .bind(price)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Multiply every recorded price by `rate`, for a currency change.
pub async fn scale_transaction_prices(conn: &mut SqliteConnection, rate: f64) -> Result<()> {
    sqlx::query("UPDATE transactions SET price = price * ?1 WHERE price IS NOT NULL")
        .bind(rate)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Copy the price and category the wallet recorded at broadcast onto the
/// mined transactions, and drop pending entries that never got mined.
pub async fn apply_pending_metadata(
    conn: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<()> {
    sqlx::query(
        "WITH upd AS (SELECT t.id_tx, p.price, p.category
        FROM pending_txs p JOIN transactions t ON p.txid = t.txid
        WHERE t.account = ?1)
        UPDATE transactions SET price = upd.price, category = upd.category
        FROM upd WHERE upd.id_tx = transactions.id_tx",
    )
    .bind(account)
    .execute(&mut *conn)
    .await?;
    // pending txs that could not be merged for more than 100 blocks were
    // probably never mined
    sqlx::query("DELETE FROM pending_txs WHERE account = ?1 AND height < ?2")
        .bind(account)
        .bind(height.saturating_sub(100))
        .execute(&mut *conn)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Assets

/// An asset row: an issuance seen on chain, one this wallet issued, or one
/// restored from a backup.
#[derive(Clone, Debug)]
pub struct AssetRow<'a> {
    pub desc_hash: &'a [u8],
    pub ik: &'a [u8],
    pub base: &'a [u8],
    pub name: Option<&'a str>,
    pub finalized: bool,
    pub first_seen_height: u32,
}

/// Record an asset; an asset already known by description hash and issuer
/// key is left unchanged.
pub async fn record_asset(conn: &mut SqliteConnection, asset: &AssetRow<'_>) -> Result<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO assets(asset_desc_hash, ik, asset_base, asset_name, finalized, first_seen_height)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(asset.desc_hash)
    .bind(asset.ik)
    .bind(asset.base)
    .bind(asset.name)
    .bind(asset.finalized)
    .bind(asset.first_seen_height)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Row id of the asset with this description hash and issuer key.
pub async fn asset_id(
    conn: &mut SqliteConnection,
    desc_hash: &[u8],
    ik: &[u8],
) -> Result<Option<u32>> {
    let id =
        sqlx::query_scalar("SELECT id_asset FROM assets WHERE asset_desc_hash = ?1 AND ik = ?2")
            .bind(desc_hash)
            .bind(ik)
            .fetch_optional(&mut *conn)
            .await?;
    Ok(id)
}

/// Set the display name of an asset; fails when no asset has that id. An
/// empty name clears it.
pub async fn set_asset_name(conn: &mut SqliteConnection, id_asset: i64, name: &str) -> Result<()> {
    let r = sqlx::query("UPDATE assets SET asset_name = ?1 WHERE id_asset = ?2")
        .bind(name)
        .bind(id_asset)
        .execute(&mut *conn)
        .await?;
    ensure!(r.rows_affected() > 0, "No asset with id_asset={id_asset}");
    Ok(())
}

/// Record an asset issuance seen on chain.
pub async fn record_issuance(conn: &mut SqliteConnection, iss: &Issuance) -> Result<()> {
    record_asset(
        conn,
        &AssetRow {
            desc_hash: &iss.asset_desc_hash,
            ik: &iss.ik,
            base: &iss.asset_base,
            name: None,
            finalized: iss.finalized,
            first_seen_height: iss.height,
        },
    )
    .await?;
    if iss.finalized {
        sqlx::query("UPDATE assets SET finalized = TRUE WHERE asset_desc_hash = ?1 AND ik = ?2")
            .bind(&iss.asset_desc_hash)
            .bind(&iss.ik)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

/// Make sure an asset discovered through a decrypted note has a row, and
/// return its id. An issuance stores the real description hash and issuer
/// key; a transfer only reveals the asset base, which is used as the
/// placeholder key until an issuance is seen.
pub async fn asset_id_for_base(
    conn: &mut SqliteConnection,
    asset_base: &[u8],
    height: u32,
) -> Result<Option<u32>> {
    if asset_base.len() != 32 || asset_base == [0u8; 32] {
        return Ok(None);
    }
    let mut ik_placeholder = vec![0u8; 33];
    ik_placeholder[1..].copy_from_slice(&asset_base[..32]);
    sqlx::query(
        "INSERT OR IGNORE INTO assets(asset_desc_hash, ik, asset_base, finalized, first_seen_height)
         VALUES (?1, ?2, ?3, 0, ?4)",
    )
    .bind(asset_base)
    .bind(&ik_placeholder)
    .bind(asset_base)
    .bind(height)
    .execute(&mut *conn)
    .await?;
    let id = sqlx::query_scalar("SELECT id_asset FROM assets WHERE asset_base = ?")
        .bind(asset_base)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(id)
}

// ---------------------------------------------------------------------------
// Notes, spends, witnesses

/// Record a note received by the account. Returns its row id, or `None` when
/// the account already has a note with this nullifier.
pub async fn record_note(
    conn: &mut SqliteConnection,
    account: u32,
    note: &NoteRow<'_>,
) -> Result<Option<u32>> {
    let id = sqlx::query_scalar(
        "INSERT INTO notes
        (account, tx, height, pool, scope, nullifier, value, cmx, taddress, position,
         diversifier, rcm, rho, locked, id_asset, diversifier_index)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT DO NOTHING RETURNING id_note",
    )
    .bind(account)
    .bind(note.id_tx)
    .bind(note.height)
    .bind(note.pool)
    .bind(note.scope)
    .bind(note.nullifier)
    .bind(note.value)
    .bind(note.cmx)
    .bind(note.taddress)
    .bind(note.position)
    .bind(note.diversifier)
    .bind(note.rcm)
    .bind(note.rho)
    .bind(note.locked)
    .bind(note.id_asset)
    .bind(note.diversifier_index)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(id)
}

/// Row id and value of the note a key names, if the account has it.
pub async fn find_note(
    conn: &mut SqliteConnection,
    account: u32,
    key: NoteKey<'_>,
) -> Result<Option<(u32, i64)>> {
    let row = match key {
        NoteKey::Id(id) => {
            sqlx::query_as("SELECT id_note, value FROM notes WHERE account = ? AND id_note = ?")
                .bind(account)
                .bind(id)
                .fetch_optional(&mut *conn)
                .await?
        }
        NoteKey::Nullifier(nf) => {
            sqlx::query_as("SELECT id_note, value FROM notes WHERE account = ? AND nullifier = ?")
                .bind(account)
                .bind(nf)
                .fetch_optional(&mut *conn)
                .await?
        }
        NoteKey::Cmx(cmx) => {
            sqlx::query_as("SELECT id_note, value FROM notes WHERE account = ? AND cmx = ?")
                .bind(account)
                .bind(cmx)
                .fetch_optional(&mut *conn)
                .await?
        }
    };
    Ok(row)
}

/// Record that a note was spent by a transaction, and release the lock the
/// wallet put on it at broadcast (a no-op when it was not locked). Returns
/// false when the spend was already recorded.
pub async fn record_spend(
    conn: &mut SqliteConnection,
    account: u32,
    spend: &SpendRow<'_>,
) -> Result<bool> {
    let (id_note, _) = find_note(conn, account, spend.note)
        .await?
        .ok_or_else(|| anyhow!("account {account} has no note {:?} to spend", spend.note))?;
    let r = sqlx::query(
        "INSERT INTO spends (id_note, tx, height, account, pool, value)
        VALUES (?, ?, ?, ?, ?, ?) ON CONFLICT DO NOTHING",
    )
    .bind(id_note)
    .bind(spend.id_tx)
    .bind(spend.height)
    .bind(account)
    .bind(spend.pool)
    .bind(spend.value)
    .execute(&mut *conn)
    .await?;
    sqlx::query("UPDATE notes SET locked = FALSE WHERE id_note = ?")
        .bind(id_note)
        .execute(&mut *conn)
        .await?;
    Ok(r.rows_affected() == 1)
}

/// Set the lock on one of the account's notes.
pub async fn set_note_lock(
    conn: &mut SqliteConnection,
    account: u32,
    id_note: u32,
    locked: bool,
) -> Result<()> {
    sqlx::query("UPDATE notes SET locked = ? WHERE account = ? AND id_note = ?")
        .bind(locked)
        .bind(account)
        .bind(id_note)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Lock every note the account received above `height`.
pub async fn lock_notes_above(
    conn: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<()> {
    sqlx::query("UPDATE notes SET locked = TRUE WHERE account = ?1 AND height > ?2")
        .bind(account)
        .bind(height)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Flip the lock on every note of the account.
pub async fn toggle_all_locks(conn: &mut SqliteConnection, account: u32) -> Result<()> {
    sqlx::query("UPDATE notes SET locked = NOT locked WHERE account = ?1")
        .bind(account)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Unlock every note of the account.
pub async fn unlock_all(conn: &mut SqliteConnection, account: u32) -> Result<()> {
    sqlx::query("UPDATE notes SET locked = FALSE WHERE account = ?1")
        .bind(account)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Lock the account's notes with these nullifiers (or outpoints) so a later
/// plan cannot select them before the spending transaction is mined.
pub async fn lock_notes(
    conn: &mut SqliteConnection,
    account: u32,
    nullifiers: &[Vec<u8>],
) -> Result<()> {
    for nf in nullifiers {
        sqlx::query("UPDATE notes SET locked = TRUE WHERE account = ? AND nullifier = ?")
            .bind(account)
            .bind(nf)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

/// Record a note's witness at a checkpoint height.
pub async fn record_witness(
    conn: &mut SqliteConnection,
    account: u32,
    note: NoteKey<'_>,
    height: u32,
    witness: &[u8],
) -> Result<()> {
    let (id_note, _) = find_note(conn, account, note)
        .await?
        .ok_or_else(|| anyhow!("account {account} has no note {note:?} for a witness"))?;
    let r =
        sqlx::query("INSERT INTO witnesses (account, note, height, witness) VALUES (?, ?, ?, ?)")
            .bind(account)
            .bind(id_note)
            .bind(height)
            .bind(witness)
            .execute(&mut *conn)
            .await?;
    ensure!(
        r.rows_affected() == 1,
        "witness for note {id_note} of account {account} at {height} was not stored"
    );
    Ok(())
}

/// Unspent shielded ZEC notes with no witness at their pool's checkpoint.
/// Empty when the database is consistent.
pub async fn notes_missing_witness(
    conn: &mut SqliteConnection,
    accounts: &[u32],
) -> Result<Vec<MissingWitness>> {
    if accounts.is_empty() {
        return Ok(vec![]);
    }
    let placeholders = vec!["?"; accounts.len()].join(", ");
    let query = format!(
        "WITH utxo AS (SELECT * FROM notes n LEFT JOIN spends s ON n.id_note = s.id_note WHERE s.id_note IS NULL),
        db_height AS (SELECT * FROM sync_heights)
        SELECT u.account, u.pool, u.height, u.value, d.height FROM utxo u
        JOIN db_height d ON d.account = u.account AND d.pool = u.pool
        LEFT JOIN witnesses w ON u.id_note = w.note AND w.account = u.account
        AND w.height = d.height
        WHERE w.id_witness IS NULL AND u.pool <> 0 AND u.id_asset IS NULL
        AND u.account IN ({placeholders})"
    );
    // Audited: `placeholders` is a generated `?, ?, ...` list; the account
    // values themselves are bound below.
    let mut q = sqlx::query(sqlx::AssertSqlSafe(query));
    for account in accounts {
        q = q.bind(account);
    }
    let rows = q
        .map(|r: SqliteRow| MissingWitness {
            account: r.get(0),
            pool: r.get(1),
            height: r.get(2),
            value: r.get(3),
            checkpoint: r.get(4),
        })
        .fetch_all(&mut *conn)
        .await?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Outputs and memos (details pass)

/// Record an output of one of the account's transactions and return its row
/// id, whether it was new or already stored.
pub async fn record_output(
    conn: &mut SqliteConnection,
    account: u32,
    output: &OutputRow<'_>,
) -> Result<u32> {
    sqlx::query(
        "INSERT INTO outputs (account, height, tx, pool, vout, value, address)
        VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT DO NOTHING",
    )
    .bind(account)
    .bind(output.height)
    .bind(output.id_tx)
    .bind(output.pool)
    .bind(output.vout)
    .bind(output.value)
    .bind(output.address)
    .execute(&mut *conn)
    .await?;
    let id =
        sqlx::query_scalar("SELECT id_output FROM outputs WHERE tx = ? AND pool = ? AND vout = ?")
            .bind(output.id_tx)
            .bind(output.pool)
            .bind(output.vout)
            .fetch_optional(&mut *conn)
            .await?
            .ok_or_else(|| {
                anyhow!(
                    "output {}/{} of transaction {} was not stored",
                    output.pool,
                    output.vout,
                    output.id_tx
                )
            })?;
    Ok(id)
}

/// Record a decrypted memo. A memo already stored for the same output is
/// left as it is.
pub async fn record_memo(
    conn: &mut SqliteConnection,
    account: u32,
    memo: &MemoRow<'_>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO memos (account, height, tx, pool, vout, note, output, memo_text, memo_bytes)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT DO NOTHING",
    )
    .bind(account)
    .bind(memo.height)
    .bind(memo.id_tx)
    .bind(memo.pool)
    .bind(memo.vout)
    .bind(memo.note)
    .bind(memo.output)
    .bind(memo.text)
    .bind(memo.bytes)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Headers and checkpoints

/// Store a block header for the account, whether or not anything of the
/// account's is at that height.
pub async fn store_header(
    conn: &mut SqliteConnection,
    account: u32,
    header: &BlockHeader,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO headers (account, height, hash, time) VALUES (?, ?, ?, ?)
        ON CONFLICT DO NOTHING",
    )
    .bind(account)
    .bind(header.height)
    .bind(&header.hash)
    .bind(header.time)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Store a block header only where it belongs: a downloaded height is shared
/// by the batch, but its header is kept for an account only when the account
/// has a transaction, checkpoint or witness there. Checkpoints keep the
/// frontier hash available for continuity checks even without transactions.
pub async fn record_relevant_header(
    conn: &mut SqliteConnection,
    account: u32,
    header: &BlockHeader,
) -> Result<()> {
    let relevant: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM transactions WHERE account = ?1 AND height = ?2)
            OR EXISTS(SELECT 1 FROM sync_heights WHERE account = ?1 AND height = ?2)
            OR EXISTS(SELECT 1 FROM witnesses WHERE account = ?1 AND height = ?2)",
    )
    .bind(account)
    .bind(header.height)
    .fetch_one(&mut *conn)
    .await?;
    if relevant {
        store_header(conn, account, header).await?;
    }
    Ok(())
}

/// Hash of the block the account last synced at `height`, for the chain
/// continuity check at the start of a scan.
pub async fn header_hash(
    conn: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<Option<Vec<u8>>> {
    let hash = sqlx::query_scalar("SELECT hash FROM headers WHERE account = ? AND height = ?")
        .bind(account)
        .bind(height)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(hash)
}

/// Set a pool's checkpoint for the account unconditionally: used when the
/// pool is being created, before the account has a key row for it.
pub async fn set_checkpoint(
    conn: &mut SqliteConnection,
    account: u32,
    pool: u8,
    height: u32,
) -> Result<()> {
    sqlx::query("INSERT OR REPLACE INTO sync_heights(account, pool, height) VALUES (?1, ?2, ?3)")
        .bind(account)
        .bind(pool)
        .bind(height)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Advance a pool's checkpoint for the account. Ignored for a pool the
/// account does not have.
pub async fn checkpoint(
    conn: &mut SqliteConnection,
    account: u32,
    pool: u8,
    height: u32,
) -> Result<()> {
    if !has_pool(conn, account, pool).await? {
        return Ok(());
    }
    set_checkpoint(conn, account, pool, height).await?;
    debug!("Checkpoint for account: {}, height: {}", account, height);
    Ok(())
}

/// Reset every pool's checkpoint to the account's birth height, clamped to
/// the pool's activation height.
pub async fn init_checkpoints(
    conn: &mut SqliteConnection,
    network: &Network,
    account: u32,
    birth_height: u32,
) -> Result<()> {
    let max_pool = if network.activation_height(NetworkUpgrade::Nu6_3).is_some() {
        4
    } else {
        3
    };
    for pool in 0..max_pool {
        let activation_height: u32 = match pool {
            0 => 0,
            1 => network
                .activation_height(NetworkUpgrade::Sapling)
                .unwrap()
                .into(),
            2 => network
                .activation_height(NetworkUpgrade::Nu5)
                .unwrap()
                .into(),
            3 => network
                .activation_height(NetworkUpgrade::Nu6_3)
                .unwrap()
                .into(),
            _ => unreachable!(),
        };
        sqlx::query("UPDATE sync_heights SET height = ?3 WHERE account = ?1 AND pool = ?2")
            .bind(account)
            .bind(pool)
            .bind(birth_height.max(activation_height))
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

/// Height of the account's last witness checkpoint strictly below `height`,
/// the point a rewind can trim back to.
pub async fn last_checkpoint_before(
    conn: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<Option<u32>> {
    let h =
        sqlx::query_scalar("SELECT MAX(height) FROM witnesses WHERE height < ? AND account = ?")
            .bind(height)
            .bind(account)
            .fetch_one(&mut *conn)
            .await?;
    Ok(h)
}

/// Drop witnesses older than `height`; they are only needed at the
/// checkpoints a rewind can land on.
pub async fn prune_witnesses_before(
    conn: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<()> {
    sqlx::query("DELETE FROM witnesses WHERE account = ? AND height < ?")
        .bind(account)
        .bind(height)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Remove everything the account synced after `height` and move its
/// checkpoints back there. Data at `height` is kept. Takes the caller's
/// transaction: a rewind touches every table and must be atomic.
pub async fn trim_to(tx: &mut Transaction<'_, Sqlite>, account: u32, height: u32) -> Result<()> {
    for table in [
        "headers",
        "notes",
        "spends",
        "transactions",
        "witnesses",
        "outputs",
        "memos",
        "user_memos",
    ] {
        // Audited: `table` comes from the fixed list above.
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "DELETE FROM {table} WHERE height > ? AND account = ?"
        )))
        .bind(height)
        .bind(account)
        .execute(&mut **tx)
        .await?;
    }
    sqlx::query("UPDATE sync_heights SET height = ? WHERE account = ?")
        .bind(height)
        .bind(account)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Remove everything the chain log holds for an account, when the account
/// itself is deleted. Takes the caller's transaction so the account's rows
/// in every table go together.
pub async fn delete_account_data(tx: &mut Transaction<'_, Sqlite>, account: u32) -> Result<()> {
    for table in [
        "outputs",
        "memos",
        "user_memos",
        "witnesses",
        "notes",
        "spends",
        "transactions",
        "headers",
        "sync_heights",
    ] {
        // Audited: `table` comes from the fixed list above.
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "DELETE FROM {table} WHERE account = ?"
        )))
        .bind(account)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_schema;
    use sqlx::Connection;

    async fn db() -> Result<SqliteConnection> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        Ok(db)
    }

    async fn give_pools(db: &mut SqliteConnection, account: u32) -> Result<()> {
        sqlx::query("INSERT INTO transparent_accounts(account) VALUES (?)")
            .bind(account)
            .execute(&mut *db)
            .await?;
        sqlx::query("INSERT INTO sapling_accounts(account, xvk) VALUES (?, X'00')")
            .bind(account)
            .execute(&mut *db)
            .await?;
        Ok(())
    }

    async fn tx(db: &mut SqliteConnection, account: u32, txid: u8, height: u32) -> Result<u32> {
        record_transaction(db, account, &TransactionRow::chain(&[txid], height, 0))
            .await?
            .ok_or_else(|| anyhow!("duplicate"))
    }

    fn shielded(
        id_tx: u32,
        height: u32,
        nf: &'static [u8],
        cmx: &'static [u8],
    ) -> NoteRow<'static> {
        NoteRow {
            id_tx,
            height,
            pool: 1,
            scope: Some(0),
            nullifier: nf,
            value: 1000,
            cmx: Some(cmx),
            position: Some(7),
            ..Default::default()
        }
    }

    fn count<'a>(
        db: &'a mut SqliteConnection,
        sql: &'static str,
    ) -> impl std::future::Future<Output = Result<i64>> + 'a {
        async move { Ok(sqlx::query_scalar(sql).fetch_one(db).await?) }
    }

    #[tokio::test]
    async fn spend_releases_the_lock_for_transparent_and_shielded_notes() -> Result<()> {
        let mut db = db().await?;
        let id_tx = tx(&mut db, 1, 1, 100).await?;
        let t_note = NoteRow {
            id_tx,
            height: 100,
            pool: 0,
            nullifier: b"outpoint",
            value: 500,
            taddress: Some(3),
            ..Default::default()
        };
        let t = record_note(&mut db, 1, &t_note).await?.unwrap();
        let s = record_note(&mut db, 1, &shielded(id_tx, 100, b"nf", b"cmx"))
            .await?
            .unwrap();
        lock_notes(&mut db, 1, &[b"outpoint".to_vec(), b"nf".to_vec()]).await?;
        assert_eq!(
            count(&mut db, "SELECT COUNT(*) FROM notes WHERE locked").await?,
            2
        );

        let spend_tx = tx(&mut db, 1, 2, 110).await?;
        let by_outpoint = SpendRow {
            note: NoteKey::Nullifier(b"outpoint"),
            id_tx: spend_tx,
            height: 110,
            pool: 0,
            value: -500,
        };
        let by_cmx = SpendRow {
            note: NoteKey::Cmx(b"cmx"),
            id_tx: spend_tx,
            height: 110,
            pool: 1,
            value: -1000,
        };
        assert!(record_spend(&mut db, 1, &by_outpoint).await?);
        assert!(record_spend(&mut db, 1, &by_cmx).await?);
        // the same spend seen again is a no-op, not an error
        assert!(!record_spend(&mut db, 1, &by_cmx).await?);

        let rows: Vec<(u32, u32, i64)> =
            sqlx::query_as("SELECT id_note, tx, value FROM spends ORDER BY id_note")
                .fetch_all(&mut db)
                .await?;
        assert_eq!(rows, vec![(t, spend_tx, -500), (s, spend_tx, -1000)]);
        assert_eq!(
            count(&mut db, "SELECT COUNT(*) FROM notes WHERE locked").await?,
            0
        );
        Ok(())
    }

    #[tokio::test]
    async fn trim_clears_every_table_above_the_height_for_one_account() -> Result<()> {
        let mut db = db().await?;
        for account in [1, 2] {
            set_checkpoint(&mut db, account, 1, 200).await?;
            for (txid, height) in [(1u8, 100u32), (2, 150), (3, 200)] {
                let id_tx = tx(&mut db, account, txid, height).await?;
                let nf = [account as u8, txid];
                let cmx = [9, account as u8, txid];
                let note = NoteRow {
                    id_tx,
                    height,
                    pool: 1,
                    nullifier: &nf,
                    value: 1,
                    cmx: Some(&cmx),
                    ..Default::default()
                };
                let id_note = record_note(&mut db, account, &note).await?.unwrap();
                record_witness(&mut db, account, NoteKey::Id(id_note), height, b"w").await?;
                record_output(
                    &mut db,
                    account,
                    &OutputRow {
                        id_tx,
                        height,
                        pool: 1,
                        vout: 0,
                        value: 1,
                        address: "z",
                    },
                )
                .await?;
                record_memo(
                    &mut db,
                    account,
                    &MemoRow {
                        id_tx,
                        height,
                        pool: 1,
                        vout: 0,
                        note: Some(id_note),
                        output: None,
                        text: None,
                        bytes: b"m",
                    },
                )
                .await?;
                let session = crate::sync::session::SyncSession::new();
                let run = session.try_begin().unwrap();
                crate::db::set_user_memo(
                    &mut run.held(&mut db),
                    account,
                    &[txid],
                    Some("note".into()),
                )
                .await?;
                store_header(
                    &mut db,
                    account,
                    &BlockHeader {
                        height,
                        hash: vec![1; 32],
                        time: height,
                    },
                )
                .await?;
            }
            let spent = SpendRow {
                note: NoteKey::Nullifier(&[account as u8, 1]),
                id_tx: 0,
                height: 150,
                pool: 1,
                value: -1,
            };
            record_spend(&mut db, account, &spent).await?;
        }

        let mut t = db.begin().await?;
        trim_to(&mut t, 1, 120).await?;
        t.commit().await?;

        for table in [
            "headers",
            "notes",
            "spends",
            "transactions",
            "witnesses",
            "outputs",
            "memos",
            "user_memos",
        ] {
            let above: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                "SELECT COUNT(*) FROM {table} WHERE account = 1 AND height > 120"
            )))
            .fetch_one(&mut db)
            .await?;
            assert_eq!(above, 0, "{table} still has rows above the trim height");
            let other: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                "SELECT COUNT(*) FROM {table} WHERE account = 2"
            )))
            .fetch_one(&mut db)
            .await?;
            assert!(other > 0, "{table} of the other account was touched");
        }
        assert_eq!(
            count(&mut db, "SELECT COUNT(*) FROM notes WHERE account = 1").await?,
            1
        );
        assert_eq!(
            count(&mut db, "SELECT COUNT(*) FROM user_memos WHERE account = 1").await?,
            1,
            "the note on the transaction at the trim height stays"
        );
        let checkpoints: Vec<(u32, u32)> =
            sqlx::query_as("SELECT account, height FROM sync_heights ORDER BY account")
                .fetch_all(&mut db)
                .await?;
        assert_eq!(checkpoints, vec![(1, 120), (2, 200)]);
        Ok(())
    }

    #[tokio::test]
    async fn headers_are_only_recorded_where_the_account_has_something() -> Result<()> {
        let mut db = db().await?;
        tx(&mut db, 1, 1, 100).await?;
        set_checkpoint(&mut db, 2, 1, 100).await?;
        let header = BlockHeader {
            height: 100,
            hash: vec![1; 32],
            time: 5,
        };
        for account in [1, 2, 3] {
            record_relevant_header(&mut db, account, &header).await?;
        }
        let owners: Vec<u32> = sqlx::query_scalar("SELECT account FROM headers ORDER BY account")
            .fetch_all(&mut db)
            .await?;
        assert_eq!(owners, vec![1, 2]);
        assert_eq!(header_hash(&mut db, 1, 100).await?, Some(vec![1; 32]));
        assert_eq!(header_hash(&mut db, 3, 100).await?, None);
        Ok(())
    }

    #[tokio::test]
    async fn checkpoint_is_skipped_for_a_pool_the_account_lacks() -> Result<()> {
        let mut db = db().await?;
        give_pools(&mut db, 1).await?;
        checkpoint(&mut db, 1, 0, 50).await?;
        checkpoint(&mut db, 1, 1, 60).await?;
        checkpoint(&mut db, 1, 2, 70).await?; // no orchard key
        let rows: Vec<(u8, u32)> =
            sqlx::query_as("SELECT pool, height FROM sync_heights WHERE account = 1 ORDER BY pool")
                .fetch_all(&mut db)
                .await?;
        assert_eq!(rows, vec![(0, 50), (1, 60)]);
        Ok(())
    }

    #[tokio::test]
    async fn missing_witness_query_reflects_witness_at_the_checkpoint() -> Result<()> {
        let mut db = db().await?;
        set_checkpoint(&mut db, 1, 1, 200).await?;
        let id_tx = tx(&mut db, 1, 1, 100).await?;
        let id_note = record_note(&mut db, 1, &shielded(id_tx, 100, b"nf", b"cmx"))
            .await?
            .unwrap();
        let missing = notes_missing_witness(&mut db, &[1]).await?;
        assert_eq!(
            missing,
            vec![MissingWitness {
                account: 1,
                pool: 1,
                height: 100,
                value: 1000,
                checkpoint: 200
            }]
        );
        record_witness(&mut db, 1, NoteKey::Cmx(b"cmx"), 200, b"w").await?;
        assert!(notes_missing_witness(&mut db, &[1]).await?.is_empty());
        // a spent note needs no witness
        let _ = id_note;
        assert!(notes_missing_witness(&mut db, &[]).await?.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn unknown_transaction_or_note_is_an_error_not_a_panic() -> Result<()> {
        let mut db = db().await?;
        assert!(transaction_ref(&mut db, 1, b"\x01").await.is_err());
        let spend = SpendRow {
            note: NoteKey::Cmx(b"nope"),
            id_tx: 1,
            height: 1,
            pool: 1,
            value: -1,
        };
        assert!(record_spend(&mut db, 1, &spend).await.is_err());
        assert!(record_witness(&mut db, 1, NoteKey::Cmx(b"nope"), 1, b"w")
            .await
            .is_err());
        // duplicates are reported, not raised
        let id_tx = tx(&mut db, 1, 1, 100).await?;
        assert!(
            record_transaction(&mut db, 1, &TransactionRow::chain(&[1], 100, 0))
                .await?
                .is_none()
        );
        assert!(
            record_note(&mut db, 1, &shielded(id_tx, 100, b"nf", b"cmx"))
                .await?
                .is_some()
        );
        assert!(
            record_note(&mut db, 1, &shielded(id_tx, 100, b"nf", b"cmx"))
                .await?
                .is_none()
        );
        Ok(())
    }

    async fn locked(db: &mut SqliteConnection) -> Result<Vec<(u32, bool)>> {
        let rows =
            sqlx::query_as("SELECT id_note, locked FROM notes WHERE account = 1 ORDER BY id_note")
                .fetch_all(db)
                .await?;
        Ok(rows)
    }

    #[tokio::test]
    async fn lock_toggles_and_annotations_touch_the_named_rows_only() -> Result<()> {
        let mut db = db().await?;
        let id_tx = tx(&mut db, 1, 1, 100).await?;
        let low = record_note(&mut db, 1, &shielded(id_tx, 100, b"a", b"ca"))
            .await?
            .unwrap();
        let high = record_note(&mut db, 1, &shielded(id_tx, 150, b"b", b"cb"))
            .await?
            .unwrap();
        let other_tx = tx(&mut db, 2, 2, 150).await?;
        record_note(&mut db, 2, &shielded(other_tx, 150, b"c", b"cc")).await?;

        lock_notes_above(&mut db, 1, 120).await?;
        assert_eq!(locked(&mut db).await?, vec![(low, false), (high, true)]);
        toggle_all_locks(&mut db, 1).await?;
        assert_eq!(locked(&mut db).await?, vec![(low, true), (high, false)]);
        set_note_lock(&mut db, 1, high, true).await?;
        unlock_all(&mut db, 1).await?;
        assert_eq!(locked(&mut db).await?, vec![(low, false), (high, false)]);
        assert_eq!(
            count(
                &mut db,
                "SELECT COUNT(*) FROM notes WHERE account = 2 AND locked"
            )
            .await?,
            0
        );

        set_transaction_category(&mut db, id_tx, Some(7)).await?;
        set_transaction_price(&mut db, id_tx, Some(2.0)).await?;
        scale_transaction_prices(&mut db, 1.5).await?;
        let (category, price): (Option<u32>, Option<f64>) =
            sqlx::query_as("SELECT category, price FROM transactions WHERE id_tx = ?")
                .bind(id_tx)
                .fetch_one(&mut db)
                .await?;
        assert_eq!((category, price), (Some(7), Some(3.0)));
        clear_category(&mut db, 7).await?;
        let category: Option<u32> =
            sqlx::query_scalar("SELECT category FROM transactions WHERE id_tx = ?")
                .bind(id_tx)
                .fetch_one(&mut db)
                .await?;
        assert_eq!(category, None);

        record_asset(
            &mut db,
            &AssetRow {
                desc_hash: b"d",
                ik: b"k",
                base: &[1u8; 32],
                name: None,
                finalized: false,
                first_seen_height: 1,
            },
        )
        .await?;
        let id = asset_id(&mut db, b"d", b"k").await?.unwrap();
        set_asset_name(&mut db, id as i64, "GOLD").await?;
        assert!(set_asset_name(&mut db, 999, "x").await.is_err());

        let mut t = db.begin().await?;
        delete_account_data(&mut t, 1).await?;
        t.commit().await?;
        assert_eq!(
            count(&mut db, "SELECT COUNT(*) FROM notes WHERE account = 1").await?,
            0
        );
        assert_eq!(
            count(
                &mut db,
                "SELECT COUNT(*) FROM transactions WHERE account = 1"
            )
            .await?,
            0
        );
        assert_eq!(
            count(&mut db, "SELECT COUNT(*) FROM notes WHERE account = 2").await?,
            1
        );
        Ok(())
    }

    #[tokio::test]
    async fn init_checkpoints_clamps_to_pool_activation() -> Result<()> {
        let mut db = db().await?;
        for pool in 0..3u8 {
            set_checkpoint(&mut db, 1, pool, 1).await?;
        }
        init_checkpoints(&mut db, &Network::Main, 1, 100).await?;
        let rows: Vec<(u8, u32)> =
            sqlx::query_as("SELECT pool, height FROM sync_heights WHERE account = 1 ORDER BY pool")
                .fetch_all(&mut db)
                .await?;
        let sapling: u32 = Network::Main
            .activation_height(NetworkUpgrade::Sapling)
            .unwrap()
            .into();
        let nu5: u32 = Network::Main
            .activation_height(NetworkUpgrade::Nu5)
            .unwrap()
            .into();
        assert_eq!(rows, vec![(0, 100), (1, sapling), (2, nu5)]);
        Ok(())
    }
}
