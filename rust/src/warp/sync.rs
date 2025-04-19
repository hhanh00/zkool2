use std::collections::HashSet;

use anyhow::Result;
use shielded::Synchronizer;
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tonic::Streaming;
use tracing::info;
use zcash_protocol::consensus::Network;

use crate::{
    lwd::CompactBlock,
    sync::{BlockHeader, WarpSyncMessage},
    warp::hasher::{OrchardHasher, SaplingHasher},
};

use super::legacy::CommitmentTreeFrontier;

// pub mod builder;
mod shielded;
// mod transparent;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Reorganization detected at block {0}")]
    Reorg(u32),
    #[error(transparent)]
    Tonic(#[from] tonic::Status),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type SaplingSync = Synchronizer<shielded::sapling::SaplingProtocol>;
pub type OrchardSync = Synchronizer<shielded::orchard::OrchardProtocol>;

pub async fn warp_sync(
    network: &Network,
    connection: &SqlitePool,
    start_height: u32,
    accounts: &[(u32, bool)],
    mut blocks: Streaming<CompactBlock>,
    mut heights_without_time: HashSet<u32>,
    sapling_state: &CommitmentTreeFrontier,
    orchard_state: &CommitmentTreeFrontier,
    tx_decrypted: Sender<WarpSyncMessage>,
) -> Result<(), SyncError> {
    let sap_hasher = SaplingHasher::default();
    let mut sap_dec = SaplingSync::new(
        network.clone(),
        connection,
        1,
        start_height,
        accounts,
        tx_decrypted.clone(),
        sapling_state.size() as u32,
        sapling_state.to_edge(&sap_hasher),
    )
    .await?;

    let orch_hasher = OrchardHasher::default();
    let mut orch_dec = OrchardSync::new(
        network.clone(),
        connection,
        2,
        start_height,
        accounts,
        tx_decrypted.clone(),
        orchard_state.size() as u32,
        orchard_state.to_edge(&orch_hasher),
    )
    .await?;

    let mut bs = vec![];
    let mut c = 0; // count of outputs & actions

    async fn flush(
        c: &mut usize,
        bs: &mut Vec<CompactBlock>,
        sap_dec: &mut SaplingSync,
        orch_dec: &mut OrchardSync,
        tx_decrypted: &Sender<WarpSyncMessage>,
    ) -> Result<(), SyncError> {
        info!("Processing {} blocks, {} outputs/actions", bs.len(), c);
        sap_dec.add(&bs).await?;
        orch_dec.add(&bs).await?;
        let lcb = bs.last().unwrap();
        let bh = BlockHeader {
            height: lcb.height as u32,
            hash: lcb.hash.clone(),
            time: lcb.time,
        };
        let _ = tx_decrypted.send(WarpSyncMessage::BlockHeader(bh)).await;
        let _ = tx_decrypted.send(WarpSyncMessage::Commit).await;
        bs.clear();
        *c = 0;

        Ok(())
    }

    let mut prev_hash = sqlx::query("SELECT hash FROM headers WHERE height = ?")
        .bind(start_height - 1)
        .map(|row: SqliteRow| row.get::<Vec<u8>, _>(0))
        .fetch_optional(connection)
        .await.unwrap();

    info!("Start sync");
    while let Some(block) = blocks.message().await? {
        // info!("Syncing block {}: {c}", block.height);
        let block_prev_hash = block.prev_hash.clone();
        if let Some(prev_hash) = prev_hash {
            if prev_hash != block_prev_hash {
                // we need to rewind the database to the previous checkpoint
                // and start syncing from there next time we get synchronize
                for (account, _) in accounts.iter() {
                    crate::sync::rewind_sync(connection, *account, start_height - 1).await?;
                }
                return Err(SyncError::Reorg(block.height as u32));
            }
        }
        prev_hash = Some(block.hash.clone());

        let bheight = block.height as u32;
        if heights_without_time.remove(&bheight) {
            let bh = BlockHeader {
                height: bheight,
                hash: block.hash.clone(),
                time: block.time,
            };
            let _ = tx_decrypted.send(WarpSyncMessage::BlockHeader(bh)).await;
        }

        for vtx in block.vtx.iter() {
            c += vtx.outputs.len();
            c += vtx.actions.len();
        }

        bs.push(block);

        if c >= 10000 {
            if !bs.is_empty() {
                flush(&mut c, &mut bs, &mut sap_dec, &mut orch_dec, &tx_decrypted).await?;
            }
        }
    }
    if !bs.is_empty() {
        flush(&mut c, &mut bs, &mut sap_dec, &mut orch_dec, &tx_decrypted).await?;
    }
    info!("Sync finished");

    Ok(())
}
