//! Helper-share confirmation, one durable pass at a time.
//!
//! There is exactly one share mechanism: a pass over the round's unconfirmed
//! shares, run by whoever is watching the round — the round page while it is
//! open, or the GraphQL submission task after a drive. Every pass is planned
//! from the durable sidecar, so there is no resident worker to start, cancel,
//! or resume, and a restart simply runs the next pass.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;
use zcash_voting::prelude::{
    track_pending_shares, ShareTimingPolicy, ShareTrackingParams, ShareTrackingReport,
};
use zcash_voting::{HelperClient, HelperHealth};

use crate::{
    api::{coin::Coin, voting::wallet_id},
    voting::{chain::ZkoolHelperTransport, sidecar::VotingSidecar},
};

/// One pass walks every unconfirmed share, and a stalled helper can hold it
/// for the per-share poll budget. The whole pass is bounded so a caller's
/// poll loop is never wedged; a cancelled pass keeps whatever it confirmed.
const PASS_BUDGET: Duration = Duration::from_secs(45);

/// Runs exactly one helper-share tracking pass for a round.
///
/// Everything the pass needs comes from the arguments and the durable sidecar.
/// The report's `next_delay_seconds` tells the caller when to run another
/// pass, or that nothing is pending.
pub(crate) async fn track_shares_pass(
    round_id: &str,
    helper_urls: &[String],
    vote_end_time_seconds: Option<u64>,
    account: u32,
    c: &Coin,
) -> Result<ShareTrackingReport> {
    let sidecar =
        VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet_id(account, c).await?).await?;
    let db = sidecar.db();
    let client = HelperClient::new(
        Arc::new(ZkoolHelperTransport::new(c.transport, &c.proxy)),
        HelperHealth::default(),
    );
    let now_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let params = ShareTrackingParams {
        round_id,
        configured_server_urls: helper_urls,
        now_seconds,
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
    Ok(track_pending_shares(&db, &params, &client, &|| {
        cancelled.load(Ordering::Relaxed)
    })
    .await?)
}

/// Runs one helper-share tracking pass and returns how many shares it
/// confirmed. The round page polls this while it is open, so confirmation
/// advances without a long-lived in-process run.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_track_shares_once(
    round_id: &str,
    helper_urls: Vec<String>,
    vote_end_time_seconds: Option<u64>,
    account: u32,
    c: &Coin,
) -> Result<u32> {
    let report =
        track_shares_pass(round_id, &helper_urls, vote_end_time_seconds, account, c).await?;
    Ok(u32::try_from(report.confirmed.len()).unwrap_or(u32::MAX))
}
