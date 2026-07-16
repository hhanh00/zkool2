use anyhow::Result;

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

/// Single-shot step (kept for FRB generated-code compatibility).
#[cfg_attr(feature = "flutter", frb)]
pub async fn step_migration(c: &Coin) -> Result<MigrationEvent> {
    let (event, _status) = do_step(c, 0, 0).await?;
    Ok(match event {
        crate::migrate::MigrationEvent::SplitComplete { fee } => MigrationEvent::SplitComplete { fee },
        crate::migrate::MigrationEvent::MigrateComplete { fee } => MigrationEvent::MigrateComplete { fee },
        crate::migrate::MigrationEvent::Complete => MigrationEvent::Complete,
        crate::migrate::MigrationEvent::NothingToDo => MigrationEvent::NothingToDo,
    })
}

/// Run migration to completion, streaming MigrationStatus to Flutter.
///
/// `mean_delay_ms` controls the mean wait time (in milliseconds) of the
/// exponential random delay between migration steps. Longer delays make
/// it harder for an observer to correlate the transactions.
#[cfg_attr(feature = "flutter", frb)]
pub async fn run_migration(
    sink: StreamSink<MigrationStatus>,
    c: &Coin,
    mean_delay_ms: u64,
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
            split_fees: 0, migrate_fees: 0, total_fees: 0,
            sd_notes_count: 0, non_sd_notes_count: 0,
            progress: 1.0,
            next_action: String::new(), work_summary: String::new(),
        }).ok();
        return Ok(());
    }

    let mut acc_split = 0u64;
    let mut acc_migrate = 0u64;

    loop {
        let (event, status) = do_step(c, acc_split, acc_migrate).await?;

        // Accumulate fees from broadcast events.
        match &event {
            crate::migrate::MigrationEvent::SplitComplete { fee } => acc_split += fee,
            crate::migrate::MigrationEvent::MigrateComplete { fee } => acc_migrate += fee,
            _ => {}
        }

        let is_complete = matches!(event, crate::migrate::MigrationEvent::Complete);

        sink.add(status).ok();

        if is_complete {
            break;
        }

        let mean = mean_delay_ms as f64;
        let u = (OsRng.next_u32() as f64 + 1.0) / (u32::MAX as f64 + 2.0);
        let delay_ms = ((-mean * u.ln()) as u64).min(mean_delay_ms * 4);
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
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
) -> Result<(crate::migrate::MigrationEvent, MigrationStatus)> {
    let network = c.network();
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;

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

    // Build status from current wallet state + accumulated fees.
    let all_notes = crate::pay::plan::fetch_unspent_notes_grouped_by_pool(&mut connection, c.account).await?;
    let orchard_zec: Vec<&crate::pay::InputNote> = all_notes
        .iter()
        .filter(|n| n.pool == 2 && n.asset_base == vec![0u8; 32])
        .collect();
    let sd_count = orchard_zec.iter().filter(|n| crate::migrate::is_sd(n.amount)).count() as u32;

    let non_sd_vals: Vec<u64> = orchard_zec
        .iter()
        .filter(|n| !crate::migrate::is_sd(n.amount))
        .map(|n| n.amount)
        .collect();
    let non_sd_total: u64 = non_sd_vals.iter().sum();
    let effective_non_sd = if non_sd_total >= crate::migrate::MIN_SD {
        non_sd_vals.len() as u32
    } else {
        0
    };

    let phase = match &event {
        crate::migrate::MigrationEvent::Complete => "complete",
        _ if effective_non_sd > 0 => "splitting",
        _ if sd_count > 0 => "migrating",
        _ => "complete",
    };

    // Re-read for updated counts after potential broadcast + sync.
    let all_notes = crate::pay::plan::fetch_unspent_notes_grouped_by_pool(&mut connection, c.account).await?;
    let orchard_zec: Vec<&crate::pay::InputNote> = all_notes
        .iter()
        .filter(|n| n.pool == 2 && n.asset_base == vec![0u8; 32])
        .collect();
    let sd_count = orchard_zec.iter().filter(|n| crate::migrate::is_sd(n.amount)).count() as u32;
    let non_sd_vals: Vec<u64> = orchard_zec
        .iter()
        .filter(|n| !crate::migrate::is_sd(n.amount))
        .map(|n| n.amount)
        .collect();
    let non_sd_total: u64 = non_sd_vals.iter().sum();
    let effective_non_sd = if non_sd_total >= crate::migrate::MIN_SD {
        non_sd_vals.len() as u32
    } else {
        0
    };

    Ok((event, MigrationStatus {
        phase: phase.to_string(),
        split_fees: acc_split,
        migrate_fees: acc_migrate,
        total_fees: acc_split + acc_migrate,
        sd_notes_count: sd_count,
        non_sd_notes_count: effective_non_sd,
        progress: if sd_count + effective_non_sd > 0 {
            sd_count as f64 / (sd_count + effective_non_sd) as f64
        } else { 1.0 },
        next_action: String::new(),
        work_summary: format!("SD: {}, non-SD: {}", sd_count, effective_non_sd),
    }))
}
