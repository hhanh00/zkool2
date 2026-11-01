use anyhow::Result;
use tracing::debug;
use zcash_primitives::transaction::Transaction;

use tokio_stream::wrappers::ReceiverStream;
use tonic::{async_trait, Request};
use zcash_protocol::consensus::{BlockHeight, BranchId};

use crate::{
    api::{coin::Network, network::LWDInfo},
    lwd::*,
    net::LwdServer,
    GRPCClient,
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

    type CompactBlockStream = ReceiverStream<Result<CompactBlock>>;
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
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<CompactBlock>>(10);
        tokio::spawn(async move {
            // The error goes down the channel as the last item: a stream
            // that failed must not look like one that ended (#1474).
            loop {
                // A consumer that dropped the stream ends the task at once,
                // not at the next block the server sends (#1538).
                let message = tokio::select! {
                    _ = tx.closed() => break,
                    message = blocks.message() => message,
                };
                match message {
                    Ok(Some(block)) => {
                        if tx.send(Ok(block)).await.is_err() {
                            break; // the consumer is gone
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let _ = tx.send(Err(e.into())).await;
                        break;
                    }
                }
            }
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

    type TransactionStream = ReceiverStream<Result<(u32, Transaction, usize)>>;
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
        let (sender, rx) = tokio::sync::mpsc::channel::<Result<(u32, Transaction, usize)>>(10);
        tokio::spawn(async move {
            // A transport or decode error is the stream's last item.
            let forward = async {
                loop {
                    // The consumer dropped the stream, on a cancel for one
                    // (#1481): stop waiting on the server and end the task.
                    let rtx = tokio::select! {
                        _ = sender.closed() => break,
                        message = txs.message() => match message? {
                            Some(rtx) => rtx,
                            None => break,
                        },
                    };
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
                    if sender.send(Ok((rtx.height as u32, tx, len))).await.is_err() {
                        break;
                    }
                }
                Ok::<_, anyhow::Error>(())
            };
            if let Err(e) = forward.await {
                let _ = sender.send(Err(e)).await;
            }
        });
        Ok(ReceiverStream::new(rx))
    }

    async fn mempool_stream(&mut self, network: &Network) -> Result<Self::TransactionStream> {
        let mut txs = self
            .get_mempool_stream(Request::new(Empty {}))
            .await?
            .into_inner();
        let network = *network;
        let (sender, rx) = tokio::sync::mpsc::channel::<Result<(u32, Transaction, usize)>>(10);
        tokio::spawn(async move {
            let forward = async {
                loop {
                    // The consumer went away (an error ended the monitor, or
                    // it was cancelled): stop holding the server's stream
                    // until its next block and end the task (#1538).
                    let rtx = tokio::select! {
                        _ = sender.closed() => break,
                        message = txs.message() => match message? {
                            Some(rtx) => rtx,
                            None => break,
                        },
                    };
                    let len = rtx.data.len();
                    let branch_id =
                        BranchId::for_height(&network, BlockHeight::from_u32(rtx.height as u32));
                    let tx = Transaction::read(&mut &rtx.data[..], branch_id)?;
                    if sender.send(Ok((rtx.height as u32, tx, len))).await.is_err() {
                        break;
                    }
                }
                Ok::<_, anyhow::Error>(())
            };
            if let Err(e) = forward.await {
                let _ = sender.send(Err(e)).await;
            }
        });
        Ok(ReceiverStream::new(rx))
    }

    async fn tree_state(&mut self, height: u32) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
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
        let ironwood_tree = hex::decode(&state.ironwood_tree).unwrap_or_default();
        Ok((sapling_tree, orchard_tree, ironwood_tree))
    }
}

/// Lists the public lightwalletd servers over `route`, the wallet's selected
/// transport, so the registry sees no more than the node connection does.
pub async fn query_lwd_list(coin: u8, route: &crate::net::http::Route) -> Result<Vec<LWDInfo>> {
    // 0 = mainnet, 1 = testnet, 2 = regtest
    if coin == 2 {
        return Ok(Vec::new());
    }
    let chain = if coin == 1 { "test" } else { "main" };
    let url = format!("https://hosh.zec.rocks/api/v0/zec.json?chain={chain}");
    let rep = crate::net::http::http_get(
        route,
        &[&url],
        crate::net::http::RetryPolicy::default(),
    )
    .await?
    .json::<serde_json::Value>()?;

    // Parse the JSON response and convert it to Vec<LWDInfo>
    let servers = rep["servers"].as_array().cloned().unwrap_or_default();
    let mut lwd_list = Vec::new();
    for item in servers {
        let hostname = item["hostname"].as_str().unwrap_or_default();
        let port = item["port"].as_u64().unwrap_or(9067);
        let scheme = if hostname.ends_with(".onion") {
            "http"
        } else {
            "https"
        };
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

#[cfg(test)]
mod producer_tests {
    use super::*;
    use crate::lwd::compact_tx_streamer_server::{CompactTxStreamer, CompactTxStreamerServer};
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::task::{Context, Poll};
    use std::time::Duration;

    /// A server stream that never yields and counts itself as live until
    /// the server drops it, which happens when the client lets go of the
    /// gRPC stream.
    struct Stalled<T> {
        live: Arc<AtomicUsize>,
        _item: std::marker::PhantomData<fn() -> T>,
    }

    impl<T> Stalled<T> {
        fn new(live: &Arc<AtomicUsize>) -> Self {
            live.fetch_add(1, Ordering::SeqCst);
            Self {
                live: live.clone(),
                _item: std::marker::PhantomData,
            }
        }
    }

    impl<T> Drop for Stalled<T> {
        fn drop(&mut self) {
            self.live.fetch_sub(1, Ordering::SeqCst);
        }
    }

    impl<T> tokio_stream::Stream for Stalled<T> {
        type Item = std::result::Result<T, tonic::Status>;
        fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Pending
        }
    }

    /// A lightwalletd whose mempool and block streams open and then say
    /// nothing, as a quiet mempool does between blocks.
    struct Quiet {
        live: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl CompactTxStreamer for Quiet {
        type GetBlockRangeStream = Stalled<CompactBlock>;
        type GetTaddressTxidsStream = Stalled<RawTransaction>;
        type GetMempoolTxStream = Stalled<CompactTx>;
        type GetMempoolStreamStream = Stalled<RawTransaction>;
        type GetAddressUtxosStreamStream = Stalled<GetAddressUtxosReply>;

        async fn get_block_range(
            &self,
            _: Request<BlockRange>,
        ) -> std::result::Result<tonic::Response<Self::GetBlockRangeStream>, tonic::Status> {
            Ok(tonic::Response::new(Stalled::new(&self.live)))
        }
        async fn get_mempool_stream(
            &self,
            _: Request<Empty>,
        ) -> std::result::Result<tonic::Response<Self::GetMempoolStreamStream>, tonic::Status> {
            Ok(tonic::Response::new(Stalled::new(&self.live)))
        }
        async fn get_latest_block(&self, _: Request<ChainSpec>) -> std::result::Result<tonic::Response<BlockId>, tonic::Status> {
            unimplemented!()
        }
        async fn get_block(&self, _: Request<BlockId>) -> std::result::Result<tonic::Response<CompactBlock>, tonic::Status> {
            unimplemented!()
        }
        async fn get_transaction(&self, _: Request<TxFilter>) -> std::result::Result<tonic::Response<RawTransaction>, tonic::Status> {
            unimplemented!()
        }
        async fn send_transaction(&self, _: Request<RawTransaction>) -> std::result::Result<tonic::Response<SendResponse>, tonic::Status> {
            unimplemented!()
        }
        async fn get_taddress_txids(
            &self,
            _: Request<TransparentAddressBlockFilter>,
        ) -> std::result::Result<tonic::Response<Self::GetTaddressTxidsStream>, tonic::Status> {
            unimplemented!()
        }
        async fn get_taddress_balance(&self, _: Request<AddressList>) -> std::result::Result<tonic::Response<Balance>, tonic::Status> {
            unimplemented!()
        }
        async fn get_taddress_balance_stream(
            &self,
            _: Request<tonic::Streaming<Address>>,
        ) -> std::result::Result<tonic::Response<Balance>, tonic::Status> {
            unimplemented!()
        }
        async fn get_mempool_tx(&self, _: Request<Exclude>) -> std::result::Result<tonic::Response<Self::GetMempoolTxStream>, tonic::Status> {
            unimplemented!()
        }
        async fn get_tree_state(&self, _: Request<BlockId>) -> std::result::Result<tonic::Response<TreeState>, tonic::Status> {
            unimplemented!()
        }
        async fn get_address_utxos(
            &self,
            _: Request<GetAddressUtxosArg>,
        ) -> std::result::Result<tonic::Response<GetAddressUtxosReplyList>, tonic::Status> {
            unimplemented!()
        }
        async fn get_address_utxos_stream(
            &self,
            _: Request<GetAddressUtxosArg>,
        ) -> std::result::Result<tonic::Response<Self::GetAddressUtxosStreamStream>, tonic::Status> {
            unimplemented!()
        }
        async fn get_lightd_info(&self, _: Request<Empty>) -> std::result::Result<tonic::Response<LightdInfo>, tonic::Status> {
            unimplemented!()
        }
        async fn ping(&self, _: Request<crate::lwd::Duration>) -> std::result::Result<tonic::Response<PingResponse>, tonic::Status> {
            unimplemented!()
        }
    }

    /// Starts the quiet server and connects a client to it.
    async fn quiet_server() -> (GRPCClient, Arc<AtomicUsize>) {
        let live = Arc::new(AtomicUsize::new(0));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let service = CompactTxStreamerServer::new(Quiet { live: live.clone() });
        tokio::spawn(
            tonic::transport::Server::builder()
                .add_service(service)
                .serve_with_incoming(tonic::transport::server::TcpIncoming::from(listener)),
        );
        let client = crate::lwd::compact_tx_streamer_client::CompactTxStreamerClient::connect(format!("http://127.0.0.1:{port}"))
            .await
            .unwrap();
        (client, live)
    }

    async fn wait_for(live: &AtomicUsize, expected: usize) -> bool {
        for _ in 0..40 {
            if live.load(Ordering::SeqCst) == expected {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        false
    }

    /// Dropping the stream a producer feeds ends the producer and releases
    /// the server's stream at once, rather than at the server's next
    /// message (#1538).
    #[tokio::test]
    async fn a_dropped_consumer_releases_the_servers_stream() {
        let (mut client, live) = quiet_server().await;

        let mempool = client.mempool_stream(&Network::Main).await.unwrap();
        assert!(wait_for(&live, 1).await, "the mempool stream is open on the server");
        drop(mempool);
        assert!(wait_for(&live, 0).await, "the server's mempool stream was released");

        let blocks = client.block_range(&Network::Main, 1, 10).await.unwrap();
        assert!(wait_for(&live, 1).await, "the block stream is open on the server");
        drop(blocks);
        assert!(wait_for(&live, 0).await, "the server's block stream was released");
    }
}
