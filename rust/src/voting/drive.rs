//! zkool's assembly of the voting crate's round executor and driver.
//!
//! The crate's [`RoundExecutor`] owns the whole cast-vote lifecycle for one
//! bound round: from the persisted ballot intents it plans Delegate /
//! CastVote / AdvanceVote steps, proves, persists, and advances chain
//! submission through the host's transport. zkool supplies only transports,
//! keys, and per-dispatch context; this module builds those adapters and the
//! [`RoundDrive`] actor that drives the executor to quiescence and then
//! carries the round through helper-share confirmation.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::watch;

use anyhow::{ensure, Result};
use halo2_proofs::pasta::group::ff::PrimeField as _;
use rand::rngs::SysRng;
use rand_core::UnwrapErr;
use zcash_keys::keys::UnifiedSpendingKey;
use zcash_voting::prelude::{
    track_pending_shares, ShareTimingPolicy, ShareTrackingParams, ShareTrackingReport, VotingDb,
    VotingHotkey,
};
use zcash_voting::vote_work::{
    DelegationStepInputs, ProposalRosterEntry, RoundBinding, RoundExecutor, RoundHostContext,
};
use zcash_voting::{
    delegation_pipeline::{DelegationAccountIdentity, DelegationPipeline, DelegationSigner},
    ChainAdvancePolicy, ChainSubmissionClientConfig, HelperClient, HelperHealth, Network, PirFleet,
    RoundHostSource,
};
use zeroize::Zeroizing;
use zip32::AccountId;

use crate::voting::chain::{
    ZkoolChainTransport, ZkoolHelperTransport, ZkoolPirTransport, ZkoolTreeTransport,
};
use crate::Sink;

/// Transports and endpoints one executor run needs.
#[derive(Clone, Debug)]
pub struct ExecutorRoutes {
    /// zkool transport selector: 0 direct, 1 Tor, 2 Nym, 3 proxy.
    pub transport: u8,
    /// External proxy URL, used when `transport` is 3.
    pub proxy: String,
    /// Ordered vote-chain base URLs; POST failover and status lookup follow
    /// this order.
    pub chain_endpoints: Vec<String>,
}

/// Builds a round executor for `binding` over zkool's routed transports.
///
/// The executor freezes the wallet scope of `db` at construction, so the
/// returned executor cannot be retargeted at another wallet. Helper share
/// delivery rides the same routes as chain submission.
pub fn build_executor(
    db: Arc<VotingDb>,
    binding: RoundBinding,
    routes: &ExecutorRoutes,
) -> Result<RoundExecutor<ZkoolChainTransport>> {
    ensure!(
        !routes.chain_endpoints.is_empty(),
        "round driving requires at least one vote-chain endpoint"
    );
    let chain_transport = ZkoolChainTransport::new(routes.transport, &routes.proxy);
    let chain_config =
        ChainSubmissionClientConfig::for_network(binding.network, routes.chain_endpoints.clone());
    let helper_client = HelperClient::new(
        Arc::new(ZkoolHelperTransport::new(routes.transport, &routes.proxy)),
        HelperHealth::default(),
    );
    let tree_transport = Arc::new(ZkoolTreeTransport::new(routes.transport, &routes.proxy));
    let executor = RoundExecutor::with_transport(db, chain_transport, chain_config, helper_client)?
        .with_tree_transport(tree_transport)
        .with_binding(binding)?;
    Ok(executor)
}

/// Builds the immutable per-round binding: round, roster, and hotkey.
///
/// `proposals` are `(proposal_id, num_options)` pairs from the authenticated
/// round configuration. The hotkey is bound so the executor can reconstruct
/// the signer on the proving thread; without it the executor refuses to plan
/// `CastVote`.
pub fn build_binding(
    round_id: &str,
    network: Network,
    proposals: &[(u32, u32)],
    hotkey: Option<&VotingHotkey>,
) -> Result<RoundBinding> {
    ensure!(!proposals.is_empty(), "round binding requires a roster");
    Ok(RoundBinding {
        round_id: round_id.to_owned(),
        network,
        proposals: proposals
            .iter()
            .map(|(proposal_id, num_options)| ProposalRosterEntry {
                proposal_id: *proposal_id,
                num_options: *num_options,
            })
            .collect(),
        hotkey_secret: hotkey.map(|hotkey| Zeroizing::new(hotkey.stored_secret().to_vec())),
    })
}

