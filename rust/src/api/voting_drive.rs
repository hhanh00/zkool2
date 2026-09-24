//! FRB API to drive one voting round from ballot decisions to chain
//! confirmation.
//!
//! `voting_drive_start` resolves the authenticated config, gathers every
//! round input, and hands the round to the voting crate's driver, which runs
//! to quiescence on a process-local task. Status is polled; cancellation is
//! cooperative through the driver's control.

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{anyhow, Result};
use bip39::Mnemonic;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;
use sqlx::SqliteConnection;
use zip32::AccountId;
use zcash_keys::keys::UnifiedSpendingKey;
use zcash_voting::delegation_pipeline::DelegationAccountIdentity;
use zcash_voting::delegate::ResolveDelegationLwdParams;
use zcash_voting::prelude::BundlePolicy;

use crate::api::coin::Coin;
use crate::api::voting::{voting_network, wallet_id};
use crate::voting::{drive, hotkey, net as voting_net, note_source, sidecar::VotingSidecar};

/// The ZIP-32 seed bytes for an account, derived from its stored mnemonic.
async fn account_seed(connection: &mut SqliteConnection, account: u32) -> Result<Vec<u8>> {
    let seed = crate::account::get_account_seed(connection, account)
        .await?
        .ok_or_else(|| anyhow!("account {account} has no mnemonic seed"))?;
    let mnemonic = Mnemonic::from_str(&seed.mnemonic)?;
    Ok(mnemonic.to_seed(&seed.phrase).to_vec())
}

