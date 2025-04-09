use anyhow::Result;

use flutter_rust_bridge::frb;
use itertools::Itertools;
use zcash_primitives::transaction::builder::{BuildConfig, Builder};
use zcash_protocol::consensus::BlockHeight;

use crate::{pay::{
    error::Error,
    fee::{FeeManager, COST_PER_ACTION},
    plan::{fetch_unspent_notes_grouped_by_pool, get_change_pool},
    pool::{PoolMask, ALL_POOLS},
    prepare::to_zec,
    InputNote, Recipient, RecipientState,
}, warp::hasher::{OrchardHasher, SaplingHasher}};

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
pub async fn wip_plan(
    account: u32,
    src_pools: u8,
    recipients: &[Recipient],
    recipient_pays_fee: bool,
) -> Result<()> {
    let c = crate::get_coin!();
    let connection = c.get_pool();
    let effective_src_pools =
        crate::pay::plan::get_effective_src_pools(connection, account, src_pools).await?;

    let mut recipients = recipients.to_vec();
    let mut recipient_pools = PoolMask(0);
    for recipient in recipients.iter() {
        let pool = PoolMask::from_address(&recipient.address)?
            .intersect(&PoolMask(recipient.pools.unwrap_or(ALL_POOLS)));
        recipient_pools = recipient_pools.union(&pool);
    }
    println!("effective_src_pools: {:#b}", effective_src_pools.0);
    println!("recipient_pools: {:#b}", recipient_pools.0);
    let change_pool = get_change_pool(effective_src_pools, recipient_pools);
    println!("change_pool: {:#b}", change_pool);

    let mut fee_manager = FeeManager::default();
    fee_manager.add_output(change_pool);

    let recipient_states = recipients
        .iter()
        .map(|r| RecipientState::new(r.clone()).unwrap())
        .collect::<Vec<_>>();
    let mut input_pools = vec![vec![]; 3];
    let inputs = fetch_unspent_notes_grouped_by_pool(connection, account).await?;

    for (group, items) in inputs.into_iter().chunk_by(|inp| inp.pool).into_iter() {
        input_pools[group as usize].extend(items);
    }

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
    let (mut single, mut double) = recipient_states
        .into_iter()
        .partition::<Vec<_>, _>(|r| r.pool_mask != PoolMask(6));

    let mut fee_paid = 0;
    fill_single_receivers(
        &mut input_pools,
        &mut single,
        &mut fee_manager,
        recipient_pays_fee,
        &mut fee_paid,
    )?;

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

    let balances = input_pools
        .iter()
        .map(|pool| pool.iter().map(|n| n.remaining).sum::<u64>())
        .collect::<Vec<_>>();

    let largest_shielded_pool = if balances[1] > balances[2] {
        PoolMask(2)
    } else {
        PoolMask(4)
    };

    for d in double.iter_mut() {
        d.pool_mask = largest_shielded_pool;
    }

    fill_single_receivers(
        &mut input_pools,
        &mut double,
        &mut fee_manager,
        recipient_pays_fee,
        &mut fee_paid,
    )?;

    let fee = fee_manager.fee();

    if recipient_pays_fee {
        if recipients.len() > 1 {
            // if there are multiple recipients, we error because we
            // do not know which recipient should pay the fee
            return Err(Error::TooManyRecipients.into());
        } else {
            let recipient = recipients.first_mut().unwrap();
            if recipient.amount < fee {
                // if the recipient does not have enough balance to pay
                // the fee, we need to pay it from the input pools
                return Err(Error::RecipientNotEnoughAmount.into());
            }
            recipient.amount -= fee;
            fee_paid += fee;
        }
    }

    if single.iter().any(|r| r.remaining > 0)
        || double.iter().any(|r| r.remaining > 0)
        || fee > fee_paid
    {
        return Err(Error::NotEnoughFunds.into());
    }

    let total_input = input_pools
        .iter()
        .map(|pool| {
            pool.iter()
                .map(|n| if n.is_used() { n.amount } else { 0 })
                .sum::<u64>()
        })
        .sum::<u64>();
    let total_output = recipients.iter().map(|r| r.amount).sum::<u64>();

    let change = total_input - total_output - fee;

    for o in single.into_iter() {
        let RecipientState {
            recipient,
            remaining,
            pool_mask,
        } = o;
        assert_eq!(remaining, 0);
        println!(
            "address: {}, pool: {}, amount: {}",
            recipient.address,
            pool_mask.to_best_pool().unwrap(),
            to_zec(recipient.amount)
        );
    }

    println!(
        "change: {}, pool: {change_pool}, fee: {}",
        to_zec(change),
        to_zec(fee)
    );

    let height = crate::sync::get_db_height(connection, account).await?;
    let mut client = c.client().await?;
    let (ts, to) = crate::sync::get_tree_state(&c.network, &mut client, height).await?;
    let sapling_anchor = ts.to_edge(&SaplingHasher::default()).root(&SaplingHasher::default());
    let orchard_anchor = to.to_edge(&OrchardHasher::default()).root(&OrchardHasher::default());

    let mut _builder = Builder::new(
        &c.network,
        BlockHeight::from_u32(height),
        BuildConfig::Standard {
            sapling_anchor: sapling_crypto::Anchor::from_bytes(sapling_anchor).into_option(),
            orchard_anchor: orchard::Anchor::from_bytes(orchard_anchor).into_option(),
        },
    );

    for pool in input_pools.iter() {
        for inp in pool.iter() {
            if inp.is_used() {
                let InputNote {
                    id, amount, pool, ..
                } = inp;
                let (nf,): (Vec<u8>,) =
                    sqlx::query_as("SELECT nullifier FROM notes WHERE id_note = ?")
                        .bind(id)
                        .fetch_one(connection)
                        .await?;
                println!(
                    "id: {id}, pool: {pool}, nullifier: {}, amount: {}",
                    hex::encode(nf),
                    to_zec(*amount)
                );
            }
        }
    }

    Ok(())
}

