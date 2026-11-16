//! Sync bench: restore one account into an empty wallet, sync it to the
//! tip, and report the time and where it went. Everything runs in this
//! process: no server, no HTTP.
//!
//! cargo run --release --features graphql --example sync_bench -- \
//!     --server https://zec.rocks:443 --birthday 2271890
//!
//! Options:
//!   --server URL       lightwalletd to sync from (required)
//!   --birthday HEIGHT  the account's birth height (required)
//!   --seed "WORDS"     mnemonic to restore (default: the all-`abandon` test seed)
//!   --network N        main (default) or test
//!   --fast             skip the memo and detail pass
//!   --json PATH        write the result as JSON
//!   --versus PATH      a `slipstream bench --json` result to print beside this one
//!   --keep             keep the temporary wallet
//!   --tip              print the server's current height and exit (no --birthday needed)
//!   --trace PATH       write the block download's scheduling events as JSON lines:
//!                      slices, streams, the scan's waits and each response's backend
//!   --trace-blocks     with --trace, also every block received and delivered
//!
//! The JSON uses the names of Slipstream's bench summary where the two
//! engines measure the same thing: `total_s`, `fetch_s`, `scan_s`, `blocks`.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, bail, Context as _, Result};
use rlz::api::account::{new_account, NewAccount};
use rlz::api::coin::Coin;
use rlz::api::sync::SyncOutcome;
use rlz::sync::bench::SyncStats;
use serde_json::{json, Value};
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::{EnvFilter, Layer as _};

const TEST_SEED: &str = "abandon abandon abandon abandon abandon abandon abandon abandon \
                         abandon abandon abandon about";
/// Transparent, Sapling and Orchard.
const POOLS: u8 = 7;

struct Args {
    server: String,
    birthday: u32,
    seed: String,
    coin: u8,
    fast: bool,
    json: Option<PathBuf>,
    versus: Option<PathBuf>,
    keep: bool,
    tip: bool,
    trace: Option<PathBuf>,
    trace_blocks: bool,
}

fn parse_args() -> Result<Args> {
    let mut values: HashMap<String, String> = HashMap::new();
    let mut flags: Vec<String> = vec![];
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--fast" | "--keep" | "--tip" | "--trace-blocks" => flags.push(arg),
            "--server" | "--birthday" | "--seed" | "--network" | "--json" | "--versus"
            | "--trace" => {
                let value = args.next().ok_or_else(|| anyhow!("{arg} needs a value"))?;
                values.insert(arg, value);
            }
            other => bail!("unknown option {other} (see the comment at the top of sync_bench.rs)"),
        }
    }
    let coin = match values.get("--network").map(String::as_str) {
        None | Some("main") => 0,
        Some("test") => 1,
        Some(other) => bail!("--network is main or test, not {other}"),
    };
    Ok(Args {
        server: values.remove("--server").ok_or_else(|| anyhow!("--server is required"))?,
        birthday: match values.get("--birthday") {
            Some(height) => height.parse().context("--birthday")?,
            None if flags.iter().any(|f| f == "--tip") => 0,
            None => bail!("--birthday is required"),
        },
        seed: values.remove("--seed").unwrap_or_else(|| TEST_SEED.to_string()),
        coin,
        fast: flags.iter().any(|f| f == "--fast"),
        json: values.remove("--json").map(PathBuf::from),
        versus: values.remove("--versus").map(PathBuf::from),
        keep: flags.iter().any(|f| f == "--keep"),
        tip: flags.iter().any(|f| f == "--tip"),
        trace: values.remove("--trace").map(PathBuf::from),
        trace_blocks: flags.iter().any(|f| f == "--trace-blocks"),
    })
}

fn duration(seconds: f64) -> String {
    if seconds >= 60.0 {
        format!("{}:{:05.2}", (seconds / 60.0) as u64, seconds % 60.0)
    } else {
        format!("{seconds:.2}s")
    }
}