/// Wallet-derived delegation facts for the note-owning account.
fn delegation_identity(
    seed: &[u8],
    network: zcash_voting::Network,
    account: u32,
) -> Result<DelegationAccountIdentity> {
    let fingerprint = zip32::fingerprint::SeedFingerprint::from_seed(seed)
        .ok_or_else(|| anyhow!("wallet seed length is not valid for ZIP-32"))?;
    let account_id =
        AccountId::try_from(account).map_err(|_| anyhow!("invalid account index {account}"))?;
    let usk = UnifiedSpendingKey::from_seed(&network, seed, account_id)
        .map_err(|error| anyhow!("account spending key derivation failed: {error}"))?;
    let fvk = orchard::keys::FullViewingKey::from(usk.orchard());
    Ok(DelegationAccountIdentity {
        fvk_bytes: fvk.to_bytes().to_vec(),
        seed_fingerprint: fingerprint.to_bytes(),
        account_index: account,
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
    let proxy = crate::net::http::proxy_url(c.transport, &c.proxy);
    for server in servers {
        let Ok((status, body, _)) =
            crate::net::votechain::round_status(server, round_id, proxy).await
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

/// Snapshot of one round's driver run for the UI.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VotingDriveStatus {
    pub round_id: String,
    pub running: bool,
    pub dispatches: u64,
    pub completed_proposals: u32,
    pub total_proposals: u32,
    pub remaining_obligations: u32,
    pub quiescence: Option<String>,
    pub failures: Vec<String>,
}

/// Starts driving one round: delegation (if still owed), vote casting, chain
/// submission, and share handoff.
///
/// Returns false when a run is already live for this wallet and round. The
/// run is idempotent and resumable: every step is planned from durable
/// sidecar state, so restarting after a crash or cancel is always safe.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_drive_start(
    round_id: &str,
    lightwalletd_url: &str,
    round_name: &str,
    c: &Coin,
) -> Result<bool> {
    ensure_nonempty(round_id, "round id")?;
    ensure_nonempty(lightwalletd_url, "lightwalletd URL")?;
    let network = voting_network(c)?;
    let wallet = wallet_id(c).await?;
    let sidecar = VotingSidecar::open(PathBuf::from(&c.db_filepath), wallet.clone()).await?;

    let http_client = crate::net::http::client(
        crate::net::http::proxy_url(c.transport, &c.proxy),
        Duration::from_secs(15),
    )?;

    // Authenticated config and round material.
    let mut connection = c.get_connection().await?;
    let source = crate::db::get_prop(&mut connection, "voting_config_url")
        .await?
        .filter(|source| !source.is_empty())
        .ok_or_else(|| anyhow!("voting configuration is missing"))?;
    let config = voting_net::voting_config_resolve(&source, &http_client).await?;
    let params = voting_net::fetch_round_params(round_id, &config, &http_client).await?;
    let servers: Vec<String> = config
        .vote_servers
        .iter()
        .map(|server| server.url.clone())
        .collect();
    let server_round = voting_net::fetch_rounds(&servers, &http_client)
        .await?
        .into_iter()
        .find(|round| round.round_id == round_id)
        .ok_or_else(|| anyhow!("round {round_id} is not active"))?;
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

    // Wallet material: hotkey, seed-derived identity, notes and witnesses.
    let hotkey = hotkey::load_or_create(&mut connection, network).await?;
    let seed = account_seed(&mut connection, c.account).await?;
    let identity = delegation_identity(&seed, network, c.account)?;
    let mut client = c.client().await?;
    let note_source = note_source::ZkoolNoteSource::load(
        &c.network(),
        &mut connection,
        &mut client,
        c.account,
        params.snapshot_height,
    )
    .await
    .map_err(|error| anyhow!("{error}"))?;
    drop(client);
    drop(connection);

    // Lightwalletd-derived anchor material for the delegation precompute.
    let lwd = zcash_voting::delegate::gather_delegation_lwd_inputs(ResolveDelegationLwdParams {
        lightwalletd_url,
        network,
        round_params: params.clone(),
        round_name,
    })
    .await
    .map_err(|error| anyhow!("{error}"))?;

    let routes = drive::ExecutorRoutes {
        transport: c.transport,
        proxy: c.proxy.clone(),
        chain_endpoints: servers.clone(),
    };
    let delegation = drive::build_delegation_step_inputs(
        sidecar.db(),
        &hotkey,
        drive::DelegationHostInputs {
            lwd,
            identity,
            note_source,
            session_json: None,
            bundle_policy: BundlePolicy::default(),
            pir_endpoints: config
                .pir_endpoints
                .iter()
                .map(|endpoint| endpoint.url.clone())
                .collect(),
            pir_layout: config.pir_layout,
            seed,
        },
        &routes,
    )?;
    // The vote servers double as the vote-tree node fleet. Round timing
    // comes from the chain status so the driver can detect the last-moment
    // share window; the overview's vote end is the fallback.
    let timing = fetch_round_timing(&servers, round_id, c).await?;
    let (ceremony_start, vote_end) = match timing {
        Some((ceremony, end)) => (Some(ceremony), Some(end)),
        None => (None, server_round.vote_end_time),
    };
    let host =
        drive::ZkoolRoundHost::new(servers.clone(), servers, ceremony_start, vote_end, Some(
            delegation,
        ));
    let binding = drive::build_binding(round_id, network, &roster, Some(&hotkey))?;
    drive::start_round_drive(sidecar.db(), binding, routes, host)
}

/// Polls one round's driver run; idle when nothing was started or it finished.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_drive_status(round_id: &str, c: &Coin) -> Result<VotingDriveStatus> {
    let wallet = wallet_id(c).await?;
    Ok(match drive::drive_status(&wallet, round_id) {
        status => VotingDriveStatus {
            round_id: status.round_id,
            running: status.running,
            dispatches: status.dispatches,
            completed_proposals: status.completed_proposals,
            total_proposals: status.total_proposals,
            remaining_obligations: status.remaining_obligations,
            quiescence: status.quiescence,
            failures: status.failures,
        },
    })
}

/// Cancels one round's driver run, waking any in-flight retry wait.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_drive_cancel(round_id: &str, c: &Coin) -> Result<bool> {
    let wallet = wallet_id(c).await?;
    Ok(drive::cancel_round_drive(&wallet, round_id))
}

fn ensure_nonempty(value: &str, name: &str) -> Result<()> {
    anyhow::ensure!(!value.trim().is_empty(), "{name} must not be empty");
    Ok(())
}
