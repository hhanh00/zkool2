use anyhow::Result;
use flutter_rust_bridge::frb;
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
