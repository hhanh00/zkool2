use anyhow::{Context as _, Result};
use sqlx::SqliteConnection;
use sqlx::{sqlite::SqliteRow, Row};
use tokio::sync::broadcast;
use tokio::sync::mpsc::{channel, Sender};
use tracing::debug;

use crate::api::coin::{Coin, Network};
use crate::api::sync::{SyncEvent, SyncOutcome, SyncProgress, CANCEL_SYNC, SYNCING};

use crate::io::SyncHeight;
use crate::Sink;
use std::collections::HashSet;

use sqlx::Connection;

pub mod chain_log;
pub mod plan;
mod shielded;
mod sweep;
mod transparent;

use plan::{plan_sync, AccountSyncState, SyncRange};

pub use shielded::{
    get_compact_block_range, get_tree_state, preload_account_key_cache, shielded_sync,
    AccountKeyCache, OrchardAccountKeys, SaplingAccountKeys, MIXNET_SYNC_CHUNK,
};
pub use sweep::transparent_sweep;
pub(crate) use transparent::transparent_sync;

pub const DEFAULT_ACTIONS_PER_SYNC: u32 = 10000u32;
pub const DEFAULT_TRANSPARENT_LIMIT: u32 = 100u32;

pub use zcash_trees::types::SyncError;
pub use zcash_trees::types::{BlockHeader, Issuance, Note, Transaction, WarpSyncMessage, UTXO};

pub struct NoteExtended {
    pub id: u32,
    pub address: Vec<u8>,
    pub memo: Vec<u8>,
}
/// Synchronize `accounts` up to `current_height`.
///
/// Observe the accounts, plan the ranges to scan (see [`plan::plan_sync`]),
/// then run each range in order: transparent scan, shielded scan, block
/// times, and the detail pass unless `skip_details`. Progress streams through
/// `progress`; the returned outcome says how the run ended. An error is a
/// failed run; the work committed before it stays.
///
/// Only one sync runs at a time: when another holds the lock the call
/// returns [`SyncOutcome::Skipped`] without touching anything.
#[allow(clippy::too_many_arguments)]
pub async fn synchronize_impl<S: Sink<SyncEvent> + Send + 'static>(
    progress: S,
    accounts: Vec<u32>,
    current_height: u32,
    actions_per_sync: u32,
    transparent_limit: u32,
    checkpoint_age: u32,
    skip_details: bool,
    c: &Coin,
) -> Result<SyncOutcome> {
    if accounts.is_empty() {
        return Ok(SyncOutcome::Synced {
            height: current_height,
        });
    }

    let Ok(_guard) = SYNCING.try_lock() else {
        return Ok(SyncOutcome::Skipped);
    };

    let (tx_cancel, mut rx_cancel) = broadcast::channel::<()>(16);
    {
        let mut cancel = CANCEL_SYNC.lock().await;
        *cancel = Some(tx_cancel.clone());
    }

    let params = SyncParams {
        current_height,
        actions_per_sync,
        transparent_limit,
        checkpoint_age,
        skip_details,
    };
    let result = run_sync(progress, &accounts, &params, &tx_cancel, c).await;

    // Inner operations may unwind with an error after receiving cancellation.
    // A user-requested stop is normal termination at the API boundary.
    let cancelled = rx_cancel.try_recv().is_ok();

    {
        let mut cancel = CANCEL_SYNC.lock().await;
        *cancel = None;
    }

    finish(result, cancelled, current_height)
}

/// Map the run's result and the cancel flag to the outcome the caller sees.
fn finish(result: Result<()>, cancelled: bool, height: u32) -> Result<SyncOutcome> {
    if cancelled {
        debug!("Synchronization cancelled by user");
        return Ok(SyncOutcome::Cancelled);
    }
    match result {
        Ok(()) => Ok(SyncOutcome::Synced { height }),
        Err(e) => {
            debug!("Error during sync: {:?}", e);
            Err(e)
        }
    }
}

struct SyncParams {
    current_height: u32,
    actions_per_sync: u32,
    transparent_limit: u32,
    checkpoint_age: u32,
    skip_details: bool,
}

