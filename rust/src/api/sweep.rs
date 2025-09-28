use anyhow::Result;
use flutter_rust_bridge::frb;
use tokio_util::sync::CancellationToken;

use crate::{frb_generated::StreamSink, get_coin, sync::transparent_sweep};

#[frb(opaque)]
pub struct TransparentScanner {
    pub(crate) cancellation_token: CancellationToken,
}

impl TransparentScanner {
    pub fn new() -> Result<Self> {
        let cancellation_token = CancellationToken::new();
        Ok(Self {
            cancellation_token,
        })
    }

    pub async fn run(&mut self, address_stream: StreamSink<String>,
        end_height: u32,
        gap_limit: u32,

    ) -> Result<()> {
        let c = get_coin!();
        let mut connection = c.get_connection().await?;
        let mut client = c.client().await?;

        transparent_sweep(
            &c.network,
            &mut connection,
            &mut client,
            c.account,
            end_height,
            gap_limit,
            |address| {
                let _ = address_stream.add(address);
            },
            &self.cancellation_token,
        )
        .await?;
        Ok(())
    }

    pub async fn cancel(&mut self) -> Result<()> {
        let cancellation_token = &self.cancellation_token;
        cancellation_token.cancel();
        Ok(())
    }
}
