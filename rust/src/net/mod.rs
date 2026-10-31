use anyhow::Result;
use futures::Stream;
use zcash_primitives::transaction::Transaction;

use tonic::async_trait;

use crate::{api::coin::Network, lwd::*};

pub mod http;
pub mod lwd;
#[cfg(feature = "nym")]
pub mod nym;
#[cfg(feature = "nym")]
pub mod nym_service;
pub mod votechain;
pub mod zebra;

#[async_trait]
pub trait LwdServer: Send {
    async fn latest_height(&mut self) -> Result<u32>;
    async fn block(&mut self, network: &Network, height: u32) -> Result<CompactBlock>;

    /// A stream item is a block, or the error that ended the stream. A
    /// stream that just ends is complete only if its consumer saw the last
    /// height it asked for: a transport failure must not look like the
    /// end of the range.
    type CompactBlockStream: Stream<Item = Result<CompactBlock>>;
    async fn block_range(
        &mut self,
        network: &Network,
        start: u32,
        end: u32,
    ) -> Result<Self::CompactBlockStream>;

    async fn transaction(&mut self, network: &Network, txid: &[u8]) -> Result<(u32, Transaction)>;
    async fn post_transaction(&mut self, height: u32, tx: &[u8]) -> Result<String>;

    /// Height, transaction and serialized size; or the error that ended the
    /// stream, so a consumer does not take a dead stream for an empty one.
    type TransactionStream: Stream<Item = Result<(u32, Transaction, usize)>>;
    async fn taddress_txs(
        &mut self,
        network: &Network,
        taddress: &str,
        start: u32,
        end: u32,
    ) -> Result<Self::TransactionStream>;

    async fn mempool_stream(&mut self, network: &Network) -> Result<Self::TransactionStream>;

    async fn tree_state(&mut self, height: u32) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)>;
}
