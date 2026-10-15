use std::collections::HashSet;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use juniper::{FieldError, FieldResult, GraphQLInputObject, GraphQLObject};

use crate::api::voting::{Decision, VotingRoundListItem};
use crate::api::voting_drive::VotingDriveStatus;
use crate::graphql::{check_auth, Context};

#[derive(Clone, GraphQLObject)]
pub struct VotingProposal {
    pub proposal_id: i32,
    pub title: String,
    pub options: Vec<String>,
}

#[derive(Clone, GraphQLObject)]
pub struct VotingRound {
    pub round_id: String,
    pub title: String,
    pub status: String,
    pub snapshot_height: i32,
    pub bundle_count: i32,
    pub action: String,
    pub proposals: Vec<VotingProposal>,
}

#[derive(GraphQLInputObject)]
pub struct VotingSelectionInput {
    pub proposal_id: i32,
    /// Zero-based option index. Omit it to explicitly skip the proposal.
    pub choice: Option<i32>,
}

#[derive(GraphQLObject)]
pub struct VotingSubmissionStatus {
    pub round_id: String,
    pub running: bool,
    pub dispatches: String,
    pub completed_proposals: i32,
    pub total_proposals: i32,
    pub remaining_obligations: i32,
    pub shares_confirmed: i32,
    pub shares_total: i32,
    pub quiescence: Option<String>,
    pub failures: Vec<String>,
}

