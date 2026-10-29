use crate::sync::{
    chain_log::{self, MemoRow, NoteKey, OutputRow, TransactionDetails},
    session::Held,
};
use anyhow::Result;
use orchard::{
    note::ExtractedNoteCommitment,
    note_encryption::{IronwoodDomain, OrchardDomain},
    zsa::OrchardZSADomain,
};
use sapling_crypto::note_encryption::SaplingDomain;
use sqlx::{sqlite::SqliteRow, Connection, Row, SqliteConnection};
use std::collections::HashMap;
use tracing::debug;
use zcash_keys::{address::UnifiedAddress, encoding::AddressCodec};
use zcash_note_encryption::note_bytes::NoteBytesData;
use zcash_note_encryption::{try_note_decryption, try_output_recovery_with_ovk};
use zcash_primitives::transaction::{components::sapling::zip212_enforcement, OrchardBundle};
use zcash_protocol::memo::Memo;

use crate::{
    api::coin::Network,
    db::{orchard_viewing_keys, sapling_viewing_keys},
    pay::fee::{script_sig_pushes_uncompressed_key, FeeManager},
    Client,
};

const TX_TYPE_MIGRATION: u8 = 16;

/// Decrypt and store the details of every transaction of `account` that
/// lacks them, under the caller's session: the sync's own detail pass.
/// The app's background pass does the same work in two halves, through
/// [`crate::api::sync::fetch_tx_details`].
pub async fn fetch_tx_details(
    network: &Network,
    connection: &mut Held<'_>,
    client: &mut Client,
    account: u32,
) -> Result<()> {
    debug!("fetch_tx_details");
    for txid in pending_details(connection, account).await? {
        let tx = decrypt_tx(network, connection, client, account, &txid).await?;
        apply_tx_details(connection, account, &tx).await?;
    }
    Ok(())
}

/// Txids of the account's transactions whose details are not stored yet.
pub async fn pending_details(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Vec<Vec<u8>>> {
    Ok(
        sqlx::query_scalar("SELECT txid FROM transactions WHERE account = ? AND details = FALSE")
            .bind(account)
            .fetch_all(&mut *connection)
            .await?,
    )
}

/// A transaction's details, decrypted and ready to store. Produced by
/// [`decrypt_tx`] from the network and the account's keys alone, so the
/// app can build it outside the sync session; stored by
/// [`apply_tx_details`], which looks the rows up again: the ones the
/// decryption saw may have been rewound since.
#[derive(Clone, Debug)]
pub struct DecryptedTx {
    pub txid: Vec<u8>,
    pub height: u32,
    pub fee: u64,
    pub outputs: Vec<DecryptedOutput>,
    pub memos: Vec<DecryptedMemo>,
}

/// An output the account sent, recovered through its outgoing key, or a
/// transparent output of one of its transactions.
#[derive(Clone, Debug)]
pub struct DecryptedOutput {
    pub pool: u8,
    pub vout: u32,
    pub value: u64,
    pub address: String,
}

#[derive(Clone, Debug)]
pub struct DecryptedMemo {
    pub pool: u8,
    pub vout: u32,
    pub link: MemoLink,
    pub text: Option<String>,
    pub bytes: Vec<u8>,
}

/// What a memo hangs off. Named by content, not row id, and resolved when
/// the memo is stored.
#[derive(Clone, Debug)]
pub enum MemoLink {
    /// A note the account received, by commitment. The memo is stored
    /// without a link when the note row is missing.
    Note { cmx: Vec<u8> },
    /// The output recorded at the memo's pool and index.
    Output,
    /// A note by commitment when the account has it, else an output
    /// recorded with this value and address (a ZSA note).
    NoteOrOutput {
        cmx: Vec<u8>,
        value: u64,
        address: String,
    },
}

async fn summarize_tx(
    connection: &mut SqliteConnection,
    tx: u32,
) -> Result<(u8, i64, Option<i32>, i64)> {
    let (value, fee) = sqlx::query(
        "WITH n AS (SELECT value, tx FROM notes WHERE id_asset IS NULL
                    UNION ALL
                    SELECT s.value, s.tx FROM spends s
                    JOIN notes n ON s.id_note = n.id_note WHERE n.id_asset IS NULL)
        SELECT SUM(n.value), t.fee FROM n JOIN transactions t ON t.id_tx = n.tx WHERE n.tx = ?",
    )
    .bind(tx)
    .map(|row: SqliteRow| {
        let value = row.get::<Option<i64>, _>(0).unwrap_or_default();
        let fee = row.get::<Option<i64>, _>(1).unwrap_or_default();
        (value, fee)
    })
    .fetch_one(&mut *connection)
    .await?;

    // Compute ZSA summary: pick the first asset's net transfer amount
    let (asset_id, zsa_value) = sqlx::query(
        "WITH n AS (
            SELECT value, id_asset, tx FROM notes WHERE id_asset IS NOT NULL
            UNION ALL
            SELECT s.value, n.id_asset, s.tx FROM spends s
            JOIN notes n ON s.id_note = n.id_note WHERE n.id_asset IS NOT NULL
        )
        SELECT id_asset, SUM(value) FROM n WHERE tx = ? GROUP BY id_asset LIMIT 1",
    )
    .bind(tx)
    .map(|row: SqliteRow| {
        let id_asset: Option<i32> = row.get(0);
        let zsa_value: Option<i64> = row.get(1);
        (id_asset, zsa_value.unwrap_or_default())
    })
    .fetch_optional(&mut *connection)
    .await?
    .unwrap_or((None, 0));

    if value > 0 || zsa_value > 0 {
        // receiving
        Ok((1, value, asset_id, zsa_value))
    } else if value < -fee || zsa_value < 0 {
        // sending
        Ok((2, value, asset_id, zsa_value))
    } else {
        // self transfer
        let (has_tspend, has_tnote, has_ospend, has_inote) = sqlx::query(
            "SELECT
                EXISTS(SELECT 1 FROM spends WHERE tx = ? AND pool = 0),
                EXISTS(SELECT 1 FROM notes WHERE tx = ? AND pool = 0),
                EXISTS(SELECT 1 FROM spends WHERE tx = ? AND pool = 2),
                EXISTS(SELECT 1 FROM notes WHERE tx = ? AND pool = 3)",
        )
        .bind(tx)
        .bind(tx)
        .bind(tx)
        .bind(tx)
        .map(|row: SqliteRow| {
            (
                row.get::<bool, _>(0),
                row.get::<bool, _>(1),
                row.get::<bool, _>(2),
                row.get::<bool, _>(3),
            )
        })
        .fetch_one(&mut *connection)
        .await?;
        let tpe = self_transfer_type(value, fee, has_tspend, has_tnote, has_ospend, has_inote);
        Ok((tpe, value, asset_id, zsa_value))
    }
}

