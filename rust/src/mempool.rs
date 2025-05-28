use crate::{api::mempool::MempoolMsg, frb_generated::StreamSink};
use crate::lwd::{CompactOrchardAction, CompactSaplingOutput, Empty};
use crate::warp::{try_orchard_decrypt, try_sapling_decrypt};
use anyhow::{Context as _, Result};
use itertools::Itertools;
use orchard::keys::Scope;
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use tokio_util::sync::CancellationToken;
use tonic::Request;
use tracing::info;
use zcash_keys::encoding::AddressCodec as _;
use zcash_note_encryption::COMPACT_NOTE_SIZE;
use zcash_primitives::{legacy::TransparentAddress, transaction::{Authorized, Transaction, TransactionData}};
use zcash_protocol::consensus::{BlockHeight, BranchId, Network};

use crate::Client;

pub async fn run_mempool(
    mempool_tx: StreamSink<MempoolMsg>,
    network: &Network,
    connection: &SqlitePool,
    client: &mut Client,
    height: u32,
    cancel_token: CancellationToken
) -> Result<()> {
    let transparent_accounts = sqlx::query(
        r#"SELECT a.id_account, a.name, ta.address FROM accounts a
        JOIN transparent_address_accounts ta
        ON a.id_account = ta.account"#,
    )
    .map(|row: SqliteRow| {
        let account: u32 = row.get(0);
        let name: String = row.get(1);
        let address: String = row.get(2);
        let address = TransparentAddress::decode(network, &address).unwrap();
        (account, name, address)
    })
    .fetch_all(connection)
    .await
    .context("transparent_accounts")?;

    let sapling_accounts = sqlx::query(
        r#"SELECT account, name, xvk FROM accounts a JOIN sapling_accounts s
        ON a.id_account = s.account"#,
    )
    .map(|row: SqliteRow| {
        let account: u32 = row.get(0);
        let name: String = row.get(1);
        let xvk: Vec<u8> = row.get(2);
        let fvk = sapling_crypto::keys::FullViewingKey::read(&*xvk).unwrap();
        (account, name, fvk)
    })
    .fetch_all(connection)
    .await
    .context("sapling_accounts")?;

    let orchard_accounts = sqlx::query(
        r#"SELECT account, name, xvk FROM accounts a JOIN orchard_accounts o
        ON a.id_account = o.account"#,
    )
    .map(|row: SqliteRow| {
        let account: u32 = row.get(0);
        let name: String = row.get(1);
        let xvk: Vec<u8> = row.get(2);
        let fvk = orchard::keys::FullViewingKey::read(&*xvk).unwrap();
        (account, name, fvk)
    })
    .fetch_all(connection)
    .await
    .context("orchard_accounts")?;

    let mut mempool_txs = client
        .get_mempool_stream(Request::new(Empty {}))
        .await?
        .into_inner();

    let consensus_branch_id = BranchId::for_height(network, BlockHeight::from_u32(height));
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                info!("Mempool stream cancelled");
                break;
            }
            tx = mempool_txs.message() => {
                match tx {
                    Ok(Some(tx)) => {
                        let txdata = tx.data;
                        let tx = Transaction::read(&*txdata, consensus_branch_id)?;
                        let txid = tx.txid();
                        let tx_hash = txid.to_string();
                        info!("Processing mempool transaction {}", tx_hash);

                        let tx_data = tx.into_data();

                        let notes = decode_raw_transaction(
                            network,
                            connection,
                            height,
                            &transparent_accounts,
                            &sapling_accounts,
                            &orchard_accounts,
                            &tx_data,
                        ).await?;
                        let a = notes
                            .iter()
                            .map(|n| ((n.account, n.name.clone()), n.value))
                            .into_group_map();
                        let amounts = a
                            .into_iter()
                            .map(|((account, name), note_values)| {
                                (account, name, note_values.iter().sum::<i64>())
                            })
                            .collect::<Vec<_>>();

                        let _ = mempool_tx.add(MempoolMsg::TxId(tx_hash, amounts, txdata.len() as u32));
                    }
                    Ok(None) => {
                        info!("No more transactions in mempool stream");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("Error receiving mempool transaction: {}", e);
                    }
                }
            }

        }
    }
    Ok(())
}

pub struct MempoolNote {
    pub account: u32,
    pub name: String,
    pub value: i64,
}

