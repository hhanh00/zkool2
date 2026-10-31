//! FRB API to drive one voting round from ballot decisions to chain
//! confirmation.
//!
//! [`VotingDrive`] is the host's handle on one round's run, like the mempool
//! and FROST drivers: `run` resolves the authenticated config, gathers every
//! round input, and hands the round to the voting crate's driver, which casts
//! the votes and confirms the helper shares on its own task while streaming
//! its status. The holder keeps the handle for as long as the run should
//! live and cancels it cooperatively.

use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, ensure, Result};
use bip39::Mnemonic;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;
use sqlx::SqliteConnection;
use zcash_voting::delegate::{DelegationLwdInputs, ResolveDelegationLwdParams};
use zcash_voting::delegation_pipeline::DelegationAccountIdentity;
use zcash_voting::prelude::BundlePolicy;
use zcash_voting::session::resume_plan;

use crate::api::coin::Coin;
use crate::api::voting::{voting_network, voting_route, wallet_id};
#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;
use crate::voting::{drive, hotkey, net as voting_net, note_source, sidecar::VotingSidecar};
use crate::Sink;

/// The ZIP-32 seed bytes for an account, derived from its stored mnemonic.
async fn account_seed(connection: &mut SqliteConnection, account: u32) -> Result<Vec<u8>> {
    let seed = crate::account::get_account_seed(connection, account)
        .await?
        .ok_or_else(|| anyhow!("account {account} has no mnemonic seed"))?;
    let mnemonic = Mnemonic::from_str(&seed.mnemonic)?;
    Ok(mnemonic.to_seed(&seed.phrase).to_vec())
}

/// Wallet-derived delegation facts for the note-owning account.
///
/// `fvk_bytes` must be the wallet's stored account viewing key — the same
/// key the note source stamps onto every note. The voting crate rebuilds
/// each note from that key and recomputes the proof's rk from it, while the
/// PCZT's rk comes from this identity; deriving the identity from the seed
/// instead lets the two keys drift (imported keys, derivation changes) and
/// the proof is then rejected with "rk does not match stored PCZT data".
///
/// `aindex` must be the account's ZIP-32 derivation index (`accounts.aindex`),
/// not the row id: the crate embeds it in the PCZT ZIP-32 path and the signer
/// derives its spending key from it, so a row id here signs with a different
/// account's key than the notes and the spend use.
fn delegation_identity(
    fvk: &orchard::keys::FullViewingKey,
    seed: &[u8],
    aindex: u32,
) -> Result<DelegationAccountIdentity> {
    let fingerprint = zip32::fingerprint::SeedFingerprint::from_seed(seed)
        .ok_or_else(|| anyhow!("wallet seed length is not valid for ZIP-32"))?;
    Ok(DelegationAccountIdentity {
        fvk_bytes: fvk.to_bytes().to_vec(),
        seed_fingerprint: fingerprint.to_bytes(),
        account_index: aindex,
    })
}

/// Parses a JSON value as Unix seconds; the chain serves ints with string
/// fallbacks.
fn json_unix_seconds(value: &serde_json::Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|raw| raw.parse().ok()))
}

/// Fetches round timing from the vote chain status endpoint, rotating
/// through equivalent servers.
///
/// Returns `(ceremony_start, vote_end)` when the chain publishes both; the
/// driver needs them to detect the last-moment share window. Missing timing
/// degrades to share tracking without boundaries rather than failing the run.
async fn fetch_round_timing(
    servers: &[String],
    round_id: &str,
    c: &Coin,
) -> Result<Option<(u64, u64)>> {
    let route = voting_route(c)?;
    for server in servers {
        let Ok((status, body, _)) =
            crate::net::votechain::round_status(server, round_id, &route).await
        else {
            continue;
        };
        if !(200..300).contains(&status) {
            continue;
        }
        let Ok(body) = serde_json::from_str::<serde_json::Value>(&body) else {
            continue;
        };
        let round = body.get("round").cloned().unwrap_or_default();
        let ceremony = round
            .get("ceremony_phase_start")
            .and_then(json_unix_seconds);
        let end = round.get("vote_end_time").and_then(json_unix_seconds);
        if let (Some(ceremony), Some(end)) = (ceremony, end) {
            return Ok(Some((ceremony, end)));
        }
    }
    Ok(None)
}

