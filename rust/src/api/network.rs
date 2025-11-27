use anyhow::Result;
use flutter_rust_bridge::frb;
use serde::Deserialize;

use crate::api::coin::Coin;

#[frb]
pub async fn init_datadir(directory: &str) -> Result<()> {
    crate::api::coin::init_datadir(directory).await
}

pub async fn get_current_height(c: &Coin) -> Result<u32> {
    let mut client = c.client().await?;
    let height = client.latest_height().await?;
    Ok(height as u32)
}

pub async fn get_coingecko_price() -> Result<f64> {
    let rep =
        reqwest::get("https://api.coingecko.com/api/v3/simple/price?ids=zcash&vs_currencies=usd")
            .await?
            .error_for_status()?
            .json::<ZcashUSD>()
            .await?;
    Ok(rep.zcash.usd)
}

#[frb]
pub async fn get_network_name(c: &Coin) -> String {
    c.get_name().to_string()
}

#[derive(Deserialize)]
struct Usd {
    usd: f64,
}

#[derive(Deserialize)]
struct ZcashUSD {
    zcash: Usd,
}

