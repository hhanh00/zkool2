use anyhow::Result;
use tracing::info;
use zcash_primitives::transaction::Transaction;

use tokio_stream::wrappers::ReceiverStream;
use tonic::{async_trait, Request};
use zcash_protocol::consensus::{BlockHeight, BranchId};

use crate::{
    GRPCClient, api::coin::Network, lwd::*, net::LwdServer
};

#[async_trait]
impl LwdServer for GRPCClient {
    async fn latest_height(&mut self) -> Result<u32> {
        let block_id = self
            .get_latest_block(Request::new(ChainSpec {}))
            .await?
            .into_inner();
        Ok(block_id.height as u32)
    }

    async fn block(&mut self, _network: &Network, height: u32) -> Result<CompactBlock> {
        let block = self
            .get_block(Request::new(BlockId {
                height: height as u64,
                hash: vec![],
            }))
            .await?
            .into_inner();
        Ok(block)
    }

    type CompactBlockStream = ReceiverStream<CompactBlock>;
    async fn block_range(
        &mut self,
        _network: &Network,
        start: u32,
        end: u32,
    ) -> Result<Self::CompactBlockStream> {
        info!("Fetching block range from {} to {}", start, end);
        let mut blocks = self
            .get_block_range(Request::new(BlockRange {
                start: Some(BlockId {
                    height: start as u64,
                    hash: vec![],
                }),
                end: Some(BlockId {
                    height: end as u64,
                    hash: vec![],
                }),
                spam_filter_threshold: 0,
            }))
            .await?
            .into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel::<CompactBlock>(10);
        tokio::spawn(async move {
            while let Some(block) = blocks.message().await? {
                tx.send(block).await?;
            }
            Ok::<_, anyhow::Error>(())
        });
        Ok(ReceiverStream::new(rx))
    }

    async fn transaction(&mut self, network: &Network, txid: &[u8]) -> Result<(u32, Transaction)> {
        let rtx = self
            .get_transaction(Request::new(TxFilter {
                hash: txid.to_vec(),
                ..Default::default()
            }))
            .await?
            .into_inner();
        let height = rtx.height as u32;
        let branch_id = BranchId::for_height(network, BlockHeight::from_u32(height));
        let tx = Transaction::read(&mut &rtx.data[..], branch_id)?;
        Ok((height, tx))
    }

    async fn post_transaction(&mut self, height: u32, tx: &[u8]) -> Result<String> {
        let rep = self
            .send_transaction(Request::new(RawTransaction {
                data: tx.to_vec(),
                height: height as u64,
            }))
            .await?
            .into_inner();
        let m = if rep.error_code == 0 {
            rep.error_message.trim_matches('"').to_string()
        } else {
            rep.error_message
        };
        Ok(m)
    }

    type TransactionStream = ReceiverStream<(u32, Transaction, usize)>;
    async fn taddress_txs(
        &mut self,
        network: &Network,
        taddress: &str,
        start: u32,
        end: u32,
    ) -> Result<Self::TransactionStream> {
        let mut txs = self
            .get_taddress_txids(Request::new(TransparentAddressBlockFilter {
                address: taddress.to_string(),
                range: Some(BlockRange {
                    start: Some(BlockId {
                        height: start as u64,
                        hash: vec![],
                    }),
                    end: Some(BlockId {
                        height: end as u64,
                        hash: vec![],
                    }),
                    spam_filter_threshold: 0,
                }),
            }))
            .await?
            .into_inner();
        let network = *network;
        let (sender, rx) = tokio::sync::mpsc::channel::<(u32, Transaction, usize)>(10);
        tokio::spawn(async move {
            while let Some(rtx) = txs.message().await? {
                let len = rtx.data.len();
                let branch_id =
                    BranchId::for_height(&network, BlockHeight::from_u32(rtx.height as u32));
                let tx = Transaction::read(&mut &rtx.data[..], branch_id)?;
                sender.send((rtx.height as u32, tx, len)).await?;
            }
            Ok::<_, anyhow::Error>(())
        });
        Ok(ReceiverStream::new(rx))
    }

    async fn mempool_stream(&mut self, network: &Network) -> Result<Self::TransactionStream> {
        let mut txs = self
            .get_mempool_stream(Request::new(Empty {}))
            .await?
            .into_inner();
        let network = *network;
        let (sender, rx) = tokio::sync::mpsc::channel::<(u32, Transaction, usize)>(10);
        tokio::spawn(async move {
            while let Some(rtx) = txs.message().await? {
                let len = rtx.data.len();
                let branch_id =
                    BranchId::for_height(&network, BlockHeight::from_u32(rtx.height as u32));
                let tx = Transaction::read(&mut &rtx.data[..], branch_id)?;
                sender.send((rtx.height as u32, tx, len)).await?;
            }
            Ok::<_, anyhow::Error>(())
        });
        Ok(ReceiverStream::new(rx))
    }

    async fn tree_state(&mut self, height: u32) -> Result<(Vec<u8>, Vec<u8>)> {
        let state = self
            .get_tree_state(Request::new(BlockId {
                height: height as u64,
                hash: vec![],
            }))
            .await?
            .into_inner();
        let sapling_tree =
            hex::decode(&state.sapling_tree).expect("Failed to decode sapling tree hex");
        let orchard_tree =
            hex::decode(&state.orchard_tree).expect("Failed to decode sapling tree hex");
        Ok((sapling_tree, orchard_tree))
    }
}

