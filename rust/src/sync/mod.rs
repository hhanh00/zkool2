use anyhow::{Context as _, Result};
use sqlx::SqliteConnection;
use tokio::sync::mpsc::{channel, Sender};
use tracing::{debug, info};

use crate::api::coin::{Coin, Network};
use crate::api::sync::{SyncEvent, SyncOutcome, SyncProgress};

use crate::io::SyncHeight;
use crate::Sink;
use std::collections::HashSet;

use sqlx::Connection;

pub mod chain_log;
pub mod bench;
pub mod driver;
pub mod fetch;
pub mod plan;
mod routes;
pub mod session;
mod shielded;
mod sweep;
mod transparent;

use plan::{plan_sync, AccountSyncState, SyncRange};
use session::{Held, SyncRun};

pub use shielded::{
    get_compact_block_range, get_tree_state, preload_account_key_cache, shielded_sync,
    shielded_sync_with, AccountKeyCache, ShieldedEnd, MIXNET_SYNC_CHUNK,
};
pub use sweep::transparent_sweep;
pub(crate) use transparent::transparent_sync;

pub use zcash_trees::types::SyncError;
pub use zcash_trees::types::{BlockHeader, Issuance, Note, Transaction, WarpSyncMessage, UTXO};

pub struct NoteExtended {
    pub id: u32,
    pub address: Vec<u8>,
    pub memo: Vec<u8>,
}
/// Synchronize `accounts` up to `current_height` under `run`, the wallet's
/// session the caller holds: the sync driver for a sync the app or the
/// server asked for, the protocol driver before it steps a DKG or signing
/// session, so its writes never overlap a sync in between.
///
/// Observe the accounts, plan the ranges to scan (see [`plan::plan_sync`]),
/// then run each range in order: transparent scan, shielded scan, block
/// times, and the detail pass unless `skip_details`. Progress streams through
/// `progress`; the returned outcome says how the run ended. An error is a
/// failed run; the work committed before it stays.
#[allow(clippy::too_many_arguments)]
pub async fn synchronize_within<S: Sink<SyncEvent> + Send + 'static>(
    run: &mut SyncRun<'_>,
    progress: S,
    accounts: Vec<u32>,
    current_height: u32,
    checkpoint_age: u32,
    skip_details: bool,
    c: &Coin,
) -> Result<SyncOutcome> {
    let accounts = normalize_accounts(accounts);
    if accounts.is_empty() {
        return Ok(SyncOutcome::Synced {
            height: current_height,
        });
    }
    let params = SyncParams {
        current_height,
        checkpoint_age,
        skip_details,
    };
    let result = run_sync(progress, &accounts, &params, run, c).await;

    // Inner operations may unwind with an error after receiving cancellation.
    // A user-requested stop is normal termination at the API boundary.
    let cancelled = run.was_cancelled();
    finish(result, cancelled, current_height)
}

