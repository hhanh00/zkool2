use anyhow::Result;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

use crate::{
    api::coin::Coin,
    frb_generated::StreamSink,
    migrate::{
        plan::next_task,
        state::MigrationState,
        step::StepOutcome,
        task::{MigrationTask, Pacing, TaskKind},
    },
};

/// Current migration status — streamed to Flutter by run_migration().
#[cfg_attr(feature = "flutter", frb)]
#[derive(Clone, Debug)]
pub struct MigrationStatus {
    pub phase: String,
    pub split_fees: u64,
    pub migrate_fees: u64,
    pub total_fees: u64,
    pub sd_notes_count: u32,
    pub non_sd_notes_count: u32,
    pub ironwood_sd_count: u32,
    // Deprecated, kept for FRB generated-code compat.
    pub progress: f64,
    pub next_action: String,
    pub work_summary: String,
}

/// Result of a single step (kept for FRB compat with step_migration).
#[cfg_attr(feature = "flutter", frb)]
pub enum MigrationEvent {
    SplitComplete { fee: u64 },
    MigrateComplete { fee: u64 },
    Complete,
    NothingToDo,
    Error { message: String },
}

#[cfg_attr(feature = "flutter", frb(opaque))]
pub struct NoteMigration {
    cancellation_token: CancellationToken,
    block_height_tx: watch::Sender<Option<u32>>,
}

impl NoteMigration {
    #[cfg_attr(feature = "flutter", frb(sync))]
    pub fn new() -> Self {
        let (block_height_tx, _) = watch::channel(None);
        Self {
            cancellation_token: CancellationToken::new(),
            block_height_tx,
        }
    }

