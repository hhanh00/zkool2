use anyhow::Result;
use flutter_rust_bridge::frb;

pub async fn transparent_sync(accounts: &[u32], end_height: u32) -> Result<()> {
    let c = crate::get_coin!();
    // Update sync height in the database
    crate::db::update_sync_transparent_height(c.get_pool(), accounts, end_height).await?;

    Ok(())
}

#[frb(dart_metadata = ("freezed"))]
pub struct Transaction {
    pub txid: String,
    pub height: u64,
    pub address: String,
}
