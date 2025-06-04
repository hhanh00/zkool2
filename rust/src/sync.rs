use std::{collections::HashSet, u32};

use crate::{
    account::{derive_transparent_address, derive_transparent_sk, get_birth_height},
    api::sync::{transparent_sync, SyncProgress},
    db::{select_account_transparent, store_account_transparent_addr},
    lwd::{BlockId, BlockRange, CompactBlock, TransparentAddressBlockFilter, TreeState},
    warp::{legacy::CommitmentTreeFrontier, sync::{warp_sync, SyncError}, Witness},
    Client,
};
use anyhow::Result;
use bincode::config;
use flutter_rust_bridge::frb;
use sqlx::{sqlite::SqliteRow, Connection, Sqlite, SqliteConnection};
use sqlx::{Row, SqlitePool};
use tokio::sync::{
    broadcast,
    mpsc::{channel, Sender},
};
use tonic::{Request, Streaming};
use tracing::info;
use zcash_keys::encoding::AddressCodec;
use zcash_protocol::consensus::{Network, Parameters};

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
    client: &mut Client,
    start: u32,
    end: u32,
) -> Result<Streaming<CompactBlock>> {
    let req = || {
        Request::new(BlockRange {
            start: Some(BlockId {
                height: start as u64,
                hash: vec![],
            }),
            end: Some(BlockId {
                height: end as u64,
                hash: vec![],
            }),
            spam_filter_threshold: 0,
        })
    };
    let blocks = client.get_block_range(req()).await?.into_inner();
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

    let tree_state = client
        .get_tree_state(Request::new(BlockId {
            height: height as u64,
            hash: vec![],
        }))
        .await?
        .into_inner();

    let TreeState {
        sapling_tree,
        orchard_tree,
        ..
    } = tree_state;

    fn decode_tree_state(s: &str) -> CommitmentTreeFrontier {
        if s.is_empty() {
            CommitmentTreeFrontier::default()
        } else {
            let tree = hex::decode(s).unwrap();
            CommitmentTreeFrontier::read(&*tree).unwrap()
        }
    }

    let sapling = decode_tree_state(&sapling_tree);
    let orchard = decode_tree_state(&orchard_tree);

    Ok((sapling, orchard))
}

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
    let accounts = accounts.to_vec();
    let db_writer_task = {
        let (s, o) = get_tree_state(network, client, start - 1).await?;

        info!("get compact block range");
        let blocks = get_compact_block_range(client, start, end).await?;
        info!("got streaming blocks");
        let (tx_messages, mut rx_messages) = channel::<WarpSyncMessage>(100);
        let pool2 = pool.clone();

        // get the list of transaction heights for which the time is 0
        // because raw transactions do not have timestamp (it comes from the block header)
        let heights_without_time: HashSet<u32> = sqlx::query(
            "SELECT DISTINCT height FROM transactions WHERE time = 0
        AND height >= ? AND height <= ?",
        )
        .bind(start)
        .bind(end)
        .map(|row: SqliteRow| {
            let height: u32 = row.get(0);
            height
        })
        .fetch_all(&pool2)
        .await?
        .into_iter()
        .collect();

        let db_writer_task = tokio::spawn(async move {
            info!("[db handler] starting");
            let mut db_tx = pool2.begin().await?;
            while let Some(msg) = rx_messages.recv().await {
                //info!("Received message: {:?}", msg);
                if let WarpSyncMessage::Commit = msg {
                    db_tx.commit().await.unwrap();
                    info!("Committing transaction");
                    db_tx = pool2.begin().await.unwrap();
                } else {
                    match handle_message(&mut db_tx, msg, &tx_progress).await {
                        Ok(_) => {}
                        Err(e) => {
                            info!("ERROR HANDLING MESSAGE: {:?}", e);
                            return Err(e);
                        }
                    }
                }
            }
            db_tx.commit().await?;
            info!("[db handler] stopped");

            Ok::<_, anyhow::Error>(())
        });

        let network = network.clone();
        let pool = pool.clone();
        tokio::spawn(async move {
            info!("Start sync");
            if let Err(e) = warp_sync(
                &network,
                &pool,
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
            .await {
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
    db_tx: &mut sqlx::Transaction<'_, Sqlite>,
    msg: WarpSyncMessage,
    tx_progress: &Sender<SyncProgress>,
) -> Result<()> {
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
            info!("Rewinding to height: {}", height);
            for account in accounts {
                rewind_sync(&mut **db_tx, account, height).await?;
            }
        }
        WarpSyncMessage::Error(e) => {
            return Err(e.into());
        }
    }

    Ok(())
}

pub async fn recover_from_partial_sync(connection: &SqlitePool, accounts: &[u32]) -> Result<()> {
    let mut connection = connection.acquire().await?;
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
pub async fn trim_sync_data(connection: &mut SqliteConnection, account: u32, height: u32) -> Result<()> {
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

// for each account, find the latest checkpoint before the given height
// and trim the synchronization data to that height
pub async fn rewind_sync(connection: &mut SqliteConnection, account: u32, height: u32) -> Result<()> {
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
    }

    // then trim the headers because there are no accounts using them
    sqlx::query("DELETE FROM headers WHERE height > ?")
        .bind(height)
        .execute(connection)
        .await?;

    Ok(())
}

pub async fn prune_old_checkpoints(
    connection: &SqlitePool,
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
            .fetch_one(connection)
            .await?;
    // delete all witnesses before the checkpoint height
    if let Some(checkpoint_height) = checkpoint_height {
        sqlx::query("DELETE FROM witnesses WHERE account = ? AND height < ?")
            .bind(account)
            .bind(checkpoint_height)
            .execute(connection)
            .await?;
    }
    Ok(())
}

pub async fn get_db_height(connection: &SqlitePool, account: u32) -> Result<u32> {
    let (height,): (u32,) =
        sqlx::query_as("SELECT MIN(height) FROM sync_heights WHERE account = ?")
            .bind(account)
            .fetch_one(connection)
            .await?;
    Ok(height)
}

pub async fn transparent_sweep(
    network: &Network,
    connection: &SqlitePool,
    client: &mut Client,
    account: u32,
    end_height: u32,
    gap_limit: u32,
) -> Result<u32> {
    let mut n_added = 0;
    let tk = select_account_transparent(connection, account).await?;
    let xvk = tk.xvk;
    if let Some(xvk) = xvk.as_ref() {
        let start_height = get_birth_height(connection, account).await?;
        for scope in 0..2 {
            let mut dindex = 0;
            let mut gap = 0;
            loop {
                let (pk, taddr) = derive_transparent_address(xvk, scope, dindex)?;
                let taddr = taddr.encode(network);
                let mut txids = client
                    .get_taddress_txids(Request::new(TransparentAddressBlockFilter {
                        address: taddr.clone(),
                        range: Some(BlockRange {
                            start: Some(BlockId {
                                height: start_height as u64,
                                hash: vec![],
                            }),
                            end: Some(BlockId {
                                height: end_height as u64,
                                hash: vec![],
                            }),
                            spam_filter_threshold: 0,
                        }),
                    }))
                    .await?
                    .into_inner();
                if txids.message().await?.is_some() {
                    let sk = if let Some(tsk) = tk.xsk.as_ref() {
                        let sk = derive_transparent_sk(tsk, scope, dindex)?;
                        Some(sk)
                    } else {
                        None
                    };
                    if store_account_transparent_addr(
                        connection, account, scope, dindex, sk, &pk, &taddr,
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

        let (tx, _) = broadcast::channel::<()>(1);

        transparent_sync(
            network,
            connection,
            client,
            std::slice::from_ref(&account),
            start_height,
            end_height,
            u32::MAX,
            tx.subscribe(),
        )
        .await?;
    }
    Ok(n_added)
}
