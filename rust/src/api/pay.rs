use anyhow::Result;

use flutter_rust_bridge::frb;

use crate::pay::{plan::get_change_pool, pool::{PoolMask, ALL_POOLS}, Recipient};

#[frb]
pub async fn prepare(
    account: u32,
    sender_pay_fees: bool,
    src_pools: u8,
) -> Result<()> {
    let c = crate::get_coin!();
    let network = &c.network;
    let connection = c.get_pool();
    let mut client = c.client().await?;
    crate::pay::prepare::prepare(
        network,
        connection,
        &mut client,
        account,
        &[],
        sender_pay_fees,
        src_pools,
    )
    .await?;

    Ok(())
}

#[frb]
pub async fn wip_plan(account: u32, src_pools: u8, recipients: &[Recipient]) -> Result<u8> {
    let c = crate::get_coin!();
    let connection = c.get_pool();
    let effective_src_pools = crate::pay::plan::get_effective_src_pools(connection, account, src_pools).await?;

    let mut recipient_pools = PoolMask(0);
    for recipient in recipients {
        let pool = PoolMask::from_address(&recipient.address)?
            .intersect(&PoolMask(recipient.pools.unwrap_or(ALL_POOLS)));
        recipient_pools = recipient_pools.union(&pool);
    }
    println!("effective_src_pools: {:#b}", effective_src_pools.0);
    println!("recipient_pools: {:#b}", recipient_pools.0);
    let change_pool = get_change_pool(effective_src_pools, recipient_pools);
    println!("change_pool: {:#b}", change_pool);

    Ok(change_pool)
}
