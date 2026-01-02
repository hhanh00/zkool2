use anyhow::{Context as _, Error, Result};
use flutter_rust_bridge::frb;
use sqlx::SqliteConnection;
use sqlx::{Row, sqlite::SqliteRow};
use std::collections::HashMap;
use tokio::sync::mpsc::channel;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tracing::info;
use zcash_transparent::address::TransparentAddress;

use crate::api::coin::{Coin, Network};
use crate::api::sync::{CANCEL_SYNC, SYNCING};
use crate::budget::merge_pending_txs;

use crate::db::store_block_header;
use crate::io::SyncHeight;
use crate::Client;
use crate::frb_generated::StreamSink;
use std::{collections::HashSet, mem};

use crate::{
    account::{derive_transparent_address, derive_transparent_sk, get_birth_height, has_pool}, api::{sync::SyncProgress}, db::{
        get_account_aindex, get_account_dindex, get_account_hw, select_account_transparent,
        store_account_transparent_addr,
    }, lwd::CompactBlock, warp::{
        Witness, legacy::CommitmentTreeFrontier, sync::{SyncError, warp_sync}
    }
};
use bincode::config;
use sqlx::{pool::PoolConnection};
use sqlx::{Connection, Sqlite, SqlitePool};
use tokio::sync::{
    mpsc::{Sender},
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;
use zcash_keys::encoding::AddressCodec;
use zcash_protocol::consensus::{NetworkUpgrade, Parameters};

use crate::api::ledger::get_hw_transparent_address;

#[frb(dart_metadata = ("freezed"))]
#[derive(Default, Debug)]
pub struct Transaction {
    pub id: u32,
    pub txid: Vec<u8>,
    pub height: u32,
    pub account: u32,
    pub time: u32,
    pub value: i64,
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "txid {} {} @{}",
            hex::encode(&self.txid),
            self.account,
            self.height
        )
    }
}

#[frb(dart_metadata = ("freezed"))]
pub struct NoteExtended {
    pub id: u32,
    pub address: Vec<u8>,
    pub memo: Vec<u8>,
}

#[derive(Default)]
pub struct UTXO {
    pub id: u32,
    pub pool: u8,
    pub account: u32,
    pub nullifier: Vec<u8>,
    pub value: u64,
    pub position: u32,
    pub cmx: Vec<u8>,
    pub witness: Witness,

    pub txid: Vec<u8>,
}

impl std::fmt::Display for UTXO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.account,
            self.pool,
            hex::encode(&self.nullifier)
        )
    }
}

impl std::fmt::Debug for UTXO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UTXO")
            .field("id", &self.id)
            .field("account", &self.account)
            .field("pool", &self.pool)
            .field("txid", &hex::encode(&self.txid))
            .field("cmx", &hex::encode(&self.cmx))
            .finish()
    }
}

#[derive(Debug)]
pub enum WarpSyncMessage {
    BlockHeader(BlockHeader),
    Transaction(Transaction),
    Note(Note),
    Witness(u32, u32, Vec<u8>, Witness),
    Checkpoint(Vec<u32>, u8, u32),
    Commit,
    Spend(UTXO),
    Rewind(Vec<u32>, u32),
    Error(SyncError),
}

impl std::fmt::Display for WarpSyncMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WarpSyncMessage::BlockHeader(block_header) => write!(
                f,
                "Header: {} {}",
                block_header.height,
                hex::encode(&block_header.hash)
            ),
            WarpSyncMessage::Transaction(transaction) => write!(f, "Tx: {transaction}"),
            WarpSyncMessage::Note(note) => write!(f, "Note: {note}"),
            WarpSyncMessage::Witness(account, height, cmx, witness) => write!(
                f,
                "Witness for {account} @{height}: {} {witness}",
                hex::encode(cmx)
            ),
            WarpSyncMessage::Checkpoint(_, pool, height) => {
                write!(f, "Checkpoint for {pool} @{height}")
            }
            WarpSyncMessage::Commit => write!(f, "Commit"),
            WarpSyncMessage::Spend(utxo) => write!(f, "Spend {utxo}"),
            WarpSyncMessage::Rewind(_, height) => write!(f, "Rewind to @{height}"),
            WarpSyncMessage::Error(sync_error) => write!(f, "SyncError: {sync_error:?}"),
        }
    }
}

