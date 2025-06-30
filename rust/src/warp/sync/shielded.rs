use std::marker::PhantomData;
use std::{collections::HashMap, mem::swap};

use anyhow::{Context as _, Result};
use bincode::config::legacy;
use futures::TryStreamExt;
use rayon::prelude::*;
use sqlx::{Row, SqliteConnection};
use tokio::sync::mpsc::Sender;
use tracing::info;
use zcash_protocol::consensus::Network;

use crate::lwd::{CompactBlock, CompactTx};
use crate::sync::{Note, Transaction, WarpSyncMessage, UTXO};
use crate::warp::{Edge, Hasher, Witness, MERKLE_DEPTH};
use crate::Hash32;

pub mod orchard;
pub mod sapling;

pub trait ShieldedProtocol {
    type Hasher: Hasher;
    type IVK: Sync;
    type NK: Sync;
    type Note: Sync + Send;
    type Spend;
    type Output: Sync;

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

        // map from (accout, nullifier) to NoteRef
        let mut utxos: HashMap<Vec<u8>, UTXO> = HashMap::new();

        for (account, _, _, _) in keys.iter() {
            // Use an anti join to get the unspent notes
            // and a join to filter based on the account and pool
            info!(
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
            .map(|row| {
                let id_note = row.get::<u32, _>(0);
                let account = row.get::<u32, _>(1);
                let position = row.get::<u32, _>(2);
                let nullifier = row.get::<Vec<u8>, _>(3);
                let value = row.get::<i64, _>(4) as u64;
                let cmx = row.get::<Vec<u8>, _>(5);
                let witness = row.get::<Vec<u8>, _>(6);
                let (witness, _) = bincode::decode_from_slice(&witness, legacy()).unwrap();
                UTXO {
                    id: id_note,
                    pool,
                    account,
                    nullifier,
                    position,
                    value,
                    cmx,
                    witness,
                    ..UTXO::default()
                }
            })
            .fetch(&mut *connection);
            while let Some(utxo) = nfs.try_next().await? {
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

        // new notes
        let mut notes = outputs
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
                    .unwrap()
                    .map(|(n, dbn)| (n, dbn, nk))
                })
            })
            .collect::<Vec<_>>();
        info!("Notes #{}", notes.len());

        let mut note_iterator = notes.iter_mut();
        let mut note = note_iterator.next();

        let mut position = self.position;
        let mut height = 0;
        // fill in the position of the notes
        for cb in blocks.iter() {
            height = cb.height as u32;
            for (ivtx, tx) in cb.vtx.iter().enumerate() {
                // skip over the txs until we find the next note
                loop {
                    // there could be more than one note in the same tx
                    // so we need to check all of them
                    match note {
                        Some((n, dbn, nk))
                            if dbn.height == cb.height as u32 && dbn.ivtx == ivtx as u32 =>
                        {
                            dbn.position = position + dbn.vout;
                            let nf = P::derive_nf(nk, dbn.position, n)?;
                            dbn.nf = nf.to_vec();

                            // send the tx if it is not already sent
                            // and before the notes it contains
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
                position += P::extract_outputs(tx).len() as u32;
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

        // notes are not used beyond this point

        let mut cmxs = vec![];
        let mut count_cmxs = 0;

        // #region update commitment tree state
        for depth in 0..MERKLE_DEPTH as usize {
            let mut position = self.position >> depth;
            // preprend previous trailing node (if resuming a half pair)
            if position % 2 == 1 {
                cmxs.insert(0, Some(self.tree_state.0[depth].unwrap()));
                position -= 1;
            }

            // slightly more efficient than doing it before the insert
            // the tree leaves are the note commitments
            // and the internal nodes are the hashes of the children
            if depth == 0 {
                for cb in blocks.iter() {
                    for vtx in cb.vtx.iter() {
                        for co in P::extract_outputs(vtx).iter() {
                            let cmx = P::extract_cmx(co);
                            cmxs.push(Some(cmx));
                        }
                        count_cmxs += P::extract_outputs(vtx).len();
                    }
                }
            }

            // loop on the *new* notes
            for n in new_utxos.iter_mut() {
                let npos = n.position >> depth;
                let nidx = (npos - position) as usize;

                if depth == 0 {
                    n.witness.position = npos;
                    n.witness.value = cmxs[nidx].unwrap();
                }

                if nidx % 2 == 0 {
                    // left node
                    if nidx + 1 < cmxs.len() {
                        // ommer is right node if it exists
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
                    // right node
                    assert!(
                        cmxs[nidx - 1].is_some(),
                        "{} {} {}",
                        depth,
                        n.position,
                        nidx
                    );
                    n.witness.ommers.0[depth] = cmxs[nidx - 1]; // ommer is left node
                }
            }

            let len = cmxs.len();
            if len >= 2 {
                // loop on *old notes*
                for n in self.utxos.values_mut() {
                    if n.witness.ommers.0[depth].is_none() {
                        // fill right ommer if
                        assert!(cmxs[1].is_some());
                        n.witness.ommers.0[depth] = cmxs[1]; // we just got it
                    }
                }
            }

            // save last node if not a full pair
            if len % 2 == 1 {
                self.tree_state.0[depth] = cmxs[len - 1];
            } else {
                self.tree_state.0[depth] = None;
            }

            // hash and combine to next depth
            let pairs = len / 2;
            let mut cmxs2 = self.hasher.parallel_combine_opt(depth as u8, &cmxs, pairs);
            swap(&mut cmxs, &mut cmxs2);
        }
        // #endregion

        tracing::info!("Old notes #{}", self.utxos.len());
        tracing::info!("New notes #{}", new_utxos.len());
        for utxo in new_utxos.into_iter() {
            let mut key = utxo.account.to_be_bytes().to_vec();
            key.extend_from_slice(&utxo.nullifier);
            self.utxos.insert(key, utxo);
        }
        for utxo in self.utxos.values() {
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

        // detect spends
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
