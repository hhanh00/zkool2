//! Block download for the shielded scan: one ordered stream of compact
//! blocks, fetched over several connections when the range is long.
//!
//! The scan reads blocks strictly in height order, so only the blocks it
//! needs next are worth fetching in parallel. A long range is fetched as
//! consecutive slices from one shared cursor. Each stream claims the next
//! slice when it becomes free, sized to take about [`FetchConfig::piece`] at
//! that stream's own recent rate, and only while the bytes in flight and
//! buffered stay within a budget. Blocks are reordered and delivered in
//! height order, so the consumer sees the stream a single request would give.
//!
//! The number of streams is not tuned on throughput, which is too noisy to
//! compare over short periods. What the scanner and the buffer show decides:
//!
//! | Scanner | Buffered ahead           | Means            | Action                   |
//! |---------|--------------------------|------------------|--------------------------|
//! | waiting | large, a gap at the head | head-of-line     | move the head slice      |
//! | waiting | small, streams busy      | fetch-bound      | add a stream             |
//! | waiting | small, streams parked    | budget too tight | raise the budget         |
//! | busy    | any                      | healthy          | hold                     |
//! | any     | a stream failed          | server trouble   | fewer streams, stop adding |

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use prost::Message as _;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt as _;
use tracing::{debug, debug_span, info, trace, Instrument as _};

use super::routes::{Cost, Routes};
use crate::api::coin::Network;
use crate::lwd::CompactBlock;
use crate::warp::sync::ScanWait;
use crate::Client;

/// Opens one more connection to the block server.
pub type ClientFactory =
    Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Result<Client>> + Send>> + Send + Sync>;

type BlockStream = ReceiverStream<Result<CompactBlock>>;

/// Ranges shorter than this are one request on the caller's connection.
pub const PARALLEL_MIN_BLOCKS: u32 = 10_000;
/// A stream's first slice, before its rate is known, and the smallest slice.
/// Small enough to be safe where blocks are tens of kilobytes each.
const PROBE_BLOCKS: u32 = 64;
/// Scanner-wait fraction of a window above which the scanner counts as waiting.
const STARVED_FRACTION: f64 = 0.25;
/// A slice is aborted once it received this many times its planned bytes…
const OVERRUN_FACTOR: u64 = 2;
/// …or, while nothing is known to plan with, this many bytes…
const UNPLANNED_MAX_BYTES: u64 = 4 * 1024 * 1024;
/// …or once it took this many times its planned time.
const BEHIND_FACTOR: f64 = 1.5;
/// A stream is slow when its backend is expected to fetch below this share
/// of the best one it could reach (or, where the server is not steered, of
/// the other streams' median).
const SLOW_RATIO: f64 = 0.5;
/// Consecutive slow slices before a stream is replaced.
const SLOW_SLICES: u32 = 2;
/// A head slice is moved when a free stream is expected to fetch the rest
/// of it this many times sooner.
const MOVE_HEAD_FACTOR: f64 = 2.0;
/// Heights per bucket of the block sizes seen.
const SIZE_BUCKET: u32 = 1000;
/// Look-ahead is never held under this many blocks.
const MIN_LOOKAHEAD_BLOCKS: f64 = 10_000.0;

#[derive(Clone, Debug)]
pub struct FetchConfig {
    /// How long one slice should take to download.
    pub piece: Duration,
    /// Ranges shorter than this start as one request on the caller's
    /// connection, with no stream opened unless that request is aborted.
    pub parallel_min_blocks: u32,
    /// Streams started at once.
    pub initial_streams: usize,
    /// Most streams, however long the scanner waits.
    pub max_streams: usize,
    /// Starting budget for bytes in flight plus buffered.
    pub budget: u64,
    /// The budget's ceiling, and the most bytes buffered before reading stops.
    pub mem_cap: u64,
    /// Length of one observation window.
    pub window: Duration,
    /// Longest a stream may go without a block while it has a slice.
    pub piece_stall: Duration,
    /// Regions to ask the server's front end for, where it routes by region
    /// (see [`super::routes`]).
    pub regions: Vec<String>,
    /// How long a region may go unmeasured before it is tried again.
    pub reprobe: Duration,
    /// Slices are handed out no further ahead of the scan than it reads in
    /// this long, so the fastest streams work near what it needs next.
    pub lookahead: Duration,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            piece: Duration::from_secs(3),
            parallel_min_blocks: PARALLEL_MIN_BLOCKS,
            initial_streams: 6,
            max_streams: 16,
            budget: 32 * 1024 * 1024,
            mem_cap: 128 * 1024 * 1024,
            window: Duration::from_secs(5),
            piece_stall: Duration::from_secs(30),
            // Fly.io regions where zec.rocks had backends (November 2026).
            // A server that does not route by region never names a backend,
            // and is then not steered.
            regions: ["syd", "nrt", "lax", "iah", "fra", "ams", "gru"]
                .map(String::from)
                .to_vec(),
            reprobe: Duration::from_secs(30),
            lookahead: Duration::from_secs(10),
        }
    }
}

/// What a download did, for logs and tests.
#[derive(Default, Debug)]
pub struct FetchStats {
    /// Streams opened.
    pub connections: AtomicU64,
    /// Most streams alive at once.
    pub peak_connections: AtomicU64,
    /// Slices handed out.
    pub pieces: AtomicU64,
    /// Slices cut short for overrunning their byte estimate.
    pub cuts: AtomicU64,
    /// Slices given back by a failed stream.
    pub requeued: AtomicU64,
    /// Slices aborted for taking too long.
    pub timeouts: AtomicU64,
    /// Head slices moved off a stream that fell behind.
    pub steals: AtomicU64,
    /// Streams replaced for staying slow.
    pub replaced: AtomicU64,
    /// Times the budget was raised.
    pub budget_raises: AtomicU64,
    /// Most bytes held undelivered at once.
    pub max_buffered: AtomicU64,
}

/// The blocks of `start..=end` in height order. Without `more_clients` this
/// is one request on `client`. With it, a long range is fetched over several
/// connections (see the module doc); a short one starts as one request on
/// `client` and moves to several only if that request is aborted.
pub async fn block_stream(
    network: &Network,
    client: &mut Client,
    start: u32,
    end: u32,
    more_clients: Option<ClientFactory>,
    scan_wait: Option<Arc<ScanWait>>,
) -> Result<BlockStream> {
    match more_clients {
        Some(factory) if end >= start => {
            block_stream_with(
                network,
                client,
                start,
                end,
                factory,
                FetchConfig::default(),
                Arc::new(FetchStats::default()),
                scan_wait,
            )
            .await
        }
        _ => client.block_range(network, start, end).await,
    }
}

pub(crate) async fn block_stream_with(
    network: &Network,
    client: &mut Client,
    start: u32,
    end: u32,
    factory: ClientFactory,
    cfg: FetchConfig,
    stats: Arc<FetchStats>,
    scan_wait: Option<Arc<ScanWait>>,
) -> Result<BlockStream> {
    // The caller's connection fetches the first slice; the streams opened
    // here take everything after it. A short range is that one slice: like
    // any slice it is aborted if it takes too long, and streams take over.
    let short = end - start < cfg.parallel_min_blocks.saturating_sub(1);
    let first_end = if short { end } else { start.saturating_add(PROBE_BLOCKS - 1).min(end) };
    let span = debug_span!("slice", stream = "first", start, end = first_end);
    let first = client
        .block_range(network, start, first_end)
        .instrument(span)
        .await?;
    let (out, rx) = mpsc::channel(10);
    let (events_tx, events_rx) = mpsc::channel(64);
    let now = Instant::now();
    let routes = Routes::new(cfg.regions.clone(), cfg.reprobe);
    let fetch = Fetch {
        network: *network,
        end,
        budget: cfg.budget.min(cfg.mem_cap),
        target: if short { 0 } else { cfg.initial_streams.max(1) },
        ceiling: cfg.max_streams.max(1),
        cfg,
        stats,
        factory,
        out,
        first: Some(first),
        first_next: start,
        first_end,
        started: now,
        frontier: first_end.saturating_add(1),
        returned: BTreeMap::new(),
        buffer: BTreeMap::new(),
        buffered: 0,
        next: start,
        contig: start,
        avg_block: 0.0,
        sizes: BTreeMap::new(),
        routes,
        served: BTreeMap::new(),
        delivered_window: 0,
        delivery_rate: None,
        workers: BTreeMap::new(),
        next_id: 0,
        events_tx,
        events_rx,
        last_error: None,
        starved_since: None,
        starved_acc: Duration::ZERO,
        received_window: 0,
        received_total: 0,
        buffered_sum: 0,
        buffered_samples: 0,
        parked_window: false,
        parked: false,
        last_steal: now,
        avoid_head: None,
        replacements: 0,
        scan_wait,
        scan_wait_seen: Duration::ZERO,
    };
    tokio::spawn(fetch.run());
    Ok(ReceiverStream::new(rx))
}