/// Where each account stands, read from the database. Accounts without a
/// checkpoint row are not synced and are left out.
async fn observe_accounts(
    connection: &mut SqliteConnection,
    accounts: &[u32],
) -> Result<Vec<AccountSyncState>> {
    let mut states = Vec::with_capacity(accounts.len());
    for &account in accounts {
        let row: (Option<u32>, bool) = sqlx::query_as(
            "SELECT MIN(s.height), a.use_internal FROM accounts a
            LEFT JOIN sync_heights s ON s.account = a.id_account
            WHERE a.id_account = ?",
        )
        .bind(account)
        .fetch_one(&mut *connection)
        .await
        .with_context(|| format!("observing account {account}"))?;
        match row {
            (Some(height), use_internal) => {
                debug!(
                    "Account {} - current DB sync height: {}, next sync height: {}, use_internal={}",
                    account,
                    height,
                    height + 1,
                    use_internal
                );
                states.push(AccountSyncState {
                    account,
                    next_height: height + 1,
                    use_internal,
                });
            }
            (None, _) => {
                debug!(
                    "Account {} - NO sync_heights entry found, will be skipped",
                    account
                );
            }
        }
    }
    Ok(states)
}

/// Observe, plan, and run every range; then merge the pending metadata.
async fn run_sync<S: Sink<SyncEvent> + Send + 'static>(
    progress: S,
    accounts: &[u32],
    params: &SyncParams,
    tx_cancel: &broadcast::Sender<()>,
    c: &Coin,
) -> Result<()> {
    let mut connection = c.get_connection().await?;

    let checkpoint_cutoff = params.current_height.saturating_sub(params.checkpoint_age);
    for account in accounts {
        prune_old_checkpoints(&mut connection, *account, checkpoint_cutoff).await?;
    }
    recover_from_partial_sync(&mut connection, accounts).await?;

    debug!("Current network height: {}", params.current_height);
    let states = observe_accounts(&mut connection, accounts).await?;
    let plan = plan_sync(&states, params.current_height);
    debug!(
        "Sync plan: {:?}",
        plan.iter()
            .map(|r| (r.start, r.end, r.account_ids()))
            .collect::<Vec<_>>()
    );

    let (tx_progress, mut rx_progress) = channel::<SyncProgress>(1);
    tokio::spawn(async move {
        while let Some(p) = rx_progress.recv().await {
            let _ = progress
                .send(SyncEvent::Progress {
                    height: p.height,
                    time: p.time,
                })
                .await;
        }
    });

    for range in &plan {
        run_range(&mut connection, range, params, &tx_progress, tx_cancel, c).await?;
    }

    for account in accounts {
        chain_log::apply_pending_metadata(&mut connection, *account, params.current_height)
            .await?;
    }
    Ok(())
}

/// Scan one range for its accounts: transparent, then shielded, then the
/// block times of the transactions found, then the detail pass.
async fn run_range(
    connection: &mut SqliteConnection,
    range: &SyncRange,
    params: &SyncParams,
    tx_progress: &Sender<SyncProgress>,
    tx_cancel: &broadcast::Sender<()>,
    c: &Coin,
) -> Result<()> {
    let network = c.network();
    let account_ids = range.account_ids();
    debug!(
        "Syncing accounts {:?} from height {} to {}",
        account_ids, range.start, range.end
    );

    let pool = c.get_pool()?;
    let mut client = c.client().await?;

    transparent_sync(
        &network,
        connection,
        &mut client,
        &account_ids,
        range.start,
        range.end,
        params.transparent_limit,
        tx_cancel.subscribe(),
    )
    .await?;

    // Over the mixnet, download blocks in small chunks so each
    // GetBlockRange stream fits in one nym-rpc session; elsewhere a
    // single stream for the whole range is cheaper.
    let chunk_size = c.is_mixnet().then_some(MIXNET_SYNC_CHUNK);
    shielded_sync(
        &network,
        &pool,
        &mut client,
        &range.accounts,
        range.start,
        range.end,
        params.actions_per_sync,
        chunk_size,
        tx_progress.clone(),
        tx_cancel.subscribe(),
    )
    .await?;

    debug!("heights_without_time");
    let heights_without_time =
        get_heights_without_time(connection, &account_ids, range.start, range.end).await?;
    for h in heights_without_time {
        debug!("fetch block @{h}");
        let block = client.block(&network, h).await?;
        let time = block.time;
        for account in &account_ids {
            chain_log::set_block_time(connection, *account, h, time, true).await?;
        }
        let block_header = BlockHeader {
            height: h,
            hash: block.hash,
            time: block.time,
        };
        for account in &account_ids {
            chain_log::record_relevant_header(connection, *account, &block_header).await?;
        }
    }

    if !params.skip_details {
        for account in &account_ids {
            crate::memo::fetch_tx_details(&network, connection, &mut client, *account).await?;
        }
    }

    debug!(
        "Sync completed for height range {}-{}",
        range.start, range.end
    );
    Ok(())
}