/// Snapshot of one round's run for the UI, streamed by `VotingDrive::run`;
/// `voting_drive_status` gives the idle standing of a round with no run.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VotingDriveStatus {
    pub round_id: String,
    /// Where the run is: "idle" (no run), "casting" (delegation, proving,
    /// chain submission), "confirming_shares" (the votes are on the chain
    /// and the helper shares are being confirmed) or "stopped" (see
    /// `quiescence`).
    pub phase: String,
    pub running: bool,
    pub dispatches: u64,
    pub completed_proposals: u32,
    pub total_proposals: u32,
    /// Foreground obligations left, including share work the run has handed to
    /// the background tracker. Prefer `shares_*` for progress the user sees.
    pub remaining_obligations: u32,
    /// Helper shares already confirmed for the round, and the total recorded.
    /// `total == 0` means the round has no share work yet (not yet cast, or
    /// an imported capability).
    pub shares_confirmed: u32,
    pub shares_total: u32,
    /// Why the run stopped: "done" once every share is confirmed,
    /// "round_closed(n)" when the vote end passed with n shares unconfirmed,
    /// "unrecoverable(n)" when n shares cannot be repaired, "cancelled", or
    /// one of the cast's own stop reasons.
    pub quiescence: Option<String>,
    pub failures: Vec<String>,
}

impl From<drive::DriveRunStatus> for VotingDriveStatus {
    fn from(status: drive::DriveRunStatus) -> Self {
        Self {
            round_id: status.round_id,
            phase: status.phase,
            running: status.running,
            dispatches: status.dispatches,
            completed_proposals: status.completed_proposals,
            total_proposals: status.total_proposals,
            remaining_obligations: status.remaining_obligations,
            shares_confirmed: status.shares_confirmed,
            shares_total: status.shares_total,
            quiescence: status.quiescence,
            failures: status.failures,
        }
    }
}

/// Adapts a `VotingDriveStatus` sink to the run's `DriveRunStatus` sends.
#[derive(Clone)]
struct StatusSink<S>(S);

impl<S: Sink<VotingDriveStatus> + Send + Sync> Sink<drive::DriveRunStatus> for StatusSink<S> {
    async fn send(&self, value: drive::DriveRunStatus) {
        self.0.send(value.into()).await;
    }

    async fn send_error(&self, error: anyhow::Error) {
        self.0.send_error(error).await;
    }
}

/// Authenticated config and round material resolved from the vote servers.
pub(crate) struct RoundConfig {
    pub(crate) sidecar: VotingSidecar,
    pub(crate) network: zcash_voting::Network,
    pub(crate) servers: Vec<String>,
    pub(crate) roster: Vec<(u32, u32)>,
    /// Vote end as the rounds overview reported it; the chain status fetch
    /// refines it with the ceremony boundary.
    pub(crate) vote_end: Option<u64>,
    pub(crate) params: zcash_voting::VotingRoundParams,
    pub(crate) pir_endpoints: Vec<String>,
    pub(crate) pir_layout: zcash_voting::config::PirLayout,
}

/// Wallet-derived keys and notes needed for delegation and vote casting.
pub(crate) struct WalletInputs {
    pub(crate) hotkey: zcash_voting::prelude::VotingHotkey,
    pub(crate) seed: Vec<u8>,
    pub(crate) identity: DelegationAccountIdentity,
    pub(crate) note_source: note_source::ZkoolNoteSource,
}

