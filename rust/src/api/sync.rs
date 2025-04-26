use anyhow::Result;
use flutter_rust_bridge::frb;
use futures::TryStreamExt as _;
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use std::collections::HashMap;
use tokio::sync::mpsc::channel;
use tracing::info;
use zcash_keys::encoding::AddressCodec as _;

use zcash_primitives::{legacy::TransparentAddress, transaction::Transaction as ZcashTransaction};
use zcash_protocol::consensus::{BranchId, Network};

use crate::db::calculate_balance;
use crate::sync::{prune_old_checkpoints, recover_from_partial_sync};
use crate::Client;
use crate::{
    frb_generated::StreamSink,
    get_coin,
    lwd::{BlockId, BlockRange, TransparentAddressBlockFilter},
    sync::shielded_sync,
};

#[frb]
pub async fn synchronize(
    progress: StreamSink<SyncProgress>,
    accounts: Vec<u32>,
    current_height: u32,
    actions_per_sync: u32,
    transparent_limit: u32,
    checkpoint_age: u32,
) -> Result<()> {
    if accounts.is_empty() {
        return Ok(());
    }

    let c = get_coin!();
    let network = c.network;
    let pool = c.get_pool();
    let progress2 = progress.clone();

    let checkpoint_cutoff = current_height - checkpoint_age;
    for account in accounts.iter() {
        prune_old_checkpoints(pool, *account, checkpoint_cutoff).await?;
    }

    let mut account_use_internal = HashMap::<u32, bool>::new();
    let res = async {
        recover_from_partial_sync(&pool, &accounts).await?;

        // Get account heights
        let mut account_heights = HashMap::new();
        for account in accounts.iter() {
            let (account, height): (u32, u32) = sqlx::query_as(
                "SELECT account, MIN(height) FROM sync_heights
        JOIN accounts ON account = id_account
        WHERE account = ?",
            )
            .bind(account)
            .fetch_one(pool)
            .await?;

            account_heights.insert(account, height + 1);

            let (use_internal,): (bool,) =
                sqlx::query_as("SELECT use_internal FROM accounts WHERE id_account = ?")
                    .bind(account)
                    .fetch_one(pool)
                    .await?;
            account_use_internal.insert(account, use_internal);
        }

        // Create a sorted list of unique heights
        let mut unique_heights: Vec<u32> = account_heights.values().cloned().collect();
        unique_heights.sort_unstable();
        unique_heights.dedup();

        let (tx_progress, mut rx_progress) = channel::<SyncProgress>(1);

        tokio::spawn(async move {
            while let Some(p) = rx_progress.recv().await {
                let _ = progress.add(p);
            }
        });

        // For each unique height, process accounts that need to be synced from that height
        for (i, &start_height) in unique_heights.iter().enumerate() {
            // Determine the end height (next height - 1 or current_height)
            let end_height = if i + 1 < unique_heights.len() {
                unique_heights[i + 1] - 1
            } else {
                current_height
            };

            // Find accounts that have a height <= this start_height
            let accounts_to_sync = account_heights
                .iter()
                .filter(|&(_, &height)| height <= start_height)
                .map(|(&account, _)| {
                    let use_internal = account_use_internal[&account];
                    (account, use_internal)
                })
                .collect::<Vec<_>>();

            // Skip if no accounts to sync
            if accounts_to_sync.is_empty() {
                continue;
            }

            // Update the sync heights for these accounts
            let mut client = c.client().await?;

            let account_ids = accounts_to_sync.iter().map(|(account, _)| *account).collect::<Vec<_>>();
            transparent_sync(
                &network,
                pool,
                &mut client,
                &account_ids,
                start_height,
                end_height,
                transparent_limit,
            )
            .await?;

            shielded_sync(
                &network,
                pool,
                &mut client,
                &accounts_to_sync,
                start_height,
                end_height,
                actions_per_sync,
                tx_progress.clone(),
            )
            .await?;

            // Update our local map as well for the next iteration
            for (account, _) in &accounts_to_sync {
                account_heights.insert(*account, end_height);
                crate::memo::fetch_tx_details(&network, pool, &mut client, *account).await?;
            }
        }

        Ok::<_, anyhow::Error>(())
    };

    match res.await {
        Ok(_) => {}
        Err(e) => {
            info!("Error during sync: {:?}", e);
            let _ = progress2.add_error(e);
        }
    }

    Ok(())
}

