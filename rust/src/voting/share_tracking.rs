//! Helper-share confirmation, one durable pass at a time.
//!
//! A pass walks the round's unconfirmed shares, asks the helpers that hold
//! them for confirmation, and resubmits the overdue ones; every pass is
//! planned from the durable sidecar, so a restart simply runs the next one.
//! The drive task runs passes until the round is settled
//! ([`track_shares_until_done`]); the one-pass API in
//! `crate::api::voting_share_tracking` runs a single one for a host that
//! watches a round with no live run.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use zcash_voting::prelude::{
    track_pending_shares, ShareTimingPolicy, ShareTrackingParams, ShareTrackingReport, VotingDb,
};
use zcash_voting::HelperClient;

/// One pass walks every unconfirmed share, and a stalled helper can hold it
/// for the per-share poll budget. The whole pass is bounded so a caller's
/// loop is never wedged; a cancelled pass keeps whatever it confirmed.
pub const PASS_BUDGET: Duration = Duration::from_secs(45);

/// Wait after a failed pass, which carries no `next_delay_seconds`; doubled
/// on every further failure in a row, up to [`MAX_RETRY_DELAY`].
pub const RETRY_DELAY: Duration = Duration::from_secs(30);
pub const MAX_RETRY_DELAY: Duration = Duration::from_secs(5 * 60);

/// Why [`track_shares_until_done`] stopped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrackingOutcome {
    /// Every share is confirmed.
    Done,
    /// The round's vote end passed with shares still unconfirmed.
    RoundClosed { unconfirmed: u32 },
    /// Nothing is pending, but these shares cannot be repaired by retrying.
    Unrecoverable { count: u32 },
    /// The caller cancelled.
    Cancelled,
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Runs exactly one helper-share tracking pass for a round against `db`.
///
/// The report's `next_delay_seconds` tells the caller when to run another
/// pass, or that nothing is pending.
pub async fn track_shares_pass(
    db: &VotingDb,
    client: &HelperClient,
    round_id: &str,
    helper_urls: &[String],
    vote_end_time_seconds: Option<u64>,
) -> Result<ShareTrackingReport> {
    let params = ShareTrackingParams {
        round_id,
        configured_server_urls: helper_urls,
        now_seconds: now_seconds(),
        vote_end_time_seconds,
        policy: ShareTimingPolicy::default(),
    };
    let cancelled = Arc::new(AtomicBool::new(false));
    {
        let cancelled = Arc::clone(&cancelled);
        tokio::spawn(async move {
            tokio::time::sleep(PASS_BUDGET).await;
            cancelled.store(true, Ordering::Relaxed);
        });
    }
    Ok(track_pending_shares(db, &params, client, &|| cancelled.load(Ordering::Relaxed)).await?)
}

/// The shares of `round_id` not yet confirmed, read on a blocking thread.
pub async fn unconfirmed_count(db: Arc<VotingDb>, round_id: &str) -> Result<u32> {
    let round = round_id.to_owned();
    let count = tokio::task::spawn_blocking(move || db.get_unconfirmed_delegations(&round))
        .await??
        .len();
    Ok(u32::try_from(count).unwrap_or(u32::MAX))
}

/// Sleeps `delay` in one-second slices, returning early once `cancelled`.
async fn sleep_unless_cancelled(delay: Duration, cancelled: &(dyn Fn() -> bool + Send + Sync)) {
    let deadline = tokio::time::Instant::now() + delay;
    while tokio::time::Instant::now() < deadline && !cancelled() {
        let left = deadline.saturating_duration_since(tokio::time::Instant::now());
        tokio::time::sleep(left.min(Duration::from_secs(1))).await;
    }
}

/// Runs tracking passes for a round until every share is confirmed, the
/// vote end passes, the remaining shares cannot be recovered, or `cancelled`.
///
/// A pass is run as soon as the SDK's timing says a share is due, and a
/// failed pass (helpers unreachable) is retried with a growing delay for as
/// long as the round is open: the host asked for the vote to be carried to
/// confirmation, not for a report of a flaky network. `on_pass` runs after
/// every pass with its report and the shares still unconfirmed, for the
/// host's progress display.
pub async fn track_shares_until_done(
    db: Arc<VotingDb>,
    client: &HelperClient,
    round_id: &str,
    helper_urls: &[String],
    vote_end_time_seconds: Option<u64>,
    cancelled: &(dyn Fn() -> bool + Send + Sync),
    mut on_pass: impl FnMut(&ShareTrackingReport, u32),
) -> TrackingOutcome {
    let closed = |unconfirmed: u32| TrackingOutcome::RoundClosed { unconfirmed };
    let mut retry_delay = RETRY_DELAY;
    loop {
        if cancelled() {
            return TrackingOutcome::Cancelled;
        }
        if vote_end_time_seconds.is_some_and(|end| now_seconds() >= end) {
            tracing::info!("voting share tracking for {round_id} stopped: vote end reached");
            return closed(
                unconfirmed_count(Arc::clone(&db), round_id)
                    .await
                    .unwrap_or(0),
            );
        }
        let pass =
            track_shares_pass(&db, client, round_id, helper_urls, vote_end_time_seconds).await;
        let delay = match pass {
            Ok(report) => {
                retry_delay = RETRY_DELAY;
                let unconfirmed = unconfirmed_count(Arc::clone(&db), round_id)
                    .await
                    .unwrap_or_else(|_| {
                        report
                            .unconfirmed_at_entry
                            .unwrap_or(u32::MAX)
                            .saturating_sub(u32::try_from(report.confirmed.len()).unwrap_or(0))
                    });
                on_pass(&report, unconfirmed);
                if cancelled() {
                    return TrackingOutcome::Cancelled;
                }
                if unconfirmed == 0 {
                    return TrackingOutcome::Done;
                }
                match report.next_delay_seconds {
                    Some(seconds) => Duration::from_secs(seconds.max(1)),
                    None if !report.unrecoverable.is_empty() => {
                        tracing::warn!(
                            "voting share tracking for {round_id}: {} share(s) cannot be recovered",
                            report.unrecoverable.len()
                        );
                        return TrackingOutcome::Unrecoverable {
                            count: u32::try_from(report.unrecoverable.len()).unwrap_or(u32::MAX),
                        };
                    }
                    None => return TrackingOutcome::Done,
                }
            }
            Err(error) => {
                tracing::warn!(
                    "voting share tracking pass for {round_id} failed, next try in {}s: {error:#}",
                    retry_delay.as_secs()
                );
                let delay = retry_delay;
                retry_delay = (retry_delay * 2).min(MAX_RETRY_DELAY);
                delay
            }
        };
        sleep_unless_cancelled(delay, cancelled).await;
    }
}