enum Event {
    /// The connection is open.
    Ready(usize),
    /// The server answered a slice's request, from this backend if it says.
    Answered { id: usize, backend: Option<String> },
    Block { id: usize, block: CompactBlock, bytes: u64 },
    /// The slice is complete (or was stopped at its cut).
    Done(usize),
    /// The stream is gone; its slice is unfinished.
    Failed { id: usize, error: anyhow::Error },
}

struct Piece {
    start: u32,
    /// Next height expected.
    next: u32,
    /// Last height of the slice.
    end: u32,
    /// Blocks it had when handed out.
    blocks: u32,
    planned: u64,
    received: u64,
    started: Instant,
    first_block: Option<Instant>,
    /// Why it was stopped short, if it was.
    ended_by: Option<&'static str>,
    /// Whether it held the next block to deliver when handed out.
    head: bool,
    /// The backend that answered, where the server says.
    backend: Option<String>,
}

struct Worker {
    assign: mpsc::Sender<(u32, u32)>,
    /// The worker stops its stream after a block at or above this height.
    stop: Arc<AtomicU32>,
    ready: bool,
    piece: Option<Piece>,
    /// The region its connection asks for.
    region: Option<String>,
    /// The backend that answered its last request, where the server says.
    backend: Option<String>,
    /// Slices finished.
    slices: u32,
    /// Consecutive slices well below the other streams' rate.
    slow_slices: u32,
    spawned: Instant,
}

struct Fetch {
    network: Network,
    end: u32,
    cfg: FetchConfig,
    stats: Arc<FetchStats>,
    factory: ClientFactory,
    out: mpsc::Sender<Result<CompactBlock>>,

    /// The first slice, on the caller's connection.
    first: Option<BlockStream>,
    first_next: u32,
    first_end: u32,
    started: Instant,

    /// First height never assigned.
    frontier: u32,
    /// Unfinished ranges given back, `start -> end`, all below `frontier`.
    returned: BTreeMap<u32, u32>,

    /// Blocks received and not yet delivered.
    buffer: BTreeMap<u32, (CompactBlock, u64)>,
    buffered: u64,
    /// Next height to deliver.
    next: u32,
    /// First height at or above `next` that is not in `buffer`.
    contig: u32,
    /// Recent bytes per block.
    avg_block: f64,
    /// Bytes and blocks received per `SIZE_BUCKET` heights.
    sizes: BTreeMap<u32, (u64, u32)>,
    /// What each backend costs, and which region to connect to.
    routes: Routes,
    /// What each backend served, for the benchmark's record.
    served: BTreeMap<String, super::bench::BackendStats>,
    delivered_window: u64,
    /// Blocks per second the scan has been reading.
    delivery_rate: Option<f64>,

    workers: BTreeMap<usize, Worker>,
    next_id: usize,
    /// Streams wanted.
    target: usize,
    /// Most streams `target` may grow to; lowered when a stream fails.
    ceiling: usize,
    /// Bytes in flight plus buffered that new slices must fit in.
    budget: u64,
    events_tx: mpsc::Sender<Event>,
    events_rx: mpsc::Receiver<Event>,
    last_error: Option<anyhow::Error>,

    starved_since: Option<Instant>,
    starved_acc: Duration,
    received_window: u64,
    received_total: u64,
    buffered_sum: u128,
    buffered_samples: u64,
    /// Whether a free stream was refused a slice by the budget this window.
    parked_window: bool,
    /// Whether the last assignment pass stopped at the budget.
    parked: bool,
    last_steal: Instant,
    /// The stream the head was just taken from: not given it straight back.
    avoid_head: Option<(usize, Instant)>,
    replacements: u32,
    /// The scan's own measure of its waiting, when it reports one.
    scan_wait: Option<Arc<ScanWait>>,
    scan_wait_seen: Duration,
}

async fn next_block(first: &mut Option<BlockStream>) -> Option<Result<CompactBlock>> {
    match first {
        Some(stream) => stream.next().await,
        None => std::future::pending().await,
    }
}

impl Fetch {
    async fn run(mut self) {
        let mut window = tokio::time::interval(self.cfg.window);
        window.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        window.tick().await;
        // Wakes the loop so a stalled head is noticed without waiting for a block.
        let mut poll = tokio::time::interval(self.cfg.piece / 4);
        poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut window_started = Instant::now();
        // A handle of its own, so a send permit does not borrow `self`.
        let out = self.out.clone();
        loop {
            if self.next > self.end {
                break;
            }
            self.expire();
            self.top_up();
            self.steal_head();
            self.assign();
            let deliverable = self.contig > self.next;
            if !deliverable && self.first.is_none() && self.workers.is_empty() {
                // Nothing left that could fetch the next block. What arrived
                // in order was delivered; the consumer sees where it stopped.
                if let Some(error) = self.last_error.take() {
                    let _ = out.send(Err(error)).await;
                }
                break;
            }
            // Waiting: nothing to hand over while the consumer has (nearly)
            // emptied what it was given. A block just sent may still sit in
            // the channel, so "empty" would never be seen from here.
            let starved = !deliverable && out.capacity() * 2 >= out.max_capacity();
            self.note_starved(starved);
            self.buffered_sum += u128::from(self.buffered);
            self.buffered_samples += 1;
            // Past the cap only the blocks that fill the gap are still read.
            let poll_events =
                !self.workers.is_empty() && (self.buffered < self.cfg.mem_cap || !deliverable);
            let poll_first = self.first.is_some() && !deliverable;
            tokio::select! {
                biased;
                permit = out.reserve(), if deliverable => {
                    let Ok(permit) = permit else { break };
                    if let Some((block, bytes)) = self.buffer.remove(&self.next) {
                        trace!(t_ms = self.t_ms(), height = self.next, bytes, "fetch: delivered");
                        self.buffered -= bytes;
                        self.delivered_window += 1;
                        self.next += 1;
                        permit.send(Ok(block));
                    }
                }
                _ = out.closed(), if !deliverable => break,
                event = self.events_rx.recv(), if poll_events => {
                    if let Some(event) = event {
                        self.on_event(event);
                    }
                }
                item = next_block(&mut self.first), if poll_first => self.on_first(item),
                _ = window.tick() => {
                    self.on_window(window_started.elapsed());
                    window_started = Instant::now();
                }
                _ = poll.tick() => {}
            }
        }
        debug!(
            connections = self.stats.connections.load(Ordering::Relaxed),
            peak_connections = self.stats.peak_connections.load(Ordering::Relaxed),
            pieces = self.stats.pieces.load(Ordering::Relaxed),
            cuts = self.stats.cuts.load(Ordering::Relaxed),
            requeued = self.stats.requeued.load(Ordering::Relaxed),
            timeouts = self.stats.timeouts.load(Ordering::Relaxed),
            steals = self.stats.steals.load(Ordering::Relaxed),
            replaced = self.stats.replaced.load(Ordering::Relaxed),
            budget_raises = self.stats.budget_raises.load(Ordering::Relaxed),
            max_buffered_bytes = self.stats.max_buffered.load(Ordering::Relaxed),
            "block download done"
        );
        let load = |counter: &AtomicU64| counter.load(Ordering::Relaxed);
        super::bench::record(|stats| {
            stats.download_bytes += self.received_total;
            stats.streams_opened += load(&self.stats.connections);
            stats.peak_streams = stats.peak_streams.max(load(&self.stats.peak_connections));
            stats.slices += load(&self.stats.pieces);
            stats.slices_over_bytes += load(&self.stats.cuts);
            stats.slices_over_time += load(&self.stats.timeouts);
            stats.slices_moved += load(&self.stats.steals);
            stats.slices_requeued += load(&self.stats.requeued);
            stats.streams_replaced += load(&self.stats.replaced);
            stats.budget_raises += load(&self.stats.budget_raises);
            stats.max_buffered_bytes = stats.max_buffered_bytes.max(load(&self.stats.max_buffered));
            for (backend, served) in &self.served {
                match stats.backends.iter_mut().find(|b| b.backend == *backend) {
                    Some(known) => known.add(served),
                    None => stats.backends.push(super::bench::BackendStats {
                        backend: backend.clone(),
                        ..served.clone()
                    }),
                }
            }
            stats.backends.sort_by_key(|b| std::cmp::Reverse(b.blocks));
        });
    }

