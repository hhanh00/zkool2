//! Durable voting workflow (durare): the vote round lifecycle as a
//! checkpointed multi-step workflow.
//!
//! This module replaces the Dart-side `VotingSubmissionJob` orchestration with
//! a durable workflow running inside the app process on the `durare` engine:
//! every side-effecting operation (prepare, prove, submit, confirm, share
//! post, tracking tick) is a checkpointed step that runs exactly once and
//! resumes from its checkpoint after an app restart.
//!
//! Ownership model (see docs/voting-workflow.md for the full spec):
//! - The **durare workflow** owns execution order and progress (checkpointed
//!   steps, retries, durable sleeps). Its state lives in the `workflow_status`
//!   / `operation_outputs` / `streams` tables inside the wallet DB (SQLCipher,
//!   via `SqliteProvider::from_pool`).
//! - The **fork's `voting_*` tables** remain the artifact source of truth
//!   (commitment bundles, tx hashes, share records, phases). Every step is
//!   idempotent against that state — re-running a step after a crash skips
//!   completed work exactly like the old plan re-read.
//! - The **Dart monitor** only starts the workflow, polls status, and offers
//!   retry; it holds no orchestration logic.
//!
//! Step inventory (each step name is durable — renaming breaks in-flight
//! workflows):
//!
//! | step | responsibility |
//! | --- | --- |
//! | `ensure_config` | persist round config (params/name/lwd/policy) or verify it exists |
//! | `delegation_work` | read plan; ensure hotkey; decide next delegation work |
//! | `prepare_delegation` | prepare bundle (fresh or resume from saved config), fill prepared-bundle cache |
//! | `build_and_submit_delegation` | resolve PIR, prove (512MB-stack thread), submit to chain, record tx hash |
//! | `poll_delegation_confirmation` | `step_with` 45×2s tx confirmation poll |
//! | `confirm_delegation` | record VAN position + verify bundle reads back confirmed |
//! | `vote_work` | read plan+drafts; write ballot intents; emit ordered vote work |
//! | `cast_vote` | commit ONE proposal (VAN interleave: commit→submit→confirm per proposal) |
//! | `submit_vote` | wire JSON → chain → record tx hash |
//! | `poll_vote` | `step_with` tx confirmation poll |
//! | `confirm_vote` | record vote confirmation |
//! | `share_work` | resolve timing; enumerate payloads from confirmed votes; plan submissions (entropy inside) |
//! | `submit_share` | wire JSON + `vote_round_id` → post per target server → record |
//! | `tracking_tick` | poll helpers, confirm shares, resubmit overdue |
//! | `final_summary` | derive done label + output from fork state |
//!
//! Determinism rules: entropy and `now` only inside step closures
//! (checkpointed); `ctx.sleep` (never `tokio::sleep`) at body level;
//! `ctx.write_stream` only at body level (durare #173); sorted iteration.

use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use sqlx::Row as _;
use zcash_voting::prelude::{
    DelegationProgress, DelegationProgressBridge, DraftVote, TxEvent, VoteCommitStageBridge,
};

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

use durare::{DurableContext, DurableEngine, Serializer, SqliteProvider, StepOptions, WorkflowOptions};

use crate::api::coin::Coin;
use crate::api::voting;
use crate::voting as vc;

// ---------------------------------------------------------------------------
// Workflow input / output
// ---------------------------------------------------------------------------

/// Everything the durable vote workflow needs to run (and to reconstruct a
/// `Coin` after a restart, since the workflow input is the only durable state
/// the engine keeps besides the fork's `voting_*` tables).
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoteRoundInput {
    pub round_id: String,
    // Wallet identity (Coin reconstruction after restart).
    pub coin: u8,
    pub account: u32,
    pub db_filepath: String,
    pub url: String,
    pub server_type: u8,
    pub transport: u8,
    pub proxy: String,
    // Vote-chain parameters (mirrors the voting status page's start args).
    pub chain_url: String,
    pub pir_server_url: String,
    pub pir_layout: Option<voting::VotingPirLayout>,
    pub round_params_json: Option<String>,
    pub round_name: Option<String>,
    pub max_real_notes_per_bundle: Option<u32>,
    pub lightwalletd_url: Option<String>,
    pub vote_node_url: String,
    pub ceremony_start: u64,
    pub vote_end: Option<u64>,
    pub share_server_urls: Vec<String>,
    pub single_share: bool,
}

/// The workflow's terminal output, shown by the UI when the round completes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteRoundOutput {
    pub delegated: bool,
    pub voted: bool,
    pub shared: bool,
    pub done_label: String,
    pub tx_hash: Option<String>,
    pub confirm_height: Option<u64>,
    pub eligible_weight_zatoshi: Option<u64>,
}

/// One durable progress event appended to the workflow's `progress` stream.
/// Stage strings use the existing UI vocabulary
/// (`idle|running|preparing|proving|submitting|confirming|voting|shares|done|error`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub stage: String,
    pub progress: Option<f64>,
    pub message: Option<String>,
    /// Set on "confirming" events so the UI can show the last confirmed height.
    pub confirm_height: Option<u64>,
    /// Set on "confirming" events so the UI can show the last confirmed tx.
    pub tx_hash: Option<String>,
}

/// Polled status of a round's durable workflow, for the Dart monitor.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VotingWorkflowStatus {
    pub round_id: String,
    /// `pending` | `success` | `error` | `cancelled` | `unknown`
    pub status: String,
    /// `idle|running|preparing|proving|submitting|confirming|voting|shares|done|error`
    pub stage: String,
    pub progress: Option<f64>,
    pub error: Option<String>,
    pub done_label: Option<String>,
    pub tx_hash: Option<String>,
    pub confirm_height: Option<u64>,
    pub eligible_weight_zatoshi: Option<u64>,
}

// ---------------------------------------------------------------------------
// Vote chain client
//
// `VoteChainClient` / `NetVoteChain` / `VOTECHAIN_OVERRIDE` live in
// `crate::voting` (the core module) rather than here so the FRB codegen
// (which scans `crate::api` only) never picks them up.
// ---------------------------------------------------------------------------

/// Resolves the chain client for a workflow input: the test override when
/// installed, otherwise the network client honoring the transport setting.
fn resolve_chain(input: &VoteRoundInput) -> Arc<dyn vc::VoteChainClient> {
    vc::resolve_chain_client(
        input.transport == 3,
        &input.proxy,
    )
}

// ---------------------------------------------------------------------------
// Live progress (display-only overlay; durable progress lives in the stream)
// ---------------------------------------------------------------------------

/// Latest proof progress per round, updated by the fork's progress bridges
/// inside steps. Not durable — on restart the UI falls back to the stream's
/// stage events until the next live update.
static LIVE_PROGRESS: LazyLock<Mutex<std::collections::HashMap<String, (String, f64)>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

fn set_live_progress(round_id: &str, stage: &str, progress: f64) {
    if let Ok(mut map) = LIVE_PROGRESS.lock() {
        map.insert(round_id.to_string(), (stage.to_string(), progress));
    }
}

fn peek_live_progress(round_id: &str) -> Option<(String, f64)> {
    LIVE_PROGRESS
        .lock()
        .ok()
        .and_then(|map| map.get(round_id).cloned())
}

fn clear_live_progress(round_id: &str) {
    if let Ok(mut map) = LIVE_PROGRESS.lock() {
        map.remove(round_id);
    }
}

// ---------------------------------------------------------------------------
// Engine registry
// ---------------------------------------------------------------------------

/// One engine per wallet DB file (the wallet pool is shared, so one engine
/// per encrypted DB is enough; the engine's dispatcher tasks live as long as
/// the process).
static ENGINES: LazyLock<Mutex<std::collections::HashMap<String, Arc<DurableEngine>>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

/// Stable identity for the voting executor. A fixed id is REQUIRED for
/// `recover_on_launch` (it re-dispatches only rows whose `executor_id`
/// matches); a stable app version lets recovery work across app updates.
const VOTING_EXECUTOR_ID: &str = "zkool-voting";
const VOTING_APP_VERSION: &str = "zkool";

/// Returns (building and launching on first use) the durable engine over the
/// wallet's SQLCipher pool. Runs durare's sqlite migrations on first build;
/// with `recover_on_launch(true)` it re-dispatches this executor's rows left
/// pending by a previous process before the dispatchers start.
async fn ensure_engine(c: &Coin) -> Result<Arc<DurableEngine>> {
    let key = c.db_filepath.clone();
    if let Some(engine) = ENGINES.lock().unwrap().get(&key) {
        return Ok(engine.clone());
    }
    let pool = c.get_pool()?;
    let provider = SqliteProvider::from_pool(pool).with_serializer(Serializer::Portable);
    let mut builder = DurableEngine::builder(Arc::new(provider));
    builder.recover_on_launch(true);
    builder.executor_id(VOTING_EXECUTOR_ID);
    builder.app_version(VOTING_APP_VERSION);
    let engine = builder.build().await?;
    engine.launch().await?;
    let engine = Arc::new(engine);
    ENGINES.lock().unwrap().insert(key, engine.clone());
    Ok(engine)
}

// ---------------------------------------------------------------------------
// Pure helpers (mirror the Dart driver's parsing)
// ---------------------------------------------------------------------------

/// Maps displayable errors (anyhow, serde_json, ...) into durare errors for
/// step closures.
fn durare_err<E: std::fmt::Display>(e: E) -> durare::Error {
    durare::Error::app(e.to_string())
}