fn report(total_s: f64, height: u32, stats: &SyncStats) {
    println!();
    println!("zkool sync: {} to height {height}, {} blocks", duration(total_s), stats.blocks);
    println!(
        "  scan busy {} | scan waiting {} | block reader waiting {} | database commit {}",
        duration(stats.scan_busy_s),
        duration(stats.scan_wait_s),
        duration(stats.download_wait_s),
        duration(stats.write_busy_s),
    );
    if stats.slices > 0 {
        println!(
            "  download: {:.1} MB, {} streams at peak ({} opened, {} replaced), {} slices \
             ({} over bytes, {} over time, {} moved), {:.0} MB buffered at peak",
            stats.download_bytes as f64 / 1_048_576.0,
            stats.peak_streams,
            stats.streams_opened,
            stats.streams_replaced,
            stats.slices,
            stats.slices_over_bytes,
            stats.slices_over_time,
            stats.slices_moved,
            stats.max_buffered_bytes as f64 / 1_048_576.0,
        );
    } else {
        println!("  download: one request");
    }
    for pool in &stats.pools {
        println!(
            "  {} scan: decrypt {} | tree {} | other {} ({} outputs, {} grafted, {} roots checked)",
            pool.pool,
            duration(pool.decrypt_s),
            duration(pool.tree_s),
            duration(pool.other_s),
            pool.outputs,
            pool.grafted,
            pool.verified,
        );
    }
    let blocks: u64 = stats.backends.iter().map(|b| b.blocks).sum();
    if blocks > 0 {
        println!(
            "  {:<16}{:>7}{:>8}{:>6}{:>10}{:>7}{:>13}",
            "backend", "blocks", "slices", "head", "blocks/s", "MB/s", "first block"
        );
        for b in &stats.backends {
            let per_s = |n: f64| if b.transfer_s > 0.0 { n / b.transfer_s } else { 0.0 };
            println!(
                "  {:<16}{:>6.0}%{:>8}{:>6}{:>10.0}{:>7.1}{:>10.0} ms",
                b.backend,
                100.0 * b.blocks as f64 / blocks as f64,
                b.slices,
                b.head_slices,
                per_s(b.blocks as f64),
                per_s(b.bytes as f64 / 1_048_576.0),
                if b.waits > 0 { 1000.0 * b.wait_s / b.waits as f64 } else { 0.0 },
            );
        }
    }
}

fn versus(path: &PathBuf, total_s: f64, stats: &SyncStats) -> Result<()> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {path:?}"))?;
    let other: Value = serde_json::from_str(&text).with_context(|| format!("parsing {path:?}"))?;
    let number = |key: &str| other.get(key).and_then(Value::as_f64);
    let other_total = number("total_s").ok_or_else(|| anyhow!("{path:?} has no total_s"))?;
    let seconds = |s: Option<f64>| s.map_or("—".to_string(), duration);
    // The figures both engines have: the scan busy, and the scan waiting
    // for blocks from the download.
    println!();
    println!(
        "  {:<12}{:>10}{:>10}{:>16}{:>12}",
        "engine", "sync", "scan", "scan waiting", "blocks"
    );
    println!(
        "  {:<12}{:>10}{:>10}{:>16}{:>12}",
        "zkool",
        duration(total_s),
        duration(stats.scan_busy_s),
        duration(stats.scan_wait_s),
        stats.blocks
    );
    println!(
        "  {:<12}{:>10}{:>10}{:>16}{:>12}",
        "slipstream",
        duration(other_total),
        seconds(number("scan_s")),
        seconds(number("scan_recv_wait_s")),
        number("blocks").map_or("—".to_string(), |b| format!("{b:.0}")),
    );
    let (fast, slow, name) = if total_s <= other_total {
        (total_s, other_total, "zkool")
    } else {
        (other_total, total_s, "slipstream")
    };
    println!("  {name} is {:.2}x faster", slow / fast.max(0.001));
    // Everything else Slipstream measured, as it names it.
    let shown = ["total_s", "scan_s", "scan_recv_wait_s", "blocks"];
    let rest: Vec<String> = other
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(key, _)| !shown.contains(&key.as_str()))
        .filter_map(|(key, value)| {
            let n = value.as_f64().filter(|n| *n != 0.0)?;
            let n = if n.fract() == 0.0 { format!("{n}") } else { format!("{n:.2}") };
            Some(format!("{key}={n}"))
        })
        .collect();
    for line in rest.chunks(6) {
        println!("  slipstream: {}", line.join(" "));
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider().install_default().ok();
    let args = parse_args()?;
    let stderr = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")));
    // Buffered, so writing the trace slows the download as little as it can;
    // flushed before exit.
    let trace_file = match &args.trace {
        Some(path) => Some(Arc::new(Mutex::new(BufWriter::new(
            File::create(path).with_context(|| path.display().to_string())?,
        )))),
        None => None,
    };
    let trace = trace_file.clone().map(|file| {
        let level = if args.trace_blocks { "trace" } else { "debug" };
        tracing_subscriber::fmt::layer()
            .json()
            .with_span_list(true)
            .with_current_span(false)
            .with_writer(move || TraceWriter(file.clone()))
            .with_filter(EnvFilter::new(format!("rlz::sync::fetch={level},rlz::net::lwd=debug")))
    });
    tracing_subscriber::registry().with(stderr).with(trace).init();
    let result = run(args).await;
    if let Some(file) = trace_file {
        file.lock().unwrap().flush()?;
    }
    result
}