/// Everything `voting_drive_start` and `voting_prepare_round` need, gathered
/// once: authenticated config, round material, and wallet keys and notes.
struct GatheredRound {
    sidecar: VotingSidecar,
    network: zcash_voting::Network,
    servers: Vec<String>,
    roster: Vec<(u32, u32)>,
    /// Vote end as the rounds overview reported it; the chain status fetch
    /// refines it with the ceremony boundary.
    vote_end: Option<u64>,
    hotkey: zcash_voting::prelude::VotingHotkey,
    inputs: drive::DelegationHostInputs,
}

/// Fetches the authenticated voting config and round material from the vote
/// servers. Opens the voting sidecar. No wallet DB access beyond reading the
/// `voting_config_url` prop and the seed fingerprint.
async fn resolve_round_config(round_id: &str, account: u32, c: &Coin) -> Result<RoundConfig> {
    let network = voting_network(c)?;
    let wallet = wallet_id(account, c).await?;
    let sidecar = VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet).await?;

    let route = voting_route(c)?;

    let mut connection = c.get_connection().await?;
    let source = crate::db::get_prop(&mut connection, "voting_config_url")
        .await?
        .filter(|source| !source.is_empty())
        .ok_or_else(|| anyhow!("voting configuration is missing"))?;
    drop(connection);

    let config = voting_net::voting_config_resolve(&source, &route).await?;
    let params = voting_net::fetch_round_params(round_id, &config, &route).await?;
    let servers: Vec<String> = config
        .vote_servers
        .iter()
        .map(|server| server.url.clone())
        .collect();
    let server_round = voting_net::fetch_rounds(&servers, &route)
        .await?
        .into_iter()
        .find(|round| round.round_id == round_id)
        .ok_or_else(|| anyhow!("round {round_id} is not active"))?;
    let vote_end = server_round.vote_end_time;
    let roster: Vec<(u32, u32)> = server_round
        .proposals
        .iter()
        .map(|proposal| {
            (
                proposal.id,
                u32::try_from(proposal.options.len().max(2)).unwrap_or(2),
            )
        })
        .collect();

    Ok(RoundConfig {
        sidecar,
        network,
        servers,
        roster,
        vote_end,
        params,
        pir_endpoints: config
            .pir_endpoints
            .iter()
            .map(|endpoint| endpoint.url.clone())
            .collect(),
        pir_layout: config.pir_layout,
    })
}

/// Loads wallet keys and notes needed for delegation.
///
/// The identity's viewing key must be the stored account key the note source
/// uses, not a fresh seed derivation — see `delegation_identity`.
/// The signer's ZIP-32 index, never the row id — see `delegation_identity`.
async fn gather_wallet_inputs(
    account: u32,
    c: &Coin,
    config: &RoundConfig,
) -> Result<WalletInputs> {
    let mut connection = c.get_connection().await?;
    let hotkey = hotkey::load_or_create(&mut connection, config.network).await?;
    let seed = account_seed(&mut connection, account).await?;
    let orchard_keys = crate::db::select_account_orchard(&mut connection, account).await?;
    let account_fvk = orchard_keys
        .xvk
        .ok_or_else(|| anyhow!("account {} has no orchard viewing key", account))?;
    let aindex = crate::db::get_account_aindex(&mut connection, account).await?;
    let identity = delegation_identity(&account_fvk, &seed, aindex)?;
    let mut client = c.client().await?;
    let note_source = note_source::ZkoolNoteSource::load(
        &c.network(),
        &mut connection,
        &mut client,
        account,
        config.params.snapshot_height,
    )
    .await
    .map_err(|error| anyhow!("{error}"))?;
    Ok(WalletInputs { hotkey, seed, identity, note_source })
}

/// Fetches lightwalletd-derived anchor and branch-id material for the
/// delegation precompute.
async fn gather_lwd_inputs<'a>(
    config: &'a RoundConfig,
    lightwalletd_url: &'a str,
    round_name: &'a str,
) -> Result<DelegationLwdInputs> {
    zcash_voting::delegate::gather_delegation_lwd_inputs(ResolveDelegationLwdParams {
        lightwalletd_url,
        network: config.network,
        round_params: config.params.clone(),
        round_name,
    })
    .await
    .map_err(|error| anyhow!("{error}"))
}

