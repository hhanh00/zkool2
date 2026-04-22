use std::{collections::HashSet, time::Duration};

use anyhow::Result;
use shielded::Synchronizer;
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};
use tokio::sync::{broadcast, mpsc::Sender};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tracing::info;
use zcash_trees::network::Network;

use crate::{
    lwd::CompactBlock,
    warp::hasher::{OrchardHasher, SaplingHasher},
};
use zcash_trees::types::{BlockHeader, WarpSyncMessage};

use super::legacy::CommitmentTreeFrontier;

pub use zcash_trees::types::SyncError;

mod shielded;

pub type SaplingSync = Synchronizer<shielded::sapling::SaplingProtocol>;
pub type OrchardSync = Synchronizer<shielded::orchard::OrchardProtocol>;

pub enum BlockMessage {
    Chunk(Vec<CompactBlock>),
    SaveHeader(BlockHeader),
    Cancel,
    StallAbort,
    Reorg(Vec<u32>, u32),
}

#[allow(clippy::too_many_arguments)]
pub async fn warp_sync(
    network: &Network,
    connection: &mut SqliteConnection,
    start_height: u32,
    accounts: &[(u32, bool)],
    mut blocks: ReceiverStream<CompactBlock>,
    mut heights_without_time: HashSet<u32>,
    actions_per_sync: u32,
    sapling_state: &CommitmentTreeFrontier,
    orchard_state: &CommitmentTreeFrontier,
    tx_decrypted: Sender<WarpSyncMessage>,
    mut rx_cancel: broadcast::Receiver<()>,
) -> Result<(), SyncError> {
    let sap_hasher = SaplingHasher::default();

    let mut sap_dec = SaplingSync::new(
        *network,
        &mut *connection,
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
        *network,
        &mut *connection,
        2,
        start_height,
        accounts,
        tx_decrypted.clone(),
        orchard_state.size() as u32,
        orchard_state.to_edge(&orch_hasher),
    )
    .await?;

    if sap_dec.has_no_keys() && orch_dec.has_no_keys() {
        info!("No keys to sync");
        return Ok(());
    }

    let mut prev_hash = sqlx::query("SELECT hash FROM headers WHERE height = ?")
        .bind(start_height - 1)
        .map(|row: SqliteRow| row.get::<Vec<u8>, _>(0))
        .fetch_optional(&mut *connection)
        .await
        .unwrap();

    let account_ids = accounts.iter().map(|(id, _)| *id).collect::<Vec<_>>();
    let (tx_blocks, mut rx_blocks) = tokio::sync::mpsc::channel::<BlockMessage>(2);
    tokio::spawn(async move {
        let mut bs = vec![];

        let mut interval = tokio::time::interval(Duration::from_secs(60));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut current_height = start_height;
        let mut prev_current_height = 0;

        let mut c = 0;
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    info!("Syncing at height {}", current_height);
                    if prev_current_height == current_height {
                        info!("Connection stalled. Aborting...");
                        let _ = tx_blocks.send(BlockMessage::StallAbort).await;
                        break;
                    }
                    prev_current_height = current_height;
                }
                _ = rx_cancel.recv() => {
                    info!("Sync cancelled");
                    let _ = tx_blocks.send(BlockMessage::Cancel).await;
                    break;
                }
                m = blocks.next() => {
                    if let Some(block) = m {
                        let block_prev_hash = block.prev_hash.clone();
                        current_height = block.height as u32;
                        if let Some(prev_hash) = prev_hash {
                            if prev_hash != block_prev_hash {
                                let _ = tx_blocks.send(BlockMessage::Reorg(account_ids, current_height - 1)).await;
                                info!("Reorganization detected at block {}", block.height);
                                break;
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
                            let _ = tx_blocks.send(BlockMessage::SaveHeader(bh)).await;
                        }

                        for vtx in block.vtx.iter() {
                            c += vtx.outputs.len();
                            c += vtx.actions.len();
                        }

                        bs.push(block);

                        if c >= actions_per_sync as usize && !bs.is_empty() {
                            tx_blocks.send(BlockMessage::Chunk(bs)).await.unwrap();
                            bs = vec![];
                            c = 0;
                        }
                    }
                    else {
                        info!("no more blocks to process");
                        if !bs.is_empty() {
                            tx_blocks.send(BlockMessage::Chunk(bs)).await.unwrap();
                        }
                        break;
                    }
                }
            }
        }

        info!("warp_sync completed");
        Ok::<_, SyncError>(())
    });

    while let Some(bm) = rx_blocks.recv().await {
        match bm {
            BlockMessage::Chunk(bs) => {
                info!("Processing {} blocks", bs.len());
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
            }
            BlockMessage::Cancel | BlockMessage::StallAbort => {
                info!("Cancelling...");
                break;
            }
            BlockMessage::Reorg(accounts, height) => {
                let _ = tx_decrypted
                    .send(WarpSyncMessage::Rewind(accounts, height))
                    .await;
                break;
            }
            BlockMessage::SaveHeader(bh) => {
                let _ = tx_decrypted.send(WarpSyncMessage::BlockHeader(bh)).await;
            }
        }
    }

    Ok(())
}
