use std::{collections::HashSet, time::Duration};

use anyhow::{Context as _, Result};
use shielded::Synchronizer;
use sqlx::SqliteConnection;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tracing::debug;
use zcash_protocol::consensus::{NetworkUpgrade, Parameters};
use zcash_trees::network::Network;

use orchard::issuance::auth::IssueValidatingKey;
use orchard::issuance::auth::ZSASchnorr;
use orchard::note::AssetBase;
use orchard::note::AssetId;

use crate::{
    lwd::CompactBlock,
    warp::hasher::{OrchardHasher, SaplingHasher},
};
use zcash_trees::types::{BlockHeader, Issuance, WarpSyncMessage};

use super::legacy::CommitmentTreeFrontier;

pub use zcash_trees::types::SyncError;

mod shielded;

pub type SaplingSync = Synchronizer<shielded::sapling::SaplingProtocol>;
pub type OrchardSync = Synchronizer<shielded::orchard::OrchardProtocol>;
pub type IronwoodSync = Synchronizer<shielded::ironwood::IronwoodProtocol>;

pub enum BlockMessage {
    Chunk(Vec<CompactBlock>),
    SaveHeader(BlockHeader),
    Cancel,
    /// No block arrived for a whole stall interval; `reached` is the last
    /// height seen.
    StallAbort { reached: u32 },
    /// The stream ended before `end`: `reached` is the last height seen.
    Interrupted { reached: u32 },
    /// The stream failed after `reached`.
    Failed { reached: u32, error: anyhow::Error },
    Reorg(Vec<u32>, u32),
}

/// How a scan ended, for the caller to decide what comes next. A scan that
/// did not reach its end is an error, not a variant: the run must not look
/// finished (#1474).
#[derive(Debug, PartialEq, Eq)]
pub enum ScanEnd {
    /// Every block up to `end` was processed.
    Completed,
    /// The user cancelled; whatever was committed stays.
    Cancelled,
    /// A block did not connect to the previous one. The rewind was sent to
    /// the writer; the caller restarts the scan from where the database
    /// stands after it. `detected_at` is the height that did not connect.
    Reorg { detected_at: u32 },
}

/// How long without a block before the connection is given up.
const STALL: Duration = Duration::from_secs(60);

