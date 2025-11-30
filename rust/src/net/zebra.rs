#![allow(unused_variables)]

use std::{
    io::Read,
    pin::Pin,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use arti_client::TorClient;
use httparse::Status;
use reqwest::Url;
use rustls::{pki_types::ServerName, ClientConfig, RootCertStore};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio_rustls::TlsConnector;
use tor_rtcompat::PreferredRuntime;
use webpki_roots::TLS_SERVER_ROOTS;
use zcash_note_encryption::COMPACT_NOTE_SIZE;
use zcash_primitives::{block::BlockHeader, transaction::Transaction};

use byteorder::{ReadBytesExt, LE};
use tokio_stream::wrappers::ReceiverStream;
use tonic::async_trait;
use zcash_protocol::consensus::{BlockHeight, BranchId};

use crate::{
    IntoAnyhow, api::coin::{Network, TOR}, lwd::*, net::LwdServer
};

#[derive(Clone)]
pub struct ZebraClient {
    url: String,
    client: reqwest::Client,

    ssl: bool,
    host: String,
    port: u16,
    path: String,
    tls_config: Arc<ClientConfig>,
}

impl ZebraClient {
    pub fn new(network: &Network, url: &str) -> Result<Self> {
        let client = reqwest::Client::new();

        let url = Url::parse(url).anyhow()?;
        let host = url.domain().ok_or(anyhow::anyhow!("Not domain"))?;
        let port = url
            .port_or_known_default()
            .ok_or(anyhow::anyhow!("No known port"))?;
        let path = url.path();
        let scheme = url.scheme();
        let ssl = match scheme {
            "http" => false,
            "https" => true,
            _ => anyhow::bail!("Unsupported URL scheme"),
        };
        // host: &str, port: u16, uri: &str
        let root_cert_store = RootCertStore::from_iter(TLS_SERVER_ROOTS.iter().cloned());

        let tls_config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth(); // We don't need client certificates for standard web browsing

        Ok(Self {
            url: url.to_string(),
            client,
            ssl,
            host: host.to_string(),
            port,
            path: path.to_string(),
            tls_config: Arc::new(tls_config),
        })
    }
}

trait AsyncRW: AsyncRead + AsyncWrite {}
impl<T: AsyncRead + AsyncWrite + Send> AsyncRW for T {}

macro_rules! jsonrpc {
    ($client: ident, $method: literal, $params: tt, $ret: ty) => {
        {
            let id = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let req = json!({
                "id": id.to_string(),
                "jsonrpc": "1.0",
                "method": $method,
                "params": $params,
            });
            $client.jsonrpc_impl::<$ret>(req).await
        }
    };
}

impl ZebraClient {
    pub async fn jsonrpc_impl<R>(
        &self,
        req: Value,
    ) -> Result<R>
    where
        R: for<'de> Deserialize<'de>,
    {
        let rep = if let Some(tor_client) = TOR.get() {
            let tor = &*tor_client.lock().await;
            self.post_tor(tor, req).await?
        } else {
            self.client
                .post(&self.url)
                .json(&req)
                .send()
                .await?
                .error_for_status()?
                .json::<Value>()
                .await?
        };
        let res: R = serde_json::from_value(rep["result"].clone())?;
        Ok(res)
    }

    pub async fn post_tor(
        &self,
        tor_client: &TorClient<PreferredRuntime>,
        req: Value,
    ) -> Result<Value> {
        let connector = TlsConnector::from(self.tls_config.clone());

        let host = self.host.clone();
        let server_name: ServerName = host.clone().try_into().anyhow()?;

        let stream = tor_client.connect((host, self.port)).await?;

        let mut stream: Pin<Box<dyn AsyncRW + Send>> = if self.ssl {
            let tls_stream = connector
                .connect(server_name, stream)
                .await
                .context("TLS handshake failed over Tor stream")?;
            Box::pin(tls_stream)
        } else {
            Box::pin(stream)
        };

        let request_json = req.to_string();

        stream
            .write_all(format!("POST /{} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{request_json}",
            self.path, self.host).as_bytes())
            .await?;

        stream.flush().await?;

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await?;

        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut rep = httparse::Response::new(&mut headers);
        let Status::Complete(offset) = rep.parse(&buf)? else {
            anyhow::bail!("Invalid HTTP response")
        };
        let body = String::from_utf8_lossy(&buf[offset..]);

        let body: Value = serde_json::from_str(&body)?;
        if let Some(error) = body.pointer("/error") {
            anyhow::bail!(
                "JSON RPC error: {}",
                error.pointer("/message").unwrap().as_str().unwrap()
            )
        }
        let result = body.pointer("/result").unwrap();

        Ok(result.clone())
    }
}

#[async_trait]
impl LwdServer for ZebraClient {
    async fn latest_height(&mut self) -> Result<u32> {
        let block_count = jsonrpc!(self, "getblockcount", (), u32)?;
        Ok(block_count as u32)
    }

    async fn block(&mut self, network: &Network, height: u32) -> Result<CompactBlock> {
        let block_hex = jsonrpc!(
            self,
            "getblock",
            [height.to_string(), 0],
            String
        )?;
        let block_bytes = hex::decode(block_hex)
            .map_err(|e| anyhow::anyhow!("Failed to decode block hex: {}", e))?;
        let branch_id = BranchId::for_height(network, BlockHeight::from_u32(height));
        let cb = parse_block(branch_id, height, &block_bytes)?;
        Ok(cb)
    }

    async fn post_transaction(&mut self, height: u32, tx: &[u8]) -> Result<String> {
        let tx_hex = hex::encode(tx);
        let rep = jsonrpc!(
            self,
            "sendrawtransaction",
            [tx_hex],
            String
        )?;
        Ok(rep)
    }

    async fn transaction(&mut self, network: &Network, txid: &[u8]) -> Result<(u32, Transaction)> {
        let mut txid = txid.to_vec();
        txid.reverse();
        let tx_hex = hex::encode(txid);
        let rep = jsonrpc!(
            self,
            "getrawtransaction",
            [tx_hex, 1],
            Value
        )?;
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
        let network = *network;
        tokio::spawn(async move {
            for height in start..=end {
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
        let rep = jsonrpc!(self, "getaddresstxids", [req], Value)?;
        let txids = rep["result"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response from node: No result field"))?;
        let txids = txids
            .iter()
            .map(|txid| {
                let txid_str = txid
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid txid in response"))?
                    .to_string();
                Ok::<_, anyhow::Error>(txid_str)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut client = self.clone();
        let network = *network;
        let (txs, rx) = tokio::sync::mpsc::channel::<(u32, Transaction, usize)>(10);
        tokio::spawn(async move {
            for txid in txids.iter() {
                let mut txid_hex = hex::decode(txid).expect("Failed to decode txid hex");
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
        let res = jsonrpc!(
            self,
            "z_gettreestate",
            [height.to_string()],
            Value
        )?;
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
    panic!("Invalid compact u32 type: {tpe}"); // 4 bytes should not never be needed
}
