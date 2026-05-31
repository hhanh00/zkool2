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

                // Process ZSA issuance data BEFORE shielded decryption so that
                // Issuance messages are queued ahead of Note messages. This
                // ensures the DB writer inserts assets before notes, allowing
                // the Note handler to JOIN on the assets table.
                let total_issuances: usize = bs.iter().map(|cb| cb.vtx.iter().map(|vtx| vtx.issuances.len()).sum::<usize>()).sum();
                if total_issuances > 0 {
                    info!("Found {} issuances in {} blocks", total_issuances, bs.len());
                }
                // Resolve which accounts own which issuance keys.
                let mut ik_owners: HashMap<Vec<u8>, u32> = HashMap::new();
                if total_issuances > 0 {
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
                }

                // Collect issuance note cmxs to add to the Orchard commitment
                // tree after the regular Orchard actions (same tx ordering).
                let mut issuance_cmxs: HashMap<(u32, usize), Vec<crate::Hash32>> =
                    HashMap::new();
                // Track orchard tree position for issuance notes (they follow
                // the Orchard bundle outputs in the same transaction).
                let mut orchard_pos = orch_dec.position;

                for cb in &bs {
                    for (ivtx, vtx) in cb.vtx.iter().enumerate() {
                        let mut tx_has_notes_for_us = false;

                        for iss in &vtx.issuances {
                            let desc_hash: [u8; 32] =
                                iss.asset_desc_hash.as_slice().try_into().unwrap();
                            let ik = IssueValidatingKey::<ZSASchnorr>::decode(&iss.ik)
                                .expect("invalid issuer key in issuance");
                            let asset_id = AssetId::new_v0(&ik, &desc_hash);
                            let asset_base = AssetBase::custom(&asset_id);

                            // Send Issuance message for asset storage
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

                            // Synthesize unencrypted issuance notes for accounts
                            // that own the matching issue validating key.
                            // The FVK for nullifier derivation is already
                            // loaded in orch_dec.keys during sync init.
                            if let Some(&account) = ik_owners.get(&iss.ik) {
                                let fvk = orch_dec
                                    .keys
                                    .iter()
                                    .find(|(a, scope, _, _)| *a == account && *scope == 0)
                                    .map(|(_, _, _, fvk)| fvk)
                                    .context("no orchard FVK for issuance account")?;
                                if !tx_has_notes_for_us {
                                    // Send Transaction message (needed for Note INSERT JOIN)
                                    tx_decrypted
                                        .send(WarpSyncMessage::Transaction(
                                            zcash_trees::types::Transaction {
                                                account,
                                                height: cb.height as u32,
                                                time: cb.time,
                                                txid: vtx.hash.clone(),
                                                ..Default::default()
                                            },
                                        ))
                                        .await
                                        .context("sending issuance transaction")?;
                                    tx_has_notes_for_us = true;
                                }

                                // Position starts after all Orchard outputs in this tx
                                let mut note_offset =
                                    orchard_pos + vtx.actions.len() as u32;
                                // Add notes from prior issuance actions in same tx
                                for prev_iss in vtx.issuances.iter() {
                                    if std::ptr::eq(prev_iss, iss) {
                                        break;
                                    }
                                    note_offset += prev_iss.notes.len() as u32;
                                }

                                for (note_idx, note_data) in iss.notes.iter().enumerate()
                                {
                                    let recipient_bytes: [u8; 43] =
                                        note_data.recipient.as_slice().try_into()
                                            .context("invalid recipient length")?;
                                    let recipient = Address::from_raw_address_bytes(
                                        &recipient_bytes,
                                    )
                                    .unwrap();
                                    let rho_bytes: [u8; 32] =
                                        note_data.rho.as_slice().try_into().unwrap();
                                    let rho =
                                        Rho::from_bytes(&rho_bytes).unwrap();
                                    let rseed_bytes: [u8; 32] =
                                        note_data.rseed.as_slice().try_into().unwrap();
                                    let rseed = RandomSeed::from_bytes(rseed_bytes, &rho)
                                        .unwrap();

                                    let note = orchard::note::Note::from_parts(
                                        recipient,
                                        NoteValue::from_raw(note_data.value),
                                        asset_base,
                                        rho,
                                        rseed,
                                    )
                                    .unwrap();

                                    let cmx =
                                        ExtractedNoteCommitment::from(note.commitment());
                                    let nf = note.nullifier(fvk);
                                    let position = note_offset + note_idx as u32;

                                    // Queue cmx for Merkle tree insertion after
                                    // Orchard actions in this transaction
                                    issuance_cmxs
                                        .entry((cb.height as u32, ivtx))
                                        .or_default()
                                        .push(cmx.to_bytes());

                                    let dbn = zcash_trees::types::Note {
                                        account,
                                        scope: 0,
                                        height: cb.height as u32,
                                        pool: 2,
                                        value: note_data.value,
                                        cmx: cmx.to_bytes().to_vec(),
                                        asset_base: asset_base.to_bytes().to_vec(),
                                        rho: note_data.rho.clone(),
                                        rcm: note.rseed().as_bytes().to_vec(),
                                        diversifier: recipient
                                            .diversifier()
                                            .as_array()
                                            .to_vec(),
                                        nf: nf.to_bytes().to_vec(),
                                        position,
                                        ivtx: ivtx as u32,
                                        vout: note_idx as u32,
                                        txid: vtx.hash.clone(),
                                        ..Default::default()
                                    };

                                    info!(
                                        "Issuance note: account={} height={} value={} position={} nf={} asset={:?}",
                                        account,
                                        cb.height,
                                        note_data.value,
                                        position,
                                        hex::encode(&nf.to_bytes()[..8]),
                                        asset_base,
                                    );

                                    tx_decrypted
                                        .send(WarpSyncMessage::Note(dbn))
                                        .await
                                        .context("sending issuance note")?;
                                }
                            }
                        }

                        orchard_pos += vtx.actions.len() as u32;
                    }
                }

                let empty_cmxs: HashMap<(u32, usize), Vec<crate::Hash32>> =
                    HashMap::new();
                sap_dec.add(&bs, &empty_cmxs).await?;
                orch_dec.add(&bs, &issuance_cmxs).await?;

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