/// Epoch seconds (used inside step closures only, where the value is
/// checkpointed — never at workflow body level).
fn now_epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Parsed proof of block inclusion from a 200 `GET tx/{hash}` response;
/// `None` means "not confirmed yet" (mirrors `parseVoteChainTxConfirmation`).
/// Crosses the durare checkpoint boundary as a step output.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct TxConfirmation {
    events_json: String,
    height: u64,
}

fn parse_tx_confirmation(body: &str) -> Option<TxConfirmation> {
    let decoded: serde_json::Value = serde_json::from_str(body).ok()?;
    if !decoded.is_object() {
        return None;
    }
    let raw_height = decoded.get("height")?;
    // The chain reports height as a JSON string ("7163319"), sometimes a number.
    let height = match raw_height {
        v if v.is_i64() => v.as_i64().unwrap(),
        v if v.is_u64() => v.as_u64().unwrap() as i64,
        v if v.is_f64() => v.as_f64().unwrap() as i64,
        v if v.is_string() => v.as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0),
        _ => 0,
    };
    if height <= 0 {
        return None;
    }
    let events = decoded
        .get("events")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    Some(TxConfirmation {
        events_json: events.to_string(),
        height: height as u64,
    })
}

/// Parses a 2xx submit response (`VotingTxResult`) and returns the tx hash,
/// rejecting a non-zero `code` (mirrors the Dart driver's checks).
fn parse_submit_response(body: &str) -> Result<String> {
    let decoded: serde_json::Value = serde_json::from_str(body)?;
    let tx_hash = decoded
        .get("tx_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let code = decoded.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
    if code != 0 || tx_hash.is_empty() {
        let log = decoded
            .get("log")
            .and_then(|v| v.as_str())
            .unwrap_or(body);
        bail!("vote chain rejected the transaction: {log}");
    }
    Ok(tx_hash)
}

/// Proposal ids from the persisted draft ballot (mirrors `voting_sessions`).
fn draft_proposal_ids(drafts_json: &str) -> Vec<u32> {
    let Ok(decoded) = serde_json::from_str::<serde_json::Value>(drafts_json) else {
        return Vec::new();
    };
    decoded
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|d| d.get("proposal_id").and_then(|v| v.as_u64()))
                .map(|n| n as u32)
                .filter(|&id| id > 0)
                .collect()
        })
        .unwrap_or_default()
}

/// Converts UI drafts to the fork's `DraftVote` JSON for the commit step:
/// drops skipped choices (validation rejects choice == num_options) and fills
/// the fields the fork requires with no serde defaults. `None` when there is
/// nothing to cast (mirrors `_sanitizedCommitDrafts`).
fn sanitized_commit_drafts(drafts_json: &str) -> Option<String> {
    let Ok(decoded) = serde_json::from_str::<serde_json::Value>(drafts_json) else {
        return None;
    };
    let commit: Vec<serde_json::Value> = decoded
        .as_array()?
        .iter()
        .filter_map(|d| {
            let choice = d.get("choice")?;
            let num_options = d.get("num_options")?;
            if choice == num_options {
                return None;
            }
            let mut m = serde_json::Map::new();
            m.insert("proposal_id".into(), d.get("proposal_id")?.clone());
            m.insert("choice".into(), choice.clone());
            m.insert("num_options".into(), num_options.clone());
            m.insert("vc_tree_position".into(), serde_json::json!(0));
            m.insert("single_share".into(), serde_json::json!(false));
            Some(serde_json::Value::Object(m))
        })
        .collect();
    if commit.is_empty() {
        None
    } else {
        serde_json::to_string(&commit).ok()
    }
}

/// Honest done label when a run performed no work (mirrors `_remainingLabel`).
fn remaining_label(pending: &[voting::VotingNextStep]) -> String {
    if pending.is_empty() {
        return "All steps already confirmed".to_string();
    }
    if pending.iter().all(|s| s.kind == "submit_shares") {
        return "Waiting for the share window".to_string();
    }
    if pending.iter().all(|s| s.kind == "confirm_share") {
        return "Waiting for share confirmations".to_string();
    }
    "Waiting for the next step".to_string()
}

/// Injects the `vote_round_id` field the helper API requires into a share
/// wire JSON body.
fn inject_vote_round_id(wire_json: &str, round_id: &str) -> Result<String> {
    let mut decoded: serde_json::Value = serde_json::from_str(wire_json)?;
    decoded["vote_round_id"] = serde_json::Value::String(round_id.to_string());
    Ok(decoded.to_string())
}

/// Reads the persisted draft ballot for a round (props table), if any.
async fn drafts_json(c: &Coin, round_id: &str) -> Result<Option<String>> {
    let mut connection = c.get_connection().await?;
    Ok(crate::db::get_prop(
        &mut connection,
        &format!("voting_drafts:{round_id}"),
    )
    .await?)
}

/// Resolves the PIR server URL and layout: input wins, then the saved props
/// (`voting_round_pir_url:{round}` / `voting_round_pir:{round}`), saving
/// whichever the input supplies so a restart resumes without re-reading the
/// chain config (mirrors `delegation_build_submission`'s resolution).
async fn resolve_pir(
    c: &Coin,
    round_id: &str,
    input: &VoteRoundInput,
) -> Result<(String, zcash_voting::config::PirLayout)> {
    let mut connection = c.get_connection().await?;
    let pir_server_url = if input.pir_server_url.is_empty() {
        crate::db::get_prop(
            &mut connection,
            &format!("voting_round_pir_url:{round_id}"),
        )
        .await?
        .ok_or_else(|| {
            anyhow!("no saved PIR server URL for round {round_id}; pass pir_server_url once")
        })?
    } else {
        crate::db::put_prop(
            &mut connection,
            &format!("voting_round_pir_url:{round_id}"),
            &input.pir_server_url,
        )
        .await?;
        input.pir_server_url.clone()
    };
    let pir_layout = match &input.pir_layout {
        Some(layout) => {
            vc::save_pir_layout(&mut connection, round_id, &layout.to_fork()).await?;
            layout.to_fork()
        }
        None => vc::load_pir_layout(&mut connection, round_id)
            .await?
            .ok_or_else(|| {
                anyhow!("no saved PIR layout for round {round_id}; pass pir_layout once")
            })?,
    };
    Ok((pir_server_url, pir_layout))
}

/// Reconstructs the wallet `Coin` from the workflow input.
fn coin_from_input(input: &VoteRoundInput) -> Coin {
    Coin {
        coin: input.coin,
        account: input.account,
        db_filepath: input.db_filepath.clone(),
        url: input.url.clone(),
        server_type: input.server_type,
        transport: input.transport,
        proxy: input.proxy.clone(),
    }
}

/// Reads the wallet id for the workflow's account.
async fn wallet_id_of(c: &Coin, account: u32) -> Result<String> {
    let mut connection = c.get_connection().await?;
    vc::voting_wallet_id(&mut connection, account).await
}

// ---------------------------------------------------------------------------
// Step output types (cross the durare checkpoint boundary)
// ---------------------------------------------------------------------------

/// Next delegation unit of work, or `None` when delegation is complete.
#[derive(Clone, Debug, Serialize, Deserialize)]
enum DelegationWork {
    Prepare { bundle_index: u32 },
    Poll { bundle_index: u32 },
}

/// One unit of vote work, in the fork plan's order.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct VoteWorkItem {
    kind: String,
    bundle_index: u32,
    proposal_id: u32,
}

/// The share phase's resolved timing + first-pass submission plan.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ShareWork {
    ceremony_start: u64,
    vote_end: Option<u64>,
    now: u64,
    payloads: Vec<voting::VotingShareSubmissionPayload>,
    plans: Vec<voting::VotingSharePlanItem>,
    unconfirmed_exist: bool,
}

/// One tracking-loop tick result.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ShareTick {
    unconfirmed: u64,
    next_delay: Option<u64>,
}

// ---------------------------------------------------------------------------
// The durable workflow
// ---------------------------------------------------------------------------

