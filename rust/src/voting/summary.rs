//! Lightweight list-page progress; execution still uses the full resume plan.

use anyhow::Result;
use serde::Serialize;
use sqlx::{Row, SqliteConnection};

/// Proposal IDs come from the server's rounds response, including unanswered ones.
#[derive(Serialize)]
pub struct RoundInput {
    pub round_id: String,
    pub proposal_ids: Vec<u32>,
}

#[derive(Debug)]
pub struct LocalRoundSummary {
    pub round_id: String,
    pub snapshot_height: Option<i64>,
    pub bundle_count: i64,
    pub intent_count: i64,
    pub proposal_count: i64,
    pub undecided_count: i64,
    pub choice_count: i64,
    pub missing_vote_count: i64,
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
        } else if self.proposal_count > 0
            && self.undecided_count == 0
            && self.missing_vote_count == 0
            && (self.choice_count == 0 || self.bundle_count > 0)
        {
            ListAction::Review
        } else {
            ListAction::Resume
        }
    }
}

/// One read query for all server rounds, scoped to the seed's wallet ID.
/// Initialize the voting schema with `open_voting_db` before calling this.
pub async fn load_local_summaries(
    conn: &mut SqliteConnection,
    wallet_id: &str,
    rounds: &[RoundInput],
) -> Result<Vec<LocalRoundSummary>> {
    let started = std::time::Instant::now();
    let rows = sqlx::query(
        r#"
        WITH requested AS (
            SELECT value ->> '$.round_id' AS round_id,
                   value -> '$.proposal_ids' AS proposals
            FROM json_each(?1)
        ), proposals AS (
            SELECT r.round_id, p.value AS proposal_id
            FROM requested r, json_each(r.proposals) p
        ), bundles AS (
            SELECT b.round_id, b.bundle_index
            FROM voting_bundles b
            WHERE b.wallet_id = ?2
                AND b.round_id IN (SELECT round_id FROM requested)
        ), bundle_counts AS (
            SELECT round_id, COUNT(*) AS bundle_count
            FROM bundles
            GROUP BY round_id
        ), decisions AS (
            SELECT p.round_id, p.proposal_id, i.proposal_id AS answered_id,
                   i.skipped, i.choice
            FROM proposals p
            LEFT JOIN voting_ballot_intent i
                ON i.round_id = p.round_id AND i.proposal_id = p.proposal_id
                AND i.wallet_id = ?2
        ), decision_counts AS (
            SELECT round_id,
                   COUNT(*) AS proposal_count,
                   COUNT(answered_id) AS intent_count,
                   SUM(answered_id IS NULL) AS undecided_count,
                   SUM(CASE WHEN skipped = 0 THEN 1 ELSE 0 END) AS choice_count
            FROM decisions
            GROUP BY round_id
        ), confirmed_votes AS (
            SELECT v.round_id, v.bundle_index, v.proposal_id, v.choice
            FROM voting_votes v
            WHERE v.wallet_id = ?2
                AND v.round_id IN (SELECT round_id FROM requested)
                AND (v.tx_hash IS NOT NULL OR v.confirmed_without_hash != 0)
                AND v.vc_tree_position IS NOT NULL
                AND v.commitment_bundle_json IS NOT NULL
        ), missing_votes AS (
            -- Each chosen proposal needs a matching confirmed vote per bundle.
            SELECT d.round_id, COUNT(*) AS missing_vote_count
            FROM decisions d
            JOIN bundles b ON b.round_id = d.round_id
            LEFT JOIN confirmed_votes v
                ON v.round_id = d.round_id AND v.bundle_index = b.bundle_index
                AND v.proposal_id = d.proposal_id AND v.choice = d.choice
            WHERE d.skipped = 0 AND v.proposal_id IS NULL
            GROUP BY d.round_id
        )
        SELECT q.round_id, r.snapshot_height,
               COALESCE(b.bundle_count, 0) AS bundle_count,
               COALESCE(d.intent_count, 0) AS intent_count,
               COALESCE(d.proposal_count, 0) AS proposal_count,
               COALESCE(d.undecided_count, 0) AS undecided_count,
               COALESCE(d.choice_count, 0) AS choice_count,
               COALESCE(v.missing_vote_count, 0) AS missing_vote_count
        FROM requested q
        LEFT JOIN voting_rounds r ON r.round_id = q.round_id AND r.wallet_id = ?2
        LEFT JOIN bundle_counts b ON b.round_id = q.round_id
        LEFT JOIN decision_counts d ON d.round_id = q.round_id
        LEFT JOIN missing_votes v ON v.round_id = q.round_id
        "#,
    )
    .bind(serde_json::to_string(rounds)?)
    .bind(wallet_id)
    .fetch_all(conn)
    .await?;
    log::info!("Voting summary SQL: {:?} ({} rounds)", started.elapsed(), rounds.len());
    rows.into_iter()
        .map(|row| {
            Ok(LocalRoundSummary {
                round_id: row.try_get("round_id")?,
                snapshot_height: row.try_get("snapshot_height")?,
                bundle_count: row.try_get("bundle_count")?,
                intent_count: row.try_get("intent_count")?,
                proposal_count: row.try_get("proposal_count")?,
                undecided_count: row.try_get("undecided_count")?,
                choice_count: row.try_get("choice_count")?,
                missing_vote_count: row.try_get("missing_vote_count")?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn summary_handles_partial_skipped_and_wallet_scoped_votes() -> Result<()> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        let mut conn = pool.acquire().await?;
        super::super::open_voting_db(pool.clone(), &mut conn, "wallet").await?;
        sqlx::query("INSERT INTO voting_rounds (round_id, wallet_id, network, snapshot_height, ea_pk, nc_root, nullifier_imt_root, created_at) VALUES ('r', 'wallet', 'regtest', 100, X'00', X'00', X'00', 1)")
            .execute(&mut *conn).await?;
        sqlx::query("INSERT INTO voting_bundles (round_id, wallet_id, bundle_index) VALUES ('r', 'wallet', 0), ('r', 'wallet', 1)")
            .execute(&mut *conn).await?;
        let input = [
            RoundInput {
                round_id: "r".into(),
                proposal_ids: vec![1, 2],
            },
            RoundInput {
                round_id: "new".into(),
                proposal_ids: vec![1],
            },
        ];
        let initial = load_local_summaries(&mut conn, "wallet", &input).await?;
        assert_eq!(initial[0].snapshot_height, Some(100));
        assert_eq!(initial[0].bundle_count, 2);
        assert_eq!(initial[0].action(false), ListAction::StartVoting);
        assert_eq!(initial[1].action(false), ListAction::StartVoting);
        assert_eq!(initial[1].action(true), ListAction::ViewResults);
        sqlx::query("INSERT INTO voting_ballot_intent (round_id, wallet_id, proposal_id, skipped, choice, created_at, updated_at) VALUES ('r', 'wallet', 1, 0, 2, 1, 1), ('r', 'wallet', 2, 1, NULL, 1, 1)")
            .execute(&mut *conn).await?;
        let pending = load_local_summaries(&mut conn, "wallet", &input).await?;
        assert_eq!(pending[0].missing_vote_count, 2);
        assert_eq!(pending[0].action(false), ListAction::Resume);
        sqlx::query("INSERT INTO voting_votes (round_id, wallet_id, bundle_index, proposal_id, choice, created_at, confirmed_without_hash, vc_tree_position, commitment_bundle_json) VALUES ('r', 'wallet', 0, 1, 2, 1, 1, 0, '{}')")
            .execute(&mut *conn).await?;
        let partial = load_local_summaries(&mut conn, "wallet", &input).await?;
        assert_eq!(partial[0].missing_vote_count, 1);
        assert_eq!(partial[0].action(false), ListAction::Resume);
        sqlx::query("INSERT INTO voting_votes (round_id, wallet_id, bundle_index, proposal_id, choice, created_at, tx_hash, vc_tree_position, commitment_bundle_json) VALUES ('r', 'wallet', 1, 1, 2, 1, 'tx', 1, '{}')")
            .execute(&mut *conn).await?;
        let complete = load_local_summaries(&mut conn, "wallet", &input).await?;
        assert_eq!(complete[0].action(false), ListAction::Review);
        assert_eq!(complete[0].action(true), ListAction::ViewResults);
        let other = load_local_summaries(&mut conn, "other-wallet", &input).await?;
        assert_eq!(other[0].snapshot_height, None);
        assert_eq!(other[0].bundle_count, 0);
        assert_eq!(other[0].intent_count, 0);
        assert_eq!(other[0].action(false), ListAction::StartVoting);
        sqlx::query(
            "UPDATE voting_ballot_intent SET skipped = 1, choice = NULL WHERE proposal_id = 1",
        )
        .execute(&mut *conn)
        .await?;
        let skipped = load_local_summaries(&mut conn, "wallet", &input).await?;
        assert_eq!(skipped[0].action(false), ListAction::Review);
        // A newly discovered unanswered proposal prevents completion.
        let expanded = [RoundInput {
            round_id: "r".into(),
            proposal_ids: vec![1, 2, 3],
        }];
        let undecided = load_local_summaries(&mut conn, "wallet", &expanded).await?;
        assert_eq!(undecided[0].action(false), ListAction::Resume);
        Ok(())
    }
}
