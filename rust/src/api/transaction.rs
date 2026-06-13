use anyhow::Result;
use crate::api::coin::Coin;

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

#[cfg_attr(feature = "flutter", frb)]
pub async fn fill_missing_tx_prices(api: String, currency: String, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::budget::fill_missing_tx_prices(&mut connection, c.account, &currency, &api).await?;
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn update_historical_prices(currency: String, exchange_rate: f64, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::budget::update_historical_prices(&mut connection, &currency, exchange_rate).await?;
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn set_user_memo(id_tx: u32, memo: Option<String>, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::set_user_memo(&mut connection, c.account, id_tx, memo).await?;
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn set_tx_category(id: u32, category: Option<u32>, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::set_tx_category(&mut connection, id, category).await?;
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn set_tx_price(id: u32, price: Option<f64>, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::set_tx_price(&mut connection, id, price).await?;
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn fetch_category_amounts(from: Option<u32>, to: Option<u32>, c: &Coin) -> Result<Vec<(String, f64, bool)>> {
    let mut connection = c.get_connection().await?;
    crate::budget::fetch_category_amounts(&mut connection, c.account, from, to).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn fetch_amounts(from: Option<u32>, to: Option<u32>, category: u32, c: &Coin) -> Result<Vec<(u32, f64)>> {
    let mut connection = c.get_connection().await?;
    crate::budget::fetch_amounts(&mut connection, c.account, from, to, category).await
}
