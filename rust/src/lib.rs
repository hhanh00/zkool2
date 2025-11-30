use lwd::compact_tx_streamer_client::CompactTxStreamerClient;
use tokio_stream::wrappers::ReceiverStream;
use zcash_primitives::transaction::Transaction;

use crate::{lwd::CompactBlock, net::LwdServer};

pub mod account;
pub mod api;
pub mod bip38;
pub mod db;
mod frb_generated;
pub mod frost;
pub mod io;
pub mod key;
#[path = "./cash.z.wallet.sdk.rpc.rs"]
pub mod lwd;
pub mod memo;
pub mod mempool;
pub mod pay;
pub mod net;
pub mod sync;
pub mod warp;
pub mod budget;
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
pub mod ledger;

pub type Hash32 = [u8; 32];
pub type GRPCClient = CompactTxStreamerClient<tonic::transport::Channel>;
pub type Client = Box<dyn LwdServer<
        CompactBlockStream = ReceiverStream<CompactBlock>,
        TransactionStream = ReceiverStream<(u32, Transaction, usize)>,
    >>;

#[macro_export]
macro_rules! tiu {
    ($x: expr) => {
        $x.try_into().unwrap()
    };
}

pub trait IntoAnyhow<T> {
    fn anyhow(self) -> Result<T, anyhow::Error>;
}

impl<T, E> IntoAnyhow<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn anyhow(self) -> Result<T, anyhow::Error> {
        self.map_err(anyhow::Error::new)
    }
}
