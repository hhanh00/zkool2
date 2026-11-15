//! Where to fetch blocks from: what each backend behind the server costs,
//! and which region to ask for when opening a connection.
//!
//! A server behind a load-balancing front end (zec.rocks, on Fly.io) sends
//! each request to one of several backends. They differ by several times,
//! and not in the same way: a near one answers sooner, a busy one spends
//! longer on each block, a far one moves bytes as fast as any once it has
//! started. So a backend is not one rate but a cost, fitted on its recent
//! slices:
//!
//! ```text
//! seconds ≈ first_block + blocks × per_block + bytes × per_byte
//! ```
//!
//! Which backend does best then depends on the slice: a short one goes best
//! to a near backend, a long run of small blocks to one that is quick per
//! block, dense blocks to one with bandwidth. Slices are sized so the wait
//! for the first block is a small part of them.
//!
//! The front end routes by region when asked (`fly-prefer-region`), so a
//! connection is opened to the region whose backend is expected to do best,
//! after each candidate region has been tried once; a region not heard from
//! for a while is tried again, one at a time. A server that never names its
//! backend is not steered, and each connection is then a backend of its own.

use std::collections::HashMap;
use std::time::Duration;

use tokio::time::Instant;

const MB: f64 = 1024.0 * 1024.0;
/// Weight an older slice keeps each time a newer one is counted: a backend's
/// load changes within a sync, so its cost follows its last few slices.
const DECAY: f64 = 0.85;
/// The wait for the first block is at most this share of a planned slice.
const FIRST_BLOCK_SHARE: f64 = 0.2;

/// What fetching from one backend costs, fitted on its recent slices.
#[derive(Default, Debug)]
pub(crate) struct Cost {
    /// Seconds to the first block of a request.
    first_block: Option<f64>,
    // Decayed sums for the least-squares fit of the transfer time `t` on
    // blocks `x` and megabytes `y`.
    xx: f64,
    xy: f64,
    yy: f64,
    xt: f64,
    yt: f64,
    samples: u32,
    /// Decayed seconds of transfer counted: tiny slices alone say little,
    /// a request's fixed overheads swamping them.
    transfer: f64,
}

impl Cost {
    /// Count a slice: the wait for its first block, if one came, and the
    /// time from there to its last.
    pub fn observe(&mut self, first_block: Option<f64>, blocks: u32, bytes: u64, transfer: f64) {
        if let Some(wait) = first_block {
            self.first_block =
                Some(self.first_block.map_or(wait, |old| old * DECAY + wait * (1.0 - DECAY)));
        }
        if blocks == 0 || transfer <= 0.0 {
            return;
        }
        let (x, y) = (f64::from(blocks), bytes as f64 / MB);
        for sum in [&mut self.xx, &mut self.xy, &mut self.yy, &mut self.xt, &mut self.yt] {
            *sum *= DECAY;
        }
        self.xx += x * x;
        self.xy += x * y;
        self.yy += y * y;
        self.xt += x * transfer;
        self.yt += y * transfer;
        self.samples += 1;
        self.transfer = self.transfer * DECAY + transfer;
    }

    /// Whether enough was fetched here to judge it: about `seconds` of
    /// transfer.
    pub fn measured(&self, seconds: f64) -> bool {
        self.transfer >= seconds
    }

    pub fn samples(&self) -> u32 {
        self.samples
    }

    pub fn first_block(&self) -> f64 {
        self.first_block.unwrap_or(0.0)
    }

    /// Seconds per block and per byte. Both are fitted when the slices seen
    /// tell them apart; otherwise, or when one comes out negative, the one
    /// term that explains the times better.
    pub fn terms(&self) -> Option<(f64, f64)> {
        if self.samples == 0 {
            return None;
        }
        let det = self.xx * self.yy - self.xy * self.xy;
        if self.samples >= 3 && det > 1e-6 * self.xx * self.yy {
            let per_block = (self.xt * self.yy - self.yt * self.xy) / det;
            let per_mb = (self.xx * self.yt - self.xy * self.xt) / det;
            if per_block >= 0.0 && per_mb >= 0.0 {
                return Some((per_block, per_mb / MB));
            }
        }
        let by_blocks = if self.xx > 0.0 { self.xt * self.xt / self.xx } else { 0.0 };
        let by_bytes = if self.yy > 0.0 { self.yt * self.yt / self.yy } else { 0.0 };
        if by_bytes >= by_blocks && self.yy > 0.0 {
            Some((0.0, self.yt / self.yy / MB))
        } else if self.xx > 0.0 {
            Some((self.xt / self.xx, 0.0))
        } else {
            None
        }
    }