/// Runs one round's voting lifecycle as a durable workflow. Checkpointed per
/// step; resumes from the last checkpoint after a crash (the fork's `voting_*`
/// tables hold the artifacts, durare holds the progress).
#[durare::workflow]
async fn vote_round(ctx: DurableContext, input: VoteRoundInput) -> durare::Result<VoteRoundOutput> {
    let round_id = input.round_id.clone();
    let c = coin_from_input(&input);
    let chain = resolve_chain(&input);

    // --- Delegation phase ---------------------------------------------------
    write_progress(&ctx, &round_id, "running", None, None).await?;

    ctx.step("ensure_config", || async {
        let mut connection = c.get_connection().await.map_err(durare_err)?;
        match &input.round_params_json {
            Some(params) => {
                let name = input.round_name.clone().unwrap_or_default();
                let lwd = input.lightwalletd_url.clone().unwrap_or_default();
                vc::save_round_config(
                    &mut connection,
                    &round_id,
                    params,
                    &name,
                    input.max_real_notes_per_bundle,
                    &lwd,
                )
                .await
                .map_err(durare_err)
            }
            None => vc::load_round_config(&mut connection, &round_id)
                .await
                .map(|_| ())
                .map_err(durare_err),
        }
    })
    .await?;

    loop {
        let work = ctx
            .step("delegation_work", || async {
                let draft_ids = drafts_json(&c, &round_id)
                    .await
                    .map(|d| d.as_deref().map(draft_proposal_ids).unwrap_or_default())
                    .map_err(durare_err)?;
                let plan = voting::voting_plan(&round_id, draft_ids, &c)
                    .await
                    .map_err(durare_err)?;
                if !plan.hotkey_bound {
                    // The delegation keys embed an app-owned voting hotkey;
                    // auto-create one when missing and the round isn't bound.
                    let mut connection = c.get_connection().await.map_err(durare_err)?;
                    let network = vc::voting_network(&c.network()).map_err(durare_err)?;
                    if crate::db::get_prop(&mut connection, vc::VOTING_HOTKEY_PROP)
                        .await
                        .map_err(durare_err)?
                        .is_none()
                    {
                        vc::voting_hotkey_create(&mut connection, network)
                            .await
                            .map_err(durare_err)?;
                    }
                }
                let delegate = plan
                    .next_steps
                    .iter()
                    .find(|s| s.kind == "delegate")
                    .map(|s| s.bundle_index);
                let poll = plan
                    .next_steps
                    .iter()
                    .find(|s| s.kind == "poll_delegation")
                    .map(|s| s.bundle_index);
                match (delegate, poll) {
                    (Some(bundle_index), _) => Ok(Some(DelegationWork::Prepare { bundle_index })),
                    (None, Some(bundle_index)) => Ok(Some(DelegationWork::Poll { bundle_index })),
                    _ => {
                        // A fresh round has no plan steps at all; the fork flags
                        // it with needs_draft_setup. Treat that as delegation
                        // work for the first bundle that still needs it.
                        if plan.needs_draft_setup {
                            let pending = plan
                                .delegation_statuses
                                .iter()
                                .find(|s| s.phase == "prepared" || s.phase == "committed")
                                .map(|s| s.bundle_index);
                            if plan.delegation_statuses.is_empty() || pending.is_some() {
                                return Ok(Some(DelegationWork::Prepare {
                                    bundle_index: pending.unwrap_or(0),
                                }));
                            }
                        }
                        Ok(None)
                    }
                }
            })
            .await?;

        let Some(work) = work else { break };

        match work {
            DelegationWork::Prepare { bundle_index } => {
                write_progress(
                    &ctx,
                    &round_id,
                    "preparing",
                    None,
                    Some(format!("bundle {bundle_index}")),
                )
                .await?;
                ctx.step("prepare_delegation", || async {
                    // Skip when the bundle already has a recorded submission
                    // (crash between broadcast and confirmation).
                    let recorded = delegation_recorded_tx(&c, &round_id, bundle_index)
                        .await
                        .map_err(durare_err)?;
                    if recorded.is_some() {
                        return Ok(());
                    }
                    // Fresh prepare from input params, or resume from the saved
                    // round config after a restart (the prepared-bundle cache is
                    // process-local).
                    match &input.round_params_json {
                        Some(params) => {
                            let name = input.round_name.clone().unwrap_or_default();
                            let lwd = input.lightwalletd_url.clone().unwrap_or_default();
                            voting::prepare_bundle(
                                params,
                                &name,
                                None,
                                bundle_index,
                                input.max_real_notes_per_bundle,
                                &lwd,
                                &c,
                            )
                            .await
                            .map(|_| ())
                            .map_err(durare_err)
                        }
                        None => {
                            let mut connection =
                                c.get_connection().await.map_err(durare_err)?;
                            let (params, name, policy, lwd) =
                                vc::load_round_config(&mut connection, &round_id)
                                    .await
                                    .map_err(durare_err)?;
                            drop(connection);
                            voting::prepare_bundle(
                                &params,
                                &name,
                                None,
                                bundle_index,
                                input.max_real_notes_per_bundle.or(policy),
                                &lwd,
                                &c,
                            )
                            .await
                            .map(|_| ())
                            .map_err(durare_err)
                        }
                    }
                })
                .await?;

                write_progress(&ctx, &round_id, "proving", None, None).await?;
                let tx_hash = ctx
                    .step("build_and_submit_delegation", || async {
                        // Skip the whole build when a tx hash was already
                        // recorded (crash after broadcast).
                        let recorded = delegation_recorded_tx(&c, &round_id, bundle_index)
                            .await
                            .map_err(durare_err)?;
                        if let Some(tx) = recorded {
                            return Ok(tx);
                        }

                        let (pir_server_url, pir_layout) =
                            resolve_pir(&c, &round_id, &input).await.map_err(durare_err)?;
                        let wallet_id = wallet_id_of(&c, input.account)
                            .await
                            .map_err(durare_err)?;
                        let seed = {
                            let mut connection =
                                c.get_connection().await.map_err(durare_err)?;
                            vc::account_seed(&mut connection, input.account)
                                .await
                                .map_err(durare_err)?
                        };
                        let prepared = vc::load_prepared_bundle(
                            &wallet_id,
                            &round_id,
                            bundle_index,
                        )
                        .map_err(durare_err)?;

                        let progress = Arc::new(DelegationProgressBridge::new({
                            let round_id = round_id.clone();
                            move |p: DelegationProgress| {
                                if let DelegationProgress::ProofProgress(progress) = p {
                                    set_live_progress(&round_id, "proving", progress)
                                }
                            }
                        }));
                        let (_, wire_json) = loop {
                            match vc::prove_and_submit_delegation_with_progress(
                                c.get_pool().map_err(durare_err)?,
                                &wallet_id,
                                &prepared,
                                &seed,
                                Vec::new(),
                                pir_layout.clone(),
                                &pir_server_url,
                                progress.clone(),
                            )
                            .await
                            {
                                Ok(v) => break v,
                                Err(e) if e.to_string().contains("refusing to overwrite") => {
                                    // A restart after a partial run leaves a
                                    // stored sighash that the internal setup
                                    // refuses to overwrite; reset and retry once.
                                    voting::voting_reset_session_state(&round_id, &c)
                                        .await
                                        .map_err(durare_err)?;
                                    continue;
                                }
                                Err(e) => return Err(durare_err(e)),
                            }
                        };
                        let (status_code, body) = chain
                            .submit_delegation(input.chain_url.clone(), wire_json)
                            .await
                            .map_err(durare_err)?;
                        if status_code == 422 {
                            return Err(durare::Error::app(format!(
                                "Delegation rejected by the vote chain: {body}"
                            )));
                        }
                        if status_code < 200 || status_code >= 300 {
                            return Err(durare::Error::app(format!(
                                "Vote chain submit failed (HTTP {status_code}): {body}"
                            )));
                        }
                        let tx_hash = parse_submit_response(&body).map_err(durare_err)?;
                        voting::delegation_mark_submitted(&round_id, bundle_index, &tx_hash, &c)
                            .await
                            .map_err(durare_err)?;
                        Ok(tx_hash)
                    })
                    .await?;

                write_progress(&ctx, &round_id, "confirming", None, None).await?;
                let conf = poll_delegation_confirmation(&ctx, &input, &chain, &tx_hash).await?;
                confirm_and_verify_delegation(
                    &ctx,
                    &input,
                    &c,
                    &round_id,
                    bundle_index,
                    &tx_hash,
                    &conf,
                )
                .await?;
                write_confirmation(&ctx, &round_id, conf.height, tx_hash.clone()).await?;
            }
            DelegationWork::Poll { bundle_index } => {
                write_progress(&ctx, &round_id, "confirming", None, None).await?;
                let tx_hash = ctx
                    .step("delegation_tx_hash", || async {
                        let tx = delegation_recorded_tx(&c, &round_id, bundle_index)
                            .await
                            .map_err(durare_err)?
                            .ok_or_else(|| {
                                durare::Error::app(format!(
                                    "Round {round_id} bundle {bundle_index} is pending but has no recorded tx hash"
                                ))
                            })?;
                        Ok(tx)
                    })
                    .await?;
                let conf = poll_delegation_confirmation(&ctx, &input, &chain, &tx_hash).await?;
                confirm_and_verify_delegation(
                    &ctx,
                    &input,
                    &c,
                    &round_id,
                    bundle_index,
                    &tx_hash,
                    &conf,
                )
                .await?;
                write_confirmation(&ctx, &round_id, conf.height, tx_hash).await?;
            }
        }
    }

    // --- Vote phase --------------------------------------------------------
    write_progress(&ctx, &round_id, "voting", None, None).await?;
    let work_items: Vec<VoteWorkItem> = ctx
        .step("vote_work", || async {
            // Durable ballot intents first (mirrors vizor's writeBallotIntents):
            // recovery can resume from the right choice if the app dies mid-vote.
            let drafts = drafts_json(&c, &round_id).await.map_err(durare_err)?;
            if let Some(drafts) = &drafts {
                let decoded: Vec<serde_json::Value> =
                    serde_json::from_str(drafts).map_err(durare_err)?;
                for d in decoded {
                    let proposal_id =
                        d.get("proposal_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let choice = d.get("choice").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let num_options =
                        d.get("num_options").and_then(|v| v.as_u64()).unwrap_or(2) as u32;
                    let skipped = choice == num_options;
                    voting::voting_set_ballot_intent(
                        &round_id,
                        proposal_id,
                        skipped,
                        if skipped { 0 } else { choice },
                        num_options,
                        &c,
                    )
                    .await
                    .map_err(durare_err)?;
                }
            }
            // With intents written, the plan now carries the cast steps.
            let draft_ids = drafts
                .as_deref()
                .map(draft_proposal_ids)
                .unwrap_or_default();
            let plan = voting::voting_plan(&round_id, draft_ids, &c)
                .await
                .map_err(durare_err)?;
            // Defer votes for bundles whose delegation still needs to run.
            let delegate_bundles: std::collections::BTreeSet<u32> = plan
                .next_steps
                .iter()
                .filter(|s| s.kind == "delegate")
                .map(|s| s.bundle_index)
                .collect();
            Ok(plan
                .next_steps
                .iter()
                .filter(|s| {
                    (s.kind == "cast_vote" || s.kind == "submit_vote" || s.kind == "poll_vote")
                        && !delegate_bundles.contains(&s.bundle_index)
                })
                .map(|s| VoteWorkItem {
                    kind: s.kind.clone(),
                    bundle_index: s.bundle_index,
                    proposal_id: s.proposal_id,
                })
                .collect())
        })
        .await?;

    if !work_items.is_empty() {
        let commit_drafts_json = drafts_json(&c, &round_id).await.map_err(durare_err)?;
        let commit_drafts_json = commit_drafts_json
            .as_deref()
            .and_then(sanitized_commit_drafts);
        if work_items.iter().any(|w| w.kind == "cast_vote") && commit_drafts_json.is_none() {
            return Err(durare::Error::app(format!(
                "No draft ballot saved for round {round_id}; open the ballot and review first"
            )));
        }

        // Cast ONE proposal at a time and submit it before the next cast: the
        // fork derives the next VAN from the submission state (the
        // proposal-authority mask clears per recorded tx_hash), so the VAN
        // chaining only works when builds interleave with submissions.
        for item in work_items.iter().filter(|w| w.kind == "cast_vote") {
            write_progress(
                &ctx,
                &round_id,
                "voting",
                Some(0.0),
                Some(format!("proposal {}", item.proposal_id)),
            )
            .await?;
            let drafts = commit_drafts_json.clone().unwrap_or_else(|| "[]".to_string());
            ctx.step("cast_vote", || async {
                let decoded: Vec<serde_json::Value> =
                    serde_json::from_str(&drafts).map_err(durare_err)?;
                let draft = decoded
                    .iter()
                    .find(|d| {
                        d.get("proposal_id").and_then(|v| v.as_u64())
                            == Some(item.proposal_id as u64)
                    })
                    .ok_or_else(|| {
                        durare::Error::app(format!(
                            "No draft for proposal {} in round {round_id}",
                            item.proposal_id
                        ))
                    })?;
                let drafts: Vec<DraftVote> = serde_json::from_value(draft.clone())
                    .map_err(durare_err)?;
                let (wallet_id, hotkey) = {
                    let mut connection = c.get_connection().await.map_err(durare_err)?;
                    let wallet_id = vc::voting_wallet_id(&mut connection, input.account)
                        .await
                        .map_err(durare_err)?;
                    let hotkey = vc::voting_hotkey_load(
                        &mut connection,
                        vc::voting_network(&c.network()).map_err(durare_err)?,
                    )
                    .await
                    .map_err(durare_err)?;
                    (wallet_id, hotkey)
                };
                let witness = vc::vote_van_witness(
                    c.get_pool().map_err(durare_err)?,
                    &wallet_id,
                    &round_id,
                    item.bundle_index,
                    &input.vote_node_url,
                )
                .await
                .map_err(durare_err)?;
                let stages = VoteCommitStageBridge::new({
                    let round_id = round_id.clone();
                    move |s: zcash_voting::vote::VoteCommitStage| {
                        if let zcash_voting::vote::VoteCommitStage::ProofProgress { progress, .. } =
                            s
                        {
                            set_live_progress(&round_id, "voting", progress)
                        }
                    }
                });
                vc::commit_votes_with_progress(
                    c.get_pool().map_err(durare_err)?,
                    &wallet_id,
                    &round_id,
                    item.bundle_index,
                    &drafts,
                    &witness,
                    &hotkey,
                    &stages,
                )
                .await
                .map_err(durare_err)?;
                Ok(())
            })
            .await?;
            submit_and_confirm_vote(&ctx, &input, &c, &chain, &round_id, item).await?;
        }
        for item in work_items.iter().filter(|w| w.kind == "submit_vote") {
            submit_and_confirm_vote(&ctx, &input, &c, &chain, &round_id, item).await?;
        }
        for item in work_items.iter().filter(|w| w.kind == "poll_vote") {
            write_progress(&ctx, &round_id, "confirming", None, None).await?;
            let tx_hash = ctx
                .step("vote_tx_hash", || async {
                    let recovery = voting::voting_recovery(&round_id, &c)
                        .await
                        .map_err(durare_err)?;
                    let tx = recovery
                        .votes
                        .iter()
                        .find(|v| {
                            v.bundle_index == item.bundle_index
                                && v.proposal_id == item.proposal_id
                        })
                        .and_then(|v| v.tx_hash.clone())
                        .filter(|t| !t.is_empty())
                        .ok_or_else(|| {
                            durare::Error::app(format!(
                                "Vote for proposal {} is pending but has no recorded tx hash",
                                item.proposal_id
                            ))
                        })?;
                    Ok(tx)
                })
                .await?;
            let conf = poll_tx_confirmation(&ctx, &input, &chain, &tx_hash, "poll_vote").await?;
            confirm_vote(&ctx, &input, &c, &round_id, item, &tx_hash, &conf).await?;
            write_confirmation(&ctx, &round_id, conf.height, tx_hash).await?;
        }
    }

    // --- Share phase -------------------------------------------------------
    write_progress(&ctx, &round_id, "shares", None, None).await?;
    let share_work = ctx
        .step("share_work", || async {
            // Timing: the voting pages never pass ceremony/vote_end — resolve
            // them from the chain round status when absent.
            let (ceremony_start, vote_end) = if input.ceremony_start == 0
                || input.vote_end.is_none()
            {
                let (status_code, body) = chain
                    .round_status(input.chain_url.clone(), round_id.clone())
                    .await
                    .map_err(durare_err)?;
                if status_code >= 200 && status_code < 300 {
                    let decoded: serde_json::Value =
                        serde_json::from_str(&body).map_err(durare_err)?;
                    let round = decoded.get("round");
                    let ceremony = round
                        .and_then(|r| r.get("ceremony_phase_start"))
                        .and_then(|v| v.as_u64());
                    let end = round
                        .and_then(|r| r.get("vote_end_time"))
                        .and_then(|v| v.as_u64());
                    match (ceremony, end) {
                        (Some(ceremony), Some(end)) => (ceremony, Some(end)),
                        _ => (input.ceremony_start, input.vote_end),
                    }
                } else {
                    (input.ceremony_start, input.vote_end)
                }
            } else {
                (input.ceremony_start, input.vote_end)
            };
            let now = now_epoch_secs();

            // First-pass source: the confirmed votes' share payloads — the
            // share delegation rows only exist after a submission records them.
            let payloads = voting::voting_share_payloads(&round_id, &c)
                .await
                .map_err(durare_err)?;
            let (plans, unconfirmed_exist) = if payloads.is_empty() {
                let unconfirmed = voting::voting_share_unconfirmed(&round_id, &c)
                    .await
                    .map_err(durare_err)?;
                (Vec::new(), !unconfirmed.is_empty())
            } else {
                let Some(vote_end) = vote_end else {
                    // No vote window: nothing can be scheduled.
                    return Ok(ShareWork {
                        ceremony_start,
                        vote_end,
                        now,
                        payloads,
                        plans: Vec::new(),
                        unconfirmed_exist: false,
                    });
                };
                let plans = voting::voting_share_plans(
                    payloads.len() as u32,
                    input.share_server_urls.clone(),
                    now,
                    vote_end,
                    ceremony_start,
                    input.single_share,
                    &c,
                )
                .await
                .map_err(durare_err)?;
                (plans, false)
            };
            Ok(ShareWork {
                ceremony_start,
                vote_end,
                now,
                payloads,
                plans,
                unconfirmed_exist,
            })
        })
        .await?;

    if share_work.vote_end.is_some() && !input.share_server_urls.is_empty() {
        if !share_work.payloads.is_empty() {
            // Pair payloads with their plans index-wise (same as the Dart
            // driver); sleep until each planned submit_at before posting.
            for (payload, plan) in share_work
                .payloads
                .iter()
                .zip(share_work.plans.iter())
            {
                if plan.submit_at > share_work.now {
                    ctx.sleep(Duration::from_secs(plan.submit_at - share_work.now))
                        .await?;
                }
                write_progress(
                    &ctx,
                    &round_id,
                    "shares",
                    None,
                    Some(format!(
                        "share {} of proposal {}",
                        payload.share_index, payload.proposal_id
                    )),
                )
                .await?;
                ctx.step("submit_share", || async {
                    let wallet_id = wallet_id_of(&c, input.account)
                        .await
                        .map_err(durare_err)?;
                    let wire = vc::share_wire_json(
                        c.get_pool().map_err(durare_err)?,
                        &wallet_id,
                        &round_id,
                        payload.bundle_index,
                        payload.proposal_id,
                        payload.share_index,
                        payload.vc_tree_position,
                        plan.submit_at,
                    )
                    .await
                    .map_err(durare_err)?;
                    let body = inject_vote_round_id(&wire, &round_id).map_err(durare_err)?;
                    for server in &plan.target_servers {
                        let (status_code, res_body) = chain
                            .submit_share(server.clone(), body.clone())
                            .await
                            .map_err(durare_err)?;
                        if status_code < 200 || status_code >= 300 {
                            return Err(durare::Error::app(format!(
                                "Share submit to {server} failed (HTTP {status_code}): {res_body}"
                            )));
                        }
                    }
                    voting::voting_share_record(
                        &round_id,
                        payload.bundle_index,
                        payload.proposal_id,
                        payload.share_index,
                        plan.target_servers.clone(),
                        plan.submit_at,
                        &c,
                    )
                    .await
                    .map_err(durare_err)?;
                    Ok(())
                })
                .await?;
            }
        }

        // Tracking loop: poll the helpers until every share confirms (or the
        // vote window ends). Replaces the Dart one-shot re-arming timer with a
        // durable loop — a crash between ticks resumes from the checkpoint.
        loop {
            let tick = ctx
                .step("tracking_tick", || async {
                    let now = now_epoch_secs();
                    let plan = voting::voting_share_plan(
                        &round_id,
                        now,
                        share_work.ceremony_start,
                        share_work.vote_end,
                        input.share_server_urls.clone(),
                        input.single_share,
                        &c,
                    )
                    .await
                    .map_err(durare_err)?;
                    let unconfirmed = voting::voting_share_unconfirmed(&round_id, &c)
                        .await
                        .map_err(durare_err)?;
                    for share in &unconfirmed {
                        if share.sent_to_urls.is_empty() {
                            continue;
                        }
                        let share_id = hex::encode(&share.nullifier);
                        let (status_code, _) = chain
                            .share_status(
                                share.sent_to_urls[0].clone(),
                                round_id.clone(),
                                share_id,
                            )
                            .await
                            .map_err(durare_err)?;
                        if status_code == 200 {
                            voting::voting_share_confirm(
                                &round_id,
                                share.bundle_index,
                                share.proposal_id,
                                share.share_index,
                                &c,
                            )
                            .await
                            .map_err(durare_err)?;
                        }
                    }
                    // Resubmit overdue shares with fresh plans.
                    if plan.summary.overdue > 0 {
                        for (i, item) in plan
                            .submissions
                            .iter()
                            .enumerate()
                            .take(unconfirmed.len())
                        {
                            let share = &unconfirmed[i];
                            let wallet_id = wallet_id_of(&c, input.account)
                                .await
                                .map_err(durare_err)?;
                            let wire = vc::share_wire_json(
                                c.get_pool().map_err(durare_err)?,
                                &wallet_id,
                                &round_id,
                                share.bundle_index,
                                share.proposal_id,
                                share.share_index,
                                None,
                                item.submit_at,
                            )
                            .await
                            .map_err(durare_err)?;
                            let body =
                                inject_vote_round_id(&wire, &round_id).map_err(durare_err)?;
                            for server in &item.target_servers {
                                let (status_code, res_body) = chain
                                    .submit_share(server.clone(), body.clone())
                                    .await
                                    .map_err(durare_err)?;
                                if status_code < 200 || status_code >= 300 {
                                    return Err(durare::Error::app(format!(
                                        "Share resubmit to {server} failed (HTTP {status_code}): {res_body}"
                                    )));
                                }
                            }
                            voting::voting_share_add_servers(
                                &round_id,
                                share.bundle_index,
                                share.proposal_id,
                                share.share_index,
                                item.target_servers.clone(),
                                &c,
                            )
                            .await
                            .map_err(durare_err)?;
                        }
                    }
                    Ok(ShareTick {
                        unconfirmed: unconfirmed.len() as u64,
                        next_delay: plan.next_tracking_delay_secs,
                    })
                })
                .await?;

            if tick.unconfirmed == 0 {
                break;
            }
            // Mirror the Dart driver: tracking stops when the vote end is
            // unknown or the plan reports no next delay.
            let Some(delay) = tick.next_delay.filter(|&d| d > 0) else {
                break;
            };
            ctx.sleep(Duration::from_secs(delay)).await?;
        }
    }

    // --- Summary -----------------------------------------------------------
    let output = ctx
        .step("final_summary", || async {
            let draft_ids = drafts_json(&c, &round_id)
                .await
                .map(|d| d.as_deref().map(draft_proposal_ids).unwrap_or_default())
                .map_err(durare_err)?;
            let plan = voting::voting_plan(&round_id, draft_ids, &c)
                .await
                .map_err(durare_err)?;
            let recovery = voting::voting_recovery(&round_id, &c)
                .await
                .map_err(durare_err)?;
            let delegated = !recovery.delegation.is_empty()
                && recovery.delegation.iter().all(|d| d.phase == "confirmed");
            let voted = recovery.votes.iter().any(|v| v.phase == "confirmed");
            let shared = recovery.share_delegations.iter().any(|s| s.confirmed);
            let tx_hash = recovery
                .delegation
                .first()
                .and_then(|d| d.tx_hash.clone())
                .or_else(|| {
                    recovery
                        .votes
                        .iter()
                        .find(|v| v.tx_hash.is_some())
                        .and_then(|v| v.tx_hash.clone())
                });
            let eligible_weight_zatoshi = voting::voting_rounds(&c)
                .await
                .map_err(durare_err)?
                .into_iter()
                .find(|r| r.round_id == round_id)
                .and_then(|r| r.eligible_weight_zatoshi);
            let done_label = if delegated {
                "Delegation confirmed".to_string()
            } else if voted {
                "Votes submitted".to_string()
            } else if shared {
                "Shares submitted".to_string()
            } else {
                remaining_label(&plan.next_steps)
            };
            Ok(VoteRoundOutput {
                delegated,
                voted,
                shared,
                done_label,
                tx_hash,
                confirm_height: None,
                eligible_weight_zatoshi,
            })
        })
        .await?;

    write_progress(
        &ctx,
        &round_id,
        "done",
        Some(1.0),
        Some(output.done_label.clone()),
    )
    .await?;
    clear_live_progress(&round_id);
    Ok(output)
}

// ---------------------------------------------------------------------------
// Workflow helpers
// ---------------------------------------------------------------------------

fn progress_event(stage: &str, progress: Option<f64>, message: Option<String>) -> WorkflowEvent {
    WorkflowEvent {
        stage: stage.to_string(),
        progress,
        message,
        confirm_height: None,
        tx_hash: None,
    }
}

impl WorkflowEvent {
    fn with_height(mut self, height: u64) -> Self {
        self.confirm_height = Some(height);
        self
    }
    fn with_tx(mut self, tx_hash: String) -> Self {
        self.tx_hash = Some(tx_hash);
        self
    }
}

/// Body-level durable progress event (stream writes are recorded as steps, so
/// they must never happen inside a step closure — durare #173).
async fn write_progress(
    ctx: &DurableContext,
    _round_id: &str,
    stage: &str,
    progress: Option<f64>,
    message: Option<String>,
) -> durare::Result<()> {
    ctx.write_stream("progress", progress_event(stage, progress, message))
        .await
}

/// Body-level "confirming" event carrying the confirmed tx + height.
async fn write_confirmation(
    ctx: &DurableContext,
    _round_id: &str,
    height: u64,
    tx_hash: String,
) -> durare::Result<()> {
    ctx.write_stream(
        "progress",
        progress_event("confirming", Some(1.0), Some(tx_hash.clone()))
            .with_height(height)
            .with_tx(tx_hash),
    )
    .await
}

/// Returns the recorded delegation tx hash for a bundle, if any.
async fn delegation_recorded_tx(
    c: &Coin,
    round_id: &str,
    bundle_index: u32,
) -> Result<Option<String>> {
    let draft_ids = drafts_json(c, round_id)
        .await
        .map(|d| d.as_deref().map(draft_proposal_ids).unwrap_or_default())?;
    let plan = voting::voting_plan(round_id, draft_ids, c).await?;
    Ok(plan
        .delegation_statuses
        .iter()
        .find(|s| s.bundle_index == bundle_index)
        .and_then(|s| s.tx_hash.clone())
        .filter(|t| !t.is_empty()))
}

/// `step_with`-based delegation confirmation poll: 45 attempts × 2s, matching
/// the old Dart driver's `_pollTxConfirmation` budget.
async fn poll_delegation_confirmation(
    ctx: &DurableContext,
    input: &VoteRoundInput,
    chain: &Arc<dyn vc::VoteChainClient>,
    tx_hash: &str,
) -> durare::Result<TxConfirmation> {
    let tx_hash = tx_hash.to_string();
    let chain_url = input.chain_url.clone();
    ctx.step_with(
        StepOptions::new("poll_delegation_confirmation")
            .max_retries(44)
            .backoff_factor(1.0)
            .base_interval(Duration::from_secs(2))
            .max_interval(Duration::from_secs(2)),
        || async {
            let (status_code, body) = chain
                .tx_confirmation(chain_url.clone(), tx_hash.clone())
                .await
                .map_err(durare_err)?;
            if status_code == 200 {
                if let Some(conf) = parse_tx_confirmation(&body) {
                    return Ok(conf);
                }
            }
            Err(durare::Error::app(format!("tx {tx_hash} not confirmed yet")))
        },
    )
    .await
}

/// Records a confirmed delegation and verifies the bundle reads back as
/// confirmed (tx hash + VAN leaf) before claiming success — the done state
/// must be backed by the fork's recorded confirmation.
async fn confirm_and_verify_delegation(
    ctx: &DurableContext,
    input: &VoteRoundInput,
    c: &Coin,
    round_id: &str,
    bundle_index: u32,
    tx_hash: &str,
    conf: &TxConfirmation,
) -> durare::Result<()> {
    let tx_hash = tx_hash.to_string();
    let events_json = conf.events_json.clone();
    let round_id = round_id.to_string();
    let bundle_index = bundle_index;
    ctx.step("confirm_delegation", || async {
        let events: Vec<TxEvent> = serde_json::from_str(&events_json).map_err(durare_err)?;
        let wallet_id = wallet_id_of(c, input.account).await.map_err(durare_err)?;
        vc::confirm_delegation(
            c.get_pool().map_err(durare_err)?,
            &wallet_id,
            &round_id,
            bundle_index,
            &tx_hash,
            &events,
        )
        .await
        .map_err(durare_err)?;
        let recorded = delegation_recorded_tx(c, &round_id, bundle_index)
            .await
            .map_err(durare_err)?;
        if recorded.is_none() {
            return Err(durare::Error::app(format!(
                "Delegation confirmation was not recorded for round {round_id} bundle {bundle_index}"
            )));
        }
        Ok(())
    })
    .await
}

/// Submits a committed vote and confirms it: wire JSON → chain → record tx
/// hash → poll → confirm. Used by both `cast_vote` and `submit_vote` items.
async fn submit_and_confirm_vote(
    ctx: &DurableContext,
    input: &VoteRoundInput,
    c: &Coin,
    chain: &Arc<dyn vc::VoteChainClient>,
    round_id: &str,
    item: &VoteWorkItem,
) -> durare::Result<()> {
    write_progress(
        ctx,
        round_id,
        "voting",
        Some(0.0),
        Some(format!("proposal {}", item.proposal_id)),
    )
    .await?;
    let tx_hash = ctx
        .step("submit_vote", || async {
            let wallet_id = wallet_id_of(c, input.account).await.map_err(durare_err)?;
            let wire = vc::vote_wire_json(
                c.get_pool().map_err(durare_err)?,
                &wallet_id,
                round_id,
                item.bundle_index,
                item.proposal_id,
            )
            .await
            .map_err(durare_err)?;
            let (status_code, body) = chain
                .submit_vote(input.chain_url.clone(), wire)
                .await
                .map_err(durare_err)?;
            if status_code == 422 {
                return Err(durare::Error::app(format!(
                    "Vote rejected by the vote chain: {body}"
                )));
            }
            if status_code < 200 || status_code >= 300 {
                return Err(durare::Error::app(format!(
                    "Vote chain submit failed (HTTP {status_code}): {body}"
                )));
            }
            let tx_hash = parse_submit_response(&body).map_err(durare_err)?;
            voting::voting_mark_vote_submitted(
                round_id,
                item.bundle_index,
                item.proposal_id,
                &tx_hash,
                c,
            )
            .await
            .map_err(durare_err)?;
            Ok(tx_hash)
        })
        .await?;
    let conf = poll_tx_confirmation(ctx, input, chain, &tx_hash, "poll_vote").await?;
    confirm_vote(ctx, input, c, round_id, item, &tx_hash, &conf).await?;
    write_confirmation(ctx, round_id, conf.height, tx_hash).await?;
    Ok(())
}

/// `step_with`-based tx confirmation poll: 45 attempts × 2s, matching the old
/// Dart driver's `_pollTxConfirmation` budget.
async fn poll_tx_confirmation(
    ctx: &DurableContext,
    input: &VoteRoundInput,
    chain: &Arc<dyn vc::VoteChainClient>,
    tx_hash: &str,
    step_name: &str,
) -> durare::Result<TxConfirmation> {
    let tx_hash = tx_hash.to_string();
    let chain_url = input.chain_url.clone();
    ctx.step_with(
        StepOptions::new(step_name)
            .max_retries(44)
            .backoff_factor(1.0)
            .base_interval(Duration::from_secs(2))
            .max_interval(Duration::from_secs(2)),
        || async {
            let (status_code, body) = chain
                .tx_confirmation(chain_url.clone(), tx_hash.clone())
                .await
                .map_err(durare_err)?;
            if status_code == 200 {
                if let Some(conf) = parse_tx_confirmation(&body) {
                    return Ok(conf);
                }
            }
            Err(durare::Error::app(format!("tx {tx_hash} not confirmed yet")))
        },
    )
    .await
}

async fn confirm_vote(
    ctx: &DurableContext,
    input: &VoteRoundInput,
    c: &Coin,
    round_id: &str,
    item: &VoteWorkItem,
    tx_hash: &str,
    conf: &TxConfirmation,
) -> durare::Result<()> {
    let tx_hash = tx_hash.to_string();
    let events_json = conf.events_json.clone();
    let round_id = round_id.to_string();
    let bundle_index = item.bundle_index;
    let proposal_id = item.proposal_id;
    ctx.step("confirm_vote", || async {
        let events: Vec<TxEvent> = serde_json::from_str(&events_json).map_err(durare_err)?;
        let wallet_id = wallet_id_of(c, input.account).await.map_err(durare_err)?;
        vc::confirm_vote(
            c.get_pool().map_err(durare_err)?,
            &wallet_id,
            &round_id,
            bundle_index,
            proposal_id,
            &tx_hash,
            &events,
        )
        .await
        .map_err(durare_err)?;
        Ok(())
    })
    .await
}

// ---------------------------------------------------------------------------
// FRB surface
// ---------------------------------------------------------------------------

const STATUS_SUCCESS: &str = "SUCCESS";
const STATUS_ERROR: &str = "ERROR";
const STATUS_CANCELLED: &str = "CANCELLED";
const STATUS_PENDING: &str = "PENDING";
const STATUS_ENQUEUED: &str = "ENQUEUED";
const STATUS_DELAYED: &str = "DELAYED";
const STATUS_MAX_RECOVERY_ATTEMPTS_EXCEEDED: &str = "MAX_RECOVERY_ATTEMPTS_EXCEEDED";

fn is_terminal_status(status: &str) -> bool {
    matches!(
        status,
        STATUS_SUCCESS
            | STATUS_ERROR
            | STATUS_CANCELLED
            | STATUS_MAX_RECOVERY_ATTEMPTS_EXCEEDED
    )
}

/// Starts (or attaches to) the durable workflow for a round. Idempotent: when
/// a workflow row already exists for `round_id` it is returned instead of
/// starting a second run (durare would otherwise spawn a concurrent duplicate).
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_workflow_start(
    input: VoteRoundInput,
    c: &Coin,
) -> Result<VotingWorkflowStatus> {
    let engine = ensure_engine(c).await?;
    let round_id = input.round_id.clone();
    match read_workflow_row(c, &round_id).await? {
        Some((status, _, _, _)) if !is_terminal_status(&status) => {
            // Already running (or queued/delayed): attach, never double-start.
            workflow_status_of(c, &round_id).await?.ok_or_else(|| {
                anyhow!("workflow for round {round_id} is running but has no status row")
            })
        }
        Some((_status, _, _, _)) => {
            // Terminal (SUCCESS / ERROR / CANCELLED): return the recorded
            // status; the Dart monitor decides whether to retry.
            workflow_status_of(c, &round_id).await?.ok_or_else(|| {
                anyhow!("workflow for round {round_id} is terminal but has no status row")
            })
        }
        None => {
            let _handle = engine
                .start_with(VoteRound, input, WorkflowOptions::with_id(round_id.clone()))
                .await?;
            // The workflow may already have progressed (or even finished) by
            // the time the status is read.
            workflow_status_of(c, &round_id).await?.ok_or_else(|| {
                anyhow!("workflow for round {round_id} did not start")
            })
        }
    }
}

/// Returns the current durable-workflow status for a round, or `None` when no
/// workflow row exists yet.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_workflow_status(
    round_id: String,
    c: &Coin,
) -> Result<Option<VotingWorkflowStatus>> {
    let _engine = ensure_engine(c).await?;
    workflow_status_of(c, &round_id).await
}