/// Signs a delegation signing request with the wallet's own ZIP-32 seed.
///
/// Mirrors the fork's wallet-example signer: verifies the request fingerprint
/// against the seed, derives the account SpendAuth key, randomizes it with the
/// request alpha, and signs the PCZT sighash.
fn sign_delegation_request(
    seed: &[u8],
    request: zcash_voting::prelude::DelegationSigningRequest,
) -> Result<[u8; 64], zcash_voting::VotingError> {
    use zcash_voting::VotingError;
    let seed_fingerprint =
        zip32::fingerprint::SeedFingerprint::from_seed(seed).ok_or_else(|| {
            VotingError::InvalidInput {
                message: "wallet seed length is not valid for ZIP-32".to_string(),
            }
        })?;
    if seed_fingerprint.to_bytes() != request.seed_fingerprint {
        return Err(VotingError::InvalidInput {
            message: "wallet seed fingerprint does not match delegation signing request"
                .to_string(),
        });
    }
    let account =
        AccountId::try_from(request.account_index).map_err(|_| VotingError::InvalidInput {
            message: format!("invalid account_index {}", request.account_index),
        })?;
    let usk = UnifiedSpendingKey::from_seed(&request.network, seed, account).map_err(|error| {
        VotingError::InvalidInput {
            message: format!("account spending key derivation failed: {error}"),
        }
    })?;
    let sk = usk.orchard().clone();
    let ask = orchard::keys::SpendAuthorizingKey::from(&sk);
    let alpha = Option::<halo2_proofs::pasta::pallas::Scalar>::from(
        halo2_proofs::pasta::pallas::Scalar::from_repr(request.alpha),
    )
    .ok_or_else(|| VotingError::InvalidInput {
        message: "delegation alpha is not a valid Pallas scalar".to_string(),
    })?;
    let rsk = ask.randomize(&alpha);
    let sig = rsk.sign(UnwrapErr(SysRng), &request.sighash);
    Ok((&sig).into())
}

/// Everything the host needs to gather before building delegation inputs.
pub struct DelegationHostInputs {
    /// Lightwalletd-derived round inputs (anchor tree state, branch ids).
    pub lwd: zcash_voting::delegate::DelegationLwdInputs,
    /// Orchard FVK bytes and ZIP-32 identity of the note-owning account.
    pub identity: DelegationAccountIdentity,
    /// Notes and snapshot witnesses loaded from the wallet.
    pub note_source: crate::voting::note_source::ZkoolNoteSource,
    /// Authenticated round session JSON, when the config supplies one.
    pub session_json: Option<String>,
    /// Note bundling policy for the round's bundles.
    pub bundle_policy: zcash_voting::prelude::BundlePolicy,
    /// Ordered PIR server URLs from the authenticated config.
    pub pir_endpoints: Vec<String>,
    /// PIR layout from the authenticated config.
    pub pir_layout: zcash_voting::config::PirLayout,
    /// ZIP-32 seed bytes of the note-owning wallet.
    pub seed: Vec<u8>,
}

/// Assembles the delegation pipeline over zkool's note source and hotkey.
///
/// The concrete pipeline exposes the host-side setup stages (round row,
/// note selection, bundle layout persistence, eligibility preview) that the
/// object-safe [`DelegationDriver`] surface hides from the executor.
pub fn build_pipeline(
    db: Arc<VotingDb>,
    hotkey: &VotingHotkey,
    inputs: DelegationHostInputs,
) -> Result<DelegationPipeline<crate::voting::note_source::ZkoolNoteSource>> {
    let hotkey = VotingHotkey::from_stored_secret(hotkey.stored_secret(), hotkey.network())
        .map_err(|error| anyhow::anyhow!("voting hotkey reload failed: {error}"))?;
    Ok(DelegationPipeline::new(
        db,
        inputs.note_source,
        inputs.lwd,
        inputs.identity,
        Some(hotkey),
        inputs.bundle_policy,
        inputs.session_json.as_deref(),
    )?)
}

