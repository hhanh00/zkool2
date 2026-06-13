use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

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

fn coingecko_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("zkool/1.0")
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub async fn get_coingecko_price(api: &str, currency: &str) -> Result<f64> {
    let rep = coingecko_client()
        .get(&format!(
            "https://api.coingecko.com/api/v3/simple/price?ids=zcash&vs_currencies={currency}&x_cg_demo_api_key={api}"
        ))
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    let price = rep["zcash"][currency]
        .as_f64()
        .ok_or(anyhow!("Price not found for currency: {currency}"))?;
    Ok(price)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_supported_vs_currencies(api: &str) -> Result<Vec<String>> {
    let rep = coingecko_client()
        .get(&format!(
            "https://api.coingecko.com/api/v3/simple/supported_vs_currencies?x_cg_demo_api_key={api}"
        ))
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<String>>()
        .await?;
    Ok(rep)
}

/// Returns the ZEC price in both `from_currency` and `to_currency`.
/// The exchange rate from `from_currency` to `to_currency` can be computed as
/// `to_price / from_price`.
#[cfg_attr(feature = "flutter", frb)]
pub async fn get_exchange_rate(
    api: &str,
    from_currency: &str,
    to_currency: &str,
) -> Result<ExchangeRate> {
    let rep = coingecko_client()
        .get(&format!(
            "https://api.coingecko.com/api/v3/simple/price?ids=zcash&vs_currencies={from_currency},{to_currency}&x_cg_demo_api_key={api}"
        ))
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    let from_price = rep["zcash"][from_currency]
        .as_f64()
        .ok_or(anyhow!("Price not found for currency: {from_currency}"))?;
    let to_price = rep["zcash"][to_currency]
        .as_f64()
        .ok_or(anyhow!("Price not found for currency: {to_currency}"))?;
    Ok(ExchangeRate {
        from_price,
        to_price,
        from_currency: from_currency.to_string(),
        to_currency: to_currency.to_string(),
    })
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_network_name(c: &Coin) -> String {
    c.get_name().to_string()
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn query_lwd_list(coin: u8) -> Result<Vec<LWDInfo>> {
    crate::net::lwd::query_lwd_list(coin).await
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub from_price: f64,
    pub to_price: f64,
    pub from_currency: String,
    pub to_currency: String,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LWDInfo {
    pub url: String,
    pub is_tor: bool,
    pub height: u32,
    pub status: String,
    pub uptime: u32,
    pub version: String,
    pub ping: u32,
}