/// Clears a failed (terminal-error) workflow row and starts a fresh run from
/// the same input under the same workflow id. The fork's `voting_*` tables
/// are untouched — artifacts persist, so steps skip completed work.
///
/// Foreign keys are disabled on the wallet pool, so the engine's
/// `ON DELETE CASCADE` never fires: the child rows must be deleted manually.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_workflow_retry(
    round_id: String,
    c: &Coin,
) -> Result<VotingWorkflowStatus> {
    let engine = ensure_engine(c).await?;
    let (status, inputs_json) = match read_workflow_row(c, &round_id).await? {
        Some((status, inputs, _, _)) => (status, inputs),
        None => {
            bail!("no workflow row for round {round_id}; nothing to retry")
        }
    };
    if !is_terminal_status(&status) {
        bail!(
            "workflow for round {round_id} is not terminal ({status}); cannot retry"
        );
    }
    {
        let mut connection = c.get_connection().await?;
        for table in ["streams", "workflow_events", "notifications", "operation_outputs"] {
            sqlx::query(&format!(
                "DELETE FROM {table} WHERE workflow_uuid = ?"
            ))
            .bind(&round_id)
            .execute(&mut *connection)
            .await?;
        }
        sqlx::query("DELETE FROM workflow_status WHERE workflow_uuid = ?")
            .bind(&round_id)
            .execute(&mut *connection)
            .await?;
    }
    let input: VoteRoundInput = serde_json::from_str(&inputs_json)
        .map_err(|e| anyhow!("failed to decode stored workflow input: {e}"))?;
    let _handle = engine
        .start_with(VoteRound, input, WorkflowOptions::with_id(round_id.clone()))
        .await?;
    workflow_status_of(c, &round_id).await?.ok_or_else(|| {
        anyhow!("workflow for round {round_id} did not restart")
    })
}

