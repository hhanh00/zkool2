use anyhow::Result;
use flutter_rust_bridge::frb;
use tonic::{Request, Streaming};
use crate::{lwd::{BlockId, BlockRange, CompactBlock, TreeState}, warp::{legacy::CommitmentTreeFrontier, Witness}, Client, Hash32};

#[frb(dart_metadata = ("freezed"))]
#[derive(Default, Debug)]
pub struct Transaction {
    pub id: u32,
    pub txid: Hash32,
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

#[derive(Debug)]
pub struct UTXO {
    pub id: u32,
    pub account: u32,
    pub nullifier: Vec<u8>,
    pub value: u64,
    pub position: u32,
    pub witness: Witness,
}

#[derive(Debug)]
pub enum WarpSyncMessage {
    BlockHeader(BlockHeader),
    Transaction(Transaction),
    Note(Note),
    Witness(Witness),
    Checkpoint(u32),
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
#[derive(Clone, Default, Debug)]
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

    pub ivtx: u32, // not stored in the database
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
    client: &mut Client,
    height: u32,
) -> Result<(CommitmentTreeFrontier, CommitmentTreeFrontier)> {
    let height: u32 = height.into();
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