pub(crate) async fn transparent_sync(
    network: &Network,
    pool: &SqlitePool,
    client: &mut Client,
    accounts: &[u32],
    start_height: u32,
    end_height: u32,
    limit: u32,
) -> Result<()> {
    let mut addresses = vec![];
    for account in accounts {
        // scan latest 5 receive and change addresses
        let mut rows = sqlx::query("
                WITH receive AS
                (SELECT * FROM transparent_address_accounts WHERE account = ?1 AND scope = 0 ORDER BY dindex DESC LIMIT ?2),
                change AS
                (SELECT * FROM transparent_address_accounts WHERE account = ?1 AND scope = 1 ORDER BY dindex DESC LIMIT ?2)
                SELECT id_taddress, address FROM receive UNION SELECT id_taddress, address FROM change")
            .bind(account)
            .bind(limit)
            .map(|row: SqliteRow| {
                let id_taddress: u32 = row.get(0);
                let address: String = row.get(1);
                (id_taddress, address)
            })
            .fetch(pool);

        while let Some((id_taddress, address)) = rows.try_next().await? {
            // Add the address to the client
            addresses.push((*account, (id_taddress, address)));
        }
    }
    Ok(for (account, address_row) in addresses.iter() {
        let my_address = TransparentAddress::decode(&network, &address_row.1)?;
        let mut txs = client
            .get_taddress_txids(TransparentAddressBlockFilter {
                address: address_row.1.clone(),
                range: Some(BlockRange {
                    start: Some(BlockId {
                        height: start_height as u64,
                        hash: vec![],
                    }),
                    end: Some(BlockId {
                        height: end_height as u64,
                        hash: vec![],
                    }),
                    spam_filter_threshold: 0,
                }),
            })
            .await?
            .into_inner();

        // Initialize variables for branch ID memoization
        let mut last_height: Option<u32> = None;
        let mut last_branch_id: Option<BranchId> = None;

        let mut db_tx = pool.begin().await?;
        while let Some(tx) = txs.message().await? {
            // Determine the consensus branch ID based on the block height
            let height = tx.height as u32;

            // Use memoized branch ID if available for this height, otherwise compute it
            let consensus_branch_id = match last_height {
                Some(h) if h == height => last_branch_id.unwrap(),
                _ => {
                    let branch_id = BranchId::for_height(&network, height.into());
                    last_height = Some(height);
                    last_branch_id = Some(branch_id);
                    branch_id
                }
            };

            // Parse the transaction
            let transaction = ZcashTransaction::read(&*tx.data, consensus_branch_id)?;

            let txid = transaction.txid().as_ref().to_vec();
            // tx time is available in the block (not here)
            sqlx::query("INSERT INTO transactions (account, txid, height, time) VALUES (?, ?, ?, 0) ON CONFLICT DO NOTHING")
                .bind(account)
                .bind(&txid)
                .bind(height)
                .execute(&mut *db_tx)
                .await?;

            // Access the transparent bundle part
            if let Some(transparent_bundle) = transaction.transparent_bundle() {
                info!("Transaction: {}", transaction.txid());
                info!("Transparent inputs: {}", transparent_bundle.vin.len());

                let vins = &transparent_bundle.vin;
                for vin in vins.iter() {
                    // The "nullifier" of a transparent input is the outpoint
                    let mut nf = vec![];
                    vin.prevout.write(&mut nf)?;

                    let row: Option<(u32, i64)> = sqlx::query_as(
                        "SELECT id_note, value FROM notes WHERE account = ?1 AND nullifier = ?2",
                    )
                    .bind(account)
                    .bind(&nf)
                    .fetch_optional(&mut *db_tx)
                    .await?;

                    if let Some((id, amount)) = row {
                        // note was found
                        // add a spent entry
                        sqlx::query(
                            "INSERT INTO spends (account, id_note, pool, tx, height, value)
                        SELECT ?, ?, 0, tx.id_tx, ?, ? FROM transactions tx WHERE tx.txid = ?
                        AND account = ? ON CONFLICT DO NOTHING",
                        )
                        .bind(account)
                        .bind(id)
                        .bind(height)
                        .bind(-amount)
                        .bind(&txid)
                        .bind(account)
                        .execute(&mut *db_tx)
                        .await?;
                    }
                }

                let vouts = &transparent_bundle.vout;
                for (i, vout) in vouts.iter().enumerate() {
                    if let Some(address) = vout.recipient_address() {
                        if address == my_address {
                            // It is for me
                            // add a new note entry
                            let mut nf = transaction.txid().as_ref().to_vec();
                            nf.extend_from_slice(&(i as u32).to_le_bytes());

                            sqlx::query("INSERT INTO notes (account, height, pool, tx, taddress, nullifier, value)
                            SELECT ?, ?, 0, tx.id_tx, ?, ?, ? FROM transactions tx WHERE tx.txid = ?
                            AND account = ? ON CONFLICT DO NOTHING")
                                .bind(account)
                                .bind(height)
                                .bind(address_row.0)
                                .bind(&nf)
                                .bind(vout.value.into_u64() as i64)
                                .bind(&txid)
                                .bind(account)
                                .execute(&mut *db_tx)
                                .await?;
                        }
                    }
                }

                info!("Transparent outputs: {}", transparent_bundle.vout.len());
            }
        }
        sqlx::query("UPDATE sync_heights SET height = ? WHERE account = ? AND pool = 0")
            .bind(end_height)
            .bind(account)
            .execute(&mut *db_tx)
            .await?;
        db_tx.commit().await?;
    })
}

#[frb]
pub async fn balance() -> Result<PoolBalance> {
    let c = get_coin!();
    let pool = c.get_pool();
    let account = c.account;

    calculate_balance(pool, account).await
}

#[frb]
pub async fn rewind_sync(height: u32) -> Result<()> {
    let c = get_coin!();
    let connection = c.get_pool();
    crate::sync::rewind_sync(connection, c.account, height).await
}

#[frb]
pub async fn get_db_height() -> Result<u32> {
    let c = get_coin!();
    let connection = c.get_pool();
    crate::sync::get_db_height(connection, c.account).await
}

#[frb]
pub async fn get_tx_details() -> Result<()> {
    let c = get_coin!();
    let connection = c.get_pool();
    let mut client = c.client().await?;
    crate::memo::fetch_tx_details(&c.network, connection, &mut client, c.account).await?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct SyncProgress {
    pub height: u32,
    pub time: u32,
}

pub struct PoolBalance(pub Vec<u64>);
