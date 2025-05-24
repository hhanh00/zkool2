use lwd::compact_tx_streamer_client::CompactTxStreamerClient;

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
pub mod pay;
pub mod sync;
pub mod warp;
pub mod mempool;

pub type Hash32 = [u8; 32];
pub type Client = CompactTxStreamerClient<tonic::transport::Channel>;
