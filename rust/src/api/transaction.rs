use anyhow::Result;
use flutter_rust_bridge::frb;

use crate::get_coin;

#[frb]
pub async fn fill_missing_tx_prices() -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    crate::budget::fill_missing_tx_prices(&mut connection, c.account).await?;
    Ok(())
}

pub async fn set_tx_category(id: u32, category: Option<u32>) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    crate::db::set_tx_category(&mut connection, id, category).await?;
    Ok(())
}