    /// Seconds to fetch `blocks` blocks of `bytes` in all.
    pub fn seconds(&self, blocks: f64, bytes: f64) -> Option<f64> {
        let (per_block, per_byte) = self.terms()?;
        Some(self.first_block() + blocks * per_block + bytes * per_byte)
    }

    /// Seconds a slice should take here: `piece`, or longer where the wait
    /// for the first block would otherwise be too large a part of it.
    pub fn slice_seconds(&self, piece: f64) -> f64 {
        piece.max(self.first_block() / FIRST_BLOCK_SHARE)
    }

    /// Blocks of `block_bytes` each fetched here in `seconds`.
    pub fn blocks_in(&self, seconds: f64, block_bytes: f64) -> Option<f64> {
        let (per_block, per_byte) = self.terms()?;
        let each = per_block + per_byte * block_bytes;
        (each > 0.0).then(|| (seconds - self.first_block()).max(0.0) / each)
    }

    /// Blocks per second over a slice planned here, the wait included.
    pub fn blocks_per_second(&self, piece: f64, block_bytes: f64) -> Option<f64> {
        let seconds = self.slice_seconds(piece);
        let blocks = self.blocks_in(seconds, block_bytes)?;
        Some(blocks / self.seconds(blocks, blocks * block_bytes)?.max(1e-3))
    }
}

/// The backends seen and the regions that can be asked for.
pub(crate) struct Routes {
    pub costs: HashMap<String, Cost>,
    /// Regions to ask for, when the server turns out to route by region.
    candidates: Vec<String>,
    /// The backend each region last sent a request to, and when.
    answered: HashMap<String, (String, Instant)>,
    /// Regions a connection was opened to.
    tried: Vec<String>,
    /// Whether the server names its backends; unknown until it answers.
    steerable: Option<bool>,
    reprobe: Duration,
    last_probe: Option<Instant>,
}

impl Routes {
    pub fn new(candidates: Vec<String>, reprobe: Duration) -> Self {
        Self {
            costs: HashMap::new(),
            candidates,
            answered: HashMap::new(),
            tried: vec![],
            steerable: None,
            reprobe,
            last_probe: None,
        }
    }

    /// The key a slice's cost is counted under: its backend, or its
    /// connection where the server does not say.
    pub fn key(backend: Option<&str>, stream: usize) -> String {
        match backend {
            Some(backend) => backend.to_string(),
            None => format!("stream {stream}"),
        }
    }

    pub fn steerable(&self) -> bool {
        self.steerable == Some(true)
    }

    /// A request on a connection opened to `region` was answered by `backend`.
    pub fn answered(&mut self, region: Option<&str>, backend: Option<&str>) {
        match backend {
            Some(backend) => {
                self.steerable = Some(true);
                if let Some(region) = region {
                    self.answered.insert(region.to_string(), (backend.to_string(), Instant::now()));
                }
            }
            None if self.steerable.is_none() => self.steerable = Some(false),
            None => {}
        }
    }

    /// The backend a connection to `region` is expected to reach.
    pub fn backend_of(&self, region: &str) -> Option<&str> {
        self.answered.get(region).map(|(backend, _)| backend.as_str())
    }

    /// The cost a connection to `region` is expected to have.
    pub fn cost_of(&self, region: &str) -> Option<&Cost> {
        self.costs.get(self.backend_of(region)?)
    }

    /// The cost of a region's backend, once it was measured enough to rank.
    fn measured_cost_of(&self, region: &str, piece: f64) -> Option<&Cost> {
        self.cost_of(region).filter(|cost| cost.measured(piece))
    }

    /// Blocks per second the best measured region is expected to fetch.
    pub fn best_region_rate(&self, piece: f64, block_bytes: f64) -> Option<f64> {
        self.candidates
            .iter()
            .filter_map(|region| {
                self.measured_cost_of(region, piece)?.blocks_per_second(piece, block_bytes)
            })
            .max_by(f64::total_cmp)
    }

    /// Blocks a slice usually has: the median over the backends measured.
    pub fn typical_blocks(&self, piece: f64, block_bytes: f64) -> Option<f64> {
        let mut blocks: Vec<f64> = self
            .costs
            .values()
            .filter(|cost| cost.measured(piece))
            .filter_map(|cost| cost.blocks_in(cost.slice_seconds(piece), block_bytes))
            .collect();
        blocks.sort_by(f64::total_cmp);
        blocks.get(blocks.len() / 2).copied()
    }

