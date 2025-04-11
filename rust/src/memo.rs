use anyhow::Result;
use orchard::{keys::Scope, note::ExtractedNoteCommitment, note_encryption::OrchardDomain};
use sapling_crypto::{keys::PreparedIncomingViewingKey, note_encryption::SaplingDomain};
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use tonic::Request;
use zcash_note_encryption::{try_note_decryption, try_output_recovery_with_ovk};
use zcash_primitives::transaction::{components::sapling::zip212_enforcement, Transaction};
use zcash_protocol::{
    consensus::{BlockHeight, BranchId, Network},
    memo::Memo,
};

use crate::{
    account::{get_orchard_vk, get_sapling_vk},
    lwd::TxFilter,
    Client,
};

pub async fn fetch_tx_details(
    network: &Network,
    connection: &SqlitePool,
    client: &mut Client,
    account: u32,
) -> Result<()> {
    let txids = sqlx::query("SELECT txid FROM transactions WHERE account = ? AND details = FALSE")
        .bind(account)
        .map(|row: SqliteRow| row.get::<Vec<u8>, _>(0))
        .fetch_all(connection)
        .await?;

    for txid in txids.iter() {
        decrypt_memo(network, connection, client, account, &txid).await?;
    }

    sqlx::query("UPDATE transactions SET details = TRUE WHERE account = ?")
        .bind(account)
        .execute(connection)
        .await?;

    Ok(())
}

pub async fn decrypt_memo(
    network: &Network,
    connection: &SqlitePool,
    client: &mut Client,
    account: u32,
    txid: &[u8],
) -> Result<()> {
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

    println!("Transaction parsed successfully");

    let (id_tx, ): (u32,) =
        sqlx::query_as("SELECT id_tx FROM transactions WHERE account = ? AND txid = ?")
            .bind(account)
            .bind(&txid)
            .fetch_one(connection)
            .await?;

    println!("Transaction ID: {}", id_tx);

    let svk = get_sapling_vk(connection, account).await?;

    let zip212_enforcement = zip212_enforcement(network, height.into());
    let domain = SaplingDomain::new(zip212_enforcement);

    if let Some(bundle) = tx_data.sapling_bundle() {
        if let Some(svk) = svk.as_ref() {
            let pivk = PreparedIncomingViewingKey::new(&svk.vk.ivk());
            let ovk = &svk.ovk;
            for (vout, sout) in bundle.shielded_outputs().iter().enumerate() {
                if let Some((note, _address, memo_bytes)) =
                    try_note_decryption(&domain, &pivk, sout)
                {
                    let cmx = &note.cmu().to_bytes();
                    process_memo(
                        connection,
                        account,
                        height,
                        id_tx,
                        1,
                        vout as u32,
                        cmx,
                        &memo_bytes,
                    )
                    .await?;
                }

                if let Some((note, _address, memo_bytes)) = try_output_recovery_with_ovk(
                    &domain,
                    ovk,
                    sout,
                    sout.cv(),
                    sout.out_ciphertext(),
                ) {
                    let cmx = &note.cmu().to_bytes();
                    process_memo(
                        connection,
                        account,
                        height,
                        id_tx,
                        1,
                        vout as u32,
                        cmx,
                        &memo_bytes,
                    )
                    .await?;
                }
            }
        }
    }

    let ovk = get_orchard_vk(connection, account).await?;

    if let Some(bundle) = tx_data.orchard_bundle() {
        if let Some(ovk) = ovk.as_ref() {
            let pivk = orchard::keys::PreparedIncomingViewingKey::new(&ovk.to_ivk(Scope::External));
            let ovk = ovk.to_ovk(Scope::External);
            for (vout, action) in bundle.actions().iter().enumerate() {
                let domain = OrchardDomain::for_action(action);

                if let Some((note, _address, memo_bytes)) =
                    try_note_decryption(&domain, &pivk, action)
                {
                    let cmx = note.commitment();
                    let cmx = ExtractedNoteCommitment::from(cmx);
                    process_memo(
                        connection,
                        account,
                        height,
                        id_tx,
                        2,
                        vout as u32,
                        &cmx.to_bytes(),
                        &memo_bytes,
                    )
                    .await?;
                }

                if let Some((note, _address, memo_bytes)) = try_output_recovery_with_ovk(
                    &domain,
                    &ovk,
                    action,
                    action.cv_net(),
                    &action.encrypted_note().out_ciphertext,
                ) {
                    let cmx = note.commitment();
                    let cmx = ExtractedNoteCommitment::from(cmx);
                    process_memo(
                        connection,
                        account,
                        height,
                        id_tx,
                        2,
                        vout as u32,
                        &cmx.to_bytes(),
                        &memo_bytes,
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}

async fn process_memo(
    connection: &SqlitePool,
    account: u32,
    height: u32,
    id_tx: u32,
    pool: u8,
    vout: u32,
    cmx: &[u8],
    memo_bytes: &[u8],
) -> Result<()> {
    if let Ok(memo) = Memo::from_bytes(&memo_bytes) {
        let (id_note,): (u32,) =
            sqlx::query_as("SELECT id_note FROM notes WHERE account = ? AND cmx = ?")
                .bind(account)
                .bind(cmx)
                .fetch_one(connection)
                .await?;

        match memo {
            Memo::Empty => {}
            Memo::Text(text_memo) => {
                let text = &*text_memo;
                sqlx::query(
                    "INSERT INTO memos
                (account, height, tx, pool, vout, note, memo_text, memo_bytes)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT DO NOTHING",
                )
                .bind(account)
                .bind(height)
                .bind(id_tx)
                .bind(pool)
                .bind(vout as u32)
                .bind(id_note)
                .bind(text)
                .bind(&memo_bytes[..])
                .execute(connection)
                .await?;
            }
            Memo::Future(_) | Memo::Arbitrary(_) => {
                sqlx::query(
                    "INSERT INTO memos
                (account, height, tx, pool, vout, note, memo_bytes)
                VALUES (?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(account)
                .bind(height)
                .bind(id_tx)
                .bind(pool)
                .bind(vout as u32)
                .bind(id_note)
                .bind(&memo_bytes[..])
                .execute(connection)
                .await?;
            }
        }
    }

    Ok(())
}
