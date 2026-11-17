use std::{collections::HashSet, time::Duration};

use anyhow::{Context as _, Result};
use shielded::Synchronizer;
pub use shielded::{ScanTiming, SubtreeRoots, SUBTREE_DEPTH};
use sqlx::SqliteConnection;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tracing::debug;
use zcash_protocol::consensus::{NetworkUpgrade, Parameters};
use zcash_trees::network::Network;

use orchard::issuance::auth::IssueValidatingKey;
use orchard::issuance::auth::ZSASchnorr;
use orchard::note::AssetBase;
use orchard::note::AssetId;

use crate::{
    lwd::CompactBlock,
    warp::hasher::{OrchardHasher, SaplingHasher},
};
use zcash_trees::types::{BlockHeader, Issuance, WarpSyncMessage};

use super::legacy::CommitmentTreeFrontier;

pub use zcash_trees::types::SyncError;

mod shielded;

pub type SaplingSync = Synchronizer<shielded::sapling::SaplingProtocol>;
pub type OrchardSync = Synchronizer<shielded::orchard::OrchardProtocol>;
pub type IronwoodSync = Synchronizer<shielded::ironwood::IronwoodProtocol>;

pub enum BlockMessage {
    Chunk(Vec<CompactBlock>),
    SaveHeader(BlockHeader),
    /// The user cancelled. The scan reads the token itself (see
    /// [`next_message`]); the reader never sends this.
    Cancel,
    /// The stream ended before `end`: `reached` is the last height seen.
    Interrupted { reached: u32 },
    /// The stream failed after `reached`.
    Failed { reached: u32, error: anyhow::Error },
    Reorg(Vec<u32>, u32),
}

/// How a scan ended, for the caller to decide what comes next. A scan that
/// did not reach its end is an error, not a variant: the run must not look
/// finished (#1474).
#[derive(Debug, PartialEq, Eq)]
pub enum ScanEnd {
    /// Every block up to `end` was processed.
    Completed,
    /// The user cancelled; whatever was committed stays.
    Cancelled,
    /// A block did not connect to the previous one. The rewind was sent to
    /// the writer; the caller restarts the scan from where the database
    /// stands after it. `detected_at` is the height that did not connect.
    Reorg { detected_at: u32 },
}

/// The blocks read go to the scan this long after the previous hand-off:
/// one scan pass and one commit, so this is also how far apart the
/// checkpoints are and how often the progress moves.
const FLUSH_INTERVAL: Duration = Duration::from_secs(15);
/// Or sooner, once they hold this many spends, outputs, actions and
/// issuances: what bounds the blocks held when the download is fast.
const FLUSH_ACTIONS: usize = 100_000;

/// When the reader hands the scan the blocks it has read.
#[derive(Clone, Copy, Debug)]
struct Flush {
    /// This long after the previous hand-off.
    every: Duration,
    /// Or once the blocks waiting count this many scanned fields.
    max_actions: usize,
}

/// What [`read_blocks`] needs to know of the range it reads.
struct BlockReader {
    start_height: u32,
    end_height: u32,
    account_ids: Vec<u32>,
    /// The hash each account holds for `start_height - 1`.
    frontier_hashes: Vec<Vec<u8>>,
    heights_without_time: HashSet<u32>,
    flush: Flush,
}

/// Hand the blocks read so far to the scan and start a new chunk. False
/// when the scan is gone or the user cancelled while it was busy: the
/// blocks are dropped, to be downloaded again by the next sync.
async fn hand_off(
    tx_blocks: &tokio::sync::mpsc::Sender<BlockMessage>,
    cancel: &CancellationToken,
    bs: &mut Vec<CompactBlock>,
    count: &mut usize,
) -> bool {
    let chunk = BlockMessage::Chunk(std::mem::take(bs));
    *count = 0;
    tokio::select! {
        biased;
        _ = cancel.cancelled() => false,
        sent = tx_blocks.send(chunk) => sent.is_ok(),
    }
}