/// Thin coordinator: resolves config, then gathers wallet and LWD inputs in
/// parallel (they are independent once the config is in hand).
async fn gather_round_inputs(
    round_id: &str,
    lightwalletd_url: &str,
    round_name: &str,
    account: u32,
    c: &Coin,
) -> Result<GatheredRound> {
    let config = resolve_round_config(round_id, account, c).await?;
    let (wallet, lwd) = tokio::try_join!(
        gather_wallet_inputs(account, c, &config),
        gather_lwd_inputs(&config, lightwalletd_url, round_name),
    )?;
    Ok(GatheredRound {
        sidecar: config.sidecar,
        network: config.network,
        servers: config.servers,
        roster: config.roster,
        vote_end: config.vote_end,
        hotkey: wallet.hotkey,
        inputs: drive::DelegationHostInputs {
            lwd,
            identity: wallet.identity,
            note_source: wallet.note_source,
            session_json: None,
            bundle_policy: BundlePolicy::default(),
            pir_endpoints: config.pir_endpoints,
            pir_layout: config.pir_layout,
            seed: wallet.seed,
        },
    })
}

/// Runs the delegation pipeline's bundle setup on a blocking thread.
///
/// Bundle setup persists the round's bundle layout (idempotent: existing rows
/// are reused when they still match the eligible note set), which the driver
/// requires before it can plan any vote work. The eligibility preview
/// describes the persisted plan.
async fn prepare_bundles(gathered: GatheredRound, _c: &Coin) -> Result<VotingEligibilityPreview> {
    let pipeline = drive::build_pipeline(gathered.sidecar.db(), &gathered.hotkey, gathered.inputs)?;
    let report =
        tokio::task::spawn_blocking(move || -> Result<zcash_voting::VotingEligibilityReport> {
            pipeline.setup_bundles()?;
            Ok(pipeline.eligibility()?)
        })
        .await
        .map_err(|error| anyhow!("bundle setup task failed: {error}"))??;
    Ok(VotingEligibilityPreview {
        note_count: u32::try_from(report.eligibility.distinct_note_count).unwrap_or(u32::MAX),
        eligible_weight_zatoshi: report.eligibility.eligible_weight,
        is_eligible: report.eligibility.is_eligible(),
        privacy_trim_dropped_value_zatoshi: report.privacy_trim_dropped_value_zatoshi,
        skipped_suffix_bundles: report.skipped_suffix_bundles,
        skipped_suffix_notes: report.skipped_suffix_notes,
        skipped_suffix_value_zatoshi: report.skipped_suffix_value_zatoshi,
    })
}

/// Eligibility preview for the round's persisted bundle plan.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VotingEligibilityPreview {
    pub note_count: u32,
    pub eligible_weight_zatoshi: u64,
    /// False when the weight is below the round's minimum voting rule.
    pub is_eligible: bool,
    /// Raw note value the privacy trim withholds from delegation.
    pub privacy_trim_dropped_value_zatoshi: u64,
    pub skipped_suffix_bundles: u32,
    pub skipped_suffix_notes: u32,
    pub skipped_suffix_value_zatoshi: u64,
}

/// Prepares one round for submission: persists the bundle layout from the
/// wallet's eligible notes and returns the eligibility preview.
///
/// The explicit user step before `voting_drive_start`: without persisted
/// bundle rows the driver quiesces with `needs_bundle_setup` and nothing can
/// be planned. Safe to repeat; existing bundle rows are reused when they
/// still match the current note set.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_prepare_round(
    round_id: &str,
    lightwalletd_url: &str,
    round_name: &str,
    account: u32,
    c: &Coin,
) -> Result<VotingEligibilityPreview> {
    ensure_nonempty(round_id, "round id")?;
    ensure_nonempty(lightwalletd_url, "lightwalletd URL")?;
    let gathered = gather_round_inputs(round_id, lightwalletd_url, round_name, account, c).await?;
    prepare_bundles(gathered, c).await
}

