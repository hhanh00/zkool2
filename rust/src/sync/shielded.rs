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

use super::chain_log::{self, NoteKey, NoteRow, SpendRow, TransactionRow};
use super::{check_witness_consistency, get_heights_without_time, rewind_within, WarpSyncMessage};
use crate::{
    api::sync::SyncProgress,
    lwd::CompactBlock,
    warp::{
        legacy::CommitmentTreeFrontier,
        sync::{warp_sync, ScanEnd},
    },
};
use bincode::config;
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
) -> Result<ReceiverStream<Result<CompactBlock>>> {
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
    // Where the last reorg was detected. A reorg is absorbed here: the
    // writer rewinds, the scan restarts from where the database stands, and
    // the caller never hears of it. Each one must be detected strictly
    // below the previous one, or the server is serving the same blocks that
    // do not connect, and the run fails instead of looping (#1474).
    let mut last_reorg: Option<u32> = None;
    loop {
        let chunk_end = chunk_start.saturating_add(chunk_size - 1).min(end);
        match shielded_sync_range(
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
        .await?
        {
            RangeEnd::Cancelled => break,
            RangeEnd::Reorg {
                detected_at,
                restart_from,
            } => {
                if let Some(previous) = last_reorg {
                    if detected_at >= previous {
                        anyhow::bail!(
                            "the server's blocks do not connect at height {detected_at} \
                             after a rewind for the same height: giving up"
                        );
                    }
                }
                info!("reorg detected at {detected_at}; rescanning from {restart_from}");
                last_reorg = Some(detected_at);
                chunk_start = restart_from.max(start);
                continue;
            }
            RangeEnd::Completed => {}
        }
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

/// How one range scan ended. A range that stopped short is an error from
/// `shielded_sync_range`, never a variant.
#[derive(Debug, PartialEq, Eq)]
pub enum RangeEnd {
    Completed,
    Cancelled,
    /// The writer rewound the accounts; `restart_from` is the first height
    /// they have not synced now.
    Reorg { detected_at: u32, restart_from: u32 },
}

/// The first height none of `accounts` has synced: one past the lowest
/// checkpoint among them.
async fn restart_height(connection: &mut SqliteConnection, accounts: &[u32]) -> Result<u32> {
    let mut lowest: Option<u32> = None;
    for account in accounts {
        let (height,): (Option<u32>,) =
            sqlx::query_as("SELECT MIN(height) FROM sync_heights WHERE account = ?")
                .bind(account)
                .fetch_one(&mut *connection)
                .await?;
        if let Some(height) = height {
            lowest = Some(lowest.map_or(height, |l| l.min(height)));
        }
    }
    lowest
        .map(|h| h + 1)
        .ok_or_else(|| anyhow::anyhow!("no checkpoint left to restart the scan from"))
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
) -> Result<RangeEnd> {
    let accounts = accounts.to_vec();
    let account_ids: Vec<u32> = accounts.iter().map(|(a, _)| *a).collect();
    let (db_writer_task, scan_task) = {
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
        let writer_account_ids = account_ids.clone();
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
                            &writer_account_ids,
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
                    &writer_account_ids,
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
            check_witness_consistency(&mut writer_connection, &writer_account_ids).await?;

            Ok::<_, anyhow::Error>(())
        });

        let scan_task = tokio::spawn(async move {
            debug!("Start sync");
            let scan = warp_sync(
                &network,
                &mut connection,
                start,
                end,
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
            .await;
            if let Err(e) = &scan {
                tracing::error!("Error during warp sync: {:?}", e);
            }
            // The writer learns of a failure through the channel too, so it
            // stops before its final flush commits anything more.
            if let Err(e) = scan {
                let _ = tx_messages.send(WarpSyncMessage::Error(e)).await;
                return Err(anyhow::anyhow!("scan failed"));
            }
            debug!("Sync finished");
            scan.map_err(|e| anyhow::anyhow!(e))
        });

        (db_writer_task, scan_task)
    };

    db_writer_task.await??;
    // A scanner that panicked dropped its sender: the writer saw a normal
    // end of stream and flushed. Joining the task turns that into an error
    // instead of a run reported as synced (V12 F-72).
    let scan = match scan_task.await {
        Ok(scan) => scan?,
        Err(e) if e.is_panic() => anyhow::bail!("the shielded scanner panicked: {e}"),
        Err(e) => anyhow::bail!("the shielded scanner was cancelled: {e}"),
    };
    match scan {
        ScanEnd::Completed => Ok(RangeEnd::Completed),
        ScanEnd::Cancelled => Ok(RangeEnd::Cancelled),
        ScanEnd::Reorg { detected_at } => {
            let mut connection = pool.acquire().await?;
            let restart_from = restart_height(&mut connection, &account_ids).await?;
            Ok(RangeEnd::Reorg {
                detected_at,
                restart_from,
            })
        }
    }
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
                rewind_within(network, db_tx, account, height).await?;
            }
        }
        WarpSyncMessage::Error(e) => {
            return Err(e.into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod stream_tests {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use tonic::async_trait;
    use zcash_primitives::transaction::Transaction;

    use super::*;
    use crate::account::new_account;
    use crate::api::account::NewAccount;
    use crate::db::create_schema;
    use crate::net::LwdServer;
    use crate::pay::pool::{POOL_ORCHARD, POOL_SAPLING};

    // A run must not report success when the block stream did not deliver
    // the range (#1474): a stream that ends or fails short is an error, a
    // reorg is rescanned from where the database stands, and a reorg the
    // server repeats at the same height fails the run.

    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn regtest() -> Network {
        use zcash_protocol::consensus::{BlockHeight, OrchardMode};
        use zcash_protocol::local_consensus::LocalNetwork;
        Network::Regtest(LocalNetwork {
            overwinter: Some(BlockHeight::from_u32(1)),
            sapling: Some(BlockHeight::from_u32(1)),
            blossom: Some(BlockHeight::from_u32(1)),
            heartwood: Some(BlockHeight::from_u32(1)),
            canopy: Some(BlockHeight::from_u32(1)),
            nu5: Some(BlockHeight::from_u32(1)),
            nu6: Some(BlockHeight::from_u32(1)),
            nu6_1: Some(BlockHeight::from_u32(1)),
            nu6_2: Some(BlockHeight::from_u32(1)),
            nu6_3: Some(BlockHeight::from_u32(250)),
            nu7: None,
            zsa: None,
            orchard_mode: OrchardMode::Normal,
        })
    }

    /// What one `block_range` call serves.
    #[derive(Clone, Copy)]
    enum Serve {
        /// Every block of the range, each connected to the previous one.
        Whole,
        /// The blocks up to this height, then the stream ends.
        EndsAfter(u32),
        /// The blocks up to this height, then an error.
        FailsAfter(u32),
        /// Every block, but this one does not connect to its predecessor.
        Broken(u32),
    }

    /// A block source following a script, one entry per `block_range` call,
    /// the last entry repeating. Records the ranges it was asked for.
    struct Blocks {
        script: VecDeque<Serve>,
        asked: Arc<Mutex<Vec<(u32, u32)>>>,
    }

    fn hash(h: u32) -> Vec<u8> {
        let mut v = vec![0xABu8; 32];
        v[..4].copy_from_slice(&h.to_le_bytes());
        v
    }

    fn block(h: u32, prev: u32) -> CompactBlock {
        CompactBlock {
            height: h as u64,
            hash: hash(h),
            prev_hash: hash(prev),
            time: 1_700_000_000 + h,
            ..Default::default()
        }
    }

    #[async_trait]
    impl LwdServer for Blocks {
        type CompactBlockStream = ReceiverStream<Result<CompactBlock>>;
        type TransactionStream = ReceiverStream<Result<(u32, Transaction, usize)>>;

        async fn latest_height(&mut self) -> Result<u32> {
            unimplemented!()
        }
        async fn block(&mut self, _: &Network, _: u32) -> Result<CompactBlock> {
            unimplemented!()
        }
        async fn block_range(
            &mut self,
            _: &Network,
            start: u32,
            end: u32,
        ) -> Result<Self::CompactBlockStream> {
            self.asked.lock().unwrap().push((start, end));
            let serve = if self.script.len() > 1 {
                self.script.pop_front().unwrap()
            } else {
                *self.script.front().unwrap()
            };
            let (tx, rx) = tokio::sync::mpsc::channel(1024);
            for h in start..=end {
                match serve {
                    Serve::EndsAfter(last) if h > last => break,
                    Serve::FailsAfter(last) if h > last => {
                        tx.send(Err(anyhow::anyhow!("connection reset"))).await.unwrap();
                        break;
                    }
                    Serve::Broken(b) if h == b => tx.send(Ok(block(h, h + 1000))).await.unwrap(),
                    _ => tx.send(Ok(block(h, h - 1))).await.unwrap(),
                }
            }
            Ok(ReceiverStream::new(rx))
        }
        async fn transaction(&mut self, _: &Network, _: &[u8]) -> Result<(u32, Transaction)> {
            unimplemented!()
        }
        async fn post_transaction(&mut self, _: u32, _: &[u8]) -> Result<String> {
            unimplemented!()
        }
        async fn taddress_txs(
            &mut self,
            _: &Network,
            _: &str,
            _: u32,
            _: u32,
        ) -> Result<Self::TransactionStream> {
            unimplemented!()
        }
        async fn mempool_stream(&mut self, _: &Network) -> Result<Self::TransactionStream> {
            unimplemented!()
        }
        /// Empty trees: the scan starts from a fresh frontier.
        async fn tree_state(&mut self, _: u32) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
            Ok((vec![], vec![], vec![]))
        }
    }

    /// A file-backed wallet, since the scan's two connections must see one
    /// database, with one account holding Sapling and Orchard keys, born at
    /// height 1.
    async fn wallet(name: &str) -> Result<(SqlitePool, String)> {
        let path = std::env::temp_dir().join(format!("{name}_{}.db", std::process::id()));
        let path = path.to_str().unwrap().to_string();
        let _ = std::fs::remove_file(&path);
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(&path)
                    .create_if_missing(true),
            )
            .await?;
        let mut db = pool.acquire().await?;
        create_schema(&mut db).await?;
        new_account(
            &regtest(),
            &mut db,
            &NewAccount {
                icon: None,
                name: "a".into(),
                restore: true,
                key: MNEMONIC.into(),
                passphrase: None,
                fingerprint: None,
                aindex: 0,
                birth: Some(1),
                folder: String::new(),
                pools: Some(POOL_SAPLING | POOL_ORCHARD),
                use_internal: false,
                internal: false,
                hw: 0,
            },
        )
        .await?;
        Ok((pool, path))
    }

    async fn synced_height(pool: &SqlitePool) -> Result<u32> {
        let (h,): (u32,) = sqlx::query_as("SELECT MIN(height) FROM sync_heights WHERE account = 1")
            .fetch_one(pool)
            .await?;
        Ok(h)
    }

    /// Scans 1..=end with the scripted source; returns the outcome and the
    /// ranges the source was asked for.
    async fn scan(pool: &SqlitePool, script: &[Serve], end: u32) -> (Result<()>, Vec<(u32, u32)>) {
        let asked = Arc::new(Mutex::new(vec![]));
        let mut client: Client = Box::new(Blocks {
            script: script.iter().copied().collect(),
            asked: asked.clone(),
        });
        let (progress, _rx_progress) = channel::<SyncProgress>(1024);
        let (_cancel, rx_cancel) = broadcast::channel(1);
        let r = shielded_sync(
            &regtest(),
            pool,
            &mut client,
            &[(1, false)],
            1,
            end,
            1000,
            None,
            progress,
            rx_cancel,
        )
        .await;
        let asked = asked.lock().unwrap().clone();
        (r, asked)
    }

    #[tokio::test]
    async fn a_stream_that_ends_early_fails_the_run_and_keeps_its_progress() -> Result<()> {
        let (pool, path) = wallet("ends_early").await?;
        let (r, _) = scan(&pool, &[Serve::EndsAfter(6)], 10).await;
        let err = format!("{:#}", r.unwrap_err());
        assert!(err.contains("ended at height 6, 10 asked"), "{err}");
        assert_eq!(synced_height(&pool).await?, 6, "the blocks that did arrive are committed");
        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[tokio::test]
    async fn a_stream_error_fails_the_run_and_keeps_its_progress() -> Result<()> {
        let (pool, path) = wallet("fails").await?;
        let (r, _) = scan(&pool, &[Serve::FailsAfter(4)], 10).await;
        let err = format!("{:#}", r.unwrap_err());
        assert!(err.contains("failed after height 4, 10 asked"), "{err}");
        assert!(err.contains("connection reset"), "the cause is kept: {err}");
        assert_eq!(synced_height(&pool).await?, 4);
        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[tokio::test]
    async fn a_reorg_is_rescanned_from_where_the_database_stands() -> Result<()> {
        let (pool, path) = wallet("reorg").await?;
        let (r, asked) = scan(&pool, &[Serve::Broken(5), Serve::Whole], 8).await;
        r?;
        // Nothing of the first pass was committed, so the rewind went back to
        // the birth checkpoint and the rescan starts just above it.
        assert_eq!(asked, vec![(1, 8), (2, 8)]);
        assert_eq!(synced_height(&pool).await?, 8);
        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[tokio::test]
    async fn a_reorg_repeated_at_the_same_height_fails_the_run() -> Result<()> {
        let (pool, path) = wallet("reorg_twice").await?;
        let (r, asked) = scan(&pool, &[Serve::Broken(5)], 8).await;
        let err = format!("{:#}", r.unwrap_err());
        assert!(err.contains("do not connect at height 5"), "{err}");
        assert_eq!(asked.len(), 2, "one rewind, then no third attempt");
        let _ = std::fs::remove_file(path);
        Ok(())
    }
}
