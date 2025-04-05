use anyhow::Result;
use flutter_rust_bridge::frb;
use futures::TryStreamExt as _;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use zcash_keys::encoding::AddressCodec as _;
use std::collections::HashMap;

use zcash_primitives::{legacy::TransparentAddress, transaction::Transaction as ZcashTransaction};
use zcash_protocol::consensus::BranchId;

use crate::{
    frb_generated::StreamSink, get_coin, lwd::{BlockId, BlockRange, TransparentAddressBlockFilter}
};

// #[frb]
pub async fn synchronize(progress: StreamSink<SyncProgress>, accounts: Vec<u32>, current_height: u32) -> Result<()> {
    if accounts.is_empty() {
        return Ok(());
    }

    let c = get_coin!();
    let network = c.network;
    let pool = c.get_pool();

    // Get account heights
    let mut account_heights = HashMap::new();
    let mut rows = sqlx::query("SELECT account, transparent FROM sync_heights")
        .map(|row: SqliteRow| {
            let account: u32 = row.get(0);
            let height: u32 = row.get(1);
            (account, height)
        })
        .fetch(pool);

    while let Some((account, height)) = rows.try_next().await? {
        account_heights.insert(account, height);
    }

    // Create a sorted list of unique heights
    let mut unique_heights: Vec<u32> = account_heights.values().cloned().collect();
    unique_heights.sort_unstable();
    unique_heights.dedup();

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

        // Fetch addresses for all accounts in this batch
        // let addresses = connection.call(move |conn| {
        let mut addresses = vec![];
        for account in accounts_to_sync.iter() {
            let mut rows = sqlx::query("
                SELECT t1.address
                FROM transparent_address_accounts t1
                JOIN (
                    SELECT account, scope, MAX(dindex) as max_dindex
                    FROM transparent_address_accounts
                    WHERE account = ?
                    GROUP BY scope
                ) t2 ON t1.account = t2.account AND t1.scope = t2.scope AND t1.dindex = t2.max_dindex
                ORDER BY t1.scope")
                .bind(account)
                .map(|row: SqliteRow| row.get::<String, _>(0))
                .fetch(pool);

            while let Some(address) = rows.try_next().await? {
                // Add the address to the client
                addresses.push((*account, address));
            }
        }

        for (account, address) in addresses.iter() {
            let my_address = TransparentAddress::decode(&network, address)?;
            let mut txs = client
                .get_taddress_txids(TransparentAddressBlockFilter {
                    address: address.clone(),
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
                        let _ = progress.add(SyncProgress {
                            height,
                            time: 0,
                        });
                        branch_id
                    }
                };

                // Parse the transaction
                let transaction = ZcashTransaction::read(&*tx.data, consensus_branch_id)?;

                // tx time is available in the block (not here)
                let (id_tx, ): (u32, ) = sqlx::query_as("INSERT INTO transactions (account, txid, height, time, value) VALUES (?, ?, ?, 0, 0) RETURNING id_tx")
                    .bind(account)
                    .bind(&transaction.txid().as_ref()[..])
                    .bind(height)
                    .fetch_one(&mut *db_tx)
                    .await?;
        
                // Access the transparent bundle part
                if let Some(transparent_bundle) = transaction.transparent_bundle() {
                    // Process the transparent bundle
                    // For example, you might want to extract:
                    // - Input transactions (transparent_bundle.vin)
                    // - Output transactions (transparent_bundle.vout)
                    // - Transaction values
                    // - Addresses

                    // For now, just print some information about the transaction
                    println!("Transaction: {}", transaction.txid());
                    println!("Transparent inputs: {}", transparent_bundle.vin.len());

                    let vins = &transparent_bundle.vin;
                    for vin in vins.iter() {
                        // The "nullifier" of a transparent input is the outpoint
                        let mut nf = vec![];
                        vin.prevout.write(&mut nf)?;

                        let row: Option<(u32, i64)> = sqlx::query_as("SELECT id_note, value FROM notes WHERE account = ?1 AND nullifier = ?2")
                        .bind(account)
                        .bind(&nf)
                        .fetch_optional(&mut *db_tx).await?;

                        if let Some((id, amount)) = row {
                            // note was found
                            // add a spent entry
                            sqlx::query("INSERT INTO spends (account, id_note, pool, tx, height, value) VALUES (?, ?, ?, ?, ?, ?)")
                            .bind(account)
                            .bind(id)
                            .bind(0)
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

                                sqlx::query("INSERT INTO notes (account, height, pool, tx, nullifier, value) VALUES (?, ?, ?, ?, ?, ?)")
                                    .bind(account)
                                    .bind(height)
                                    .bind(0)
                                    .bind(id_tx)
                                    .bind(&nf)
                                    .bind(vout.value.into_u64() as i64)
                                    .execute(&mut *db_tx)
                                    .await?;
                            }
                        }
                    }

                    println!("Transparent outputs: {}", transparent_bundle.vout.len());
                }
            }
            sqlx::query("UPDATE sync_heights SET transparent = ? WHERE account = ?")
            .bind(end_height)
            .bind(account)
            .execute(&mut *db_tx)
            .await?;
            let _ = progress.add(SyncProgress {
                height: end_height,
                time: 0,
            });
            db_tx.commit().await?;
        }

        // Update our local map as well for the next iteration
        for account in &accounts_to_sync {
            account_heights.insert(*account, end_height);
        }
    }

    Ok(())
}

#[frb]
pub async fn balance() -> Result<PoolBalance> {
    let c = get_coin!();
    let pool = c.get_pool();
    let account = c.account;

    let mut balance = PoolBalance {
        balance: vec![0, 0, 0],
    };

    let mut rows = sqlx::query("
    WITH N AS (SELECT value, pool FROM notes WHERE account = ?1 UNION ALL SELECT value, pool FROM spends WHERE account = ?1)
    SELECT pool, SUM(value) FROM N GROUP BY pool")
        .bind(account)
        .map(|row: SqliteRow| (row.get::<u8, _>(0), row.get::<i64, _>(1)))
        .fetch(pool);
    while let Some((pool, value)) = rows.try_next().await? {
        balance.balance[pool as usize] += value as u64;
    }

    Ok(balance)
}

pub struct SyncProgress {
    pub height: u32,
    pub time: u32,
}

pub struct PoolBalance {
    pub balance: Vec<u64>,
}