/// Assembles the per-dispatch delegation inputs: pipeline driver, software
/// signer, and PIR fleet over zkool's routed transport.
pub fn build_delegation_step_inputs(
    db: Arc<VotingDb>,
    hotkey: &VotingHotkey,
    mut inputs: DelegationHostInputs,
    routes: &ExecutorRoutes,
) -> Result<DelegationStepInputs> {
    ensure!(
        !inputs.pir_endpoints.is_empty(),
        "delegation requires at least one PIR endpoint"
    );
    let pir = PirFleet::new(
        &inputs.pir_endpoints,
        inputs.pir_layout,
        Arc::new(ZkoolPirTransport::new(routes.transport, &routes.proxy)),
    )?;
    let seed = std::mem::take(&mut inputs.seed);
    let driver = build_pipeline(db, hotkey, inputs)?;
    let signer = DelegationSigner::Software(Arc::new(move |request| {
        sign_delegation_request(&seed, request)
    }));
    Ok(DelegationStepInputs {
        driver: Arc::new(driver),
        signer,
        pir: Arc::new(pir),
    })
}

/// Per-dispatch host inputs for one round's driver run.
///
/// The voting crate reads the context once per dispatch so a long run plans
/// against the clock it actually runs under; the delegation inputs are fixed
/// per run and cloned into each context.
pub struct ZkoolRoundHost {
    helper_urls: Vec<String>,
    vote_tree_node_urls: Vec<String>,
    ceremony_start: Option<u64>,
    vote_end: Option<u64>,
    delegation: Option<DelegationStepInputs>,
}

impl ZkoolRoundHost {
    pub fn new(
        helper_urls: Vec<String>,
        vote_tree_node_urls: Vec<String>,
        ceremony_start: Option<u64>,
        vote_end: Option<u64>,
        delegation: Option<DelegationStepInputs>,
    ) -> Self {
        Self {
            helper_urls,
            vote_tree_node_urls,
            ceremony_start,
            vote_end,
            delegation,
        }
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl RoundHostSource for ZkoolRoundHost {
    fn host_context(&self) -> RoundHostContext {
        RoundHostContext {
            configured_helper_urls: self.helper_urls.clone(),
            now_seconds: now_seconds(),
            ceremony_start_seconds: self.ceremony_start,
            vote_end_time_seconds: self.vote_end,
            vote_tree_node_urls: self.vote_tree_node_urls.clone(),
            delegation: self.delegation.clone(),
            // Fresh submissions start status-only; persisted work always
            // starts with exact-tree recovery inside the client.
            chain_policy: ChainAdvancePolicy::default(),
            // Proofs run on a dedicated big-stack thread via the crate's
            // proving runtime; one at a time keeps memory bounded on phones.
            max_proof_concurrency: 1,
        }
    }
}

/// Where a run is in the round's lifecycle.
pub const PHASE_IDLE: &str = "idle";
/// Delegation, proving, chain submission: the ballot is being cast.
pub const PHASE_CASTING: &str = "casting";
/// The votes are on the chain; the helper shares are being confirmed.
pub const PHASE_CONFIRMING_SHARES: &str = "confirming_shares";
/// The run stopped; `quiescence` says why.
pub const PHASE_STOPPED: &str = "stopped";

/// A snapshot of one run, published on every change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveRunStatus {
    pub round_id: String,
    /// One of the `PHASE_*` labels.
    pub phase: String,
    /// False once the run reached quiescence or was cancelled.
    pub running: bool,
    /// Dispatched obligations so far, from the run report.
    pub dispatches: u64,
    /// Vote-work progress: completed and total proposal obligations.
    pub completed_proposals: u32,
    pub total_proposals: u32,
    /// Remaining bundle obligations the run can still execute.
    pub remaining_obligations: u32,
    /// Helper shares confirmed so far, and the total the round recorded.
    /// `total == 0` until the cast created them.
    pub shares_confirmed: u32,
    pub shares_total: u32,
    /// Why the run stopped, formatted for display and logs.
    pub quiescence: Option<String>,
    /// Failure messages in dispatch order.
    pub failures: Vec<String>,
}

impl DriveRunStatus {
    fn new(round_id: &str, phase: &str, running: bool) -> Self {
        Self {
            round_id: round_id.to_owned(),
            phase: phase.to_owned(),
            running,
            dispatches: 0,
            completed_proposals: 0,
            total_proposals: 0,
            remaining_obligations: 0,
            shares_confirmed: 0,
            shares_total: 0,
            quiescence: None,
            failures: Vec::new(),
        }
    }

