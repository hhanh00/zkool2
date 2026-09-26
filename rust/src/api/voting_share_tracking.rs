//! FRB controls for the voting sidecar's durable helper-share retry workers.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};
use zcash_voting::prelude::{track_pending_shares, ShareTimingPolicy, ShareTrackingParams};
use zcash_voting::{HelperClient, HelperHealth};

use crate::{
    api::{coin::Coin, voting::wallet_id},
    voting::sidecar::{VotingSidecar, ZkoolHelperTransport},
};

/// A round whose persisted helper shares need a foreground retry worker.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VotingPendingShareRound {
    pub round_id: String,
    pub session_json: Option<String>,
}

/// Lists the current wallet's durable share-recovery work. This is empty when
/// resuming the app has nothing to do.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_pending_share_rounds(c: &Coin) -> Result<Vec<VotingPendingShareRound>> {
    let sidecar = VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet_id(c).await?).await?;
    Ok(sidecar
        .pending_share_rounds()
        .await?
        .into_iter()
        .map(|round| VotingPendingShareRound {
            round_id: round.round_id,
            session_json: round.session_json,
        })
        .collect())
}

/// Starts durable helper-share tracking for a round. The caller supplies the
/// freshly resolved helper fleet and vote-end boundary; the sidecar supplies
/// the selected wallet transport and owns the cancellable background task.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_start_share_tracking(
    round_id: &str,
    helper_urls: Vec<String>,
    vote_end_time_seconds: Option<u64>,
    c: &Coin,
) -> Result<bool> {
    let sidecar = VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet_id(c).await?).await?;
    sidecar.start_share_tracking(
        round_id.to_owned(),
        helper_urls,
        vote_end_time_seconds,
        c.transport,
        c.proxy.clone(),
    )
}

/// Cancels one round's share-tracking task. Pending shares remain durable and
/// can be resumed safely on the next foreground lifecycle transition.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_cancel_share_tracking(round_id: &str, c: &Coin) -> Result<bool> {
    let sidecar = VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet_id(c).await?).await?;
    Ok(sidecar.cancel_share_tracking(round_id))
}

/// Cancels every live share-tracking task for the current wallet.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_cancel_all_share_tracking(c: &Coin) -> Result<usize> {
    let sidecar = VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet_id(c).await?).await?;
    Ok(sidecar.cancel_all_share_tracking())
}

/// Runs exactly one helper-share tracking pass for a round and returns how many
/// shares it confirmed.
///
/// This is the page-scoped alternative to a resident `ShareTrackingDriver`: the
/// caller polls it while the round is open, so confirmation advances every pass
/// without a long-lived in-process run. Everything the pass needs comes from
/// the arguments and the durable sidecar.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_track_shares_once(
    round_id: &str,
    helper_urls: Vec<String>,
    vote_end_time_seconds: Option<u64>,
    c: &Coin,
) -> Result<u32> {
    let sidecar = VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet_id(c).await?).await?;
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
        configured_server_urls: &helper_urls,
        now_seconds,
        vote_end_time_seconds,
        policy: ShareTimingPolicy::default(),
    };
    // One pass walks every unconfirmed share, and a stalled helper can hold it
    // for the per-share poll budget. Bound the whole pass so the page's poll
    // loop is never wedged; a cancelled pass keeps whatever it confirmed.
    let cancelled = Arc::new(AtomicBool::new(false));
    {
        let cancelled = Arc::clone(&cancelled);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(45)).await;
            cancelled.store(true, Ordering::Relaxed);
        });
    }
    let report = track_pending_shares(&db, &params, &client, &|| {
        cancelled.load(Ordering::Relaxed)
    })
    .await?;
    Ok(u32::try_from(report.confirmed.len()).unwrap_or(u32::MAX))
}