/// The accounts to scan, each once: a repeated id scanned every address
/// and every key once per occurrence, and the second copy of a note it
/// found failed the range (#1510).
fn normalize_accounts(mut accounts: Vec<u32>) -> Vec<u32> {
    accounts.sort_unstable();
    accounts.dedup();
    accounts
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
    run: &SyncRun<'_>,
    c: &Coin,
) -> Result<()> {
    let mut connection = c.get_connection().await?;

    for account in accounts {
        prune_old_checkpoints(
            &mut connection,
            *account,
            params.current_height,
            params.checkpoint_age,
        )
        .await?;
    }
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

    // A reorg trims the fork and the run plans again from where the
    // database stands, each account from its own height (#1653).
    let mut last_reorg: Option<u32> = None;
    'plan: loop {
        recover_from_partial_sync(&c.network(), &mut connection, accounts).await?;

        debug!("Current network height: {}", params.current_height);
        let states = observe_accounts(&mut connection, accounts).await?;
        let plan = plan_sync(&states, params.current_height);
        debug!(
            "Sync plan: {:?}",
            plan.iter()
                .map(|r| (r.start, r.end, r.account_ids()))
                .collect::<Vec<_>>()
        );

        for range in &plan {
            if let ShieldedEnd::Rewound { detected_at } =
                run_range(&mut connection, range, params, &tx_progress, run, c).await?
            {
                check_reorg_progress(last_reorg, detected_at)?;
                info!("reorg detected at {detected_at}; planning again from the rewound heights");
                last_reorg = Some(detected_at);
                continue 'plan;
            }
        }
        break;
    }

    for account in accounts {
        chain_log::apply_pending_metadata(&mut connection, *account, params.current_height).await?;
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
    run: &SyncRun<'_>,
    c: &Coin,
) -> Result<ShieldedEnd> {
    let network = c.network();
    let account_ids = range.account_ids();
    debug!(
        "Syncing accounts {:?} from height {} to {}",
        account_ids, range.start, range.end
    );

    let pool = c.get_pool()?;
    let mut client = c.sync_client().await?;

    // A cancel is checked before every stage and inside the two loops that
    // have no scanner of their own, so none of them runs to completion
    // after the user asked to stop (#1571).
    run.check_cancelled()?;
    transparent_sync(
        &network,
        connection,
        &mut client,
        &account_ids,
        range.start,
        range.end,
        run.cancellation(),
    )
    .await?;

    // Over the mixnet, download blocks in small chunks so each
    // GetBlockRange stream fits in one nym-rpc session; elsewhere a
    // single stream for the whole range is cheaper.
    let chunk_size = c.is_mixnet().then_some(MIXNET_SYNC_CHUNK);
    // Over a direct lightwalletd connection a long range is downloaded on
    // several connections (see `fetch`); every other transport keeps its
    // one stream.
    let more_clients = (c.server_type == 0 && c.transport == 0 && !c.is_mixnet()).then(|| {
        let coin = c.clone();
        std::sync::Arc::new(move || {
            let coin = coin.clone();
            Box::pin(async move { coin.sync_client().await })
                as std::pin::Pin<Box<dyn std::future::Future<Output = Result<crate::Client>> + Send>>
        }) as fetch::ClientFactory
    });
    run.check_cancelled()?;
    let end = shielded_sync_with(
        &network,
        &pool,
        &mut client,
        &range.accounts,
        range.start,
        range.end,
        params.current_height,
        chunk_size,
        tx_progress.clone(),
        run.cancellation(),
        more_clients,
    )
    .await?;
    if let ShieldedEnd::Rewound { .. } = end {
        // The rewind trimmed the fork from every table, transparent ones
        // included; the next plan scans it again.
        return Ok(end);
    }

    debug!("heights_without_time");
    let heights_without_time =
        get_heights_without_time(connection, &account_ids, range.start, range.end).await?;
    for h in heights_without_time {
        run.check_cancelled()?;
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
            run.check_cancelled()?;
            crate::memo::fetch_tx_details(
                &network,
                &mut run.held(connection),
                &mut client,
                *account,
            )
            .await?;
        }
    }

    debug!(
        "Sync completed for height range {}-{}",
        range.start, range.end
    );
    Ok(ShieldedEnd::Done)
}

/// A reorg must be detected strictly below the previous one in the same
/// run. One at the same or a higher height means the server keeps serving
/// blocks that do not connect, and the run fails instead of looping (#1474).
fn check_reorg_progress(last_reorg: Option<u32>, detected_at: u32) -> Result<()> {
    if let Some(previous) = last_reorg {
        if detected_at >= previous {
            anyhow::bail!(
                "the server's blocks do not connect at height {detected_at} \
                 after a rewind for the same height: giving up"
            );
        }
    }
    Ok(())
}

