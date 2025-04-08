use anyhow::Result;

use flutter_rust_bridge::frb;

use crate::{
    api::sync::balance,
    db::calculate_balance,
    pay::{
        fee::FeeManager,
        plan::get_change_pool,
        pool::{PoolMask, ALL_POOLS},
        Recipient, RecipientState,
    },
};

use super::sync::PoolBalance;

#[frb]
pub async fn prepare(account: u32, sender_pay_fees: bool, src_pools: u8) -> Result<()> {
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

// #[frb]
pub async fn wip_plan(account: u32, src_pools: u8, recipients: &[Recipient]) -> Result<u8> {
    let c = crate::get_coin!();
    let connection = c.get_pool();
    let effective_src_pools =
        crate::pay::plan::get_effective_src_pools(connection, account, src_pools).await?;

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

    let mut fee_manager = FeeManager::default();
    let mut current_fee = fee_manager.add_output(change_pool);
    let recipients = recipients
        .iter()
        .map(|r| RecipientState::new(r.clone()).unwrap())
        .collect::<Vec<_>>();
    let mut balances = calculate_balance(connection, account).await?;

    // we can merge notes from the same pool because they are fully fungible
    // but we should keep the funds from different pools separate
    // because even though they can participate in the same transaction
    // they don't have the same properties.
    // calculate_balance will return the balance for each pool
    // and we have to pick up notes to send to the recipients
    // There can be multiple recipients in the single transaction
    // Recipients can accept multiple receivers when they use a unified address
    // The simplest way to do this would be to choose any allowed receiver
    // and then pick up randomly notes from the wallet until we cover the
    // amount needed for the transaction
    // but this could be inefficient and leak information about the wallet
    // Instead we will choose based on the balances available and the
    // recipients
    //
    // We use two passes. In the first pass, we only consider the recipients
    // that have single receiver addresses. For these, there is no option
    // to choose the receiver. The only decision we need to make is to
    // choose what pool to use for the inputs.
    // This is handled by the function fill_single_receivers
    //
    let (single, mut double) = recipients
        .into_iter()
        .partition::<Vec<_>, _>(|r| r.pool_mask != PoolMask(6));

    fill_single_receivers(&mut balances, &single, &mut fee_manager, &mut current_fee).await?;

    // In the second pass, we will consider the recipients that have
    // multiple receivers. We always favor shielded receivers over
    // transparent ones. Hence, if a UA has a transparent and a
    // sapling receiver, it counts as a single sapling receiver.
    // Then, the only time we can have a multiple receiver recipient
    // is when we have a sapling and an orchard receiver, ie.
    // when we have to choose between shielded pools
    //
    // In the second pass, we constrain the receiver to be the pool
    // that we have the most balance in. This is because we hope
    // to minimize the amount that would have to go through the
    // turnstile.

    let largest_shielded_pool = if balances.0[1] > balances.0[2] {
        PoolMask(2)
    } else {
        PoolMask(4)
    };

    for d in double.iter_mut() {
        d.pool_mask = largest_shielded_pool;
    }

    fill_single_receivers(&mut balances, &single, &mut fee_manager, &mut current_fee).await?;

    Ok(change_pool)
}

async fn fill_single_receivers(
    balances: &mut PoolBalance,
    recipients: &[RecipientState],
    fee_manager: &mut FeeManager,
    current_fee: &mut u64,
) -> Result<()> {
    todo!()
}
