use anyhow::Result;
use futures::TryStreamExt as _;
use sqlx::sqlite::SqliteRow;
use std::collections::HashMap;
use sqlx::Row;

use zcash_primitives::transaction::Transaction as ZcashTransaction;
use zcash_protocol::consensus::BranchId;

use crate::{
    get_coin,
    lwd::{BlockId, BlockRange, TransparentAddressBlockFilter},
};

// #[frb]
pub async fn get_transparent_transactions(accounts: Vec<u32>, current_height: u32) -> Result<()> {
    if accounts.is_empty() {
        return Ok(());
    }

    let c = get_coin!();
    let network = c.network;

    // Get account heights
    let mut account_heights = HashMap::new();
    let mut rows = sqlx::query("SELECT account, transparent FROM sync_heights")
    .map(|row: SqliteRow| {
        let account: u32 = row.get(0);
        let height: u32 = row.get(1);
        (account, height)
    })
    .fetch(c.get_pool());

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
                .fetch(c.get_pool());

            while let Some(address) = rows.try_next().await? {
                // Add the address to the client
                addresses.push((*account, address));
            }
        }

        for (_account, address) in addresses.iter() {
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

                    // TODO: Look up the inputs in the table utxos
                    // resolve the txin to a outpoint
                    // add a tspend entry

                    println!("Transparent outputs: {}", transparent_bundle.vout.len());
                    // add a utxo entry
                }
            }
        }
        
        // Update the sync heights for these accounts
        let accounts_to_sync_clone = accounts_to_sync.clone();
        let end_height_clone = end_height;
        for account in &accounts_to_sync_clone {
            sqlx::query(
                "UPDATE sync_heights SET transparent = ? WHERE account = ?")
                .bind(end_height_clone)
                .bind(account)
                .execute(c.get_pool())
                .await?;
        }
        
        // Update our local map as well for the next iteration
        for account in &accounts_to_sync {
            account_heights.insert(*account, end_height);
        }
    }

    Ok(())
}
