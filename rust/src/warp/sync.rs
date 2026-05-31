use std::{collections::HashSet, time::Duration};

use anyhow::{Context as _, Result};
use bip39::Mnemonic;
use shielded::Synchronizer;
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc::Sender};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tracing::info;
use zcash_protocol::consensus::Parameters;
use zcash_trees::network::Network;

use orchard::{
    note::{AssetBase, AssetId, ExtractedNoteCommitment, RandomSeed, Rho},
    issuance::auth::{IssueAuthKey, IssueValidatingKey, ZSASchnorr},
    value::NoteValue,
    Address,
};

use crate::{
    lwd::CompactBlock,
    warp::hasher::{OrchardHasher, SaplingHasher},
};
use zcash_trees::types::{BlockHeader, Issuance, WarpSyncMessage};

use super::legacy::CommitmentTreeFrontier;

pub use zcash_trees::types::SyncError;

pub mod block;
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

/// Preprocess CompactBlocks into SyncBlocks, merging issuance notes into the
/// unified `orchard_outputs` list per transaction. Resolves ik→account ownership
/// so `try_decrypt` can match issuance notes without re-deriving keys.
async fn preprocess(
    blocks: &[CompactBlock],
    accounts: &[(u32, bool)],
    connection: &mut SqliteConnection,
    network: &Network,
) -> Result<Vec<block::SyncBlock>> {
    use orchard::note::{AssetBase as OrchardAssetBase, AssetId as OrchardAssetId};

    // Resolve which accounts own which issuance keys
    let mut ik_owners: HashMap<Vec<u8>, u32> = HashMap::new();
    let coin_type = match network.network_type() {
        zcash_protocol::consensus::NetworkType::Main => 133u32,
        _ => 1u32,
    };
    for (account, _) in accounts.iter() {
        if let Ok(Some(seed_info)) =
            crate::account::get_account_seed(&mut *connection, *account).await
        {
            if let Ok(mnemonic) = Mnemonic::parse(seed_info.mnemonic) {
                let seed = mnemonic.to_seed(&seed_info.phrase);
                if let Ok(isk) =
                    IssueAuthKey::<ZSASchnorr>::from_zip32_seed(&seed, coin_type, 0)
                {
                    let my_ik: IssueValidatingKey<ZSASchnorr> =
                        IssueValidatingKey::from(&isk);
                    ik_owners.insert(my_ik.encode(), *account);
                }
            }
        }
    }

    let mut sync_blocks = Vec::with_capacity(blocks.len());
    for cb in blocks {
        let mut sync_vtx = Vec::with_capacity(cb.vtx.len());
        for vtx in &cb.vtx {
            // Copy orchard actions as OrchardOutput::Action
            let mut orchard_outputs: Vec<block::OrchardOutput> = vtx
                .actions
                .iter()
                .cloned()
                .map(block::OrchardOutput::Action)
                .collect();

            // Append issuance notes as OrchardOutput::Issuance
            for iss in &vtx.issuances {
                let desc_hash: [u8; 32] =
                    iss.asset_desc_hash.as_slice().try_into().unwrap();
                let ik = IssueValidatingKey::<ZSASchnorr>::decode(&iss.ik)
                    .expect("invalid issuer key in issuance");
                let oasset_id = OrchardAssetId::new_v0(&ik, &desc_hash);
                let asset_base = OrchardAssetBase::custom(&oasset_id);
                let asset_base_bytes = asset_base.to_bytes().to_vec();
                let ik_owner = ik_owners.get(&iss.ik).copied();

                for note_data in &iss.notes {
                    // Zero-value reference notes (consensus requirement for
                    // first issuance) go into the Merkle tree but are not
                    // merged into the wallet — set owner to None.
                    let owner = if note_data.value == 0 {
                        None
                    } else {
                        ik_owner
                    };
                    // Compute cmx from plaintext note fields
                    let recipient_bytes: [u8; 43] =
                        note_data.recipient.as_slice().try_into().unwrap();
                    let recipient =
                        Address::from_raw_address_bytes(&recipient_bytes).unwrap();
                    let rho_bytes: [u8; 32] =
                        note_data.rho.as_slice().try_into().unwrap();
                    let rho = Rho::from_bytes(&rho_bytes).unwrap();
                    let rseed_bytes: [u8; 32] =
                        note_data.rseed.as_slice().try_into().unwrap();
                    let rseed = RandomSeed::from_bytes(rseed_bytes, &rho).unwrap();
                    let note = orchard::note::Note::from_parts(
                        recipient,
                        NoteValue::from_raw(note_data.value),
                        asset_base,
                        rho,
                        rseed,
                    )
                    .unwrap();
                    let cmx =
                        ExtractedNoteCommitment::from(note.commitment()).to_bytes();

                    orchard_outputs.push(block::OrchardOutput::Issuance {
                        note: note_data.clone(),
                        ik: iss.ik.clone(),
                        asset_desc_hash: iss.asset_desc_hash.clone(),
                        asset_base: asset_base_bytes.clone(),
                        cmx,
                        owner,
                    });
                }
            }

            sync_vtx.push(block::SyncTx {
                hash: vtx.hash.clone(),
                spends: vtx.spends.clone(),
                sapling_outputs: vtx.outputs.clone(),
                orchard_actions: vtx.actions.clone(),
                orchard_outputs,
                issuances: vtx.issuances.clone(),
            });
        }
        sync_blocks.push(block::SyncBlock {
            height: cb.height,
            hash: cb.hash.clone(),
            prev_hash: cb.prev_hash.clone(),
            time: cb.time,
            vtx: sync_vtx,
        });
    }
    Ok(sync_blocks)
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

                // Preprocess: convert CompactBlock → SyncBlock, merging
                // issuance notes into orchard_outputs per transaction.
                let sync_blocks = preprocess(&bs, accounts, &mut *connection, network)
                    .await
                    .context("preprocessing blocks")?;

                // Send Issuance messages for asset storage before any Note
                // messages (same transaction ordering guarantee).
                for cb in &sync_blocks {
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

                sap_dec.add(&sync_blocks).await?;
                orch_dec.add(&sync_blocks).await?;

                let lcb = bs.last().unwrap();
                // Use the last original block for header (sync_blocks is
                // already consumed by add()).
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
