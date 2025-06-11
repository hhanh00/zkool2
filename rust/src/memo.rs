use anyhow::{Context as _, Result};
use orchard::{keys::Scope, note::ExtractedNoteCommitment, note_encryption::OrchardDomain};
use sapling_crypto::{keys::PreparedIncomingViewingKey, note_encryption::SaplingDomain};
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use tonic::Request;
use tracing::info;
use zcash_keys::{address::UnifiedAddress, encoding::AddressCodec};
use zcash_note_encryption::{try_note_decryption, try_output_recovery_with_ovk};
use zcash_primitives::transaction::{
    components::sapling::zip212_enforcement, fees::transparent::OutputView, Transaction,
};
use zcash_protocol::{
    consensus::{BlockHeight, BranchId, Network},
    memo::Memo,
};

use crate::{
    account::{get_orchard_vk, get_sapling_vk},
    lwd::TxFilter,
    pay::fee::FeeManager,
    Client,
};

pub async fn fetch_tx_details(
    network: &Network,
    connection: &SqlitePool,
    client: &mut Client,
    account: u32,
) -> Result<()> {
    info!("fetch_tx_details");
    let txids =
        sqlx::query("SELECT id_tx, txid FROM transactions WHERE account = ? AND details = FALSE")
            .bind(account)
            .map(|row: SqliteRow| {
                let id_tx: u32 = row.get(0);
                let txid: Vec<u8> = row.get(1);
                (id_tx, txid)
            })
            .fetch_all(connection)
            .await?;

    for (id_tx, txid) in txids.iter() {
        decrypt_memo(network, connection, client, account, &txid).await?;
        let (tpe, value) = summarize_tx(connection, *id_tx).await?;
        sqlx::query("UPDATE transactions SET details = TRUE, tpe = ?, value = ? WHERE id_tx = ?")
            .bind(tpe)
            .bind(value)
            .bind(*id_tx)
            .execute(connection)
            .await?;
    }

    Ok(())
}

async fn summarize_tx(connection: &SqlitePool, tx: u32) -> Result<(u8, i64)> {
    let (value, fee) = sqlx::query(
        "WITH n AS (SELECT value, tx FROM notes UNION ALL SELECT value, tx FROM spends)
        SELECT SUM(n.value), t.fee FROM n JOIN transactions t ON t.id_tx = n.tx WHERE n.tx = ?",
    )
    .bind(tx)
    .map(|row: SqliteRow| {
        let value = row.get::<Option<i64>, _>(0).unwrap_or_default();
        let fee = row.get::<Option<i64>, _>(1).unwrap_or_default();
        (value, fee)
    })
    .fetch_one(connection)
    .await?;
    if value > 0 {
        // receiving
        return Ok((1, value));
    } else if value < -fee {
        // sending
        return Ok((2, value));
    } else {
        // self transfer
        let has_tspend = sqlx::query("SELECT 1 FROM spends WHERE tx = ? AND pool = 0")
            .bind(tx)
            .fetch_optional(connection)
            .await?
            .is_some();
        let has_tnote = sqlx::query("SELECT 1 FROM notes WHERE tx = ? AND pool = 0")
            .bind(tx)
            .fetch_optional(connection)
            .await?
            .is_some();
        let tpe: u8 = (if has_tspend { 8 } else { 0 }) | (if has_tnote { 4 } else { 0 });
        return Ok((tpe, value));
    }
}

