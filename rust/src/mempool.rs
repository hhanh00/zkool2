use crate::api::coin::Network;
use crate::api::mempool::{MempoolAmount, MempoolMsg, MempoolNote, MempoolTx};
use anyhow::{Context as _, Result};
use itertools::Itertools;
use orchard::{keys::Scope, note_encryption::OrchardDomain};
use sapling_crypto::{
    keys::PreparedIncomingViewingKey,
    note_encryption::SaplingDomain,
    zip32::DiversifiableFullViewingKey,
};
use sqlx::SqliteConnection;
use sqlx::{sqlite::SqliteRow, Row};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use zcash_keys::{address::UnifiedAddress, encoding::AddressCodec as _};
use zcash_note_encryption::try_note_decryption;
use zcash_primitives::transaction::{
    components::sapling::zip212_enforcement, Authorized, Transaction, TransactionData,
};
use zcash_protocol::memo::Memo;
use zcash_transparent::address::TransparentAddress;

use crate::{Client, Sink};
#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;

#[cfg(feature = "flutter")]
pub async fn run_mempool(
    mempool_tx: StreamSink<MempoolMsg>,
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    cancel_token: CancellationToken,
) -> Result<()> {
    run_mempool_impl(mempool_tx, network, connection, client, cancel_token).await
}

pub async fn run_mempool_impl<S: Sink<MempoolMsg> + Send + 'static>(
    mempool_tx: S,
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    cancel_token: CancellationToken,
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
    .fetch_all(&mut *connection)
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
        let fvk = DiversifiableFullViewingKey::from_bytes(&xvk.try_into().unwrap()).unwrap();
        (account, name, fvk)
    })
    .fetch_all(&mut *connection)
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
    .fetch_all(&mut *connection)
    .await
    .context("orchard_accounts")?;

    'outer: loop {
        let height = client.latest_height().await?;
        mempool_tx.send(MempoolMsg::BlockHeight(height)).await;

        let mut mempool_txs = client.mempool_stream(network).await?;

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    break 'outer;
                }
                r = mempool_txs.next() => {
                    match r {
                        Some((_, tx, len)) => {
                            let txid = tx.txid();
                            let tx_hash = txid.to_string();
                            tracing::info!("MP {tx_hash}");

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
                            let amounts = notes
                                .iter()
                                .map(|n| ((n.account, n.name.clone()), n.value))
                                .into_group_map()
                                .into_iter()
                                .map(|((account, name), values)| MempoolAmount {
                                    account,
                                    name,
                                    value: values.iter().sum(),
                                })
                                .collect::<Vec<_>>();
                            mempool_tx.send(MempoolMsg::TxId(MempoolTx {
                                txid: tx_hash,
                                amounts,
                                notes,
                                size: len as u32,
                            })).await;
                        }
                        None => {
                            break;
                        }
                    }
                }

            }
        }
    }
    Ok(())
}

fn memo_bytes_to_text(bytes: &[u8]) -> Option<String> {
    Memo::from_bytes(bytes).ok().and_then(|m| match m {
        Memo::Text(t) => Some(t.to_string()),
        _ => None,
    })
}

