use anyhow::Result;
use flutter_rust_bridge::frb;

use crate::api::coin::Coin;

#[frb]
pub async fn fill_missing_tx_prices(c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::budget::fill_missing_tx_prices(&mut connection, c.account).await?;
    Ok(())
}

#[frb]
pub async fn set_tx_category(id: u32, category: Option<u32>, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::set_tx_category(&mut connection, id, category).await?;
    Ok(())
}

#[frb]
pub async fn set_tx_price(id: u32, price: Option<f64>, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::set_tx_price(&mut connection, id, price).await?;
    Ok(())
}

#[frb]
pub async fn fetch_category_amounts(from: Option<u32>, to: Option<u32>, c: &Coin) -> Result<Vec<(String, f64, bool)>> {
    let mut connection = c.get_connection().await?;
    crate::budget::fetch_category_amounts(&mut connection, c.account, from, to).await
}

#[frb]
pub async fn fetch_amounts(from: Option<u32>, to: Option<u32>, category: u32, c: &Coin) -> Result<Vec<(u32, f64)>> {
    let mut connection = c.get_connection().await?;
    crate::budget::fetch_amounts(&mut connection, c.account, from, to, category).await
}
