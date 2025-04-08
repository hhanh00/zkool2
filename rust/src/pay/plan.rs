use super::{error::Result, pool::PoolMask};
use sqlx::{Pool, Sqlite};
use zcash_protocol::consensus::Network;

use crate::Client;

use super::Recipient;

pub async fn prepare(
    network: &Network,
    connection: &Pool<Sqlite>,
    client: &mut Client,
    account: u32,
    recipients: &[Recipient],
    sender_pay_fees: bool,
    src_pools: u8,
) -> Result<()> {
    get_account_pool_mask(connection, account).await?;
    // #endregion

    Ok(())
}

pub async fn get_effective_src_pools(connection: &Pool<Sqlite>, account: u32, src_pools: u8) -> Result<PoolMask> {
    let apm = get_account_pool_mask(connection, account).await?;
    let spm = PoolMask(src_pools);
    let src_pool_mask = apm.intersect(&spm);
    Ok(src_pool_mask)
}

pub fn get_change_pool(src_pool_mask: PoolMask, dest_pool_mask: PoolMask) -> u8 {
    // Determine which pool to use for the change
    // If the source pools and the destinations pools intersect, pick
    // the best pool from the intersection
    let common_pools = src_pool_mask.intersect(&dest_pool_mask);
    if common_pools != PoolMask::empty() {
        return common_pools.to_best_pool().unwrap();
    }
    // Otherwise pick the best pool from the source pools
    // because it can minimize the fees and reduce the amount going
    // through the turnstile
    src_pool_mask.to_best_pool().unwrap()
}

pub async fn get_account_pool_mask(connection: &Pool<Sqlite>, account: u32) -> Result<PoolMask> {
    let (has_transparent,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM transparent_accounts WHERE account = ?)")
            .bind(account)
            .fetch_one(connection)
            .await?;
    let (has_sapling,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM sapling_accounts WHERE account = ?)")
            .bind(account)
            .fetch_one(connection)
            .await?;
    let (has_orchard,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM orchard_accounts WHERE account = ?)")
            .bind(account)
            .fetch_one(connection)
            .await?;
    let account_pool_mask = PoolMask(
        (has_transparent as u8) << 0 | (has_sapling as u8) << 1 | (has_orchard as u8) << 2,
    );

    Ok(account_pool_mask)
}

pub async fn get_pool_address(network: &Network, address: &str) -> Result<()> {

    Ok(())
}
