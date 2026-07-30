use anyhow::Result;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

use crate::{api::coin::Coin, frb_generated::StreamSink};

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
    let (event, _status) = do_step(c, 0, 0, true, true, crate::migrate::ANCHOR_BUCKET_SIZE).await?;
    Ok(match event {
        crate::migrate::MigrationEvent::SplitComplete { fee } => {
            MigrationEvent::SplitComplete { fee }
        }
        crate::migrate::MigrationEvent::MigrateComplete { fee } => {
            MigrationEvent::MigrateComplete { fee }
        }
        crate::migrate::MigrationEvent::Complete => MigrationEvent::Complete,
        crate::migrate::MigrationEvent::NothingToDo => MigrationEvent::NothingToDo,
    })
}

/// Run migration to completion, streaming MigrationStatus to Flutter.
///
/// `mean_delay_ms` controls the mean wait time (in milliseconds) of the
/// exponential random delay before migration steps. O→I steps additionally
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
    use rand_core::{OsRng, RngCore};
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
    let mut last_action_height: Option<u32> = None;
    let anchor_bucket_size = crate::migrate::migration_anchor_bucket_size(mean_delay_ms);
    tracing::info!(
        "Migration anchor interval: {} blocks for mean delay {}ms",
        anchor_bucket_size,
        mean_delay_ms,
    );
    let mut status = current_migration_status(c, acc_split, acc_migrate).await?;
    sink.add(status.clone()).ok();

    loop {
        if status.phase == "complete" {
            break;
        }

        // Delay before doing any migration-specific network activity. This
        // also applies to the first transaction.
        let mean = mean_delay_ms as f64;
        let u = (OsRng.next_u32() as f64 + 1.0) / (u32::MAX as f64 + 2.0);
        let delay_ms = ((-mean * u.ln()) as u64).min(mean_delay_ms * 4);
        let delay_secs = delay_ms / 1000;

        tracing::info!(
            "Migration delay: {}ms (mean={}ms, u={:.6})",
            delay_ms,
            mean_delay_ms,
            u
        );

        status.next_action = format!("Waiting {}s...", delay_secs);
        sink.add(status.clone()).ok();

        let cancelled = tokio::select! {
            biased;
            _ = cancellation_token.cancelled() => {
                tracing::info!("Note migration cancelled");
                true
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(delay_ms)) => false,
        };
        if cancelled {
            break;
        }

        if let Some(height) = last_action_height {
            if client.latest_height().await? <= height {
                continue;
            }
        }

        // O→I transactions are prepared only when the wallet checkpoint and
        // the current anchor are the same shared bucket boundary. Until then,
        // only query the tip height; do not fetch or synchronize tree state.
        let align_to_boundary = status.phase == "migrating";
        if align_to_boundary {
            block_height_tx.send_replace(None);
            let reached_boundary = tokio::select! {
                biased;
                _ = cancellation_token.cancelled() => {
                    tracing::info!("Note migration cancelled");
                    false
                }
                result = wait_for_anchor_boundary(
                    &sink,
                    c,
                    block_height_tx.subscribe(),
                    &status,
                    anchor_bucket_size,
                ) => {
                    result?;
                    true
                }
            };
            if !reached_boundary {
                break;
            }

            status.next_action = "Preparing migration transaction...".into();
            sink.add(status.clone()).ok();
        }

        let (event, next_status) = tokio::select! {
            biased;
            _ = cancellation_token.cancelled() => {
                tracing::info!("Note migration cancelled");
                break;
            }
            result = do_step(
                c,
                acc_split,
                acc_migrate,
                align_to_boundary,
                !align_to_boundary,
                anchor_bucket_size,
            ) => result?,
        };

        match event {
            crate::migrate::MigrationEvent::SplitComplete { fee } => {
                acc_split += fee;
                last_action_height = Some(client.latest_height().await?);
            }
            crate::migrate::MigrationEvent::MigrateComplete { fee } => {
                acc_migrate += fee;
                last_action_height = Some(client.latest_height().await?);
            }
            _ => {}
        }

        status = MigrationStatus {
            split_fees: acc_split,
            migrate_fees: acc_migrate,
            total_fees: acc_split + acc_migrate,
            ..next_status
        };
        sink.add(status.clone()).ok();
    }

    Ok(())
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

/// Shared step logic. Returns the internal event + a MigrationStatus
/// built from the current wallet state and accumulated fees.
async fn do_step(
    c: &Coin,
    acc_split: u64,
    acc_migrate: u64,
    allow_migrate: bool,
    sync_before: bool,
    anchor_bucket_size: u32,
) -> Result<(crate::migrate::MigrationEvent, MigrationStatus)> {
    let network = c.network();
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;

    if sync_before {
        let current_height = client.latest_height().await?;
        let _ = synchronize_to(c, current_height).await;
    }

    let before = current_migration_status(c, acc_split, acc_migrate).await?;
    let event = if before.phase == "complete" {
        crate::migrate::MigrationEvent::Complete
    } else if before.phase == "migrating" && !allow_migrate {
        // A normal sync may finish the splitting phase at a non-boundary
        // height. Return to the runner so it can delay and align the O→I
        // transaction instead of broadcasting it immediately.
        crate::migrate::MigrationEvent::NothingToDo
    } else {
        crate::migrate::step(
            &network,
            &mut connection,
            &mut client,
            c.account,
            anchor_bucket_size,
        )
        .await
        .map_err(|e| anyhow::anyhow!("step: {e}"))?
    };

    let status = current_migration_status(c, acc_split, acc_migrate).await?;
    Ok((event, status))
}