pub async fn decrypt_memo(
    network: &Network,
    connection: &SqlitePool,
    client: &mut Client,
    account: u32,
    txid: &[u8],
) -> Result<()> {
    info!("decrypt_memo {account} {}", hex::encode(txid));
    let txid = txid.to_vec();
    let raw_tx = client
        .get_transaction(Request::new(TxFilter {
            block: None,
            index: 0,
            hash: txid.clone(),
        }))
        .await?
        .into_inner();

    let data = &*raw_tx.data;
    let height = raw_tx.height as u32;
    let branch_id = BranchId::for_height(network, BlockHeight::from_u32(height));
    let tx = Transaction::read(data, branch_id)?;
    let tx_data = tx.into_data();

    let (id_tx,): (u32,) =
        sqlx::query_as("SELECT id_tx FROM transactions WHERE account = ? AND txid = ?")
            .bind(account)
            .bind(&txid)
            .fetch_one(connection)
            .await
            .context("Failed to find transaction")?;

    let mut fee_manager = FeeManager::default();
    let svk = get_sapling_vk(connection, account).await?;

    let zip212_enforcement = zip212_enforcement(network, height.into());
    let domain = SaplingDomain::new(zip212_enforcement);

    if let Some(bundle) = tx_data.transparent_bundle() {
        for _vin in bundle.vin.iter() {
            fee_manager.add_input(0);
        }
        for (vout, output) in bundle.vout.iter().enumerate() {
            fee_manager.add_output(0);
            let address = output
                .recipient_address()
                .map(|addr| addr.encode(network))
                .unwrap_or_default();
            store_output(
                connection,
                account,
                height,
                id_tx,
                0, // Transparent pool
                vout as u32,
                output.value().into_u64(),
                &address,
            )
            .await?;
        }
    }

    if let Some(bundle) = tx_data.sapling_bundle() {
        for _spend in bundle.shielded_spends().iter() {
            fee_manager.add_input(1);
        }
        for _output in bundle.shielded_outputs().iter() {
            fee_manager.add_output(1);
        }

        if let Some(svk) = svk.as_ref() {
            let pivk = PreparedIncomingViewingKey::new(&svk.fvk().vk.ivk());
            let ovk = &svk.fvk().ovk;
            for (vout, sout) in bundle.shielded_outputs().iter().enumerate() {
                if let Some((note, _address, memo_bytes)) =
                    try_note_decryption(&domain, &pivk, sout)
                {
                    let cmx = &note.cmu().to_bytes();
                    let id_note =
                        sqlx::query("SELECT id_note FROM notes WHERE account = ? AND cmx = ?")
                            .bind(account)
                            .bind(cmx.as_slice())
                            .map(|row: SqliteRow| row.get::<u32, _>(0))
                            .fetch_optional(connection)
                            .await?;

                    process_memo(
                        connection,
                        account,
                        height,
                        id_tx,
                        id_note,
                        None,
                        1,
                        vout as u32,
                        &memo_bytes,
                    )
                    .await?;
                } else if let Some((note, address, memo_bytes)) = try_output_recovery_with_ovk(
                    &domain,
                    ovk,
                    sout,
                    sout.cv(),
                    sout.out_ciphertext(),
                ) {
                    let address = address.encode(network);
                    let id_output = store_output(
                        connection,
                        account,
                        height,
                        id_tx,
                        1, // Sapling pool
                        vout as u32,
                        note.value().inner(),
                        &address,
                    )
                    .await?;

                    process_memo(
                        connection,
                        account,
                        height,
                        id_tx,
                        None,
                        Some(id_output),
                        1,
                        vout as u32,
                        &memo_bytes,
                    )
                    .await?;
                }
            }
        }
    }

    let ovk = get_orchard_vk(connection, account).await?;

    if let Some(bundle) = tx_data.orchard_bundle() {
        for _action in bundle.actions().iter() {
            fee_manager.add_input(2);
            fee_manager.add_output(2);
        }

        if let Some(ovk) = ovk.as_ref() {
            let pivk = orchard::keys::PreparedIncomingViewingKey::new(&ovk.to_ivk(Scope::External));
            let ovk = ovk.to_ovk(Scope::External);
            for (vout, action) in bundle.actions().iter().enumerate() {
                let domain = OrchardDomain::for_action(action);

                if let Some((note, _address, memo_bytes)) =
                    try_note_decryption(&domain, &pivk, action)
                {
                    let cmx: ExtractedNoteCommitment = note.commitment().into();
                    let id_note =
                        sqlx::query("SELECT id_note FROM notes WHERE account = ? AND cmx = ?")
                            .bind(account)
                            .bind(&cmx.to_bytes()[..])
                            .map(|row: SqliteRow| row.get::<u32, _>(0))
                            .fetch_one(connection)
                            .await
                            .context("Failed to find note")?;

                    process_memo(
                        connection,
                        account,
                        height,
                        id_tx,
                        Some(id_note),
                        None,
                        2,
                        vout as u32,
                        &memo_bytes,
                    )
                    .await?;
                } else if let Some((note, address, memo_bytes)) = try_output_recovery_with_ovk(
                    &domain,
                    &ovk,
                    action,
                    action.cv_net(),
                    &action.encrypted_note().out_ciphertext,
                ) {
                    let address =
                        UnifiedAddress::from_receivers(Some(address), None, None).unwrap();
                    let id_output = store_output(
                        connection,
                        account,
                        height,
                        id_tx,
                        2, // Orchard pool
                        vout as u32,
                        note.value().inner(),
                        &address.encode(network),
                    )
                    .await?;

                    process_memo(
                        connection,
                        account,
                        height,
                        id_tx,
                        None,
                        Some(id_output),
                        2,
                        vout as u32,
                        &memo_bytes,
                    )
                    .await?;
                }
            }
        }
    }

    let fee = fee_manager.fee();
    sqlx::query("UPDATE transactions SET fee = ? WHERE id_tx = ?")
        .bind(fee as i64)
        .bind(id_tx)
        .execute(connection)
        .await?;

    Ok(())
}

