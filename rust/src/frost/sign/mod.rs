mod exec;
mod plan;
mod pubkey;
mod state;
mod step;
mod task;

pub use step::{sign_step, SignStepOutcome};

pub(super) const COMMITMENT_PREFIX: &[u8] = b"CMT6";
/// One signer's part of an input's signing package. The coordinator sends
/// one memo per (input, signer) rather than the whole package, so that no
/// memo grows with the threshold: SPK5 carried the whole package and ran
/// out of room past three signers.
pub(super) const SIGPACKAGE_PREFIX: &[u8] = b"SPK6";
pub(super) const SIGSHARE_PREFIX: &[u8] = b"SSH3";
/// The coordinator's mailbox address for the session, broadcast alongside
/// the signing packages so that members need no stored address for it.
pub(super) const COORDINATOR_PREFIX: &[u8] = b"CRD1";

// ── Public API (unchanged surface for flutter / graphql callers) ──────────────

use anyhow::{anyhow, bail, Context as _, Result};
use bincode::config;
use pczt::{roles::verifier::Verifier, Pczt};
use sqlx::SqliteConnection;
use tracing::info;

use crate::{
    api::{
        coin::Network,
        frost::{FrostSignParams, SigningStatus},
        pay::PcztPackage,
    },
    account::get_orchard_vk,
    frost::dkg::delete_frost_state,
    Client, Sink,
};

use task::SignTask;


/// Checks that a transaction to sign is this group's. Every value-carrying
/// shielded spend of a PCZT carries the full viewing key of the note it
/// spends (a dummy spend has a random key and no value), so a file prepared
/// for another account is one whose spends carry another key. Caught here,
/// with a clear message, rather than minutes into a session that cannot
/// sign.
pub async fn check_group_pczt(
    connection: &mut SqliteConnection,
    account: u32,
    pczt: &PcztPackage,
) -> Result<()> {
    let ours = get_orchard_vk(&mut *connection, account)
        .await?
        .context("this account has no Orchard viewing key")?
        .to_bytes();
    let parsed = Pczt::parse(&pczt.pczt).map_err(|e| anyhow!("invalid transaction: {e:?}"))?;
    let mut spends = 0;
    let mut foreign = false;
    let mut check = |bundle: &orchard::pczt::Bundle| {
        for action in bundle.actions() {
            let spend = action.spend();
            if spend.value().is_none_or(|v| v.inner() == 0) {
                continue;
            }
            spends += 1;
            if spend.fvk().as_ref().map(|fvk| fvk.to_bytes()) != Some(ours) {
                foreign = true;
            }
        }
        Ok::<_, pczt::roles::verifier::OrchardError<()>>(())
    };
    Verifier::new(parsed)
        .with_orchard(&mut check)
        .map_err(|e| anyhow!("invalid Orchard bundle: {e:?}"))?
        .with_ironwood(&mut check)
        .map_err(|e| anyhow!("invalid Ironwood bundle: {e:?}"))?;
    if foreign {
        bail!("This transaction does not spend from this group: it was prepared for another account");
    }
    if spends == 0 {
        bail!("This transaction spends nothing from the group's shielded notes");
    }
    Ok(())
}

/// Starts the wallet's signing session. There is one session at a time:
/// a session already stored is an error, not a silent no-op, so a caller
/// whose PCZT was not taken learns it (#1546). The session's three props
/// are written in one transaction, so a crash cannot leave a PCZT without
/// its params.
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
    check_no_session(&mut *connection).await?;
    check_no_dkg(&mut *connection).await?;
    if !crate::frost::is_group_account(&mut *connection, account).await? {
        bail!("Account {account} is not a FROST group");
    }
    let n = crate::frost::dkg::get_dkg_params(&mut *connection, account).await?.n;
    if !(1..=n).contains(&coordinator) {
        bail!("The coordinator id must be between 1 and {n}, got {coordinator}");
    }
    check_group_pczt(&mut *connection, account, pczt).await?;
    let pczt_bytes = bincode::encode_to_vec(pczt, config::legacy()).unwrap();
    let params = FrostSignParams {
        account,
        coordinator,
        funding_account,
    };
    let params_json = serde_json::to_string(&params).unwrap();
    let mut tx = crate::db::begin_immediate(&mut *connection).await?;
    sqlx::query("INSERT INTO props(key, value) VALUES ('frost_pczt', ?)")
        .bind(&pczt_bytes)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO props(key, value) VALUES ('frost_sign_params', ?)")
        .bind(&params_json)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO props(key, value) VALUES ('dkg_account', ?)")
        .bind(funding_account)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

