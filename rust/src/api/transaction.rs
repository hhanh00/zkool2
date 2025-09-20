use anyhow::Result;
use flutter_rust_bridge::frb;

use crate::get_coin;

#[frb]
pub async fn get_tx_without_usd_range() -> Result<(Option<u32>, Option<u32>)> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    let (min, max) = crate::budget::get_tx_without_usd_range(&mut connection, c.account).await?;
    Ok((min, max))
}