/// Gathers one round's inputs and starts its run, with status to `sink`.
async fn start_round_drive<S>(
    round_id: &str,
    lightwalletd_url: &str,
    round_name: &str,
    account: u32,
    c: &Coin,
    sink: S,
) -> Result<drive::RoundDrive>
where
    S: Sink<VotingDriveStatus> + Send + Sync + 'static,
{
    ensure_nonempty(round_id, "round id")?;
    ensure_nonempty(lightwalletd_url, "lightwalletd URL")?;
    let gathered = gather_round_inputs(round_id, lightwalletd_url, round_name, account, c).await?;
    let GatheredRound {
        sidecar,
        network,
        servers,
        roster,
        vote_end,
        hotkey,
        inputs,
    } = gathered;

    let routes = drive::ExecutorRoutes {
        transport: c.transport,
        proxy: c.proxy.clone(),
        chain_endpoints: servers.clone(),
    };
    let delegation = drive::build_delegation_step_inputs(sidecar.db(), &hotkey, inputs, &routes)?;
    // The vote servers double as the vote-tree node fleet. Round timing
    // comes from the chain status so the driver can detect the last-moment
    // share window; the overview's vote end is the fallback.
    // Deliver helper shares immediately: leaving the ceremony start unset
    // makes the crate's last-moment buffer absent, so every share is planned
    // with `submit_at = 0` instead of a randomized delay spread across the
    // round. This trades the timing-correlation resistance of the spread for a
    // single foreground delivery pass. Vote end is still carried for recovery.
    let vote_end = match fetch_round_timing(&servers, round_id, c).await? {
        Some((_ceremony, end)) => Some(end),
        None => vote_end,
    };
    let host =
        drive::ZkoolRoundHost::new(servers.clone(), servers, None, vote_end, Some(delegation));
    let binding = drive::build_binding(round_id, network, &roster, Some(&hotkey))?;
    Ok(drive::RoundDrive::start(
        sidecar.db(),
        binding,
        routes,
        host,
        StatusSink(sink),
    ))
}

/// The host's handle on one round's submission run: delegation (if still
/// owed), vote casting, chain submission, then helper-share confirmation.
///
/// The run is idempotent and resumable: every step is planned from durable
/// sidecar state, so starting again after a crash or cancel is always safe.
/// The holder decides whether a round already has a run; `run` refuses to
/// start a second one while the first is live.
#[cfg_attr(feature = "flutter", frb(opaque))]
pub struct VotingDrive {
    drive: Option<drive::RoundDrive>,
}

impl VotingDrive {
    #[cfg_attr(feature = "flutter", frb(sync))]
    #[allow(clippy::new_without_default)] // Default would surface in Dart
    pub fn new() -> Self {
        Self { drive: None }
    }

    /// Starts the run and streams its status to `status` until the run
    /// stops: the last item has `running == false`.
    #[cfg(feature = "flutter")]
    pub async fn run(
        &mut self,
        round_id: &str,
        lightwalletd_url: &str,
        round_name: &str,
        account: u32,
        c: &Coin,
        status: StreamSink<VotingDriveStatus>,
    ) -> Result<()> {
        self.start_with(round_id, lightwalletd_url, round_name, account, c, status)
            .await
    }

    /// Starts the run for a host that polls `status()`.
    #[cfg_attr(feature = "flutter", frb(ignore))]
    pub async fn start(
        &mut self,
        round_id: &str,
        lightwalletd_url: &str,
        round_name: &str,
        account: u32,
        c: &Coin,
    ) -> Result<()> {
        self.start_with(round_id, lightwalletd_url, round_name, account, c, ())
            .await
    }

