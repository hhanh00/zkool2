use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem::swap;

use zcash_trees::network::Network;
use anyhow::{Context as _, Result};
use bincode::config::legacy;
use futures::TryStreamExt;
use rayon::prelude::*;
use sqlx::{Row, SqliteConnection};
use tokio::sync::mpsc::Sender;
use tracing::{enabled, debug};

use ::orchard::issuance::auth::{IssueValidatingKey, ZSASchnorr};
use ::orchard::note::{AssetBase, AssetId};
use crate::lwd::{CompactBlock, CompactIssueNote, CompactTx};
use crate::warp::{Edge, Hasher, Witness, MERKLE_DEPTH};
use crate::Hash32;
use zcash_trees::types::{Note, Transaction, WarpSyncMessage, UTXO};

pub mod ironwood;
pub mod orchard;
pub mod sapling;

pub trait ShieldedProtocol {
    type Hasher: Hasher;
    type IVK: Sync;
    type NK: Sync;
    type Note: Sync + Send;
    type Spend;
    type Output: Sync;

    /// Issuance key type. Set to `()` for all protocols — issuance note
    /// synthesis has been removed in favor of trial decryption via actions.
    type IssueAuth: Sync;

    fn extract_ivk(
        connection: &mut SqliteConnection,
        account: u32,
        scope: u8,
    ) -> impl std::future::Future<Output = Result<Option<(Self::IVK, Self::NK)>>>;
    fn extract_inputs(tx: &CompactTx) -> &Vec<Self::Spend>;
    fn extract_outputs(tx: &CompactTx) -> &Vec<Self::Output>;

    fn extract_nf(i: &Self::Spend) -> Hash32;
    fn extract_cmx(o: &Self::Output) -> Hash32;

    #[allow(clippy::too_many_arguments)]
    fn try_decrypt(
        network: &Network,
        account: u32,
        scope: u8,
        ivk: &Self::IVK,
        height: u32,
        ivtx: u32,
        vout: u32,
        output: &Self::Output,
    ) -> Result<Option<(Self::Note, Note)>>;

    fn derive_nf(nk: &Self::NK, position: u32, note: &mut Self::Note) -> Result<Hash32>;

    /// Process a plaintext issuance note. No trial decryption — the note fields
    /// are unencrypted. Checks if `recipient` matches `ivk`, constructs the
    /// protocol note, computes its cmx, and returns the result.
    #[allow(clippy::too_many_arguments)]
    fn try_decrypt_issuance(
        _network: &Network,
        _account: u32,
        _scope: u8,
        _ivk: &Self::IVK,
        _height: u32,
        _ivtx: u32,
        _vout: u32,
        _issue_note: &CompactIssueNote,
        _asset_base: &AssetBase,
    ) -> Result<Option<(Self::Note, Note)>> {
        Ok(None) // default: no issuance support
    }

    /// Compute the note commitment (cmx) for a plaintext issuance note.
    /// This is independent of wallet keys — used for tree building for all
    /// issuance notes, including ones we don't own.
    /// Returns `Ok(None)` for protocols that don't support issuance;
    /// `Ok(Some(cmx))` on success; `Err` only on malformed data.
    fn compute_issuance_cmx(
        _issue_note: &CompactIssueNote,
        _asset_base: &AssetBase,
    ) -> Result<Option<Hash32>> {
        Ok(None) // default: issuance not supported
    }
}

#[derive(Debug)]
pub struct Synchronizer<P: ShieldedProtocol> {
    pub hasher: P::Hasher,
    pub network: Network,
    pub pool: u8,
    pub keys: Vec<(u32, u8, P::IVK, P::NK)>,
    pub position: u32,
    pub utxos: HashMap<Vec<u8>, UTXO>,
    pub tree_state: Edge,
    pub tx_decrypted: Sender<WarpSyncMessage>,
    pub _data: PhantomData<P>,
}