#[frb(dart_metadata = ("freezed"))]
pub struct BlockHeader {
    pub height: u32,
    pub hash: Vec<u8>,
    pub time: u32,
}

impl std::fmt::Debug for BlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockHeader")
            .field("height", &self.height)
            .field("hash", &hex::encode(&self.hash))
            .finish()
    }
}

#[frb(dart_metadata = ("freezed"))]
#[derive(Clone, Default)]
pub struct Note {
    pub id: u32,
    pub account: u32,
    pub scope: u8,
    pub height: u32,
    pub position: u32,
    pub pool: u8,
    pub id_tx: u32,
    pub vout: u32,
    pub diversifier: Vec<u8>,
    pub value: u64,
    pub rcm: Vec<u8>,
    pub rho: Vec<u8>,
    pub nf: Vec<u8>,

    pub ivtx: u32,     // not stored in the database
    pub cmx: Vec<u8>,  // unique identifier for the note
    pub txid: Vec<u8>, // transaction id
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.account,
            self.height,
            self.position,
            self.pool,
            hex::encode(&self.nf)
        )
    }
}

impl std::fmt::Debug for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Note")
            .field("id", &self.id)
            .field("account", &self.account)
            .field("height", &self.height)
            .field("position", &self.position)
            .field("pool", &self.pool)
            .field("id_tx", &self.id_tx)
            .field("vout", &self.vout)
            .field("diversifier", &hex::encode(&self.diversifier))
            .field("value", &self.value)
            .field("rcm", &hex::encode(&self.rcm))
            .field("rho", &hex::encode(&self.rho))
            .field("nf", &hex::encode(&self.nf))
            .finish()
    }
}

pub trait Sink<V>: Clone {
    fn add(&self, value: V);
    fn add_error(&self, e: Error);
}

impl Sink<SyncProgress> for StreamSink<SyncProgress> {
    fn add(&self, value: SyncProgress) {
        let _ = self.add(value);
    }

    fn add_error(&self, error: Error) {
        let _ = self.add_error(error);
    }
}

impl<T> Sink<T> for () {
    fn add(&self, _value: T) {}
    fn add_error(&self, _error: Error) {}
}