    async fn start_with<S>(
        &mut self,
        round_id: &str,
        lightwalletd_url: &str,
        round_name: &str,
        account: u32,
        c: &Coin,
        sink: S,
    ) -> Result<()>
    where
        S: Sink<VotingDriveStatus> + Send + Sync + 'static,
    {
        ensure!(
            !self.is_running(),
            "a submission run is already live for round {round_id}"
        );
        let drive =
            start_round_drive(round_id, lightwalletd_url, round_name, account, c, sink).await?;
        self.drive = Some(drive);
        Ok(())
    }

    /// Cancels the run cooperatively, waking any in-flight retry wait. The
    /// stream ends with the cancelled snapshot once the step in flight ends.
    #[cfg_attr(feature = "flutter", frb(sync))]
    pub fn cancel(&self) {
        if let Some(drive) = &self.drive {
            drive.cancel();
        }
    }

    /// The latest snapshot of the run; idle before the first `run`.
    #[cfg_attr(feature = "flutter", frb(sync))]
    pub fn status(&self) -> Option<VotingDriveStatus> {
        self.drive.as_ref().map(|drive| drive.status().into())
    }

    #[cfg_attr(feature = "flutter", frb(sync))]
    pub fn is_running(&self) -> bool {
        self.drive
            .as_ref()
            .is_some_and(drive::RoundDrive::is_running)
    }
}

/// The round's standing without a run as a status snapshot: phase idle,
/// with the helper share counts from the sidecar. The GraphQL server
/// answers a status poll with it when it holds no run for the round; the
/// app reads `voting_round_standing` instead.
#[cfg_attr(feature = "flutter", frb(ignore))]
pub async fn voting_drive_status(
    round_id: &str,
    account: u32,
    c: &Coin,
) -> Result<VotingDriveStatus> {
    let wallet = wallet_id(account, c).await?;
    let sidecar = VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet).await?;
    let (shares_confirmed, shares_total) = drive::share_counts(sidecar.db(), round_id).await?;
    Ok(VotingDriveStatus {
        shares_confirmed,
        shares_total,
        ..drive::DriveRunStatus::idle(round_id).into()
    })
}

/// Where a round stands in the sidecar, for a page that opens it: whether a
/// submission was started and whether it still has work, so the page can
/// resume the run without asking (the run is idempotent) or show the vote
/// as settled.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VotingRoundStanding {
    /// The submission was started: a bundle moved past preparation, or a
    /// vote or a helper share exists. Preparing the bundles for the
    /// eligibility preview does not count.
    pub submission_started: bool,
    /// A vote artifact exists for the ballot.
    pub votes_cast: bool,
    pub shares_confirmed: u32,
    pub shares_total: u32,
    /// The submission was started and is not settled: casting, chain
    /// recovery or share confirmation remains, so a run should be started.
    pub work_pending: bool,
}

/// Reads the round's standing from the sidecar. `proposal_ids` is the
/// authenticated roster, which the plan needs to tell a complete ballot.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_round_standing(
    round_id: &str,
    proposal_ids: Vec<u32>,
    account: u32,
    c: &Coin,
) -> Result<VotingRoundStanding> {
    let wallet = wallet_id(account, c).await?;
    let sidecar = VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet).await?;
    let (shares_confirmed, shares_total) = drive::share_counts(sidecar.db(), round_id).await?;
    let round = round_id.to_owned();
    let plan = sidecar
        .run(move |db| resume_plan(db, &round, &proposal_ids))
        .await?;
    let settled = plan.completed_for_display && !plan.has_unconfirmed_shares;
    Ok(VotingRoundStanding {
        submission_started: plan.hotkey_bound,
        votes_cast: plan.completed_vote_artifact,
        shares_confirmed,
        shares_total,
        work_pending: plan.hotkey_bound && !settled,
    })
}

fn ensure_nonempty(value: &str, name: &str) -> Result<()> {
    anyhow::ensure!(!value.trim().is_empty(), "{name} must not be empty");
    Ok(())
}