/// The trace file, shared by every event written to it.
struct TraceWriter(Arc<Mutex<BufWriter<File>>>);

impl Write for TraceWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}

async fn run(args: Args) -> Result<()> {
    let dir = std::env::temp_dir().join(format!("zkool-sync-bench-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let db = dir.join("wallet.db").to_string_lossy().to_string();
    rlz::api::db::create_db(&db, None, args.coin).await?;
    let coin = Coin::new(Some(args.coin))
        .open_database(db, None)
        .await?
        .set_lwd(0, args.server.clone())?;

    if args.tip {
        println!("{}", rlz::api::network::get_current_height(&coin).await?);
        let _ = std::fs::remove_dir_all(&dir);
        return Ok(());
    }

    let account = new_account(
        &NewAccount {
            icon: None,
            name: "bench".into(),
            restore: false,
            key: args.seed.clone(),
            passphrase: None,
            fingerprint: None,
            aindex: 0,
            birth: Some(args.birthday),
            folder: String::new(),
            pools: Some(POOLS),
            use_internal: Some(false),
            internal: false,
            hw: 0,
        },
        &coin,
    )
    .await?;
    let tip = rlz::api::network::get_current_height(&coin).await?;
    eprintln!("syncing account {account} from {} to {tip} against {}", args.birthday, args.server);

    let timed =
        rlz::sync::bench::sync(&coin, vec![account], tip, args.fast).await?;
    let height = match timed.outcome {
        SyncOutcome::Synced { height } => height,
        other => bail!("the sync did not complete: {other:?}"),
    };
    let total_s = timed.total.as_secs_f64();
    report(total_s, height, &timed.stats);

    if let Some(path) = &args.json {
        let result = json!({
            "engine": "zkool",
            "server": args.server,
            "birthday": args.birthday,
            "synced_height": height,
            "fast": args.fast,
            // Slipstream's names for the figures both engines have.
            "total_s": total_s,
            "fetch_s": timed.stats.download_wait_s,
            "scan_s": timed.stats.scan_busy_s,
            "blocks": timed.stats.blocks,
            "stats": timed.stats,
        });
        std::fs::write(path, serde_json::to_string_pretty(&result)?)?;
        eprintln!("wrote {}", path.display());
    }
    if let Some(path) = &args.versus {
        versus(path, total_s, &timed.stats)?;
    }
    if args.keep {
        eprintln!("wallet kept at {}", dir.display());
    } else {
        let _ = std::fs::remove_dir_all(&dir);
    }
    Ok(())
}
