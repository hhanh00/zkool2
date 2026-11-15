use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use anyhow::{Context as _, Result};
use bincode::config::legacy;
use futures::TryStreamExt;
use rayon::prelude::*;
use sqlx::{Row, SqliteConnection};
use tokio::sync::mpsc::Sender;
use tracing::{debug, enabled};
use zcash_trees::network::Network;

use crate::lwd::{CompactBlock, CompactIssueNote, CompactTx};
use crate::warp::{Edge, Hasher, Witness, MERKLE_DEPTH};
use crate::Hash32;
use ::orchard::issuance::auth::{IssueValidatingKey, ZSASchnorr};
use ::orchard::note::{AssetBase, AssetId};
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

    /// The account's keys at `scope`: the incoming key, and the nullifier
    /// key when the account has its full key. An incoming-only account has
    /// no nullifier key; its notes get their commitment as a stand-in.
    fn extract_ivk(
        connection: &mut SqliteConnection,
        account: u32,
        scope: u8,
    ) -> impl std::future::Future<Output = Result<Option<(Self::IVK, Option<Self::NK>)>>>;
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

/// Where one pool's scan spent its time, for benchmarks.
#[derive(Debug, Default, Clone)]
pub struct ScanTiming {
    /// Outputs (or actions) trial-decrypted, each against every key.
    pub outputs: u64,
    /// Trial decryption, on all cores.
    pub decrypt: std::time::Duration,
    /// Hashing the note commitment tree and updating witnesses.
    pub tree: std::time::Duration,
    /// The rest: issuance, and handing notes, witnesses and spends to the
    /// writer, waiting on it when it is behind.
    pub other: std::time::Duration,
    /// Leaves not hashed: their subtree's root was grafted instead.
    pub grafted: u64,
    /// Subtrees hashed whose root the server gave too: the two matched.
    pub verified: u64,
}

/// Leaves of a subtree, as `GetSubtreeRoots` counts them: 2^16.
pub const SUBTREE_DEPTH: usize = 16;
/// Every this many subtrees, one is hashed even when it could be grafted,
/// and its root checked against the server's.
const VERIFY_EVERY: u32 = 16;

/// Roots of completed subtrees of a pool's tree, as the server gives them:
/// `roots[i]` is subtree `first + i`.
#[derive(Debug, Default, Clone)]
pub struct SubtreeRoots {
    pub first: u32,
    pub roots: Vec<Hash32>,
}

impl SubtreeRoots {
    fn get(&self, index: u32) -> Option<&Hash32> {
        self.roots.get(index.checked_sub(self.first)? as usize)
    }
}

#[derive(Debug)]
pub struct Synchronizer<P: ShieldedProtocol> {
    pub hasher: P::Hasher,
    pub network: Network,
    pub pool: u8,
    pub keys: Vec<(u32, u8, P::IVK, Option<P::NK>)>,
    pub position: u32,
    pub utxos: HashMap<Vec<u8>, UTXO>,
    pub tree_state: Edge,
    pub tx_decrypted: Sender<WarpSyncMessage>,
    pub timing: ScanTiming,
    /// Roots of completed subtrees, grafted in place of hashing them.
    pub subtree_roots: SubtreeRoots,
    /// Leaves after `position` not hashed yet (see `Tree::pending`).
    pending: Vec<Hash32>,
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
            timing: ScanTiming::default(),
            subtree_roots: SubtreeRoots::default(),
            pending: vec![],
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
        let started = std::time::Instant::now();

        let network = self.network;
        let outputs = blocks.into_par_iter().flat_map_iter(|b| {
            b.vtx.iter().enumerate().flat_map(move |(ivtx, vtx)| {
                P::extract_outputs(vtx)
                    .iter()
                    .enumerate()
                    .map(move |(vout, o)| (b.height as u32, ivtx, vout, o))
            })
        });

        let mut notes: Vec<(
            <P as ShieldedProtocol>::Note,
            Note,
            &Option<<P as ShieldedProtocol>::NK>,
        )> = outputs
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
        let decrypt = started.elapsed();
        self.timing.outputs += blocks
            .iter()
            .flat_map(|b| b.vtx.iter())
            .map(|tx| P::extract_outputs(tx).len() as u64)
            .sum::<u64>();

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
                    let desc_hash: [u8; 32] = iss
                        .asset_desc_hash
                        .as_slice()
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

