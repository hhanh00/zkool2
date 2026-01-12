use anyhow::Error;
use lwd::compact_tx_streamer_client::CompactTxStreamerClient;
use tokio_stream::wrappers::ReceiverStream;
use zcash_primitives::transaction::Transaction;

use crate::{frb_generated::StreamSink, lwd::CompactBlock, net::LwdServer};

pub mod account;
pub mod api;
pub mod bip38;
pub mod budget;
pub mod db;
mod frb_generated;
pub mod frost;
#[cfg(feature = "graphql")]
pub mod graphql;
pub mod io;
pub mod key;
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
pub mod ledger;
#[path = "./cash.z.wallet.sdk.rpc.rs"]
pub mod lwd;
pub mod memo;
pub mod mempool;
pub mod net;
pub mod pay;
pub mod recover;
pub mod sync;
pub mod warp;

pub type Hash32 = [u8; 32];
pub type GRPCClient = CompactTxStreamerClient<tonic::transport::Channel>;
pub type Client = Box<
    dyn LwdServer<
        CompactBlockStream = ReceiverStream<CompactBlock>,
        TransactionStream = ReceiverStream<(u32, Transaction, usize)>,
    >,
>;

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

pub trait Sink<V>: Clone {
    fn send(&self, value: V) -> impl std::future::Future<Output = ()> + Send;
    fn send_error(&self, e: Error) -> impl std::future::Future<Output = ()> + Send;
}

impl<T: Clone + frb_generated::SseEncode + Send + Sync> Sink<T> for StreamSink<T> {
    async fn send(&self, value: T) {
        let _ = self.add(value);
    }

    async fn send_error(&self, error: Error) {
        let _ = self.add_error(error);
    }
}

impl<T: Send + Sync + std::fmt::Debug> Sink<T> for () {
    async fn send(&self, value: T) {
        tracing::info!("{value:?}");
    }
    async fn send_error(&self, _error: Error) {}
}
