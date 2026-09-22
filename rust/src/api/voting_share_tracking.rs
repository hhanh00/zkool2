//! FRB controls for the voting sidecar's durable helper-share retry workers.

use std::path::PathBuf;

use anyhow::Result;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

use crate::{
    api::{coin::Coin, voting::wallet_id},
    voting::sidecar::VotingSidecar,
};

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
