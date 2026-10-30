use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

use crate::api::coin::Coin;
use crate::net::http;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

#[cfg_attr(feature = "flutter", frb)]
pub async fn init_datadir(directory: &str) -> Result<()> {
    crate::api::coin::init_datadir(directory).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn is_ironwood_active(c: &Coin) -> Result<bool> {
    use zcash_protocol::consensus::{BlockHeight, NetworkUpgrade, Parameters};
    let network = c.network();
    let mut client = c.client().await?;
    let height = client.latest_height().await?;
    Ok(network.is_nu_active(NetworkUpgrade::Nu6_3, BlockHeight::from_u32(height)))
}

pub async fn get_current_height(c: &Coin) -> Result<u32> {
    let mut client = c.client().await?;
    let height = client.latest_height().await?;
    Ok(height as u32)
}

/// The CoinGecko API base for this wallet; see [`crate::budget::coingecko_url`].
async fn coingecko_url(c: &Coin) -> Result<String> {
    let mut connection = c.get_connection().await?;
    crate::budget::coingecko_url(&mut connection).await
}

pub async fn get_coingecko_price(api: &str, currency: &str, c: &Coin) -> Result<f64> {
    let base = coingecko_url(c).await?;
    let client = http::client(
        http::proxy_url(c.transport, &c.proxy),
        Duration::from_secs(15),
    )?;
    let rep = http::http_get(
        &client,
        &[&format!(
            "{base}/api/v3/simple/price?ids=zcash&vs_currencies={currency}&x_cg_demo_api_key={api}"
        )],
        http::RetryPolicy::default(),
    )
    .await?
    .json::<Value>()?;
    let price = rep["zcash"][currency]
        .as_f64()
        .ok_or(anyhow!("Price not found for currency: {currency}"))?;
    Ok(price)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_supported_vs_currencies(api: &str, c: &Coin) -> Result<Vec<String>> {
    let base = coingecko_url(c).await?;
    let client = http::client(
        http::proxy_url(c.transport, &c.proxy),
        Duration::from_secs(15),
    )?;
    let rep = http::http_get(
        &client,
        &[&format!(
            "{base}/api/v3/simple/supported_vs_currencies?x_cg_demo_api_key={api}"
        )],
        http::RetryPolicy::default(),
    )
    .await?
    .json::<Vec<String>>()?;
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
    c: &Coin,
) -> Result<ExchangeRate> {
    let base = coingecko_url(c).await?;
    let client = http::client(
        http::proxy_url(c.transport, &c.proxy),
        Duration::from_secs(15),
    )?;
    let rep = http::http_get(
        &client,
        &[&format!(
            "{base}/api/v3/simple/price?ids=zcash&vs_currencies={from_currency},{to_currency}&x_cg_demo_api_key={api}"
        )],
        http::RetryPolicy::default(),
    )
    .await?
    .json::<Value>()?;
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

/// The most recent `limit` candles of `interval` for the market chart,
/// oldest first; see [`crate::market`]. `interval` is a Binance kline interval
/// such as `1h`, `4h`, `1d` or `1w`; `limit` is 1..=1500.
#[cfg_attr(feature = "flutter", frb)]
pub async fn fetch_candles(interval: &str, limit: u32, c: &Coin) -> Result<Vec<Candle>> {
    let base = {
        let mut connection = c.get_connection().await?;
        crate::market::klines_url(&mut connection).await?
    };
    let client = http::client(
        http::proxy_url(c.transport, &c.proxy),
        Duration::from_secs(15),
    )?;
    crate::market::get_klines(&base, interval, limit, &client).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_network_name(c: &Coin) -> String {
    c.get_name().to_string()
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn query_lwd_list(coin: u8) -> Result<Vec<LWDInfo>> {
    crate::net::lwd::query_lwd_list(coin).await
}

/// True when `url` is a mixnet-native server address
/// (`nym://<identity>.<encryption>@<gateway>`).
#[cfg(feature = "nym")]
#[cfg_attr(feature = "flutter", frb(sync))]
pub fn is_valid_nym_url(url: String) -> bool {
    crate::net::nym_service::parse_nym_url(&url).is_some()
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub from_price: f64,
    pub to_price: f64,
    pub from_currency: String,
    pub to_currency: String,
}

/// One kline of the market chart. `time` is the open time in seconds since
/// the epoch; prices are in the quote currency (USDC); `volume` is in ZEC.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candle {
    pub time: u32,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
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