// ---------------------------------------------------------------------------
// Status helpers
// ---------------------------------------------------------------------------

/// `(status, inputs, output, error)` of the workflow row for a round.
async fn read_workflow_row(
    c: &Coin,
    round_id: &str,
) -> Result<Option<(String, String, String, Option<String>)>> {
    let mut connection = c.get_connection().await?;
    let row = sqlx::query(
        "SELECT status, inputs, output, error FROM workflow_status WHERE workflow_uuid = ?",
    )
    .bind(round_id)
    .fetch_optional(&mut *connection)
    .await?;
    Ok(row.map(|r| {
        (
            r.get::<String, _>(0),
            r.get::<String, _>(1),
            r.get::<String, _>(2),
            r.get::<Option<String>, _>(3),
        )
    }))
}

/// Decoded progress stream events for a round, oldest first. Values are plain
/// JSON because the engine uses the Portable serializer.
async fn read_stream_events(c: &Coin, round_id: &str) -> Result<Vec<WorkflowEvent>> {
    let mut connection = c.get_connection().await?;
    let rows = sqlx::query(
        "SELECT value FROM streams WHERE workflow_uuid = ? AND key = 'progress' ORDER BY \"offset\"",
    )
    .bind(round_id)
    .fetch_all(&mut *connection)
    .await?;
    let mut events = Vec::with_capacity(rows.len());
    for row in rows {
        let value: String = row.get(0);
        if let Ok(event) = serde_json::from_str::<WorkflowEvent>(&value) {
            events.push(event);
        }
    }
    Ok(events)
}

