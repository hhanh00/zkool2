//! Task execution: the side effects, and nothing else.
//!
//! Every decision has already been made by the time control reaches here — a
//! task arrives naming exactly what to do, and these functions carry it out.
//! Keeping the effects behind this boundary is what makes the decision path
//! testable without a chain.
//!
//! Only the two broadcasts live here. `WaitDelay` and `WaitBoundary` consume
//! time rather than touching the wallet, so the driver schedules them; a
//! durable-execution engine would own them as timers instead.
//!
//! Migration never synchronizes. The wallet's own autosync advances the
//! checkpoint and migration observes where it got to — which is also why it
//! cannot force the wallet onto an anchor boundary and must wait for one.
//! A sync started here would in any case be unreliable: `synchronize_impl`
//! takes `SYNCING` with `try_lock` and returns silently when autosync already
//! holds it.

use anyhow::Result;

use crate::{
    account::get_account_full_address,
    api::coin::Coin,
    db::get_account_hw,
    pay::{
        plan::{extract_transaction, plan_transaction, sign_transaction},
        pool::PoolMask,
        send, Recipient,
    },
};

/// Broadcast the O→O split: consume `inputs`, mint `outputs` as standard
/// denominations back to the wallet's own address.
///
/// Returns the fee paid. The inputs are locked as part of the broadcast, so a
/// re-planned split cannot select them again.
pub async fn split(c: &Coin, inputs: &[u32], outputs: &[(u64, u8)]) -> Result<u64> {
    let network = c.network();
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    let height = client.latest_height().await?;
    let own_address = own_address(c, &mut connection).await?;

    let mut recipients: Vec<Recipient> = Vec::new();
    for &(denom, count) in outputs {
        for _ in 0..count {
            recipients.push(Recipient {
                address: own_address.clone(),
                amount: denom,
                pools: Some(PoolMask::from_pool(2).0), // Orchard only
                ..Recipient::default()
            });
        }
    }

    tracing::info!(
        "Migration split: {} non-SD notes → {} SD outputs",
        inputs.len(),
        recipients.len(),
    );

    let pczt = plan_transaction(
        &network,
        &mut connection,
        &mut client,
        c.account,
        PoolMask::from_pool(2).0, // Orchard source
        &recipients,
        false,
        None,
        false,
        None,
        None,
        true, // migration
        Some(inputs),
        None, // anchor_height
    )
    .await?;

    broadcast(c, &mut connection, &mut client, height, pczt).await
}

/// Broadcast the O→I hop: spend one Orchard SD note at `anchor`, mint
/// `amount` into Ironwood.
///
/// Returns the fee paid.
pub async fn migrate(c: &Coin, note: u32, amount: u64, anchor: u32) -> Result<u64> {
    let network = c.network();
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    let height = client.latest_height().await?;
    let own_address = own_address(c, &mut connection).await?;

    // One Ironwood output; the dummy Orchard input and dummy output that pad
    // the bundle are added by the builder.
    let recipients = vec![Recipient {
        address: own_address,
        amount,
        pools: Some(PoolMask::from_pool(3).0), // Ironwood
        ..Recipient::default()
    }];

    tracing::info!(
        "Migration: note id={} → Ironwood amount={}, anchor={}",
        note,
        amount,
        anchor,
    );

    let pczt = plan_transaction(
        &network,
        &mut connection,
        &mut client,
        c.account,
        PoolMask::from_pool(2).0, // Orchard source
        &recipients,
        false,
        None,
        false,
        None,
        None,
        true, // migration — O→I
        Some(&[note]),
        Some(anchor),
    )
    .await?;

    broadcast(c, &mut connection, &mut client, height, pczt).await
}

/// Sign, extract, send, and lock the inputs. Returns the fee.
///
/// Locking is what makes a re-planned task safe: the spent notes drop out of
/// the next observation, so the stale task fails its precondition rather than
/// producing a second transaction over the same inputs. A crash between the
/// send and the lock leaves that window open — closing it needs the two to
/// commit together, which is what a durable-execution transaction step buys.
async fn broadcast(
    c: &Coin,
    connection: &mut sqlx::SqliteConnection,
    client: &mut crate::Client,
    height: u32,
    pczt: crate::api::pay::PcztPackage,
) -> Result<u64> {
    let network = c.network();
    let fee = crate::pay::TxPlan::from_package(&network, &pczt)
        .map(|p| p.fee)
        .unwrap_or(0);
    let pczt = sign_transaction(&mut *connection, c.account, &network, &pczt).await?;
    let tx_bytes = extract_transaction(&pczt).await?;
    let _txid = send(client, height, &tx_bytes).await?;
    crate::pay::lock_spent_notes(&mut *connection, c.account, &tx_bytes).await?;
    Ok(fee)
}

async fn own_address(c: &Coin, connection: &mut sqlx::SqliteConnection) -> Result<String> {
    let network = c.network();
    let hw = get_account_hw(&mut *connection, c.account).await?;
    get_account_full_address(&network, &mut *connection, c.account, 0, hw).await
}
