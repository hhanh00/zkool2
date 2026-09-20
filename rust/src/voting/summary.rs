//! Lightweight list-page progress derived from the voting sidecar.
//!
//! This is display state only. Execution must use the crate's full
//! [`zcash_voting::session::resume_plan`] immediately before doing work.

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use serde::Serialize;
use zcash_voting::session::{resume_plan, Decision};

use super::sidecar::VotingSidecar;

/// Proposal IDs come from the authenticated rounds config, including
/// proposals for which the voter has not made a decision yet.
#[derive(Clone, Debug, Serialize)]
pub struct RoundInput {
    pub round_id: String,
    pub proposal_ids: Vec<u32>,
}

#[derive(Debug)]
pub struct LocalRoundSummary {
    pub round_id: String,
    pub snapshot_height: Option<u64>,
    pub bundle_count: u32,
    pub intent_count: usize,
    pub proposal_count: usize,
    pub undecided_count: usize,
    pub choice_count: usize,
    all_decided: bool,
    completed_for_display: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ListAction {
    StartVoting,
    Resume,
    Review,
    ViewResults,
}

impl LocalRoundSummary {
    /// Display-only action. It must not be used to decide recovery operations.
    pub fn action(&self, chain_finished: bool) -> ListAction {
        if chain_finished {
            ListAction::ViewResults
        } else if self.snapshot_height.is_none() || self.intent_count == 0 {
            ListAction::StartVoting
        } else if self.all_decided && (self.choice_count == 0 || self.completed_for_display) {
            ListAction::Review
        } else {
            ListAction::Resume
        }
    }
}

/// Loads list-page state through the crate-owned voting sidecar.
///
/// The voting crate remains the authority for completion and recovery state;
/// this adapter only combines its plan with the authenticated proposal roster.
pub async fn load_local_summaries(
    sidecar: &VotingSidecar,
    rounds: &[RoundInput],
) -> Result<Vec<LocalRoundSummary>> {
    let started = std::time::Instant::now();
    let rounds = rounds.to_vec();
    let count = rounds.len();
    let summaries = sidecar
        .run(move |db| {
            let stored: HashMap<_, _> = db
                .list_rounds()?
                .into_iter()
                .map(|round| (round.round_id.clone(), round))
                .collect();

            rounds
                .into_iter()
                .map(|input| {
                    let proposal_count = input.proposal_ids.len();
                    let Some(round) = stored.get(&input.round_id) else {
                        return Ok(LocalRoundSummary {
                            round_id: input.round_id,
                            snapshot_height: None,
                            bundle_count: 0,
                            intent_count: 0,
                            proposal_count,
                            undecided_count: proposal_count,
                            choice_count: 0,
                            all_decided: false,
                            completed_for_display: false,
                        });
                    };

                    let roster: HashSet<_> = input.proposal_ids.iter().copied().collect();
                    let intents = db.ballot_intents(&input.round_id)?;
                    let intent_count = intents
                        .iter()
                        .filter(|(proposal_id, _)| roster.contains(proposal_id))
                        .count();
                    let choice_count = intents
                        .iter()
                        .filter(|(proposal_id, decision)| {
                            roster.contains(proposal_id) && matches!(decision, Decision::Choice(_))
                        })
                        .count();
                    let plan = resume_plan(db, &input.round_id, &input.proposal_ids)?;

                    Ok(LocalRoundSummary {
                        round_id: input.round_id,
                        snapshot_height: Some(round.snapshot_height),
                        bundle_count: db.get_bundle_count(&round.round_id)?,
                        intent_count,
                        proposal_count,
                        undecided_count: plan.open_proposals.len(),
                        choice_count,
                        all_decided: plan.all_decided,
                        completed_for_display: plan.completed_for_display,
                    })
                })
                .collect()
        })
        .await?;
    log::info!(
        "Voting sidecar summary: {:?} ({} rounds)",
        started.elapsed(),
        count
    );
    Ok(summaries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(
        snapshot_height: Option<u64>,
        intent_count: usize,
        choice_count: usize,
        all_decided: bool,
        completed_for_display: bool,
    ) -> LocalRoundSummary {
        LocalRoundSummary {
            round_id: "round".into(),
            snapshot_height,
            bundle_count: 0,
            intent_count,
            proposal_count: 2,
            undecided_count: usize::from(!all_decided),
            choice_count,
            all_decided,
            completed_for_display,
        }
    }

    #[test]
    fn actions_follow_sidecar_plan_state() {
        assert_eq!(
            summary(None, 0, 0, false, false).action(false),
            ListAction::StartVoting
        );
        assert_eq!(
            summary(Some(100), 1, 1, false, false).action(false),
            ListAction::Resume
        );
        assert_eq!(
            summary(Some(100), 2, 1, true, false).action(false),
            ListAction::Resume
        );
        assert_eq!(
            summary(Some(100), 2, 1, true, true).action(false),
            ListAction::Review
        );
        assert_eq!(
            summary(Some(100), 2, 0, true, false).action(false),
            ListAction::Review
        );
        assert_eq!(
            summary(Some(100), 1, 1, false, false).action(true),
            ListAction::ViewResults
        );
    }

    #[tokio::test]
    async fn unknown_rounds_are_startable() -> Result<()> {
        let db = zcash_voting::prelude::VotingDb::open(":memory:")?;
        db.set_wallet_id("wallet");
        let sidecar = VotingSidecar::from_db_for_test(db);
        let rows = load_local_summaries(
            &sidecar,
            &[RoundInput {
                round_id: "00".repeat(32),
                proposal_ids: vec![1, 2],
            }],
        )
        .await?;

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].snapshot_height, None);
        assert_eq!(rows[0].proposal_count, 2);
        assert_eq!(rows[0].undecided_count, 2);
        assert_eq!(rows[0].action(false), ListAction::StartVoting);
        Ok(())
    }

    #[tokio::test]
    async fn stored_skipped_ballot_is_reviewable() -> Result<()> {
        use zcash_voting::prelude::{Network, RoundParams};

        let db = zcash_voting::prelude::VotingDb::open(":memory:")?;
        db.set_wallet_id("wallet");
        let round_id = "00".repeat(32);
        db.create_round(
            Network::Regtest,
            &RoundParams {
                vote_round_id: round_id.clone(),
                snapshot_height: 100,
                ea_pk: vec![0; 32],
                nc_root: vec![0; 32],
                nullifier_imt_root: vec![0; 32],
            },
            None,
        )?;
        db.set_ballot_intent(&round_id, 1, Decision::Skipped, 2)?;
        db.set_ballot_intent(&round_id, 2, Decision::Skipped, 2)?;
        let sidecar = VotingSidecar::from_db_for_test(db);

        let rows = load_local_summaries(
            &sidecar,
            &[RoundInput {
                round_id,
                proposal_ids: vec![1, 2],
            }],
        )
        .await?;

        assert_eq!(rows[0].snapshot_height, Some(100));
        assert_eq!(rows[0].intent_count, 2);
        assert_eq!(rows[0].choice_count, 0);
        assert_eq!(rows[0].undecided_count, 0);
        assert_eq!(rows[0].action(false), ListAction::Review);
        Ok(())
    }
}
