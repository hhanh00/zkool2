use anyhow::Result;
use serde::Deserialize;

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

pub async fn get_coingecko_price(api: &str) -> Result<f64> {
    let rep =
        reqwest::get(&format!("https://api.coingecko.com/api/v3/simple/price?ids=zcash&vs_currencies=usd&x_cg_demo_api_key={api}"))
            .await?
            .error_for_status()?
            .json::<ZcashUSD>()
            .await?;
    Ok(rep.zcash.usd)
}

#[cfg_attr(feature = "flutter", frb)]
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

