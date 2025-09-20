use lwd::compact_tx_streamer_client::CompactTxStreamerClient;
use tokio_stream::wrappers::ReceiverStream;
use zcash_primitives::transaction::Transaction;

use crate::{lwd::CompactBlock, zebra::LwdServer};

pub mod account;
pub mod api;
pub mod bip38;
pub mod coin;
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
pub mod sync;
pub mod warp;
pub mod zebra;
pub mod budget;

pub type Hash32 = [u8; 32];
pub type GRPCClient = CompactTxStreamerClient<tonic::transport::Channel>;
pub type Client = Box<dyn LwdServer<
        CompactBlockStream = ReceiverStream<CompactBlock>,
        TransactionStream = ReceiverStream<(u32, Transaction, usize)>,
    >>;