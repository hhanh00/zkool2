use std::collections::HashSet;

use anyhow::Result;
use rusqlite::{Connection, Transaction};
use zcash_client_backend::encoding::AddressCodec;
use zcash_keys::address::Address as RecipientAddress;
use zcash_primitives::legacy::TransparentAddress;

use crate::{
    db::{
        account::{list_transparent_addresses, TransparentDerPath},
        notes::{list_all_utxos, mark_transparent_spent, store_utxo},
        tx::add_tx_value,
    },
    network::Network,
    warp::{OutPoint, TransparentTx, UTXO},
};

use super::{IdSpent, ReceivedTx, TxValueUpdate};

pub struct TransparentSync {
    pub network: Network,
    pub addresses: Vec<(TransparentDerPath, TransparentAddress)>,
    pub utxos: Vec<UTXO>,
    pub txs: Vec<(ReceivedTx, OutPoint, u64)>,
    pub tx_updates: Vec<(TxValueUpdate, IdSpent<OutPoint>)>,
    pub heights: HashSet<u32>,
}

impl TransparentSync {
    pub fn new(network: &Network, connection: &Connection) -> Result<Self> {
        let addresses = list_transparent_addresses(connection)?
            .into_iter()
            .map(|(path, address)| {
                let RecipientAddress::Transparent(ta) =
                    RecipientAddress::decode(network, &address).unwrap()
                else {
                    unreachable!()
                };
                (path, ta)
            })
            .collect::<Vec<_>>();
        let utxos = list_all_utxos(connection)?;

        Ok(Self {
            network: network.clone(),
            addresses,
            utxos,
            txs: vec![],
            tx_updates: vec![],
            heights: HashSet::new(),
        })
    }

    pub fn process_txs(&mut self, address: &str, txs: &[TransparentTx]) -> Result<()> {
        for tx in txs {
            for vin in tx.vins.iter() {
                let r = self.utxos.iter().find(|&utxo| {
                    utxo.txid == vin.txid
                        && utxo.vout == vin.vout
                        && utxo.account == tx.account
                        && &utxo.address == address
                });
                if let Some(utxo) = r {
                    let id_spent = IdSpent::<OutPoint> {
                        id_note: 0,
                        account: utxo.account,
                        height: tx.height,
                        txid: tx.txid.clone(),
                        note_ref: vin.clone(),
                    };
                    let tx_value = TxValueUpdate {
                        id_tx: 0,
                        account: tx.account,
                        txid: tx.txid,
                        value: -(utxo.value as i64),
                        height: tx.height,
                        timestamp: tx.timestamp,
                    };
                    self.tx_updates.push((tx_value, id_spent));
                    self.heights.insert(tx.height);
                }
            }
            for txout in tx.vouts.iter() {
                let rtx = ReceivedTx {
                    id: 0,
                    account: tx.account,
                    height: tx.height,
                    txid: tx.txid,
                    timestamp: tx.timestamp,
                    ivtx: 0,
                    value: 0,
                };
                self.txs.push((
                    rtx,
                    OutPoint {
                        txid: tx.txid,
                        vout: txout.vout,
                    },
                    txout.value,
                ));
                // outputs are filtered for our account
                let address = tx.address.encode(&self.network);
                let utxo = UTXO {
                    is_new: true,
                    id: 0,
                    account: tx.account,
                    external: tx.external,
                    addr_index: tx.addr_index,
                    height: tx.height,
                    timestamp: tx.timestamp,
                    txid: tx.txid,
                    vout: txout.vout,
                    address,
                    value: txout.value,
                };
                self.utxos.push(utxo);
                self.heights.insert(tx.height);
            }
        }

        Ok(())
    }

    pub fn flush(self, db_tx: &Transaction) -> Result<()> {
        for utxo in self.utxos.iter() {
            store_utxo(db_tx, utxo)?;
        }
        for (tx, spend) in self.tx_updates.iter() {
            add_tx_value(db_tx, &tx)?;
            mark_transparent_spent(db_tx, spend)?;
        }
        Ok(())
    }
}
