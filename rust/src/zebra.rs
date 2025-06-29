#![allow(unused_variables)]

use std::io::Read;

use anyhow::Result;
use futures::Stream;
use serde_json::{json, Value};
use tracing::info;
use zcash_note_encryption::COMPACT_NOTE_SIZE;
use zcash_primitives::{block::BlockHeader, transaction::Transaction};

use byteorder::{ReadBytesExt, LE};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{async_trait, Request};
use zcash_protocol::consensus::{BlockHeight, BranchId, Network};

use crate::{lwd::*, GRPCClient};

#[derive(Clone)]
pub struct ZebraClient {
    url: String,
    client: reqwest::Client,
}

impl ZebraClient {
    pub fn new(network: &Network, url: &str) -> Self {
        let client = reqwest::Client::new();
        Self {
            url: url.to_string(),
            client,
        }
    }
}

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

#[async_trait]
impl LwdServer for GRPCClient {
    async fn latest_height(&mut self) -> Result<u32> {
        let block_id = self
            .get_latest_block(Request::new(ChainSpec {}))
            .await?
            .into_inner();
        Ok(block_id.height as u32)
    }

    async fn block(&mut self, network: &Network, height: u32) -> Result<CompactBlock> {
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
        network: &Network,
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
        Ok(rep.error_message)
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
        let network = network.clone();
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
        let network = network.clone();
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

#[async_trait]
impl LwdServer for ZebraClient {
    async fn latest_height(&mut self) -> Result<u32> {
        let rep = self
            .client
            .post(&self.url)
            .json(&json!({
                "id": "0",
                "jsonrpc": "1.0",
                "method": "getblockcount",
                "params": []
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        let block_count = rep["result"].as_u64().unwrap_or_default();
        Ok(block_count as u32)
    }

    async fn block(&mut self, network: &Network, height: u32) -> Result<CompactBlock> {
        let rep = self
            .client
            .post(&self.url)
            .json(&json!({
                "id": "0",
                "jsonrpc": "1.0",
                "method": "getblock",
                "params": [height.to_string(), 0]
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        let block_hex = rep["result"].as_str().unwrap_or_default();
        let block_bytes = hex::decode(block_hex)
            .map_err(|e| anyhow::anyhow!("Failed to decode block hex: {}", e))?;
        let branch_id = BranchId::for_height(network, BlockHeight::from_u32(height));
        let cb = parse_block(branch_id, height, &block_bytes)?;
        Ok(cb)
    }

    async fn post_transaction(&mut self, height: u32, tx: &[u8]) -> Result<String> {
        let tx_hex = hex::encode(tx);
        let rep = self
            .client
            .post(&self.url)
            .json(&json!({
                "id": "0",
                "jsonrpc": "1.0",
                "method": "sendrawtransaction",
                "params": [tx_hex]
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        Ok(rep["result"].as_str().unwrap_or_default().to_string())
    }

    async fn transaction(&mut self, network: &Network, txid: &[u8]) -> Result<(u32, Transaction)> {
        let mut txid = txid.to_vec();
        txid.reverse();
        let tx_hex = hex::encode(txid);
        let rep = self
            .client
            .post(&self.url)
            .json(&json!({
                "id": "0",
                "jsonrpc": "1.0",
                "method": "getrawtransaction",
                "params": [tx_hex, 1]
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        let data = rep["result"]["hex"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response from node: No data field"))?
            .to_string();
        let height = rep["result"]["height"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("Invalid response from node: No height field"))?;
        let branch_id = BranchId::for_height(network, BlockHeight::from_u32(height as u32));
        let tx = Transaction::read(&mut hex::decode(data)?.as_slice(), branch_id)?;
        Ok((height as u32, tx))
    }

    type CompactBlockStream = ReceiverStream<CompactBlock>;
    async fn block_range(
        &mut self,
        network: &Network,
        start: u32,
        end: u32,
    ) -> Result<Self::CompactBlockStream> {
        let (tx, rx) = tokio::sync::mpsc::channel::<CompactBlock>(10);
        let mut client = self.clone();
        let network = network.clone();
        tokio::spawn(async move {
            for height in start..=end {
                let height = height as u32;
                let cb = client.block(&network, height).await?;
                tx.send(cb).await.ok();
            }
            Ok::<_, anyhow::Error>(())
        });
        Ok(ReceiverStream::new(rx))
    }

    type TransactionStream = ReceiverStream<(u32, Transaction, usize)>;
    async fn taddress_txs(
        &mut self,
        network: &Network,
        taddress: &str,
        start: u32,
        end: u32,
    ) -> Result<Self::TransactionStream> {
        let req = json!({
            "addresses": [taddress],
            "start": start,
            "end": end
        });
        let rep = self
            .client
            .post(&self.url)
            .json(&json!({
                "id": "0",
                "jsonrpc": "1.0",
                "method": "getaddresstxids",
                "params": [req]
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        let txids = rep["result"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response from node: No result field"))?;
        let txids = txids
            .into_iter()
            .map(|txid| {
                let txid_str = txid
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid txid in response"))?
                    .to_string();
                Ok::<_, anyhow::Error>(txid_str)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut client = self.clone();
        let network = network.clone();
        let (txs, rx) = tokio::sync::mpsc::channel::<(u32, Transaction, usize)>(10);
        tokio::spawn(async move {
            for txid in txids.iter() {
                let mut txid_hex = hex::decode(&txid).expect("Failed to decode txid hex");
                txid_hex.reverse();
                let (height, tx) = client.transaction(&network, &txid_hex).await?;
                txs.send((height, tx, 0)).await?;
            }

            Ok::<_, anyhow::Error>(())
        });

        Ok(ReceiverStream::new(rx))
    }

    async fn mempool_stream(&mut self, network: &Network) -> Result<Self::TransactionStream> {
        let (_, rx) = tokio::sync::mpsc::channel::<(u32, Transaction, usize)>(10);
        Ok(ReceiverStream::new(rx))
    }

    async fn tree_state(&mut self, height: u32) -> Result<(Vec<u8>, Vec<u8>)> {
        let rep = self
            .client
            .post(&self.url)
            .json(&json!({
                "id": "0",
                "jsonrpc": "1.0",
                "method": "z_gettreestate",
                "params": [height.to_string()]
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        let res = &rep["result"];
        let sapling_tree = res["sapling"]["commitments"]["finalState"]
            .as_str()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid response from node: No sapling commitments final state field"
                )
            })?
            .to_string();
        let orchard_tree = res["orchard"]["commitments"]["finalState"]
            .as_str()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid response from node: No orchard commitments final state field"
                )
            })?
            .to_string();
        Ok((hex::decode(sapling_tree)?, hex::decode(orchard_tree)?))
    }
}

pub fn parse_block(
    branch_id: BranchId,
    height: u32,
    mut block_bytes: &[u8],
) -> Result<CompactBlock> {
    let bh = BlockHeader::read(&mut block_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to parse block header: {}", e))?;
    let tx_count = read_compact_u32(&mut block_bytes);
    let mut vtx = vec![];
    for ivtx in 0..tx_count {
        let tx = Transaction::read(&mut block_bytes, branch_id)?;
        let txid = tx.txid().as_ref().to_vec();
        let tx_data = tx.into_data();
        // Skip fully transparent transactions
        if tx_data.sapling_bundle().is_none() && tx_data.orchard_bundle().is_none() {
            continue;
        }
        let mut spends = vec![];
        let mut outputs = vec![];
        if let Some(sapling_bundle) = tx_data.sapling_bundle() {
            for spend in sapling_bundle.shielded_spends().iter() {
                spends.push(CompactSaplingSpend {
                    nf: spend.nullifier().0.to_vec(),
                });
            }
            for output in sapling_bundle.shielded_outputs().iter() {
                outputs.push(CompactSaplingOutput {
                    cmu: output.cmu().to_bytes().to_vec(),
                    epk: output.ephemeral_key().0.to_vec(),
                    ciphertext: output.enc_ciphertext()[..COMPACT_NOTE_SIZE].to_vec(),
                });
            }
        }
        let mut actions = vec![];
        if let Some(orchard_bundle) = tx_data.orchard_bundle() {
            for action in orchard_bundle.actions().iter() {
                actions.push(CompactOrchardAction {
                    nullifier: action.nullifier().to_bytes().to_vec(),
                    cmx: action.cmx().to_bytes().to_vec(),
                    ephemeral_key: action.encrypted_note().epk_bytes.to_vec(),
                    ciphertext: action.encrypted_note().enc_ciphertext[..COMPACT_NOTE_SIZE]
                        .to_vec(),
                });
            }
        }

        vtx.push(CompactTx {
            index: ivtx as u64,
            hash: txid,
            spends,
            outputs,
            actions,
            ..Default::default()
        });
    }

    Ok(CompactBlock {
        height: height as u64,
        hash: bh.hash().0.to_vec(),
        prev_hash: bh.prev_block.0.to_vec(),
        time: bh.time,
        vtx,
        ..Default::default()
    })
}

pub fn read_compact_u32<R: Read>(mut reader: R) -> u32 {
    let tpe = reader.read_u8().unwrap();
    if tpe < 0xFD {
        return tpe as u32;
    }
    if tpe == 0xFD {
        return reader.read_u16::<LE>().unwrap() as u32;
    }
    if tpe == 0xFE {
        return reader.read_u32::<LE>().unwrap();
    }
    panic!("Invalid compact u32 type: {}", tpe); // 4 bytes should not never be needed
}
