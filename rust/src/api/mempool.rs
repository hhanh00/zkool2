use anyhow::Result;
use flutter_rust_bridge::frb;

use crate::{frb_generated::StreamSink, get_coin};

#[frb]
pub async fn run_mempool(mempool_sink: StreamSink<MempoolMsg>, height: u32) -> Result<()> {
    let c = get_coin!();
    let connection = c.get_pool();
    crate::mempool::run_mempool(
        mempool_sink,
        &c.network,
        &connection,
        &mut c.client().await?,
        height,
    )
    .await?;
    Ok(())
}

#[frb]
pub enum MempoolMsg {
    TxId(String, Vec<(String, i64)>, u32),
}

