use crate::{
    lwd::{BlockId, BlockRange, CompactBlock, TreeState},
    warp::{legacy::CommitmentTreeFrontier, sync::warp_sync, Witness},
    Client, Hash32,
};
use anyhow::Result;
use bincode::config;
use flutter_rust_bridge::frb;
use sqlx::{Pool, Sqlite};
use tokio::sync::mpsc::channel;
use tonic::{Request, Streaming};
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
    pub position: u32,
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
}

#[frb(dart_metadata = ("freezed"))]
#[derive(Debug)]
pub struct BlockHeader {
    pub height: u32,
    pub hash: Hash32,
    pub time: u32,
}

#[frb(dart_metadata = ("freezed"))]
#[derive(Clone, Default)]
pub struct Note {
    pub id: u32,
    pub account: u32,
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
    pool: &Pool<Sqlite>,
    client: &mut Client,
    accounts: Vec<u32>,
    start: u32,
    end: u32,
) -> Result<()> {
    let (s, o) = get_tree_state(network, client, start - 1).await?;

    println!("get compact block range");
    let blocks = get_compact_block_range(client, start, end).await?;
    println!("got streaming blocks");
    let (tx_messages, mut rx_messages) = channel::<WarpSyncMessage>(100);
    let pool2 = pool.clone();

    tokio::spawn(async move {
        println!("[db handler] starting");
        let mut db_tx = pool2.begin().await?;
        while let Some(msg) = rx_messages.recv().await {
            //println!("Received message: {:?}", msg);
            if let WarpSyncMessage::Commit = msg {
                db_tx.commit().await.unwrap();
                println!("Committing transaction");
                db_tx = pool2.begin().await.unwrap();
            } else {
                match handle_message(&mut db_tx, msg).await {
                    Ok(_) => {}
                    Err(e) => {
                        println!("ERROR HANDLING MESSAGE: {:?}", e);
                    }
                }
            }
        }
        db_tx.commit().await?;
        println!("[db handler] stopped");

        Ok::<_, anyhow::Error>(())
    });
    warp_sync(
        &network,
        &pool.clone(),
        start,
        &accounts,
        blocks,
        &s,
        &o,
        tx_messages.clone(),
    )
    .await?;
    Ok(())
}

async fn handle_message(
    db_tx: &mut sqlx::Transaction<'_, Sqlite>,
    msg: WarpSyncMessage,
) -> Result<()> {
    match msg {
        WarpSyncMessage::Transaction(tx) => {
            sqlx::query
                // ignore duplicate transactions because they could have been created
                // by a previous type of scan (i.e transparent)
                ("INSERT INTO transactions (account, txid, height, time, value) VALUES (?, ?, ?, ?, 0)
                ON CONFLICT DO NOTHING")
                .bind(tx.account)
                .bind(&tx.txid)
                .bind(tx.height)
                .bind(tx.time)
                .execute(&mut **db_tx).await?;
            println!("Processing Transaction: id={}, height={}", tx.id, tx.height);
        }
        WarpSyncMessage::Note(note) => {
            let r = sqlx::query
                ("INSERT INTO notes
                        (account, height, pool, tx, nullifier, value, cmx, position, diversifier, rcm, rho)
                        SELECT t.account, ?, ?, t.id_tx, ?, ?, ?, ?, ?, ?, ? FROM transactions t
                        WHERE t.account = ? AND t.txid = ?")
                .bind(note.height)
                .bind(note.pool)
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
            println!(
                "Processing Note: id={}, account={}, height={}",
                note.id, note.account, note.height
            );
            println!("{:?}", note);
            assert_eq!(r.rows_affected(), 1);
        }
        WarpSyncMessage::Witness(account, height, cmx, witness) => {
            let w = bincode::encode_to_vec(&witness, config::legacy())?;
            let r = sqlx::query(
                "INSERT INTO witnesses (note, height, witness)
                        SELECT n.id_note, ?, ? FROM notes n
                        WHERE n.account = ? AND n.cmx = ?",
            )
            .bind(height)
            .bind(&w)
            .bind(account)
            .bind(&cmx)
            .execute(&mut **db_tx)
            .await?;
            println!("Processing Witness: account={account}, height={height}");
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
            println!("Processing Spend: {:?}", &utxo);
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
                println!("Checkpoint for account: {}, height: {}", a, height);
            }
        }
        _ => (),
    }

    Ok(())
}
