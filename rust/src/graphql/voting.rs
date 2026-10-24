use std::collections::HashSet;

use juniper::{FieldError, FieldResult, GraphQLInputObject, GraphQLObject};

use crate::api::voting::{Decision, VotingRoundListItem};
use crate::api::voting_drive::{VotingDrive, VotingDriveStatus};
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
    /// idle, casting, confirming_shares or stopped.
    pub phase: String,
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
            phase: status.phase,
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

fn authorized_account(id_account: i32, context: &Context, write: bool) -> FieldResult<u32> {
    check_auth(context, id_account, write)?;
    u32::try_from(id_account)
        .map_err(|_| FieldError::new("Account id must be non-negative", juniper::Value::Null))
}
pub async fn rounds(id_account: i32, context: &Context) -> FieldResult<Vec<VotingRound>> {
    let account = authorized_account(id_account, context, false)?;
    crate::api::voting::voting_round_list(account, &context.coin)
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
    let account = authorized_account(id_account, context, false)?;
    crate::api::voting::voting_round_list(account, &context.coin)
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
    let account = authorized_account(id_account, context, false)?;
    let live = context
        .voting_drives
        .lock()
        .await
        .get(&round_id)
        .and_then(VotingDrive::status);
    let status = match live {
        Some(status) => status,
        None => {
            crate::api::voting_drive::voting_drive_status(&round_id, account, &context.coin).await?
        }
    };
    Ok(status.into())
}

pub async fn submit_vote(
    id_account: i32,
    round_id: String,
    selections: Vec<VotingSelectionInput>,
    context: &Context,
) -> FieldResult<VotingSubmissionStatus> {
    let account = authorized_account(id_account, context, true)?;
    let coin = &context.coin;
    if selections.is_empty() {
        return Err("At least one voting selection is required".into());
    }

    // Resolve the authenticated roster rather than trusting caller-provided
    // option counts. This also supplies the display name used by the driver.
    let round = crate::api::voting::voting_round_list(account, coin)
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
            account,
            coin,
        )
        .await?;
    }

    // Bundle preparation is the explicit durable setup phase. The run
    // itself casts the votes, then confirms the helper shares until the
    // round is settled, and its progress is polled through
    // votingSubmissionStatus. The shares and their confirmations live in the
    // sidecar, so a server restart loses nothing: the next submission simply
    // resumes where the run left off. A round whose run is still live keeps
    // it.
    crate::api::voting_drive::voting_prepare_round(
        &round.round_id,
        &coin.url,
        &round.title,
        account,
        coin,
    )
    .await?;
    {
        let mut drives = context.voting_drives.lock().await;
        let drive = drives
            .entry(round.round_id.clone())
            .or_insert_with(VotingDrive::new);
        if !drive.is_running() {
            drive
                .start(&round.round_id, &coin.url, &round.title, account, coin)
                .await?;
        }
    }

    submission_status(id_account, round.round_id, context).await
}
