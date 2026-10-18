//! Shielded scan: compact blocks streamed through the warp synchronizer,
//! decrypted with preloaded account keys, and recorded through the chain log
//! one block batch at a time.

use anyhow::Result;
use sqlx::SqliteConnection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::mpsc::channel;
use tracing::{debug, info};

use crate::api::coin::Network;
use crate::keys::{OrchardViewingKeys, SaplingViewingKeys};

use crate::Client;
use std::mem;

use crate::{
    api::sync::SyncProgress,
    lwd::CompactBlock,
    warp::{legacy::CommitmentTreeFrontier, sync::warp_sync},
};
use bincode::config;
use super::chain_log::{self, NoteKey, NoteRow, SpendRow, TransactionRow};
use super::{check_witness_consistency, get_heights_without_time, rewind_sync, WarpSyncMessage};
use sqlx::{Connection, Sqlite, SqlitePool};

use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use zcash_protocol::consensus::{NetworkUpgrade, Parameters};

/// Cache of all per-account key material needed during shielded sync.
/// Preloaded once and shared via Arc between the db_writer and warp_sync tasks.
pub struct AccountKeyCache {
    pub sapling: HashMap<u32, SaplingViewingKeys>,
    pub orchard: HashMap<u32, OrchardViewingKeys>,
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
    let sapling_rows: Vec<(u32, Option<Vec<u8>>, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT account, xvk, ivk FROM sapling_accounts")
            .fetch_all(&mut *connection)
            .await?;
    for (account, xvk, ivk) in sapling_rows {
        if let Some(keys) = SaplingViewingKeys::from_stored(xvk, ivk) {
            sapling.insert(account, keys);
        }
    }

    let mut orchard = HashMap::new();
    let orchard_rows: Vec<(u32, Option<Vec<u8>>, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT account, xvk, ivk FROM orchard_accounts")
            .fetch_all(&mut *connection)
            .await?;
    for (account, xvk, ivk) in orchard_rows {
        if let Some(keys) = OrchardViewingKeys::from_stored(xvk, ivk) {
            orchard.insert(account, keys);
        }
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
        1 => cache
            .sapling
            .get(&account)
            .and_then(|keys| keys.diversifier_index(scope, diversifier)),
        2 | 3 => cache
            .orchard
            .get(&account)
            .and_then(|keys| keys.diversifier_index(scope, diversifier)),
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