#[allow(clippy::too_many_arguments)]
pub async fn warp_sync(
    network: &Network,
    connection: &mut SqliteConnection,
    start_height: u32,
    end_height: u32,
    accounts: &[(u32, bool)],
    mut blocks: ReceiverStream<anyhow::Result<CompactBlock>>,
    mut heights_without_time: HashSet<u32>,
    actions_per_sync: u32,
    sapling_state: &CommitmentTreeFrontier,
    orchard_state: &CommitmentTreeFrontier,
    ironwood_state: &CommitmentTreeFrontier,
    tx_decrypted: Sender<WarpSyncMessage>,
    cancel: CancellationToken,
) -> Result<ScanEnd, SyncError> {
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

    let ironwood_active = network.activation_height(NetworkUpgrade::Nu6_3).is_some();
    let ironwood_hasher = OrchardHasher::default();
    let mut ironwood_dec = if ironwood_active {
        Some(
            IronwoodSync::new(
                *network,
                &mut *connection,
                3,
                start_height,
                accounts,
                tx_decrypted.clone(),
                ironwood_state.size() as u32,
                ironwood_state.to_edge(&ironwood_hasher),
            )
            .await?,
        )
    } else {
        None
    };

    let ironwood_has_keys = ironwood_dec.as_ref().map_or(false, |d| !d.has_no_keys());
    if sap_dec.has_no_keys() && orch_dec.has_no_keys() && !ironwood_has_keys {
        debug!("No keys to sync");
        return Ok(ScanEnd::Completed);
    }

    // Check every account: accounts at the same height can still be on different chains.
    let mut frontier_hashes = Vec::new();
    for (account, _) in accounts {
        let hash = crate::sync::chain_log::header_hash(&mut *connection, *account, start_height - 1)
            .await
            .context("reading account frontier header")?;
        if let Some(hash) = hash {
            frontier_hashes.push(hash);
        }
    }
    let mut prev_hash: Option<Vec<u8>> = None;
    let account_ids = accounts.iter().map(|(id, _)| *id).collect::<Vec<_>>();
    let (tx_blocks, mut rx_blocks) = tokio::sync::mpsc::channel::<BlockMessage>(2);
    tokio::spawn(async move {
        let mut bs = vec![];

        let mut interval = tokio::time::interval(STALL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        // The last height seen; `start_height - 1` until the first block.
        let mut current_height = start_height.saturating_sub(1);
        let mut prev_current_height = u32::MAX;

        let mut c = 0;
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    debug!("Syncing at height {}", current_height);
                    if prev_current_height == current_height {
                        debug!("Connection stalled. Aborting...");
                        let _ = tx_blocks.send(BlockMessage::StallAbort { reached: current_height }).await;
                        break;
                    }
                    prev_current_height = current_height;
                }
                _ = cancel.cancelled() => {
                    debug!("Sync cancelled");
                    let _ = tx_blocks.send(BlockMessage::Cancel).await;
                    break;
                }
                m = blocks.next() => {
                    let block = match m {
                        Some(Ok(block)) => block,
                        Some(Err(error)) => {
                            // Keep what was scanned, then report the break.
                            if !bs.is_empty() {
                                let _ = tx_blocks.send(BlockMessage::Chunk(bs)).await;
                            }
                            let _ = tx_blocks.send(BlockMessage::Failed { reached: current_height, error }).await;
                            break;
                        }
                        None => {
                            if !bs.is_empty() {
                                let _ = tx_blocks.send(BlockMessage::Chunk(bs)).await;
                            }
                            if current_height < end_height {
                                debug!("block stream ended at {} of {}", current_height, end_height);
                                let _ = tx_blocks.send(BlockMessage::Interrupted { reached: current_height }).await;
                            } else {
                                debug!("no more blocks to process");
                            }
                            break;
                        }
                    };
                    let block_prev_hash = block.prev_hash.clone();
                    current_height = block.height as u32;
                    let discontinuity = match &prev_hash {
                        Some(hash) => hash != &block_prev_hash,
                        None => frontier_hashes.iter().any(|hash| hash != &block_prev_hash),
                    };
                    if discontinuity {
                        let _ = tx_blocks.send(BlockMessage::Reorg(account_ids, current_height - 1)).await;
                        debug!("Reorganization detected at block {}", block.height);
                        break;
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

                    // Every field the scanners process counts toward the
                    // batch, or a range of Ironwood-only blocks would be
                    // held until the stream ends (V12 F-183).
                    for vtx in block.vtx.iter() {
                        c += vtx.spends.len();
                        c += vtx.outputs.len();
                        c += vtx.actions.len();
                        c += vtx.ironwood_actions.len();
                        c += vtx.issuances.len();
                    }

                    bs.push(block);

                    if c >= actions_per_sync as usize && !bs.is_empty() {
                        if tx_blocks.send(BlockMessage::Chunk(bs)).await.is_err() {
                            break; // the writer is gone
                        }
                        bs = vec![];
                        c = 0;
                    }
                }
            }
        }

        debug!("warp_sync block reader done");
    });

    let mut end = ScanEnd::Completed;
    while let Some(bm) = rx_blocks.recv().await {
        match bm {
            BlockMessage::Chunk(bs) => {
                debug!("Processing {} blocks", bs.len());

                // Send Issuance messages for asset storage before any Note
                // messages (same transaction ordering guarantee).
                for cb in &bs {
                    for vtx in &cb.vtx {
                        for iss in &vtx.issuances {
                            let desc_hash: [u8; 32] =
                                iss.asset_desc_hash.as_slice().try_into().unwrap();
                            let ik = IssueValidatingKey::<ZSASchnorr>::decode(&iss.ik)
                                .expect("invalid issuer key in issuance");
                            let asset_id = AssetId::new_v0(&ik, &desc_hash);
                            let asset_base = AssetBase::custom(&asset_id);
                            tx_decrypted
                                .send(WarpSyncMessage::Issuance(Issuance {
                                    asset_desc_hash: iss.asset_desc_hash.clone(),
                                    ik: iss.ik.clone(),
                                    asset_base: asset_base.to_bytes().to_vec(),
                                    finalized: iss.finalize,
                                    height: cb.height as u32,
                                }))
                                .await
                                .context("sending issuance")?;
                        }
                    }
                }

                sap_dec.add(&bs).await?;
                orch_dec.add(&bs).await?;
                if let Some(ref mut ironwood_dec) = ironwood_dec {
                    ironwood_dec.add(&bs).await?;
                }

                let lcb = bs.last().unwrap();
                let bh = BlockHeader {
                    height: lcb.height as u32,
                    hash: lcb.hash.clone(),
                    time: lcb.time,
                };
                let _ = tx_decrypted.send(WarpSyncMessage::BlockHeader(bh)).await;
                let _ = tx_decrypted.send(WarpSyncMessage::Commit).await;
            }
            BlockMessage::Cancel => {
                debug!("Cancelling...");
                end = ScanEnd::Cancelled;
                break;
            }
            BlockMessage::StallAbort { reached } => {
                return Err(SyncError::Other(anyhow::anyhow!(
                    "no block received for {} s after height {reached}, {end_height} asked",
                    STALL.as_secs()
                )));
            }
            BlockMessage::Interrupted { reached } => {
                return Err(SyncError::Other(anyhow::anyhow!(
                    "the block stream ended at height {reached}, {end_height} asked"
                )));
            }
            BlockMessage::Failed { reached, error } => {
                return Err(SyncError::Other(error.context(format!(
                    "the block stream failed after height {reached}, {end_height} asked"
                ))));
            }
            BlockMessage::Reorg(accounts, height) => {
                let _ = tx_decrypted
                    .send(WarpSyncMessage::Rewind(accounts, height))
                    .await;
                end = ScanEnd::Reorg { detected_at: height + 1 };
                break;
            }
            BlockMessage::SaveHeader(bh) => {
                let _ = tx_decrypted.send(WarpSyncMessage::BlockHeader(bh)).await;
            }
        }
    }

    Ok(end)
}