fn self_transfer_type(
    value: i64,
    fee: i64,
    has_tspend: bool,
    has_tnote: bool,
    has_ospend: bool,
    has_inote: bool,
) -> u8 {
    if has_ospend && has_inote && value == -fee {
        TX_TYPE_MIGRATION
    } else {
        (if has_tspend { 8 } else { 0 }) | (if has_tnote { 4 } else { 0 })
    }
}

/// Try ZSA decryption using the raw 612-byte enc_ciphertext with OrchardZSADomain.
/// Called when vanilla OrchardDomain decryption fails and raw ZSA ciphertext is available.
fn try_zsa_decrypt(
    action: &orchard::Action<
        <orchard::bundle::Authorized as orchard::bundle::Authorization>::SpendAuth,
    >,
    raw_enc: &[u8],
    pivk: &orchard::keys::PreparedIncomingViewingKey,
    ovk: Option<&orchard::keys::OutgoingViewingKey>,
) -> Option<(orchard::Note, orchard::Address, [u8; 512])> {
    use orchard::note::TransmittedNoteCiphertext;
    use zcash_note_encryption::{try_note_decryption, try_output_recovery_with_ovk};

    let vanilla_nc = action.encrypted_note();

    // Reconstruct the ZSA TransmittedNoteCiphertext from raw bytes
    let mut enc = NoteBytesData([0u8; 612]);
    enc.0.copy_from_slice(raw_enc);

    let zsa_nc = TransmittedNoteCiphertext::<OrchardZSADomain> {
        epk_bytes: vanilla_nc.epk_bytes,
        enc_ciphertext: enc,
        out_ciphertext: vanilla_nc.out_ciphertext,
    };

    let zsa_action = orchard::Action::from_parts(
        *action.nullifier(),
        action.rk().clone(),
        orchard::note::ExtractedNoteCommitment::from_bytes(&action.cmx().to_bytes()).unwrap(),
        zsa_nc,
        action.cv_net().clone(),
        (),
    )
    .ok()?;

    let zsa_domain = OrchardZSADomain {
        rho: zsa_action.rho(),
    };

    if let Some((note, _address, memo_bytes)) = try_note_decryption(&zsa_domain, pivk, &zsa_action)
    {
        Some((note, _address, memo_bytes))
    } else {
        ovk.and_then(|ovk| {
            try_output_recovery_with_ovk(
                &zsa_domain,
                ovk,
                &zsa_action,
                zsa_action.cv_net(),
                &zsa_action.encrypted_note().out_ciphertext,
            )
        })
    }
}