pub async fn decode_raw_transaction(
    network: &Network,
    connection: &mut SqliteConnection,
    height: u32,
    tkeys: &[(u32, String, TransparentAddress)],
    zkeys: &[(u32, String, DiversifiableFullViewingKey)],
    okeys: &[(u32, String, orchard::keys::FullViewingKey)],
    tx_data: &TransactionData<Authorized>,
) -> Result<Vec<MempoolNote>> {
    let mut notes = vec![];

    if let Some(tbundle) = tx_data.transparent_bundle() {
        for v in tbundle.vin.iter() {
            let mut nf = vec![];
            v.prevout().write(&mut nf)?;
            let spent_amount = sqlx::query(
                "SELECT a.id_account, a.name, n.value, n.pool, n.scope, n.diversifier, m.memo_text
                FROM notes n
                JOIN accounts a ON n.account = a.id_account
                LEFT JOIN memos m ON m.note = n.id_note
                WHERE n.nullifier = ?",
            )
            .bind(&nf)
            .map(|row: SqliteRow| MempoolNote {
                account: row.get(0),
                name: row.get(1),
                value: -(row.get::<i64, _>(2)),
                pool: row.get(3),
                scope: row.get::<Option<u8>, _>(4).unwrap_or(0),
                diversifier: row.get(5),
                diversifier_index: None,
                address: None,
                memo: row.get(6),
            })
            .fetch_all(&mut *connection)
            .await?;
            notes.extend(spent_amount);
        }
        for v in tbundle.vout.iter() {
            if let Some(vout_address) = v.recipient_address() {
                if let Some((account, name, _)) = tkeys.iter().find(|(_, _, a)| a == &vout_address)
                {
                    notes.push(MempoolNote {
                        account: *account,
                        name: name.clone(),
                        value: v.value().into_u64() as i64,
                        pool: 0,
                        scope: 0,
                        diversifier: None,
                        diversifier_index: None,
                        address: Some(vout_address.encode(network)),
                        memo: None,
                    });
                }
            }
        }
    }

    if let Some(sbundle) = tx_data.sapling_bundle() {
        for v in sbundle.shielded_spends().iter() {
            let nf = &v.nullifier().to_vec();
            let spent_amount = sqlx::query(
                "SELECT a.id_account, a.name, n.value, n.pool, n.scope, n.diversifier, m.memo_text
                FROM notes n
                JOIN accounts a ON n.account = a.id_account
                LEFT JOIN memos m ON m.note = n.id_note
                WHERE n.nullifier = ?",
            )
            .bind(nf)
            .map(|row: SqliteRow| MempoolNote {
                account: row.get(0),
                name: row.get(1),
                value: -(row.get::<i64, _>(2)),
                pool: row.get(3),
                scope: row.get::<Option<u8>, _>(4).unwrap_or(0),
                diversifier: row.get(5),
                diversifier_index: None,
                address: None,
                memo: row.get(6),
            })
            .fetch_all(&mut *connection)
            .await?;
            notes.extend(spent_amount);
        }
        let domain = SaplingDomain::new(zip212_enforcement(network, height.into()));
        for v in sbundle.shielded_outputs().iter() {
            for (account, name, dfvk) in zkeys.iter() {
                for scope in [zip32::Scope::External, zip32::Scope::Internal] {
                    let ivk = dfvk.to_ivk(scope);
                    let pivk = PreparedIncomingViewingKey::new(&ivk);
                    if let Some((note, recipient, memo_bytes)) =
                        try_note_decryption(&domain, &pivk, v)
                    {
                        let diversifier = recipient.diversifier().0.to_vec();
                        let address = recipient.encode(network);
                        let diversifier_index = dfvk
                            .decrypt_diversifier(&recipient)
                            .and_then(|(di, _)| di.try_into().ok())
                            .map(|d: u64| d as i64);
                        notes.push(MempoolNote {
                            account: *account,
                            name: name.clone(),
                            value: note.value().inner() as i64,
                            pool: 1,
                            scope: if scope == zip32::Scope::External { 0 } else { 1 },
                            diversifier: Some(diversifier),
                            diversifier_index,
                            address: Some(address),
                            memo: memo_bytes_to_text(&memo_bytes),
                        });
                        break;
                    }
                }
            }
        }
    }

    if let Some(obundle) = tx_data.orchard_bundle() {
        for v in obundle.actions().iter() {
            let nf = v.nullifier().to_bytes().to_vec();
            let spent_amount = sqlx::query(
                "SELECT a.id_account, a.name, n.value, n.pool, n.scope, n.diversifier, m.memo_text
                FROM notes n
                JOIN accounts a ON n.account = a.id_account
                LEFT JOIN memos m ON m.note = n.id_note
                WHERE n.nullifier = ?",
            )
            .bind(&nf)
            .map(|row: SqliteRow| MempoolNote {
                account: row.get(0),
                name: row.get(1),
                value: -(row.get::<i64, _>(2)),
                pool: row.get(3),
                scope: row.get::<Option<u8>, _>(4).unwrap_or(0),
                diversifier: row.get(5),
                diversifier_index: None,
                address: None,
                memo: row.get(6),
            })
            .fetch_all(&mut *connection)
            .await?;
            notes.extend(spent_amount);

            let domain = OrchardDomain::for_action(v);
            for (account, name, fvk) in okeys.iter() {
                for scope in [Scope::External, Scope::Internal] {
                    let ivk = fvk.to_ivk(scope);
                    let pivk = orchard::keys::PreparedIncomingViewingKey::new(&ivk);
                    if let Some((note, recipient, memo_bytes)) =
                        try_note_decryption(&domain, &pivk, v)
                    {
                        let diversifier = recipient.diversifier().as_array().to_vec();
                        let ua =
                            UnifiedAddress::from_receivers(Some(recipient), None, None).unwrap();
                        let address = ua.encode(network);
                        let diversifier_index = ivk
                            .diversifier_index(&recipient)
                            .and_then(|di| di.try_into().ok())
                            .map(|d: u64| d as i64);
                        notes.push(MempoolNote {
                            account: *account,
                            name: name.clone(),
                            value: note.value().inner() as i64,
                            pool: 2,
                            scope: if scope == Scope::External { 0 } else { 1 },
                            diversifier: Some(diversifier),
                            diversifier_index,
                            address: Some(address),
                            memo: memo_bytes_to_text(&memo_bytes),
                        });
                        break;
                    }
                }
            }
        }
    }

    Ok(notes)
}

pub async fn get_mempool_tx(
    network: &Network,
    client: &mut Client,
    tx_id: &str,
) -> Result<Transaction> {
    let mut tx_id = hex::decode(tx_id)?;
    tx_id.reverse();
    let (_, tx) = client.transaction(network, &tx_id).await?;
    Ok(tx)
}