pub async fn recover_from_partial_sync(
    connection: &mut SqliteConnection,
    accounts: &[u32],
) -> Result<()> {
    for account in accounts {
        let account_heights = sqlx::query(
            "SELECT account, MIN(height) FROM sync_heights
            WHERE account = ?",
        )
        .bind(account)
        .map(|row: SqliteRow| {
            let account: u32 = row.get(0);
            let height: u32 = row.get(1);
            (account, height)
        })
        .fetch_all(&mut *connection)
        .await?;

        for (account, height) in account_heights {
            trim_sync_data(&mut *connection, account, height).await?;
        }
    }

    Ok(())
}

/// Remove the account's synchronization data after `height` (keeping the
/// data at `height`) in one transaction, or a savepoint when the caller
/// already holds one.
pub async fn trim_sync_data(
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<()> {
    let mut db_tx = connection.begin().await?;
    chain_log::trim_to(&mut db_tx, account, height).await?;
    db_tx.commit().await?;
    Ok(())
}

#[cfg(debug_assertions)]
pub async fn check_witness_consistency(
    connection: &mut SqliteConnection,
    accounts: &[u32],
) -> Result<()> {
    let notes = chain_log::notes_missing_witness(connection, accounts).await?;
    for n in notes.iter() {
        debug!(
            "Missing witness for note {} {} {} of account {} at height {}",
            n.pool, n.height, n.value, n.account, n.checkpoint
        );
    }
    if !notes.is_empty() {
        anyhow::bail!("Some notes have no witness data. Abort Sync");
    }
    debug!("Db check passed");
    Ok(())
}

#[cfg(not(debug_assertions))]
pub async fn check_witness_consistency(
    _connection: &mut SqliteConnection,
    _accounts: &[u32],
) -> Result<()> {
    Ok(())
}

// for each account, find the latest checkpoint before the given height
// and trim the synchronization data to that height
pub async fn rewind_sync(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<()> {
    let prev_height = chain_log::last_checkpoint_before(&mut *connection, account, height).await?;

    if let Some(prev_height) = prev_height {
        trim_sync_data(&mut *connection, account, prev_height).await?;
    } else {
        crate::account::reset_sync(network, &mut *connection, account).await?;
    }

    Ok(())
}

pub async fn prune_old_checkpoints(
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<()> {
    // keep the witnesses from the latest checkpoint before the given height
    if let Some(checkpoint_height) =
        chain_log::last_checkpoint_before(&mut *connection, account, height).await?
    {
        chain_log::prune_witnesses_before(&mut *connection, account, checkpoint_height).await?;
    }
    Ok(())
}

pub async fn get_db_height(connection: &mut SqliteConnection, account: u32) -> Result<SyncHeight> {
    // Use an outer join because the time stamp may not be present if we didn't
    // have to scan the chain (i.e. the account is transparent only)
    let (height, time): (u32, u32) = sqlx::query_as(
        "WITH mh AS (SELECT MIN(height) AS min_height
            FROM sync_heights
            WHERE account = ?1)
            SELECT mh.min_height, COALESCE(h.time, 0) FROM mh
            LEFT JOIN headers h ON h.height = mh.min_height AND h.account = ?1
            WHERE mh.min_height IS NOT NULL",
    )
    .bind(account)
    .fetch_one(connection)
    .await?;
    Ok(SyncHeight {
        pool: 0,
        height,
        time,
    })
}

pub async fn get_heights_without_time(
    connection: &mut SqliteConnection,
    accounts: &[u32],
    start: u32,
    end: u32,
) -> Result<HashSet<u32>> {
    let mut heights = HashSet::new();
    for account in accounts {
        let missing: Vec<u32> = sqlx::query_scalar(
            "SELECT height FROM transactions WHERE account = ?1 AND time = 0
                AND height >= ?2 AND height <= ?3
            UNION
            SELECT sh.height FROM sync_heights sh
                LEFT JOIN headers h ON sh.account = h.account AND sh.height = h.height
                WHERE sh.account = ?1 AND h.time IS NULL AND sh.height > 0",
        )
        .bind(account)
        .bind(start)
        .bind(end)
        .fetch_all(&mut *connection)
        .await?;
        heights.extend(missing);
    }
    Ok(heights)
}

#[cfg(test)]
mod outcome_tests {
    use super::*;

    #[test]
    fn cancel_wins_over_both_success_and_error() {
        assert_eq!(finish(Ok(()), true, 10).unwrap(), SyncOutcome::Cancelled);
        assert_eq!(
            finish(Err(anyhow::anyhow!("boom")), true, 10).unwrap(),
            SyncOutcome::Cancelled
        );
    }

    #[test]
    fn success_reports_the_height_and_errors_propagate() {
        assert_eq!(
            finish(Ok(()), false, 10).unwrap(),
            SyncOutcome::Synced { height: 10 }
        );
        assert!(finish(Err(anyhow::anyhow!("boom")), false, 10).is_err());
    }
}

#[cfg(test)]
mod header_tests {
    use super::*;
    use crate::db::create_schema;
    use chain_log::set_checkpoint;

    #[tokio::test]
    async fn backfill_headers_only_belong_to_relevant_accounts() -> Result<()> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        // Two owners share one fetch; account 3 needs the frontier without a
        // transaction; account 4 is unrelated, and account 5 is outside the batch.
        for account in [1, 2, 5] {
            sqlx::query(
                "INSERT INTO transactions(account, height, txid, time) VALUES (?, 100, X'01', 0)",
            )
            .bind(account)
            .execute(&mut db)
            .await?;
        }
        set_checkpoint(&mut db, 3, 1, 100).await?;
        let accounts = [1, 2, 3, 4];
        assert_eq!(
            get_heights_without_time(&mut db, &accounts, 1, 200).await?,
            HashSet::from([100])
        );
        let header = BlockHeader {
            height: 100,
            hash: vec![1; 32],
            time: 123,
        };
        for account in accounts {
            chain_log::record_relevant_header(&mut db, account, &header).await?;
        }
        let owners: Vec<u32> = sqlx::query_scalar("SELECT account FROM headers ORDER BY account")
            .fetch_all(&mut db)
            .await?;
        assert_eq!(owners, vec![1, 2, 3]);
        Ok(())
    }

    #[tokio::test]
    async fn rewinding_one_account_preserves_the_other_anchor() -> Result<()> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        for account in [1, 2] {
            set_checkpoint(&mut db, account, 1, 200).await?;
            for height in [100, 150, 200] {
                chain_log::store_header(
                    &mut db,
                    account,
                    &BlockHeader {
                        height,
                        hash: vec![1; 32],
                        time: height + account,
                    },
                )
                .await?;
            }
        }
        sqlx::query(
            "INSERT INTO witnesses(account, height, note, witness) VALUES (1, 100, 0, X'00')",
        )
        .execute(&mut db)
        .await?;
        rewind_sync(&Network::Main, &mut db, 1, 175).await?;
        let a = get_db_height(&mut db, 1).await?;
        let b = get_db_height(&mut db, 2).await?;
        assert_eq!((a.height, a.time), (100, 101));
        assert_eq!((b.height, b.time), (200, 202));
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM headers WHERE account = 1 AND height > 100")
                .fetch_one(&mut db)
                .await?;
        assert_eq!(count, 0);
        // A can scan a replacement chain without overwriting B's frontier.
        chain_log::store_header(
            &mut db,
            1,
            &BlockHeader {
                height: 200,
                hash: vec![2; 32],
                time: 999,
            },
        )
        .await?;
        let hashes: Vec<Vec<u8>> =
            sqlx::query_scalar("SELECT hash FROM headers WHERE height = 200 ORDER BY account")
                .fetch_all(&mut db)
                .await?;
        assert_eq!(hashes, vec![vec![2; 32], vec![1; 32]]);
        Ok(())
    }

    #[tokio::test]
    async fn missing_timestamps_are_optional_and_account_scoped() -> Result<()> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        set_checkpoint(&mut db, 1, 0, 100).await?;
        set_checkpoint(&mut db, 2, 0, 100).await?;
        chain_log::store_header(
            &mut db,
            2,
            &BlockHeader {
                height: 100,
                hash: vec![1; 32],
                time: 123,
            },
        )
        .await?;
        let a = get_db_height(&mut db, 1).await?;
        assert_eq!((a.height, a.time), (100, 0));
        assert_eq!(get_db_height(&mut db, 2).await?.time, 123);
        assert_eq!(
            get_heights_without_time(&mut db, &[1], 1, 200).await?,
            HashSet::from([100])
        );
        assert!(get_heights_without_time(&mut db, &[2], 1, 200)
            .await?
            .is_empty());
        assert!(get_db_height(&mut db, 3).await.is_err());
        Ok(())
    }
}
