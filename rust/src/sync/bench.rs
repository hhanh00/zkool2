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
    /// The scan's time in each pool, by phase.
    pub pools: Vec<PoolScan>,
    /// What each backend served, most blocks first: the server's own name
    /// for it where it gives one (`x-backend`), else the connection.
    pub backends: Vec<BackendStats>,
}

/// Where the scan of one pool spent its time. The pools are scanned one
/// after the other, so their times add up to most of `scan_busy_s`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct PoolScan {
    pub pool: String,
    /// Outputs (or actions) trial-decrypted, each against every key.
    pub outputs: u64,
    /// Trial decryption, on all cores.
    pub decrypt_s: f64,
    /// Hashing the note commitment tree and updating witnesses.
    pub tree_s: f64,
    /// The rest: issuance, and handing notes, witnesses and spends to the
    /// writer, waiting on it when it is behind.
    pub other_s: f64,
    /// Leaves not hashed: their subtree's root, from the server, was grafted.
    pub grafted: u64,
    /// Subtrees hashed whose root the server gave too: the two matched.
    pub verified: u64,
}

impl PoolScan {
    pub(crate) fn add(&mut self, timing: &crate::warp::sync::ScanTiming) {
        self.outputs += timing.outputs;
        self.decrypt_s += timing.decrypt.as_secs_f64();
        self.tree_s += timing.tree.as_secs_f64();
        self.other_s += timing.other.as_secs_f64();
        self.grafted += timing.grafted;
        self.verified += timing.verified;
    }
}

/// What one backend served in the download.
#[derive(Clone, Debug, Default, Serialize)]
pub struct BackendStats {
    pub backend: String,
    pub slices: u64,
    /// Slices that held the next block the scan needed.
    pub head_slices: u64,
    pub blocks: u64,
    pub bytes: u64,
    /// From each slice's first block to its last.
    pub transfer_s: f64,
    /// Waits for a slice's first block, summed, over `waits` slices.
    pub wait_s: f64,
    pub waits: u64,
}

impl BackendStats {
    pub(crate) fn add(&mut self, other: &BackendStats) {
        self.slices += other.slices;
        self.head_slices += other.head_slices;
        self.blocks += other.blocks;
        self.bytes += other.bytes;
        self.transfer_s += other.transfer_s;
        self.wait_s += other.wait_s;
        self.waits += other.waits;
    }
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
