use anyhow::Result;

use crate::api::coin::Coin;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

#[cfg_attr(feature = "flutter", frb)]
pub async fn init_datadir(directory: &str) -> Result<()> {
    crate::api::coin::init_datadir(directory).await
}

pub async fn get_current_height(c: &Coin) -> Result<u32> {
    let mut client = c.client().await?;
    let height = client.latest_height().await?;
    Ok(height as u32)
}

pub async fn get_coingecko_price(api: &str, currency: &str) -> Result<f64> {
    // CoinGecko echoes the requested vs_currency as the JSON key, e.g.
    // {"zcash":{"eur":12.34}}. The currency is user-selected from a fixed list
    // so we lower-case it and parse the response dynamically.
    let currency = currency.to_lowercase();
    let rep: serde_json::Value = reqwest::get(&format!(
        "https://api.coingecko.com/api/v3/simple/price?ids=zcash&vs_currencies={currency}&x_cg_demo_api_key={api}"
    ))
    .await?
    .error_for_status()?
    .json()
    .await?;
    let price = rep
        .pointer(&format!("/zcash/{currency}"))
        .and_then(|v| v.as_f64())
        .ok_or_else(|| anyhow::anyhow!("no zcash/{currency} price in response"))?;
    Ok(price)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_network_name(c: &Coin) -> String {
    c.get_name().to_string()
}