/// Fetch `txid` and decrypt what the account can see of it. Reads only the
/// account's keys from the database; see [`DecryptedTx`].
pub async fn decrypt_tx(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    account: u32,
    txid: &[u8],
) -> Result<DecryptedTx> {
    debug!("decrypt_tx {account} {}", hex::encode(txid));
    let (height, tx) = client.transaction(network, txid).await?;

    // Extract raw ZSA enc_ciphertexts before consuming tx with into_data()
    let zsa_raw_ciphertexts = tx.zsa_action_enc_ciphertexts.clone();

    let tx_data = tx.into_data();

    let mut fee_manager = FeeManager::default();
    let mut outputs = vec![];
    let mut memos = vec![];
    // Full or incoming-only keys: an incoming key decrypts what the account
    // received; only a full key also recovers what it sent, through the ovk.
    let skeys = sapling_viewing_keys(connection, account).await?;

    let zip212_enforcement = zip212_enforcement(network, height.into());
    let domain = SaplingDomain::new(zip212_enforcement);

    if let Some(bundle) = tx_data.transparent_bundle() {
        for vin in bundle.vin.iter() {
            fee_manager.add_transparent_input(script_sig_pushes_uncompressed_key(vin.script_sig()));
        }
        for (vout, output) in bundle.vout.iter().enumerate() {
            fee_manager.add_output(0);
            let address = output
                .recipient_address()
                .map(|addr| addr.encode(network))
                .unwrap_or_default();
            outputs.push(DecryptedOutput {
                pool: 0,
                vout: vout as u32,
                value: output.value().into_u64(),
                address,
            });
        }
    }

    if let Some(bundle) = tx_data.sapling_bundle() {
        for _spend in bundle.shielded_spends().iter() {
            fee_manager.add_input(1);
        }
        for _output in bundle.shielded_outputs().iter() {
            fee_manager.add_output(1);
        }

        if let Some(skeys) = skeys.as_ref() {
            let pivk = skeys.external_prepared_ivk();
            let ovk = skeys.external_ovk();
            for (vout, sout) in bundle.shielded_outputs().iter().enumerate() {
                if let Some((note, _address, memo_bytes)) =
                    try_note_decryption(&domain, &pivk, sout)
                {
                    let cmx = note.cmu().to_bytes().to_vec();
                    collect_memo(
                        &mut memos,
                        1,
                        vout as u32,
                        MemoLink::Note { cmx },
                        &memo_bytes,
                    );
                } else if let Some((note, address, memo_bytes)) = ovk.as_ref().and_then(|ovk| {
                    try_output_recovery_with_ovk(
                        &domain,
                        ovk,
                        sout,
                        sout.cv(),
                        sout.out_ciphertext(),
                    )
                }) {
                    outputs.push(DecryptedOutput {
                        pool: 1,
                        vout: vout as u32,
                        value: note.value().inner(),
                        address: address.encode(network),
                    });
                    collect_memo(&mut memos, 1, vout as u32, MemoLink::Output, &memo_bytes);
                }
            }
        }
    }

    let okeys = orchard_viewing_keys(connection, account).await?;

    macro_rules! process_orchard_memo {
        ($bundle:expr, $pool:expr, $domain:ident) => {{
            let bundle = $bundle;
            let pool: u8 = $pool;
            for _action in bundle.actions().iter() {
                fee_manager.add_input(pool);
                fee_manager.add_output(pool);
            }

            if let Some(okeys) = okeys.as_ref() {
                let pivk = okeys.external_prepared_ivk();
                let ovk = okeys.external_ovk();
                for (vout, action) in bundle.actions().iter().enumerate() {
                    let domain = $domain::for_action(action);

                    if let Some((note, _address, memo_bytes)) =
                        try_note_decryption(&domain, &pivk, action)
                    {
                        debug!("decrypt_tx: ivk decrypt ok for vout={vout} pool={pool}");
                        let cmx: ExtractedNoteCommitment = note.commitment().into();
                        // The note row may be missing (e.g. a tx scanned
                        // before its note was stored); the memo is then
                        // stored without a note link, like the sapling
                        // path, instead of failing the whole tx.
                        collect_memo(
                            &mut memos,
                            pool,
                            vout as u32,
                            MemoLink::Note {
                                cmx: cmx.to_bytes().to_vec(),
                            },
                            &memo_bytes,
                        );
                    } else if let Some((note, address, memo_bytes)) = ovk.as_ref().and_then(|ovk| {
                        try_output_recovery_with_ovk(
                            &domain,
                            ovk,
                            action,
                            action.cv_net(),
                            &action.encrypted_note().out_ciphertext,
                        )
                    }) {
                        let address =
                            UnifiedAddress::from_receivers(Some(address), None, None, None, None).unwrap();
                        outputs.push(DecryptedOutput {
                            pool,
                            vout: vout as u32,
                            value: note.value().inner(),
                            address: address.encode(network),
                        });
                        collect_memo(&mut memos, pool, vout as u32, MemoLink::Output, &memo_bytes);
                    } else if pool == 2 && vout < zsa_raw_ciphertexts.len() {
                        // Try ZSA decryption with the original 612-byte enc_ciphertext
                        if let Some((note, address, memo_bytes)) = try_zsa_decrypt(
                            action,
                            &zsa_raw_ciphertexts[vout],
                            &pivk,
                            ovk.as_ref(),
                        ) {
                            let cmx: ExtractedNoteCommitment = note.commitment().into();
                            let address = UnifiedAddress::from_receivers(
                                Some(address), None, None, None, None)
                            .unwrap();
                            collect_memo(
                                &mut memos,
                                pool,
                                vout as u32,
                                MemoLink::NoteOrOutput {
                                    cmx: cmx.to_bytes().to_vec(),
                                    value: note.value().inner(),
                                    address: address.encode(network),
                                },
                                &memo_bytes,
                            );
                        }
                    } else {
                        debug!(
                            "decrypt_tx: both ivk and ovk decrypt failed for vout={vout} pool={pool}"
                        );
                    }
                }
            }
        }};
    }

    if let Some(bundle) = tx_data.orchard_bundle() {
        match bundle {
            OrchardBundle::OrchardVanilla(b) => {
                process_orchard_memo!(b, 2, OrchardDomain);
            }
            OrchardBundle::OrchardZSA(_) => {}
        }
    }
    if let Some(bundle) = tx_data.ironwood_bundle() {
        debug!(
            "decrypt_tx: ironwood bundle with {} actions",
            bundle.actions().len()
        );
        process_orchard_memo!(bundle, 3, IronwoodDomain);
    } else {
        debug!("decrypt_tx: no ironwood bundle in tx {}", hex::encode(txid));
    }

    Ok(DecryptedTx {
        txid: txid.to_vec(),
        height,
        fee: fee_manager.fee(),
        outputs,
        memos,
    })
}

