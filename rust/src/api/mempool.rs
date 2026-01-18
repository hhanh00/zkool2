use anyhow::Result;
use tokio::runtime::Runtime;
pub use tokio_util::sync::CancellationToken;

use crate::{api::coin::Coin};
#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

#[cfg(feature = "flutter")]
async fn run_mempool(
    mempool_sink: StreamSink<MempoolMsg>,
    cancel_token: CancellationToken,
    c: &Coin
) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let r = crate::mempool::run_mempool(
        mempool_sink,
        &c.network(),
        &mut connection,
        &mut c.client().await?,
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

#[cfg_attr(feature = "flutter", frb)]
#[derive(Clone)]
pub enum MempoolMsg {
    BlockHeight(u32),
    TxId(String, Vec<(u32, String, i64)>, u32),
}

#[cfg_attr(feature = "flutter", frb(opaque))]
pub struct Mempool {
    runtime: Runtime,
    cancel_token: Option<CancellationToken>,
}

impl Mempool {
    #[cfg_attr(feature = "flutter", frb(sync))]
    pub fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");
        Mempool {
            runtime,
            cancel_token: None,
        }
    }

    #[cfg(feature = "flutter")]
    pub fn run(&mut self, mempool_sink: StreamSink<MempoolMsg>, c: Coin) -> Result<()> {
        let ct = CancellationToken::new();
        self.cancel_token = Some(ct.clone());
        self.runtime.spawn(async move {
            if let Err(e) = run_mempool(mempool_sink, ct, &c).await {
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

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_mempool_tx(tx_id: &str, c: &Coin) -> Result<Vec<u8>> {
    let mut client = c.client().await?;
    let tx = crate::mempool::get_mempool_tx(&c.network(), &mut client, tx_id).await?;
    let mut tx_bytes = vec![];
    tx.write(&mut tx_bytes)?;

    Ok(tx_bytes)
}