/// Refuses to start a signing session while one is stored.
async fn check_no_session(connection: &mut SqliteConnection) -> Result<()> {
    if is_signing_in_progress(&mut *connection).await? {
        bail!("A signing session is in progress; finish or cancel it first");
    }
    Ok(())
}

/// Refuses to start a signing session while a DKG runs. One protocol at
/// a time: the driver syncs one funding account per pass, and the
/// teardown of either protocol deletes the state of both (#1549).
async fn check_no_dkg(connection: &mut SqliteConnection) -> Result<()> {
    if crate::frost::dkg::dkg_in_progress(&mut *connection).await? {
        bail!("A DKG is in progress; finish or cancel it first");
    }
    Ok(())
}

pub async fn reset_sign(connection: &mut SqliteConnection) -> Result<()> {
    delete_frost_state(&mut *connection).await
}

/// The account paying for the messages of the signing session in
/// progress, if any.
pub async fn signing_funding_account(connection: &mut SqliteConnection) -> Result<Option<u32>> {
    if !is_signing_in_progress(&mut *connection).await? {
        return Ok(None);
    }
    Ok(Some(state::load_sign_params(&mut *connection).await?.funding_account))
}

/// The group account of the signing session in progress, if any.
pub async fn signing_account(connection: &mut SqliteConnection) -> Result<Option<u32>> {
    if !is_signing_in_progress(&mut *connection).await? {
        return Ok(None);
    }
    Ok(Some(state::load_sign_params(&mut *connection).await?.account))
}

/// A signing session is stored. The PCZT is the session: `init_sign`
/// writes it with the params in one transaction and `delete_frost_state`
/// removes every `frost_%` prop together, so this is the one predicate.
pub async fn is_signing_in_progress(connection: &mut SqliteConnection) -> Result<bool> {
    let exists =
        sqlx::query_as::<_, (bool,)>("SELECT TRUE FROM props WHERE key = 'frost_pczt'")
            .fetch_optional(&mut *connection)
            .await?;
    Ok(exists.is_some())
}

pub async fn in_sign(connection: &mut SqliteConnection) -> Result<bool> {
    is_signing_in_progress(connection).await
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
    stopping: &(dyn Fn() -> bool + Send + Sync),
) -> Result<()> {
    info!("sign: starting at height {}", height);

    // The transaction being signed carries an expiry height; past it the
    // network drops it, so the session is worthless. Say so, once, instead
    // of letting the final broadcast fail.
    let expiry = match session_expiry(connection).await? {
        Some(expiry) => expiry,
        None => {
            info!("sign: no signing session in progress");
            return Ok(());
        }
    };
    if expiry != 0 && height >= expiry {
        info!("sign: the transaction expired at {expiry}, tip is {height}; ending the session");
        delete_frost_state(&mut *connection).await?;
        status.send(SigningStatus::Expired).await;
        return Ok(());
    }

    let Some(outcome) = sign_step(network, connection, client, height, stopping).await? else {
        info!("sign: no signing session in progress");
        return Ok(());
    };
    report_outcome(&outcome, &status).await;
    let ended = outcome.txid.is_some() || outcome.executed.last() == Some(&SignTask::Done);
    if expiry != 0 && !ended && expiry - height <= EXPIRY_WARNING_BLOCKS {
        status
            .send(SigningStatus::ExpiringSoon {
                blocks_left: expiry - height,
            })
            .await;
    }
    Ok(())
}

/// Warn when the transaction being signed has this many blocks left.
pub const EXPIRY_WARNING_BLOCKS: u32 = 10;

/// The expiry height of the transaction being signed, 0 when it has none,
/// None when no session is in progress.
async fn session_expiry(connection: &mut SqliteConnection) -> Result<Option<u32>> {
    if !is_signing_in_progress(&mut *connection).await? {
        return Ok(None);
    }
    let pkg = state::load_pczt_pkg(&mut *connection).await?;
    let parsed = Pczt::parse(&pkg.pczt).map_err(|e| anyhow!("invalid transaction: {e:?}"))?;
    Ok(Some(*parsed.global().expiry_height()))
}