fn graphql_int(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

impl TryFrom<VotingRoundListItem> for VotingRound {
    type Error = FieldError;

    fn try_from(round: VotingRoundListItem) -> Result<Self, Self::Error> {
        Ok(Self {
            round_id: round.round_id,
            title: round.title,
            status: round.status,
            snapshot_height: i32::try_from(round.snapshot_height).map_err(|_| {
                FieldError::new(
                    "Voting round snapshot height is out of range",
                    juniper::Value::Null,
                )
            })?,
            bundle_count: graphql_int(round.bundle_count),
            action: round.action,
            proposals: round
                .proposals
                .into_iter()
                .map(|proposal| VotingProposal {
                    proposal_id: graphql_int(proposal.proposal_id),
                    title: proposal.title,
                    options: proposal.options,
                })
                .collect(),
        })
    }
}

impl From<VotingDriveStatus> for VotingSubmissionStatus {
    fn from(status: VotingDriveStatus) -> Self {
        Self {
            round_id: status.round_id,
            running: status.running,
            dispatches: status.dispatches.to_string(),
            completed_proposals: graphql_int(status.completed_proposals),
            total_proposals: graphql_int(status.total_proposals),
            remaining_obligations: graphql_int(status.remaining_obligations),
            shares_confirmed: graphql_int(status.shares_confirmed),
            shares_total: graphql_int(status.shares_total),
            quiescence: status.quiescence,
            failures: status.failures,
        }
    }
}

fn coin_for_account(
    id_account: i32,
    context: &Context,
    write: bool,
) -> FieldResult<crate::api::coin::Coin> {
    check_auth(context, id_account, write)?;
    let mut coin = context.coin.clone();
    coin.account = u32::try_from(id_account)
        .map_err(|_| FieldError::new("Account id must be non-negative", juniper::Value::Null))?;
    Ok(coin)
}
pub async fn rounds(id_account: i32, context: &Context) -> FieldResult<Vec<VotingRound>> {
    let coin = coin_for_account(id_account, context, false)?;
    crate::api::voting::voting_round_list(&coin)
        .await?
        .into_iter()
        .map(TryInto::try_into)
        .collect()
}

pub async fn round(
    id_account: i32,
    round_id: String,
    context: &Context,
) -> FieldResult<VotingRound> {
    let coin = coin_for_account(id_account, context, false)?;
    crate::api::voting::voting_round_list(&coin)
        .await?
        .into_iter()
        .find(|round| round.round_id == round_id)
        .ok_or_else(|| FieldError::new("Unknown voting round", juniper::Value::Null))?
        .try_into()
}

pub async fn submission_status(
    id_account: i32,
    round_id: String,
    context: &Context,
) -> FieldResult<VotingSubmissionStatus> {
    let coin = coin_for_account(id_account, context, false)?;
    Ok(
        crate::api::voting_drive::voting_drive_status(&round_id, &coin)
            .await?
            .into(),
    )
}

pub async fn submit_vote(
    id_account: i32,
    round_id: String,
    selections: Vec<VotingSelectionInput>,
    context: &Context,
) -> FieldResult<VotingSubmissionStatus> {
    let coin = coin_for_account(id_account, context, true)?;
    if selections.is_empty() {
        return Err("At least one voting selection is required".into());
    }

    // Resolve the authenticated roster rather than trusting caller-provided
    // option counts. This also supplies the display name used by the driver.
    let round = crate::api::voting::voting_round_list(&coin)
        .await?
        .into_iter()
        .find(|round| round.round_id == round_id)
        .ok_or_else(|| FieldError::new("Unknown voting round", juniper::Value::Null))?;

    let mut seen = HashSet::new();
    let mut validated = Vec::with_capacity(selections.len());
    for selection in selections {
        let proposal_id =
            u32::try_from(selection.proposal_id).map_err(|_| "Proposal id must be non-negative")?;
        if !seen.insert(proposal_id) {
            return Err(format!("Duplicate proposal id {proposal_id}").into());
        }
        let proposal = round
            .proposals
            .iter()
            .find(|proposal| proposal.proposal_id == proposal_id)
            .ok_or_else(|| format!("Proposal {proposal_id} does not belong to this round"))?;
        let num_options =
            u32::try_from(proposal.options.len()).map_err(|_| "Proposal has too many options")?;
        let decision = match selection.choice {
            Some(choice) => {
                let choice =
                    u32::try_from(choice).map_err(|_| "Voting choice must be non-negative")?;
                if choice >= num_options {
                    return Err(format!(
                        "Choice {choice} is out of range for proposal {proposal_id}"
                    )
                    .into());
                }
                Decision::Choice { choice }
            }
            None => Decision::Skipped,
        };
        validated.push((proposal_id, decision, num_options));
    }
    for (proposal_id, decision, num_options) in validated {
        crate::api::voting::voting_save_selection(
            &round.round_id,
            proposal_id,
            decision,
            num_options,
            &coin,
        )
        .await?;
    }

    // Bundle preparation is the explicit durable setup phase. The driver
    // itself is spawned and returns immediately, so chain/share delivery is
    // asynchronous and can be polled through votingSubmissionStatus.
    crate::api::voting_drive::voting_prepare_round(&round.round_id, &coin.url, &round.title, &coin)
        .await?;
    crate::api::voting_drive::voting_drive_start(&round.round_id, &coin.url, &round.title, &coin)
        .await?;

    // The foreground driver creates the helper shares. Once it quiesces, this
    // task runs tracking passes — the same mechanism the round page uses —
    // until every share is confirmed. The shares and their confirmations live
    // in the sidecar, so a server restart loses nothing; the next submission
    // or status poll simply runs the next pass.
    let tracking_coin = coin.clone();
    let tracking_round_id = round.round_id.clone();
    let helper_urls = round.helper_urls.clone();
    let vote_end_time = round.vote_end_time;
    tokio::spawn(async move {
        loop {
            match crate::api::voting_drive::voting_drive_status(&tracking_round_id, &tracking_coin)
                .await
            {
                Ok(status) if status.running => {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Ok(_) => break,
                Err(error) => {
                    tracing::error!("voting submission status failed: {error:#}");
                    return;
                }
            }
        }
        track_shares_until_done(&tracking_round_id, &helper_urls, vote_end_time, &tracking_coin)
            .await;
    });

    submission_status(id_account, round.round_id, context).await
}

/// Failed passes in a row after which the submission task stops retrying.
const MAX_CONSECUTIVE_SHARE_PASS_FAILURES: u32 = 5;
/// Wait after a failed pass, which carries no `next_delay_seconds`.
const SHARE_PASS_RETRY_DELAY: Duration = Duration::from_secs(30);

/// Runs share-tracking passes for a round until nothing is pending, the vote
/// end passes, or the helpers keep failing.
async fn track_shares_until_done(
    round_id: &str,
    helper_urls: &[String],
    vote_end_time: Option<u64>,
    coin: &crate::api::coin::Coin,
) {
    if helper_urls.is_empty() {
        return;
    }
    let mut failures = 0u32;
    loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if vote_end_time.is_some_and(|end| now >= end) {
            tracing::info!("voting share tracking for {round_id} stopped: vote end reached");
            return;
        }
        let delay = match crate::api::voting_share_tracking::track_shares_pass(
            round_id,
            helper_urls,
            vote_end_time,
            coin,
        )
        .await
        {
            Ok(report) => {
                failures = 0;
                if !report.unrecoverable.is_empty() {
                    tracing::warn!(
                        "voting share tracking for {round_id}: {} share(s) cannot be recovered",
                        report.unrecoverable.len()
                    );
                }
                match report.next_delay_seconds {
                    Some(seconds) => Duration::from_secs(seconds.max(1)),
                    None => return,
                }
            }
            Err(error) => {
                failures += 1;
                if failures >= MAX_CONSECUTIVE_SHARE_PASS_FAILURES {
                    tracing::error!(
                        "voting share tracking for {round_id} gave up after {failures} failed passes: {error:#}"
                    );
                    return;
                }
                tracing::warn!("voting share tracking pass for {round_id} failed: {error:#}");
                SHARE_PASS_RETRY_DELAY
            }
        };
        tokio::time::sleep(delay).await;
    }
}
