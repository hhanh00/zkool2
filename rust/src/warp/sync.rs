use std::sync::Arc;

use anyhow::Result;
use shielded::Synchronizer;
use sqlx::{Pool, Sqlite};
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tonic::Streaming;
use tracing::info;
use zcash_protocol::consensus::Network;

use crate::{
    lwd::CompactBlock,
    sync::WarpSyncMessage,
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
    connection: &Pool<Sqlite>,
    mut height: u32,
    accounts: &[u32],
    mut blocks: Streaming<CompactBlock>,
    sapling_state: &CommitmentTreeFrontier,
    orchard_state: &CommitmentTreeFrontier,
    tx_decrypted: Sender<WarpSyncMessage>,
) -> Result<(), SyncError> {
    let sap_hasher = SaplingHasher::default();
    let mut sap_dec = SaplingSync::new(
        network.clone(),
        connection,
        1,
        height,
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
        height,
        accounts,
        tx_decrypted.clone(),
        orchard_state.size() as u32,
        orchard_state.to_edge(&orch_hasher),
    )
    .await?;

    let mut bs = vec![];
    let mut c = 0; // count of outputs & actions

    println!("Start sync");
    while let Some(block) = blocks.message().await? {
        // println!("Syncing block {}: {c}", block.height);
        // TODO: Reorg detection
        // bh = BlockHeader {
        //     height: block.height as u32,
        //     hash: block.hash.clone().try_into().unwrap(),
        //     prev_hash: block.prev_hash.clone().try_into().unwrap(),
        //     timestamp: block.time,
        // };
        // if prev_hash != bh.prev_hash {
        //     rewind_checkpoint(&coin.network, &mut connection, &mut client).await?;
        //     return Err(SyncError::Reorg(bh.height));
        // }
        // prev_hash = bh.hash;

        for vtx in block.vtx.iter() {
            c += vtx.outputs.len();
            c += vtx.actions.len();
        }

        bs.push(block);

        if c >= 10000 {
            println!("Processing {} blocks, {} outputs/actions", bs.len(), c);
            sap_dec.add(&bs).await?;
            orch_dec.add(&bs).await?;
            let _ = tx_decrypted.send(WarpSyncMessage::Commit).await;
            bs.clear();
            c = 0;
        }
        height += 1;
    }
    println!("Processing {} blocks, {} outputs/actions", bs.len(), c);
    sap_dec.add(&bs).await?;
    orch_dec.add(&bs).await?;
    let _ = tx_decrypted.send(WarpSyncMessage::Commit).await;
    println!("Sync finished");

    Ok(())
}