/// Builds the polled status from the workflow row + progress stream.
async fn workflow_status_of(c: &Coin, round_id: &str) -> Result<Option<VotingWorkflowStatus>> {
    let Some((status, _inputs, output_json, error)) = read_workflow_row(c, round_id).await? else {
        return Ok(None);
    };
    let events = read_stream_events(c, round_id).await?;
    let last = events.last();

    let status_label = match status.as_str() {
        STATUS_SUCCESS => "success",
        STATUS_ERROR | STATUS_MAX_RECOVERY_ATTEMPTS_EXCEEDED => "error",
        STATUS_CANCELLED => "cancelled",
        STATUS_PENDING | STATUS_ENQUEUED | STATUS_DELAYED => "pending",
        _ => "unknown",
    };

    let (stage, progress) = match (status.as_str(), last) {
        (STATUS_SUCCESS, _) => ("done", Some(1.0)),
        (STATUS_ERROR | STATUS_MAX_RECOVERY_ATTEMPTS_EXCEEDED, _) => {
            ("error", last.and_then(|e| e.progress))
        }
        (STATUS_CANCELLED, _) => ("cancelled", None),
        (_, Some(event)) => (event.stage.as_str(), event.progress),
        _ => ("running", None),
    };
    // Live overlay: the in-process proof progress (display-only).
    let (stage, progress) = peek_live_progress(round_id)
        .map(|(s, p)| (s, Some(p)))
        .unwrap_or((stage.to_string(), progress));

    // Terminal success: decode the workflow output.
    let output: Option<VoteRoundOutput> = if status.as_str() == STATUS_SUCCESS {
        serde_json::from_str(&output_json).ok()
    } else {
        None
    };
    // The last "confirming" event carries the latest confirmed tx + height.
    let (last_tx, last_height) = match events
        .iter()
        .rev()
        .find_map(|e| e.tx_hash.as_ref().map(|t| (t.clone(), e.confirm_height)))
    {
        Some((tx, height)) => (Some(tx), height),
        None => (None, None),
    };

    Ok(Some(VotingWorkflowStatus {
        round_id: round_id.to_string(),
        status: status_label.to_string(),
        stage: stage.to_string(),
        progress,
        error,
        done_label: output.as_ref().map(|o| o.done_label.clone()),
        tx_hash: last_tx.or_else(|| output.as_ref().and_then(|o| o.tx_hash.clone())),
        confirm_height: last_height.or_else(|| output.as_ref().and_then(|o| o.confirm_height)),
        eligible_weight_zatoshi: output
            .as_ref()
            .and_then(|o| o.eligible_weight_zatoshi),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    #[test]
    fn parse_tx_confirmation_accepts_string_heights_and_events() {
        let conf = parse_tx_confirmation(
            r#"{"height":"7163319","events":[{"type":"vote"}]}"#,
        )
        .unwrap();
        assert_eq!(conf.height, 7_163_319);
        assert!(conf.events_json.contains("\"vote\""));
    }

    #[test]
    fn parse_tx_confirmation_rejects_missing_or_nonpositive_height() {
        assert!(parse_tx_confirmation(r#"{"height":0}"#).is_none());
        assert!(parse_tx_confirmation(r#"{"height":"0"}"#).is_none());
        assert!(parse_tx_confirmation(r#"{"height":-1}"#).is_none());
        assert!(parse_tx_confirmation(r#"{}"#).is_none());
        assert!(parse_tx_confirmation(r#"[]"#).is_none());
        assert!(parse_tx_confirmation("not json").is_none());
    }

    #[test]
    fn parse_submit_response_returns_tx_hash_and_rejects_bad_code() {
        assert_eq!(
            parse_submit_response(r#"{"tx_hash":"abc","code":0}"#).unwrap(),
            "abc"
        );
        let err = parse_submit_response(r#"{"tx_hash":"abc","code":1,"log":"nope"}"#).unwrap_err();
        assert!(err.to_string().contains("nope"));
        let err = parse_submit_response(r#"{"code":0}"#).unwrap_err();
        assert!(err.to_string().contains("rejected"));
    }

    #[test]
    fn sanitized_commit_drafts_drops_skipped_and_fills_fields() {
        let drafts = r#"[{"proposal_id":1,"choice":1,"num_options":2},
                          {"proposal_id":2,"choice":2,"num_options":2}]"#;
        let sanitized = sanitized_commit_drafts(drafts).unwrap();
        let decoded: Vec<serde_json::Value> = serde_json::from_str(&sanitized).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0]["proposal_id"], serde_json::json!(1));
        assert_eq!(decoded[0]["vc_tree_position"], serde_json::json!(0));
        assert_eq!(decoded[0]["single_share"], serde_json::json!(false));
        assert!(sanitized_commit_drafts("[]").is_none());
        assert!(sanitized_commit_drafts("not json").is_none());
    }

    #[test]
    fn draft_proposal_ids_extracts_positive_ids() {
        assert_eq!(
            draft_proposal_ids(r#"[{"proposal_id":3},{"proposal_id":0},{"proposal_id":9}]"#),
            vec![3, 9]
        );
        assert!(draft_proposal_ids("[]").is_empty());
        assert!(draft_proposal_ids("garbage").is_empty());
    }

    #[test]
    fn remaining_label_matches_dart_semantics() {
        let step = |kind: &str| voting::VotingNextStep {
            kind: kind.to_string(),
            bundle_index: 0,
            proposal_id: 0,
            share_index: 0,
            choice: 0,
        };
        assert_eq!(
            remaining_label(&[step("submit_shares"), step("submit_shares")]),
            "Waiting for the share window"
        );
        assert_eq!(
            remaining_label(&[step("confirm_share")]),
            "Waiting for share confirmations"
        );
        assert_eq!(
            remaining_label(&[step("cast_vote")]),
            "Waiting for the next step"
        );
        assert_eq!(remaining_label(&[]), "All steps already confirmed");
    }

    #[test]
    fn inject_vote_round_id_adds_field() {
        let body = inject_vote_round_id(r#"{"share":"x"}"#, "round-7").unwrap();
        assert_eq!(
            body,
            r#"{"share":"x","vote_round_id":"round-7"}"#
        );
    }

    // ---------------------------------------------------------------------
    // Durable-engine integration (the wiring the workflow relies on)
    // ---------------------------------------------------------------------

    // Per-test statics: cargo runs tests in parallel, and two tests sharing
    // one workflow fn's statics race on the panic flag and the run counter.
    static RECOVER_STEP_RUNS: AtomicU32 = AtomicU32::new(0);
    static RECOVER_PANIC_FLAG: AtomicBool = AtomicBool::new(false);

    /// Minimal durable workflow used to prove checkpoint replay + recovery:
    /// step `a` counts its executions; a body-level panic (a "crash") leaves
    /// the row non-terminal so the next `recover_on_launch` re-dispatches it
    /// and the re-run serves step `a` from its checkpoint.
    #[durare::workflow]
    async fn recover_probe(ctx: DurableContext, _input: ()) -> durare::Result<u32> {
        let a: u32 = ctx.step("a", || async {
            RECOVER_STEP_RUNS.fetch_add(1, Ordering::SeqCst);
            Ok(1u32)
        })
        .await?;
        if RECOVER_PANIC_FLAG.load(Ordering::SeqCst) {
            panic!("simulated crash after step a");
        }
        Ok(a)
    }

    static REPLAY_STEP_RUNS: AtomicU32 = AtomicU32::new(0);

    /// Never-panicking twin of [`recover_probe`] for the terminal-replay test
    /// (keeps its counter isolated from the recovery test's statics).
    #[durare::workflow]
    async fn replay_probe(ctx: DurableContext, _input: ()) -> durare::Result<u32> {
        let a: u32 = ctx.step("a", || async {
            REPLAY_STEP_RUNS.fetch_add(1, Ordering::SeqCst);
            Ok(1u32)
        })
        .await?;
        Ok(a)
    }

    /// A panicked (non-terminal) workflow row is re-dispatched by a fresh
    /// engine's `recover_on_launch`, and the checkpointed step is served from
    /// its recorded output instead of re-running — the exact guarantee the
    /// voting workflow depends on after an app restart.
    #[tokio::test]
    async fn panicked_workflow_recovers_from_checkpoints() {
        let dir = std::env::temp_dir().join(format!(
            "durare-test-{}-rec",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let url = format!("sqlite://{}/wf.db", dir.display());

        RECOVER_STEP_RUNS.store(0, Ordering::SeqCst);
        RECOVER_PANIC_FLAG.store(true, Ordering::SeqCst);
        {
            let provider = SqliteProvider::connect(&url)
                .await
                .unwrap()
                .with_serializer(Serializer::Portable);
            let mut builder = DurableEngine::builder(Arc::new(provider));
            builder.recover_on_launch(true);
            builder.executor_id("test-exec");
            builder.app_version("test-app");
            let engine = builder.build().await.unwrap();
            engine.launch().await.unwrap();
            // The body panics after step `a`; the row is left non-terminal.
            let _handle = engine
                .start_with(RecoverProbe, (), WorkflowOptions::with_id("wf-recover"))
                .await
                .unwrap();
            // Give the run time to checkpoint step `a` and panic.
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        } // engine dropped; the row stays PENDING with step `a` checkpointed

        RECOVER_PANIC_FLAG.store(false, Ordering::SeqCst);
        {
            let provider = SqliteProvider::connect(&url)
                .await
                .unwrap()
                .with_serializer(Serializer::Portable);
            let mut builder = DurableEngine::builder(Arc::new(provider));
            builder.recover_on_launch(true);
            builder.executor_id("test-exec");
            builder.app_version("test-app");
            let engine = builder.build().await.unwrap();
            engine.launch().await.unwrap();
            // The recovered run completes; awaiting the polling handle returns
            // its output (the fork's status FRB reads the same row).
            let handle = engine
                .retrieve_workflow::<u32>("wf-recover")
                .await
                .unwrap();
            let out = tokio::time::timeout(
                std::time::Duration::from_secs(15),
                handle,
            )
            .await
            .expect("recovered workflow did not complete")
            .unwrap();
            assert_eq!(out, 1);
        }

        assert_eq!(
            RECOVER_STEP_RUNS.load(Ordering::SeqCst),
            1,
            "the checkpointed step must run exactly once across the crash"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A terminal (SUCCESS) row restarts under the same id by returning the
    /// recorded output without re-running — what `voting_workflow_start`'
    /// idempotent re-entry relies on.
    #[tokio::test]
    async fn terminal_workflow_restart_returns_recorded_output() {
        let dir = std::env::temp_dir().join(format!(
            "durare-test-{}-replay",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let url = format!("sqlite://{}/wf.db", dir.display());

        REPLAY_STEP_RUNS.store(0, Ordering::SeqCst);
        for _ in 0..2 {
            let provider = SqliteProvider::connect(&url)
                .await
                .unwrap()
                .with_serializer(Serializer::Portable);
            let mut builder = DurableEngine::builder(Arc::new(provider));
            builder.recover_on_launch(true);
            builder.executor_id("test-exec");
            builder.app_version("test-app");
            let engine = builder.build().await.unwrap();
            engine.launch().await.unwrap();
            let handle = engine
                .start_with(ReplayProbe, (), WorkflowOptions::with_id("wf-replay"))
                .await
                .unwrap();
            let out = tokio::time::timeout(std::time::Duration::from_secs(15), handle)
                .await
                .expect("workflow did not complete")
                .unwrap();
            assert_eq!(out, 1);
        }

        assert_eq!(
            REPLAY_STEP_RUNS.load(Ordering::SeqCst),
            1,
            "a terminal workflow must not re-run on same-id restart"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The test seam wins over the transport rule — `VOTECHAIN_OVERRIDE`
    /// must be honored by every step's chain resolution.
    #[tokio::test]
    async fn chain_override_wins_over_transport_proxy_rule() {
        struct MockChain;
        impl vc::VoteChainClient for MockChain {
            fn submit_delegation(
                &self,
                _base_url: String,
                _submission_json: String,
            ) -> futures::future::BoxFuture<'_, Result<(u16, String)>> {
                Box::pin(async { Ok((200, "mock".to_string())) })
            }
            fn submit_vote(
                &self,
                _base_url: String,
                _commitment_json: String,
            ) -> futures::future::BoxFuture<'_, Result<(u16, String)>> {
                Box::pin(async { Ok((200, "mock".to_string())) })
            }
            fn tx_confirmation(
                &self,
                _base_url: String,
                _tx_hash: String,
            ) -> futures::future::BoxFuture<'_, Result<(u16, String)>> {
                Box::pin(async { Ok((404, String::new())) })
            }
            fn submit_share(
                &self,
                _server_url: String,
                _payload_json: String,
            ) -> futures::future::BoxFuture<'_, Result<(u16, String)>> {
                Box::pin(async { Ok((200, "mock".to_string())) })
            }
            fn share_status(
                &self,
                _server_url: String,
                _round_id: String,
                _share_id: String,
            ) -> futures::future::BoxFuture<'_, Result<(u16, String)>> {
                Box::pin(async { Ok((404, String::new())) })
            }
            fn round_status(
                &self,
                _base_url: String,
                _round_id: String,
            ) -> futures::future::BoxFuture<'_, Result<(u16, String)>> {
                Box::pin(async { Ok((404, String::new())) })
            }
        }

        let _ = vc::VOTECHAIN_OVERRIDE.set(Arc::new(MockChain));
        let client = vc::resolve_chain_client(false, "");
        let (status, _) = client
            .submit_delegation("http://x".to_string(), "{}".to_string())
            .await
            .unwrap();
        assert_eq!(status, 200);
    }

    /// Ballot-intent writes (the `vote_work` step's durable side effect) are
    /// upserts: re-writing the same proposal after a crash must not duplicate
    /// the row, and a changed choice must update in place.
    #[tokio::test]
    async fn ballot_intent_writes_are_idempotent_upserts() {
        let dir = std::env::temp_dir().join(format!(
            "durare-test-{}-intent",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let db_filepath = format!("{}/wallet.db", dir.display());

        let coin = crate::api::coin::Coin::new(Some(0))
            .open_database(db_filepath.clone(), None)
            .await
            .unwrap();
        {
            let mut connection = coin.get_connection().await.unwrap();
            // Minimal account with a 32-byte seed fingerprint (voting_wallet_id
            // requires exactly 32 bytes).
            sqlx::query(
                "INSERT INTO accounts (id_account, name, passphrase, aindex, dindex, def_dindex, birth, position, use_internal, hidden, saved, seed_fingerprint)
                 VALUES (0, 'test', '', 0, 0, 0, 0, 0, 0, 0, 1, ?)",
            )
            .bind(vec![7u8; 32])
            .execute(&mut *connection)
            .await
            .unwrap();
            // Run the fork's migrations first (open_voting_db → from_pool
            // migrates), so the voting_* tables exist for the fixture rows.
            let wallet_id = vc::voting_wallet_id(&mut connection, 0).await.unwrap();
            let _db = vc::open_voting_db(coin.get_pool().unwrap(), &mut connection, &wallet_id)
                .await
                .unwrap();
            // Minimal round row scoped to this wallet.
            sqlx::query(
                "INSERT INTO voting_rounds (round_id, wallet_id, network, snapshot_height, ea_pk, nc_root, nullifier_imt_root, created_at)
                 VALUES ('round-1', ?, 'testnet', 100, ?, ?, ?, 0)",
            )
            .bind(&wallet_id)
            .bind(vec![0u8; 32])
            .bind(vec![0u8; 32])
            .bind(vec![0u8; 32])
            .execute(&mut *connection)
            .await
            .unwrap();
        }

        let round_id = "round-1";
        for _ in 0..2 {
            voting::voting_set_ballot_intent(round_id, 1, false, 1, 2, &coin)
                .await
                .unwrap();
        }
        let intents = voting::voting_ballot_intents(round_id, &coin)
            .await
            .unwrap();
        assert_eq!(intents.len(), 1, "duplicate intent writes must upsert");
        assert!(!intents[0].skipped);
        assert_eq!(intents[0].choice, Some(1));

        // A changed decision updates the same row.
        voting::voting_set_ballot_intent(round_id, 1, false, 2, 3, &coin)
            .await
            .unwrap();
        let intents = voting::voting_ballot_intents(round_id, &coin)
            .await
            .unwrap();
        assert_eq!(intents.len(), 1);
        assert_eq!(intents[0].choice, Some(2));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