async fn current_migration_status(
    c: &Coin,
    acc_split: u64,
    acc_migrate: u64,
) -> Result<MigrationStatus> {
    let mut connection = c.get_connection().await?;
    let all_notes =
        crate::pay::plan::fetch_unspent_notes_grouped_by_pool(&mut connection, c.account).await?;
    let orchard_zec: Vec<&crate::pay::InputNote> = all_notes
        .iter()
        .filter(|n| n.pool == 2 && n.asset_base == vec![0u8; 32])
        .collect();
    let sd_count = orchard_zec
        .iter()
        .filter(|n| crate::migrate::is_sd(n.amount))
        .count() as u32;
    let non_sd_vals: Vec<u64> = orchard_zec
        .iter()
        .filter(|n| !crate::migrate::is_sd(n.amount))
        .map(|n| n.amount)
        .collect();
    let has_split = crate::migrate::has_split_transaction(non_sd_vals.clone());
    let effective_non_sd = if has_split {
        non_sd_vals.len() as u32
    } else {
        0
    };

    // Count Ironwood SD notes for phase 2 progress (amount is value - SD_FEE_PAD).
    let ironwood_sd = all_notes
        .iter()
        .filter(|n| {
            n.pool == 3 && n.asset_base == vec![0u8; 32] && crate::migrate::is_iw_sd(n.amount)
        })
        .count() as u32;
    let total_sd = sd_count + ironwood_sd;

    let phase = match () {
        _ if has_split => "splitting",
        _ if sd_count > 0 => "migrating",
        _ => "complete",
    };

    let progress = match phase {
        "splitting" if sd_count + effective_non_sd > 0 => {
            sd_count as f64 / (sd_count + effective_non_sd) as f64
        }
        "migrating" if total_sd > 0 => ironwood_sd as f64 / total_sd as f64,
        _ => 1.0,
    };

    Ok(MigrationStatus {
        phase: phase.to_string(),
        split_fees: acc_split,
        migrate_fees: acc_migrate,
        total_fees: acc_split + acc_migrate,
        sd_notes_count: sd_count,
        non_sd_notes_count: effective_non_sd,
        ironwood_sd_count: ironwood_sd,
        progress,
        next_action: String::new(),
        work_summary: format!("SD: {}, non-SD: {}", sd_count, effective_non_sd),
    })
}

async fn synchronize_to(c: &Coin, height: u32) -> Result<u32> {
    crate::sync::synchronize_impl(
        (),
        vec![c.account],
        height,
        100_000,
        10_000,
        10_000,
        false,
        c,
    )
    .await
}

/// Wait for heights supplied by Dart's shared block-height service. Tree state
/// is synchronized exactly once, while the boundary is the current tip.
#[cfg(feature = "flutter")]
async fn wait_for_anchor_boundary(
    sink: &StreamSink<MigrationStatus>,
    c: &Coin,
    mut block_heights: watch::Receiver<Option<u32>>,
    status: &MigrationStatus,
    anchor_bucket_size: u32,
) -> Result<()> {
    let mut waiting = status.clone();
    waiting.next_action = "Waiting for anchor block...".into();
    sink.add(waiting).ok();

    let mut boundary = None;
    loop {
        block_heights.changed().await?;
        let Some(tip) = *block_heights.borrow_and_update() else {
            continue;
        };

        let target = match boundary {
            Some(boundary) => boundary,
            None => {
                let db_height = wallet_height(c).await?;
                let boundary = crate::migrate::next_anchor_bucket_height(
                    tip.max(db_height),
                    anchor_bucket_size,
                );
                let mut waiting = status.clone();
                waiting.next_action = format!("Waiting for anchor block {}...", boundary);
                sink.add(waiting).ok();
                boundary
            }
        };

        if tip > target {
            // If height polling missed the boundary, do not fetch its
            // historical tree state. Wait for a boundary that is observed as
            // the current tip.
            let next_boundary = crate::migrate::next_anchor_bucket_height(
                tip.saturating_add(1),
                anchor_bucket_size,
            );
            boundary = Some(next_boundary);
            let mut waiting = status.clone();
            waiting.next_action = format!("Waiting for anchor block {}...", next_boundary);
            sink.add(waiting).ok();
            continue;
        }

        boundary = Some(target);
        if tip == target {
            tracing::info!(
                "Migration anchor boundary reached: tip={}, boundary={}",
                tip,
                target,
            );
            synchronize_to(c, target).await?;
            let synced_height = wallet_height(c).await?;
            if synced_height == target {
                return Ok(());
            }

            if synced_height > target {
                // Another sync advanced the wallet while we were waiting.
                // Move to a future boundary instead of preparing from a
                // checkpoint that no longer represents the current tip.
                let next_boundary = crate::migrate::next_anchor_bucket_height(
                    tip.max(synced_height).saturating_add(1),
                    anchor_bucket_size,
                );
                boundary = Some(next_boundary);
                let mut waiting = status.clone();
                waiting.next_action = format!("Waiting for anchor block {}...", next_boundary);
                sink.add(waiting).ok();
                continue;
            }

            anyhow::bail!(
                "Migration boundary sync did not advance: wallet={}, boundary={}",
                synced_height,
                target,
            );
        }
    }
}

#[cfg(feature = "flutter")]
async fn wallet_height(c: &Coin) -> Result<u32> {
    let mut connection = c.get_connection().await?;
    Ok(crate::sync::get_db_height(&mut connection, c.account)
        .await?
        .height)
}