        // Leaves in the tree so far, those waiting to be hashed included.
        let mut position = self.position + self.pending.len() as u32;
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
                            dbn.nf = match nk {
                                Some(nk) => P::derive_nf(nk, dbn.position, n)?.to_vec(),
                                // An incoming-only account cannot derive
                                // nullifiers. The note commitment stands in:
                                // unique per note, so rescans deduplicate,
                                // and never equal to a spend on chain, so the
                                // note is never marked spent.
                                None => dbn.cmx.clone(),
                            };

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

        let new_utxos = notes
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

        let tree_started = std::time::Instant::now();
        // The batch's leaves in tree order: each transaction's outputs, then
        // the notes it issued.
        let issuance_cmx_map: HashMap<(u32, u32), &[[u8; 32]]> = issuance_cmxs
            .iter()
            .map(|(h, i, c)| ((*h, *i), c.as_slice()))
            .collect();
        let mut leaves: Vec<Hash32> = vec![];
        for cb in blocks.iter() {
            for (ivtx, vtx) in cb.vtx.iter().enumerate() {
                leaves.extend(P::extract_outputs(vtx).iter().map(P::extract_cmx));
                if let Some(iss_cmxs) = issuance_cmx_map.get(&(cb.height as u32, ivtx as u32)) {
                    leaves.extend_from_slice(iss_cmxs);
                }
            }
        }
        tracing::debug!("Old notes #{}", self.utxos.len());
        tracing::debug!("New notes #{}", new_utxos.len());
        let mut tree = Tree {
            hasher: &self.hasher,
            position: &mut self.position,
            edge: &mut self.tree_state,
            pending: &mut self.pending,
            utxos: &mut self.utxos,
        };
        let (grafted, mut verified) =
            tree.add_leaves(&leaves, new_utxos, &self.subtree_roots, SUBTREE_DEPTH)?;
        if enabled!(target: "warp", tracing::Level::DEBUG) {
            // The anchors checked below need the edge up to date.
            verified += tree.flush(&self.subtree_roots, SUBTREE_DEPTH)?;
        }
        self.timing.grafted += grafted;
        self.timing.verified += verified;
        let tree = tree_started.elapsed();
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

        self.timing.decrypt += decrypt;
        self.timing.tree += tree;
        self.timing.other += started.elapsed().saturating_sub(decrypt + tree);
        Ok(())
    }
}

/// A pool's note commitment tree as the scan keeps it: the leaves hashed so
/// far, the edge (the left nodes still waiting for a right sibling), the
/// leaves after them not hashed yet, and the unspent notes with their
/// witnesses.
struct Tree<'a, H: Hasher> {
    hasher: &'a H,
    /// Leaves hashed; the edge is the tree's as of this one.
    position: &'a mut u32,
    edge: &'a mut Edge,
    /// The leaves after `position`, all in one subtree that is not complete
    /// and holds no note: hashed or grafted once it completes.
    pending: &'a mut Vec<Hash32>,
    utxos: &'a mut HashMap<Vec<u8>, UTXO>,
}

