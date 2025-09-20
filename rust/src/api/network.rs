use anyhow::Result;
use flutter_rust_bridge::frb;
use serde::Deserialize;

use crate::{coin::ServerType, get_coin};

#[frb]
pub async fn init_datadir(directory: &str) -> Result<()> {
    crate::coin::init_datadir(directory).await
}

#[frb(sync)]
pub fn set_lwd(server_type: ServerType, lwd: &str) {
    let mut coin = crate::coin::COIN.lock().unwrap();
    coin.set_url(server_type, lwd);
}

#[frb(sync)]
pub fn set_use_tor(use_tor: bool) {
    let mut coin = crate::coin::COIN.lock().unwrap();
    coin.set_use_tor(use_tor);
}

pub async fn get_current_height() -> Result<u32> {
    let c = crate::get_coin!();
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
pub async fn get_network_name() -> String {
    let c = get_coin!();
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

