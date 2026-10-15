use anyhow::{Context as _, Result};
use sqlx::SqliteConnection;
use sqlx::{sqlite::SqliteRow, Row};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;
use tracing::{debug, info};
use zcash_transparent::address::TransparentAddress;

use crate::api::account::get_ledger;
use crate::api::coin::{Coin, Network};
use crate::api::sync::{CANCEL_SYNC, SYNCING};

use crate::io::SyncHeight;
use crate::{Client, Sink};
use std::{collections::HashSet, mem};

use crate::{
    account::{derive_transparent_address, derive_transparent_sk, get_birth_height},
    api::sync::SyncProgress,
    db::{
        get_account_aindex, get_account_dindex, get_account_hw, select_account_transparent,
        store_account_transparent_addr,
    },
    lwd::CompactBlock,
    warp::{legacy::CommitmentTreeFrontier, sync::warp_sync},
};
use bincode::config;
use chain_log::{NoteKey, NoteRow, SpendRow, TransactionRow};
use sqlx::pool::PoolConnection;
use sqlx::{Connection, Sqlite, SqlitePool};

pub mod chain_log;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;
use zcash_keys::encoding::AddressCodec;
use zcash_protocol::consensus::{NetworkUpgrade, Parameters};

pub const DEFAULT_ACTIONS_PER_SYNC: u32 = 10000u32;
pub const DEFAULT_TRANSPARENT_LIMIT: u32 = 100u32;

pub use zcash_trees::types::SyncError;
pub use zcash_trees::types::{BlockHeader, Issuance, Note, Transaction, WarpSyncMessage, UTXO};

pub struct NoteExtended {
    pub id: u32,
    pub address: Vec<u8>,
    pub memo: Vec<u8>,
}

/// Pre-derived Sapling account keys — loaded once before sync.
pub struct SaplingAccountKeys {
    pub dfvk: sapling_crypto::zip32::DiversifiableFullViewingKey,
    pub external_ivk: sapling_crypto::keys::SaplingIvk,
    pub internal_ivk: sapling_crypto::keys::SaplingIvk,
    pub external_nk: sapling_crypto::keys::NullifierDerivingKey,
    pub internal_nk: sapling_crypto::keys::NullifierDerivingKey,
}

/// Pre-derived Orchard account keys — loaded once before sync.
pub struct OrchardAccountKeys {
    pub fvk: orchard::keys::FullViewingKey,
    pub external_ivk: orchard::keys::IncomingViewingKey,
    pub internal_ivk: orchard::keys::IncomingViewingKey,
    // Orchard NK = FullViewingKey (see warp/sync/shielded/orchard.rs line 27)
}

