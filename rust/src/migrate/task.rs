//! The unit of migration work.
//!
//! A task is plain data: it names an action without performing it, so the
//! same value can be planned, displayed, re-validated against fresh state,
//! and only then executed. Time- and network-bound work are distinct variants
//! rather than steps buried inside a larger function, which is what lets a
//! driver — or, later, a durable-execution engine — schedule, retry, and
//! resume them independently.

use super::state::MigrationState;

/// What kind of work a task represents. A driver uses this to decide how to
/// run it: a timer wants a durable sleep, a network read wants retry with
/// backoff, an effect wants exactly-once semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskKind {
    /// Elapses wall-clock time. No wallet state changes.
    Time,
    /// Broadcasts a transaction. The only kind that spends notes.
    Effect,
    /// Nothing left to do.
    Terminal,
}

/// One step of the migration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MigrationTask {
    /// Pace the migration. The first step of every cycle, including the
    /// first: acting on a predictable schedule would leak the wallet's
    /// activity to an observer.
    WaitDelay { ms: u64 },
    /// Hold until the wallet's checkpoint lands on a shared anchor boundary.
    /// Migration does not sync — the wallet's own autosync does — so this
    /// waits for that to happen rather than making it happen. `target` is the
    /// boundary being waited for, for display.
    WaitBoundary { target: u32 },
    /// O→O self-send: consume `inputs` and mint standard denominations.
    Split {
        inputs: Vec<u32>,
        outputs: Vec<(u64, u8)>,
    },
    /// O→I: spend one Orchard SD note, mint `amount` into Ironwood.
    Migrate { note: u32, amount: u64, anchor: u32 },
    /// No further progress is possible.
    Done,
}

impl MigrationTask {
    pub fn kind(&self) -> TaskKind {
        match self {
            MigrationTask::WaitDelay { .. } | MigrationTask::WaitBoundary { .. } => TaskKind::Time,
            MigrationTask::Split { .. } | MigrationTask::Migrate { .. } => TaskKind::Effect,
            MigrationTask::Done => TaskKind::Terminal,
        }
    }

    /// Whether this task is still valid against freshly observed state.
    ///
    /// A task is planned from one snapshot and executed against another: the
    /// chain advances, a sync lands, an earlier broadcast locks its inputs.
    /// Re-checking here turns a stale plan into a re-plan rather than an
    /// error or, worse, a second transaction over notes already spent.
    pub fn is_satisfied_by(&self, s: &MigrationState) -> bool {
        match self {
            MigrationTask::WaitDelay { .. } | MigrationTask::WaitBoundary { .. } => true,
            MigrationTask::Split { inputs, .. } => inputs
                .iter()
                .all(|id| s.orchard_non_sd.iter().any(|n| n.id == *id)),
            MigrationTask::Migrate { note, anchor, .. } => {
                s.checkpoint_height == *anchor
                    && anchor.is_multiple_of(s.anchor_bucket_size)
                    && s.sd_spendable_at(*anchor).iter().any(|n| n.id == *note)
            }
            MigrationTask::Done => true,
        }
    }
}

/// Driver-local pacing state, threaded through the decision function so that
/// it stays pure.
///
/// `sampled_delay_ms` is drawn by the driver rather than computed during
/// planning: a durable-execution engine replays control flow, so a random
/// draw has to be a journaled input, never something re-rolled on replay.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pacing {
    pub sampled_delay_ms: u64,
    /// Tip height at the last broadcast, so the migration does not act twice
    /// in one block.
    pub last_action_height: Option<u32>,
    /// Whether this cycle's delay has already elapsed.
    pub delay_served: bool,
}

impl Pacing {
    /// Begin a fresh cycle, delaying again. Used when a planned task turns
    /// out to be stale, so that retrying cannot become a hot loop.
    pub fn restart(&mut self) {
        self.delay_served = false;
    }

    /// Begin a new cycle after an effect has been broadcast at `height`.
    pub fn after_effect(&mut self, height: u32) {
        self.last_action_height = Some(height);
        self.restart();
    }
}