    /// The standing of a round with no run.
    pub fn idle(round_id: &str) -> Self {
        Self::new(round_id, PHASE_IDLE, false)
    }

    fn stop(&mut self, quiescence: String) {
        self.running = false;
        self.phase = PHASE_STOPPED.to_owned();
        self.quiescence = Some(quiescence);
    }
}

/// The live status of one run: a watch channel whose sender the run's task
/// and reporter update, and whose receivers the status forwarder and the
/// pollers read.
type StatusSender = Arc<watch::Sender<DriveRunStatus>>;

/// Records [`RoundDriveEvent`]s into the run status.
///
/// Called from several concurrent bundle tasks; `send_modify` serializes the
/// updates and never blocks on I/O.
struct DriveRunRecorder {
    status: StatusSender,
}

impl zcash_voting::RoundDriveReporter for DriveRunRecorder {
    fn report(&self, event: zcash_voting::RoundDriveEvent) {
        self.status.send_modify(|status| match event {
            zcash_voting::RoundDriveEvent::PlanRefreshed { tally, .. } => {
                status.completed_proposals = tally.completed_proposals;
                status.total_proposals = tally.total_proposals;
                status.remaining_obligations = tally.remaining_obligations;
            }
            zcash_voting::RoundDriveEvent::StepFinished { .. } => {
                status.dispatches += 1;
            }
            zcash_voting::RoundDriveEvent::StepFailed { kind, message, .. } => {
                status.failures.push(format!("{kind:?}: {message}"));
            }
            _ => {}
        });
    }
}

/// Formats the stop reason for display.
fn quiescence_label(quiescence: &zcash_voting::RoundQuiescence) -> String {
    use zcash_voting::RoundQuiescence as Q;
    match quiescence {
        Q::NoWorkLeft => "done".to_string(),
        Q::NeedsBundleSetup => "needs_bundle_setup".to_string(),
        Q::PersistedChainTerminal => "chain_terminal".to_string(),
        Q::NeedsBallot { open_proposals, .. } => {
            format!("needs_ballot({} open)", open_proposals.len())
        }
        Q::NeedsDelegationSignatures { bundles } => {
            format!("needs_delegation_signatures({bundles:?})")
        }
        Q::BackgroundShareWorkOnly { shares } => {
            format!("background_shares({})", shares.len())
        }
        Q::Cancelled => "cancelled".to_string(),
        Q::ChainTerminal { .. } => "chain_terminal".to_string(),
        Q::ChainRecoveryStalled { .. } => "chain_recovery_stalled".to_string(),
        Q::Failures => "failures".to_string(),
        Q::PassBudgetExhausted { remaining } => {
            format!("pass_budget_exhausted({} remaining)", remaining.len())
        }
        _ => format!("{quiescence:?}"),
    }
}

/// Operation epochs for the SDK's round locks: a run queued behind a
/// cancelled predecessor stops waiting instead of holding its place.
static NEXT_DRIVE_GENERATION: AtomicU64 = AtomicU64::new(1);

/// One run of one round: the task that casts the ballot (delegation,
/// proving, chain submission) and then confirms the helper shares until
/// every share is confirmed, the vote end passes, the rest cannot be
/// recovered, or the run is cancelled.
///
/// The host that starts the run owns it, like the mempool and FROST
/// drivers: it holds the handle for as long as the run should live, cancels
/// it on backgrounding or wallet switch, and is the one place that decides
/// whether a round already has a run. Dropping the handle cancels the run.
/// Its durable queue is the voting sidecar, so a run started later, in the
/// same process or after a restart, simply discovers the unfinished work.
pub struct RoundDrive {
    control: zcash_voting::ChainSubmissionControl,
    status: watch::Receiver<DriveRunStatus>,
}

impl RoundDrive {
    /// Starts the run for `binding`. Every status change is sent to `sink`,
    /// beginning with the casting snapshot and ending with one whose
    /// `running` is false; `status()` always has the latest.
    pub fn start<S>(
        db: Arc<VotingDb>,
        binding: RoundBinding,
        routes: ExecutorRoutes,
        host: ZkoolRoundHost,
        sink: S,
    ) -> Self
    where
        S: Sink<DriveRunStatus> + Send + Sync + 'static,
    {
        let generation = NEXT_DRIVE_GENERATION.fetch_add(1, Ordering::Relaxed);
        let control = zcash_voting::ChainSubmissionControl::new(generation);
        let (sender, receiver) =
            watch::channel(DriveRunStatus::new(&binding.round_id, PHASE_CASTING, true));
        let sender = Arc::new(sender);

        // The forwarder turns the coalesced watch updates into sink sends and
        // leaves once it has forwarded the final snapshot.
        let mut updates = receiver.clone();
        tokio::spawn(async move {
            loop {
                let snapshot = updates.borrow_and_update().clone();
                let running = snapshot.running;
                sink.send(snapshot).await;
                if !running || updates.changed().await.is_err() {
                    break;
                }
            }
        });

        let task_control = control.clone();
        tokio::spawn(async move {
            let outcome = run_round(db, binding, &routes, host, &task_control, &sender).await;
            sender.send_modify(|current| current.stop(outcome));
        });
        Self {
            control,
            status: receiver,
        }
    }

