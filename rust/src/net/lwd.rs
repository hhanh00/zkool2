use anyhow::Result;
use tracing::debug;
use zcash_primitives::transaction::Transaction;

use tokio_stream::wrappers::ReceiverStream;
use tonic::{async_trait, Request};
use zcash_protocol::consensus::{BlockHeight, BranchId};

use crate::{
    GRPCClient, api::{coin::Network, network::LWDInfo}, lwd::*, net::LwdServer
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
        debug!("Fetching block range from {} to {}", start, end);
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
                let txid = tx.txid();
                let has_tb = tx.transparent_bundle().is_some();
                let (n_vin, n_vout) = tx
                    .transparent_bundle()
                    .map(|tb| (tb.vin.len(), tb.vout.len()))
                    .unwrap_or((0, 0));
                debug!(
                    "LWD raw_tx: height={} branch_id={:?} txid={} size={} has_transparent={} n_vin={} n_vout={} raw_hex_first120={}",
                    rtx.height,
                    branch_id,
                    txid,
                    len,
                    has_tb,
                    n_vin,
                    n_vout,
                    hex::encode(&rtx.data[..rtx.data.len().min(120)]),
                );
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

pub async fn query_lwd_list(coin: u8) -> Result<Vec<LWDInfo>> {
    // 0 = mainnet, 1 = testnet, 2 = regtest
    if coin == 2 {
        return Ok(Vec::new());
    }
    let chain = if coin == 1 { "test" } else { "main" };
    let url = format!("https://hosh.zec.rocks/api/v0/zec.json?chain={chain}");
    let rep = reqwest::get(&url)
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    // Parse the JSON response and convert it to Vec<LWDInfo>
    let servers = rep["servers"].as_array().cloned().unwrap_or_default();
    let mut lwd_list = Vec::new();
    for item in servers {
        let hostname = item["hostname"].as_str().unwrap_or_default();
        let port = item["port"].as_u64().unwrap_or(9067);
        let scheme = if hostname.ends_with(".onion") { "http" } else { "https" };
        let url = format!("{}://{}:{}", scheme, hostname, port);
        let is_tor = hostname.ends_with(".onion");
        let height = item["height"].as_u64().unwrap_or(0) as u32;
        let status = if item["online"].as_bool().unwrap_or(false) {
            "online"
        } else {
            "offline"
        };
        let uptime = (item["uptime_30d"].as_f64().unwrap_or(0.0) * 100.0) as u32;
        let ping = item["ping"].as_f64().unwrap_or(0.0) as u32;
        // These fields are not present in the API response, use defaults
        let version = item["node_version"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let lwd_info = LWDInfo {
            url,
            is_tor,
            height,
            status: status.to_string(),
            uptime,
            version,
            ping,
        };
        lwd_list.push(lwd_info);
    }

    Ok(lwd_list)
}