fn fill_single_receivers(
    input_pools: &mut Vec<Vec<InputNote>>,
    recipients: &mut [RecipientState],
    fee_manager: &mut FeeManager,
    recipient_pays_fee: bool,
    fee_paid: &mut u64,
) -> Result<()> {
    let fill_order: [(u8, u8); 9] = [
        (2, 2),
        (1, 1), // O->O, S->S
        (2, 1),
        (1, 2), // O->S, S->O
        (0, 2),
        (0, 1), // T->O, T->S
        (2, 0),
        (1, 0), // O->T, S->T
        (0, 0), // T->T
    ];

    for (src, dst) in fill_order {
        for r in recipients.iter_mut() {
            if r.remaining == 0 {
                continue;
            }
            for inp in input_pools[src as usize].iter_mut() {
                if inp.remaining == 0 || inp.amount < COST_PER_ACTION {
                    continue;
                }
                // skip if the recipient is not interested in this pool
                if r.pool_mask.intersect(&PoolMask::from_pool(dst)).is_empty() {
                    continue;
                }
                // first time we see this note, add it to the fee manager
                if inp.amount == inp.remaining {
                    fee_manager.add_input(src)
                }

                // if the recipient pays the fees, we do not need to pay now
                let fee_remaining = if recipient_pays_fee {
                    0
                } else {
                    fee_manager.fee() - *fee_paid
                };

                // if the fee is not paid, we need to pay it on top of the output
                let to_pay = r.remaining + fee_remaining;
                // transfer the amount to the recipient
                let mut amount = inp.remaining.min(to_pay);

                // pay the fee first
                let a = amount.min(fee_remaining);
                *fee_paid += a;
                inp.remaining -= a;
                amount -= a;

                // use the rest to pay the output
                r.remaining -= amount;
                inp.remaining -= amount;
            }
        }
    }

    Ok(())
}