    /// The region to open the next connection to: a candidate not tried
    /// yet, else the one expected to fetch fastest. None where the server is
    /// known not to route by region.
    pub fn pick_region(&mut self, piece: f64, block_bytes: f64) -> Option<String> {
        if self.steerable == Some(false) || self.candidates.is_empty() {
            return None;
        }
        if let Some(region) = self.candidates.iter().find(|r| !self.tried.contains(r)).cloned() {
            self.tried.push(region.clone());
            return Some(region);
        }
        self.candidates
            .iter()
            .filter_map(|region| {
                let cost = self.measured_cost_of(region, piece)?;
                Some((region, cost.blocks_per_second(piece, block_bytes)?))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(region, _)| region.clone())
            .or_else(|| self.candidates.first().cloned())
    }

    /// A region no connection is on that has not answered for `reprobe`,
    /// the stalest first, once per `reprobe`: a backend's load changes, and
    /// one passed over may have become the best.
    pub fn stale_region(&self, in_use: &[&str]) -> Option<String> {
        if self.last_probe.is_some_and(|at| at.elapsed() < self.reprobe) {
            return None;
        }
        self.candidates
            .iter()
            .filter(|region| !in_use.contains(&region.as_str()))
            .filter(|region| {
                self.answered.get(*region).is_none_or(|(_, at)| at.elapsed() >= self.reprobe)
            })
            .min_by_key(|region| self.answered.get(*region).map(|(_, at)| *at))
            .cloned()
    }

    /// A connection was opened to the region [`Self::stale_region`] gave.
    pub fn probed(&mut self) {
        self.last_probe = Some(Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A backend that waits `first` seconds, then takes `per_block` seconds
    /// a block and moves `mbps` megabytes a second.
    fn fitted(first: f64, per_block: f64, mbps: f64, slices: &[(u32, u64)]) -> Cost {
        let mut cost = Cost::default();
        for &(blocks, bytes) in slices {
            let transfer = f64::from(blocks) * per_block + bytes as f64 / MB / mbps;
            cost.observe(Some(first), blocks, bytes, transfer);
        }
        cost
    }

    #[test]
    fn both_terms_are_recovered_from_slices_of_different_density() {
        let slices = [(20_000, 4 << 20), (500, 40 << 20), (5_000, 1 << 20), (2_000, 100 << 20)];
        let cost = fitted(0.3, 300e-6, 10.0, &slices);
        let (per_block, per_byte) = cost.terms().unwrap();
        assert!((per_block - 300e-6).abs() < 1e-6, "{per_block}");
        assert!((per_byte * MB - 0.1).abs() < 1e-3, "{per_byte}");
    }

    #[test]
    fn the_better_backend_depends_on_the_blocks() {
        // Near and busy: answers at once, slow per block, good bandwidth.
        let slices = [(20_000, 4 << 20), (500, 40 << 20), (5_000, 1 << 20), (2_000, 100 << 20)];
        let near = fitted(0.1, 300e-6, 10.0, &slices);
        // Far and idle: slow to answer, quick per block, less bandwidth.
        let far = fitted(0.6, 60e-6, 5.0, &slices);
        let small = 200.0;
        let dense = 50_000.0;
        assert!(far.blocks_per_second(3.0, small) > near.blocks_per_second(3.0, small));
        assert!(near.blocks_per_second(3.0, dense) > far.blocks_per_second(3.0, dense));
    }

    #[test]
    fn a_far_backend_gets_longer_slices() {
        let slices = [(20_000, 4 << 20), (500, 40 << 20), (5_000, 1 << 20)];
        let far = fitted(0.9, 60e-6, 5.0, &slices);
        assert!((far.slice_seconds(3.0) - 4.5).abs() < 1e-9);
        assert_eq!(fitted(0.1, 60e-6, 5.0, &slices).slice_seconds(3.0), 3.0);
    }

    #[test]
    fn regions_are_tried_then_the_fastest_is_picked() {
        let mut routes = Routes::new(vec!["a".into(), "b".into()], Duration::from_secs(3600));
        assert_eq!(routes.pick_region(3.0, 200.0), Some("a".into()));
        assert_eq!(routes.pick_region(3.0, 200.0), Some("b".into()));
        routes.answered(Some("a"), Some("be-a"));
        routes.answered(Some("b"), Some("be-b"));
        let slices = [(20_000, 4 << 20), (500, 40 << 20), (5_000, 1 << 20)];
        routes.costs.insert("be-a".into(), fitted(0.2, 300e-6, 10.0, &slices));
        routes.costs.insert("be-b".into(), fitted(0.2, 60e-6, 10.0, &slices));
        assert_eq!(routes.pick_region(3.0, 200.0), Some("b".into()));
        assert_eq!(routes.stale_region(&["b"]), None, "nothing is stale yet");
    }

    #[test]
    fn a_server_that_names_no_backend_is_not_steered() {
        let mut routes = Routes::new(vec!["a".into()], Duration::from_secs(3600));
        routes.answered(Some("a"), None);
        assert_eq!(routes.pick_region(3.0, 200.0), None);
    }
}
