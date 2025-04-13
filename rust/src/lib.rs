use lwd::compact_tx_streamer_client::CompactTxStreamerClient;

#[path ="./cash.z.wallet.sdk.rpc.rs"]
pub mod lwd;
pub mod coin;
pub mod db;
pub mod bip38;
pub mod key;
pub mod account;
pub mod sync;
pub mod memo;
pub mod warp;
pub mod pay;
pub mod io;
pub mod api;
mod frb_generated;

pub type Hash32 = [u8; 32];
pub type Client = CompactTxStreamerClient<tonic::transport::Channel>;
