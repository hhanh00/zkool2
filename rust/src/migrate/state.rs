//! A snapshot of everything the migration needs in order to decide what to do.
//!
//! `observe` is the migration's single reader of wallet state. Prior to this,
//! the executing path and the status path each ran their own note query with
//! their own filters, and could therefore disagree about what work remained —
//! which is how a wallet ended up looping forever on a split it was never
//! going to build. One query, one snapshot, one set of facts.

use anyhow::Result;
use sqlx::{Row, SqliteConnection};

use crate::{account::get_account_full_address, api::coin::Network, db::get_account_hw, Client};

use super::{is_iw_sd, is_sd, MAX_SPLIT_INPUTS};

/// An unspent, unlocked note the migration may act on.
#[derive(Clone, Debug)]
pub struct NoteRef {
    pub id: u32,
    pub height: u32,
    pub value: u64,
    pub cmx: Option<Vec<u8>>,
    /// Whether a witness exists for this note at the observed checkpoint.
    /// A note without one cannot be spent at that anchor.
    pub has_checkpoint: bool,
}

/// Wallet state at one instant, as the migration sees it.
#[derive(Clone, Debug)]
pub struct MigrationState {
    pub tip_height: u32,
    pub checkpoint_height: u32,
    pub anchor_bucket_size: u32,
    /// Orchard ZEC notes already at a standard denomination (`10^k + SD_FEE_PAD`),
    /// ready for the O→I hop.
    pub orchard_sd: Vec<NoteRef>,
    /// Orchard ZEC notes that must be split before they can be migrated.
    pub orchard_non_sd: Vec<NoteRef>,
    /// Ironwood notes already at a pure denomination — migration's output.
    pub ironwood_sd_count: u32,
    pub own_address: String,
}

impl MigrationState {
    /// The non-SD notes a single split may consume: the largest first, capped
    /// at [`MAX_SPLIT_INPUTS`] to keep the bundle a size nodes will accept.
    /// Whatever is left over is picked up by later splits.
    pub fn capped_non_sd(&self) -> Vec<&NoteRef> {
        let mut sorted: Vec<&NoteRef> = self.orchard_non_sd.iter().collect();
        sorted.sort_by_key(|n| std::cmp::Reverse(n.value));
        sorted.truncate(MAX_SPLIT_INPUTS);
        sorted
    }

    /// Total value available to one split transaction.
    pub fn split_input_total(&self) -> u64 {
        self.capped_non_sd().iter().map(|n| n.value).sum()
    }

    /// SD notes spendable at `anchor`: present at that checkpoint, with a
    /// witness. Migration never rewinds a witness to a historical anchor.
    pub fn sd_spendable_at(&self, anchor: u32) -> Vec<&NoteRef> {
        self.orchard_sd
            .iter()
            .filter(|n| n.height <= anchor && n.has_checkpoint)
            .collect()
    }
}

/// Read wallet state. The only I/O in the decision path: one note query, the
/// wallet checkpoint, the chain tip, and the account's own address.
pub async fn observe(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    account: u32,
    anchor_bucket_size: u32,
) -> Result<MigrationState> {
    let tip_height = client.latest_height().await?;
    let checkpoint_height = crate::sync::get_db_height(&mut *connection, account)
        .await?
        .height;

    let hw = get_account_hw(&mut *connection, account).await?;
    let own_address = get_account_full_address(network, &mut *connection, account, 0, hw).await?;

    // Orchard (pool 2) and Ironwood (pool 3) ZEC notes in one pass. Locked
    // notes are excluded: a broadcast transaction locks its inputs, so they
    // disappear here and any task that named them fails its precondition
    // instead of being planned a second time.
    let notes = sqlx::query(
        "SELECT a.id_note, a.height, a.pool, a.value, a.cmx,
                EXISTS (
                    SELECT 1
                    FROM witnesses w
                    WHERE w.account = a.account
                    AND w.note = a.id_note
                    AND w.height = ?1
                )
         FROM notes a
         LEFT JOIN spends b ON a.id_note = b.id_note
         WHERE b.id_note IS NULL
         AND a.account = ?2
         AND a.pool IN (2, 3)
         AND a.id_asset IS NULL
         AND a.locked = 0",
    )
    .bind(checkpoint_height)
    .bind(account)
    .map(|row| {
        let pool: u8 = row.get(2);
        let note = NoteRef {
            id: row.get(0),
            height: row.get(1),
            value: row.get::<i64, _>(3) as u64,
            cmx: row.get(4),
            has_checkpoint: row.get(5),
        };
        (pool, note)
    })
    .fetch_all(&mut *connection)
    .await?;

    let mut orchard_sd = Vec::new();
    let mut orchard_non_sd = Vec::new();
    let mut ironwood_sd_count = 0u32;

    for (pool, note) in notes {
        match pool {
            2 if is_sd(note.value) => orchard_sd.push(note),
            2 => orchard_non_sd.push(note),
            3 if is_iw_sd(note.value) => ironwood_sd_count += 1,
            _ => {}
        }
    }

    Ok(MigrationState {
        tip_height,
        checkpoint_height,
        anchor_bucket_size,
        orchard_sd,
        orchard_non_sd,
        ironwood_sd_count,
        own_address,
    })
}
