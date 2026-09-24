//! zkool's assembly of the voting crate's round executor and driver.
//!
//! The crate's [`RoundExecutor`] owns the whole cast-vote lifecycle for one
//! bound round: from the persisted ballot intents it plans Delegate /
//! CastVote / AdvanceVote steps, proves, persists, and advances chain
//! submission through the host's transport. zkool supplies only transports,
//! keys, and per-dispatch context; this module builds those adapters and the
//! process-local run registry that drives the executor to quiescence.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{ensure, Result};
use halo2_proofs::pasta::group::ff::PrimeField as _;
use rand_core::OsRng;
use zeroize::Zeroizing;
use zcash_keys::keys::UnifiedSpendingKey;
use zip32::AccountId;
use zcash_voting::prelude::{VotingDb, VotingHotkey};
use zcash_voting::vote_work::{
    DelegationStepInputs, ProposalRosterEntry, RoundBinding, RoundExecutor, RoundHostContext,
};
use zcash_voting::{
    delegation_pipeline::{
        DelegationAccountIdentity, DelegationPipeline, DelegationSigner,
    },
    ChainAdvancePolicy, ChainSubmissionClientConfig, HelperClient, HelperHealth, Network,
    PirFleet, RoundHostSource,
};

use crate::voting::chain::{ZkoolChainTransport, ZkoolPirTransport, ZkoolTreeTransport};
use crate::voting::sidecar::ZkoolHelperTransport;

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
        Arc::new(ZkoolHelperTransport::new(
            routes.transport,
            &routes.proxy,
        )),
        HelperHealth::default(),
    );
    let tree_transport = Arc::new(ZkoolTreeTransport::new(
        routes.transport,
        &routes.proxy,
    ));
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
    let seed_fingerprint = zip32::fingerprint::SeedFingerprint::from_seed(seed).ok_or_else(
        || VotingError::InvalidInput {
            message: "wallet seed length is not valid for ZIP-32".to_string(),
        },
    )?;
    if seed_fingerprint.to_bytes() != request.seed_fingerprint {
        return Err(VotingError::InvalidInput {
            message: "wallet seed fingerprint does not match delegation signing request"
                .to_string(),
        });
    }
    let account = AccountId::try_from(request.account_index).map_err(|_| {
        VotingError::InvalidInput {
            message: format!("invalid account_index {}", request.account_index),
        }
    })?;
    let usk = UnifiedSpendingKey::from_seed(&request.network, seed, account).map_err(
        |error| VotingError::InvalidInput {
            message: format!("account spending key derivation failed: {error}"),
        },
    )?;
    let sk = *usk.orchard();
    let ask = orchard::keys::SpendAuthorizingKey::from(&sk);
    let alpha = Option::<halo2_proofs::pasta::pallas::Scalar>::from(
        halo2_proofs::pasta::pallas::Scalar::from_repr(request.alpha),
    )
    .ok_or_else(|| VotingError::InvalidInput {
        message: "delegation alpha is not a valid Pallas scalar".to_string(),
    })?;
    let rsk = ask.randomize(&alpha);
    let sig = rsk.sign(OsRng, &request.sighash);
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