pub async fn decode_raw_transaction(
    network: &Network,
    connection: &SqlitePool,
    height: u32,
    tkeys: &[(u32, String, TransparentAddress)],
    zkeys: &[(u32, String, sapling_crypto::keys::FullViewingKey)],
    okeys: &[(u32, String, orchard::keys::FullViewingKey)],
    tx_data: &TransactionData<Authorized>,
) -> Result<Vec<MempoolNote>> {
    let mut notes = vec![];

    if let Some(tbundle) = tx_data.transparent_bundle() {
        for v in tbundle.vin.iter() {
            let mut nf = vec![];
            v.prevout.write(&mut nf)?;
            let spent_amount = sqlx::query(
                "SELECT account, name, value FROM notes n
                JOIN accounts a ON n.account = a.id_account
                WHERE nullifier = ?",
            )
            .bind(&nf)
            .map(|row: SqliteRow| {
                let account: u32 = row.get(0);
                let name: String = row.get(1);
                let value: i64 = row.get(2);
                MempoolNote {
                    account,
                    name,
                    value: -value,
                }
            })
            .fetch_all(connection)
            .await?;
            notes.extend(spent_amount);
        }
        for v in tbundle.vout.iter() {
            if let Some(vout_address) = v.script_pubkey.address() {
                if let Some((account, name, _)) = tkeys.iter().find(|(_, _, a)| a == &vout_address)
                {
                    let n = MempoolNote {
                        account: *account,
                        name: name.clone(),
                        value: v.value.into_u64() as i64,
                    };
                    notes.push(n);
                }
            }
        }
    }

    if let Some(sbundle) = tx_data.sapling_bundle() {
        for v in sbundle.shielded_spends().iter() {
            let nf = &v.nullifier().to_vec();
            let spent_amount = sqlx::query(
                "SELECT account, name, value FROM notes n
                JOIN accounts a ON n.account = a.id_account
                WHERE nullifier = ?",
            )
            .bind(nf)
            .map(|row: SqliteRow| {
                let account: u32 = row.get(0);
                let name: String = row.get(1);
                let value: i64 = row.get(2);
                MempoolNote {
                    account,
                    name,
                    value: -value,
                }
            })
            .fetch_all(connection)
            .await?;
            notes.extend(spent_amount);
        }
        for v in sbundle.shielded_outputs().iter() {
            for (account, name, fvk) in zkeys.iter() {
                let ivk = fvk.vk.ivk();
                let co = CompactSaplingOutput {
                    cmu: v.cmu().to_bytes().to_vec(),
                    epk: v.ephemeral_key().0.to_vec(),
                    ciphertext: v.enc_ciphertext()[..COMPACT_NOTE_SIZE].to_vec(),
                };
                if let Some((note, _)) =
                    try_sapling_decrypt(network, *account, 0, &ivk, height, 0, 0, &co)?
                {
                    let amount = note.value().inner() as i64;
                    notes.push(MempoolNote {
                        account: *account,
                        name: name.clone(),
                        value: amount,
                    });
                }
            }
        }
    }

    if let Some(obundle) = tx_data.orchard_bundle() {
        for v in obundle.actions().iter() {
            let nf = v.nullifier().to_bytes().to_vec();
            let spent_amount = sqlx::query(
                "SELECT account, name, value FROM notes n
                JOIN accounts a ON n.account = a.id_account
                WHERE nullifier = ?",
            )
            .bind(&nf)
            .map(|row: SqliteRow| {
                let account: u32 = row.get(0);
                let name: String = row.get(1);
                let value: i64 = row.get(2);
                MempoolNote {
                    account,
                    name,
                    value: -value,
                }
            })
            .fetch_all(connection)
            .await?;
            notes.extend(spent_amount);

            for (account, name, fvk) in okeys.iter() {
                let ca = CompactOrchardAction {
                    nullifier: v.nullifier().to_bytes().to_vec(),
                    cmx: v.cmx().to_bytes().to_vec(),
                    ephemeral_key: v.encrypted_note().epk_bytes.to_vec(),
                    ciphertext: v.encrypted_note().enc_ciphertext[..COMPACT_NOTE_SIZE].to_vec(),
                };
                for scope in 0..2 {
                    let s = if scope == 0 {
                        Scope::External
                    } else {
                        Scope::Internal
                    };
                    let ivk = fvk.to_ivk(s);
                    if let Some((note, _)) =
                        try_orchard_decrypt(network, *account, scope, &ivk, height, 0, 0, &ca)?
                    {
                        let amount = note.value().inner() as i64;
                        notes.push(MempoolNote {
                            account: *account,
                            name: name.clone(),
                            value: amount,
                        });
                    }
                }
            }
        }
    }

    Ok(notes)
}