    /// Milliseconds since the download started, for the trace.
    fn t_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    /// Who is to fetch `height`: a stream, the first slice, or nobody yet.
    fn holder(&self, height: u32) -> String {
        if let Some(id) = self.workers.iter().find_map(|(id, w)| {
            let piece = w.piece.as_ref()?;
            (piece.next <= height && height <= piece.end).then_some(*id)
        }) {
            return format!("stream {id}");
        }
        if self.first.is_some() && self.first_next <= height && height <= self.first_end {
            return "first".to_string();
        }
        match self.returned.range(..=height).next_back() {
            Some((_, &end)) if height <= end => "returned".to_string(),
            _ => "unassigned".to_string(),
        }
    }

    fn note_starved(&mut self, starved: bool) {
        match (self.starved_since, starved) {
            (None, true) => {
                self.starved_since = Some(Instant::now());
                debug!(
                    t_ms = self.t_ms(),
                    height = self.next,
                    holder = self.holder(self.contig),
                    buffered_bytes = self.buffered,
                    frontier = self.frontier,
                    "fetch: scan waiting"
                );
            }
            (Some(since), false) => {
                self.starved_acc += since.elapsed();
                self.starved_since = None;
                debug!(
                    t_ms = self.t_ms(),
                    height = self.next,
                    waited_ms = since.elapsed().as_millis() as u64,
                    "fetch: scan resumed"
                );
            }
            _ => {}
        }
    }

    fn insert(&mut self, height: u32, block: CompactBlock, bytes: u64) {
        if height < self.next || self.buffer.contains_key(&height) {
            return;
        }
        self.buffer.insert(height, (block, bytes));
        self.buffered += bytes;
        self.received_window += bytes;
        self.received_total += bytes;
        self.stats.max_buffered.fetch_max(self.buffered, Ordering::Relaxed);
        let size = self.sizes.entry(height / SIZE_BUCKET).or_default();
        size.0 += bytes;
        size.1 += 1;
        self.avg_block = if self.avg_block == 0.0 {
            bytes as f64
        } else {
            self.avg_block + (bytes as f64 - self.avg_block) / 64.0
        };
        if height == self.contig {
            self.contig += 1;
            while self.buffer.contains_key(&self.contig) {
                self.contig += 1;
            }
        }
    }

    fn on_first(&mut self, item: Option<Result<CompactBlock>>) {
        match item {
            Some(Ok(block)) => {
                let height = block.height as u32;
                let bytes = block.encoded_len() as u64;
                trace!(
                    t_ms = self.t_ms(),
                    stream = "first",
                    height,
                    bytes,
                    "fetch: block"
                );
                self.first_next = height.saturating_add(1);
                self.insert(height, block, bytes);
                if height >= self.first_end {
                    self.first = None;
                    debug!(t_ms = self.t_ms(), end = height, "fetch: first slice done");
                }
            }
            Some(Err(error)) => {
                self.last_error = Some(error);
                self.drop_first();
            }
            None => {
                self.last_error = Some(anyhow!(
                    "the block stream ended at height {}, {} asked",
                    self.first_next.saturating_sub(1),
                    self.first_end
                ));
                self.drop_first();
            }
        }
    }

    /// Give up on the first slice: what it did not deliver is unassigned.
    fn drop_first(&mut self) {
        debug!(
            t_ms = self.t_ms(),
            next = self.first_next,
            end = self.first_end,
            error = self.last_error.as_ref().map(|e| format!("{e:#}")),
            "fetch: first slice dropped"
        );
        self.first = None;
        if self.first_next <= self.first_end {
            self.returned.insert(self.first_next, self.first_end);
            if self.target == 0 {
                self.target = self.cfg.initial_streams.max(1);
            }
        }
    }

    fn on_event(&mut self, event: Event) {
        match event {
            Event::Ready(id) => {
                let t_ms = self.t_ms();
                if let Some(worker) = self.workers.get_mut(&id) {
                    worker.ready = true;
                    debug!(
                        t_ms,
                        stream = id,
                        connect_ms = worker.spawned.elapsed().as_millis() as u64,
                        "fetch: stream ready"
                    );
                }
            }
            Event::Answered { id, backend } => {
                let t_ms = self.t_ms();
                let Some(worker) = self.workers.get_mut(&id) else {
                    return;
                };
                worker.backend = backend.clone();
                if let Some(piece) = worker.piece.as_mut() {
                    piece.backend = backend.clone();
                }
                let region = worker.region.clone();
                self.routes.answered(region.as_deref(), backend.as_deref());
                debug!(t_ms, stream = id, region, backend, "fetch: slice answered");
            }
            Event::Block { id, block, bytes } => {
                let height = block.height as u32;
                let t_ms = self.t_ms();
                let head = height == self.contig;
                let Some(piece) = self.workers.get_mut(&id).and_then(|w| w.piece.as_mut()) else {
                    return;
                };
                if height > piece.end {
                    return; // sent before the worker saw its slice was cut
                }
                if height != piece.next {
                    let error = anyhow!("block {height} received, {} expected", piece.next);
                    self.fail_worker(id, error);
                    return;
                }
                piece.next = height + 1;
                piece.received += bytes;
                trace!(t_ms, stream = id, height, bytes, head, "fetch: block");
                if piece.first_block.is_none() {
                    piece.first_block = Some(Instant::now());
                    debug!(
                        t_ms,
                        stream = id,
                        start = piece.start,
                        first_block_ms = piece.started.elapsed().as_millis() as u64,
                        "fetch: slice first block"
                    );
                }
                let limit = if piece.planned > 0 {
                    OVERRUN_FACTOR * piece.planned
                } else {
                    UNPLANNED_MAX_BYTES
                };
                let overrun = piece.received > limit && piece.next <= piece.end;
                self.insert(height, block, bytes);
                if overrun {
                    self.cut(id, "over_bytes");
                    self.stats.cuts.fetch_add(1, Ordering::Relaxed);
                }
            }
            Event::Done(id) => {
                let t_ms = self.t_ms();
                let Some(worker) = self.workers.get_mut(&id) else {
                    return;
                };
                let Some(piece) = worker.piece.take() else {
                    return;
                };
                worker.slices += 1;
                let key = Routes::key(piece.backend.as_deref(), id);
                let blocks = piece.next - piece.start;
                let elapsed = piece.started.elapsed().as_secs_f64();
                let first_block = piece
                    .first_block
                    .map(|first| (first - piece.started).as_secs_f64());
                let transfer = piece
                    .first_block
                    .map_or(0.0, |first| first.elapsed().as_secs_f64());
                let cost = self.routes.costs.entry(key.clone()).or_default();
                let predicted = cost.seconds(f64::from(blocks), piece.received as f64);
                // A slice ended before its first block tells only that the
                // wait was at least this long.
                let wait = first_block.or((elapsed > cost.first_block()).then_some(elapsed));
                cost.observe(wait, blocks, piece.received, transfer);
                let served = self.served.entry(key.clone()).or_default();
                served.slices += 1;
                served.head_slices += u64::from(piece.head);
                served.blocks += u64::from(blocks);
                served.bytes += piece.received;
                served.transfer_s += transfer;
                if let Some(wait) = first_block {
                    served.wait_s += wait;
                    served.waits += 1;
                }
                debug!(
                    t_ms,
                    stream = id,
                    start = piece.start,
                    end = piece.end,
                    blocks_planned = piece.blocks,
                    blocks = piece.next - piece.start,
                    planned_bytes = piece.planned,
                    bytes = piece.received,
                    elapsed_ms = piece.started.elapsed().as_millis() as u64,
                    first_block_ms = piece
                        .first_block
                        .map(|first| (first - piece.started).as_millis() as u64),
                    predicted_ms = predicted.map(|s| (s * 1000.0) as u64),
                    backend = key,
                    head = piece.head,
                    reason = piece.ended_by.unwrap_or("done"),
                    "fetch: slice ended"
                );
                if piece.next <= piece.end {
                    self.returned.insert(piece.next, piece.end);
                }
                if piece.first_block.is_some() {
                    self.judge(id);
                }
            }
            Event::Failed { id, error } => self.fail_worker(id, error),
        }
    }

