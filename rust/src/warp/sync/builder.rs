use anyhow::Result;
use prost::Message as _;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;

use crate::{
    lwd::rpc::{CompactBlock, CompactTx},
    warp::{
        hasher::{OrchardHasher, SaplingHasher},
        legacy::CommitmentTreeFrontier,
        Edge, Hasher, MERKLE_DEPTH,
    },
    Hash,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct BridgeLevel {
    pub head: Option<Either>,
    pub tail: Option<Either>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bridge {
    pub start: u32,
    pub len: u32,
    pub levels: Vec<BridgeLevel>,
    pub closed: bool,
    pub s: i32,
    pub e: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Either {
    #[serde(with = "serde_bytes")]
    Left(Hash),
    #[serde(with = "serde_bytes")]
    Right(Hash),
}

pub trait CompactTxCMXExtractor {
    fn items(tx: &CompactTx) -> impl Iterator<Item = Hash>;
    fn len(tx: &CompactTx) -> usize;
}

impl CompactTxCMXExtractor for SaplingHasher {
    fn len(tx: &CompactTx) -> usize {
        tx.outputs.len()
    }

    fn items(tx: &CompactTx) -> impl Iterator<Item = Hash> {
        tx.outputs.iter().map(|o| o.cmu.clone().try_into().unwrap())
    }
}

impl CompactTxCMXExtractor for OrchardHasher {
    fn len(tx: &CompactTx) -> usize {
        tx.actions.len()
    }

    fn items(tx: &CompactTx) -> impl Iterator<Item = Hash> {
        tx.actions.iter().map(|a| a.cmx.clone().try_into().unwrap())
    }
}

#[derive(Default, Debug)]
pub struct Bdge {
    pub len: u32,
    pub start: Edge,
    pub end: Edge,
}

impl Edge {
    pub fn to_rpc(&self) -> crate::lwd::rpc::Edge {
        let levels = self
            .0
            .iter()
            .map(|oh| crate::lwd::rpc::OptLevel {
                hash: match oh {
                    Some(h) => h.to_vec(),
                    None => vec![],
                },
            })
            .collect::<Vec<_>>();
        crate::lwd::rpc::Edge { levels }
    }
}

impl Bdge {
    pub fn to_rpc(&self) -> crate::lwd::rpc::Bridge {
        crate::lwd::rpc::Bridge {
            len: self.len,
            start: Some(self.start.to_rpc()),
            end: Some(self.end.to_rpc()),
        }
    }
}

pub struct BridgeBuilder<H: Hasher + CompactTxCMXExtractor> {
    pub hasher: H,
    position: u32,
    edge: Edge,
    cmxs: Vec<Hash>,
}

impl<H: Hasher + CompactTxCMXExtractor> BridgeBuilder<H> {
    pub fn new(start: &CommitmentTreeFrontier, hasher: H) -> Self {
        let s = start.to_edge(&hasher);
        Self {
            hasher,
            position: start.size() as u32,
            edge: s,
            cmxs: vec![],
        }
    }

    pub fn add<I: IntoIterator<Item = Hash>>(&mut self, values: I) {
        self.cmxs.extend(values);
    }

    pub fn flush(&mut self) -> Bdge {
        let mut bridge = Bdge::default();
        let mut p = self.position;
        bridge.len = self.cmxs.len() as u32;
        self.position += bridge.len;

        for depth in 0..MERKLE_DEPTH as usize {
            if p % 2 == 1 {
                let ps = self.edge.0[depth].unwrap();
                self.cmxs.insert(0, ps);
                p -= 1;
            }

            let len = self.cmxs.len();
            if len >= 2 {
                bridge.start.0[depth] = Some(self.cmxs[1]);
            }
            if len % 2 == 1 {
                bridge.end.0[depth] = Some(self.cmxs[len - 1]);
            }

            let mut new_cmxs = self
                .hasher
                .parallel_combine(depth as u8, &self.cmxs, len / 2);
            std::mem::swap(&mut new_cmxs, &mut self.cmxs);

            p /= 2;
        }

        self.cmxs.clear();
        self.edge = bridge.end.clone();
        bridge
    }
}

pub async fn purge_blocks(
    connection: PooledConnection<SqliteConnectionManager>,
    mut blocks: Receiver<CompactBlock>,
    s: &CommitmentTreeFrontier,
    o: &CommitmentTreeFrontier,
) -> Result<()> {
    let mut sb = BridgeBuilder::new(s, SaplingHasher::default());
    let mut ob = BridgeBuilder::new(o, OrchardHasher::default());
    while let Some(mut cb) = blocks.recv().await {
        if cb.height % 100_000 == 0 {
            tracing::info!("Current height: {}", cb.height);
        }
        for tx in cb.vtx.iter_mut() {
            if tx.outputs.len() < 32 {
                sb.add(tx.outputs.iter().map(|o| o.cmu.clone().try_into().unwrap()));
            } else {
                tx.spends.clear();
                sb.flush();
                sb.add(tx.outputs.iter().map(|o| o.cmu.clone().try_into().unwrap()));
                let bridge = sb.flush();
                tx.outputs.clear();
                tx.sapling_bridge = Some(bridge.to_rpc());
            }

            if tx.actions.len() < 32 {
                ob.add(tx.actions.iter().map(|a| a.cmx.clone().try_into().unwrap()));
            } else {
                ob.flush();
                ob.add(tx.actions.iter().map(|a| a.cmx.clone().try_into().unwrap()));
                let bridge = ob.flush();
                tx.actions.clear();
                tx.orchard_bridge = Some(bridge.to_rpc());
            }
        }
        let enc = cb.encode_to_vec();
        connection.execute(
            "INSERT INTO cp_blk(height, data)
            VALUES (?1, ?2) ON CONFLICT DO UPDATE SET
            data = excluded.data",
            params![cb.height, &enc],
        )?;
    }

    Ok(())
}