/// Tell the status sink what a pass did, in order.
async fn report_outcome(outcome: &SignStepOutcome, status: &impl Sink<SigningStatus>) {
    for task in &outcome.executed {
        for s in sign_statuses_for(task) {
            status.send(s).await;
        }
    }
    if let Some((what, have, of)) = &outcome.progress {
        status
            .send(SigningStatus::WaitProgress {
                what: what.clone(),
                have: *have,
                of: *of,
            })
            .await;
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
        SignTask::AnnounceCoordinator => vec![SigningStatus::AnnouncingCoordinator],
        SignTask::WaitCoordinator => vec![SigningStatus::WaitingForCoordinator],
        SignTask::WaitOwnPublish => vec![SigningStatus::WaitOwnPublish],
        SignTask::SendCommitments => vec![SigningStatus::SendingCommitment],
        SignTask::WaitCommitments => vec![SigningStatus::WaitingForCommitments],
        SignTask::MakeSigpackages => vec![SigningStatus::SendingSigningPackage],
        SignTask::WaitSigpackages => vec![SigningStatus::WaitingForSigningPackage],
        SignTask::SendSigshares => vec![SigningStatus::SendingSignatureShare],
        SignTask::WaitSigshares => vec![SigningStatus::WaitingForSignatureShares],
        SignTask::Aggregate => vec![
            SigningStatus::PreparingTransaction,
            SigningStatus::SendingTransaction,
        ],
        SignTask::Done => vec![SigningStatus::SigningCompleted],
    }
}

#[cfg(test)]
mod guard_tests {
    use super::*;
    use crate::db::create_schema;
    use sqlx::Connection;

    async fn wallet() -> SqliteConnection {
        let mut db = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        create_schema(&mut db).await.unwrap();
        db
    }

    fn pczt() -> PcztPackage {
        PcztPackage {
            pczt: vec![],
            n_spends: [0; 4],
            sapling_indices: vec![],
            orchard_indices: vec![],
            ironwood_indices: vec![],
            can_sign: true,
            can_broadcast: true,
            price: None,
            category: None,
            is_issuance: false,
        }
    }

    async fn message(result: Result<()>) -> String {
        result.unwrap_err().to_string()
    }

    /// #1546: a second session is refused, and the caller is told why.
    #[tokio::test]
    async fn a_second_signing_session_is_an_error() {
        let mut db = wallet().await;
        sqlx::query("INSERT INTO props(key, value) VALUES ('frost_pczt', x'00')")
            .execute(&mut db)
            .await
            .unwrap();
        let m = message(init_sign(&mut db, 1, 2, 1, &pczt()).await).await;
        assert!(m.contains("signing session is in progress"), "{m}");
        assert!(in_sign(&mut db).await.unwrap());
    }

    /// #1531: only a FROST group account can have a signing session.
    #[tokio::test]
    async fn a_session_for_an_ordinary_account_is_an_error() {
        let mut db = wallet().await;
        let m = message(init_sign(&mut db, 1, 2, 1, &pczt()).await).await;
        assert!(m.contains("not a FROST group"), "{m}");
        assert!(!in_sign(&mut db).await.unwrap());
    }

    /// A group account of three, as the DKG leaves it.
    async fn group(db: &mut SqliteConnection, account: u32) {
        sqlx::query("INSERT INTO dkg_state(account, key_pkg) VALUES (?, x'00')")
            .bind(account)
            .execute(&mut *db)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO dkg_params(account, id, n, t, seed, birth_height, name)
             VALUES (?, 1, 3, 2, '', 0, 'group')",
        )
        .bind(account)
        .execute(&mut *db)
        .await
        .unwrap();
    }

    /// #1552: the coordinator is one of the group's participants.
    #[tokio::test]
    async fn a_coordinator_outside_the_group_is_an_error() {
        let mut db = wallet().await;
        group(&mut db, 1).await;
        for coordinator in [0, 4] {
            let m = message(init_sign(&mut db, 1, 2, coordinator, &pczt()).await).await;
            assert!(m.contains("coordinator id must be between 1 and 3"), "{m}");
        }
        assert!(!in_sign(&mut db).await.unwrap());
    }

    /// #1549: a DKG and a signing session never run together.
    #[tokio::test]
    async fn a_signing_session_cannot_start_during_a_dkg() {
        let mut db = wallet().await;
        sqlx::query("INSERT INTO props(key, value) VALUES ('dkg_account', '2')")
            .execute(&mut db)
            .await
            .unwrap();
        let m = message(init_sign(&mut db, 1, 2, 1, &pczt()).await).await;
        assert!(m.contains("DKG is in progress"), "{m}");
        assert!(!in_sign(&mut db).await.unwrap());
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
            progress: None,
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

        // The share goes out in one pass; the session ends, and the signer
        // is told so, in the pass that finds it mined.
        let e = events(outcome(vec![SignTask::SendSigshares])).await;
        assert_eq!(e, ["SendingSignatureShare"]);
        let e = events(outcome(vec![SignTask::Done])).await;
        assert_eq!(e, ["SigningCompleted"]);
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