/// The scan's next message; `None` once the reader is done. A cancel wins
/// over the chunks already queued, so the scan stops after the chunk in
/// hand instead of after everything downloaded.
async fn next_message(
    rx_blocks: &mut tokio::sync::mpsc::Receiver<BlockMessage>,
    cancel: &CancellationToken,
) -> Option<BlockMessage> {
    tokio::select! {
        biased;
        _ = cancel.cancelled() => Some(BlockMessage::Cancel),
        message = rx_blocks.recv() => message,
    }
}

/// Read `blocks` and hand them to the scan in chunks (see [`Flush`]),
/// checking that each block connects to the one before. Ends with the
/// stream, on a cancel, or with the message that says why it stopped.
async fn read_blocks(
    mut blocks: impl tokio_stream::Stream<Item = anyhow::Result<CompactBlock>> + Unpin,
    tx_blocks: tokio::sync::mpsc::Sender<BlockMessage>,
    cancel: CancellationToken,
    reader: BlockReader,
) {
    let BlockReader {
        start_height,
        end_height,
        account_ids,
        frontier_hashes,
        mut heights_without_time,
        flush,
    } = reader;
    let mut prev_hash: Option<Vec<u8>> = None;
    let mut bs = vec![];

    // A timer of its own, not a check made when a block arrives: what was
    // read is scanned and saved even while the download is stalled.
    let flush_at = tokio::time::sleep(flush.every);
    tokio::pin!(flush_at);
    // The last height seen; `start_height - 1` until the first block.
    let mut current_height = start_height.saturating_sub(1);

    let mut c = 0;
    // [bench instrumentation] download lane: cumulative time awaiting blocks
    // from the network, and the reader task's total wall.
    let mut download_wait = std::time::Duration::ZERO;
    let mut blocks_read: u64 = 0;
    let reader_started = std::time::Instant::now();
    loop {
        let iter_start = std::time::Instant::now();
        tokio::select! {
            _ = cancel.cancelled() => {
                // The scan reads the token too; the blocks not handed to
                // it are dropped.
                debug!("Sync cancelled");
                break;
            }
            _ = &mut flush_at => {
                if !bs.is_empty() && !hand_off(&tx_blocks, &cancel, &mut bs, &mut c).await {
                    break;
                }
                flush_at.as_mut().reset(tokio::time::Instant::now() + flush.every);
            }
            m = blocks.next() => {
                download_wait += iter_start.elapsed();
                blocks_read += 1;
                let block = match m {
                    Some(Ok(block)) => block,
                    Some(Err(error)) => {
                        // Keep what was scanned, then report the break.
                        if !bs.is_empty() && !hand_off(&tx_blocks, &cancel, &mut bs, &mut c).await {
                            break;
                        }
                        let _ = tx_blocks.send(BlockMessage::Failed { reached: current_height, error }).await;
                        break;
                    }
                    None => {
                        if !bs.is_empty() && !hand_off(&tx_blocks, &cancel, &mut bs, &mut c).await {
                            break;
                        }
                        if current_height < end_height {
                            debug!("block stream ended at {} of {}", current_height, end_height);
                            let _ = tx_blocks.send(BlockMessage::Interrupted { reached: current_height }).await;
                        } else {
                            debug!("no more blocks to process");
                        }
                        break;
                    }
                };
                let block_prev_hash = block.prev_hash.clone();
                current_height = block.height as u32;
                let discontinuity = match &prev_hash {
                    Some(hash) => hash != &block_prev_hash,
                    None => frontier_hashes.iter().any(|hash| hash != &block_prev_hash),
                };
                if discontinuity {
                    let _ = tx_blocks.send(BlockMessage::Reorg(account_ids, current_height - 1)).await;
                    debug!("Reorganization detected at block {}", block.height);
                    break;
                }
                prev_hash = Some(block.hash.clone());

                let bheight = block.height as u32;
                if heights_without_time.remove(&bheight) {
                    let bh = BlockHeader {
                        height: bheight,
                        hash: block.hash.clone(),
                        time: block.time,
                    };
                    let _ = tx_blocks.send(BlockMessage::SaveHeader(bh)).await;
                }

                // Every field the scanners process counts toward the
                // chunk, or a range of Ironwood-only blocks would not be
                // bounded (V12 F-183).
                for vtx in block.vtx.iter() {
                    c += vtx.spends.len();
                    c += vtx.outputs.len();
                    c += vtx.actions.len();
                    c += vtx.ironwood_actions.len();
                    c += vtx.issuances.len();
                }

                bs.push(block);

                if c >= flush.max_actions {
                    if !hand_off(&tx_blocks, &cancel, &mut bs, &mut c).await {
                        break;
                    }
                    flush_at.as_mut().reset(tokio::time::Instant::now() + flush.every);
                }
            }
        }
    }

    tracing::debug!(
        download_wait_ms = download_wait.as_millis() as u64,
        reader_wall_ms = reader_started.elapsed().as_millis() as u64,
        "zkool download lane: wait=awaiting blocks from network, wall=reader task total"
    );
    crate::sync::bench::record(|stats| {
        stats.blocks += blocks_read;
        stats.download_wait_s += download_wait.as_secs_f64();
    });
    debug!("warp_sync block reader done");
}

