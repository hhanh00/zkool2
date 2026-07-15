use anyhow::Result;

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

use crate::api::coin::Coin;

/// Current migration status for the UI.
#[cfg_attr(feature = "flutter", frb)]
#[derive(Clone, Debug)]
pub struct MigrationStatus {
    pub phase: String,            // "idle" | "splitting" | "migrating" | "complete"
    pub progress: f64,            // 0.0 – 1.0
    pub next_action: String,      // human-readable next step
    pub work_summary: String,     // remaining work breakdown
    pub sd_notes_count: u32,
    pub non_sd_notes_count: u32,
}

/// Result of a migration step.
#[cfg_attr(feature = "flutter", frb)]
pub enum MigrationEvent {
    /// A split transaction was broadcast.
    SplitComplete { fee: u64 },
    /// A migration transaction was broadcast.
    MigrateComplete { fee: u64 },
    /// Migration is complete.
    Complete,
    /// No action needed (idempotent — call again later).
    NothingToDo,
    /// An error occurred.
    Error { message: String },
}

/// Run one migration step. Fully idempotent — re-scans notes on every call.
/// The caller should poll this periodically (e.g., every 6 seconds).
#[cfg_attr(feature = "flutter", frb)]
pub async fn step_migration(c: &Coin) -> Result<MigrationEvent> {
    let network = c.network();
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;

    // Sync the wallet before scanning notes, so we have the latest state.
    // Don't fail if sync is already running or encounters an error.
    let current_height = client.latest_height().await?;
    let _ = crate::sync::synchronize_impl(
        (),
        vec![c.account],
        current_height,
        100_000,
        10_000,
        10_000,
        false,
        c,
    )
    .await;

    let event = crate::migrate::step(&network, &mut connection, &mut client, c.account)
        .await
        .map_err(|e| anyhow::anyhow!("step: {e}"))?;

    // After a successful broadcast, sync again to capture the new transaction.
    let needs_sync = matches!(
        event,
        crate::migrate::MigrationEvent::SplitComplete { .. }
            | crate::migrate::MigrationEvent::MigrateComplete { .. }
    );
    if needs_sync {
        let height = client.latest_height().await?;
        let _ = crate::sync::synchronize_impl(
            (),
            vec![c.account],
            height,
            100_000,
            10_000,
            10_000,
            false,
            c,
        )
        .await;
    }

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

/// Get current migration status for the UI.
/// Returns phase, progress, and work breakdown based on current note state.
#[cfg_attr(feature = "flutter", frb)]
pub async fn get_migration_status(c: &Coin) -> Result<MigrationStatus> {
    let mut connection = c.get_connection().await?;
    let status = crate::migrate::get_status(&mut connection, c.account).await?;

    Ok(MigrationStatus {
        phase: status.phase,
        progress: status.progress,
        next_action: status.next_action,
        work_summary: status.work_summary,
        sd_notes_count: status.sd_notes_count,
        non_sd_notes_count: status.non_sd_notes_count,
    })
}
