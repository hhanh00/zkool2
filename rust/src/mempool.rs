use anyhow::Result;
use tonic::Request;
use zcash_primitives::transaction::Transaction;
use zcash_protocol::consensus::{BlockHeight, BranchId, Network};
use crate::api::mempool::MempoolMsg;
use crate::frb_generated::StreamSink;
use crate::lwd::Empty;

use crate::Client;

pub async fn run_mempool(mempool_tx: StreamSink<MempoolMsg>, network: &Network, client: &mut Client, height: u32) -> Result<()> {
    let mut mempool_txs = client.get_mempool_stream(Request::new(Empty {})).await?.into_inner();
    let consensus_branch_id = BranchId::for_height(network, BlockHeight::from_u32(height));
    while let Some(tx) = mempool_txs.message().await? {
        let txdata = tx.data;
        let tx = Transaction::read(&*txdata, consensus_branch_id)?;
        let txid = tx.txid();
        let tx_hash = txid.to_string();
        let _ = mempool_tx.add(MempoolMsg::TxId(tx_hash));
    }
    Ok(())
}