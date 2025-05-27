use anyhow::Result;
use flutter_rust_bridge::frb;
use tokio::sync::{mpsc, Mutex};

use crate::{frb_generated::StreamSink, get_coin};

#[frb]
pub async fn run_mempool(mempool_sink: StreamSink<MempoolMsg>, height: u32) -> Result<()> {
    tracing::info!("Starting mempool stream at height {}", height);
    let c = get_coin!();
    let connection = c.get_pool();
    let r = crate::mempool::run_mempool(
        mempool_sink,
        &c.network,
        &connection,
        &mut c.client().await?,
        height,
    )
    .await;
    match r {
        Ok(_) => {
            tracing::info!("Mempool stream finished successfully");
        }
        Err(e) => {
            tracing::error!("Error running mempool: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

#[frb]
pub async fn cancel_mempool() -> Result<()> {
    tracing::info!("Cancelling mempool stream");
    let mut tx_cancel = MEMPOOL_TX_CANCEL.lock().await;
    if let Some(tx_cancel) = tx_cancel.take() {
        let _ = tx_cancel.send(());
    }

    Ok(())
}

#[frb]
pub enum MempoolMsg {
    TxId(String, Vec<(u32, String, i64)>, u32),
}

lazy_static::lazy_static! {
    pub static ref MEMPOOL_TX_CANCEL: Mutex<Option<mpsc::Sender<()>>> =
    Mutex::new(None);
}
