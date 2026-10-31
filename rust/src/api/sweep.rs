use anyhow::Result;
use tokio_util::sync::CancellationToken;

#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;
#[cfg(feature = "flutter")]
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

    /// Runs the scan to its end, or to a cancel, and returns its result:
    /// a failure reaches the Dart side as the stream's error, and the
    /// stream closes when the scan ends (#1502). Takes `&self` so that
    /// `cancel` is not locked out while the scan runs.
    #[cfg(feature = "flutter")]
    pub async fn run(
        &self,
        address_stream: StreamSink<String>,
        end_height: u32,
        gap_limit: u32,
        account: u32,
        c: &Coin,
    ) -> Result<()> {
        let mut connection = c.get_connection().await?;
        let mut client = c.client().await?;
        transparent_sweep(
            &c.network(),
            &mut connection,
            &mut client,
            account,
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
