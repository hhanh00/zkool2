use anyhow::Result;
use tokio_util::sync::CancellationToken;

#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;
use crate::{api::coin::Coin, sync::transparent_sweep};
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

#[cfg_attr(feature = "flutter", frb(opaque))]
pub struct TransparentScanner {
    pub(crate) cancellation_token: CancellationToken,
}

impl TransparentScanner {
    pub fn new() -> Result<Self> {
        Ok(Self {
            cancellation_token: CancellationToken::new(),
        })
    }

    #[cfg(feature = "flutter")]
    pub async fn run(
        &mut self,
        address_stream: StreamSink<String>,
        end_height: u32,
        gap_limit: u32,
        c: &Coin,
    ) -> Result<()> {
        let connection = c.get_connection().await?;
        let client = c.client().await?;
        transparent_sweep(
            &c.network(),
            connection,
            client,
            c.account,
            end_height,
            gap_limit,
            move |address| {
                let _ = address_stream.add(address);
            },
            self.cancellation_token.clone(),
        )
        .await?;
        Ok(())
    }

    pub fn cancel(&self) -> Result<()> {
        self.cancellation_token.cancel();
        Ok(())
    }
}
