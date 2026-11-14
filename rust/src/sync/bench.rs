//! A timed sync and what it spent its time on, for benchmarks.
//!
//! The sync's stages add to one process-wide record as they finish;
//! [`sync`] clears it, runs a sync through the wallet's driver, and returns
//! what was recorded. A benchmark runs one sync at a time, which is all this
//! record can tell apart.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::Serialize;

use crate::api::coin::Coin;
use crate::api::sync::SyncOutcome;

/// Where a sync's time went. Seconds are summed over the ranges scanned.
#[derive(Clone, Debug, Default, Serialize)]
pub struct SyncStats {
    /// Compact blocks handed to the scan.
    pub blocks: u64,
    /// Batches the scan processed.
    pub scan_chunks: u64,
    /// The block reader waiting for the next block from the download.
    pub download_wait_s: f64,
    /// The scan decrypting and building trees.
    pub scan_busy_s: f64,
    /// The scan waiting for blocks.
    pub scan_wait_s: f64,
    /// The writer committing to the database.
    pub write_busy_s: f64,
    pub write_commits: u64,

    /// The multi-connection download; all zero when one request served the range.
    pub download_bytes: u64,
    pub streams_opened: u64,
    pub peak_streams: u64,
    pub slices: u64,
    pub slices_over_bytes: u64,
    pub slices_over_time: u64,
    pub slices_moved: u64,
    pub slices_requeued: u64,
    pub streams_replaced: u64,
    pub budget_raises: u64,
    pub max_buffered_bytes: u64,
}

static STATS: Mutex<Option<SyncStats>> = Mutex::new(None);

/// Add to the record of the sync in progress.
pub(crate) fn record(update: impl FnOnce(&mut SyncStats)) {
    let mut stats = STATS.lock().unwrap_or_else(|e| e.into_inner());
    update(stats.get_or_insert_with(SyncStats::default));
}

fn take() -> SyncStats {
    STATS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take()
        .unwrap_or_default()
}

/// The result of [`sync`].
#[derive(Debug)]
pub struct TimedSync {
    pub outcome: SyncOutcome,
    /// Wall time of the sync call.
    pub total: Duration,
    pub stats: SyncStats,
}

/// Sync `accounts` to `current_height` through the wallet's driver, as the
/// server does, and report the time and where it went.
pub async fn sync(
    coin: &Coin,
    accounts: Vec<u32>,
    current_height: u32,
    actions_per_sync: u32,
    skip_details: bool,
) -> Result<TimedSync> {
    take();
    let (progress, _) = tokio::sync::mpsc::channel(1);
    let started = Instant::now();
    let outcome = coin
        .driver()?
        .sync(
            coin,
            super::driver::SyncRequest {
                accounts,
                current_height,
                actions_per_sync,
                checkpoint_age: 10_000,
                skip_details,
            },
            progress,
        )
        .await?;
    let total = started.elapsed();
    Ok(TimedSync { outcome, total, stats: take() })
}
