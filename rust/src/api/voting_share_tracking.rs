//! FRB API for one helper-share tracking pass.
//!
//! The submission run (`voting_drive_run`) confirms the shares itself; this
//! single pass is for a host that watches a round with no live run, such as
//! the GraphQL server after a restart. Everything the pass needs comes from
//! the arguments and the durable sidecar.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;
use zcash_voting::prelude::ShareTrackingReport;
use zcash_voting::{HelperClient, HelperHealth};

use crate::{
    api::{coin::Coin, voting::wallet_id},
    voting::{chain::ZkoolHelperTransport, share_tracking, sidecar::VotingSidecar},
};

/// Runs exactly one helper-share tracking pass for a round of `account`.
pub(crate) async fn track_shares_pass(
    round_id: &str,
    helper_urls: &[String],
    vote_end_time_seconds: Option<u64>,
    account: u32,
    c: &Coin,
) -> Result<ShareTrackingReport> {
    let sidecar =
        VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet_id(account, c).await?).await?;
    let client = HelperClient::new(
        Arc::new(ZkoolHelperTransport::new(c.transport, &c.proxy)),
        HelperHealth::default(),
    );
    share_tracking::track_shares_pass(
        &sidecar.db(),
        &client,
        round_id,
        helper_urls,
        vote_end_time_seconds,
    )
    .await
}

/// Runs one helper-share tracking pass and returns how many shares it
/// confirmed.
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