impl RoundHostSource for ZkoolRoundHost {
    fn host_context(&self) -> RoundHostContext {
        RoundHostContext {
            configured_helper_urls: self.helper_urls.clone(),
            now_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
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

/// A snapshot of one driver run for the FRB layer to poll.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveRunStatus {
    pub round_id: String,
    /// False once the run reached quiescence or was cancelled.
    pub running: bool,
    /// Dispatched obligations so far, from the run report.
    pub dispatches: u64,
    /// Vote-work progress: completed and total proposal obligations.
    pub completed_proposals: u32,
    pub total_proposals: u32,
    /// Remaining bundle obligations the run can still execute.
    pub remaining_obligations: u32,
    /// Why the run stopped, formatted for display and logs.
    pub quiescence: Option<String>,
    /// Failure messages in dispatch order.
    pub failures: Vec<String>,
}

impl DriveRunStatus {
    fn idle(round_id: &str) -> Self {
        Self {
            round_id: round_id.to_owned(),
            running: false,
            dispatches: 0,
            completed_proposals: 0,
            total_proposals: 0,
            remaining_obligations: 0,
            quiescence: None,
            failures: Vec::new(),
        }
    }
}

/// Mutable per-run state the reporter records and status reads.
#[derive(Default)]
struct DriveRunState {
    running: bool,
    dispatches: u64,
    completed_proposals: u32,
    total_proposals: u32,
    remaining_obligations: u32,
    quiescence: Option<String>,
    failures: Vec<String>,
}

impl DriveRunState {
    fn snapshot(&self, round_id: &str) -> DriveRunStatus {
        DriveRunStatus {
            round_id: round_id.to_owned(),
            running: self.running,
            dispatches: self.dispatches,
            completed_proposals: self.completed_proposals,
            total_proposals: self.total_proposals,
            remaining_obligations: self.remaining_obligations,
            quiescence: self.quiescence.clone(),
            failures: self.failures.clone(),
        }
    }
}

/// Records [`RoundDriveEvent`]s into the run state.
///
/// Called from several concurrent bundle tasks, so it is Mutex-guarded and
/// never blocks on I/O.
struct DriveRunRecorder {
    state: Arc<Mutex<DriveRunState>>,
}

impl zcash_voting::RoundDriveReporter for DriveRunRecorder {
    fn report(&self, event: zcash_voting::RoundDriveEvent) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        match event {
            zcash_voting::RoundDriveEvent::PlanRefreshed { tally, .. } => {
                state.completed_proposals = tally.completed_proposals;
                state.total_proposals = tally.total_proposals;
                state.remaining_obligations = tally.remaining_obligations;
            }
            zcash_voting::RoundDriveEvent::StepFinished { .. } => {
                state.dispatches += 1;
            }
            zcash_voting::RoundDriveEvent::StepFailed { kind, message, .. } => {
                state
                    .failures
                    .push(format!("{kind:?}: {message}"));
            }
            _ => {}
        }
    }
}

/// Formats the stop reason for display.
fn quiescence_label(quiescence: &zcash_voting::RoundQuiescence) -> String {
    use zcash_voting::RoundQuiescence as Q;
    match quiescence {
        Q::NoWorkLeft => "done".to_string(),
        Q::NeedsBundleSetup => "needs_bundle_setup".to_string(),
        Q::PersistedChainTerminal => "chain_terminal".to_string(),
        Q::NeedsBallot {
            open_proposals, ..
        } => format!("needs_ballot({} open)", open_proposals.len()),
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct DriveKey {
    wallet_id: String,
    round_id: String,
}

#[derive(Clone)]
struct ActiveDrive {
    control: zcash_voting::ChainSubmissionControl,
    state: Arc<Mutex<DriveRunState>>,
}

/// Live driver runs are process-local: their durable queue is the voting
/// sidecar, so a restart simply discovers unfinished work again. Keyed by
/// wallet and round so a wallet switch cannot touch another wallet's run.
static ACTIVE_DRIVES: LazyLock<Mutex<HashMap<DriveKey, ActiveDrive>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_DRIVE_GENERATION: AtomicU64 = AtomicU64::new(1);

/// Starts one driver run for `binding`, or reports `Ok(false)` when a run is
/// already live for the wallet and round.
///
/// The executor is constructed and driven entirely on the spawned task; a
/// cancelled predecessor's SDK admission guard makes a replacement wait for
/// the departing run rather than double-dispatching.
pub fn start_round_drive(
    db: Arc<VotingDb>,
    binding: RoundBinding,
    routes: ExecutorRoutes,
    host: ZkoolRoundHost,
) -> Result<bool> {
    let key = DriveKey {
        wallet_id: db.wallet_id().to_owned(),
        round_id: binding.round_id.clone(),
    };
    let generation = NEXT_DRIVE_GENERATION.fetch_add(1, Ordering::Relaxed);
    let control = zcash_voting::ChainSubmissionControl::new(generation);
    {
        let mut active = ACTIVE_DRIVES
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if active
            .get(&key)
            .is_some_and(|drive| !drive.control.is_cancelled())
        {
            return Ok(false);
        }
        active.insert(
            key.clone(),
            ActiveDrive {
                control: control.clone(),
                state: Arc::new(Mutex::new(DriveRunState {
                    running: true,
                    ..Default::default()
                })),
            },
        );
    }

    tokio::spawn(async move {
        let executor = match build_executor(db, binding, &routes) {
            Ok(executor) => executor,
            Err(error) => {
                let state = run_state(&key);
                let mut state = state.lock().unwrap_or_else(|error| error.into_inner());
                state.running = false;
                state
                    .failures
                    .push(format!("executor construction failed: {error}"));
                return;
            }
        };
        let state = run_state(&key);
        let recorder = DriveRunRecorder {
            state: Arc::clone(&state),
        };
        let report = zcash_voting::RoundDriver::new(&executor)
            .run(&host, &control, &recorder)
            .await;
        {
            let mut state = state.lock().unwrap_or_else(|error| error.into_inner());
            state.running = false;
            state.quiescence = Some(quiescence_label(&report.quiescence));
            state.completed_proposals = report.tally.completed_proposals;
            state.total_proposals = report.tally.total_proposals;
            state.remaining_obligations = report.tally.remaining_obligations;
            if matches!(
                report.quiescence,
                zcash_voting::RoundQuiescence::Failures
            ) {
                for failure in &report.failures {
                    state
                        .failures
                        .push(format!("{:?}: {}", failure.failure.kind, failure.failure.message));
                }
            }
        }
    });
    Ok(true)
}

fn run_state(key: &DriveKey) -> Arc<Mutex<DriveRunState>> {
    let active = ACTIVE_DRIVES
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    active
        .get(key)
        .map(|drive| Arc::clone(&drive.state))
        .unwrap_or_default()
}

/// Cancels one round's driver run. The SDK wakes retry waits rather than
/// leaving the task parked until its next pass.
pub fn cancel_round_drive(wallet_id: &str, round_id: &str) -> bool {
    let key = DriveKey {
        wallet_id: wallet_id.to_owned(),
        round_id: round_id.to_owned(),
    };
    let active = ACTIVE_DRIVES
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let Some(drive) = active.get(&key) else {
        return false;
    };
    drive.control.cancel();
    true
}

/// Cancels all driver runs for this wallet, leaving other wallets untouched.
pub fn cancel_all_round_drives(wallet_id: &str) -> usize {
    let active = ACTIVE_DRIVES
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let mut cancelled = 0;
    for (key, drive) in active.iter() {
        if key.wallet_id == wallet_id && !drive.control.is_cancelled() {
            drive.control.cancel();
            cancelled += 1;
        }
    }
    cancelled
}

/// Snapshots one round's run status; idle when nothing is registered.
pub fn drive_status(wallet_id: &str, round_id: &str) -> DriveRunStatus {
    let key = DriveKey {
        wallet_id: wallet_id.to_owned(),
        round_id: round_id.to_owned(),
    };
    let active = ACTIVE_DRIVES
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    match active.get(&key) {
        Some(drive) => drive
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .snapshot(round_id),
        None => DriveRunStatus::idle(round_id),
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
        let hotkey = zcash_voting::prelude::generate_random_voting_hotkey(Network::Regtest)
            .expect("hotkey");
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
        let request = |seed_fingerprint: [u8; 32]| zcash_voting::prelude::DelegationSigningRequest {
            account_index: 0,
            network: Network::Regtest,
            seed_fingerprint,
            sighash: [9u8; 32],
            alpha: [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0,
            ],
        };
        assert!(sign_delegation_request(&seed, request([0u8; 32])).is_err());
        let signature = sign_delegation_request(&seed, request(fingerprint)).expect("signs");
        assert_eq!(signature.len(), 64);
        assert!(signature.iter().any(|&byte| byte != 0));
    }
}