pub async fn synchronize_impl<S: Sink<SyncProgress> + Send + 'static>(
    progress: S,
    accounts: Vec<u32>,
    current_height: u32,
    actions_per_sync: u32,
    transparent_limit: u32,
    checkpoint_age: u32,
    c: &Coin,
) -> Result<()> {
    if accounts.is_empty() {
        return Ok(());
    }

    let Ok(_guard) = SYNCING.try_lock() else {
        return Ok(());
    };

    let (tx_cancel, _rx_cancel) = broadcast::channel::<()>(1);
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
                account_heights.insert(account, height + 1);

                let (use_internal,): (bool,) =
                    sqlx::query_as("SELECT use_internal FROM accounts WHERE id_account = ?")
                        .bind(account)
                        .fetch_one(&mut *connection)
                        .await
                        .context("Fetch use_internal")?;
                account_use_internal.insert(account, use_internal);
            }
        }

        // Create a sorted list of unique heights
        let mut unique_heights: Vec<u32> = account_heights.values().cloned().collect();
        unique_heights.sort_unstable();
        unique_heights.dedup();

        let (tx_progress, mut rx_progress) = channel::<SyncProgress>(1);

        tokio::spawn(async move {
            while let Some(p) = rx_progress.recv().await {
                progress.add(p);
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
                continue;
            }

            let pool = c.get_pool()?;
            // Update the sync heights for these accounts
            let mut client = c.client().await?;

            info!("Start height: {}", start_height);
            info!("End height: {}", end_height);

            if start_height > end_height {
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

            shielded_sync(
                &network,
                &pool,
                &mut client,
                &accounts_to_sync,
                start_height,
                end_height,
                actions_per_sync,
                tx_progress.clone(),
                tx_cancel.subscribe(),
            )
            .await?;

            let heights_without_time =
                get_heights_without_time(&mut connection, start_height, end_height).await?;
            for h in heights_without_time {
                let block = client.block(&network, h).await?;
                let time = block.time;
                sqlx::query("UPDATE transactions SET time = ? WHERE height = ? AND time = 0")
                    .bind(time)
                    .bind(h)
                    .execute(&mut *connection)
                    .await?;
                let block_header = BlockHeader {
                    height: h,
                    hash: block.hash,
                    time: block.time,
                };
                store_block_header(&mut connection, &block_header).await?;
            }

            // Update our local map as well for the next iteration
            for (account, _) in &accounts_to_sync {
                account_heights.insert(*account, end_height);
                crate::memo::fetch_tx_details(&network, &mut connection, &mut client, *account)
                    .await?;
            }

            info!(
                "Sync completed for height range {}-{}",
                start_height, end_height
            );
        }

        for account in accounts.iter() {
            merge_pending_txs(&mut connection, *account, current_height).await?;
        }

        Ok::<_, anyhow::Error>(())
    };

    match res.await {
        Ok(_) => {}
        Err(e) => {
            info!("Error during sync: {:?}", e);
            progress2.add_error(e);
        }
    }

    {
        let mut cancel = CANCEL_SYNC.lock().await;
        *cancel = None;
    }

    Ok(())
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
    for account in accounts {
        // scan latest 5 receive and change addresses
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

        while let Some((id_taddress, address)) = rows.try_next().await? {
            // Add the address to the client
            addresses.push((*account, (id_taddress, address)));
        }
    }
    for (account, address_row) in addresses.iter() {
        let my_address = TransparentAddress::decode(&network, &address_row.1)?;
        let mut txs = client
            .taddress_txs(network, &address_row.1, start_height, end_height)
            .await?
            .into_inner();

        let mut db_tx = connection.begin().await?;
        loop {
            tokio::select! {
                _ = rx_cancel.recv() => {
                    info!("Canceling sync");
                    anyhow::bail!("Sync canceled");
                }
                m = txs.recv() => {
                    if let Some((height, transaction, _)) = m {
                        let txid = transaction.txid().as_ref().to_vec();
                        // tx time is available in the block (not here)
                        sqlx::query("INSERT INTO transactions (account, txid, height, time) VALUES (?, ?, ?, 0) ON CONFLICT DO NOTHING")
                        .bind(account)
                        .bind(&txid)
                        .bind(height)
                        .execute(&mut *db_tx)
                        .await?;

                        // Access the transparent bundle part
                        if let Some(transparent_bundle) = transaction.transparent_bundle() {
                            info!("Transaction: {}", transaction.txid());
                            info!("Transparent inputs: {}", transparent_bundle.vin.len());

                            let vins = &transparent_bundle.vin;
                            for vin in vins.iter() {
                                // The "nullifier" of a transparent input is the outpoint
                                let mut nf = vec![];
                                vin.prevout().write(&mut nf)?;

                                let row: Option<(u32, i64)> = sqlx::query_as(
                                "SELECT id_note, value FROM notes WHERE account = ?1 AND nullifier = ?2",
                            )
                            .bind(account)
                            .bind(&nf)
                            .fetch_optional(&mut *db_tx)
                            .await?;

                                if let Some((id, amount)) = row {
                                    // note was found
                                    // add a spent entry
                                    sqlx::query(
                                        "INSERT INTO spends (account, id_note, pool, tx, height, value)
                                SELECT ?, ?, 0, tx.id_tx, ?, ? FROM transactions tx WHERE tx.txid = ?
                                AND account = ? ON CONFLICT DO NOTHING",
                                    )
                                    .bind(account)
                                    .bind(id)
                                    .bind(height)
                                    .bind(-amount)
                                    .bind(&txid)
                                    .bind(account)
                                    .execute(&mut *db_tx)
                                    .await?;
                                }
                            }

                            let vouts = &transparent_bundle.vout;
                            for (i, vout) in vouts.iter().enumerate() {
                                if let Some(address) = vout.recipient_address() {
                                    if address == my_address {
                                        // It is for me
                                        // add a new note entry
                                        let mut nf = transaction.txid().as_ref().to_vec();
                                        nf.extend_from_slice(&(i as u32).to_le_bytes());

                                        sqlx::query("INSERT INTO notes (account, height, pool, tx, taddress, nullifier, value)
                                    SELECT ?, ?, 0, tx.id_tx, ?, ?, ? FROM transactions tx WHERE tx.txid = ?
                                    AND account = ? ON CONFLICT DO NOTHING")
                                        .bind(account)
                                        .bind(height)
                                        .bind(address_row.0)
                                        .bind(&nf)
                                        .bind(vout.value().into_u64() as i64)
                                        .bind(&txid)
                                        .bind(account)
                                        .execute(&mut *db_tx)
                                        .await?;
                                    }
                                }
                            }

                            info!("Transparent outputs: {}", transparent_bundle.vout.len());
                        }
                    }
                    else {
                        // No more transactions
                        break;
                    }
                }
            }
        }

        sqlx::query("UPDATE sync_heights SET height = ? WHERE account = ? AND pool = 0")
            .bind(end_height)
            .bind(account)
            .execute(&mut *db_tx)
            .await?;
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
) -> Result<(CommitmentTreeFrontier, CommitmentTreeFrontier)> {
    let min_height: u32 = network
        .activation_height(zcash_protocol::consensus::NetworkUpgrade::Sapling)
        .unwrap()
        .into();

    if height < min_height {
        return Ok((
            CommitmentTreeFrontier::default(),
            CommitmentTreeFrontier::default(),
        ));
    }

    let (sapling_tree, orchard_tree) = client.tree_state(height).await?;

    fn decode_tree_state(tree: &[u8]) -> CommitmentTreeFrontier {
        if tree.is_empty() {
            CommitmentTreeFrontier::default()
        } else {
            CommitmentTreeFrontier::read(tree).unwrap()
        }
    }

    let sapling = decode_tree_state(&sapling_tree);
    let orchard = decode_tree_state(&orchard_tree);

    Ok((sapling, orchard))
}

#[allow(clippy::too_many_arguments)]
pub async fn shielded_sync(
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
    let activation_height: u32 = network
        .activation_height(NetworkUpgrade::Sapling)
        .unwrap()
        .into();
    let start = start.max(activation_height);
    let end = end.max(activation_height);

    let accounts = accounts.to_vec();
    let db_writer_task = {
        let (s, o) = get_tree_state(network, client, start - 1).await?;

        info!("get compact block range");
        let blocks = get_compact_block_range(network, client, start, end).await?;
        info!("got streaming blocks");
        let (tx_messages, mut rx_messages) = channel::<WarpSyncMessage>(100);

        let mut connection = pool.acquire().await?;
        // get the list of transaction heights for which the time is 0
        // because raw transactions do not have timestamp (it comes from the block header)
        let heights_without_time = get_heights_without_time(&mut connection, start, end).await?;

        let mut writer_connection = pool.acquire().await?;
        let network = *network;
        let mut messages = vec![];
        let db_writer_task = tokio::spawn(async move {
            info!("[db handler] starting");
            while let Some(msg) = rx_messages.recv().await {
                //info!("Received message: {:?}", msg);
                if let WarpSyncMessage::Commit = msg {
                    let mut db_tx = writer_connection.begin().await.unwrap();
                    let mut new_messages = vec![];
                    mem::swap(&mut new_messages, &mut messages);
                    for msg in new_messages {
                        match handle_message(&network, &mut db_tx, msg, &tx_progress).await {
                            Ok(_) => {}
                            Err(e) => {
                                info!("ERROR HANDLING MESSAGE: {:?}", e);
                                return Err(e);
                            }
                        }
                    }
                    db_tx.commit().await.unwrap();
                    info!("Committing transaction");
                } else {
                    messages.push(msg);
                }
            }

            let mut db_tx = writer_connection.begin().await.unwrap();
            for msg in messages {
                match handle_message(&network, &mut db_tx, msg, &tx_progress).await {
                    Ok(_) => {}
                    Err(e) => {
                        info!("ERROR HANDLING MESSAGE: {:?}", e);
                        return Err(e);
                    }
                }
            }
            db_tx.commit().await.unwrap();

            info!("[db handler] stopped");
            check_witness_consistency(&mut writer_connection).await?;

            Ok::<_, anyhow::Error>(())
        });

        tokio::spawn(async move {
            info!("Start sync");
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
                tx_messages.clone(),
                rx_cancel,
            )
            .await
            {
                tracing::error!("Error during warp sync: {:?}", e);
                let _ = tx_messages.send(WarpSyncMessage::Error(e)).await;
            }

            info!("Sync finished");
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
) -> Result<()> {
    tracing::info!(target: "warp", "Warp Message: {msg}");
    match msg {
        WarpSyncMessage::Transaction(tx) => {
            // ignore duplicate transactions because they could have been created
            // by a previous type of scan (i.e transparent)
            sqlx::query(
                "INSERT INTO transactions (account, txid, height, time) VALUES (?, ?, ?, ?)
                ON CONFLICT DO NOTHING",
            )
            .bind(tx.account)
            .bind(&tx.txid)
            .bind(tx.height)
            .bind(tx.time)
            .execute(&mut **db_tx)
            .await?;
            info!("Processing Transaction: id={}, height={}", tx.id, tx.height);
        }
        WarpSyncMessage::Note(note) => {
            let r = sqlx::query
                    ("INSERT INTO notes
                        (account, height, pool, scope, tx, nullifier, value, cmx, position, diversifier, rcm, rho)
                        SELECT t.account, ?, ?, ?, t.id_tx, ?, ?, ?, ?, ?, ?, ? FROM transactions t
                        WHERE t.account = ? AND t.txid = ?")
                    .bind(note.height)
                    .bind(note.pool)
                    .bind(note.scope)
                    .bind(&note.nf)
                    .bind(note.value as i64)
                    .bind(&note.cmx)
                    .bind(note.position)
                    .bind(&note.diversifier)
                    .bind(&note.rcm)
                    .bind(&note.rho)
                    .bind(note.account)
                    .bind(&note.txid)
                    .execute(&mut **db_tx).await?;
            info!(
                "Processing Note: id={}, account={}, height={}",
                note.id, note.account, note.height
            );
            info!("{:?}", note);
            assert_eq!(r.rows_affected(), 1);
        }
        WarpSyncMessage::Witness(account, height, cmx, witness) => {
            let w = bincode::encode_to_vec(&witness, config::legacy())?;
            let r = sqlx::query(
                "INSERT INTO witnesses (account, note, height, witness)
                        SELECT ?, n.id_note, ?, ? FROM notes n
                        WHERE n.account = ? AND n.cmx = ?",
            )
            .bind(account)
            .bind(height)
            .bind(&w)
            .bind(account)
            .bind(&cmx)
            .execute(&mut **db_tx)
            .await?;
            assert_eq!(r.rows_affected(), 1);
        }
        WarpSyncMessage::Spend(utxo) => {
            // note does not belong to the tx because the tx is spending the note
            // and not creating it, do not join n with t!
            let r = sqlx::query(
                "INSERT INTO spends (id_note, account, height, pool, tx, value)
                    SELECT n.id_note, ?1, t.height, ?2, t.id_tx, ?3 FROM notes n, transactions t
                    WHERE n.account = ?1 AND n.cmx = ?4
                    AND t.txid = ?5 AND t.account = ?1",
            )
            .bind(utxo.account)
            .bind(utxo.pool)
            .bind(-(utxo.value as i64))
            .bind(&utxo.cmx)
            .bind(&utxo.txid)
            .execute(&mut **db_tx)
            .await?;
            info!("Processing Spend: {:?}", &utxo);
            assert_eq!(r.rows_affected(), 1);
        }
        WarpSyncMessage::Checkpoint(accounts, pool, height) => {
            for a in accounts {
                if has_pool(db_tx, a, pool).await? {
                    sqlx::query(
                        "UPDATE sync_heights SET height = ?3
                        WHERE account = ?1 AND pool = ?2",
                    )
                    .bind(a)
                    .bind(pool)
                    .bind(height)
                    .execute(&mut **db_tx)
                    .await?;
                    info!("Checkpoint for account: {}, height: {}", a, height);
                }
                let _ = tx_progress.send(SyncProgress { height, time: 0 }).await;
            }
        }
        WarpSyncMessage::BlockHeader(block_header) => {
            info!("Processing BlockHeader: {:?}", block_header);
            // ignore dups because we could have already inserted the block header
            // if a transparent transaction needs it
            // to resolve the time of the transaction
            sqlx::query(
                "INSERT INTO headers (height, hash, time)
                    VALUES (?, ?, ?) ON CONFLICT DO NOTHING",
            )
            .bind(block_header.height)
            .bind(&block_header.hash)
            .bind(block_header.time)
            .execute(&mut **db_tx)
            .await?;
            sqlx::query("UPDATE transactions SET time = ? WHERE height = ?")
                .bind(block_header.time)
                .bind(block_header.height)
                .execute(&mut **db_tx)
                .await?;
        }
        WarpSyncMessage::Commit => {
            // handled in the caller
        }
        WarpSyncMessage::Rewind(accounts, height) => {
            info!("Discard height: {}", height);
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

// remove synchronization data (notes, spends, transactions, witnesses) after the given height
// keep the data at the given height
// do not remove headers because they are used by multiple accounts
pub async fn trim_sync_data(
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<()> {
    let mut db_tx = connection.begin().await?;
    sqlx::query("DELETE FROM notes WHERE height > ? AND account = ?")
        .bind(height)
        .bind(account)
        .execute(&mut *db_tx)
        .await?;
    sqlx::query("DELETE FROM spends WHERE height > ? AND account = ?")
        .bind(height)
        .bind(account)
        .execute(&mut *db_tx)
        .await?;
    sqlx::query("DELETE FROM transactions WHERE height > ? AND account = ?")
        .bind(height)
        .bind(account)
        .execute(&mut *db_tx)
        .await?;
    sqlx::query("DELETE FROM witnesses WHERE height > ? AND account = ?")
        .bind(height)
        .bind(account)
        .execute(&mut *db_tx)
        .await?;
    sqlx::query("DELETE FROM outputs WHERE height > ? AND account = ?")
        .bind(height)
        .bind(account)
        .execute(&mut *db_tx)
        .await?;
    sqlx::query("DELETE FROM memos WHERE height > ? AND account = ?")
        .bind(height)
        .bind(account)
        .execute(&mut *db_tx)
        .await?;
    sqlx::query("UPDATE sync_heights SET height = ? WHERE account = ?")
        .bind(height)
        .bind(account)
        .execute(&mut *db_tx)
        .await?;

    db_tx.commit().await?;
    Ok(())
}

pub async fn check_witness_consistency(connection: &mut SqliteConnection) -> Result<()> {
    let notes = sqlx::query(
    "WITH utxo AS (SELECT * FROM notes n LEFT JOIN spends s ON n.id_note = s.id_note WHERE s.id_note IS NULL),
    db_height AS (SELECT * FROM sync_heights)
    SELECT u.account, u.pool, u.height, u.value, d.height FROM utxo u
    JOIN db_height d ON d.account = u.account AND d.pool = u.pool
    JOIN witnesses w ON u.id_note = w.note AND w.account = u.account
    AND w.height = d.height
    AND w.note IS NULL
    ")
    .map(|r: SqliteRow| {
        let account: u32 = r.get(0);
        let pool: u8 = r.get(1);
        let height: u32 = r.get(2);
        let value: u64 = r.get(3);
        let db_height: u32 = r.get(4);
        (account, pool, height, value, db_height)
    })
    .fetch_all(connection).await?;

    for (account, pool, height, value, db_height) in notes.iter() {
        info!("Missing witness for note {pool} {height} {value} of account {account} at height {db_height}");
    }
    if !notes.is_empty() {
        anyhow::bail!("Some notes have no witness data. Abort Sync");
    }
    info!("Db check passed");
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
    let prev_height =
        sqlx::query("SELECT MAX(height) FROM witnesses WHERE height < ? AND account = ?")
            .bind(height)
            .bind(account)
            .map(|row: SqliteRow| {
                let height: Option<u32> = row.get(0);
                height
            })
            .fetch_one(&mut *connection)
            .await?;

    if let Some(prev_height) = prev_height {
        trim_sync_data(&mut *connection, account, prev_height).await?;
    } else {
        crate::account::reset_sync(network, &mut *connection, account).await?;
    }

    // then trim the headers because there are no accounts using them
    sqlx::query("DELETE FROM headers WHERE height > ?")
        .bind(height)
        .execute(connection)
        .await?;

    Ok(())
}

pub async fn prune_old_checkpoints(
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<()> {
    // find the latest checkpoint before the given height
    let checkpoint_height =
        sqlx::query("SELECT MAX(height) FROM witnesses WHERE account = ? AND height < ?")
            .bind(account)
            .bind(height)
            .map(|row: SqliteRow| {
                let height: Option<u32> = row.get(0);
                height
            })
            .fetch_one(&mut *connection)
            .await?;
    // delete all witnesses before the checkpoint height
    if let Some(checkpoint_height) = checkpoint_height {
        sqlx::query("DELETE FROM witnesses WHERE account = ? AND height < ?")
            .bind(account)
            .bind(checkpoint_height)
            .execute(&mut *connection)
            .await?;
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
            SELECT h.height, COALESCE(h.time, 0) FROM headers h
            JOIN mh ON h.height = mh.min_height",
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
    tokio::spawn(async move {
        let mut n_added = 0;
        let tk = select_account_transparent(&mut connection, account, dindex).await?;
        let xvk = tk.xvk;
        let start_height = get_birth_height(&mut connection, account).await?;
        for scope in 0..2 {
            let mut dindex = 0;
            let mut gap = 0;
            loop {
                let (pk, taddr) = match xvk.as_ref() {
                    Some(xvk) => derive_transparent_address(xvk, scope, dindex)?,
                    None if hw != 0 => {
                        get_hw_transparent_address(&network, aindex, scope, dindex).await?
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
                            let sk = if let Some(tsk) = tk.xsk.as_ref() {
                                let sk = derive_transparent_sk(tsk, scope, dindex)?;
                                Some(sk)
                            } else {
                                None
                            };
                            if store_account_transparent_addr(
                                &mut connection, account, scope, dindex, sk, &pk, &taddr,
                            )
                            .await?
                            {
                                n_added += 1;
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
        Ok(n_added)
    });
    Ok(())
}

pub async fn get_heights_without_time(
    connection: &mut SqliteConnection,
    start: u32,
    end: u32,
) -> Result<HashSet<u32>> {
    let mut tx_without_time: HashSet<u32> = sqlx::query(
        "SELECT DISTINCT height FROM transactions WHERE time = 0
        AND height >= ? AND height <= ?",
    )
    .bind(start)
    .bind(end)
    .map(|row: SqliteRow| {
        let height: u32 = row.get(0);
        height
    })
    .fetch_all(&mut *connection)
    .await?
    .into_iter()
    .collect();

    let synced_heights_without_time = sqlx::query(
        "SELECT sh.height FROM sync_heights sh
        LEFT JOIN headers h ON sh.height = h.height
        WHERE h.time IS NULL",
    )
    .map(|row: SqliteRow| {
        let height: u32 = row.get(0);
        height
    })
    .fetch_all(&mut *connection)
    .await?
    .into_iter();
    tx_without_time.extend(synced_heights_without_time);

    Ok(tx_without_time)
}
