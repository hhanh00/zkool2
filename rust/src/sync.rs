use std::{collections::HashSet, mem};

use crate::{
    account::{derive_transparent_address, derive_transparent_sk, get_birth_height},
    api::sync::SyncProgress,
    coin::Network,
    db::{
        get_account_aindex, get_account_dindex, get_account_hw, select_account_transparent,
        store_account_transparent_addr,
    },
    io::SyncHeight,
    lwd::CompactBlock,
    warp::{
        legacy::CommitmentTreeFrontier,
        sync::{warp_sync, SyncError},
        Witness,
    },
    Client,
};
use anyhow::Result;
use bincode::config;
use flutter_rust_bridge::frb;
use sqlx::{pool::PoolConnection, Row};
use sqlx::{sqlite::SqliteRow, Connection, Sqlite, SqliteConnection, SqlitePool};
use tokio::sync::{
    broadcast,
    mpsc::{channel, Sender},
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::info;
use zcash_keys::encoding::AddressCodec;
use zcash_protocol::consensus::{NetworkUpgrade, Parameters};

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
use crate::ledger::fvk::get_hw_transparent_address;
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
use crate::no_ledger::get_hw_transparent_address;

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
    tracing::debug!(target: "warp", "Warp Message: {msg}");
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
                sqlx::query(
                    "INSERT INTO sync_heights (pool, account, height)
                        VALUES (?, ?, ?) ON CONFLICT DO UPDATE SET height = excluded.height",
                )
                .bind(pool)
                .bind(a)
                .bind(height)
                .execute(&mut **db_tx)
                .await?;
                info!("Checkpoint for account: {}, height: {}", a, height);
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