impl<P: ShieldedProtocol> Synchronizer<P> {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        network: Network,
        connection: &mut SqliteConnection,
        pool: u8,
        height: u32,
        accounts: &[(u32, bool)],
        tx_decrypted: Sender<WarpSyncMessage>,
        position: u32,
        tree_state: Edge,
    ) -> Result<Self> {
        let mut keys = vec![];
        for (id, use_internal) in accounts.iter() {
            if let Some((ivk, nk)) = P::extract_ivk(&mut *connection, *id, 0).await? {
                keys.push((*id, 0u8, ivk, nk));
            }
            if *use_internal {
                if let Some((ivk, nk)) = P::extract_ivk(&mut *connection, *id, 1).await? {
                    keys.push((*id, 1u8, ivk, nk));
                }
            }
        }

        let mut utxos: HashMap<Vec<u8>, UTXO> = HashMap::new();

        for (account, _, _, _) in keys.iter() {
            debug!(
                "fetch UTXOs - account: {}, pool: {}, height: {}",
                account, pool, height
            );
            let mut nfs = sqlx::query(
                r"
            WITH unspent AS (SELECT a.*
                FROM notes a
                LEFT JOIN spends b ON a.id_note = b.id_note
                WHERE b.id_note IS NULL)
            SELECT u.id_note, u.account, position, nullifier, value, cmx, witness FROM unspent u
            JOIN witnesses w
                ON u.id_note = w.note
                WHERE pool = ? AND u.account = ? AND w.height = ?",
            )
            .bind(pool)
            .bind(account)
            .bind(height - 1)
            .fetch(&mut *connection);
            while let Some(row) = nfs.try_next().await? {
                let id_note = row.get::<u32, _>(0);
                let account = row.get::<u32, _>(1);
                let position = row.get::<u32, _>(2);
                let nullifier = row.get::<Vec<u8>, _>(3);
                let value = row.get::<i64, _>(4) as u64;
                let cmx = row.get::<Vec<u8>, _>(5);
                let witness = row.get::<Vec<u8>, _>(6);
                let (witness, _) = bincode::decode_from_slice(&witness, legacy()).unwrap();
                let utxo = UTXO {
                    id: id_note,
                    pool,
                    account,
                    nullifier,
                    position,
                    value,
                    cmx,
                    witness,
                    ..UTXO::default()
                };

                let mut key = account.to_be_bytes().to_vec();
                key.extend_from_slice(&utxo.nullifier);
                utxos.insert(key, utxo);
            }
        }

        Ok(Self {
            hasher: P::Hasher::default(),
            network,
            pool,
            keys,
            position,
            utxos,
            tree_state,
            tx_decrypted,
            _data: Default::default(),
        })
    }

    pub fn has_no_keys(&self) -> bool {
        self.keys.is_empty()
    }

    pub async fn add(&mut self, blocks: &[CompactBlock]) -> Result<()> {
        if blocks.is_empty() {
            return Ok(());
        }

        let network = self.network;
        let outputs = blocks.into_par_iter().flat_map_iter(|b| {
            b.vtx.iter().enumerate().flat_map(move |(ivtx, vtx)| {
                P::extract_outputs(vtx)
                    .iter()
                    .enumerate()
                    .map(move |(vout, o)| (b.height as u32, ivtx, vout, o))
            })
        });

        let mut notes: Vec<(<P as ShieldedProtocol>::Note, Note, &<P as ShieldedProtocol>::NK)> = outputs
            .flat_map_iter(|(height, ivtx, vout, o)| {
                self.keys.iter().flat_map(move |(account, scope, ivk, nk)| {
                    P::try_decrypt(
                        &network,
                        *account,
                        *scope,
                        ivk,
                        height,
                        ivtx as u32,
                        vout as u32,
                        o,
                    )
                    .unwrap_or_else(|e| {
                        tracing::warn!("decrypt error: {e}");
                        None
                    })
                    .map(|(n, dbn)| (n, dbn, nk))
                })
            })
            .collect::<Vec<_>>();
        debug!("Action notes #{}", notes.len());

        // Process issuance notes from vtx.issuances — plaintext, no trial
        // decryption needed.  Per tx we track the cmxs (for tree building)
        // and the total count (for position tracking).  Only the Orchard
        // protocol supports ZSA issuance (gated by supports_issuance()).
        let mut issuance_cmxs: Vec<(u32, u32, Vec<[u8; 32]>)> = Vec::new();
        for cb in blocks.iter() {
            for (ivtx, tx) in cb.vtx.iter().enumerate() {
                let height = cb.height as u32;
                let actions_len = P::extract_outputs(tx).len() as u32;
                let mut tx_issuance_cmxs: Vec<[u8; 32]> = Vec::new();
                let mut note_vout = actions_len;

                for iss in &tx.issuances {
                    let desc_hash: [u8; 32] = iss.asset_desc_hash.as_slice()
                        .try_into()
                        .map_err(|_| anyhow::anyhow!("Invalid asset_desc_hash length"))?;
                    let ik = IssueValidatingKey::<ZSASchnorr>::decode(&iss.ik)
                        .map_err(|e| anyhow::anyhow!("Invalid issuer key: {e}"))?;
                    let asset_id = AssetId::new_v0(&ik, &desc_hash);
                    let asset_base = AssetBase::custom(&asset_id);

                    for note in &iss.notes {
                        // Compute cmx for tree building. Returns None for
                        // protocols that don't support issuance (Sapling, Ironwood).
                        if let Some(cmx) = P::compute_issuance_cmx(note, &asset_base)? {
                            debug!(
                                "Issuance cmx: height={} ivtx={} vout={} cmx={}",
                                height,
                                ivtx,
                                note_vout,
                                hex::encode(cmx)
                            );
                            tx_issuance_cmxs.push(cmx);
                        }

                        // Check if this note belongs to any of our wallet keys.
                        for (account, scope, ivk, nk) in &self.keys {
                            if let Some((n, dbn)) = P::try_decrypt_issuance(
                                &network,
                                *account,
                                *scope,
                                ivk,
                                height,
                                ivtx as u32,
                                note_vout,
                                note,
                                &asset_base,
                            )
                            .unwrap_or_else(|e| {
                                tracing::warn!("issuance decrypt error: {e}");
                                None
                            }) {
                                notes.push((n, dbn, nk));
                            }
                        }
                        note_vout += 1;
                    }
                }

                if !tx_issuance_cmxs.is_empty() {
                    issuance_cmxs.push((height, ivtx as u32, tx_issuance_cmxs));
                }
            }
        }
        debug!("Notes total (actions + issuances) #{}", notes.len());

        // Build a lookup of per-tx issuance note counts for position tracking.
        let mut issuance_count: std::collections::HashMap<(u32, u32), u32> =
            std::collections::HashMap::new();
        for (height, ivtx, cmxs) in &issuance_cmxs {
            issuance_count.insert((*height, *ivtx), cmxs.len() as u32);
        }

        // Sort by (height, ivtx, vout) so issuance notes are interleaved
        // with action notes within the same transaction. Without this sort,
        // issuance notes appended after all action notes would never be
        // reached by the block-iteration note_iterator below.
        notes.sort_by(|(_, a, _), (_, b, _)| {
            a.height
                .cmp(&b.height)
                .then_with(|| a.ivtx.cmp(&b.ivtx))
                .then_with(|| a.vout.cmp(&b.vout))
        });

        let mut note_iterator = notes.iter_mut();
        let mut note = note_iterator.next();

        let mut position = self.position;
        let mut height = 0;
        for cb in blocks.iter() {
            height = cb.height as u32;
            for (ivtx, tx) in cb.vtx.iter().enumerate() {
                loop {
                    match note {
                        Some((n, dbn, nk))
                            if dbn.height == cb.height as u32 && dbn.ivtx == ivtx as u32 =>
                        {
                            dbn.position = position + dbn.vout;
                            let nf = P::derive_nf(nk, dbn.position, n)?;
                            dbn.nf = nf.to_vec();

                            let transaction = Transaction {
                                account: dbn.account,
                                height: cb.height as u32,
                                time: cb.time,
                                txid: (*tx.hash).into(),
                                ..Transaction::default()
                            };
                            self.tx_decrypted
                                .send(WarpSyncMessage::Transaction(transaction))
                                .await
                                .context("sending transaction")?;

                            dbn.txid = tx.hash.clone();
                            self.tx_decrypted
                                .send(WarpSyncMessage::Note(dbn.clone()))
                                .await
                                .context("sending note")?;
                            note = note_iterator.next();
                        }
                        _ => break,
                    }
                }
                let extra = issuance_count
                    .get(&(cb.height as u32, ivtx as u32))
                    .copied()
                    .unwrap_or(0);
                position += P::extract_outputs(tx).len() as u32 + extra;
            }
        }

        let mut new_utxos = notes
            .into_iter()
            .map(|(_, dbn, _)| UTXO {
                id: dbn.id,
                account: dbn.account,
                pool: self.pool,
                position: dbn.position,
                nullifier: dbn.nf.to_vec(),
                value: dbn.value,
                cmx: dbn.cmx.clone(),
                witness: Witness::default(),
                ..UTXO::default()
            })
            .collect::<Vec<_>>();

        let mut cmxs = vec![];
        let mut count_cmxs = 0;

        debug!("WS starting position {}-{}", self.position, position);
        for depth in 0..MERKLE_DEPTH as usize {
            let mut position = self.position >> depth;
            if position % 2 == 1 {
                cmxs.insert(0, Some(self.tree_state.0[depth].unwrap()));
                position -= 1;
            }

            if depth == 0 {
                // Build lookup for issuance cmxs per (height, ivtx)
                let issuance_cmx_map: std::collections::HashMap<(u32, u32), &[[u8; 32]]> =
                    issuance_cmxs.iter().map(|(h, i, c)| ((*h, *i), c.as_slice())).collect();

                for cb in blocks.iter() {
                    for (ivtx, vtx) in cb.vtx.iter().enumerate() {
                        for co in P::extract_outputs(vtx).iter() {
                            let cmx = P::extract_cmx(co);
                            cmxs.push(Some(cmx));
                        }
                        // Append issuance note cmxs after actions within the same tx
                        if let Some(iss_cmxs) =
                            issuance_cmx_map.get(&(cb.height as u32, ivtx as u32))
                        {
                            for cmx in *iss_cmxs {
                                cmxs.push(Some(*cmx));
                            }
                            count_cmxs += iss_cmxs.len();
                        }
                        count_cmxs += P::extract_outputs(vtx).len();
                    }
                }
            }

            for n in new_utxos.iter_mut() {
                let npos = n.position >> depth;
                let nidx = (npos - position) as usize;

                if depth == 0 {
                    n.witness.position = npos;
                    n.witness.value = cmxs[nidx].unwrap();
                }

                if nidx.is_multiple_of(2) {
                    if nidx + 1 < cmxs.len() {
                        assert!(
                            cmxs[nidx + 1].is_some(),
                            "{} {} {}",
                            depth,
                            n.position,
                            nidx
                        );
                        n.witness.ommers.0[depth] = cmxs[nidx + 1];
                    } else {
                        n.witness.ommers.0[depth] = None;
                    }
                } else {
                    assert!(
                        cmxs[nidx - 1].is_some(),
                        "{} {} {}",
                        depth,
                        n.position,
                        nidx
                    );
                    n.witness.ommers.0[depth] = cmxs[nidx - 1];
                }
            }

            let len = cmxs.len();
            if len >= 2 {
                for n in self.utxos.values_mut() {
                    if n.witness.ommers.0[depth].is_none() {
                        assert!(cmxs[1].is_some());
                        n.witness.ommers.0[depth] = cmxs[1];
                    }
                }
            }

            if len % 2 == 1 {
                self.tree_state.0[depth] = cmxs[len - 1];
            } else {
                self.tree_state.0[depth] = None;
            }

            let pairs = len / 2;
            let mut cmxs2 = self.hasher.parallel_combine_opt(depth as u8, &cmxs, pairs);
            swap(&mut cmxs, &mut cmxs2);
        }

        tracing::debug!("Old notes #{}", self.utxos.len());
        tracing::debug!("New notes #{}", new_utxos.len());
        for utxo in new_utxos.into_iter() {
            let mut key = utxo.account.to_be_bytes().to_vec();
            key.extend_from_slice(&utxo.nullifier);
            self.utxos.insert(key, utxo);
        }
        let auth_path = self.tree_state.to_auth_path(&self.hasher);
        let mut root: Option<[u8; 32]> = None;
        for utxo in self.utxos.values_mut() {
            if enabled!(target: "warp", tracing::Level::DEBUG) {
                let w = &mut utxo.witness;
                let anchor = w.root(&auth_path.0, &self.hasher);
                w.anchor = anchor;
                if let Some(root) = root {
                    if root != anchor {
                        tracing::error!("Anchor mismatch for UTXO {utxo:?}");
                    }
                } else {
                    root = Some(anchor);
                }
            }
            self.tx_decrypted
                .send(WarpSyncMessage::Witness(
                    utxo.account,
                    height,
                    utxo.cmx.clone(),
                    utxo.witness.clone(),
                ))
                .await
                .context("sending witness")?;
        }
        self.position += count_cmxs as u32;
        let accounts = self
            .keys
            .iter()
            .map(|(account, _, _, _)| *account)
            .collect::<Vec<_>>();

        for cb in blocks.iter() {
            for vtx in cb.vtx.iter() {
                for sp in P::extract_inputs(vtx).iter() {
                    let nf = P::extract_nf(sp);
                    let nf = nf.as_slice();
                    for account in accounts.iter() {
                        let mut key = account.to_be_bytes().to_vec();
                        key.extend_from_slice(nf);

                        if let Some(mut utxo) = self.utxos.remove(&key) {
                            utxo.txid = vtx.hash.clone();
                            let tx = Transaction {
                                account: utxo.account,
                                height: cb.height as u32,
                                time: cb.time,
                                txid: (*vtx.hash).into(),
                                ..Transaction::default()
                            };
                            self.tx_decrypted
                                .send(WarpSyncMessage::Transaction(tx))
                                .await
                                .context("sending transaction")?;
                            self.tx_decrypted
                                .send(WarpSyncMessage::Spend(utxo))
                                .await
                                .context("sending spend")?;
                        }
                    }
                }
            }
        }
        self.tx_decrypted
            .send(WarpSyncMessage::Checkpoint(accounts, self.pool, height))
            .await
            .context("sending checkpoint")?;

        Ok(())
    }
}