    /// Abort every slice past its time limit, wherever it is in the range.
    fn expire(&mut self) {
        let limit = self.cfg.piece.mul_f64(BEHIND_FACTOR);
        if self.first.is_some() && self.started.elapsed() > limit {
            debug!("block download: first slice timed out at {}", self.first_next);
            self.drop_first();
            self.stats.timeouts.fetch_add(1, Ordering::Relaxed);
        }
        let late: Vec<usize> = self
            .workers
            .iter()
            .filter(|(_, w)| {
                w.piece
                    .as_ref()
                    .is_some_and(|p| p.next <= p.end && p.started.elapsed() > limit)
            })
            .map(|(id, _)| *id)
            .collect();
        for id in late {
            debug!(stream = id, "block download: slice timed out");
            self.cut(id, "timed_out");
            self.stats.timeouts.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Stop a stream's slice at the last block it delivered; the rest goes
    /// back to the queue. What it did is counted when the stream reports it
    /// stopped, so its backend's cost follows a slice that went badly too.
    fn cut(&mut self, id: usize, reason: &'static str) {
        let t_ms = self.t_ms();
        let Some(worker) = self.workers.get_mut(&id) else {
            return;
        };
        let Some(piece) = worker.piece.as_mut() else {
            return;
        };
        if piece.next > piece.end {
            return;
        }
        piece.ended_by.get_or_insert(reason);
        debug!(
            t_ms,
            stream = id,
            start = piece.start,
            at = piece.next,
            end = piece.end,
            bytes = piece.received,
            planned_bytes = piece.planned,
            elapsed_ms = piece.started.elapsed().as_millis() as u64,
            reason,
            "fetch: slice cut"
        );
        self.returned.insert(piece.next, piece.end);
        piece.end = piece.next - 1;
        worker.stop.store(piece.end, Ordering::SeqCst);
    }

    /// Replace a stream whose backend stays well below the best: where the
    /// server routes by region, by one to the best region; elsewhere by a
    /// new connection, which may land on a better path.
    fn judge(&mut self, id: usize) {
        let piece = self.cfg.piece.as_secs_f64();
        let block_bytes = self.block_bytes(self.frontier);
        let Some(worker) = self.workers.get(&id) else {
            return;
        };
        let key = Routes::key(worker.backend.as_deref(), id);
        let Some(rate) = self
            .routes
            .costs
            .get(&key)
            .filter(|cost| cost.measured(piece))
            .and_then(|cost| cost.blocks_per_second(piece, block_bytes))
        else {
            return;
        };
        let best = if self.routes.steerable() {
            self.routes.best_region_rate(piece, block_bytes)
        } else {
            let mut others: Vec<f64> = self
                .workers
                .keys()
                .filter(|other| **other != id)
                .filter_map(|other| self.stream_rate(*other, block_bytes))
                .collect();
            others.sort_by(f64::total_cmp);
            (others.len() >= 2).then(|| others[others.len() / 2])
        };
        let Some(best) = best else {
            return;
        };
        let Some(worker) = self.workers.get_mut(&id) else {
            return;
        };
        if rate < SLOW_RATIO * best {
            worker.slow_slices += 1;
        } else {
            worker.slow_slices = 0;
        }
        if worker.slow_slices >= SLOW_SLICES
            && (self.replacements as usize) < 2 * self.cfg.max_streams
        {
            info!(
                "block download: stream {id} on {key} stayed slow ({rate:.0} blocks/s, \
                 best {best:.0}): replacing it"
            );
            self.workers.remove(&id);
            debug!(
                t_ms = self.t_ms(),
                stream = id,
                backend = key,
                blocks_per_s = rate as u64,
                best_blocks_per_s = best as u64,
                "fetch: stream replaced"
            );
            self.replacements += 1;
            self.stats.replaced.fetch_add(1, Ordering::Relaxed);
            let region = self.next_region();
            self.spawn_worker(region);
        }
    }

    /// Where the server routes by region and one has gone unmeasured for a
    /// while, move the slowest free stream there to measure it again.
    fn reprobe(&mut self) {
        if !self.routes.steerable() || self.workers.len() < 2 {
            return;
        }
        let in_use: Vec<&str> = self
            .workers
            .values()
            .filter_map(|w| w.region.as_deref())
            .collect();
        let Some(region) = self.routes.stale_region(&in_use) else {
            return;
        };
        let block_bytes = self.block_bytes(self.frontier);
        let slowest = self
            .workers
            .iter()
            .filter(|(_, w)| w.ready && w.piece.is_none())
            .map(|(id, _)| (*id, self.stream_rate(*id, block_bytes).unwrap_or(0.0)))
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let Some((id, _)) = slowest else {
            return;
        };
        self.routes.probed();
        self.workers.remove(&id);
        debug!(
            t_ms = self.t_ms(),
            stream = id,
            region,
            "fetch: region probed again"
        );
        self.spawn_worker(Some(region));
    }

    /// Drop a stream; the rest of its slice goes back to the queue. A failed
    /// stream is the server's signal to ask for less.
    fn fail_worker(&mut self, id: usize, error: anyhow::Error) {
        let Some(worker) = self.workers.remove(&id) else {
            return;
        };
        debug!(
            t_ms = self.t_ms(),
            stream = id,
            start = worker.piece.as_ref().map(|p| p.start),
            at = worker.piece.as_ref().map(|p| p.next),
            end = worker.piece.as_ref().map(|p| p.end),
            error = format!("{error:#}"),
            "fetch: stream failed"
        );
        if let Some(piece) = worker.piece {
            if piece.next <= piece.end {
                self.returned.insert(piece.next, piece.end);
                self.stats.requeued.fetch_add(1, Ordering::Relaxed);
            }
        }
        self.target = self.target.saturating_sub(1);
        self.ceiling = self.ceiling.min(self.target.max(1));
        info!(
            "block download: stream {id} dropped ({} wanted now): {error:#}",
            self.target
        );
        self.last_error = Some(error);
    }

    /// Bytes per block expected at `height`: what arrived nearest below it.
    fn block_bytes(&self, height: u32) -> f64 {
        self.sizes
            .range(..=height / SIZE_BUCKET)
            .next_back()
            .map(|(_, &(bytes, blocks))| bytes as f64 / f64::from(blocks.max(1)))
            .unwrap_or(self.avg_block)
            .max(1.0)
    }

    /// The cost a stream is expected to have: its backend's once it has
    /// one, else that of the backend its region reached last.
    fn stream_cost(&self, id: usize) -> Option<&Cost> {
        let worker = self.workers.get(&id)?;
        let own = self
            .routes
            .costs
            .get(&Routes::key(worker.backend.as_deref(), id));
        match own {
            Some(cost) if cost.samples() > 0 => Some(cost),
            _ => self.routes.cost_of(worker.region.as_deref()?),
        }
    }

    /// Blocks per second a stream is expected to fetch.
    fn stream_rate(&self, id: usize, block_bytes: f64) -> Option<f64> {
        self.stream_cost(id)?
            .blocks_per_second(self.cfg.piece.as_secs_f64(), block_bytes)
    }

    /// Blocks in a stream's next slice from `start`: what its backend is
    /// expected to fetch in a slice's time. A backend not measured enough
    /// yet gets half a usual slice, long enough to measure it, and a probe
    /// while nothing is known at all.
    fn piece_blocks(&self, id: usize, start: u32) -> u32 {
        if self.avg_block == 0.0 {
            return PROBE_BLOCKS;
        }
        let piece = self.cfg.piece.as_secs_f64();
        let block_bytes = self.block_bytes(start);
        let planned = |cost: &Cost| cost.blocks_in(cost.slice_seconds(piece), block_bytes);
        let cost = self.stream_cost(id);
        let blocks = cost
            .filter(|cost| cost.measured(piece))
            .and_then(planned)
            .or_else(|| self.routes.typical_blocks(piece, block_bytes).map(|b| b / 2.0))
            .or_else(|| cost.and_then(planned));
        let Some(blocks) = blocks else {
            return PROBE_BLOCKS;
        };
        let most = self.cfg.mem_cap as f64 / 4.0 / block_bytes;
        (blocks.min(most).ceil().min(u32::MAX as f64) as u32).max(PROBE_BLOCKS)
    }

    /// The region for the next connection (see [`Routes::pick_region`]).
    fn next_region(&mut self) -> Option<String> {
        let block_bytes = self.block_bytes(self.frontier);
        self.routes
            .pick_region(self.cfg.piece.as_secs_f64(), block_bytes)
    }

    fn has_unassigned(&self) -> bool {
        !self.returned.is_empty() || self.frontier <= self.end
    }

    fn spawn_worker(&mut self, region: Option<String>) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let (assign, assignments) = mpsc::channel(1);
        let stop = Arc::new(AtomicU32::new(u32::MAX));
        tokio::spawn(worker(
            id,
            self.network,
            self.factory.clone(),
            region.clone(),
            assignments,
            stop.clone(),
            self.events_tx.clone(),
            self.cfg.piece_stall,
        ));
        self.workers.insert(
            id,
            Worker {
                assign,
                stop,
                ready: false,
                piece: None,
                region: region.clone(),
                backend: None,
                slices: 0,
                slow_slices: 0,
                spawned: Instant::now(),
            },
        );
        self.stats.connections.fetch_add(1, Ordering::Relaxed);
        debug!(
            t_ms = self.t_ms(),
            stream = id,
            region,
            streams = self.workers.len(),
            "fetch: stream opened"
        );
        self.stats
            .peak_connections
            .fetch_max(self.workers.len() as u64, Ordering::Relaxed);
        id
    }

    /// Open streams up to the number wanted.
    fn top_up(&mut self) {
        while self.workers.len() < self.target && self.has_unassigned() {
            let region = self.next_region();
            self.spawn_worker(region);
        }
    }

    fn inflight_remaining(&self) -> u64 {
        self.workers
            .values()
            .filter_map(|w| w.piece.as_ref())
            .map(|p| p.planned.saturating_sub(p.received))
            .sum()
    }

    /// Hand the next slices to free streams, fastest first, so the head goes
    /// to the best of them, while they fit the budget and the look-ahead.
    fn assign(&mut self) {
        let block_bytes = self.block_bytes(self.frontier);
        let mut free: Vec<(usize, f64)> = self
            .workers
            .iter()
            .filter(|(_, w)| w.ready && w.piece.is_none())
            .map(|(id, _)| (*id, self.stream_rate(*id, block_bytes).unwrap_or(0.0)))
            .collect();
        free.sort_by(|a, b| b.1.total_cmp(&a.1));
        let free_count = free.len();
        let mut parked = false;
        let lookahead = self
            .delivery_rate
            .map(|rate| (rate * self.cfg.lookahead.as_secs_f64()).max(MIN_LOOKAHEAD_BLOCKS));
        for (id, _) in free {
            let (start, limit) = match self.returned.first_key_value() {
                Some((&start, &end)) => (start, end),
                None if self.frontier <= self.end => (self.frontier, self.end),
                None => break,
            };
            // The slice holding the next block to deliver is always fetched,
            // whatever the budget says: a full buffer must not wait on a gap.
            let head = start == self.contig;
            if head && self.workers.len() > 1 {
                if let Some((avoided, since)) = self.avoid_head {
                    if avoided == id && since.elapsed() < self.cfg.piece {
                        continue;
                    }
                }
            }
            if !head && lookahead.is_some_and(|ahead| f64::from(start - self.next) > ahead) {
                break;
            }
            let blocks = self.piece_blocks(id, start);
            let end = start.saturating_add(blocks - 1).min(limit);
            let planned = (f64::from(end - start + 1) * self.block_bytes(start)) as u64;
            let inflight = self.inflight_remaining();
            if !head && self.buffered + inflight + planned > self.budget {
                self.parked_window = true;
                parked = true;
                if !self.parked {
                    debug!(
                        t_ms = self.t_ms(),
                        free = free_count,
                        buffered_bytes = self.buffered,
                        inflight_bytes = inflight,
                        planned_bytes = planned,
                        budget_bytes = self.budget,
                        "fetch: budget parked"
                    );
                }
                break;
            }
            let requeued = self.returned.remove(&start).is_some();
            if requeued {
                if end < limit {
                    self.returned.insert(end + 1, limit);
                }
            } else {
                self.frontier = end + 1;
            }
            debug!(
                t_ms = self.t_ms(),
                stream = id,
                start,
                end,
                blocks = end - start + 1,
                planned_bytes = planned,
                predicted_ms = self
                    .stream_cost(id)
                    .and_then(|cost| cost.seconds(f64::from(end - start + 1), planned as f64))
                    .map(|s| (s * 1000.0) as u64),
                region = self.workers.get(&id).and_then(|w| w.region.clone()),
                backend = self.workers.get(&id).and_then(|w| w.backend.clone()),
                block_bytes = self.block_bytes(start) as u64,
                head,
                requeued,
                next = self.next,
                contig = self.contig,
                frontier = self.frontier,
                buffered_bytes = self.buffered,
                inflight_bytes = inflight,
                budget_bytes = self.budget,
                "fetch: slice assigned"
            );
            let Some(worker) = self.workers.get_mut(&id) else {
                continue;
            };
            worker.stop.store(u32::MAX, Ordering::SeqCst);
            worker.piece = Some(Piece {
                start,
                next: start,
                end,
                blocks: end - start + 1,
                planned,
                received: 0,
                started: Instant::now(),
                first_block: None,
                ended_by: None,
                head,
                backend: None,
            });
            self.stats.pieces.fetch_add(1, Ordering::Relaxed);
            if worker.assign.try_send((start, end)).is_err() {
                self.fail_worker(id, anyhow!("stream task ended"));
            }
        }
        if self.parked && !parked && free_count > 0 {
            debug!(
                t_ms = self.t_ms(),
                buffered_bytes = self.buffered,
                "fetch: budget unparked"
            );
        }
        if free_count > 0 {
            self.parked = parked;
        }
    }

    /// Whether a slice is on course to take much longer than planned.
    fn behind(&self, piece: &Piece) -> bool {
        let planned = self.cfg.piece.as_secs_f64();
        let elapsed = piece.started.elapsed().as_secs_f64();
        if elapsed < planned / 4.0 {
            return false;
        }
        let done = piece.next - piece.start;
        if done == 0 {
            return true;
        }
        elapsed * f64::from(piece.blocks) / f64::from(done) > BEHIND_FACTOR * planned
    }

    /// Head-of-line: the consumer waits on a slice whose stream fell behind.
    /// Take the rest of that slice from it; a free stream fetches it, or the
    /// stream working furthest ahead is stopped to do so. A stream that is
    /// slow on a slice nobody waits for is left alone.
    fn steal_head(&mut self) {
        if self.starved_since.is_none() || self.last_steal.elapsed() < self.cfg.piece / 2 {
            return;
        }
        let head = self.contig;
        let holder = self.workers.iter().find_map(|(id, w)| {
            let piece = w.piece.as_ref()?;
            (piece.next == head && piece.next <= piece.end).then_some(*id)
        });
        let Some(holder) = holder else {
            return;
        };
        let behind = self.workers[&holder]
            .piece
            .as_ref()
            .is_some_and(|p| self.behind(p));
        let better = self.better_for_head(holder);
        if !behind && better.is_none() {
            return;
        }
        let free = self
            .workers
            .iter()
            .any(|(id, w)| *id != holder && w.ready && w.piece.is_none());
        if !free {
            // Nobody is free. Stopping another stream only pays when blocks
            // past the gap are already here.
            if self.buffered == 0 {
                return;
            }
            let furthest = self
                .workers
                .iter()
                .filter(|(id, _)| **id != holder)
                .filter_map(|(id, w)| {
                    let piece = w.piece.as_ref()?;
                    (piece.next <= piece.end).then_some((piece.next, *id))
                })
                .max();
            let Some((_, furthest)) = furthest else {
                return;
            };
            self.cut(furthest, "yield_to_head");
        }
        debug!(
            holder,
            height = head,
            backend = self.workers.get(&holder).and_then(|w| w.backend.clone()),
            behind,
            expected_s = better.map(|(holder_s, _)| holder_s),
            elsewhere_s = better.map(|(_, other_s)| other_s),
            buffered_bytes = self.buffered,
            "block download: head slice fell behind: moving it"
        );
        self.cut(holder, "moved");
        self.avoid_head = Some((holder, Instant::now()));
        self.stats.steals.fetch_add(1, Ordering::Relaxed);
        self.last_steal = Instant::now();
    }

    /// Where the head slice's stream is expected to take much longer over
    /// the rest of it than a free stream would: the seconds each would take.
    fn better_for_head(&self, holder: usize) -> Option<(f64, f64)> {
        let piece = self.workers.get(&holder)?.piece.as_ref()?;
        let left = f64::from(piece.end - piece.next + 1);
        let bytes = left * self.block_bytes(piece.next);
        let done = piece.next - piece.start;
        // What it is doing now, once it has started; what its backend is
        // expected to do before.
        let holder_s = match piece.first_block {
            Some(first) if done > 0 => first.elapsed().as_secs_f64() * left / f64::from(done),
            _ => {
                let cost = self.stream_cost(holder)?;
                let waited = piece.started.elapsed().as_secs_f64();
                cost.seconds(left, bytes)? - waited.min(cost.first_block())
            }
        };
        if holder_s < self.cfg.piece.as_secs_f64() / 2.0 {
            return None; // nearly done: moving it would not pay
        }
        let other_s = self
            .workers
            .iter()
            .filter(|(id, w)| **id != holder && w.ready && w.piece.is_none())
            .filter_map(|(id, _)| self.stream_cost(*id)?.seconds(left, bytes))
            .min_by(f64::total_cmp)?;
        (other_s * MOVE_HEAD_FACTOR < holder_s).then_some((holder_s, other_s))
    }

    /// Whether enough of the range is left for another stream to help.
    fn worth_adding(&self) -> bool {
        let left = self.end.saturating_sub(self.frontier);
        let slice = self
            .workers
            .keys()
            .map(|id| self.piece_blocks(*id, self.frontier))
            .max()
            .unwrap_or(PROBE_BLOCKS);
        left > slice.saturating_mul(2)
    }

    /// End of a window: read the scanner and the buffer (see the module doc).
    fn on_window(&mut self, elapsed: Duration) {
        if let Some(since) = self.starved_since {
            self.starved_acc += since.elapsed();
            self.starved_since = Some(Instant::now());
        }
        let secs = elapsed.as_secs_f64().max(0.001);
        // What the scan measured, if it reports; else what is seen from here,
        // which reads high when blocks are batched before the scan.
        let waiting = match &self.scan_wait {
            Some(gauge) => {
                let total = gauge.total();
                let waited = total.saturating_sub(self.scan_wait_seen);
                self.scan_wait_seen = total;
                (waited.as_secs_f64() / secs).min(1.0)
            }
            None => (self.starved_acc.as_secs_f64() / secs).min(1.0),
        };
        let rate = self.received_window as f64 / secs;
        let buffered = if self.buffered_samples == 0 {
            self.buffered
        } else {
            (self.buffered_sum / u128::from(self.buffered_samples)) as u64
        };
        let parked = self.parked_window;
        let delivered = self.delivered_window as f64 / secs;
        self.delivery_rate = Some(
            self.delivery_rate
                .map_or(delivered, |old| (old + delivered) / 2.0),
        );
        self.delivered_window = 0;
        self.starved_acc = Duration::ZERO;
        self.received_window = 0;
        self.buffered_sum = 0;
        self.buffered_samples = 0;
        self.parked_window = false;
        // "Small": no more than the slices in flight would explain.
        let block_bytes = self.block_bytes(self.frontier);
        let blocks_per_s: f64 = self
            .workers
            .keys()
            .filter_map(|id| self.stream_rate(*id, block_bytes))
            .sum();
        let in_flight = (blocks_per_s * block_bytes * self.cfg.piece.as_secs_f64()) as u64;
        let small = buffered <= in_flight.max(1);
        for (backend, cost) in &self.routes.costs {
            if let Some((per_block, per_byte)) = cost.terms() {
                debug!(
                    t_ms = self.t_ms(),
                    backend,
                    samples = cost.samples(),
                    first_block_ms = (cost.first_block() * 1000.0) as u64,
                    per_block_us = (per_block * 1e6) as u64,
                    mb_per_s = (per_byte > 0.0).then(|| 1.0 / per_byte / 1048576.0),
                    blocks_per_s = cost
                        .blocks_per_second(self.cfg.piece.as_secs_f64(), block_bytes)
                        .map(|r| r as u64),
                    "fetch: backend cost"
                );
            }
        }
        self.reprobe();
        debug!(
            t_ms = self.t_ms(),
            rate_bps = rate as u64,
            waiting_pct = (waiting * 100.0) as u32,
            buffered_bytes = buffered,
            budget_bytes = self.budget,
            next = self.next,
            frontier = self.frontier,
            streams = self.workers.len(),
            busy = self.workers.values().filter(|w| w.piece.is_some()).count(),
            parked,
            "block download window"
        );
        if waiting <= STARVED_FRACTION || !small {
            return; // healthy, or head-of-line, which `steal_head` handles
        }
        if parked {
            if self.budget < self.cfg.mem_cap {
                self.budget = (self.budget + self.budget / 2).min(self.cfg.mem_cap);
                self.stats.budget_raises.fetch_add(1, Ordering::Relaxed);
                info!(
                    "block download: scanner waited {:.0}% with streams parked: budget now {} bytes",
                    waiting * 100.0,
                    self.budget
                );
            }
        } else if self.target < self.ceiling && self.worth_adding() {
            self.target += 1;
            info!(
                "block download: scanner waited {:.0}% with every stream busy: {} streams now",
                waiting * 100.0,
                self.target
            );
        }
    }
}

/// One stream: fetches the slices it is assigned, one at a time.
#[allow(clippy::too_many_arguments)]
async fn worker(
    id: usize,
    network: Network,
    factory: ClientFactory,
    region: Option<String>,
    mut assignments: mpsc::Receiver<(u32, u32)>,
    stop: Arc<AtomicU32>,
    events: mpsc::Sender<Event>,
    stall: Duration,
) {
    let failed = |error: anyhow::Error| Event::Failed { id, error };
    let mut client = match tokio::time::timeout(stall, factory()).await {
        Ok(Ok(client)) => client,
        Ok(Err(error)) => {
            let _ = events.send(failed(error)).await;
            return;
        }
        Err(_) => {
            let _ = events.send(failed(anyhow!("connecting timed out"))).await;
            return;
        }
    };
    client.prefer_region(region.as_deref());
    if events.send(Event::Ready(id)).await.is_err() {
        return;
    }
    while let Some((start, end)) = assignments.recv().await {
        let span = debug_span!("slice", stream = id, start, end);
        let mut stream = match client
            .block_range(&network, start, end)
            .instrument(span)
            .await
        {
            Ok(stream) => stream,
            Err(error) => {
                let _ = events.send(failed(error)).await;
                return;
            }
        };
        let answered = Event::Answered {
            id,
            backend: client.last_backend(),
        };
        if events.send(answered).await.is_err() {
            return;
        }
        let mut last = start.saturating_sub(1);
        loop {
            let item = match tokio::time::timeout(stall, stream.next()).await {
                Ok(item) => item,
                Err(_) => {
                    let error = anyhow!("no block for {} s after height {last}", stall.as_secs());
                    let _ = events.send(failed(error)).await;
                    return;
                }
            };
            match item {
                Some(Ok(block)) => {
                    let height = block.height as u32;
                    let bytes = block.encoded_len() as u64;
                    last = height;
                    if events.send(Event::Block { id, block, bytes }).await.is_err() {
                        return;
                    }
                    if height >= end || height >= stop.load(Ordering::SeqCst) {
                        break;
                    }
                }
                Some(Err(error)) => {
                    let _ = events.send(failed(error)).await;
                    return;
                }
                None => {
                    let error = anyhow!("the block stream ended at height {last}, {end} asked");
                    let _ = events.send(failed(error)).await;
                    return;
                }
            }
        }
        drop(stream);
        if events.send(Event::Done(id)).await.is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use tonic::async_trait;
    use zcash_primitives::transaction::Transaction;

    use super::*;
    use crate::net::LwdServer;

    type Sizes = Arc<dyn Fn(u32) -> usize + Send + Sync>;

    /// A block server that paces its blocks and can be told to break.
    #[derive(Clone)]
    struct Fake {
        delay: Duration,
        size: Sizes,
        /// The stream ends after this height.
        ends_after: Option<u32>,
        /// The stream fails after this height.
        fails_after: Option<u32>,
        /// The stream fails after this many blocks of a request.
        fails_after_blocks: Option<u32>,
        asked: Arc<Mutex<Vec<(u32, u32)>>>,
        /// Per region, the delay a block: a server that routes by region and
        /// names its backends (`be-<region>`).
        regions: Option<Arc<dyn Fn(&str) -> Duration + Send + Sync>>,
        region: Option<String>,
        /// The region each request went to.
        served: Arc<Mutex<Vec<String>>>,
    }

    impl Fake {
        fn new(delay_ms: u64, size: usize) -> Self {
            Self {
                delay: Duration::from_millis(delay_ms),
                size: Arc::new(move |_| size),
                ends_after: None,
                fails_after: None,
                fails_after_blocks: None,
                asked: Arc::new(Mutex::new(vec![])),
                regions: None,
                region: None,
                served: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    #[async_trait]
    impl LwdServer for Fake {
        type CompactBlockStream = ReceiverStream<Result<CompactBlock>>;
        type TransactionStream = ReceiverStream<Result<(u32, Transaction, usize)>>;

        async fn latest_height(&mut self) -> Result<u32> {
            unimplemented!()
        }
        async fn block(&mut self, _: &Network, _: u32) -> Result<CompactBlock> {
            unimplemented!()
        }
        async fn block_range(
            &mut self,
            _: &Network,
            start: u32,
            end: u32,
        ) -> Result<Self::CompactBlockStream> {
            self.asked.lock().unwrap().push((start, end));
            let mut fake = self.clone();
            if let (Some(delays), Some(region)) = (&self.regions, &self.region) {
                fake.delay = delays(region);
                self.served.lock().unwrap().push(region.clone());
            }
            let (tx, rx) = mpsc::channel(10);
            tokio::spawn(async move {
                for (sent, height) in (start..=end).enumerate() {
                    if fake.ends_after.is_some_and(|last| height > last) {
                        return;
                    }
                    if fake.fails_after.is_some_and(|last| height > last)
                        || fake.fails_after_blocks.is_some_and(|n| sent as u32 >= n)
                    {
                        let _ = tx.send(Err(anyhow!("connection reset"))).await;
                        return;
                    }
                    if !fake.delay.is_zero() {
                        tokio::time::sleep(fake.delay).await;
                    }
                    let block = CompactBlock {
                        height: u64::from(height),
                        header: vec![0u8; (fake.size)(height)],
                        ..Default::default()
                    };
                    if tx.send(Ok(block)).await.is_err() {
                        return;
                    }
                }
            });
            Ok(ReceiverStream::new(rx))
        }
        async fn transaction(&mut self, _: &Network, _: &[u8]) -> Result<(u32, Transaction)> {
            unimplemented!()
        }
        async fn post_transaction(&mut self, _: u32, _: &[u8]) -> Result<String> {
            unimplemented!()
        }
        async fn taddress_txs(
            &mut self,
            _: &Network,
            _: &str,
            _: u32,
            _: u32,
        ) -> Result<Self::TransactionStream> {
            unimplemented!()
        }
        async fn mempool_stream(&mut self, _: &Network) -> Result<Self::TransactionStream> {
            unimplemented!()
        }
        async fn tree_state(&mut self, _: u32) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
            unimplemented!()
        }

        fn prefer_region(&mut self, region: Option<&str>) {
            self.region = region.map(String::from);
        }

        fn last_backend(&self) -> Option<String> {
            self.regions.as_ref()?;
            Some(format!("be-{}", self.region.as_deref()?))
        }
    }

    fn cfg(mem_cap: u64) -> FetchConfig {
        FetchConfig {
            piece: Duration::from_millis(100),
            parallel_min_blocks: 0,
            initial_streams: 3,
            max_streams: 6,
            budget: mem_cap,
            mem_cap,
            window: Duration::from_millis(50),
            piece_stall: Duration::from_secs(5),
            regions: vec![],
            reprobe: Duration::from_secs(3600),
            lookahead: Duration::from_secs(3600),
        }
    }

    /// A factory handing out `first` once, then copies of `rest`, counting
    /// the connections it opened.
    fn factory(first: Option<Fake>, rest: Fake) -> (ClientFactory, Arc<AtomicU64>) {
        let first = Arc::new(Mutex::new(first));
        let opened = Arc::new(AtomicU64::new(0));
        let count = opened.clone();
        let factory: ClientFactory = Arc::new(move || {
            count.fetch_add(1, Ordering::Relaxed);
            let fake = first.lock().unwrap().take().unwrap_or_else(|| rest.clone());
            Box::pin(async move { Ok(Box::new(fake) as Client) })
        });
        (factory, opened)
    }

    async fn open(
        first: Fake,
        factory: ClientFactory,
        end: u32,
        cfg: FetchConfig,
    ) -> (BlockStream, Arc<FetchStats>) {
        let stats = Arc::new(FetchStats::default());
        let mut client: Client = Box::new(first);
        let stream =
            block_stream_with(&Network::Main, &mut client, 1, end, factory, cfg, stats.clone(), None)
                .await
                .unwrap();
        (stream, stats)
    }

    /// Reads the stream to its end: the heights seen and the error, if any.
    async fn drain(stream: &mut BlockStream) -> (Vec<u32>, Option<String>) {
        let mut heights = vec![];
        while let Some(item) = stream.next().await {
            match item {
                Ok(block) => heights.push(block.height as u32),
                Err(error) => return (heights, Some(format!("{error:#}"))),
            }
        }
        (heights, None)
    }

    #[tokio::test]
    async fn blocks_arrive_in_order_across_streams() {
        let server = Fake::new(1, 100);
        let asked = server.asked.clone();
        let (more, opened) = factory(None, server.clone());
        let (mut stream, stats) = open(server, more, 3000, cfg(64 << 20)).await;
        let (heights, error) = drain(&mut stream).await;
        assert_eq!(error, None);
        assert_eq!(heights, (1..=3000).collect::<Vec<_>>());
        assert!(opened.load(Ordering::Relaxed) >= 3, "the streams were not opened at the start");
        assert!(stats.pieces.load(Ordering::Relaxed) > 3);
        // The first slice is a probe, not the whole range.
        assert_eq!(asked.lock().unwrap()[0], (1, PROBE_BLOCKS));
    }

    #[tokio::test]
    async fn a_short_range_is_one_request_on_the_callers_connection() {
        let server = Fake::new(0, 100);
        let asked = server.asked.clone();
        let (more, opened) = factory(None, server.clone());
        let mut client: Client = Box::new(server);
        let end = PARALLEL_MIN_BLOCKS - 1;
        let mut stream = block_stream(&Network::Main, &mut client, 1, end, Some(more), None)
            .await
            .unwrap();
        let (heights, error) = drain(&mut stream).await;
        assert_eq!(error, None);
        assert_eq!(heights.len() as u32, end);
        assert_eq!(*asked.lock().unwrap(), vec![(1, end)]);
        assert_eq!(opened.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn a_short_range_that_takes_too_long_is_aborted_and_split() {
        // 2000 blocks at a millisecond each, against a 150 ms limit.
        let server = Fake::new(1, 100);
        let asked = server.asked.clone();
        let (more, opened) = factory(None, server.clone());
        let mut config = cfg(64 << 20);
        config.parallel_min_blocks = 10_000;
        let (mut stream, stats) = open(server, more, 2000, config).await;
        let (heights, error) = drain(&mut stream).await;
        assert_eq!(error, None);
        assert_eq!(heights, (1..=2000).collect::<Vec<_>>());
        assert_eq!(asked.lock().unwrap()[0], (1, 2000), "it starts as one request");
        assert!(stats.timeouts.load(Ordering::Relaxed) >= 1, "the request was never aborted");
        assert!(opened.load(Ordering::Relaxed) >= 1, "no stream took over");
    }

    #[tokio::test]
    async fn a_slow_slice_nobody_waits_for_is_aborted_too() {
        let server = Fake::new(1, 100);
        let slow = Fake { delay: Duration::from_millis(40), ..server.clone() };
        let (more, _) = factory(Some(slow), server.clone());
        let (mut stream, stats) = open(server, more, 3000, cfg(64 << 20)).await;
        // The consumer is not reading, so nothing is "needed": only the
        // time limit can end the slow stream's slice.
        tokio::time::sleep(Duration::from_millis(500)).await;
        assert!(stats.timeouts.load(Ordering::Relaxed) >= 1, "the slow slice was left running");
        let (heights, error) = drain(&mut stream).await;
        assert_eq!(error, None);
        assert_eq!(heights, (1..=3000).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn a_waiting_consumer_gets_more_streams() {
        let server = Fake::new(1, 100);
        let (more, _) = factory(None, server.clone());
        let (mut stream, stats) = open(server, more, 4000, cfg(64 << 20)).await;
        let (heights, error) = drain(&mut stream).await;
        assert_eq!(error, None);
        assert_eq!(heights.len(), 4000);
        assert!(
            stats.peak_connections.load(Ordering::Relaxed) > 3,
            "no stream was added while the consumer waited"
        );
    }

    #[tokio::test]
    async fn undelivered_bytes_stay_near_the_budget() {
        let cap = 200_000;
        let server = Fake::new(1, 1000);
        let (more, _) = factory(None, server.clone());
        let (mut stream, stats) = open(server, more, 3000, cfg(cap)).await;
        // Read for a while, then stop reading: the download must stop too.
        let mut heights = vec![];
        let reading = Instant::now();
        while reading.elapsed() < Duration::from_millis(400) {
            heights.push(stream.next().await.unwrap().unwrap().height as u32);
        }
        tokio::time::sleep(Duration::from_millis(700)).await;
        let (rest, error) = drain(&mut stream).await;
        heights.extend(rest);
        assert_eq!(error, None);
        assert_eq!(heights, (1..=3000).collect::<Vec<_>>());
        let max = stats.max_buffered.load(Ordering::Relaxed);
        assert!(max <= cap * 2, "{max} bytes buffered, budget {cap}");
    }

    #[tokio::test]
    async fn a_slice_denser_than_planned_is_cut_and_refetched() {
        // Blocks grow fifty-fold past height 800: slices planned on the
        // small ones overrun.
        let mut server = Fake::new(1, 100);
        server.size = Arc::new(|height| if height > 800 { 5000 } else { 100 });
        let (more, _) = factory(None, server.clone());
        let (mut stream, stats) = open(server, more, 2500, cfg(64 << 20)).await;
        let (heights, error) = drain(&mut stream).await;
        assert_eq!(error, None);
        assert_eq!(heights, (1..=2500).collect::<Vec<_>>());
        assert!(stats.cuts.load(Ordering::Relaxed) >= 1, "no slice was cut");
    }

    #[tokio::test]
    async fn a_failed_stream_has_its_slice_refetched() {
        let server = Fake::new(1, 100);
        let mut breaks = server.clone();
        breaks.fails_after_blocks = Some(20);
        let (more, _) = factory(Some(breaks), server.clone());
        let (mut stream, stats) = open(server, more, 2000, cfg(64 << 20)).await;
        let (heights, error) = drain(&mut stream).await;
        assert_eq!(error, None);
        assert_eq!(heights, (1..=2000).collect::<Vec<_>>());
        assert!(stats.requeued.load(Ordering::Relaxed) >= 1, "no slice was given back");
    }

    #[tokio::test]
    async fn when_every_stream_fails_the_error_follows_what_arrived_in_order() {
        let mut server = Fake::new(1, 100);
        server.fails_after = Some(300);
        let (more, _) = factory(None, server.clone());
        let (mut stream, _) = open(server, more, 2000, cfg(64 << 20)).await;
        let (heights, error) = drain(&mut stream).await;
        assert_eq!(heights, (1..=300).collect::<Vec<_>>());
        assert!(error.unwrap().contains("connection reset"));
    }

    #[tokio::test]
    async fn a_stream_that_falls_behind_at_the_head_loses_its_slice() {
        // One stream takes 40 ms a block: its 64-block probe alone would
        // hold the head for 2.5 s.
        let server = Fake::new(1, 100);
        let slow = Fake { delay: Duration::from_millis(40), ..server.clone() };
        let (more, _) = factory(Some(slow), server.clone());
        let started = Instant::now();
        let (mut stream, stats) = open(server, more, 3000, cfg(64 << 20)).await;
        let (heights, error) = drain(&mut stream).await;
        assert_eq!(error, None);
        assert_eq!(heights, (1..=3000).collect::<Vec<_>>());
        assert!(stats.steals.load(Ordering::Relaxed) >= 1, "the head slice was never moved");
        assert!(started.elapsed() < Duration::from_secs(6), "the slow stream held the scan back");
    }

    #[tokio::test]
    async fn a_stream_that_stays_slow_is_replaced() {
        let server = Fake::new(1, 100);
        let slow = Fake { delay: Duration::from_millis(8), ..server.clone() };
        let (more, opened) = factory(Some(slow), server.clone());
        let mut config = cfg(64 << 20);
        config.initial_streams = 4;
        let (mut stream, stats) = open(server, more, 9000, config).await;
        let (heights, error) = drain(&mut stream).await;
        assert_eq!(error, None);
        assert_eq!(heights.len(), 9000);
        assert!(stats.replaced.load(Ordering::Relaxed) >= 1, "the slow stream was kept");
        assert!(opened.load(Ordering::Relaxed) >= 5);
    }

    #[tokio::test]
    async fn connections_go_to_the_faster_region() {
        // Two regions: "slow" takes 6 ms a block, "fast" 1 ms.
        let mut server = Fake::new(1, 100);
        server.regions = Some(Arc::new(|region: &str| {
            Duration::from_millis(if region == "slow" { 6 } else { 1 })
        }));
        let served = server.served.clone();
        let (more, _) = factory(None, server.clone());
        let mut config = cfg(64 << 20);
        config.initial_streams = 4;
        config.regions = vec!["slow".into(), "fast".into()];
        let (mut stream, stats) = open(server, more, 12_000, config).await;
        let (heights, error) = drain(&mut stream).await;
        assert_eq!(error, None);
        assert_eq!(heights, (1..=12_000).collect::<Vec<_>>());
        assert!(
            stats.replaced.load(Ordering::Relaxed) >= 1,
            "no stream left the slow region"
        );
        // Once both were measured, the streams open to the fast one.
        let served = served.lock().unwrap();
        let late = &served[served.len() / 2..];
        let fast = late.iter().filter(|region| *region == "fast").count();
        assert!(
            fast * 4 >= late.len() * 3,
            "{fast} of the last {} requests went to fast",
            late.len()
        );
    }
}
