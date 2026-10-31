//! Transparent scan: lightwalletd address history for the accounts'
//! recent receive and change addresses, recorded through the chain log one
//! address at a time.

use anyhow::Result;
use sqlx::SqliteConnection;
use sqlx::{sqlite::SqliteRow, Row};
use tokio_util::sync::CancellationToken;
use tokio_stream::StreamExt;
use tracing::debug;
use zcash_transparent::address::TransparentAddress;

use crate::api::coin::Network;

use crate::Client;

use super::chain_log::{self, NoteKey, NoteRow, SpendRow, TransactionRow};
use sqlx::Connection;

use zcash_keys::encoding::AddressCodec;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn transparent_sync(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    accounts: &[u32],
    start_height: u32,
    end_height: u32,
    limit: u32,
    cancel: CancellationToken,
) -> Result<()> {
    let mut addresses = vec![];
    debug!(
        "transparent_sync: scanning accounts {:?} from height {} to {} with limit {}",
        accounts, start_height, end_height, limit
    );
    for account in accounts {
        // scan the most recent receive and change addresses, bounded by `limit`
        let mut rows = sqlx::query("
                WITH receive AS
                (SELECT * FROM transparent_address_accounts WHERE account = ?1 AND scope = 0 ORDER BY dindex DESC LIMIT ?2),
                change AS
                (SELECT * FROM transparent_address_accounts WHERE account = ?1 AND scope = 1 ORDER BY dindex DESC LIMIT ?2)
                SELECT id_taddress, address FROM receive UNION ALL SELECT id_taddress, address FROM change")
            .bind(account)
            .bind(limit)
            .map(|row: SqliteRow| {
                let id_taddress: u32 = row.get(0);
                let address: String = row.get(1);
                (id_taddress, address)
            })
            .fetch(&mut *connection);

        let mut addr_count = 0u32;
        while let Some((id_taddress, address)) = rows.try_next().await? {
            debug!(
                "transparent_sync: account {} has taddress id={} addr={}",
                account, id_taddress, address
            );
            addr_count += 1;
            // Add the address to the client
            addresses.push((*account, (id_taddress, address)));
        }
        debug!(
            "transparent_sync: account {} has {} transparent addresses to scan",
            account, addr_count
        );
    }
    debug!(
        "transparent_sync: total {} addresses to scan across all accounts",
        addresses.len()
    );
    for (account, address_row) in addresses.iter() {
        let my_address = TransparentAddress::decode(&network, &address_row.1)?;
        debug!(
            "transparent_sync: scanning account {} address {} (decoded: {:?})",
            account,
            address_row.1,
            my_address.encode(network)
        );
        let mut txs = client
            .taddress_txs(network, &address_row.1, start_height, end_height)
            .await?
            .into_inner();

        let mut db_tx = connection.begin().await?;
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    debug!("Canceling sync");
                    anyhow::bail!("Sync canceled");
                }
                m = txs.recv() => {
                    // A dead stream must not pass for an empty one: the
                    // checkpoint below would then skip the transactions it
                    // never delivered (V12 F-77).
                    let m = match m {
                        Some(Err(e)) => {
                            return Err(e.context(format!(
                                "the transaction stream for {} failed",
                                address_row.1
                            )));
                        }
                        Some(Ok(item)) => Some(item),
                        None => None,
                    };
                    if let Some((height, transaction, _)) = m {
                        let txid = transaction.txid().as_ref().to_vec();
                        debug!(
                            "transparent_sync: found tx {} at height {} for account {} version={:?} branch_id={:?}",
                            hex::encode(&txid),
                            height,
                            account,
                            transaction.version(),
                            transaction.consensus_branch_id(),
                        );
                        // tx time is available in the block (not here)
                        let inserted = chain_log::record_transaction(
                            &mut db_tx,
                            *account,
                            &TransactionRow::chain(&txid, height, 0),
                        )
                        .await?;
                        debug!(
                            "transparent_sync: tx {} inserted into transactions (new={})",
                            hex::encode(&txid),
                            inserted.is_some()
                        );
                        let (id_tx, _) = chain_log::transaction_ref(&mut db_tx, *account, &txid).await?;

                        // Access the transparent bundle part
                        if let Some(transparent_bundle) = transaction.transparent_bundle() {
                            debug!(
                                "transparent_sync: tx {} has transparent bundle: {} vins, {} vouts",
                                transaction.txid(),
                                transparent_bundle.vin.len(),
                                transparent_bundle.vout.len()
                            );

                            let vins = &transparent_bundle.vin;
                            for vin in vins.iter() {
                                // The "nullifier" of a transparent input is the outpoint
                                let mut nf = vec![];
                                vin.prevout().write(&mut nf)?;

                                let row =
                                    chain_log::find_note(&mut db_tx, *account, NoteKey::Nullifier(&nf))
                                        .await?;

                                if let Some((id, amount)) = row {
                                    debug!(
                                        "transparent_sync: tx {} vin spends note {} amount {}",
                                        transaction.txid(),
                                        id,
                                        amount
                                    );
                                    // note was found: add a spent entry (this also
                                    // releases any broadcast-time lock)
                                    chain_log::record_spend(
                                        &mut db_tx,
                                        *account,
                                        &SpendRow {
                                            note: NoteKey::Id(id),
                                            id_tx,
                                            height,
                                            pool: 0,
                                            value: -amount,
                                        },
                                    )
                                    .await?;
                                }
                            }

                            let vouts = &transparent_bundle.vout;
                            for (i, vout) in vouts.iter().enumerate() {
                                let vout_value = vout.value().into_u64();
                                if let Some(vout_addr) = vout.recipient_address() {
                                    let vout_addr_encoded = vout_addr.encode(network);
                                    let my_addr_encoded = my_address.encode(network);
                                    let is_match = vout_addr == my_address;
                                    debug!(
                                        "transparent_sync: tx {} vout[{}] value={} recipient={} my_address={} match={}",
                                        transaction.txid(),
                                        i,
                                        vout_value,
                                        vout_addr_encoded,
                                        my_addr_encoded,
                                        is_match,
                                    );
                                    if is_match {
                                        // It is for me
                                        // add a new note entry
                                        let mut nf = transaction.txid().as_ref().to_vec();
                                        nf.extend_from_slice(&(i as u32).to_le_bytes());

                                        let note = chain_log::record_note(
                                            &mut db_tx,
                                            *account,
                                            &NoteRow {
                                                id_tx,
                                                height,
                                                pool: 0,
                                                nullifier: &nf,
                                                value: vout_value as i64,
                                                taddress: Some(address_row.0),
                                                ..Default::default()
                                            },
                                        )
                                        .await?;
                                        debug!(
                                            "transparent_sync: tx {} vout[{}] NOTE CREATED value={} new={}",
                                            transaction.txid(),
                                            i,
                                            vout_value,
                                            note.is_some()
                                        );
                                    }
                                } else {
                                    debug!(
                                        "transparent_sync: tx {} vout[{}] value={} has NO recipient address (script cannot be decoded)",
                                        transaction.txid(),
                                        i,
                                        vout_value,
                                    );
                                }
                            }
                        } else {
                            debug!(
                                "transparent_sync: tx {} has NO transparent bundle (shielded-only tx) version={:?} branch_id={:?} height={}",
                                transaction.txid(),
                                transaction.version(),
                                transaction.consensus_branch_id(),
                                height,
                            );
                        }
                    }
                    else {
                        // No more transactions
                        break;
                    }
                }
            }
        }

        chain_log::checkpoint(&mut db_tx, *account, 0, end_height).await?;
        db_tx.commit().await?;
    }

    Ok(())
}