    #[cfg(feature = "flutter")]
    pub async fn run(
        &self,
        sink: StreamSink<MigrationStatus>,
        c: &Coin,
        mean_delay_ms: u64,
    ) -> Result<()> {
        run_migration(
            sink,
            c,
            mean_delay_ms,
            self.cancellation_token.clone(),
            self.block_height_tx.clone(),
        )
        .await
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    /// Supplies a height observed by the shared Dart block-height service.
    #[cfg_attr(feature = "flutter", frb(sync))]
    pub fn update_height(&self, height: u32) {
        self.block_height_tx.send_replace(Some(height));
    }
}

/// Single-shot step (kept for FRB generated-code compatibility).
#[cfg_attr(feature = "flutter", frb)]
pub async fn step_migration(c: &Coin) -> Result<MigrationEvent> {
    Ok(match crate::migrate::step::step_once(c, c.account).await? {
        StepOutcome::Split { fee } => MigrationEvent::SplitComplete { fee },
        StepOutcome::Migrated { fee } => MigrationEvent::MigrateComplete { fee },
        StepOutcome::Complete => MigrationEvent::Complete,
        StepOutcome::NothingToDo => MigrationEvent::NothingToDo,
    })
}

/// Run migration to completion, streaming MigrationStatus to Flutter.
///
/// The loop is deliberately thin: observe the wallet, ask `next_task` what to
/// do, re-check that the answer still holds, execute it. Every decision lives
/// in `crate::migrate::plan`, so what the user is shown and what actually
/// happens are derived from one function and cannot drift apart.
///
/// `mean_delay_ms` controls the mean wait time (in milliseconds) of the
/// exponential random delay that paces every cycle. O→I steps additionally
/// wait for the next anchor bucket boundary before syncing, preparing, and
/// broadcasting.
#[cfg(feature = "flutter")]
async fn run_migration(
    sink: StreamSink<MigrationStatus>,
    c: &Coin,
    mean_delay_ms: u64,
    cancellation_token: CancellationToken,
    block_height_tx: watch::Sender<Option<u32>>,
) -> Result<()> {
    use zcash_protocol::consensus::{BlockHeight, NetworkUpgrade, Parameters};

    // Migration only makes sense when Ironwood (NU6.3) is active.
    let network = c.network();
    let mut client = c.client().await?;
    let height = client.latest_height().await?;
    if !network.is_nu_active(NetworkUpgrade::Nu6_3, BlockHeight::from_u32(height)) {
        sink.add(MigrationStatus {
            phase: "complete".into(),
            split_fees: 0,
            migrate_fees: 0,
            total_fees: 0,
            sd_notes_count: 0,
            non_sd_notes_count: 0,
            ironwood_sd_count: 0,
            progress: 1.0,
            next_action: String::new(),
            work_summary: String::new(),
        })
        .ok();
        return Ok(());
    }

    let mut acc_split = 0u64;
    let mut acc_migrate = 0u64;
    let mut pacing = Pacing::default();
    let anchor_bucket_size = crate::migrate::migration_anchor_bucket_size(mean_delay_ms);
    tracing::info!(
        "Migration anchor interval: {} blocks for mean delay {}ms",
        anchor_bucket_size,
        mean_delay_ms,
    );

    loop {
        // Draw this cycle's delay before planning. The value is an input to
        // the decision, never something the decision computes: a durable
        // engine replays control flow, and a re-rolled random number would
        // replay differently.
        pacing.sampled_delay_ms = sample_delay(mean_delay_ms);

        let state = observe_state(c, anchor_bucket_size).await?;
        let task = next_task(&state, &pacing);
        let status = status_from(&state, &task, acc_split, acc_migrate);
        sink.add(status.clone()).ok();

        if matches!(task, MigrationTask::Done) {
            break;
        }

        // Re-validate before spending anything. The snapshot the task was
        // planned from is a round-trip old, and a concurrent sync or an
        // earlier broadcast may have moved the notes underneath it — so an
        // effect is checked against state read immediately before it runs.
        // A stale task is re-planned, never executed.
        if matches!(task.kind(), TaskKind::Effect) {
            let fresh = observe_state(c, anchor_bucket_size).await?;
            if !task.is_satisfied_by(&fresh) {
                tracing::info!("Migration task {:?} no longer applies; replanning", task);
                // Pace the retry, so a task that stays stale cannot spin.
                pacing.restart();
                continue;
            }
        }

        let cancelled = tokio::select! {
            biased;
            _ = cancellation_token.cancelled() => {
                tracing::info!("Note migration cancelled");
                true
            }
            result = execute(c, &task, &block_height_tx) => {
                match result? {
                    Executed::Waited => pacing.delay_served = true,
                    Executed::Split { fee } => {
                        acc_split += fee;
                        pacing.after_effect(client.latest_height().await?);
                    }
                    Executed::Migrated { fee } => {
                        acc_migrate += fee;
                        pacing.after_effect(client.latest_height().await?);
                    }
                }
                false
            }
        };
        if cancelled {
            break;
        }
    }

    Ok(())
}

/// What running a task changed, so the driver can advance its pacing.
#[cfg(feature = "flutter")]
enum Executed {
    Waited,
    Split { fee: u64 },
    Migrated { fee: u64 },
}

/// Carry out one task. Everything that touches the wallet or the chain lives
/// in `crate::migrate::exec`; what remains here is waiting — on the clock, or
/// on the block heights Dart supplies — which decides nothing.
#[cfg(feature = "flutter")]
async fn execute(
    c: &Coin,
    task: &MigrationTask,
    block_height_tx: &watch::Sender<Option<u32>>,
) -> Result<Executed> {
    Ok(match task {
        MigrationTask::WaitDelay { ms } => {
            tracing::info!("Migration delay: {}ms", ms);
            tokio::time::sleep(std::time::Duration::from_millis(*ms)).await;
            Executed::Waited
        }
        MigrationTask::WaitBoundary { .. } => {
            block_height_tx.send_replace(None);
            wait_for_next_height(&mut block_height_tx.subscribe()).await?;
            // Waiting changes no wallet state. The next cycle re-plans and
            // sees where autosync has taken the checkpoint by then.
            Executed::Waited
        }
        MigrationTask::Split { inputs, outputs } => Executed::Split {
            fee: crate::migrate::exec::split(c, inputs, outputs).await?,
        },
        MigrationTask::Migrate {
            note,
            amount,
            anchor,
        } => Executed::Migrated {
            fee: crate::migrate::exec::migrate(c, *note, *amount, *anchor).await?,
        },
        MigrationTask::Done => Executed::Waited,
    })
}

/// Exponentially distributed delay, capped at four times the mean. Drawn once
/// per cycle; the caller logs it if and when it is actually served.
#[cfg(feature = "flutter")]
fn sample_delay(mean_delay_ms: u64) -> u64 {
    use rand_core::{OsRng, RngCore};

    let mean = mean_delay_ms as f64;
    let u = (OsRng.next_u32() as f64 + 1.0) / (u32::MAX as f64 + 2.0);
    ((-mean * u.ln()) as u64).min(mean_delay_ms * 4)
}

/// Stub kept for FRB generated-code compatibility.
#[cfg_attr(feature = "flutter", frb)]
pub async fn get_migration_status(_c: &Coin) -> Result<MigrationStatus> {
    Ok(MigrationStatus {
        phase: "complete".into(),
        split_fees: 0,
        migrate_fees: 0,
        total_fees: 0,
        sd_notes_count: 0,
        non_sd_notes_count: 0,
        ironwood_sd_count: 0,
        progress: 1.0,
        next_action: String::new(),
        work_summary: String::new(),
    })
}

async fn observe_state(c: &Coin, anchor_bucket_size: u32) -> Result<MigrationState> {
    let network = c.network();
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    crate::migrate::state::observe(
        &network,
        &mut connection,
        &mut client,
        c.account,
        anchor_bucket_size,
    )
    .await
}

/// Build the UI status from the same snapshot and task the runner is acting
/// on, so the phase shown can never claim work that will not be attempted.
fn status_from(
    state: &MigrationState,
    task: &MigrationTask,
    acc_split: u64,
    acc_migrate: u64,
) -> MigrationStatus {
    let phase = crate::migrate::plan::phase(state);
    let sd_count = state.orchard_sd.len() as u32;
    let ironwood_sd = state.ironwood_sd_count;
    // Non-SD notes only count as outstanding work while a split is actually
    // available; below the threshold they are dust the migration leaves alone.
    let non_sd_count = if phase == "splitting" {
        state.orchard_non_sd.len() as u32
    } else {
        0
    };
    let total_sd = sd_count + ironwood_sd;

    let progress = match phase {
        "splitting" if sd_count + non_sd_count > 0 => {
            sd_count as f64 / (sd_count + non_sd_count) as f64
        }
        "migrating" if total_sd > 0 => ironwood_sd as f64 / total_sd as f64,
        _ => 1.0,
    };

    let next_action = match task {
        MigrationTask::WaitDelay { ms } => format!("Waiting {}s...", ms / 1000),
        MigrationTask::WaitBoundary { target } => format!("Waiting for anchor block {}...", target),
        MigrationTask::Split { .. } | MigrationTask::Migrate { .. } => {
            "Preparing migration transaction...".into()
        }
        MigrationTask::Done => String::new(),
    };

    MigrationStatus {
        phase: phase.to_string(),
        split_fees: acc_split,
        migrate_fees: acc_migrate,
        total_fees: acc_split + acc_migrate,
        sd_notes_count: sd_count,
        non_sd_notes_count: non_sd_count,
        ironwood_sd_count: ironwood_sd,
        progress,
        next_action,
        work_summary: format!("SD: {}, non-SD: {}", sd_count, non_sd_count),
    }
}

/// Block until the block-height service reports another height.
///
/// No protocol logic lives here: which boundary the migration needs, and
/// whether the wallet has reached one, are decided by `crate::migrate::plan`.
/// This waits for the next height Dart supplies — the same stream that drives
/// autosync, so by the following cycle the checkpoint reflects it — and then
/// returns so the migration can re-plan against what it finds.
#[cfg(feature = "flutter")]
async fn wait_for_next_height(block_heights: &mut watch::Receiver<Option<u32>>) -> Result<()> {
    loop {
        block_heights.changed().await?;
        if let Some(tip) = *block_heights.borrow_and_update() {
            tracing::info!("Migration observed height {}", tip);
            return Ok(());
        }
    }
}