/// Keep a memo worth storing: one that parses and is not empty.
fn collect_memo(
    memos: &mut Vec<DecryptedMemo>,
    pool: u8,
    vout: u32,
    link: MemoLink,
    memo_bytes: &[u8],
) {
    debug!("memo bytes: {}", hex::encode(&memo_bytes[0..32]));
    if let Ok(memo) = Memo::from_bytes(memo_bytes) {
        let text = match &memo {
            Memo::Empty => return,
            Memo::Text(text_memo) => Some(text_memo.to_string()),
            Memo::Future(_) | Memo::Arbitrary(_) => None,
        };
        memos.push(DecryptedMemo {
            pool,
            vout,
            link,
            text,
            bytes: memo_bytes.to_vec(),
        });
    }
}

/// Store a decrypted transaction's outputs, memos, fee and summary, in one
/// database transaction. Returns `false`, storing nothing, when the account
/// no longer has the transaction: it was rewound since the decryption, and
/// the sync that records it again leaves it for the next detail pass.
pub async fn apply_tx_details(
    connection: &mut Held<'_>,
    account: u32,
    tx: &DecryptedTx,
) -> Result<bool> {
    let Some((id_tx, _)) = chain_log::find_transaction(connection, account, &tx.txid).await? else {
        debug!(
            "details of {} dropped: account {account} no longer has it",
            hex::encode(&tx.txid)
        );
        return Ok(false);
    };
    let mut db_tx = connection.begin().await?;

    let mut output_ids = HashMap::new();
    for output in &tx.outputs {
        let id = store_output(&mut db_tx, account, tx.height, id_tx, output).await?;
        output_ids.insert((output.pool, output.vout), id);
    }

    for memo in &tx.memos {
        let (note, output) = match &memo.link {
            MemoLink::Note { cmx } => (
                chain_log::find_note(&mut db_tx, account, NoteKey::Cmx(cmx))
                    .await?
                    .map(|(id, _)| id),
                None,
            ),
            MemoLink::Output => (None, output_ids.get(&(memo.pool, memo.vout)).copied()),
            MemoLink::NoteOrOutput {
                cmx,
                value,
                address,
            } => match chain_log::find_note(&mut db_tx, account, NoteKey::Cmx(cmx)).await? {
                Some((id_note, _)) => (Some(id_note), None),
                None => {
                    let output = DecryptedOutput {
                        pool: memo.pool,
                        vout: memo.vout,
                        value: *value,
                        address: address.clone(),
                    };
                    let id = store_output(&mut db_tx, account, tx.height, id_tx, &output).await?;
                    (None, Some(id))
                }
            },
        };
        chain_log::record_memo(
            &mut db_tx,
            account,
            &MemoRow {
                id_tx,
                height: tx.height,
                pool: memo.pool,
                vout: memo.vout,
                note,
                output,
                text: memo.text.as_deref(),
                bytes: &memo.bytes,
            },
        )
        .await?;
    }

    chain_log::set_transaction_fee(&mut db_tx, id_tx, tx.fee).await?;
    let (tpe, value, asset_id, zsa_value) = summarize_tx(&mut db_tx, id_tx).await?;
    chain_log::set_transaction_details(
        &mut db_tx,
        id_tx,
        &TransactionDetails {
            tpe,
            value,
            zsa_value,
            asset_id,
        },
    )
    .await?;
    db_tx.commit().await?;
    Ok(true)
}

