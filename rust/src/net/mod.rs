use anyhow::Result;
use futures::Stream;
use zcash_primitives::transaction::Transaction;

use tonic::async_trait;

use crate::{
    api::coin::Network,
    lwd::*,
};

pub mod lwd;
pub mod zebra;

#[async_trait]
pub trait LwdServer: Send {
    async fn latest_height(&mut self) -> Result<u32>;
    async fn block(&mut self, network: &Network, height: u32) -> Result<CompactBlock>;

    type CompactBlockStream: Stream<Item = CompactBlock>;
    async fn block_range(
        &mut self,
        network: &Network,
        start: u32,
        end: u32,
    ) -> Result<Self::CompactBlockStream>;

    async fn transaction(&mut self, network: &Network, txid: &[u8]) -> Result<(u32, Transaction)>;
    async fn post_transaction(&mut self, height: u32, tx: &[u8]) -> Result<String>;

    type TransactionStream: Stream<Item = (u32, Transaction, usize)>;
    async fn taddress_txs(
        &mut self,
        network: &Network,
        taddress: &str,
        start: u32,
        end: u32,
    ) -> Result<Self::TransactionStream>;

    async fn mempool_stream(&mut self, network: &Network) -> Result<Self::TransactionStream>;

    async fn tree_state(&mut self, height: u32) -> Result<(Vec<u8>, Vec<u8>)>;
}