impl<H: Hasher> Tree<'_, H> {
    /// Append a batch's leaves, with the notes found among them.
    ///
    /// A subtree of `2^subtree_depth` leaves is not hashed when it completes,
    /// the server gave its root, and no unspent note is in it: its root is
    /// grafted in its place, and the edge below it, all inside it, is
    /// dropped. No witness needs a node inside such a subtree. Every
    /// `VERIFY_EVERY`th subtree is hashed regardless and its root checked
    /// against the server's.
    ///
    /// Until the last subtree completes, its leaves wait in `pending` while
    /// no note is in it, so that it can still be grafted whole. Only the
    /// edge lags meanwhile: a witness takes nothing from a subtree that is
    /// not complete, and nothing outside the scan reads the edge (spends
    /// build their paths from the server's tree state). A note found in it
    /// has the waiting leaves hashed with the new ones, as if they never
    /// waited. Returns the leaves grafted and the subtree roots checked.
    fn add_leaves(
        &mut self,
        leaves: &[Hash32],
        mut new_utxos: Vec<UTXO>,
        roots: &SubtreeRoots,
        subtree_depth: usize,
    ) -> Result<(u64, u64)> {
        let mut all = std::mem::take(self.pending);
        all.extend_from_slice(leaves);
        let leaves = all;
        let start = u64::from(*self.position);
        let end = start + leaves.len() as u64;
        let size = 1u64 << subtree_depth;
        let noted: HashSet<u64> = self
            .utxos
            .values()
            .chain(new_utxos.iter())
            .map(|utxo| u64::from(utxo.position) >> subtree_depth)
            .collect();
        let graftable = |k: u64| {
            (k + 1) * size <= end
                && !noted.contains(&k)
                && k % u64::from(VERIFY_EVERY) != 0
                && roots.get(k as u32).is_some()
        };
        // The last subtree, not complete, waits while no note is in it.
        let waits = |k: u64| (k + 1) * size > end && !noted.contains(&k);
        new_utxos.sort_by_key(|utxo| utxo.position);
        let mut new_utxos = new_utxos.into_iter().peekable();
        let mut grafted = 0;
        let mut verified = 0;
        let mut at = start;
        while at < end {
            let first = at >> subtree_depth;
            let mut k = first;
            if graftable(k) {
                let mut layer = vec![];
                while k * size < end && graftable(k) {
                    layer.push(Some(*roots.get(k as u32).unwrap()));
                    k += 1;
                }
                grafted += k * size - at;
                // The edge below the subtree level is inside the first
                // subtree, and its root replaces it.
                self.edge.0[..subtree_depth].fill(None);
                *self.position = (first * size) as u32;
                self.append_layer(subtree_depth, layer, vec![], None)?;
                at = k * size;
            } else if waits(k) {
                self.pending.extend_from_slice(&leaves[(at - start) as usize..]);
                at = end;
            } else {
                while k * size < end && !graftable(k) && !waits(k) {
                    k += 1;
                }
                let to = (k * size).min(end);
                let layer = leaves[(at - start) as usize..(to - start) as usize]
                    .iter()
                    .map(|leaf| Some(*leaf))
                    .collect();
                let mut found = vec![];
                while new_utxos.peek().is_some_and(|utxo| u64::from(utxo.position) < to) {
                    found.extend(new_utxos.next());
                }
                verified += self.append_layer(0, layer, found, Some((roots, subtree_depth)))?;
                at = to;
            }
        }
        Ok((grafted, verified))
    }

    /// Hash the leaves waiting, bringing the edge up to date. The subtree
    /// they are in can no longer be grafted. Returns the roots checked.
    fn flush(&mut self, roots: &SubtreeRoots, subtree_depth: usize) -> Result<u64> {
        let pending = std::mem::take(self.pending);
        if pending.is_empty() {
            return Ok(0);
        }
        let layer = pending.into_iter().map(Some).collect();
        self.append_layer(0, layer, vec![], Some((roots, subtree_depth)))
    }

    /// Append `layer`, consecutive nodes at `depth0` starting where the tree
    /// is, and hash them up to the root: the new notes' witnesses, the older
    /// notes' missing ommers and the edge are updated on the way. Subtree
    /// roots met at `check`'s depth are checked against the server's; the
    /// number checked is returned.
    fn append_layer(
        &mut self,
        depth0: usize,
        mut cmxs: Vec<Option<Hash32>>,
        mut new_utxos: Vec<UTXO>,
        check: Option<(&SubtreeRoots, usize)>,
    ) -> Result<u64> {
        let count = cmxs.len() as u32;
        let mut verified = 0;
        for depth in depth0..MERKLE_DEPTH as usize {
            let mut position = *self.position >> depth;
            if position % 2 == 1 {
                cmxs.insert(0, Some(self.edge.0[depth].unwrap()));
                position -= 1;
            }

            if let Some((roots, subtree_depth)) = check {
                if depth == subtree_depth {
                    // A node taken from the edge was checked, or grafted, before.
                    let from_edge = usize::from(*self.position >> depth != position);
                    for (i, node) in cmxs.iter().enumerate().skip(from_edge) {
                        let index = position + i as u32;
                        if let (Some(node), Some(root)) = (node, roots.get(index)) {
                            anyhow::ensure!(
                                node == root,
                                "the server's root of subtree {index} is not the one hashed"
                            );
                            verified += 1;
                        }
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
                self.edge.0[depth] = cmxs[len - 1];
            } else {
                self.edge.0[depth] = None;
            }

            let pairs = len / 2;
            cmxs = self.hasher.parallel_combine_opt(depth as u8, &cmxs, pairs);
        }
        *self.position += count << depth0;
        for utxo in new_utxos {
            let mut key = utxo.account.to_be_bytes().to_vec();
            key.extend_from_slice(&utxo.nullifier);
            self.utxos.insert(key, utxo);
        }
        Ok(verified)
    }
}

#[cfg(test)]
mod graft_tests {
    use super::*;
    use crate::warp::hasher::OrchardHasher;

    /// Subtrees of 8 leaves, so that many fit in a test.
    const DEPTH: usize = 3;

    /// A leaf that is a valid field element.
    fn leaf(i: u32) -> Hash32 {
        let mut leaf = [0u8; 32];
        leaf[..4].copy_from_slice(&i.to_le_bytes());
        leaf[4] = 7;
        leaf
    }

    fn note(position: u32) -> UTXO {
        UTXO { account: 1, position, nullifier: position.to_be_bytes().to_vec(), ..UTXO::default() }
    }

    /// The root of subtree `k`, hashed directly.
    fn subtree_root(hasher: &OrchardHasher, k: u32) -> Hash32 {
        let size = 1 << DEPTH;
        let mut layer: Vec<Hash32> = (k * size..(k + 1) * size).map(leaf).collect();
        for depth in 0..DEPTH {
            layer = layer.chunks(2).map(|p| hasher.combine(depth as u8, &p[0], &p[1])).collect();
        }
        layer[0]
    }

    struct State {
        position: u32,
        edge: Edge,
        utxos: HashMap<Vec<u8>, UTXO>,
    }

    /// How a run builds the tree.
    #[derive(Clone, Copy, PartialEq)]
    enum Mode {
        /// Every leaf hashed by the plain loop, as before grafting: the
        /// reference.
        Hash,
        /// Grafting, the leaves waiting hashed after each batch.
        Graft,
        /// Grafting, leaves waiting for their subtree to complete.
        Wait,
    }

    /// Feeds `batches` leaves at a time, with notes at `notes`: the state
    /// after each batch, after a last flush, and the leaves grafted in all.
    fn run(
        roots: &SubtreeRoots,
        batches: &[u32],
        notes: &[u32],
        mode: Mode,
    ) -> Result<(Vec<State>, State, u64)> {
        let hasher = OrchardHasher::default();
        let mut state = State { position: 0, edge: Edge::default(), utxos: HashMap::new() };
        let mut pending = vec![];
        let mut states = vec![];
        let mut grafted = 0;
        let snapshot = |state: &State| State {
            position: state.position,
            edge: state.edge.clone(),
            utxos: state
                .utxos
                .iter()
                .map(|(k, u)| (k.clone(), UTXO { witness: u.witness.clone(), ..note(u.position) }))
                .collect(),
        };
        let mut next = 0;
        for &n in batches {
            let leaves: Vec<Hash32> = (next..next + n).map(leaf).collect();
            let found =
                notes.iter().filter(|p| (next..next + n).contains(*p)).map(|p| note(*p)).collect();
            next += n;
            let mut tree = Tree {
                hasher: &hasher,
                position: &mut state.position,
                edge: &mut state.edge,
                pending: &mut pending,
                utxos: &mut state.utxos,
            };
            if mode == Mode::Hash {
                tree.append_layer(0, leaves.into_iter().map(Some).collect(), found, None)?;
            } else {
                grafted += tree.add_leaves(&leaves, found, roots, DEPTH)?.0;
            }
            if mode == Mode::Graft {
                tree.flush(roots, DEPTH)?;
            }
            let expected = notes.iter().filter(|p| **p < next).count();
            assert_eq!(state.utxos.len(), expected, "notes lost");
            // What the scan numbers new notes from.
            assert_eq!(state.position + pending.len() as u32, next, "leaves lost or doubled");
            states.push(snapshot(&state));
        }
        let mut tree = Tree {
            hasher: &hasher,
            position: &mut state.position,
            edge: &mut state.edge,
            pending: &mut pending,
            utxos: &mut state.utxos,
        };
        tree.flush(roots, DEPTH)?;
        Ok((states, snapshot(&state), grafted))
    }

    fn all_roots(count: u32) -> SubtreeRoots {
        let hasher = OrchardHasher::default();
        SubtreeRoots { first: 0, roots: (0..count).map(|k| subtree_root(&hasher, k)).collect() }
    }

    fn same_witnesses(a: &State, b: &State, what: &str) {
        assert_eq!(a.utxos.len(), b.utxos.len(), "{what}");
        for (key, utxo) in &a.utxos {
            let at = utxo.position;
            assert!(utxo.witness == b.utxos[key].witness, "{what}: the note at {at}");
        }
    }

    fn same_tree(a: &State, b: &State, what: &str) {
        assert_eq!(a.position, b.position, "{what}");
        assert_eq!(a.edge, b.edge, "{what}");
        same_witnesses(a, b, what);
    }

    #[test]
    fn grafting_gives_the_tree_and_witnesses_hashing_gives() {
        // Batches straddling subtrees and spanning several, some ending on a
        // subtree's last leaf (32, 120), others not, some smaller than a
        // subtree; notes in a few subtrees, one found while its subtree
        // straddles two batches, one (130) after its subtree's leaves waited.
        let notes = [2, 40, 41, 75, 130];
        let patterns: [&[u32]; 3] = [
            &[5, 20, 3, 30, 1, 49, 13, 40],
            &[5, 20, 3, 4, 26, 1, 49, 12, 41],
            &[3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 30, 2, 2, 2, 2, 75],
        ];
        for batches in patterns {
            let (hashed, hashed_end, none) = run(&all_roots(40), batches, &notes, Mode::Hash).unwrap();
            let (grafted, grafted_end, some) =
                run(&all_roots(40), batches, &notes, Mode::Graft).unwrap();
            let (waited, waited_end, most) =
                run(&all_roots(40), batches, &notes, Mode::Wait).unwrap();
            assert_eq!(none, 0);
            assert!(some > 40, "{batches:?}: only {some} leaves grafted");
            assert!(most >= some, "{batches:?}: {most} grafted waiting, {some} without");
            for (i, ((a, b), c)) in hashed.iter().zip(&grafted).zip(&waited).enumerate() {
                let what = format!("{batches:?}, batch {i}");
                same_tree(a, b, &what);
                same_witnesses(a, c, &what);
            }
            same_tree(&hashed_end, &grafted_end, &format!("{batches:?}, at the end"));
            same_tree(&hashed_end, &waited_end, &format!("{batches:?}, at the end"));
        }
    }

    #[test]
    fn leaves_waiting_are_grafted_once_their_subtree_completes() {
        // Subtree 1 (leaves 8 to 15) arrives three leaves at a time.
        let (_, _, grafted) = run(&all_roots(4), &[8, 3, 3, 2], &[], Mode::Wait).unwrap();
        assert_eq!(grafted, 8);
        let (_, _, grafted) = run(&all_roots(4), &[8, 3, 3, 2], &[], Mode::Graft).unwrap();
        assert_eq!(grafted, 2, "without waiting, only the last part is grafted");
    }

    #[test]
    fn a_wrong_root_from_the_server_fails_the_scan() {
        // Subtree 0 is always hashed and checked.
        let mut roots = all_roots(10);
        roots.roots[0][0] ^= 1;
        let error = run(&roots, &[30], &[], Mode::Wait).err().expect("the wrong root was taken");
        assert!(format!("{error}").contains("subtree 0"), "{error}");
    }
}