async fn process_memo(
    connection: &SqlitePool,
    account: u32,
    height: u32,
    id_tx: u32,
    id_note: Option<u32>,
    id_output: Option<u32>,
    pool: u8,
    vout: u32,
    memo_bytes: &[u8],
) -> Result<()> {
    info!("memo bytes: {}", hex::encode(&memo_bytes[0..32]));
    if let Ok(memo) = Memo::from_bytes(&memo_bytes) {
        match memo {
            Memo::Empty => {}
            Memo::Text(text_memo) => {
                let text = &*text_memo;
                sqlx::query(
                    "INSERT INTO memos
                (account, height, tx, pool, vout, note, output, memo_text, memo_bytes)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT DO NOTHING",
                )
                .bind(account)
                .bind(height)
                .bind(id_tx)
                .bind(pool)
                .bind(vout as u32)
                .bind(id_note)
                .bind(id_output)
                .bind(text)
                .bind(&memo_bytes[..])
                .execute(connection)
                .await?;
            }
            Memo::Future(_) | Memo::Arbitrary(_) => {
                sqlx::query(
                    "INSERT INTO memos
                (account, height, tx, pool, vout, note, output, memo_bytes)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT DO NOTHING",
                )
                .bind(account)
                .bind(height)
                .bind(id_tx)
                .bind(pool)
                .bind(vout as u32)
                .bind(id_note)
                .bind(id_output)
                .bind(&memo_bytes[..])
                .execute(connection)
                .await?;
            }
        }
    }

    Ok(())
}

async fn store_output(
    connection: &SqlitePool,
    account: u32,
    height: u32,
    id_tx: u32,
    pool: u8,
    vout: u32,
    value: u64,
    address: &str,
) -> Result<u32> {
    sqlx::query(
        "INSERT INTO outputs
        (account, height, tx, pool, vout, value, address)
        VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT DO NOTHING",
    )
    .bind(account)
    .bind(height)
    .bind(id_tx)
    .bind(pool) // Sapling pool
    .bind(vout as u32)
    .bind(value as i64)
    .bind(&address)
    .execute(connection)
    .await?;
    let id_output =
        sqlx::query("SELECT id_output FROM outputs WHERE tx = ? AND pool = ? AND vout = ?")
            .bind(id_tx)
            .bind(pool) // Sapling pool
            .bind(vout as u32)
            .map(|row: SqliteRow| row.get::<u32, _>(0))
            .fetch_one(connection)
            .await
            .context("Failed to find output")?;

    Ok(id_output)
}
