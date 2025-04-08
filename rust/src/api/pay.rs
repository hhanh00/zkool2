use anyhow::Result;

use flutter_rust_bridge::frb;

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