    /// Cancels the run cooperatively: the SDK wakes retry waits rather than
    /// leaving the task parked until its next pass, and the final snapshot
    /// follows once the step in flight ends. Durable effects stay.
    pub fn cancel(&self) {
        self.control.cancel();
    }

    /// The latest snapshot.
    pub fn status(&self) -> DriveRunStatus {
        self.status.borrow().clone()
    }

    pub fn is_running(&self) -> bool {
        self.status.borrow().running
    }
}

impl Drop for RoundDrive {
    fn drop(&mut self) {
        self.control.cancel();
    }
}

/// Casts the ballot, then confirms the helper shares. Returns the final
/// quiescence label.
async fn run_round(
    db: Arc<VotingDb>,
    binding: RoundBinding,
    routes: &ExecutorRoutes,
    host: ZkoolRoundHost,
    control: &zcash_voting::ChainSubmissionControl,
    status: &StatusSender,
) -> String {
    let round_id = binding.round_id.clone();
    let executor = match build_executor(Arc::clone(&db), binding, routes) {
        Ok(executor) => executor,
        Err(error) => {
            status.send_modify(|status| {
                status
                    .failures
                    .push(format!("executor construction failed: {error}"));
            });
            return "failures".to_owned();
        }
    };
    let recorder = DriveRunRecorder {
        status: Arc::clone(status),
    };
    let report = zcash_voting::RoundDriver::new(&executor)
        .run(&host, control, &recorder)
        .await;
    let label = quiescence_label(&report.quiescence);
    status.send_modify(|status| {
        status.completed_proposals = report.tally.completed_proposals;
        status.total_proposals = report.tally.total_proposals;
        status.remaining_obligations = report.tally.remaining_obligations;
        if matches!(report.quiescence, zcash_voting::RoundQuiescence::Failures) {
            for failure in &report.failures {
                status.failures.push(format!(
                    "{:?}: {}",
                    failure.failure.kind, failure.failure.message
                ));
            }
        }
    });
    // The executor is done once the votes are cast; the remaining work is
    // the helper shares it handed to the background.
    let cast = matches!(
        report.quiescence,
        zcash_voting::RoundQuiescence::NoWorkLeft
            | zcash_voting::RoundQuiescence::BackgroundShareWorkOnly { .. }
    );
    if !cast {
        return label;
    }
    let (confirmed, total) = match share_counts(Arc::clone(&db), &round_id).await {
        Ok(counts) => counts,
        Err(error) => {
            status.send_modify(|status| {
                status
                    .failures
                    .push(format!("share lookup failed: {error}"));
            });
            return "failures".to_owned();
        }
    };
    status.send_modify(|status| {
        status.shares_confirmed = confirmed;
        status.shares_total = total;
        status.phase = PHASE_CONFIRMING_SHARES.to_owned();
    });
    if confirmed == total {
        return "done".to_owned();
    }
    let client = HelperClient::new(
        Arc::new(ZkoolHelperTransport::new(routes.transport, &routes.proxy)),
        HelperHealth::default(),
    );
    let cancelled = move || control.is_cancelled();
    let outcome = track_shares_until_done(
        Arc::clone(&db),
        &client,
        &round_id,
        &host.helper_urls,
        host.vote_end,
        &cancelled,
        |_report, unconfirmed| {
            status.send_modify(|status| {
                status.shares_confirmed = total.saturating_sub(unconfirmed);
            });
        },
    )
    .await;
    match outcome {
        TrackingOutcome::Done => "done".to_owned(),
        TrackingOutcome::RoundClosed { unconfirmed } => format!("round_closed({unconfirmed})"),
        TrackingOutcome::Unrecoverable { count } => format!("unrecoverable({count})"),
        TrackingOutcome::Cancelled => "cancelled".to_owned(),
    }
}

/// `(confirmed, total)` helper shares of a round, from the sidecar.
pub async fn share_counts(db: Arc<VotingDb>, round_id: &str) -> Result<(u32, u32)> {
    let round = round_id.to_owned();
    tokio::task::spawn_blocking(move || -> Result<(u32, u32)> {
        let total = u32::try_from(db.get_share_delegations(&round)?.len()).unwrap_or(u32::MAX);
        let unconfirmed =
            u32::try_from(db.get_unconfirmed_delegations(&round)?.len()).unwrap_or(u32::MAX);
        Ok((total.saturating_sub(unconfirmed), total))
    })
    .await?
}

// ---------------------------------------------------------------------------
// Helper-share confirmation, one durable pass at a time.
//
// A pass walks the round's unconfirmed shares, asks the helpers that hold
// them for confirmation, and resubmits the overdue ones; every pass is
// planned from the durable sidecar, so a restart simply runs the next one.

/// One pass walks every unconfirmed share, and a stalled helper can hold it
/// for the per-share poll budget. The whole pass is bounded so a caller's
/// loop is never wedged; a cancelled pass keeps whatever it confirmed.
const PASS_BUDGET: Duration = Duration::from_secs(45);

/// Wait after a failed pass, which carries no `next_delay_seconds`; doubled
/// on every further failure in a row, up to `MAX_RETRY_DELAY`.
const RETRY_DELAY: Duration = Duration::from_secs(30);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(5 * 60);

/// Why [`track_shares_until_done`] stopped.
#[derive(Clone, Debug, PartialEq, Eq)]
enum TrackingOutcome {
    /// Every share is confirmed.
    Done,
    /// The round's vote end passed with shares still unconfirmed.
    RoundClosed { unconfirmed: u32 },
    /// Nothing is pending, but these shares cannot be repaired by retrying.
    Unrecoverable { count: u32 },
    /// The caller cancelled.
    Cancelled,
}

/// Runs exactly one helper-share tracking pass for a round against `db`.
///
/// The report's `next_delay_seconds` tells the caller when to run another
/// pass, or that nothing is pending.
async fn track_shares_pass(
    db: &VotingDb,
    client: &HelperClient,
    round_id: &str,
    helper_urls: &[String],
    vote_end_time_seconds: Option<u64>,
) -> Result<ShareTrackingReport> {
    let params = ShareTrackingParams {
        round_id,
        configured_server_urls: helper_urls,
        now_seconds: now_seconds(),
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
    Ok(track_pending_shares(db, &params, client, &|| cancelled.load(Ordering::Relaxed)).await?)
}

/// The shares of `round_id` not yet confirmed, read on a blocking thread.
async fn unconfirmed_count(db: Arc<VotingDb>, round_id: &str) -> Result<u32> {
    let round = round_id.to_owned();
    let count = tokio::task::spawn_blocking(move || db.get_unconfirmed_delegations(&round))
        .await??
        .len();
    Ok(u32::try_from(count).unwrap_or(u32::MAX))
}

/// Sleeps `delay` in one-second slices, returning early once `cancelled`.
async fn sleep_unless_cancelled(delay: Duration, cancelled: &(dyn Fn() -> bool + Send + Sync)) {
    let deadline = tokio::time::Instant::now() + delay;
    while tokio::time::Instant::now() < deadline && !cancelled() {
        let left = deadline.saturating_duration_since(tokio::time::Instant::now());
        tokio::time::sleep(left.min(Duration::from_secs(1))).await;
    }
}

/// Runs tracking passes for a round until every share is confirmed, the
/// vote end passes, the remaining shares cannot be recovered, or `cancelled`.
///
/// A pass is run as soon as the SDK's timing says a share is due, and a
/// failed pass (helpers unreachable) is retried with a growing delay for as
/// long as the round is open: the host asked for the vote to be carried to
/// confirmation, not for a report of a flaky network. `on_pass` runs after
/// every pass with its report and the shares still unconfirmed, for the
/// host's progress display.
async fn track_shares_until_done(
    db: Arc<VotingDb>,
    client: &HelperClient,
    round_id: &str,
    helper_urls: &[String],
    vote_end_time_seconds: Option<u64>,
    cancelled: &(dyn Fn() -> bool + Send + Sync),
    mut on_pass: impl FnMut(&ShareTrackingReport, u32),
) -> TrackingOutcome {
    let closed = |unconfirmed: u32| TrackingOutcome::RoundClosed { unconfirmed };
    let mut retry_delay = RETRY_DELAY;
    loop {
        if cancelled() {
            return TrackingOutcome::Cancelled;
        }
        if vote_end_time_seconds.is_some_and(|end| now_seconds() >= end) {
            tracing::info!("voting share tracking for {round_id} stopped: vote end reached");
            return closed(
                unconfirmed_count(Arc::clone(&db), round_id)
                    .await
                    .unwrap_or(0),
            );
        }
        let pass =
            track_shares_pass(&db, client, round_id, helper_urls, vote_end_time_seconds).await;
        let delay = match pass {
            Ok(report) => {
                retry_delay = RETRY_DELAY;
                let unconfirmed = unconfirmed_count(Arc::clone(&db), round_id)
                    .await
                    .unwrap_or_else(|_| {
                        report
                            .unconfirmed_at_entry
                            .unwrap_or(u32::MAX)
                            .saturating_sub(u32::try_from(report.confirmed.len()).unwrap_or(0))
                    });
                on_pass(&report, unconfirmed);
                if cancelled() {
                    return TrackingOutcome::Cancelled;
                }
                if unconfirmed == 0 {
                    return TrackingOutcome::Done;
                }
                match report.next_delay_seconds {
                    Some(seconds) => Duration::from_secs(seconds.max(1)),
                    None if !report.unrecoverable.is_empty() => {
                        tracing::warn!(
                            "voting share tracking for {round_id}: {} share(s) cannot be recovered",
                            report.unrecoverable.len()
                        );
                        return TrackingOutcome::Unrecoverable {
                            count: u32::try_from(report.unrecoverable.len()).unwrap_or(u32::MAX),
                        };
                    }
                    None => return TrackingOutcome::Done,
                }
            }
            Err(error) => {
                tracing::warn!(
                    "voting share tracking pass for {round_id} failed, next try in {}s: {error:#}",
                    retry_delay.as_secs()
                );
                let delay = retry_delay;
                retry_delay = (retry_delay * 2).min(MAX_RETRY_DELAY);
                delay
            }
        };
        sleep_unless_cancelled(delay, cancelled).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_rejects_an_empty_roster() {
        assert!(build_binding(&"01".repeat(32), Network::Regtest, &[], None).is_err());
    }

    #[test]
    fn binding_maps_the_roster_and_hotkey() {
        let hotkey =
            zcash_voting::prelude::generate_random_voting_hotkey(Network::Regtest).expect("hotkey");
        let binding = build_binding(
            &"01".repeat(32),
            Network::Regtest,
            &[(1, 2), (3, 4)],
            Some(&hotkey),
        )
        .expect("binding");
        assert_eq!(binding.proposals.len(), 2);
        assert_eq!(binding.proposals[1].num_options, 4);
        assert_eq!(
            binding.hotkey_secret.expect("secret").as_slice(),
            hotkey.stored_secret()
        );
    }

    #[test]
    fn signer_rejects_a_foreign_fingerprint_and_signs_its_own() {
        let seed = vec![7u8; 32];
        let fingerprint = zip32::fingerprint::SeedFingerprint::from_seed(&seed)
            .expect("fingerprint")
            .to_bytes();
        let request =
            |seed_fingerprint: [u8; 32]| zcash_voting::prelude::DelegationSigningRequest {
                account_index: 0,
                network: Network::Regtest,
                seed_fingerprint,
                sighash: [9u8; 32],
                alpha: [
                    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ],
            };
        assert!(sign_delegation_request(&seed, request([0u8; 32])).is_err());
        let signature = sign_delegation_request(&seed, request(fingerprint)).expect("signs");
        assert_eq!(signature.len(), 64);
        assert!(signature.iter().any(|&byte| byte != 0));
    }
}
