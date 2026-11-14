//! Raw download bench: fetch one block range over K lightwalletd connections
//! and discard the blocks. No scanning, no reordering, no buffering: each
//! connection gets an equal slice and one GetBlockRange.
//!
//! cargo run --release --features graphql --example dl_bench -- \
//!     https://zec.rocks:443 3000000 3200000 6 3
//! (url, start, end, max connections, rounds)

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use prost::Message as _;
use rlz::lwd::{compact_tx_streamer_client::CompactTxStreamerClient, BlockId, BlockRange};
use tonic::transport::{Channel, ClientTlsConfig};

const MB: f64 = 1_048_576.0;

/// Same channel setup as `Coin::client` for a direct lightwalletd connection.
async fn connect(url: &str) -> anyhow::Result<CompactTxStreamerClient<Channel>> {
    let mut endpoint = Channel::from_shared(url.to_string())?;
    if url.starts_with("https") {
        endpoint = endpoint.tls_config(ClientTlsConfig::new().with_enabled_roots())?;
    }
    Ok(CompactTxStreamerClient::new(endpoint.connect().await?))
}

struct ConnStats {
    first_block_ms: f64,
    secs: f64,
    bytes: u64,
}

async fn run(url: &str, start: u64, end: u64, k: u64, round: u32) -> anyhow::Result<()> {
    let connect_started = Instant::now();
    let mut clients = Vec::new();
    for _ in 0..k {
        clients.push(connect(url).await?);
    }
    let connect_ms = connect_started.elapsed().as_secs_f64() * 1e3;

    let span = end - start + 1;
    let total = Arc::new(AtomicU64::new(0));
    // (seconds, bytes received by all connections) when the first one finished:
    // the aggregate rate while all K were active.
    let first_done: Arc<Mutex<Option<(f64, u64)>>> = Arc::new(Mutex::new(None));
    let started = Instant::now();
    let mut tasks = Vec::new();
    for (i, mut client) in clients.into_iter().enumerate() {
        let i = i as u64;
        let s = start + span * i / k;
        let e = start + span * (i + 1) / k - 1;
        let total = Arc::clone(&total);
        let first_done = Arc::clone(&first_done);
        tasks.push(tokio::spawn(async move {
            let mut blocks = client
                .get_block_range(BlockRange {
                    start: Some(BlockId { height: s, hash: vec![] }),
                    end: Some(BlockId { height: e, hash: vec![] }),
                    ..Default::default()
                })
                .await?
                .into_inner();
            let mut bytes = 0u64;
            let mut first_block_ms = 0.0;
            while let Some(block) = blocks.message().await? {
                if bytes == 0 {
                    first_block_ms = started.elapsed().as_secs_f64() * 1e3;
                }
                let n = block.encoded_len() as u64;
                bytes += n;
                total.fetch_add(n, Ordering::Relaxed);
            }
            let secs = started.elapsed().as_secs_f64();
            let mut first = first_done.lock().unwrap();
            if first.is_none() {
                *first = Some((secs, total.load(Ordering::Relaxed)));
            }
            Ok::<_, anyhow::Error>(ConnStats { first_block_ms, secs, bytes })
        }));
    }
    let mut stats = Vec::new();
    for t in tasks {
        stats.push(t.await??);
    }
    let wall = started.elapsed().as_secs_f64();
    let bytes: u64 = stats.iter().map(|s| s.bytes).sum();
    let (fs, fb) = first_done.lock().unwrap().unwrap();
    let per_conn: Vec<String> = stats
        .iter()
        .map(|s| format!("{:.2}", s.bytes as f64 / MB / s.secs))
        .collect();
    let first_block: Vec<String> = stats.iter().map(|s| format!("{:.0}", s.first_block_ms)).collect();
    println!(
        "round={round} k={k} total_mb={:.1} wall_s={:.1} overall_mbps={:.2} all_active_mbps={:.2} \
         per_conn_mbps=[{}] first_block_ms=[{}] connect_ms={:.0}",
        bytes as f64 / MB,
        wall,
        bytes as f64 / MB / wall,
        fb as f64 / MB / fs,
        per_conn.join(","),
        first_block.join(","),
        connect_ms,
    );
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider().install_default().ok();
    let args: Vec<String> = std::env::args().collect();
    anyhow::ensure!(args.len() == 6, "usage: dl_bench <url> <start> <end> <max_conns> <rounds>");
    let (url, start, end) = (&args[1], args[2].parse::<u64>()?, args[3].parse::<u64>()?);
    let (max_k, rounds) = (args[4].parse::<u64>()?, args[5].parse::<u32>()?);
    for round in 1..=rounds {
        for k in 1..=max_k {
            run(url, start, end, k, round).await?;
        }
    }
    Ok(())
}
