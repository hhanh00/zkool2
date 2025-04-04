use anyhow::Result;
use tonic::Request;
use flutter_rust_bridge::frb;

use crate::{get_coin, lwd::{BlockId, BlockRange, TransparentAddressBlockFilter}, sync::Transaction};

// #[frb]
pub async fn get_transparent_transactions(accounts: Vec<u32>) -> Result<()> {
    let c = get_coin!();
    let mut client = c.client().await?;

    Ok(())
}
