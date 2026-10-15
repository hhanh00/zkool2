use anyhow::{Context as _, Result};
use sqlx::SqliteConnection;
use sqlx::{sqlite::SqliteRow, Row};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tokio::sync::mpsc::channel;
use tracing::debug;

use crate::api::coin::{Coin, Network};
use crate::api::sync::{CANCEL_SYNC, SYNCING};

use crate::io::SyncHeight;
use crate::Sink;
use std::collections::HashSet;

use crate::api::sync::SyncProgress;
use sqlx::Connection;

pub mod chain_log;
mod shielded;
mod sweep;
mod transparent;

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
#[allow(clippy::too_many_arguments)]
pub async fn synchronize_impl<S: Sink<SyncProgress> + Send + 'static>(
    progress: S,
    accounts: Vec<u32>,
    current_height: u32,
    actions_per_sync: u32,
    transparent_limit: u32,
    checkpoint_age: u32,
    noskip_details: bool,
    c: &Coin,
) -> Result<u32> {
    if accounts.is_empty() {
        return Ok(current_height);
    }

    let Ok(_guard) = SYNCING.try_lock() else {
        return Ok(current_height);
    };

    let (tx_cancel, mut rx_cancel) = broadcast::channel::<()>(16);
    {
        let mut cancel = CANCEL_SYNC.lock().await;
        *cancel = Some(tx_cancel.clone());
    }

    let network = c.network();
    let mut connection = c.get_connection().await?;
    let progress2 = progress.clone();

    let checkpoint_cutoff = current_height.saturating_sub(checkpoint_age);
    for account in accounts.iter() {
        prune_old_checkpoints(&mut connection, *account, checkpoint_cutoff).await?;
    }

    let mut account_use_internal = HashMap::<u32, bool>::new();
    let res = async {
        recover_from_partial_sync(&mut connection, &accounts).await?;

        // Get account heights
        let mut account_heights = HashMap::new();
        debug!("Current network height: {}", current_height);
        for account in accounts.iter() {
            let r: (Option<u32>, Option<u32>) = sqlx::query_as(
                r#"SELECT account, MIN(height) FROM sync_heights
                JOIN accounts ON account = id_account
                WHERE account = ?"#,
            )
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;
            if let (Some(account), Some(height)) = r {
                debug!(
                    "Account {} - current DB sync height: {}, next sync height: {}",
                    account,
                    height,
                    height + 1
                );
                account_heights.insert(account, height + 1);

                let (use_internal,): (bool,) =
                    sqlx::query_as("SELECT use_internal FROM accounts WHERE id_account = ?")
                        .bind(account)
                        .fetch_one(&mut *connection)
                        .await
                        .context("Fetch use_internal")?;
                account_use_internal.insert(account, use_internal);

                // Check which pools this account has
                let t_count: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM transparent_address_accounts WHERE account = ?",
                )
                .bind(account)
                .fetch_one(&mut *connection)
                .await?;
                debug!(
                    "Account {} - has {} transparent addresses, use_internal={}",
                    account, t_count.0, use_internal
                );
            } else {
                debug!(
                    "Account {} - NO sync_heights entry found, will be skipped",
                    account
                );
            }
        }

        // Create a sorted list of unique heights
        let mut unique_heights: Vec<u32> = account_heights.values().cloned().collect();
        unique_heights.sort_unstable();
        unique_heights.dedup();
        debug!(
            "Unique sync start heights for accounts: {:?}",
            unique_heights
        );

        let (tx_progress, mut rx_progress) = channel::<SyncProgress>(1);

        tokio::spawn(async move {
            while let Some(p) = rx_progress.recv().await {
                let _ = progress.send(p).await;
            }
        });

        // For each unique height, process accounts that need to be synced from that height
        for (i, &start_height) in unique_heights.iter().enumerate() {
            // Determine the end height (next height - 1 or current_height)
            let end_height = if i + 1 < unique_heights.len() {
                unique_heights[i + 1] - 1
            } else {
                current_height
            };

            // Find accounts that have a height <= this start_height
            let accounts_to_sync = account_heights
                .iter()
                .filter(|&(_, &height)| height <= start_height)
                .map(|(&account, _)| {
                    let use_internal = account_use_internal[&account];
                    (account, use_internal)
                })
                .collect::<Vec<_>>();

            // Skip if no accounts to sync
            if accounts_to_sync.is_empty() {
                debug!("No accounts to sync for start_height {}", start_height);
                continue;
            }

            debug!(
                "Syncing accounts {:?} from height {} to {}",
                accounts_to_sync.iter().map(|(a, _)| a).collect::<Vec<_>>(),
                start_height,
                end_height
            );

            let pool = c.get_pool()?;
            // Update the sync heights for these accounts
            let mut client = c.client().await?;

            debug!("Start height: {}", start_height);
            debug!("End height: {}", end_height);

            if start_height > end_height {
                debug!("Skipping sync: start_height ({}) > end_height ({}), wallet is ahead of network", start_height, end_height);
                return Ok(());
            }

            let account_ids = accounts_to_sync
                .iter()
                .map(|(account, _)| *account)
                .collect::<Vec<_>>();
            transparent_sync(
                &network,
                &mut connection,
                &mut client,
                &account_ids,
                start_height,
                end_height,
                transparent_limit,
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
                &accounts_to_sync,
                start_height,
                end_height,
                actions_per_sync,
                chunk_size,
                tx_progress.clone(),
                tx_cancel.subscribe(),
            )
            .await?;

            debug!("heights_without_time");
            let heights_without_time =
                get_heights_without_time(&mut connection, &account_ids, start_height, end_height)
                    .await?;
            for h in heights_without_time {
                debug!("fetch block @{h}");
                let block = client.block(&network, h).await?;
                let time = block.time;
                for account in &account_ids {
                    chain_log::set_block_time(&mut connection, *account, h, time, true).await?;
                }
                let block_header = BlockHeader {
                    height: h,
                    hash: block.hash,
                    time: block.time,
                };
                for account in &account_ids {
                    chain_log::record_relevant_header(&mut connection, *account, &block_header)
                        .await?;
                }
            }

            // Update our local map as well for the next iteration
            for (account, _) in &accounts_to_sync {
                account_heights.insert(*account, end_height);
                if !noskip_details {
                    crate::memo::fetch_tx_details(&network, &mut connection, &mut client, *account)
                        .await?;
                }
            }

            debug!(
                "Sync completed for height range {}-{}",
                start_height, end_height
            );
        }

        for account in accounts.iter() {
            chain_log::apply_pending_metadata(&mut connection, *account, current_height).await?;
        }

        Ok::<_, anyhow::Error>(())
    };

    let result = res.await;
    // Inner operations may unwind with an error after receiving cancellation.
    // A user-requested stop is normal termination at the API boundary.
    let cancelled = rx_cancel.try_recv().is_ok();
    if cancelled {
        debug!("Synchronization cancelled by user");
    } else if let Err(e) = result {
        debug!("Error during sync: {:?}", e);
        progress2.send_error(e).await;
    }

    {
        let mut cancel = CANCEL_SYNC.lock().await;
        *cancel = None;
    }

    Ok(current_height)
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