/// Cache of all per-account key material needed during shielded sync.
/// Preloaded once and shared via Arc between the db_writer and warp_sync tasks.
pub struct AccountKeyCache {
    pub sapling: HashMap<u32, SaplingAccountKeys>,
    pub orchard: HashMap<u32, OrchardAccountKeys>,
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

#[allow(clippy::too_many_arguments)]
pub(crate) async fn transparent_sync(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    accounts: &[u32],
    start_height: u32,
    end_height: u32,
    limit: u32,
    mut rx_cancel: broadcast::Receiver<()>,
) -> Result<()> {
    let mut addresses = vec![];
    debug!(
        "transparent_sync: scanning accounts {:?} from height {} to {} with limit {}",
        accounts, start_height, end_height, limit
    );
    for account in accounts {
        // scan the most recent receive and change addresses, bounded by `limit`
        let mut rows = sqlx::query("
                WITH receive AS
                (SELECT * FROM transparent_address_accounts WHERE account = ?1 AND scope = 0 ORDER BY dindex DESC LIMIT ?2),
                change AS
                (SELECT * FROM transparent_address_accounts WHERE account = ?1 AND scope = 1 ORDER BY dindex DESC LIMIT ?2)
                SELECT id_taddress, address FROM receive UNION ALL SELECT id_taddress, address FROM change")
            .bind(account)
            .bind(limit)
            .map(|row: SqliteRow| {
                let id_taddress: u32 = row.get(0);
                let address: String = row.get(1);
                (id_taddress, address)
            })
            .fetch(&mut *connection);

        let mut addr_count = 0u32;
        while let Some((id_taddress, address)) = rows.try_next().await? {
            debug!(
                "transparent_sync: account {} has taddress id={} addr={}",
                account, id_taddress, address
            );
            addr_count += 1;
            // Add the address to the client
            addresses.push((*account, (id_taddress, address)));
        }
        debug!(
            "transparent_sync: account {} has {} transparent addresses to scan",
            account, addr_count
        );
    }
    debug!(
        "transparent_sync: total {} addresses to scan across all accounts",
        addresses.len()
    );
    for (account, address_row) in addresses.iter() {
        let my_address = TransparentAddress::decode(&network, &address_row.1)?;
        debug!(
            "transparent_sync: scanning account {} address {} (decoded: {:?})",
            account,
            address_row.1,
            my_address.encode(network)
        );
        let mut txs = client
            .taddress_txs(network, &address_row.1, start_height, end_height)
            .await?
            .into_inner();

        let mut db_tx = connection.begin().await?;
        loop {
            tokio::select! {
                _ = rx_cancel.recv() => {
                    debug!("Canceling sync");
                    anyhow::bail!("Sync canceled");
                }
                m = txs.recv() => {
                    if let Some((height, transaction, _)) = m {
                        let txid = transaction.txid().as_ref().to_vec();
                        debug!(
                            "transparent_sync: found tx {} at height {} for account {} version={:?} branch_id={:?}",
                            hex::encode(&txid),
                            height,
                            account,
                            transaction.version(),
                            transaction.consensus_branch_id(),
                        );
                        // tx time is available in the block (not here)
                        let inserted = chain_log::record_transaction(
                            &mut db_tx,
                            *account,
                            &TransactionRow::chain(&txid, height, 0),
                        )
                        .await?;
                        debug!(
                            "transparent_sync: tx {} inserted into transactions (new={})",
                            hex::encode(&txid),
                            inserted.is_some()
                        );
                        let (id_tx, _) = chain_log::transaction_ref(&mut db_tx, *account, &txid).await?;

                        // Access the transparent bundle part
                        if let Some(transparent_bundle) = transaction.transparent_bundle() {
                            debug!(
                                "transparent_sync: tx {} has transparent bundle: {} vins, {} vouts",
                                transaction.txid(),
                                transparent_bundle.vin.len(),
                                transparent_bundle.vout.len()
                            );

                            let vins = &transparent_bundle.vin;
                            for vin in vins.iter() {
                                // The "nullifier" of a transparent input is the outpoint
                                let mut nf = vec![];
                                vin.prevout().write(&mut nf)?;

                                let row =
                                    chain_log::find_note(&mut db_tx, *account, NoteKey::Nullifier(&nf))
                                        .await?;

                                if let Some((id, amount)) = row {
                                    debug!(
                                        "transparent_sync: tx {} vin spends note {} amount {}",
                                        transaction.txid(),
                                        id,
                                        amount
                                    );
                                    // note was found: add a spent entry (this also
                                    // releases any broadcast-time lock)
                                    chain_log::record_spend(
                                        &mut db_tx,
                                        *account,
                                        &SpendRow {
                                            note: NoteKey::Id(id),
                                            id_tx,
                                            height,
                                            pool: 0,
                                            value: -amount,
                                        },
                                    )
                                    .await?;
                                }
                            }

                            let vouts = &transparent_bundle.vout;
                            for (i, vout) in vouts.iter().enumerate() {
                                let vout_value = vout.value().into_u64();
                                if let Some(vout_addr) = vout.recipient_address() {
                                    let vout_addr_encoded = vout_addr.encode(network);
                                    let my_addr_encoded = my_address.encode(network);
                                    let is_match = vout_addr == my_address;
                                    debug!(
                                        "transparent_sync: tx {} vout[{}] value={} recipient={} my_address={} match={}",
                                        transaction.txid(),
                                        i,
                                        vout_value,
                                        vout_addr_encoded,
                                        my_addr_encoded,
                                        is_match,
                                    );
                                    if is_match {
                                        // It is for me
                                        // add a new note entry
                                        let mut nf = transaction.txid().as_ref().to_vec();
                                        nf.extend_from_slice(&(i as u32).to_le_bytes());

                                        let note = chain_log::record_note(
                                            &mut db_tx,
                                            *account,
                                            &NoteRow {
                                                id_tx,
                                                height,
                                                pool: 0,
                                                nullifier: &nf,
                                                value: vout_value as i64,
                                                taddress: Some(address_row.0),
                                                ..Default::default()
                                            },
                                        )
                                        .await?;
                                        debug!(
                                            "transparent_sync: tx {} vout[{}] NOTE CREATED value={} new={}",
                                            transaction.txid(),
                                            i,
                                            vout_value,
                                            note.is_some()
                                        );
                                    }
                                } else {
                                    debug!(
                                        "transparent_sync: tx {} vout[{}] value={} has NO recipient address (script cannot be decoded)",
                                        transaction.txid(),
                                        i,
                                        vout_value,
                                    );
                                }
                            }
                        } else {
                            debug!(
                                "transparent_sync: tx {} has NO transparent bundle (shielded-only tx) version={:?} branch_id={:?} height={}",
                                transaction.txid(),
                                transaction.version(),
                                transaction.consensus_branch_id(),
                                height,
                            );
                        }
                    }
                    else {
                        // No more transactions
                        break;
                    }
                }
            }
        }

        chain_log::checkpoint(&mut db_tx, *account, 0, end_height).await?;
        db_tx.commit().await?;
    }

    Ok(())
}

pub async fn get_compact_block_range(
    network: &Network,
    client: &mut Client,
    start: u32,
    end: u32,
) -> Result<ReceiverStream<CompactBlock>> {
    let blocks = client.block_range(network, start, end).await?;
    Ok(blocks)
}

pub async fn get_tree_state(
    network: &Network,
    client: &mut Client,
    height: u32,
) -> Result<(
    CommitmentTreeFrontier,
    CommitmentTreeFrontier,
    CommitmentTreeFrontier,
)> {
    let min_height: u32 = network
        .activation_height(zcash_protocol::consensus::NetworkUpgrade::Sapling)
        .unwrap()
        .into();

    if height < min_height {
        return Ok((
            CommitmentTreeFrontier::default(),
            CommitmentTreeFrontier::default(),
            CommitmentTreeFrontier::default(),
        ));
    }

    let (sapling_tree, orchard_tree, ironwood_tree) = client.tree_state(height).await?;

    fn decode_tree_state(tree: &[u8]) -> CommitmentTreeFrontier {
        if tree.is_empty() {
            CommitmentTreeFrontier::default()
        } else {
            CommitmentTreeFrontier::read(tree).unwrap()
        }
    }

    let sapling = decode_tree_state(&sapling_tree);
    let orchard = decode_tree_state(&orchard_tree);
    let ironwood = decode_tree_state(&ironwood_tree);

    Ok((sapling, orchard, ironwood))
}

/// Preload all Sapling and Orchard account keys from the database.
/// All key derivations happen exactly once, before any sync work starts.
pub async fn preload_account_key_cache(
    connection: &mut SqliteConnection,
) -> Result<AccountKeyCache> {
    let mut sapling = HashMap::new();
    let sapling_rows: Vec<(u32, Vec<u8>)> =
        sqlx::query_as("SELECT account, xvk FROM sapling_accounts")
            .fetch_all(&mut *connection)
            .await?;

    for (account, xvk) in sapling_rows {
        let dfvk = sapling_crypto::zip32::DiversifiableFullViewingKey::from_bytes(
            &xvk.try_into().unwrap(),
        )
        .unwrap();
        let external_ivk = dfvk.fvk().vk.ivk();
        let internal_ivk = dfvk.to_internal_fvk().vk.ivk();
        let external_nk = *dfvk.fvk().vk.nk();
        let internal_nk = *dfvk.to_internal_fvk().vk.nk();
        sapling.insert(
            account,
            SaplingAccountKeys {
                dfvk,
                external_ivk,
                internal_ivk,
                external_nk,
                internal_nk,
            },
        );
    }

    let mut orchard = HashMap::new();
    let orchard_rows: Vec<(u32, Vec<u8>)> =
        sqlx::query_as("SELECT account, xvk FROM orchard_accounts")
            .fetch_all(&mut *connection)
            .await?;

    for (account, xvk) in orchard_rows {
        let fvk = orchard::keys::FullViewingKey::from_bytes(&xvk.try_into().unwrap()).unwrap();
        let external_ivk = fvk.to_ivk(orchard::keys::Scope::External);
        let internal_ivk = fvk.to_ivk(orchard::keys::Scope::Internal);
        orchard.insert(
            account,
            OrchardAccountKeys {
                fvk,
                external_ivk,
                internal_ivk,
            },
        );
    }

    Ok(AccountKeyCache { sapling, orchard })
}

/// Resolve the diversifier index from a note's raw diversifier bytes.
/// Pure computation — no DB access, uses the preloaded key cache.
fn resolve_diversifier_index(
    cache: &AccountKeyCache,
    account: u32,
    pool: u8,
    scope: u8,
    diversifier: &[u8],
) -> Option<i64> {
    match pool {
        1 => cache.sapling.get(&account).and_then(|keys| {
            crate::db::resolve_sapling_diversifier_index(&keys.dfvk, scope, diversifier)
        }),
        2 | 3 => cache.orchard.get(&account).and_then(|keys| {
            crate::db::resolve_orchard_diversifier_index(&keys.fvk, scope, diversifier)
        }),
        _ => None,
    }
}

/// Blocks per `GetBlockRange` request when syncing over the Nym mixnet.
/// A single full-range stream starves the nym-rpc session of reply SURBs
/// and trips its 120s stall watchdog; small chunks keep each stream within
/// one session's budget and commit progress between chunks, so a dropped
/// session only costs the current chunk.
pub const MIXNET_SYNC_CHUNK: u32 = 1000;

#[allow(clippy::too_many_arguments)]
pub async fn shielded_sync(
    network: &Network,
    pool: &SqlitePool,
    client: &mut Client,
    accounts: &[(u32, bool)],
    start: u32,
    end: u32,
    actions_per_sync: u32,
    chunk_size: Option<u32>,
    tx_progress: Sender<SyncProgress>,
    mut rx_cancel: broadcast::Receiver<()>,
) -> Result<()> {
    let activation_height: u32 = network
        .activation_height(NetworkUpgrade::Sapling)
        .unwrap()
        .into();
    let start = start.max(activation_height);
    let end = end.max(activation_height);

    let chunk_size = chunk_size.unwrap_or(u32::MAX).max(1);
    let mut chunk_start = start;
    loop {
        let chunk_end = chunk_start.saturating_add(chunk_size - 1).min(end);
        shielded_sync_range(
            network,
            pool,
            client,
            accounts,
            chunk_start,
            chunk_end,
            actions_per_sync,
            tx_progress.clone(),
            rx_cancel.resubscribe(),
        )
        .await?;
        if chunk_end >= end {
            break;
        }
        // A cancel delivered during the finished chunk was consumed by that
        // chunk's resubscribed receiver; check ours before starting the next.
        match rx_cancel.try_recv() {
            Err(broadcast::error::TryRecvError::Empty)
            | Err(broadcast::error::TryRecvError::Closed) => {}
            _ => {
                debug!("Sync cancelled between chunks");
                break;
            }
        }
        chunk_start = chunk_end + 1;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn shielded_sync_range(
    network: &Network,
    pool: &SqlitePool,
    client: &mut Client,
    accounts: &[(u32, bool)],
    start: u32,
    end: u32,
    actions_per_sync: u32,
    tx_progress: Sender<SyncProgress>,
    rx_cancel: broadcast::Receiver<()>,
) -> Result<()> {
    let accounts = accounts.to_vec();
    let account_ids: Vec<u32> = accounts.iter().map(|(a, _)| *a).collect();
    let db_writer_task = {
        let (s, o, i) = get_tree_state(network, client, start - 1).await?;

        debug!("get compact block range");
        let blocks = get_compact_block_range(network, client, start, end).await?;
        debug!("got streaming blocks");
        let (tx_messages, mut rx_messages) = channel::<WarpSyncMessage>(100);

        let mut connection = pool.acquire().await?;
        // get the list of transaction heights for which the time is 0
        // because raw transactions do not have timestamp (it comes from the block header)
        let heights_without_time =
            get_heights_without_time(&mut connection, &account_ids, start, end).await?;

        let mut writer_connection = pool.acquire().await?;

        // Preload all account keys ONCE before sync starts.
        // Key derivations happen once upfront — no per-note DB queries in the hot path.
        let key_cache = Arc::new(preload_account_key_cache(&mut writer_connection).await?);

        let network = *network;
        let mut messages = vec![];
        let db_writer_task = tokio::spawn(async move {
            debug!("[db handler] starting");
            while let Some(msg) = rx_messages.recv().await {
                //debug!("Received message: {:?}", msg);
                if let WarpSyncMessage::Commit = msg {
                    let mut db_tx = writer_connection.begin().await?;
                    let mut new_messages = vec![];
                    mem::swap(&mut new_messages, &mut messages);
                    for msg in new_messages {
                        match handle_message(
                            &network,
                            &mut db_tx,
                            msg,
                            &tx_progress,
                            &key_cache,
                            &account_ids,
                        )
                        .await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                info!("ERROR HANDLING MESSAGE: {:?}", e);
                                return Err(e);
                            }
                        }
                    }
                    db_tx.commit().await?;
                    debug!("Committing transaction");
                } else {
                    messages.push(msg);
                }
            }

            let mut db_tx = writer_connection.begin().await?;
            for msg in messages {
                match handle_message(
                    &network,
                    &mut db_tx,
                    msg,
                    &tx_progress,
                    &key_cache,
                    &account_ids,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        info!("ERROR HANDLING MESSAGE: {:?}", e);
                        return Err(e);
                    }
                }
            }
            db_tx.commit().await?;

            debug!("[db handler] stopped");
            check_witness_consistency(&mut writer_connection, &account_ids).await?;

            Ok::<_, anyhow::Error>(())
        });

        tokio::spawn(async move {
            debug!("Start sync");
            if let Err(e) = warp_sync(
                &network,
                &mut connection,
                start,
                &accounts,
                blocks,
                heights_without_time,
                actions_per_sync,
                &s,
                &o,
                &i,
                tx_messages.clone(),
                rx_cancel,
            )
            .await
            {
                tracing::error!("Error during warp sync: {:?}", e);
                let _ = tx_messages.send(WarpSyncMessage::Error(e)).await;
            }

            debug!("Sync finished");
        });

        db_writer_task
    };

    db_writer_task.await??;
    Ok(())
}

async fn handle_message(
    network: &Network,
    db_tx: &mut sqlx::Transaction<'_, Sqlite>,
    msg: WarpSyncMessage,
    tx_progress: &Sender<SyncProgress>,
    key_cache: &AccountKeyCache,
    accounts: &[u32],
) -> Result<()> {
    tracing::debug!(target: "warp", "Warp Message: {msg:?}");
    match msg {
        WarpSyncMessage::Issuance(iss) => {
            chain_log::record_issuance(db_tx, &iss).await?;
            tracing::debug!("asset base {}", hex::encode(&iss.asset_base));
            debug!(
                "Processing Issuance: height={}, finalized={}",
                iss.height, iss.finalized
            );
        }
        WarpSyncMessage::Transaction(tx) => {
            // duplicates are ignored: the transparent scan may have created
            // the transaction already
            chain_log::record_transaction(
                db_tx,
                tx.account,
                &TransactionRow::chain(&tx.txid, tx.height, tx.time),
            )
            .await?;
            debug!("Processing Transaction: id={}, height={}", tx.id, tx.height);
        }
        WarpSyncMessage::Note(note) => {
            tracing::debug!("note asset base {}", hex::encode(&note.asset_base));
            // A ZSA note found through decryption registers its asset by
            // base when no issuance has been seen; a ZEC note has no asset.
            let id_asset =
                chain_log::asset_id_for_base(db_tx, &note.asset_base, note.height).await?;

            // Resolve diversifier_index from the preloaded key cache
            let diversifier_index = resolve_diversifier_index(
                key_cache,
                note.account,
                note.pool,
                note.scope,
                &note.diversifier,
            );

            let (id_tx, _) = chain_log::transaction_ref(db_tx, note.account, &note.txid).await?;
            let stored = chain_log::record_note(
                db_tx,
                note.account,
                &NoteRow {
                    id_tx,
                    height: note.height,
                    pool: note.pool,
                    scope: Some(note.scope),
                    nullifier: &note.nf,
                    value: note.value as i64,
                    cmx: Some(&note.cmx),
                    position: Some(note.position),
                    diversifier: Some(&note.diversifier),
                    rcm: Some(&note.rcm),
                    rho: Some(&note.rho),
                    id_asset,
                    diversifier_index,
                    ..Default::default()
                },
            )
            .await?;
            debug!(
                "Processing Note: id={}, account={}, height={}",
                note.id, note.account, note.height
            );
            debug!("{:?}", note);
            anyhow::ensure!(
                stored.is_some(),
                "note {} of account {} at {} was already recorded",
                note.id,
                note.account,
                note.height
            );
        }
        WarpSyncMessage::Witness(account, height, cmx, witness) => {
            let w = bincode::encode_to_vec(&witness, config::legacy())?;
            chain_log::record_witness(db_tx, account, NoteKey::Cmx(&cmx), height, &w).await?;
        }
        WarpSyncMessage::Spend(utxo) => {
            // the spending transaction, not the one that created the note
            let (id_tx, height) =
                chain_log::transaction_ref(db_tx, utxo.account, &utxo.txid).await?;
            let recorded = chain_log::record_spend(
                db_tx,
                utxo.account,
                &SpendRow {
                    note: NoteKey::Cmx(&utxo.cmx),
                    id_tx,
                    height,
                    pool: utxo.pool,
                    value: -(utxo.value as i64),
                },
            )
            .await?;
            debug!("Processing Spend: {:?}", &utxo);
            anyhow::ensure!(
                recorded,
                "spend of note {} by account {} was already recorded",
                hex::encode(&utxo.cmx),
                utxo.account
            );
        }
        WarpSyncMessage::Checkpoint(accounts, pool, height) => {
            for a in accounts {
                chain_log::checkpoint(db_tx, a, pool, height).await?;
                let _ = tx_progress.send(SyncProgress { height, time: 0 }).await;
            }
        }
        WarpSyncMessage::BlockHeader(block_header) => {
            debug!("Processing BlockHeader: {:?}", block_header);
            for account in accounts {
                chain_log::record_relevant_header(db_tx, *account, &block_header).await?;
                chain_log::set_block_time(
                    db_tx,
                    *account,
                    block_header.height,
                    block_header.time,
                    false,
                )
                .await?;
            }
        }
        WarpSyncMessage::Commit => {
            // handled in the caller
        }
        WarpSyncMessage::Rewind(accounts, height) => {
            debug!("Discard height: {}", height);
            for account in accounts {
                rewind_sync(network, db_tx, account, height).await?;
            }
        }
        WarpSyncMessage::Error(e) => {
            return Err(e.into());
        }
    }

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

#[allow(clippy::too_many_arguments)]
pub async fn transparent_sweep(
    network: &Network,
    mut connection: PoolConnection<Sqlite>,
    mut client: Client,
    account: u32,
    end_height: u32,
    gap_limit: u32,
    progress_fn: impl Fn(String) + 'static + Send,
    cancellation_token: CancellationToken,
) -> Result<()> {
    let network = *network;
    let hw = get_account_hw(&mut connection, account).await?;
    let aindex = get_account_aindex(&mut connection, account).await?;
    let dindex = get_account_dindex(&mut connection, account).await?;
    let (use_internal,): (bool,) =
        sqlx::query_as("SELECT use_internal FROM accounts WHERE id_account = ?")
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;
    tokio::spawn(async move {
        let ledger = get_ledger(&mut connection, account).await?;
        let mut n_added = 0;
        let tk = select_account_transparent(&mut connection, account, dindex).await?;
        let xvk = tk.xvk;
        let start_height = get_birth_height(&mut connection, account).await?;
        for scope in 0..2 {
            let mut dindex = 0;
            let mut gap = 0;
            loop {
                let (pk, taddr) = match xvk.as_ref() {
                    Some(xvk) => derive_transparent_address(xvk, scope, dindex, false)?,
                    None if hw != 0 => {
                        ledger
                            .get_transparent_pubkey(&network, aindex, scope, dindex)
                            .await?
                    }
                    _ => anyhow::bail!("Sweep needs an xpub key"),
                };
                let taddr = taddr.encode(&network);
                progress_fn(taddr.clone());

                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        return Ok::<_, anyhow::Error>(n_added)
                    }

                    txids = client
                        .taddress_txs(&network, &taddr, start_height, end_height)
                        => {
                        let mut txids = txids?;
                        if txids.next().await.is_some() {
                            // A used address resets the gap count: only
                            // consecutive unused addresses end the scan.
                            gap = 0;
                            let sk = if let Some(tsk) = tk.xsk.as_ref() {
                                Some(derive_transparent_sk(tsk, scope, dindex)?)
                            } else {
                                None
                            };
                            let added = store_account_transparent_addr(
                                &mut connection, account, scope, dindex, sk, &pk, &taddr, false,
                            )
                            .await?;
                            if added {
                                n_added += 1;
                            }

                            let want_counterpart = if scope == 0 {
                                use_internal
                            } else {
                                true
                            };
                            if want_counterpart {
                                let counterpart_scope = 1 - scope;
                                let existing: (i64,) = sqlx::query_as(
                                    "SELECT COUNT(*) FROM transparent_address_accounts
                                    WHERE account = ?1 AND scope = ?2 AND dindex = ?3",
                                )
                                .bind(account)
                                .bind(counterpart_scope)
                                .bind(dindex)
                                .fetch_one(&mut *connection)
                                .await?;
                                if existing.0 == 0 {
                                    let (csk, cpk, ctaddr) = match xvk.as_ref() {
                                        Some(xvk) => {
                                            let sk = if let Some(tsk) = tk.xsk.as_ref() {
                                                Some(derive_transparent_sk(
                                                    tsk,
                                                    counterpart_scope,
                                                    dindex,
                                                )?)
                                            } else {
                                                None
                                            };
                                            let (pk, address) = derive_transparent_address(
                                                xvk,
                                                counterpart_scope,
                                                dindex,
                                                false,
                                            )?;
                                            (sk, pk, address)
                                        }
                                        None if hw != 0 => {
                                            let (pk, address) = ledger
                                                .get_transparent_pubkey(
                                                    &network,
                                                    aindex,
                                                    counterpart_scope,
                                                    dindex,
                                                )
                                                .await?;
                                            (None, pk, address)
                                        }
                                        _ => anyhow::bail!("Sweep needs an xpub key"),
                                    };
                                    store_account_transparent_addr(
                                        &mut connection,
                                        account,
                                        counterpart_scope,
                                        dindex,
                                        csk,
                                        &cpk,
                                        &ctaddr.encode(&network),
                                        false,
                                    )
                                    .await?;
                                }
                            }
                        } else {
                            gap += 1;
                        }
                        dindex += 1;
                        if gap > gap_limit {
                            break;
                        }
                    }
                }
            }
        }
        // Advance the account dindex to the greatest stored external (scope 0)
        // used index, never backwards. The scan only stores used addresses and
        // address generation keeps accounts.dindex at the newest row, so the
        // table's max external dindex is the receiving frontier.
        sqlx::query(
            "UPDATE accounts
            SET dindex = max(
                dindex,
                COALESCE(
                    (SELECT max(dindex) FROM transparent_address_accounts
                     WHERE account = accounts.id_account AND scope = 0),
                    0))
            WHERE id_account = ?",
        )
        .bind(account)
        .execute(&mut *connection)
        .await?;
        Ok(n_added)
    });
    Ok(())
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
