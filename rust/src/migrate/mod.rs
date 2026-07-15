use anyhow::Result;
use sqlx::{Row, SqliteConnection};
use tracing::info;

use crate::{
    account::get_account_full_address,
    api::coin::Network,
    db::get_account_hw,
    pay::{
        fee::{COST_PER_ACTION, FeeManager},
        plan::{
            extract_transaction, fetch_unspent_notes_grouped_by_pool, plan_transaction,
            sign_transaction,
        },
        pool::PoolMask,
        Recipient, send,
    },
    Client,
};

/// Minimum standard denomination: 20 × COST_PER_ACTION (100,000 zats).
/// Notes smaller than this are not economical to split or migrate.
const MIN_SD: u64 = 20 * COST_PER_ACTION;

/// Maximum number of non-SD notes to split in a single transaction.
/// Caps transaction size to avoid oversized bundles that nodes reject.
const MAX_SPLIT_INPUTS: usize = 50;

/// Fee padding embedded in each standard denomination.
/// Covers Orchard input + change (2 actions in sum mode) and Ironwood
/// output (2 actions, padded) = 4 × COST_PER_ACTION = 20,000 zats.
const SD_FEE_PAD: u64 = 4 * COST_PER_ACTION;

/// Standard denominations: powers of 10 up to 10^15.
const STANDARD_DENOMINATIONS: &[u64] = &[
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
    10_000_000_000,
    100_000_000_000,
    1_000_000_000_000,
    10_000_000_000_000,
    100_000_000_000_000,
    1_000_000_000_000_000,
];

/// Decompose a total amount into standard denomination notes with embedded fees.
///
/// Each standard denomination is `10^k + P` where P = 2*COST_PER_ACTION:
/// 1_000_010_000, 100_010_000, …, 110_000.
///
/// Greedy from largest to smallest. Returns sparse `(denom, count)` pairs
/// and any leftover below the smallest denomination.
pub fn decompose_to_sd(total: u64) -> (Vec<(u64, u8)>, u64) {
    let p = SD_FEE_PAD;
    let d_min = 10u64.pow(5) + p; // 110_000
    let k_min = 5u32;
    let k_max = 16u32; // 10^16 + P covers 100M ZEC
    let mut result = Vec::new();
    let mut remainder = total;

    for k in (k_min..k_max).rev() {
        if remainder < d_min {
            break;
        }
        let d = 10u64.pow(k) + p;
        let count = (remainder / d) as u8;
        remainder %= d;
        if count > 0 {
            result.push((d, count));
        }
    }

    (result, remainder)
}

/// Check if a value is a fee-inclusive standard denomination:
/// `10^(i+5) + 2*COST_PER_ACTION` (110_000, 1_010_000, 10_010_000, …).
pub fn is_sd(value: u64) -> bool {
    let base = SD_FEE_PAD;
    if value <= base {
        return false;
    }
    let n = value - base; // must be exactly 10^(i+5), i ≥ 0
    // n must be ≥ 100_000, a multiple thereof, and reduce to 1 after
    // dividing out factors of 10.
    n >= 100_000 && n % 100_000 == 0 && {
        let mut x = n / 100_000;
        while x % 10 == 0 {
            x /= 10;
        }
        x == 1
    }
}

/// Result of a migration step.
pub enum MigrationEvent {
    /// A split transaction was broadcast.
    SplitComplete { fee: u64 },
    /// A migration transaction was broadcast.
    MigrateComplete { fee: u64 },
    /// Migration is complete — no more Orchard notes to migrate.
    Complete,
    /// No action needed (e.g., all notes are already SD but no migration
    /// target yet, or waiting for confirmation).
    NothingToDo,
}

/// Current migration status for the UI.
pub struct MigrationStatus {
    pub phase: String,
    pub progress: f64,
    pub next_action: String,
    pub work_summary: String,
    pub sd_notes_count: u32,
    pub non_sd_notes_count: u32,
}

/// Notes grouped by pool and ZEC/ZSA.
struct OrchardZecNote {
    id: u32,
    value: u64,
    cmx: Option<Vec<u8>>,
}