/// How long the scan has waited for blocks so far. The scan writes it, the
/// block download reads it: seen from the download side the scan looks
/// hungry even while it is busy, because blocks are batched in between.
#[derive(Debug)]
pub struct ScanWait {
    base: std::time::Instant,
    waited_ns: std::sync::atomic::AtomicU64,
    /// Nanoseconds since `base` at which the current wait began, plus one;
    /// zero while the scan is not waiting.
    since_ns: std::sync::atomic::AtomicU64,
}

impl ScanWait {
    pub fn new() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            base: std::time::Instant::now(),
            waited_ns: 0.into(),
            since_ns: 0.into(),
        })
    }

    fn now_ns(&self) -> u64 {
        self.base.elapsed().as_nanos() as u64
    }

    fn begin(&self) {
        self.since_ns
            .store(self.now_ns() + 1, std::sync::atomic::Ordering::Relaxed);
    }

    fn end(&self) {
        let since = self.since_ns.swap(0, std::sync::atomic::Ordering::Relaxed);
        if since != 0 {
            let waited = self.now_ns().saturating_sub(since - 1);
            self.waited_ns
                .fetch_add(waited, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Total time waited, the wait in progress included.
    pub fn total(&self) -> Duration {
        let mut ns = self.waited_ns.load(std::sync::atomic::Ordering::Relaxed);
        let since = self.since_ns.load(std::sync::atomic::Ordering::Relaxed);
        if since != 0 {
            ns += self.now_ns().saturating_sub(since - 1);
        }
        Duration::from_nanos(ns)
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn warp_sync(
    network: &Network,
    connection: &mut SqliteConnection,
    start_height: u32,
    end_height: u32,
    tip: u32,
    accounts: &[(u32, bool)],
    blocks: ReceiverStream<anyhow::Result<CompactBlock>>,
    heights_without_time: HashSet<u32>,
    sapling_state: &CommitmentTreeFrontier,
    orchard_state: &CommitmentTreeFrontier,
    ironwood_state: &CommitmentTreeFrontier,
    subtree_roots: [SubtreeRoots; 3],
    tx_decrypted: Sender<WarpSyncMessage>,
    cancel: CancellationToken,
    scan_wait: Option<std::sync::Arc<ScanWait>>,
) -> Result<ScanEnd, SyncError> {
    let sap_hasher = SaplingHasher::default();

    let mut sap_dec = SaplingSync::new(
        *network,
        &mut *connection,
        1,
        start_height,
        accounts,
        tx_decrypted.clone(),
        sapling_state.size() as u32,
        sapling_state.to_edge(&sap_hasher),
    )
    .await?;

    let orch_hasher = OrchardHasher::default();
    let mut orch_dec = OrchardSync::new(
        *network,
        &mut *connection,
        2,
        start_height,
        accounts,
        tx_decrypted.clone(),
        orchard_state.size() as u32,
        orchard_state.to_edge(&orch_hasher),
    )
    .await?;

    let ironwood_active = network.activation_height(NetworkUpgrade::Nu6_3).is_some();
    let ironwood_hasher = OrchardHasher::default();
    let mut ironwood_dec = if ironwood_active {
        Some(
            IronwoodSync::new(
                *network,
                &mut *connection,
                3,
                start_height,
                accounts,
                tx_decrypted.clone(),
                ironwood_state.size() as u32,
                ironwood_state.to_edge(&ironwood_hasher),
            )
            .await?,
        )
    } else {
        None
    };

    let [sapling_roots, orchard_roots, ironwood_roots] = subtree_roots;
    sap_dec.subtree_roots = sapling_roots;
    orch_dec.subtree_roots = orchard_roots;
    if let Some(dec) = ironwood_dec.as_mut() {
        dec.subtree_roots = ironwood_roots;
    }
    let ironwood_has_keys = ironwood_dec.as_ref().map_or(false, |d| !d.has_no_keys());
    if sap_dec.has_no_keys() && orch_dec.has_no_keys() && !ironwood_has_keys {
        debug!("No keys to sync");
        return Ok(ScanEnd::Completed);
    }

    // Check every account: accounts at the same height can still be on different chains.
    let mut frontier_hashes = Vec::new();
    for (account, _) in accounts {
        let hash = crate::sync::chain_log::header_hash(&mut *connection, *account, start_height - 1)
            .await
            .context("reading account frontier header")?;
        if let Some(hash) = hash {
            frontier_hashes.push(hash);
        }
    }
    let account_ids = accounts.iter().map(|(id, _)| *id).collect::<Vec<_>>();
    let (tx_blocks, mut rx_blocks) = tokio::sync::mpsc::channel::<BlockMessage>(2);
    tokio::spawn(read_blocks(
        blocks,
        tx_blocks,
        cancel.clone(),
        BlockReader {
            start_height,
            end_height,
            account_ids,
            frontier_hashes,
            heights_without_time,
            flush: Flush { every: FLUSH_INTERVAL, max_actions: FLUSH_ACTIONS },
        },
    ));

    let mut end = ScanEnd::Completed;
    // [bench instrumentation] Split the scan lane into compute-busy vs
    // starved-waiting-for-downloaded-blocks. `scan_wait` is time blocked on the
    // reader task (the download lane); `scan_busy` is trial-decrypt + tree work.
    // A large scan_wait => download-bound; scan_busy ~= total => compute-bound.
    let mut scan_busy = std::time::Duration::ZERO;
    let scan_gauge = scan_wait;
    let mut scan_wait = std::time::Duration::ZERO;
    let mut scan_chunks: u64 = 0;
    loop {
        let waited = std::time::Instant::now();
        if let Some(gauge) = &scan_gauge {
            gauge.begin();
        }
        let received = next_message(&mut rx_blocks, &cancel).await;
        if let Some(gauge) = &scan_gauge {
            gauge.end();
        }
        let Some(bm) = received else { break };
        scan_wait += waited.elapsed();
        let busy = std::time::Instant::now();
        match bm {
            BlockMessage::Chunk(bs) => {
                scan_chunks += 1;
                debug!("Processing {} blocks", bs.len());

                // Send Issuance messages for asset storage before any Note
                // messages (same transaction ordering guarantee).
                for cb in &bs {
                    for vtx in &cb.vtx {
                        for iss in &vtx.issuances {
                            let desc_hash: [u8; 32] =
                                iss.asset_desc_hash.as_slice().try_into().unwrap();
                            let ik = IssueValidatingKey::<ZSASchnorr>::decode(&iss.ik)
                                .expect("invalid issuer key in issuance");
                            let asset_id = AssetId::new_v0(&ik, &desc_hash);
                            let asset_base = AssetBase::custom(&asset_id);
                            tx_decrypted
                                .send(WarpSyncMessage::Issuance(Issuance {
                                    asset_desc_hash: iss.asset_desc_hash.clone(),
                                    ik: iss.ik.clone(),
                                    asset_base: asset_base.to_bytes().to_vec(),
                                    finalized: iss.finalize,
                                    height: cb.height as u32,
                                }))
                                .await
                                .context("sending issuance")?;
                        }
                    }
                }

                sap_dec.add(&bs).await?;
                orch_dec.add(&bs).await?;
                if let Some(ref mut ironwood_dec) = ironwood_dec {
                    ironwood_dec.add(&bs).await?;
                }

                let lcb = bs.last().unwrap();
                let bh = BlockHeader {
                    height: lcb.height as u32,
                    hash: lcb.hash.clone(),
                    time: lcb.time,
                };
                let _ = tx_decrypted.send(WarpSyncMessage::BlockHeader(bh)).await;
                let _ = tx_decrypted.send(WarpSyncMessage::Commit).await;
            }
            BlockMessage::Cancel => {
                debug!("Cancelling...");
                end = ScanEnd::Cancelled;
                break;
            }
            BlockMessage::Interrupted { reached } => {
                return Err(SyncError::Other(anyhow::anyhow!(
                    "the block stream ended at height {reached}, {end_height} asked"
                )));
            }
            BlockMessage::Failed { reached, error } => {
                return Err(SyncError::Other(error.context(format!(
                    "the block stream failed after height {reached}, {end_height} asked"
                ))));
            }
            BlockMessage::Reorg(accounts, height) => {
                // A fork cannot reach a final block: the server's chain does
                // not connect to what the accounts hold, and trimming would
                // only rescan what is already right.
                if crate::sync::chain_log::is_final(height, tip) {
                    return Err(SyncError::Other(anyhow::anyhow!(
                        "the server's block {} does not connect to the final block {height} \
                         the accounts hold (tip {tip}): not rewinding",
                        height + 1
                    )));
                }
                let _ = tx_decrypted
                    .send(WarpSyncMessage::Rewind(accounts, height))
                    .await;
                end = ScanEnd::Reorg { detected_at: height + 1 };
                break;
            }
            BlockMessage::SaveHeader(bh) => {
                let _ = tx_decrypted.send(WarpSyncMessage::BlockHeader(bh)).await;
            }
        }
        scan_busy += busy.elapsed();
    }
    tracing::debug!(
        scan_busy_ms = scan_busy.as_millis() as u64,
        scan_wait_ms = scan_wait.as_millis() as u64,
        scan_chunks,
        "zkool scan lane: busy=decrypt+tree, wait=starved waiting for downloaded blocks"
    );
    let mut timings = vec![("sapling", &sap_dec.timing), ("orchard", &orch_dec.timing)];
    if let Some(ironwood_dec) = &ironwood_dec {
        timings.push(("ironwood", &ironwood_dec.timing));
    }
    crate::sync::bench::record(|stats| {
        stats.scan_chunks += scan_chunks;
        stats.scan_busy_s += scan_busy.as_secs_f64();
        stats.scan_wait_s += scan_wait.as_secs_f64();
        for (pool, timing) in timings {
            match stats.pools.iter_mut().find(|p| p.pool == pool) {
                Some(known) => known.add(timing),
                None => {
                    let mut scan =
                        crate::sync::bench::PoolScan { pool: pool.to_string(), ..Default::default() };
                    scan.add(timing);
                    stats.pools.push(scan);
                }
            }
        }
    });

    Ok(end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lwd::{CompactOrchardAction, CompactTx};
    use tokio::sync::mpsc;
    use tokio::time::timeout;

    const SOON: Duration = Duration::from_secs(5);
    const NEVER: Duration = Duration::from_secs(3600);

    /// Block `height` with `actions` Orchard actions, connected to the one
    /// before it.
    fn block(height: u32, actions: usize) -> CompactBlock {
        CompactBlock {
            height: height as u64,
            hash: height.to_le_bytes().to_vec(),
            prev_hash: (height - 1).to_le_bytes().to_vec(),
            vtx: vec![CompactTx {
                actions: vec![CompactOrchardAction::default(); actions],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    struct Reader {
        /// The download: blocks sent here reach the reader.
        blocks: mpsc::Sender<anyhow::Result<CompactBlock>>,
        /// The scan's end of the reader's channel.
        scan: mpsc::Receiver<BlockMessage>,
        cancel: CancellationToken,
        task: tokio::task::JoinHandle<()>,
    }

    /// A reader of heights 101 to 200 whose scan holds `queued` messages
    /// before the reader waits on it.
    fn reader(flush: Flush, queued: usize) -> Reader {
        let (blocks, download) = mpsc::channel(64);
        let (tx_blocks, scan) = mpsc::channel(queued);
        let cancel = CancellationToken::new();
        let task = tokio::spawn(read_blocks(
            ReceiverStream::new(download),
            tx_blocks,
            cancel.clone(),
            BlockReader {
                start_height: 101,
                end_height: 200,
                account_ids: vec![1],
                frontier_hashes: vec![],
                heights_without_time: HashSet::new(),
                flush,
            },
        ));
        Reader { blocks, scan, cancel, task }
    }

    fn heights(message: Option<BlockMessage>) -> Vec<u64> {
        match message {
            Some(BlockMessage::Chunk(bs)) => bs.iter().map(|b| b.height).collect(),
            Some(_) => panic!("not a chunk"),
            None => panic!("the reader ended without a chunk"),
        }
    }

    #[tokio::test]
    async fn blocks_go_to_the_scan_when_the_interval_ends_though_no_more_arrive() {
        let flush = Flush { every: Duration::from_millis(300), max_actions: usize::MAX };
        let mut r = reader(flush, 2);
        for h in 101..=103 {
            r.blocks.send(Ok(block(h, 1))).await.unwrap();
        }
        // The download stalls here: nothing more is sent, the stream stays open.
        assert!(
            timeout(Duration::from_millis(100), r.scan.recv()).await.is_err(),
            "flushed before the interval"
        );
        let chunk = timeout(SOON, r.scan.recv()).await.expect("no flush on the timer");
        assert_eq!(heights(chunk), vec![101, 102, 103]);

        // The next interval starts at that hand-off.
        r.blocks.send(Ok(block(104, 1))).await.unwrap();
        assert!(timeout(Duration::from_millis(100), r.scan.recv()).await.is_err());
        let chunk = timeout(SOON, r.scan.recv()).await.expect("no second flush");
        assert_eq!(heights(chunk), vec![104]);
    }

    #[tokio::test]
    async fn blocks_go_to_the_scan_at_the_cap_without_waiting_for_the_interval() {
        let flush = Flush { every: NEVER, max_actions: 10 };
        let mut r = reader(flush, 2);
        for h in 101..=104 {
            r.blocks.send(Ok(block(h, 4))).await.unwrap();
        }
        // Twelve actions at the third block; the fourth starts a new chunk.
        let chunk = timeout(SOON, r.scan.recv()).await.expect("no flush at the cap");
        assert_eq!(heights(chunk), vec![101, 102, 103]);
        assert!(timeout(Duration::from_millis(100), r.scan.recv()).await.is_err());
    }

    #[tokio::test]
    async fn a_cancel_ends_a_stalled_download_and_drops_what_was_not_scanned() {
        let mut r = reader(Flush { every: NEVER, max_actions: usize::MAX }, 2);
        r.blocks.send(Ok(block(101, 1))).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        r.cancel.cancel();
        timeout(SOON, r.task).await.expect("the reader hung on the stalled download").unwrap();
        assert!(r.scan.recv().await.is_none(), "the block was handed to the scan");
    }

    #[tokio::test]
    async fn a_cancel_ends_a_hand_off_the_scan_is_not_taking() {
        // Each block is a chunk; the scan takes none, so the second waits.
        let mut r = reader(Flush { every: NEVER, max_actions: 1 }, 1);
        for h in 101..=103 {
            r.blocks.send(Ok(block(h, 1))).await.unwrap();
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(!r.task.is_finished());
        r.cancel.cancel();
        timeout(SOON, r.task).await.expect("the reader hung on the busy scan").unwrap();
        // What the scan would do next: the cancel, not the chunk queued.
        assert!(matches!(next_message(&mut r.scan, &r.cancel).await, Some(BlockMessage::Cancel)));
    }

    #[tokio::test]
    async fn the_end_of_the_stream_hands_over_the_last_blocks() {
        let mut r = reader(Flush { every: NEVER, max_actions: usize::MAX }, 2);
        for h in 101..=200 {
            r.blocks.send(Ok(block(h, 0))).await.unwrap();
        }
        drop(r.blocks);
        let chunk = timeout(SOON, r.scan.recv()).await.unwrap();
        assert_eq!(heights(chunk).len(), 100);
        assert!(timeout(SOON, r.scan.recv()).await.unwrap().is_none());
    }
}
