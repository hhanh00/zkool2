use anyhow::Result;
use flutter_rust_bridge::frb;
use tokio::runtime::Runtime;
pub use tokio_util::sync::CancellationToken;

use crate::{frb_generated::StreamSink, get_coin};

async fn run_mempool(
    mempool_sink: StreamSink<MempoolMsg>,
    height: u32,
    cancel_token: CancellationToken,
) -> Result<()> {
    let c = get_coin!();
    let connection = c.get_pool();
    let r = crate::mempool::run_mempool(
        mempool_sink,
        &c.network,
        &connection,
        &mut c.client().await?,
        height,
        cancel_token,
    )
    .await;
    match r {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error running mempool: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

#[frb]
pub enum MempoolMsg {
    TxId(String, Vec<(u32, String, i64)>, u32),
}

#[frb(opaque)]
pub struct Mempool {
    runtime: Runtime,
    cancel_token: Option<CancellationToken>,
}

impl Mempool {
    #[frb(sync)]
    pub fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build().expect("Failed to create tokio runtime");
        Mempool { runtime, cancel_token: None }
    }

    pub fn run(&mut self, mempool_sink: StreamSink<MempoolMsg>, height: u32) -> Result<()> {
        let ct = CancellationToken::new();
        self.cancel_token = Some(ct.clone());
        self.runtime.spawn(async move {
            if let Err(e) = run_mempool(mempool_sink, height, ct).await {
                tracing::error!("Error running mempool: {}", e);
            }
        });
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<()> {
        if let Some(token) = self.cancel_token.take() {
            token.cancel();
        }
        Ok(())
    }
}