/// Fetch unspent Orchard ZEC notes with their cmx values.
///
/// Like `fetch_unspent_notes_grouped_by_pool` but restricted to Orchard ZEC
/// (pool 2, no asset) and includes `cmx` so callers don't need a second
/// pass to fetch commitments.
async fn fetch_unspent_orchard_notes_with_cmx(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Vec<OrchardZecNote>> {
    sqlx::query(
        "SELECT a.id_note, a.value, a.cmx
         FROM notes a
         LEFT JOIN spends b ON a.id_note = b.id_note
         WHERE b.id_note IS NULL
         AND a.account = ?
         AND a.pool = 2
         AND a.id_asset IS NULL
         AND a.locked = 0",
    )
    .bind(account)
    .map(|row| {
        OrchardZecNote {
            id: row.get(0),
            value: row.get::<i64, _>(1) as u64,
            cmx: row.get(2),
        }
    })
    .fetch_all(connection)
    .await
    .map_err(Into::into)
}

/// Run one migration step. Fully idempotent — re-scans notes on every call.
pub async fn step(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    account: u32,
) -> Result<MigrationEvent> {
    let height = client.latest_height().await?;

    // Get the wallet's own Orchard/Ironwood address
    let hw = get_account_hw(&mut *connection, account).await?;
    let own_address =
        get_account_full_address(network, &mut *connection, account, 0, hw).await?;

    // Fetch all unspent Orchard ZEC notes with cmx.
    let orchard_zec =
        fetch_unspent_orchard_notes_with_cmx(&mut *connection, account).await?;

    info!(
        "Migration step: {} Orchard ZEC notes found",
        orchard_zec.len(),
    );
    if orchard_zec.is_empty() {
        return Ok(MigrationEvent::Complete);
    }

    // Separate SD vs non-SD
    let sd_notes: Vec<&OrchardZecNote> =
        orchard_zec.iter().filter(|n| is_sd(n.value)).collect();
    let non_sd_notes: Vec<&OrchardZecNote> =
        orchard_zec.iter().filter(|n| !is_sd(n.value)).collect();
    info!(
        "SD notes: {:?}, non-SD notes: {:?}",
        sd_notes.iter().map(|n| n.value).collect::<Vec<_>>(),
        non_sd_notes.iter().map(|n| n.value).collect::<Vec<_>>(),
    );

    // ── Splitting phase ──

    // Cap inputs to keep transaction size manageable. Sort by value
    // descending so the largest notes are split first; remaining non-SD
    // notes will be handled in subsequent step() calls.
    let capped_non_sd: Vec<&OrchardZecNote> = {
        let mut sorted = non_sd_notes.clone();
        sorted.sort_by_key(|n| std::cmp::Reverse(n.value));
        sorted.truncate(MAX_SPLIT_INPUTS);
        sorted
    };

    // Calculate total from capped non-SD notes.
    let total: u64 = capped_non_sd.iter().map(|n| n.value).sum();

    if total >= MIN_SD {
    // Decompose into standard denomination counts (digits) and remainder.
    let (mut digits, mut remainder) = decompose_to_sd(total);
    info!(
        "SD split: {:?}",
        digits,
    );

    let mut num_outputs: u64 = digits.iter().map(|&(_, c)| c as u64).sum();
    let num_inputs = capped_non_sd.len() as u64;

    // Build a FeeManager matching what plan_transaction will construct,
    // including the change output, so our fee estimate is exact.
    let mut fm = FeeManager {
        migration: true,
        ..FeeManager::default()
    };
    for _ in 0..num_inputs {
        fm.add_input(2);
    }
    for _ in 0..num_outputs {
        fm.add_output(2);
    }
    fm.add_output(2); // change output

    // Fee loop: if fee exceeds remainder, trim the lowest-denomination
    // output to make room, then retry. Exit when fee fits or no outputs
    // remain (fall through to migration).
    loop {
        let fee = fm.fee();

        if fee <= remainder || num_outputs == 0 {
            break;
        }

        // Remove one unit from the lowest denomination (last, since
        // denominations are sorted largest-first).
        if let Some((denom, count)) = digits.last_mut() {
            *count -= 1;
            remainder += *denom;
            num_outputs -= 1;
            fm.remove_output(2);
            if *count == 0 {
                digits.pop();
            }
        }
    }

    if num_outputs > 0 {
        // Build recipients from (denom, count) pairs.
        let mut recipients: Vec<Recipient> = Vec::new();
        for &(denom, count) in &digits {
            for _ in 0..count {
                recipients.push(Recipient {
                    address: own_address.clone(),
                    amount: denom,
                    pools: Some(PoolMask::from_pool(2).0), // Orchard only
                    ..Recipient::default()
                });
            }
        }

        info!(
            "Migration split: {} non-SD notes (total {}) → {} SD outputs (remainder {})",
            capped_non_sd.len(),
            total,
            recipients.len(),
            remainder,
        );

        let preselected: Vec<u32> = capped_non_sd.iter().map(|n| n.id).collect();

        let pczt = plan_transaction(
            network,
            &mut *connection,
            client,
            account,
            PoolMask::from_pool(2).0, // Orchard source
            &recipients,
            false,
            None,
            false,
            None,
            None,
            true, // migration
            Some(&preselected),
        )
        .await?;

        let fee = crate::pay::TxPlan::from_package(network, &pczt)
            .map(|p| p.fee)
            .unwrap_or(0);
        let pczt =
            sign_transaction(&mut *connection, account, network, &pczt).await?;
        let tx_bytes = extract_transaction(&pczt).await?;
        let _txid = send(client, height, &tx_bytes).await?;

        return Ok(MigrationEvent::SplitComplete { fee });
    }
    // If no outputs after trimming, fall through to migration phase.
    } // end if total >= MIN_SD

    if !sd_notes.is_empty() {

        /*
        # migrate one orchard SD note at a time
        - inputs:
            - select 1 SD note, it include 2 COST_ACTIONS
            - dummy input
        - outputs
            - ironwood SD - 2 COST_ACTIONS = "real" SD
            - dummy output
         */

        // ── Migrating phase ──
        // Sort by cmx for deterministic random order
        let mut sorted_sd: Vec<&&OrchardZecNote> = sd_notes.iter().collect();
        sorted_sd.sort_by(|a, b| {
            let a_cmx = a.cmx.as_deref().unwrap_or(&[]);
            let b_cmx = b.cmx.as_deref().unwrap_or(&[]);
            a_cmx.cmp(b_cmx)
        });

        // Pick one SD note (largest cmx). Its value embeds 2*COST_PER_ACTION
        // for Orchard fees; the Ironwood output is the "real" denomination.
        let note = sorted_sd.last().unwrap();
        let ironwood_amount = note.value - SD_FEE_PAD;

        // One Ironwood output (dummy output for padding is handled by the
        // builder, as is the dummy Orchard input).
        let recipients = vec![Recipient {
            address: own_address.clone(),
            amount: ironwood_amount,
            pools: Some(PoolMask::from_pool(3).0), // Ironwood
            ..Recipient::default()
        }];

        info!(
            "Migration: note id={} value={} → Ironwood amount={}",
            note.id, note.value, ironwood_amount,
        );

        let preselected: Vec<u32> = vec![note.id];

        let pczt = plan_transaction(
            network,
            &mut *connection,
            client,
            account,
            PoolMask::from_pool(2).0, // Orchard source
            &recipients,
            false,
            None,
            false,
            None,
            None,
            true, // migration — O→I
            Some(&preselected),
        )
        .await?;

        let fee = crate::pay::TxPlan::from_package(network, &pczt)
            .map(|p| p.fee)
            .unwrap_or(0);
        let pczt =
            sign_transaction(&mut *connection, account, network, &pczt).await?;
        let tx_bytes = extract_transaction(&pczt).await?;
        let _txid = send(client, height, &tx_bytes).await?;

        return Ok(MigrationEvent::MigrateComplete { fee });
    }

    // No SD and no non-SD orchard notes
    Ok(MigrationEvent::Complete)
}

/// Get the current migration status for the UI.
pub async fn get_status(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<MigrationStatus> {
    let all_notes = fetch_unspent_notes_grouped_by_pool(connection, account).await?;
    let orchard_zec: Vec<&crate::pay::InputNote> = all_notes
        .iter()
        .filter(|n| n.pool == 2 && n.asset_base == vec![0u8; 32])
        .collect();

    let sd_count = orchard_zec.iter().filter(|n| is_sd(n.amount)).count() as u32;
    let non_sd_count = orchard_zec.len() as u32 - sd_count;

    let (phase, next_action, work_summary, progress) = if non_sd_count > 0 {
        let total: u64 = orchard_zec
            .iter()
            .filter(|n| !is_sd(n.amount))
            .map(|n| n.amount)
            .sum();
        let (sd_amounts, remainder) = decompose_to_sd(total);
        (
            "splitting",
            format!(
                "Split {} non-SD notes (total {}) → {} SD outputs{}",
                non_sd_count,
                total,
                sd_amounts.len(),
                if remainder > 0 {
                    format!(" + {} remainder", remainder)
                } else {
                    String::new()
                }
            ),
            format!(
                "{} SD notes created so far, {} non-SD remaining",
                sd_count, non_sd_count
            ),
            if sd_count + non_sd_count > 0 {
                sd_count as f64 / (sd_count + non_sd_count) as f64
            } else {
                0.0
            },
        )
    } else if sd_count > 0 {
        (
            "migrating",
            format!("Migrate next SD note to Ironwood ({} remaining)", sd_count),
            format!("{} SD notes remaining to migrate", sd_count),
            0.5, // halfway point: splitting done, migration in progress
        )
    } else {
        ("complete", "Done".to_string(), "No Orchard notes to migrate".to_string(), 1.0)
    };

    Ok(MigrationStatus {
        phase: phase.to_string(),
        progress,
        next_action,
        work_summary,
        sd_notes_count: sd_count,
        non_sd_notes_count: non_sd_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sd() {
        // SD_FEE_PAD = 20_000, so SD = 10^k + 20_000
        assert!(!is_sd(10_001));     // not a multiple of 10,000
        assert!(!is_sd(20_001));     // (20001-20000) % 100000 = 1 ≠ 0
        assert!(is_sd(120_000));     // 10^5 + 20_000
        assert!(is_sd(1_020_000));   // 10^6 + 20_000
        assert!(is_sd(10_020_000));  // 10^7 + 20_000
        assert!(!is_sd(1_000_000));  // missing +base
        assert!(!is_sd(120_001));    // (120001-20000) % 100000 = 1 ≠ 0
        // Old P=10_000 values are no longer SD
        assert!(!is_sd(110_000));
        assert!(!is_sd(1_010_000));
    }

    #[test]
    fn test_decompose_below_min_denom() {
        // Below d_min (120_000).
        let (pairs, leftover) = decompose_to_sd(10_000);
        assert!(pairs.is_empty());
        assert_eq!(leftover, 10_000);
    }

    #[test]
    fn test_decompose_zero() {
        let (pairs, leftover) = decompose_to_sd(0);
        assert!(pairs.is_empty());
        assert_eq!(leftover, 0);
    }

    #[test]
    fn test_decompose_exact_sd() {
        // 120_000 = 10^5 + 20_000.
        let (pairs, leftover) = decompose_to_sd(120_000);
        assert_eq!(pairs, vec![(120_000, 1)]);
        assert_eq!(leftover, 0);
    }

    #[test]
    fn test_decompose_multiple() {
        // 4 × 120_000 = 480_000, leftover 20_000.
        let (pairs, leftover) = decompose_to_sd(500_000);
        assert_eq!(pairs, vec![(120_000, 4)]);
        assert_eq!(leftover, 20_000);
    }

    #[test]
    fn test_decompose_two_positions() {
        // 1_140_000 → 1×1_020_000 + 1×120_000.
        let (pairs, leftover) = decompose_to_sd(1_140_000);
        assert_eq!(pairs, vec![(1_020_000, 1), (120_000, 1)]);
        assert_eq!(leftover, 0);
    }

    #[test]
    fn test_decompose_with_remainder() {
        // 130_000 → 1×120_000, leftover 10_000 (below d_min).
        let (pairs, leftover) = decompose_to_sd(130_000);
        assert_eq!(pairs, vec![(120_000, 1)]);
        assert_eq!(leftover, 10_000);
    }

    /// Round-trip invariant: sum(denom × count) + leftover ≡ original total.
    #[test]
    fn test_decompose_round_trip() {
        let cases = &[0, 10_000, 120_000, 500_000, 1_140_000, 130_000, 5_000_000];
        for &total in cases {
            let (pairs, leftover) = decompose_to_sd(total);
            let represented: u64 = pairs.iter().map(|&(d, c)| d * c as u64).sum();
            assert_eq!(
                represented + leftover,
                total,
                "round-trip failed for total={total}"
            );
        }
    }
}
