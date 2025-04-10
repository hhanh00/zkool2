use anyhow::Result;

use crate::pay::{plan::plan_transaction, Recipient, TxPlan};
use flutter_rust_bridge::frb;

#[frb]
pub async fn prepare(
    src_pools: u8,
    recipients: &[Recipient],
    recipient_pays_fee: bool,
) -> Result<TxPlan> {
    let c = crate::get_coin!();
    let account = c.account;
    let network = &c.network;
    let connection = c.get_pool();
    let mut client = c.client().await?;

    plan_transaction(
        network,
        connection,
        &mut client,
        account,
        src_pools,
        recipients,
        recipient_pays_fee,
    )
    .await
}

#[frb]
pub async fn send(height: u32, data: &[u8]) -> Result<String> {
    let c = crate::get_coin!();
    let mut client = c.client().await?;

    let tx = crate::pay::send(&mut client, height, data).await?;
    Ok(tx)
}
