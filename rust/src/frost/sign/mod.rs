mod exec;
mod plan;
mod pubkey;
mod state;
mod step;
mod task;

pub use step::{sign_step, SignStepOutcome};

pub(super) const COMMITMENT_PREFIX: &[u8] = b"CMT6";
pub(super) const SIGPACKAGE_PREFIX: &[u8] = b"SPK5";
pub(super) const SIGSHARE_PREFIX: &[u8] = b"SSH3";
/// The coordinator's mailbox address for the session, broadcast alongside
/// the signing packages so that members need no stored address for it.
pub(super) const COORDINATOR_PREFIX: &[u8] = b"CRD1";

// ── Public API (unchanged surface for flutter / graphql callers) ──────────────

use anyhow::Result;
use bincode::config;
use sqlx::SqliteConnection;
use tracing::info;

use crate::{
    api::{
        coin::Network,
        frost::{FrostSignParams, SigningStatus},
        pay::PcztPackage,
    },
    frost::dkg::delete_frost_state,
    Client, Sink,
};

use task::SignTask;


pub async fn init_sign(
    connection: &mut SqliteConnection,
    account: u32,
    funding_account: u32,
    coordinator: u8,
    pczt: &PcztPackage,
) -> Result<()> {
    info!(
        "init_sign: account={}, funding_account={}, coordinator={}",
        account, funding_account, coordinator
    );
    let pczt_bytes = bincode::encode_to_vec(pczt, config::legacy()).unwrap();
    sqlx::query("INSERT INTO props(key, value) VALUES ('frost_pczt', ?) ON CONFLICT DO NOTHING")
        .bind(&pczt_bytes)
        .execute(&mut *connection)
        .await?;
    let params = FrostSignParams {
        account,
        coordinator,
        funding_account,
    };
    let params_json = serde_json::to_string(&params).unwrap();
    sqlx::query(
        "INSERT INTO props(key, value) VALUES ('frost_sign_params', ?) ON CONFLICT DO NOTHING",
    )
    .bind(&params_json)
    .execute(&mut *connection)
    .await?;
    sqlx::query("INSERT INTO props(key, value) VALUES ('dkg_account', ?) ON CONFLICT DO NOTHING")
        .bind(funding_account)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

pub async fn reset_sign(connection: &mut SqliteConnection) -> Result<()> {
    delete_frost_state(&mut *connection).await
}

pub async fn is_signing_in_progress(connection: &mut SqliteConnection) -> Result<bool> {
    let exists =
        sqlx::query_as::<_, (bool,)>("SELECT TRUE FROM props WHERE key = 'frost_pczt'")
            .fetch_optional(&mut *connection)
            .await?;
    Ok(exists.is_some())
}

pub async fn in_sign(connection: &mut SqliteConnection) -> Result<bool> {
    let exists = sqlx::query("SELECT 1 FROM props WHERE key LIKE 'frost_%'")
        .fetch_optional(connection)
        .await?;
    Ok(exists.is_some())
}

// ── do_sign / do_sign_impl ───────────────────────────────────────────────────
//
// One signing pass, reported to the status sink the way the sequential
// orchestrator used to report its phases: one event per task the pass
// executed, then the wait or terminal it stopped at.

pub async fn do_sign_impl(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    height: u32,
    status: impl Sink<SigningStatus>,
) -> Result<()> {
    info!("sign: starting at height {}", height);

    let Some(outcome) = sign_step(network, connection, client, height).await? else {
        info!("sign: no signing session in progress");
        return Ok(());
    };
    report_outcome(&outcome, &status).await;
    Ok(())
}

/// Tell the status sink what a pass did, in order.
async fn report_outcome(outcome: &SignStepOutcome, status: &impl Sink<SigningStatus>) {
    for task in &outcome.executed {
        for s in sign_statuses_for(task) {
            status.send(s).await;
        }
    }
    if outcome.waiting_for_funds {
        status.send(SigningStatus::WaitingForFunds).await;
    }
    if let Some(txid) = &outcome.txid {
        info!("sign: transaction broadcast: {txid}");
        status.send(SigningStatus::TransactionSent(txid.clone())).await;
    }
}

/// The status events a task stands for, in the order the orchestrator used
/// to emit them while doing the same work.
fn sign_statuses_for(task: &SignTask) -> Vec<SigningStatus> {
    match task {
        SignTask::EnsureMailbox => vec![],
        SignTask::SendCommitments => vec![SigningStatus::SendingCommitment],
        SignTask::WaitCommitments => vec![SigningStatus::WaitingForCommitments],
        SignTask::MakeSigpackages => vec![SigningStatus::SendingSigningPackage],
        SignTask::WaitSigpackages => vec![SigningStatus::WaitingForSigningPackage],
        SignTask::SendSigshares => vec![
            SigningStatus::SendingSignatureShare,
            SigningStatus::SigningCompleted,
        ],
        SignTask::WaitSigshares => vec![SigningStatus::WaitingForSignatureShares],
        SignTask::Aggregate => vec![
            SigningStatus::PreparingTransaction,
            SigningStatus::SendingTransaction,
        ],
        SignTask::Done => vec![SigningStatus::SigningCompleted],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// A sink that remembers what it was told.
    #[derive(Clone, Default)]
    struct Recorder(Arc<Mutex<Vec<String>>>);

    impl Sink<SigningStatus> for Recorder {
        async fn send(&self, value: SigningStatus) {
            self.0.lock().unwrap().push(format!("{value:?}"));
        }
        async fn send_error(&self, error: anyhow::Error) {
            self.0.lock().unwrap().push(format!("error: {error}"));
        }
    }

    fn outcome(executed: Vec<SignTask>) -> SignStepOutcome {
        SignStepOutcome {
            executed,
            waiting_for_funds: false,
            txid: None,
        }
    }

    async fn events(outcome: SignStepOutcome) -> Vec<String> {
        let sink = Recorder::default();
        report_outcome(&outcome, &sink).await;
        let events = sink.0.lock().unwrap().clone();
        events
    }

    #[tokio::test]
    async fn a_participant_pass_reports_each_phase_in_order() {
        let e = events(outcome(vec![
            SignTask::EnsureMailbox,
            SignTask::SendCommitments,
            SignTask::WaitSigpackages,
        ]))
        .await;
        assert_eq!(e, ["SendingCommitment", "WaitingForSigningPackage"]);

        let e = events(outcome(vec![SignTask::SendSigshares, SignTask::Done])).await;
        assert_eq!(
            e,
            ["SendingSignatureShare", "SigningCompleted", "SigningCompleted"]
        );
    }

    #[tokio::test]
    async fn the_coordinator_sees_the_transaction_id_last() {
        let mut o = outcome(vec![SignTask::Aggregate]);
        o.txid = Some("ab".into());
        let e = events(o).await;
        assert_eq!(
            e,
            [
                "PreparingTransaction",
                "SendingTransaction",
                "TransactionSent(\"ab\")"
            ]
        );
    }

    #[tokio::test]
    async fn an_unfunded_publish_is_a_warning_not_a_phase() {
        let mut o = outcome(vec![SignTask::EnsureMailbox]);
        o.waiting_for_funds = true;
        assert_eq!(events(o).await, ["WaitingForFunds"]);
    }
}