/// Bring each account back to a state a scan can resume from, before a run
/// plans its ranges:
/// - every pool at the same height, the lowest one: a run that stopped
///   between the transparent and the shielded scan left them apart;
/// - every shielded note unspent at that height has a witness there. A scan
///   that resumed above a rewound account (#1653) left notes without one,
///   and those notes were never loaded again. Such an account goes back to
///   its last height where they all have one, or to its birth.
pub async fn recover_from_partial_sync(
    network: &Network,
    connection: &mut SqliteConnection,
    accounts: &[u32],
) -> Result<()> {
    for &account in accounts {
        let lowest: Option<u32> =
            sqlx::query_scalar("SELECT MIN(height) FROM sync_heights WHERE account = ?")
                .bind(account)
                .fetch_one(&mut *connection)
                .await?;
        let Some(lowest) = lowest else { continue };
        match chain_log::last_resumable_at_or_below(&mut *connection, account, lowest).await? {
            Some(height) => {
                if height < lowest {
                    info!("account {account}: notes without a witness at {lowest}; rescanning from {height}");
                }
                trim_sync_data(&mut *connection, account, height).await?;
            }
            None => {
                info!("account {account}: notes without a witness at {lowest} and no earlier point; rescanning from birth");
                crate::account::reset_within(network, &mut *connection, account).await?;
            }
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

/// Rewind `account` to the last height below `height` a scan can resume
/// from (see [`chain_log::last_rewind_point_before`]), or to its birth
/// when it has none. For a caller holding the session: the sync
/// driver, for the user's rewind.
pub async fn rewind_sync(
    network: &Network,
    connection: &mut Held<'_>,
    account: u32,
    height: u32,
) -> Result<()> {
    rewind_within(network, connection, account, height).await
}

/// [`rewind_sync`] from inside a sync run, on its own transaction: the
/// scan found a reorg. Called under the session by construction.
pub(crate) async fn rewind_within(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<()> {
    let prev_height =
        chain_log::last_rewind_point_before(&mut *connection, account, height).await?;

    if let Some(prev_height) = prev_height {
        trim_sync_data(&mut *connection, account, prev_height).await?;
    } else {
        crate::account::reset_within(network, &mut *connection, account).await?;
    }

    Ok(())
}

/// Drop the witnesses and headers of `account` that no rewind can need:
/// those below its latest rewind point at or below the cutoff, which is
/// `checkpoint_age` below the tip and never above the final blocks. The
/// point itself is kept, and it is never above the account's own sync
/// height, so an account behind the cutoff keeps where it resumes from.
pub async fn prune_old_checkpoints(
    connection: &mut SqliteConnection,
    account: u32,
    tip: u32,
    checkpoint_age: u32,
) -> Result<()> {
    let cutoff = tip.saturating_sub(checkpoint_age.max(chain_log::FINALITY_DEPTH));
    if let Some(point) =
        chain_log::last_rewind_point_before(&mut *connection, account, cutoff.saturating_add(1))
            .await?
    {
        chain_log::prune_before(&mut *connection, account, point).await?;
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
    fn accounts_are_scanned_once_however_the_list_was_built() {
        assert_eq!(normalize_accounts(vec![1, 1, 1]), vec![1]);
        assert_eq!(normalize_accounts(vec![3, 1, 2, 1]), vec![1, 2, 3]);
        assert!(normalize_accounts(vec![]).is_empty());
    }

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
        // Account 1 holds an unspent Sapling note with a witness only at
        // 100, so that is where a rewind can land.
        sqlx::query("INSERT INTO sapling_accounts(account) VALUES (1)")
            .execute(&mut db)
            .await?;
        sqlx::query(
            "INSERT INTO notes(id_note, height, account, pool, nullifier, tx, value)
            VALUES (0, 90, 1, 1, X'00', 0, 1)",
        )
        .execute(&mut db)
        .await?;
        sqlx::query(
            "INSERT INTO witnesses(account, height, note, witness) VALUES (1, 100, 0, X'00')",
        )
        .execute(&mut db)
        .await?;
        let session = session::SyncSession::new();
        let run = session.try_begin().unwrap();
        rewind_sync(&Network::Main, &mut run.held(&mut db), 1, 175).await?;
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

#[cfg(test)]
mod rewind_tests {
    use super::*;
    use crate::db::create_schema;
    use chain_log::{is_final, last_rewind_point_before, set_checkpoint, FINALITY_DEPTH};

    async fn wallet() -> Result<SqliteConnection> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        Ok(db)
    }

    async fn shielded(db: &mut SqliteConnection, account: u32) -> Result<()> {
        sqlx::query("INSERT INTO sapling_accounts(account) VALUES (?)")
            .bind(account)
            .execute(&mut *db)
            .await?;
        Ok(())
    }

    /// Every pool of the account at `height`.
    async fn synced(db: &mut SqliteConnection, account: u32, height: u32) -> Result<()> {
        for pool in 0..3 {
            set_checkpoint(db, account, pool, height).await?;
        }
        Ok(())
    }

    async fn headers(db: &mut SqliteConnection, account: u32, heights: &[u32]) -> Result<()> {
        for &height in heights {
            chain_log::store_header(
                db,
                account,
                &BlockHeader {
                    height,
                    hash: vec![1; 32],
                    time: height,
                },
            )
            .await?;
        }
        Ok(())
    }

    async fn note(db: &mut SqliteConnection, account: u32, id: u32, height: u32) -> Result<()> {
        sqlx::query(
            "INSERT INTO notes(id_note, height, account, pool, nullifier, tx, value)
            VALUES (?1, ?2, ?3, 1, ?4, 0, 1)",
        )
        .bind(id)
        .bind(height)
        .bind(account)
        .bind(id.to_le_bytes().to_vec())
        .execute(&mut *db)
        .await?;
        Ok(())
    }

    async fn spend(db: &mut SqliteConnection, account: u32, id: u32, height: u32) -> Result<()> {
        sqlx::query(
            "INSERT INTO spends(id_note, height, account, pool, tx, value)
            VALUES (?, ?, ?, 1, 0, 1)",
        )
        .bind(id)
        .bind(height)
        .bind(account)
        .execute(&mut *db)
        .await?;
        Ok(())
    }

    async fn witnesses(
        db: &mut SqliteConnection,
        account: u32,
        id: u32,
        heights: &[u32],
    ) -> Result<()> {
        for &height in heights {
            sqlx::query(
                "INSERT INTO witnesses(account, height, note, witness) VALUES (?, ?, ?, X'00')",
            )
            .bind(account)
            .bind(height)
            .bind(id)
            .execute(&mut *db)
            .await?;
        }
        Ok(())
    }

    async fn heights(db: &mut SqliteConnection, table: &str, account: u32) -> Result<Vec<u32>> {
        let h = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT DISTINCT height FROM {table} WHERE account = ? ORDER BY height"
        )))
        .bind(account)
        .fetch_all(&mut *db)
        .await?;
        Ok(h)
    }

    #[tokio::test]
    async fn an_account_without_notes_goes_back_one_sync_not_to_its_birth() -> Result<()> {
        let mut db = wallet().await?;
        shielded(&mut db, 1).await?;
        synced(&mut db, 1, 300).await?;
        headers(&mut db, 1, &[100, 200, 300]).await?;
        rewind_within(&Network::Main, &mut db, 1, 300).await?;
        assert_eq!(get_db_height(&mut db, 1).await?.height, 200);
        assert_eq!(heights(&mut db, "headers", 1).await?, vec![100, 200]);
        Ok(())
    }

    #[tokio::test]
    async fn the_witness_of_a_spent_note_does_not_hold_the_rewind_back() -> Result<()> {
        // Account 8 of #1653: its only witness belongs to a note spent since.
        let mut db = wallet().await?;
        shielded(&mut db, 1).await?;
        synced(&mut db, 1, 300).await?;
        headers(&mut db, 1, &[100, 200, 300]).await?;
        note(&mut db, 1, 1, 50).await?;
        witnesses(&mut db, 1, 1, &[100]).await?;
        spend(&mut db, 1, 1, 150).await?;
        assert_eq!(last_rewind_point_before(&mut db, 1, 300).await?, Some(200));
        Ok(())
    }

    #[tokio::test]
    async fn a_note_unspent_at_the_point_needs_its_witness_there() -> Result<()> {
        let mut db = wallet().await?;
        shielded(&mut db, 1).await?;
        synced(&mut db, 1, 300).await?;
        // 250 is a transaction's header, not a checkpoint: no witness there.
        headers(&mut db, 1, &[100, 200, 250, 300]).await?;
        note(&mut db, 1, 1, 50).await?;
        witnesses(&mut db, 1, 1, &[100, 200, 300]).await?;
        assert_eq!(last_rewind_point_before(&mut db, 1, 300).await?, Some(200));
        // A note received after the point does not need a witness there.
        note(&mut db, 1, 2, 220).await?;
        witnesses(&mut db, 1, 2, &[300]).await?;
        assert_eq!(last_rewind_point_before(&mut db, 1, 300).await?, Some(200));
        Ok(())
    }

    #[tokio::test]
    async fn an_account_without_a_shielded_pool_needs_no_header() -> Result<()> {
        let mut db = wallet().await?;
        synced(&mut db, 1, 300).await?;
        assert_eq!(last_rewind_point_before(&mut db, 1, 300).await?, Some(299));
        Ok(())
    }

    #[tokio::test]
    async fn a_rewind_never_moves_a_pool_forward() -> Result<()> {
        let mut db = wallet().await?;
        shielded(&mut db, 1).await?;
        synced(&mut db, 1, 300).await?;
        set_checkpoint(&mut db, 1, 1, 200).await?;
        headers(&mut db, 1, &[100, 200, 250, 300]).await?;
        assert_eq!(last_rewind_point_before(&mut db, 1, 300).await?, Some(200));
        Ok(())
    }

    #[tokio::test]
    async fn no_point_when_nothing_qualifies() -> Result<()> {
        let mut db = wallet().await?;
        shielded(&mut db, 1).await?;
        synced(&mut db, 1, 300).await?;
        headers(&mut db, 1, &[100, 200, 300]).await?;
        note(&mut db, 1, 1, 50).await?;
        witnesses(&mut db, 1, 1, &[300]).await?;
        assert_eq!(last_rewind_point_before(&mut db, 1, 300).await?, None);
        // An account that never synced has no checkpoint at all.
        assert_eq!(last_rewind_point_before(&mut db, 2, 300).await?, None);
        Ok(())
    }

    #[tokio::test]
    async fn a_run_rolls_back_an_account_whose_notes_lost_their_witnesses() -> Result<()> {
        // Accounts 7, 9 and 13 of #1653: synced to 300, but the note's
        // witnesses stopped at 100.
        let mut db = wallet().await?;
        shielded(&mut db, 1).await?;
        synced(&mut db, 1, 300).await?;
        headers(&mut db, 1, &[100, 300]).await?;
        note(&mut db, 1, 1, 50).await?;
        witnesses(&mut db, 1, 1, &[100]).await?;
        // A healthy account next to it does not move.
        shielded(&mut db, 2).await?;
        synced(&mut db, 2, 300).await?;
        headers(&mut db, 2, &[100, 300]).await?;
        note(&mut db, 2, 2, 50).await?;
        witnesses(&mut db, 2, 2, &[100, 300]).await?;

        recover_from_partial_sync(&Network::Main, &mut db, &[1, 2]).await?;
        assert_eq!(get_db_height(&mut db, 1).await?.height, 100);
        assert_eq!(heights(&mut db, "headers", 1).await?, vec![100]);
        assert_eq!(get_db_height(&mut db, 2).await?.height, 300);
        assert_eq!(heights(&mut db, "witnesses", 2).await?, vec![100, 300]);
        Ok(())
    }

    #[tokio::test]
    async fn a_run_levels_the_pools_as_before() -> Result<()> {
        let mut db = wallet().await?;
        synced(&mut db, 1, 300).await?;
        set_checkpoint(&mut db, 1, 0, 250).await?;
        recover_from_partial_sync(&Network::Main, &mut db, &[1]).await?;
        let pools: Vec<u32> =
            sqlx::query_scalar("SELECT height FROM sync_heights WHERE account = 1")
                .fetch_all(&mut db)
                .await?;
        assert_eq!(pools, vec![250, 250, 250]);
        Ok(())
    }

    #[tokio::test]
    async fn pruning_keeps_one_point_below_the_cutoff_and_every_block_that_is_not_final(
    ) -> Result<()> {
        let mut db = wallet().await?;
        shielded(&mut db, 1).await?;
        synced(&mut db, 1, 1000).await?;
        headers(&mut db, 1, &[850, 880, 905, 950, 1000]).await?;
        note(&mut db, 1, 1, 800).await?;
        witnesses(&mut db, 1, 1, &[850, 880, 905, 950, 1000]).await?;
        // An age below the finality depth is raised to it: the cutoff is 900.
        prune_old_checkpoints(&mut db, 1, 1000, 1).await?;
        assert_eq!(heights(&mut db, "headers", 1).await?, vec![880, 905, 950, 1000]);
        assert_eq!(heights(&mut db, "witnesses", 1).await?, vec![880, 905, 950, 1000]);
        Ok(())
    }

    #[tokio::test]
    async fn pruning_keeps_where_an_account_behind_the_cutoff_resumes() -> Result<()> {
        let mut db = wallet().await?;
        shielded(&mut db, 1).await?;
        synced(&mut db, 1, 300).await?;
        headers(&mut db, 1, &[200, 300]).await?;
        prune_old_checkpoints(&mut db, 1, 1000, 1).await?;
        assert_eq!(heights(&mut db, "headers", 1).await?, vec![300]);
        assert_eq!(get_db_height(&mut db, 1).await?.height, 300);
        Ok(())
    }

    #[test]
    fn blocks_a_hundred_deep_are_final() {
        assert!(is_final(900, 900 + FINALITY_DEPTH));
        assert!(!is_final(901, 900 + FINALITY_DEPTH));
        assert!(!is_final(0, 50));
    }

    #[test]
    fn each_reorg_of_a_run_must_be_found_lower_than_the_last() {
        assert!(check_reorg_progress(None, 500).is_ok());
        assert!(check_reorg_progress(Some(500), 499).is_ok());
        assert!(check_reorg_progress(Some(500), 500).is_err());
        assert!(check_reorg_progress(Some(500), 501).is_err());
    }
}
