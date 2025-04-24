use anyhow::Result;

use crate::pay::{plan::{plan_transaction, PcztPackage}, Recipient, TxPlan};
use flutter_rust_bridge::frb;

use super::account::Tx;

#[frb]
pub async fn prepare(
    src_pools: u8,
    recipients: &[Recipient],
    recipient_pays_fee: bool,
) -> Result<PcztPackage> {
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
pub fn to_plan(package: &PcztPackage) -> Result<TxPlan> {
    TxPlan::from(package)
}

#[frb]
pub async fn send(height: u32, data: &[u8]) -> Result<String> {
    let c = crate::get_coin!();
    let mut client = c.client().await?;

    let tx = crate::pay::send(&mut client, height, data).await?;
    Ok(tx)
}
