use anyhow::Result;
use flutter_rust_bridge::frb;
use serde::Deserialize;
use tonic::Request;

use crate::lwd::ChainSpec;

#[frb(sync)]
pub fn set_lwd(lwd: &str) {
    let mut coin = crate::coin::COIN.lock().unwrap();
    coin.set_lwd(lwd);
}

pub async fn get_current_height() -> Result<u32> {
    let c = crate::get_coin!();
    let mut client = c.client().await?;
    let height = client.get_latest_block(Request::new(ChainSpec {})).await?.into_inner().height;
    Ok(height as u32)
}

pub async fn get_coingecko_price() -> Result<f64> {
    let rep = reqwest::get("https://api.coingecko.com/api/v3/simple/price?ids=zcash&vs_currencies=usd")
        .await?
        .error_for_status()?
        .json::<ZcashUSD>()
        .await?;
    Ok(rep.zcash.usd)
}

#[derive(Deserialize)]
struct USD {
    usd: f64,
}

#[derive(Deserialize)]
struct ZcashUSD {
    zcash: USD,
}