async fn store_output(
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
    id_tx: u32,
    output: &DecryptedOutput,
) -> Result<u32> {
    chain_log::record_output(
        connection,
        account,
        &OutputRow {
            id_tx,
            height,
            pool: output.pool,
            vout: output.vout,
            value: output.value as i64,
            address: &output.address,
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orchard_to_ironwood_fee_only_transfer_is_migration() {
        assert_eq!(
            self_transfer_type(-10_000, 10_000, false, false, true, true),
            TX_TYPE_MIGRATION
        );
    }

    #[test]
    fn migration_requires_exact_fee_value_and_both_pools() {
        assert_eq!(
            self_transfer_type(-9_999, 10_000, false, false, true, true),
            0
        );
        assert_eq!(
            self_transfer_type(-10_000, 10_000, false, false, true, false),
            0
        );
        assert_eq!(
            self_transfer_type(-10_000, 10_000, false, false, false, true),
            0
        );
    }

    #[test]
    fn existing_transparent_self_transfer_types_are_preserved() {
        assert_eq!(
            self_transfer_type(-10_000, 10_000, true, true, false, false),
            12
        );
    }

    #[tokio::test]
    async fn details_are_stored_against_the_transaction_as_found_or_dropped() -> Result<()> {
        use crate::sync::chain_log::{record_transaction, TransactionRow};
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        crate::db::create_schema(&mut db).await?;
        let id_tx = record_transaction(&mut db, 1, &TransactionRow::chain(&[7], 100, 0))
            .await?
            .unwrap();
        let mut tx = DecryptedTx {
            txid: vec![7],
            height: 100,
            fee: 1_000,
            outputs: vec![DecryptedOutput {
                pool: 2,
                vout: 0,
                value: 5,
                address: "u1".into(),
            }],
            memos: vec![DecryptedMemo {
                pool: 2,
                vout: 0,
                link: MemoLink::Output,
                text: Some("hi".into()),
                bytes: vec![0; 512],
            }],
        };

        let session = crate::sync::session::SyncSession::new();
        let run = session.try_begin().unwrap();
        assert!(apply_tx_details(&mut run.held(&mut db), 1, &tx).await?);
        let (details, fee): (bool, i64) =
            sqlx::query_as("SELECT details, fee FROM transactions WHERE id_tx = ?")
                .bind(id_tx)
                .fetch_one(&mut db)
                .await?;
        assert!(details);
        assert_eq!(fee, 1_000);
        let (memo_tx, output): (u32, Option<u32>) =
            sqlx::query_as("SELECT tx, output FROM memos WHERE account = 1")
                .fetch_one(&mut db)
                .await?;
        assert_eq!(memo_tx, id_tx);
        assert!(
            output.is_some(),
            "the memo hangs off the output recorded with it"
        );

        // Rewound since the decryption: the account no longer has it.
        tx.txid = vec![8];
        assert!(!apply_tx_details(&mut run.held(&mut db), 1, &tx).await?);
        let memos: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM memos")
            .fetch_one(&mut db)
            .await?;
        assert_eq!(memos, 1, "nothing stored for a transaction that is gone");
        Ok(())
    }
}
