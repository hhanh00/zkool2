use anyhow::Result;
use flutter_rust_bridge::frb;
use futures::TryStreamExt as _;
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use tracing::info;
use std::collections::HashMap;
use tokio::sync::mpsc::channel;
use zcash_keys::encoding::AddressCodec as _;

use zcash_primitives::{legacy::TransparentAddress, transaction::Transaction as ZcashTransaction};
use zcash_protocol::consensus::{BranchId, Network};

use crate::db::calculate_balance;
use crate::sync::recover_from_partial_sync;
use crate::Client;
use crate::{
    frb_generated::StreamSink,
    get_coin,
    lwd::{BlockId, BlockRange, TransparentAddressBlockFilter},
    sync::shielded_sync,
};

pub async fn synchronize(
    progress: StreamSink<SyncProgress>,
    accounts: Vec<u32>,
    current_height: u32,
) -> Result<()> {
    if accounts.is_empty() {
        return Ok(());
    }

    let c = get_coin!();
    let network = c.network;
    let pool = c.get_pool();

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

        // Find accounts that have a transparent height <= this start_height
        let accounts_to_sync: Vec<u32> = account_heights
            .iter()
            .filter(|&(_, &height)| height <= start_height)
            .map(|(&account, _)| account)
            .collect();

        // Skip if no accounts to sync
        if accounts_to_sync.is_empty() {
            continue;
        }

        // Update the sync heights for these accounts
        let mut client = c.client().await?;

        transparent_sync(
            &network,
            pool,
            start_height,
            end_height,
            &accounts_to_sync,
            &mut client,
        )
        .await?;

        shielded_sync(
            &network,
            pool,
            &mut client,
            accounts.clone(),
            start_height,
            end_height,
            tx_progress.clone(),
        )
        .await?;

        // Update our local map as well for the next iteration
        for account in &accounts_to_sync {
            account_heights.insert(*account, end_height);
        }
    }

    Ok(())
}

async fn transparent_sync(
    network: &Network,
    pool: &SqlitePool,
    start_height: u32,
    end_height: u32,
    accounts: &Vec<u32>,
    client: &mut Client,
) -> Result<()> {
    let mut addresses = vec![];
    for account in accounts.iter() {
        // scan latest 5 receive and change addresses
        let mut rows = sqlx::query("
                WITH receive AS
                (SELECT * FROM transparent_address_accounts WHERE account = ?1 AND scope = 0 ORDER BY dindex DESC LIMIT 5),
                change AS
                (SELECT * FROM transparent_address_accounts WHERE account = ?1 AND scope = 1 ORDER BY dindex DESC LIMIT 5)
                SELECT id_taddress, address FROM receive UNION SELECT id_taddress, address FROM change")
            .bind(account)
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

            // tx time is available in the block (not here)
            let (id_tx, ): (u32, ) = sqlx::query_as("INSERT INTO transactions (account, txid, height, time) VALUES (?, ?, ?, 0) RETURNING id_tx")
                .bind(account)
                .bind(&transaction.txid().as_ref()[..])
                .bind(height)
                .fetch_one(&mut *db_tx)
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
                        sqlx::query("INSERT INTO spends (account, id_note, pool, tx, height, value) VALUES (?, ?, 0, ?, ?, ?)")
                        .bind(account)
                        .bind(id)
                        .bind(id_tx)
                        .bind(height)
                        .bind(-amount)
                        .execute(&mut *db_tx).await?;
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

                            let r = sqlx::query("INSERT INTO notes (account, height, pool, tx, taddress, nullifier, value) VALUES (?, ?, 0, ?, ?, ?, ?)")
                                .bind(account)
                                .bind(height)
                                .bind(id_tx)
                                .bind(address_row.0)
                                .bind(&nf)
                                .bind(vout.value.into_u64() as i64)
                                .execute(&mut *db_tx)
                                .await?;
                            assert_eq!(r.rows_affected(), 1);
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
    crate::sync::rewind_sync(connection, height).await
}

#[frb]
pub async fn get_db_height(account: u32) -> Result<u32> {
    let c = get_coin!();
    let connection = c.get_pool();
    crate::sync::get_db_height(connection, account).await
}

#[frb]
pub async fn get_tx_details() -> Result<()> {
    let c = get_coin!();
    let connection = c.get_pool();
    let mut client = c.client().await?;
    crate::memo::fetch_tx_details(&c.network, connection, &mut client, c.account).await?;
    Ok(())
}

pub struct SyncProgress {
    pub height: u32,
    pub time: u32,
}

pub struct PoolBalance(pub Vec<u64>);
