//! FRB wrappers for the shielded voting flow (ZIP 262 delegation + cast votes).
//!
//! The fork's `zcash_voting` types are not FRB-visible, so this module defines
//! JSON-serializable mirror structs and converts at the boundary. State
//! transitions follow the plan: prepare → setup → sign/prove/submit → confirm,
//! then van witness → commit → payloads → record execution → confirm.

use std::sync::Arc;

use anyhow::{anyhow, Result};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use zcash_voting::prelude::{
    BundlePolicy, DelegationProgress, DelegationProgressBridge, DraftVote, NoopProgressReporter,
    ShareTimingPolicy, TxEvent, VoteCommitStageBridge,
};
use zcash_voting::recovery::{
    DelegationRecovery as ForkDelegationRecovery, RoundRecoverySnapshot as ForkRoundRecovery,
    ShareWorkflow as ForkShareWorkflow, VoteRecovery as ForkVoteRecovery,
};
use zcash_voting::round::RoundInfo as ForkRoundInfo;
use zcash_voting::session::{Decision, NextStep, RoundPlan as ForkRoundPlan};
use zcash_voting::types::ShareDelegationRecord as ForkShareDelegationRecord;
use zcash_voting::{Network as VotingNetwork, VotingRoundParams};
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

use crate::{api::coin::Coin, frb_generated::StreamSink, voting};

// ---------------------------------------------------------------------------
// Mirror types
// ---------------------------------------------------------------------------

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingPreparedInfo {
    pub round_id: String,
    pub bundle_index: u32,
    pub eligible_weight_zatoshi: u64,
    pub delegated_weight_zatoshi: u64,
    pub round_name: String,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingPirLayout {
    pub pir_depth: u32,
    pub tier0_layers: u32,
    pub tier1_layers: u32,
    pub poly_len: u32,
}

impl VotingPirLayout {
    fn to_fork(&self) -> zcash_voting::config::PirLayout {
        zcash_voting::config::PirLayout {
            pir_depth: self.pir_depth,
            tier0_layers: self.tier0_layers,
            tier1_layers: self.tier1_layers,
            poly_len: self.poly_len,
        }
    }
}

impl From<zcash_voting::config::PirLayout> for VotingPirLayout {
    fn from(l: zcash_voting::config::PirLayout) -> Self {
        Self {
            pir_depth: l.pir_depth,
            tier0_layers: l.tier0_layers,
            tier1_layers: l.tier1_layers,
            poly_len: l.poly_len,
        }
    }
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingDelegationSetup {
    pub pczt_bytes: Vec<u8>,
    pub pczt_sighash: Vec<u8>,
    pub rk: Vec<u8>,
    pub action_index: u32,
    pub action_bytes: Vec<u8>,
    pub tx1_effects: Vec<u8>,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingDelegationSubmission {
    pub proof: Vec<u8>,
    pub rk: Vec<u8>,
    pub nf_signed: Vec<u8>,
    pub cmx_new: Vec<u8>,
    pub gov_comm: Vec<u8>,
    pub gov_nullifiers: Vec<Vec<u8>>,
    pub alpha: Vec<u8>,
    pub vote_round_id: String,
    pub spend_auth_sig: Vec<u8>,
    pub sighash: Vec<u8>,
    pub tx1_effects: Vec<u8>,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingDelegationBuild {
    pub submission: VotingDelegationSubmission,
    pub wire_json: String,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingDelegationConfirmation {
    pub tx_hash: String,
    pub van_leaf_position: u32,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingVoteConfirmation {
    pub tx_hash: String,
    pub van_leaf_position: u32,
    pub vc_tree_position: u64,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingVanWitness {
    pub auth_path: Vec<Vec<u8>>,
    pub position: u32,
    pub anchor_height: u32,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingSignedVoteCommitment {
    pub proposal_id: u32,
    pub choice: u32,
    pub vote_round_id: String,
    pub van_nullifier: Vec<u8>,
    pub vote_authority_note_new: Vec<u8>,
    pub vote_commitment: Vec<u8>,
    pub proof: Vec<u8>,
    pub anchor_height: u32,
    pub r_vpk: Vec<u8>,
    pub vote_auth_sig: Vec<u8>,
    pub commitment_bundle_json: String,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingVoteCommitments {
    pub bundle_index: u32,
    pub commitments: Vec<VotingSignedVoteCommitment>,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingEncryptedShare {
    pub c1: Vec<u8>,
    pub c2: Vec<u8>,
    pub share_index: u32,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingSharePayload {
    pub shares_hash: Vec<u8>,
    pub proposal_id: u32,
    pub vote_decision: u32,
    pub enc_share: VotingEncryptedShare,
    pub tree_position: u64,
    pub all_enc_shares: Vec<VotingEncryptedShare>,
    pub share_comms: Vec<Vec<u8>>,
    pub primary_blind: Vec<u8>,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingVoteSubmission {
    pub vote_round_id: String,
    pub proposal_id: u32,
    pub van_nullifier: Vec<u8>,
    pub vote_authority_note_new: Vec<u8>,
    pub vote_commitment: Vec<u8>,
    pub proof: Vec<u8>,
    pub r_vpk: Vec<u8>,
    pub vote_auth_sig: Vec<u8>,
    pub anchor_height: u32,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingVotePayloads {
    pub submission: VotingVoteSubmission,
    pub share_payloads: Vec<VotingSharePayload>,
}

/// One helper-share delivery result for `voting_record_execution`.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingShareDelivery {
    pub share_index: u32,
    pub sent_to_urls: Vec<String>,
    pub submit_at: u64,
    pub confirmed: bool,
}

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

impl From<zcash_voting::prelude::DelegationSetup> for VotingDelegationSetup {
    fn from(setup: zcash_voting::prelude::DelegationSetup) -> Self {
        Self {
            pczt_bytes: setup.pczt_bytes,
            pczt_sighash: setup.pczt_sighash.to_vec(),
            rk: setup.rk.to_vec(),
            action_index: setup.action_index as u32,
            action_bytes: setup.action_bytes,
            tx1_effects: setup.tx1_effects,
        }
    }
}

impl From<zcash_voting::prelude::DelegationSubmission> for VotingDelegationSubmission {
    fn from(submission: zcash_voting::prelude::DelegationSubmission) -> Self {
        Self {
            proof: submission.proof,
            rk: submission.rk.to_vec(),
            nf_signed: submission.nf_signed.to_vec(),
            cmx_new: submission.cmx_new.to_vec(),
            gov_comm: submission.gov_comm.to_vec(),
            gov_nullifiers: submission.gov_nullifiers.into_iter().map(|v| v.to_vec()).collect(),
            alpha: submission.alpha.to_vec(),
            vote_round_id: submission.vote_round_id,
            spend_auth_sig: submission.spend_auth_sig.to_vec(),
            sighash: submission.sighash.to_vec(),
            tx1_effects: submission.tx1_effects,
        }
    }
}

impl From<zcash_voting::prelude::DelegationConfirmation> for VotingDelegationConfirmation {
    fn from(confirmation: zcash_voting::prelude::DelegationConfirmation) -> Self {
        Self {
            tx_hash: confirmation.tx_hash,
            van_leaf_position: confirmation.van_leaf_position,
        }
    }
}

impl From<zcash_voting::prelude::VoteConfirmation> for VotingVoteConfirmation {
    fn from(confirmation: zcash_voting::prelude::VoteConfirmation) -> Self {
        Self {
            tx_hash: confirmation.tx_hash,
            van_leaf_position: confirmation.van_leaf_position,
            vc_tree_position: confirmation.vc_tree_position,
        }
    }
}

impl From<zcash_voting::prelude::VanWitness> for VotingVanWitness {
    fn from(witness: zcash_voting::prelude::VanWitness) -> Self {
        Self {
            auth_path: witness.auth_path,
            position: witness.position,
            anchor_height: witness.anchor_height,
        }
    }
}

impl From<zcash_voting::prelude::SignedVoteCommitments> for VotingVoteCommitments {
    fn from(commitments: zcash_voting::prelude::SignedVoteCommitments) -> Self {
        Self {
            bundle_index: commitments.bundle_index,
            commitments: commitments
                .commitments
                .into_iter()
                .map(|c| VotingSignedVoteCommitment {
                    proposal_id: c.proposal_id,
                    choice: c.choice,
                    vote_round_id: c.vote_round_id,
                    van_nullifier: c.van_nullifier.to_vec(),
                    vote_authority_note_new: c.vote_authority_note_new.to_vec(),
                    vote_commitment: c.vote_commitment.to_vec(),
                    proof: c.proof,
                    anchor_height: c.anchor_height,
                    r_vpk: c.r_vpk.to_vec(),
                    vote_auth_sig: c.vote_auth_sig.to_vec(),
                    commitment_bundle_json: c.commitment_bundle_json,
                })
                .collect(),
        }
    }
}

impl From<&zcash_voting::WireEncryptedShare> for VotingEncryptedShare {
    fn from(share: &zcash_voting::WireEncryptedShare) -> Self {
        Self {
            c1: share.c1.clone(),
            c2: share.c2.clone(),
            share_index: share.share_index,
        }
    }
}

impl From<&zcash_voting::prelude::SharePayload> for VotingSharePayload {
    fn from(payload: &zcash_voting::prelude::SharePayload) -> Self {
        Self {
            shares_hash: payload.shares_hash.clone(),
            proposal_id: payload.proposal_id,
            vote_decision: payload.vote_decision,
            enc_share: (&payload.enc_share).into(),
            tree_position: payload.tree_position,
            all_enc_shares: payload.all_enc_shares.iter().map(Into::into).collect(),
            share_comms: payload.share_comms.clone(),
            primary_blind: payload.primary_blind.clone(),
        }
    }
}

impl From<zcash_voting::prelude::VoteSubmission> for VotingVoteSubmission {
    fn from(submission: zcash_voting::prelude::VoteSubmission) -> Self {
        Self {
            vote_round_id: submission.vote_round_id,
            proposal_id: submission.proposal_id,
            van_nullifier: submission.van_nullifier.to_vec(),
            vote_authority_note_new: submission.vote_authority_note_new.to_vec(),
            vote_commitment: submission.vote_commitment.to_vec(),
            proof: submission.proof,
            r_vpk: submission.r_vpk.to_vec(),
            vote_auth_sig: submission.vote_auth_sig.to_vec(),
            anchor_height: submission.anchor_height,
        }
    }
}

// ---------------------------------------------------------------------------
// Delegation flow
// ---------------------------------------------------------------------------

/// Creates and persists a fresh app-owned voting hotkey (hex stored secret).
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_hotkey_create(c: &Coin) -> Result<String> {
    let network = voting::voting_network(&c.network())?;
    let mut connection = c.get_connection().await?;
    let hotkey = voting::voting_hotkey_create(&mut connection, network).await?;
    Ok(hex::encode(hotkey.stored_secret()))
}

/// Returns the persisted voting hotkey stored secret (hex), if any.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_hotkey_get(c: &Coin) -> Result<String> {
    let network = voting::voting_network(&c.network())?;
    let mut connection = c.get_connection().await?;
    let hotkey = voting::voting_hotkey_load(&mut connection, network).await?;
    Ok(hex::encode(hotkey.stored_secret()))
}

/// Prepares one delegation bundle from the wallet's own Ironwood notes.
///
/// `round_params_json` is the JSON-serialized `VotingRoundParams` from the
/// vote chain. The wallet must be synced through the round snapshot height;
/// witnesses are rooted at the snapshot's Ironwood `nc_root`. On success the
/// round inputs are persisted (props table) so a restart can re-prepare via
/// [`delegation_prepare_resume`].
#[cfg_attr(feature = "flutter", frb)]
#[allow(clippy::too_many_arguments)]
pub async fn delegation_prepare(
    round_params_json: &str,
    round_name: &str,
    session_json: Option<String>,
    bundle_index: u32,
    max_real_notes_per_bundle: Option<u32>,
    lightwalletd_url: &str,
    c: &Coin,
) -> Result<VotingPreparedInfo> {
    let info = prepare_bundle(
        round_params_json,
        round_name,
        session_json,
        bundle_index,
        max_real_notes_per_bundle,
        lightwalletd_url,
        c,
    )
    .await?;
    let mut connection = c.get_connection().await?;
    voting::save_round_config(
        &mut connection,
        &info.round_id,
        round_params_json,
        round_name,
        max_real_notes_per_bundle,
        lightwalletd_url,
    )
    .await?;
    Ok(info)
}

/// Re-runs [`delegation_prepare`] for a round whose prepared bundle was lost
/// with the process (the prepared-bundle cache is process-local). Inputs come
/// from the config saved by the first prepare; the optional params override
/// the saved values when present.
#[cfg_attr(feature = "flutter", frb)]
pub async fn delegation_prepare_resume(
    round_id: &str,
    bundle_index: u32,
    max_real_notes_per_bundle: Option<u32>,
    lightwalletd_url: Option<String>,
    c: &Coin,
) -> Result<VotingPreparedInfo> {
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let (round_params_json, round_name, saved_policy, saved_lwd) =
        voting::load_round_config(&mut connection, &round_id).await?;
    drop(connection);

    let bundle_policy = max_real_notes_per_bundle.or(saved_policy);
    let lightwalletd_url = lightwalletd_url.unwrap_or(saved_lwd);
    prepare_bundle(
        &round_params_json,
        &round_name,
        None,
        bundle_index,
        bundle_policy,
        &lightwalletd_url,
        c,
    )
    .await
}

/// Shared prepare pipeline; see [`delegation_prepare`].
async fn prepare_bundle(
    round_params_json: &str,
    round_name: &str,
    session_json: Option<String>,
    bundle_index: u32,
    max_real_notes_per_bundle: Option<u32>,
    lightwalletd_url: &str,
    c: &Coin,
) -> Result<VotingPreparedInfo> {
    let account = c.account;
    let wallet_network = &c.network();
    let network = voting::voting_network(wallet_network)?;
    let round_params: VotingRoundParams = serde_json::from_str(round_params_json)?;
    let snapshot_height = u32::try_from(round_params.snapshot_height)
        .map_err(|_| anyhow!("snapshot height {} does not fit u32", round_params.snapshot_height))?;

    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;

    let lwd =
        voting::gather_lwd_inputs(lightwalletd_url, network, &round_params, round_name).await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let inputs = voting::load_round_inputs(
        wallet_network,
        &mut connection,
        &mut client,
        account,
        snapshot_height,
        &round_params.nc_root,
    )
    .await?;
    let identity =
        voting::load_voting_identity(&mut connection, account, network, &lwd.resolved_round_name)
            .await?;
    let bundle_policy = BundlePolicy::from_optional_max_real_notes_per_bundle(
        max_real_notes_per_bundle,
    )?;

    let prepared = voting::prepare_delegation_bundle(
        c.get_pool()?,
        &wallet_id,
        lwd,
        session_json.as_deref(),
        inputs.note_infos,
        identity.delegation_keys,
        inputs.witnesses,
        bundle_index,
        bundle_policy,
    )
    .await?;

    let info = VotingPreparedInfo {
        round_id: prepared.round_id.clone(),
        bundle_index: prepared.bundle_index,
        eligible_weight_zatoshi: prepared.eligible_weight_zatoshi(),
        delegated_weight_zatoshi: prepared.delegated_weight_zatoshi()?,
        round_name: prepared.round_name.clone(),
    };
    voting::cache_prepared_bundle(&wallet_id, prepared);
    Ok(info)
}

/// Builds and persists the governance PCZT setup for a prepared bundle.
#[cfg_attr(feature = "flutter", frb)]
pub async fn delegation_setup(
    round_id: &str,
    bundle_index: u32,
    c: &Coin,
) -> Result<VotingDelegationSetup> {
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, c.account).await?;
    let prepared = voting::load_prepared_bundle(&wallet_id, round_id, bundle_index)?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;

    let setup = prepared.setup(&db, &NoopProgressReporter).await?;
    Ok(setup.into())
}

/// Signs with the wallet seed, proves against the PIR server, and assembles
/// the chain-ready delegation submission for the vote chain.
#[cfg_attr(feature = "flutter", frb)]
#[allow(clippy::too_many_arguments)]
pub async fn delegation_sign_and_submit(
    round_id: &str,
    bundle_index: u32,
    pczt_bytes: Vec<u8>,
    pir_layout: VotingPirLayout,
    pir_server_url: &str,
    c: &Coin,
) -> Result<VotingDelegationSubmission> {
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, c.account).await?;
    let prepared = voting::load_prepared_bundle(&wallet_id, round_id, bundle_index)?;
    let seed = voting::account_seed(&mut connection, c.account).await?;

    let (submission, _wire_json) = voting::prove_and_submit_delegation(
        c.get_pool()?,
        &wallet_id,
        &prepared,
        &seed,
        pczt_bytes,
        pir_layout.to_fork(),
        pir_server_url,
    )
    .await?;
    Ok(submission.into())
}

/// Records a confirmed delegation transaction and persists the bundle's VAN
/// position (required before any vote).
#[cfg_attr(feature = "flutter", frb)]
pub async fn delegation_confirm(
    round_id: &str,
    bundle_index: u32,
    tx_hash: &str,
    events_json: &str,
    c: &Coin,
) -> Result<VotingDelegationConfirmation> {
    let events: Vec<TxEvent> = serde_json::from_str(events_json)?;
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, c.account).await?;
    let confirmation = voting::confirm_delegation(
        c.get_pool()?,
        &wallet_id,
        round_id,
        bundle_index,
        tx_hash,
        &events,
    )
    .await?;
    Ok(confirmation.into())
}

/// Builds and signs the delegation payload with live progress events
/// (`delegation_sign_and_submit` without the progress stream).
///
/// `pir_layout` is persisted on first use; pass `None` after a restart to
/// resume with the saved layout. Returns the submission together with its
/// vote-chain wire JSON body (ready for `votechain_submit_delegation`).
#[cfg_attr(feature = "flutter", frb)]
#[allow(clippy::too_many_arguments)]
pub async fn delegation_build_submission(
    sink: StreamSink<VotingDelegationProgress>,
    round_id: &str,
    bundle_index: u32,
    pczt_bytes: Vec<u8>,
    pir_layout: Option<VotingPirLayout>,
    pir_server_url: &str,
    c: &Coin,
) -> Result<VotingDelegationBuild> {
    let account = c.account;
    let round_id = round_id.to_string();
    let pir_server_url = pir_server_url.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let pir_server_url = if pir_server_url.is_empty() {
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
            &pir_server_url,
        )
        .await?;
        pir_server_url
    };
    let pir_layout = match pir_layout {
        Some(layout) => {
            voting::save_pir_layout(&mut connection, &round_id, &layout.to_fork()).await?;
            layout
        }
        None => voting::load_pir_layout(&mut connection, &round_id)
            .await?
            .map(Into::into)
            .ok_or_else(|| {
                anyhow!("no saved PIR layout for round {round_id}; pass pir_layout once")
            })?,
    };
    let prepared = voting::load_prepared_bundle(&wallet_id, &round_id, bundle_index)?;
    let seed = voting::account_seed(&mut connection, account).await?;

    let progress = Arc::new(DelegationProgressBridge::new({
        let sink_for_progress = sink.clone();
        move |p| {
            let _ = sink_for_progress.add(p.into());
        }
    }));
    let (submission, wire_json) =
        match voting::prove_and_submit_delegation_with_progress(
            c.get_pool()?,
            &wallet_id,
            &prepared,
            &seed,
            pczt_bytes,
            pir_layout.to_fork(),
            &pir_server_url,
            progress.clone(),
        )
        .await
        {
            Ok(v) => v,
            Err(e) => {
                // The FRB stream binding runs the call with `unawaited` and
                // discards the returned future, so a plain `Err` would surface
                // as an unhandled exception the app can never catch. Deliver
                // the error through the sink (decoded as AnyhowException on
                // the Dart stream) and return a benign Ok — the binding
                // discards this value anyway.
                let _ = sink.add_error(e);
                return Ok(VotingDelegationBuild {
                    submission: VotingDelegationSubmission {
                        proof: Vec::new(),
                        rk: Vec::new(),
                        nf_signed: Vec::new(),
                        cmx_new: Vec::new(),
                        gov_comm: Vec::new(),
                        gov_nullifiers: Vec::new(),
                        alpha: Vec::new(),
                        vote_round_id: String::new(),
                        spend_auth_sig: Vec::new(),
                        sighash: Vec::new(),
                        tx1_effects: Vec::new(),
                    },
                    wire_json: String::new(),
                });
            }
        };
    // The FRB boundary drops this return value (StreamSink params take over),
    // so persist the wire body for `delegation_wire_json` to pick up. This also
    // makes a crash between proving and broadcasting resumable without
    // re-proving.
    crate::db::put_prop(
        &mut connection,
        &format!("voting_round_delegation_wire:{round_id}:{bundle_index}"),
        &wire_json,
    )
    .await?;
    Ok(VotingDelegationBuild {
        submission: submission.into(),
        wire_json,
    })
}

/// Returns the vote-chain wire JSON built by the last
/// [`delegation_build_submission`] run for a bundle, if any.
#[cfg_attr(feature = "flutter", frb)]
pub async fn delegation_wire_json(
    round_id: &str,
    bundle_index: u32,
    c: &Coin,
) -> Result<Option<String>> {
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    Ok(
        crate::db::get_prop(
            &mut connection,
            &format!("voting_round_delegation_wire:{round_id}:{bundle_index}"),
        )
        .await?,
    )
}

/// Atomically records a delegation transaction hash with idempotency checks,
/// so a restart between broadcast and confirmation resumes via `PollDelegation`
/// instead of re-broadcasting.
#[cfg_attr(feature = "flutter", frb)]
pub async fn delegation_mark_submitted(
    round_id: &str,
    bundle_index: u32,
    tx_hash: &str,
    c: &Coin,
) -> Result<()> {
    let account = c.account;
    let round_id = round_id.to_string();
    let tx_hash = tx_hash.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    db.mark_delegation_submitted(&round_id, bundle_index, &tx_hash)
        .await?;
    Ok(())
}

/// Returns the recorded delegation transaction hash for a bundle, if any.
#[cfg_attr(feature = "flutter", frb)]
pub async fn delegation_tx_hash(
    round_id: &str,
    bundle_index: u32,
    c: &Coin,
) -> Result<Option<String>> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    Ok(db.get_delegation_tx_hash(&mut *connection, &round_id, bundle_index).await?)
}

// ---------------------------------------------------------------------------
// Vote flow
// ---------------------------------------------------------------------------

/// Persists the voter's terminal decision for one proposal before any
/// zero-knowledge work, so a crash cannot lose the ballot and later votes are
/// conflict-checked against it.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_set_ballot_intent(
    round_id: &str,
    proposal_id: u32,
    skipped: bool,
    choice: u32,
    num_options: u32,
    c: &Coin,
) -> Result<()> {
    let account = c.account;
    let round_id = round_id.to_string();
    let decision = if skipped {
        Decision::Skipped
    } else {
        Decision::Choice(choice)
    };
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    db.set_ballot_intent(&round_id, proposal_id, decision, num_options)
        .await?;
    Ok(())
}

/// Returns the quantized voting weight (zatoshi) for the account's eligible
/// shielded notes at `snapshot_height`, computed with the same canonical
/// bundle planning as the delegation prepare step — but from the local DB
/// only (no witnesses, no tree state). Shown pre-submission as an estimate.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_eligible_weight(snapshot_height: u32, c: &Coin) -> Result<u64> {
    let mut connection = c.get_connection().await?;
    voting::eligible_voting_weight(&mut connection, c.account, snapshot_height).await
}

/// Persists the draft ballot for a round (props table, wallet-scoped).
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_drafts_save(round_id: &str, drafts_json: &str, c: &Coin) -> Result<()> {
    let round_id = round_id.to_string();
    let drafts_json = drafts_json.to_string();
    let mut connection = c.get_connection().await?;
    crate::db::put_prop(
        &mut connection,
        &format!("voting_drafts:{round_id}"),
        &drafts_json,
    )
    .await?;
    Ok(())
}

/// Returns the persisted draft ballot for a round, if any.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_drafts_load(round_id: &str, c: &Coin) -> Result<Option<String>> {
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    Ok(crate::db::get_prop(&mut connection, &format!("voting_drafts:{round_id}")).await?)
}

/// Commits one bundle's votes with live stage events. Draft votes are
/// JSON-serialized fork `DraftVote`s; the VAN witness is derived internally
/// after syncing the vote tree.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_commit_with_progress(
    sink: StreamSink<VotingVoteCommitStage>,
    round_id: &str,
    bundle_index: u32,
    drafts_json: &str,
    vote_node_url: &str,
    c: &Coin,
) -> Result<VotingVoteCommitments> {
    let account = c.account;
    let round_id = round_id.to_string();
    let drafts_json = drafts_json.to_string();
    let vote_node_url = vote_node_url.to_string();
    let drafts: Vec<DraftVote> = serde_json::from_str(&drafts_json)?;
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let hotkey = voting::voting_hotkey_load(
        &mut connection,
        voting::voting_network(&c.network())?,
    )
    .await?;
    let witness =
        voting::vote_van_witness(c.get_pool()?, &wallet_id, &round_id, bundle_index, &vote_node_url)
            .await?;

    let stages = VoteCommitStageBridge::new({
        let sink_for_stages = sink.clone();
        move |s| {
            let _ = sink_for_stages.add(s.into());
        }
    });
    let commitments = match voting::commit_votes_with_progress(
        c.get_pool()?,
        &wallet_id,
        &round_id,
        bundle_index,
        &drafts,
        &witness,
        &hotkey,
        &stages,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            // Same FRB stream footgun as delegation_build_submission: the
            // binding drops the returned future, so deliver the error via the
            // sink and return a benign Ok (the value is discarded anyway).
            let _ = sink.add_error(e);
            return Ok(VotingVoteCommitments {
                bundle_index,
                commitments: Vec::new(),
            });
        }
    };
    Ok(commitments.into())
}

/// Reconstructs the chain-ready wire JSON for a committed vote.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_vote_wire_json(
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
    c: &Coin,
) -> Result<String> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    voting::vote_wire_json(
        c.get_pool()?,
        &wallet_id,
        &round_id,
        bundle_index,
        proposal_id,
    )
    .await
}

/// Atomically records a cast-vote transaction hash with idempotency checks, so
/// a restart between broadcast and confirmation resumes via `PollVote`.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_mark_vote_submitted(
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
    tx_hash: &str,
    c: &Coin,
) -> Result<()> {
    let account = c.account;
    let round_id = round_id.to_string();
    let tx_hash = tx_hash.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    db.mark_vote_submitted(&round_id, bundle_index, proposal_id, &tx_hash)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Share flow
// ---------------------------------------------------------------------------

/// Records a helper-share submission (derives the nullifier from recovery
/// state).
#[cfg_attr(feature = "flutter", frb)]
#[allow(clippy::too_many_arguments)]
pub async fn voting_share_record(
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
    share_index: u32,
    sent_to_urls: Vec<String>,
    submit_at: u64,
    c: &Coin,
) -> Result<()> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    zcash_voting::share::record(
        &db,
        &round_id,
        bundle_index,
        proposal_id,
        share_index,
        &sent_to_urls,
        submit_at,
    )
    .await?;
    Ok(())
}

/// Lists unconfirmed helper-share records for a round.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_share_unconfirmed(
    round_id: &str,
    c: &Coin,
) -> Result<Vec<VotingShareDelegationRecord>> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    Ok(zcash_voting::share::unconfirmed(&db, &mut *connection, &round_id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

/// Marks one helper-share record confirmed.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_share_confirm(
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
    share_index: u32,
    c: &Coin,
) -> Result<()> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    zcash_voting::share::confirm(&db, &round_id, bundle_index, proposal_id, share_index).await?;
    Ok(())
}

/// Adds helper URLs to an existing share record after resubmission.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_share_add_servers(
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
    share_index: u32,
    new_urls: Vec<String>,
    c: &Coin,
) -> Result<()> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    zcash_voting::share::add_sent_servers(
        &db,
        &round_id,
        bundle_index,
        proposal_id,
        share_index,
        &new_urls,
    )
    .await?;
    Ok(())
}

/// Reconstructs one helper-share payload as helper wire JSON from the
/// persisted commitment bundle.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_share_wire_json(
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
    share_index: u32,
    vc_tree_position: Option<u64>,
    submit_at: u64,
    c: &Coin,
) -> Result<String> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    voting::share_wire_json(
        c.get_pool()?,
        &wallet_id,
        &round_id,
        bundle_index,
        proposal_id,
        share_index,
        vc_tree_position,
        submit_at,
    )
    .await
}

/// Best-effort pre-sync of the vote commitment tree for a round, returning
/// the latest synced tree height. Requires the round to exist locally (it is
/// created by the first prepare); callers may ignore failures.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_sync_tree(
    round_id: &str,
    vote_node_url: &str,
    c: &Coin,
) -> Result<u32> {
    let account = c.account;
    let round_id = round_id.to_string();
    let vote_node_url = vote_node_url.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    Ok(zcash_voting::precompute::sync_vote_tree(&db, &round_id, &vote_node_url).await?)
}

/// Computes the share tracking plan for a round: summary counts, next poll
/// delay, last-moment flag, and freshly planned submissions (with local
/// entropy) for the unconfirmed shares.
#[cfg_attr(feature = "flutter", frb)]
/// One share of a confirmed vote pending helper submission. First-pass
/// submission must enumerate from the confirmed votes' recovery bundles —
/// the `voting_share_delegations` rows only exist after a submission
/// records them.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingShareSubmissionPayload {
    pub bundle_index: u32,
    pub proposal_id: u32,
    pub share_index: u32,
    pub vc_tree_position: Option<u64>,
}

/// Enumerates the share payloads of the round's confirmed votes — the
/// first-pass submission source.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_share_payloads(
    round_id: &str,
    c: &Coin,
) -> Result<Vec<VotingShareSubmissionPayload>> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    let snapshot = zcash_voting::recovery::round_snapshot(&db, &mut *connection, &round_id).await?;
    let recorded: std::collections::BTreeSet<(u32, u32, u32)> = db
        .share_phases(&mut *connection, &round_id)
        .await?
        .into_iter()
        .map(|(b, p, s, _)| (b, p, s))
        .collect();
    let mut payloads = Vec::new();
    for vote in snapshot.votes {
        if vote.phase != zcash_voting::phases::VotePhase::Confirmed {
            continue;
        }
        let Some(bundle) =
            zcash_voting::vote::recovery_bundle(&db, &round_id, vote.bundle_index, vote.proposal_id)
                .await?
        else {
            continue;
        };
        for payload in zcash_voting::share::recover_payloads(&bundle)? {
            // Skip shares already recorded — a resume must not re-send them
            // (the tracking loop polls the helpers for their confirmations).
            if recorded.contains(&(
                vote.bundle_index,
                vote.proposal_id,
                payload.enc_share.share_index,
            )) {
                continue;
            }
            payloads.push(VotingShareSubmissionPayload {
                bundle_index: vote.bundle_index,
                proposal_id: vote.proposal_id,
                share_index: payload.enc_share.share_index,
                vc_tree_position: vote.vc_tree_position.map(|p| p as u64),
            });
        }
    }
    Ok(payloads)
}

/// Count-based share submission plans (submitAt + target servers per share),
/// mirroring vizor's `planShareSubmissions`: policy-sized CSPRNG entropy
/// drawn per call, timing from the round's ceremony start / vote end.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_share_plans(
    share_count: u32,
    server_urls: Vec<String>,
    now: u64,
    vote_end: u64,
    ceremony_start: u64,
    single_share: bool,
    c: &Coin,
) -> Result<Vec<VotingSharePlanItem>> {
    let buffer =
        zcash_voting::share_policy::last_moment_buffer_seconds(ceremony_start, vote_end);
    let share_count = share_count as usize;
    let required = zcash_voting::share_policy::share_submission_random_bytes_required(
        share_count,
        server_urls.len(),
        now,
        vote_end,
        buffer,
        single_share,
    );
    let mut submit_at_random_bytes = vec![0u8; required.submit_at_random_bytes];
    let mut server_random_bytes = vec![0u8; required.server_random_bytes];
    OsRng
        .try_fill_bytes(&mut submit_at_random_bytes)
        .map_err(|e| anyhow!("failed to draw submit_at entropy: {e}"))?;
    OsRng
        .try_fill_bytes(&mut server_random_bytes)
        .map_err(|e| anyhow!("failed to draw share-server entropy: {e}"))?;
    let plans = zcash_voting::share_policy::plan_share_submissions(
        share_count,
        &server_urls,
        now,
        vote_end,
        buffer,
        single_share,
        &submit_at_random_bytes,
        &server_random_bytes,
    )?;
    Ok(plans
        .into_iter()
        .map(|p| VotingSharePlanItem {
            submit_at: p.submit_at,
            target_count: p.target_count,
            target_servers: p.target_servers,
        })
        .collect())
}

pub async fn voting_share_plan(
    round_id: &str,
    now: u64,
    ceremony_start: u64,
    vote_end: Option<u64>,
    server_urls: Vec<String>,
    single_share: bool,
    c: &Coin,
) -> Result<VotingSharePlan> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    let shares = zcash_voting::share::unconfirmed(&db, &mut *connection, &round_id).await?;

    let policy = ShareTimingPolicy::default();
    let summary = zcash_voting::share_policy::summarize_share_tracking(
        &shares,
        now,
        vote_end,
        policy,
    );
    let next_tracking_delay_secs =
        zcash_voting::share_policy::next_tracking_delay_seconds(&shares, now, policy);
    let last_moment = vote_end.is_some_and(|vote_end| {
        zcash_voting::share_policy::is_last_moment(now, ceremony_start, vote_end)
    });

    let submissions = match vote_end {
        Some(vote_end) if !shares.is_empty() => {
            let mut submit_at_random_bytes = vec![0u8; 512];
            let mut server_random_bytes = vec![0u8; 512];
            OsRng.fill_bytes(&mut submit_at_random_bytes);
            OsRng.fill_bytes(&mut server_random_bytes);
            zcash_voting::share_policy::plan_share_submissions(
                shares.len(),
                &server_urls,
                now,
                vote_end,
                zcash_voting::share_policy::last_moment_buffer_seconds(ceremony_start, vote_end),
                single_share,
                &submit_at_random_bytes,
                &server_random_bytes,
            )?
            .into_iter()
            .map(|p| VotingSharePlanItem {
                submit_at: p.submit_at,
                target_count: p.target_count,
                target_servers: p.target_servers,
            })
            .collect()
        }
        _ => Vec::new(),
    };

    Ok(VotingSharePlan {
        summary: summary.into(),
        next_tracking_delay_secs,
        last_moment,
        submissions,
    })
}

// ---------------------------------------------------------------------------
// Config resolution
// ---------------------------------------------------------------------------

/// Resolves and authenticates the voting config for a source URL.
///
/// The wallet owns transport: it fetches the static bytes, learns the dynamic
/// URL, fetches the dynamic bytes, then Rust authenticates both and classifies
/// the config switch against the previously resolved summary. The result is
/// cached in the props table so [`voting_config_cached`] can serve as a
/// last-good fallback.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_config_resolve(source: &str, c: &Coin) -> Result<VotingConfig> {
    let source = source.to_string();
    let proxy = votechain_proxy(c);
    let mut connection = c.get_connection().await?;

    let static_bytes = crate::net::votechain::fetch_bytes(&source, &proxy).await?;
    let resolved_static =
        zcash_voting::config::resolve_static_voting_config(&source, &static_bytes)?;
    let dynamic_bytes = crate::net::votechain::fetch_bytes(
        &resolved_static.dynamic_config_url,
        &proxy,
    )
    .await?;
    let resolved = zcash_voting::config::resolve_dynamic_voting_config(
        resolved_static,
        &dynamic_bytes,
        zcash_voting::config::ResolveVotingConfigOptions::default(),
    )?;

    let previous = crate::db::get_prop(
        &mut connection,
        &format!("voting_config_prev:{source}"),
    )
    .await?
    .map(|json| {
        serde_json::from_str::<zcash_voting::config::ResolvedVotingConfigSummary>(&json)
            .map_err(anyhow::Error::from)
    })
    .transpose()?;
    let decision = zcash_voting::config::decide_config_switch(
        previous.clone(),
        zcash_voting::config::ResolvedVotingConfigSummary::from(&resolved),
    );

    let config = VotingConfig::from_resolved(source.clone(), &resolved, decision.kind);
    let fork_json = serde_json::to_string(&resolved)?;
    crate::db::put_prop(&mut connection, &format!("voting_config:{source}"), &fork_json)
        .await?;
    let mirror_json = serde_json::to_string(&config)?;
    crate::db::put_prop(
        &mut connection,
        &format!("voting_config_mirror:{source}"),
        &mirror_json,
    )
    .await?;
    let prev_json = serde_json::to_string(
        &zcash_voting::config::ResolvedVotingConfigSummary::from(&resolved),
    )?;
    crate::db::put_prop(
        &mut connection,
        &format!("voting_config_prev:{source}"),
        &prev_json,
    )
    .await?;
    Ok(config)
}

/// Returns the last cached resolved config for a source URL, if any.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_config_cached(source: &str, c: &Coin) -> Result<Option<VotingConfig>> {
    let source = source.to_string();
    let mut connection = c.get_connection().await?;
    let Some(json) =
        crate::db::get_prop(&mut connection, &format!("voting_config_mirror:{source}")).await?
    else {
        return Ok(None);
    };
    Ok(Some(serde_json::from_str(&json)?))
}

/// Builds the round params JSON for `delegation_prepare` from the cached
/// authenticated config plus chain-reported snapshot fields (`ea_pk` is
/// pinned to the authenticated config, so a stale endpoint cannot steer
/// voting to the wrong authority or roots).
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_round_params_json(
    source: &str,
    round_id: &str,
    snapshot_height: u64,
    nc_root: Vec<u8>,
    nullifier_imt_root: Vec<u8>,
    c: &Coin,
) -> Result<String> {
    let source = source.to_string();
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let json = crate::db::get_prop(&mut connection, &format!("voting_config:{source}"))
        .await?
        .ok_or_else(|| anyhow!("no cached voting config for source {source}; resolve it first"))?;
    let config: zcash_voting::config::ResolvedVotingConfig = serde_json::from_str(&json)?;
    let params = config.trusted_voting_round_params(
        round_id,
        snapshot_height,
        nc_root,
        nullifier_imt_root,
    )?;
    Ok(serde_json::to_string(&params)?)
}

/// Clears the cached resolved configs (all sources).
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_config_clear_cache(c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::delete_prop_prefix(&mut connection, "voting_config:").await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Vote casting flow
// ---------------------------------------------------------------------------

/// Syncs the vote-authority-note tree and derives this bundle's VAN witness.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_van_witness(
    round_id: &str,
    bundle_index: u32,
    vote_node_url: &str,
    c: &Coin,
) -> Result<VotingVanWitness> {
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, c.account).await?;
    let witness = voting::vote_van_witness(
        c.get_pool()?,
        &wallet_id,
        round_id,
        bundle_index,
        vote_node_url,
    )
    .await?;
    Ok(witness.into())
}

/// Commits a batch of vote drafts for one bundle (hotkey-signed).
///
/// Chains the VAN witness derivation internally, so this may be called right
/// after `voting_van_witness` or standalone.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_commit(
    round_id: &str,
    bundle_index: u32,
    drafts_json: &str,
    vote_node_url: &str,
    c: &Coin,
) -> Result<VotingVoteCommitments> {
    let drafts: Vec<zcash_voting::prelude::DraftVote> = serde_json::from_str(drafts_json)?;
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, c.account).await?;
    let hotkey = voting::voting_hotkey_load(
        &mut connection,
        voting::voting_network(&c.network())?,
    )
    .await?;

    let witness = voting::vote_van_witness(
        c.get_pool()?,
        &wallet_id,
        round_id,
        bundle_index,
        vote_node_url,
    )
    .await?;
    let commitments = voting::commit_votes(
        c.get_pool()?,
        &wallet_id,
        round_id,
        bundle_index,
        &drafts,
        &witness,
        &hotkey,
    )
    .await?;
    Ok(commitments.into())
}

/// Returns the chain-ready vote submission and helper-share payloads for one
/// committed vote.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_payloads(
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
    c: &Coin,
) -> Result<VotingVotePayloads> {
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, c.account).await?;
    let (submission, share_payloads) = voting::vote_payloads(
        c.get_pool()?,
        &wallet_id,
        round_id,
        bundle_index,
        proposal_id,
    )
    .await?;
    Ok(VotingVotePayloads {
        submission: submission.into(),
        share_payloads: share_payloads.iter().map(Into::into).collect(),
    })
}

/// Records successful vote-chain and helper-share submissions for one vote.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_record_execution(
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
    vote_tx_hash: &str,
    vc_tree_position: u64,
    share_deliveries_json: &str,
    c: &Coin,
) -> Result<()> {
    let share_deliveries: Vec<VotingShareDelivery> = serde_json::from_str(share_deliveries_json)?;
    let shares: Vec<(u32, Vec<String>, u64, bool)> = share_deliveries
        .into_iter()
        .map(|d| (d.share_index, d.sent_to_urls, d.submit_at, d.confirmed))
        .collect();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, c.account).await?;
    voting::record_vote_execution(
        c.get_pool()?,
        &wallet_id,
        round_id,
        bundle_index,
        proposal_id,
        vote_tx_hash,
        vc_tree_position,
        &shares,
    )
    .await
}

/// Records a confirmed cast-vote transaction.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_confirm(
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
    tx_hash: &str,
    events_json: &str,
    c: &Coin,
) -> Result<VotingVoteConfirmation> {
    let events: Vec<TxEvent> = serde_json::from_str(events_json)?;
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, c.account).await?;
    let confirmation = voting::confirm_vote(
        c.get_pool()?,
        &wallet_id,
        round_id,
        bundle_index,
        proposal_id,
        tx_hash,
        &events,
    )
    .await?;
    Ok(confirmation.into())
}

// ---------------------------------------------------------------------------
// Recovery / plan mirrors
// ---------------------------------------------------------------------------

/// Round row from the voting DB (rounds list).
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingRoundInfo {
    pub round_id: String,
    pub network: String,
    pub snapshot_height: u64,
    pub hotkey_address: Option<String>,
    pub eligible_weight_zatoshi: Option<u64>,
    pub bundle_count: u32,
    pub created_at: u64,
}

/// One remaining unit of recovery work for a round.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingNextStep {
    pub kind: String,
    pub bundle_index: u32,
    pub proposal_id: u32,
    pub choice: u32,
    pub share_index: u32,
}

/// Durable delegation state for one eligible bundle.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingDelegationStatus {
    pub bundle_index: u32,
    pub phase: String,
    pub tx_hash: Option<String>,
}

/// Display choice for one proposal in a completed round.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingCompletedVoteChoice {
    pub proposal_id: u32,
    pub choice: Option<u32>,
}

/// Read-only display summary for a locally completed vote.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingCompletedVoteDisplay {
    pub choices: Vec<VotingCompletedVoteChoice>,
    pub voted_at: Option<u64>,
}

/// Derived resume state for one round.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingRoundPlan {
    pub round_id: String,
    pub pending_recovery: bool,
    pub next_steps: Vec<VotingNextStep>,
    pub open_proposals: Vec<u32>,
    pub all_decided: bool,
    pub delegation_statuses: Vec<VotingDelegationStatus>,
    pub blocking_recovery: bool,
    pub blocking_share_work: bool,
    pub hotkey_bound: bool,
    pub completed_vote_artifact: bool,
    pub completed_for_display: bool,
    pub completed_vote_display: Option<VotingCompletedVoteDisplay>,
    pub needs_draft_setup: bool,
    pub primary_action: String,
}

/// Delegation recovery state for one bundle.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingDelegationRecovery {
    pub bundle_index: u32,
    pub phase: String,
    pub workflow_phase: String,
    pub tx_hash: Option<String>,
    pub van_leaf_position: Option<u32>,
}

/// Vote recovery state for one vote key.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingVoteRecovery {
    pub bundle_index: u32,
    pub proposal_id: u32,
    pub choice: u32,
    pub phase: String,
    pub workflow_phase: String,
    pub tx_hash: Option<String>,
    pub vc_tree_position: Option<u64>,
    pub has_commitment_bundle: bool,
}

/// Share recovery state for one delegated share.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingShareWorkflow {
    pub bundle_index: u32,
    pub proposal_id: u32,
    pub share_index: u32,
    pub phase: String,
}

/// A share delegation record from the local DB.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingShareDelegationRecord {
    pub round_id: String,
    pub bundle_index: u32,
    pub proposal_id: u32,
    pub share_index: u32,
    pub sent_to_urls: Vec<String>,
    pub nullifier: Vec<u8>,
    pub confirmed: bool,
    pub submit_at: u64,
    pub created_at: u64,
}

/// Full read-only recovery snapshot for one round.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingRoundRecovery {
    pub round_id: String,
    pub bundle_count: u32,
    pub delegation: Vec<VotingDelegationRecovery>,
    pub votes: Vec<VotingVoteRecovery>,
    pub shares: Vec<VotingShareWorkflow>,
    pub share_delegations: Vec<VotingShareDelegationRecord>,
    pub unconfirmed_share_delegations: Vec<VotingShareDelegationRecord>,
}

/// The voter's terminal decision for one proposal.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingBallotIntent {
    pub proposal_id: u32,
    pub skipped: bool,
    pub choice: Option<u32>,
}

/// Delegation proof/signing progress event, one-to-one with the fork's
/// `DelegationProgress`. The bookend variants (`SelectingNotes`,
/// `SigningPayload`, `PayloadReady`) are emitted by the host wrapper; the
/// PCZT/proof stages come from the library.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VotingDelegationProgress {
    SelectingNotes,
    PcztBuilding,
    PcztBuilt,
    ProofStarting,
    ProofProgress { progress: f64 },
    ProofComplete,
    SigningPayload,
    PayloadReady,
}

impl From<DelegationProgress> for VotingDelegationProgress {
    fn from(p: DelegationProgress) -> Self {
        match p {
            DelegationProgress::SelectingNotes => Self::SelectingNotes,
            DelegationProgress::PcztBuilding => Self::PcztBuilding,
            DelegationProgress::PcztBuilt => Self::PcztBuilt,
            DelegationProgress::ProofStarting => Self::ProofStarting,
            DelegationProgress::ProofProgress(progress) => Self::ProofProgress { progress },
            DelegationProgress::ProofComplete => Self::ProofComplete,
            DelegationProgress::SigningPayload => Self::SigningPayload,
            DelegationProgress::PayloadReady => Self::PayloadReady,
            _ => Self::PcztBuilding,
        }
    }
}

/// Cast-vote commitment stage event, one-to-one with the fork's
/// `VoteCommitStage`.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VotingVoteCommitStage {
    ProofStarting {
        proposal_id: u32,
        bundle_index: u32,
    },
    ProofProgress {
        proposal_id: u32,
        bundle_index: u32,
        progress: f64,
    },
    SharePayloadsBuilding {
        proposal_id: u32,
        bundle_index: u32,
    },
    Signing {
        proposal_id: u32,
        bundle_index: u32,
    },
}

impl From<zcash_voting::vote::VoteCommitStage> for VotingVoteCommitStage {
    fn from(s: zcash_voting::vote::VoteCommitStage) -> Self {
        match s {
            zcash_voting::vote::VoteCommitStage::ProofStarting {
                proposal_id,
                bundle_index,
            } => Self::ProofStarting {
                proposal_id,
                bundle_index,
            },
            zcash_voting::vote::VoteCommitStage::ProofProgress {
                proposal_id,
                bundle_index,
                progress,
            } => Self::ProofProgress {
                proposal_id,
                bundle_index,
                progress,
            },
            zcash_voting::vote::VoteCommitStage::SharePayloadsBuilding {
                proposal_id,
                bundle_index,
            } => Self::SharePayloadsBuilding {
                proposal_id,
                bundle_index,
            },
            zcash_voting::vote::VoteCommitStage::Signing {
                proposal_id,
                bundle_index,
            } => Self::Signing {
                proposal_id,
                bundle_index,
            },
            _ => Self::Signing {
                proposal_id: 0,
                bundle_index: 0,
            },
        }
    }
}

/// Share tracking summary, one-to-one with the fork's `ShareTrackingSummary`.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingShareTrackingSummary {
    pub total: u64,
    pub confirmed: u64,
    pub waiting: u64,
    pub ready: u64,
    pub overdue: u64,
}

/// One planned helper-share submission.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingSharePlanItem {
    pub submit_at: u64,
    pub target_count: u32,
    pub target_servers: Vec<String>,
}

/// The share tracking plan for a round: summary, next poll delay, last-moment
/// flag, and freshly planned submissions for the unconfirmed shares.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingSharePlan {
    pub summary: VotingShareTrackingSummary,
    pub next_tracking_delay_secs: Option<u64>,
    pub last_moment: bool,
    pub submissions: Vec<VotingSharePlanItem>,
}

// ---------------------------------------------------------------------------
// Recovery / plan conversions
// ---------------------------------------------------------------------------

fn fork_network_string(network: VotingNetwork) -> String {
    match network {
        VotingNetwork::Mainnet => "mainnet".to_string(),
        VotingNetwork::Testnet => "testnet".to_string(),
        VotingNetwork::Regtest => "regtest".to_string(),
    }
}

impl From<ForkRoundInfo> for VotingRoundInfo {
    fn from(r: ForkRoundInfo) -> Self {
        Self {
            round_id: r.round_id,
            network: fork_network_string(r.network),
            snapshot_height: r.snapshot_height,
            hotkey_address: r.hotkey_address,
            eligible_weight_zatoshi: r.eligible_weight,
            bundle_count: r.bundle_count,
            created_at: r.created_at,
        }
    }
}

impl From<NextStep> for VotingNextStep {
    fn from(step: NextStep) -> Self {
        let kind = step.kind().to_string();
        let (bundle_index, proposal_id, choice, share_index) = match step {
            NextStep::Delegate { bundle_index } => (bundle_index, 0, 0, 0),
            NextStep::PollDelegation { bundle_index } => (bundle_index, 0, 0, 0),
            NextStep::CastVote {
                bundle_index,
                proposal_id,
                choice,
            } => (bundle_index, proposal_id, choice, 0),
            NextStep::SubmitVote {
                bundle_index,
                proposal_id,
            } => (bundle_index, proposal_id, 0, 0),
            NextStep::PollVote {
                bundle_index,
                proposal_id,
            } => (bundle_index, proposal_id, 0, 0),
            NextStep::SubmitShares {
                bundle_index,
                proposal_id,
                share_index,
            } => (bundle_index, proposal_id, 0, share_index),
            NextStep::ConfirmShare {
                bundle_index,
                proposal_id,
                share_index,
            } => (bundle_index, proposal_id, 0, share_index),
            _ => (0, 0, 0, 0),
        };
        Self {
            kind,
            bundle_index,
            proposal_id,
            choice,
            share_index,
        }
    }
}

impl From<ForkRoundPlan> for VotingRoundPlan {
    fn from(plan: ForkRoundPlan) -> Self {
        Self {
            round_id: plan.round_id,
            pending_recovery: plan.pending_recovery,
            next_steps: plan.next_steps.into_iter().map(Into::into).collect(),
            open_proposals: plan.open_proposals,
            all_decided: plan.all_decided,
            delegation_statuses: plan
                .delegation_statuses
                .into_iter()
                .map(|d| VotingDelegationStatus {
                    bundle_index: d.bundle_index,
                    phase: d.phase.as_str().to_string(),
                    tx_hash: d.tx_hash,
                })
                .collect(),
            blocking_recovery: plan.blocking_recovery,
            blocking_share_work: plan.blocking_share_work,
            hotkey_bound: plan.hotkey_bound,
            completed_vote_artifact: plan.completed_vote_artifact,
            completed_for_display: plan.completed_for_display,
            completed_vote_display: plan.completed_vote_display.map(|d| VotingCompletedVoteDisplay {
                choices: d
                    .choices
                    .into_iter()
                    .map(|c| VotingCompletedVoteChoice {
                        proposal_id: c.proposal_id,
                        choice: c.choice,
                    })
                    .collect(),
                voted_at: d.voted_at,
            }),
            needs_draft_setup: plan.needs_draft_setup,
            primary_action: plan.primary_action.as_str().to_string(),
        }
    }
}

impl From<ForkDelegationRecovery> for VotingDelegationRecovery {
    fn from(r: ForkDelegationRecovery) -> Self {
        Self {
            bundle_index: r.bundle_index,
            phase: r.phase.as_str().to_string(),
            workflow_phase: r.workflow_phase().as_str().to_string(),
            tx_hash: r.tx_hash,
            van_leaf_position: r.van_leaf_position,
        }
    }
}

impl From<ForkVoteRecovery> for VotingVoteRecovery {
    fn from(r: ForkVoteRecovery) -> Self {
        Self {
            bundle_index: r.bundle_index,
            proposal_id: r.proposal_id,
            choice: r.choice,
            phase: r.phase.as_str().to_string(),
            workflow_phase: r.workflow_phase().as_str().to_string(),
            tx_hash: r.tx_hash,
            vc_tree_position: r.vc_tree_position,
            has_commitment_bundle: r.has_commitment_bundle,
        }
    }
}

impl From<ForkShareWorkflow> for VotingShareWorkflow {
    fn from(s: ForkShareWorkflow) -> Self {
        Self {
            bundle_index: s.bundle_index,
            proposal_id: s.proposal_id,
            share_index: s.share_index,
            phase: s.phase.as_str().to_string(),
        }
    }
}

impl From<ForkShareDelegationRecord> for VotingShareDelegationRecord {
    fn from(r: ForkShareDelegationRecord) -> Self {
        Self {
            round_id: r.round_id,
            bundle_index: r.bundle_index,
            proposal_id: r.proposal_id,
            share_index: r.share_index,
            sent_to_urls: r.sent_to_urls,
            nullifier: r.nullifier,
            confirmed: r.confirmed,
            submit_at: r.submit_at,
            created_at: r.created_at,
        }
    }
}

impl From<ForkRoundRecovery> for VotingRoundRecovery {
    fn from(s: ForkRoundRecovery) -> Self {
        Self {
            round_id: s.round_id,
            bundle_count: s.bundle_count,
            delegation: s.delegation.into_iter().map(Into::into).collect(),
            votes: s.votes.into_iter().map(Into::into).collect(),
            shares: s.shares.into_iter().map(Into::into).collect(),
            share_delegations: s.share_delegations.into_iter().map(Into::into).collect(),
            unconfirmed_share_delegations: s
                .unconfirmed_share_delegations
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<(u32, Decision)> for VotingBallotIntent {
    fn from((proposal_id, decision): (u32, Decision)) -> Self {
        match decision {
            Decision::Choice(choice) => Self {
                proposal_id,
                skipped: false,
                choice: Some(choice),
            },
            Decision::Skipped => Self {
                proposal_id,
                skipped: true,
                choice: None,
            },
        }
    }
}

impl From<zcash_voting::share_policy::ShareTrackingSummary> for VotingShareTrackingSummary {
    fn from(s: zcash_voting::share_policy::ShareTrackingSummary) -> Self {
        Self {
            total: s.total,
            confirmed: s.confirmed,
            waiting: s.waiting,
            ready: s.ready,
            overdue: s.overdue,
        }
    }
}

/// Endpoint advertised by a voting service config.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingServiceEndpoint {
    pub url: String,
    pub label: String,
}

/// Round authenticated by the dynamic voting config.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingConfigRound {
    pub round_id: String,
    pub ea_pk: Vec<u8>,
}

/// Authenticated dynamic voting config, ready for wallet use.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingConfig {
    pub source: String,
    pub source_fingerprint: String,
    pub trusted_key_fingerprint: String,
    pub switch_kind: String,
    pub vote_servers: Vec<VotingServiceEndpoint>,
    pub pir_servers: Vec<VotingServiceEndpoint>,
    pub pir_layout: Option<VotingPirLayout>,
    pub rounds: Vec<VotingConfigRound>,
}

fn config_switch_kind_string(kind: zcash_voting::config::ConfigSwitchKind) -> String {
    match kind {
        zcash_voting::config::ConfigSwitchKind::Unchanged => "unchanged".to_string(),
        zcash_voting::config::ConfigSwitchKind::InitialLoad => "initial_load".to_string(),
        zcash_voting::config::ConfigSwitchKind::SameChainServiceUpdate => {
            "same_chain_service_update".to_string()
        }
        zcash_voting::config::ConfigSwitchKind::NewChainOrRound => "new_chain_or_round".to_string(),
        zcash_voting::config::ConfigSwitchKind::ProtocolChanged => "protocol_changed".to_string(),
    }
}

impl VotingConfig {
    fn from_resolved(
        source: String,
        resolved: &zcash_voting::config::ResolvedVotingConfig,
        switch_kind: zcash_voting::config::ConfigSwitchKind,
    ) -> Self {
        Self {
            source,
            source_fingerprint: resolved.source_fingerprint.clone(),
            trusted_key_fingerprint: resolved.trusted_key_fingerprint.clone(),
            switch_kind: config_switch_kind_string(switch_kind),
            vote_servers: resolved
                .vote_servers
                .iter()
                .map(|e| VotingServiceEndpoint {
                    url: e.url.clone(),
                    label: e.label.clone(),
                })
                .collect(),
            pir_servers: resolved
                .pir_endpoints
                .iter()
                .map(|e| VotingServiceEndpoint {
                    url: e.url.clone(),
                    label: e.label.clone(),
                })
                .collect(),
            pir_layout: if resolved.pir_layout == zcash_voting::config::PirLayout::UNKNOWN {
                None
            } else {
                Some(resolved.pir_layout.into())
            },
            rounds: resolved
                .authenticated_rounds
                .iter()
                .map(|r| VotingConfigRound {
                    round_id: r.round_id.clone(),
                    ea_pk: r.ea_pk.clone(),
                })
                .collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// Recovery / plan reads
// ---------------------------------------------------------------------------

/// Lists rounds persisted in the voting DB for the current wallet.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_rounds(c: &Coin) -> Result<Vec<VotingRoundInfo>> {
    let account = c.account;
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    let rounds = db.rounds(&mut *connection).await?;
    Ok(rounds.into_iter().map(Into::into).collect())
}

/// Returns the derived resume plan for a round (the ordered work that remains
/// after any restart; empty `next_steps` with `primary_action == "done"` means
/// the round is complete for this wallet).
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_plan(round_id: &str, proposal_ids: Vec<u32>, c: &Coin) -> Result<VotingRoundPlan> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    let plan = zcash_voting::session::resume_plan(&db, &mut *connection, &round_id, &proposal_ids).await?;
    Ok(plan.into())
}

/// Returns the full read-only recovery snapshot for a round.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_recovery(round_id: &str, c: &Coin) -> Result<VotingRoundRecovery> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    let snapshot = zcash_voting::recovery::round_snapshot(&db, &mut *connection, &round_id).await?;
    Ok(snapshot.into())
}

/// Clears unconfirmed recovery artifacts for a round. Ballot intents, recorded
/// confirmations, and imported delegation capabilities are preserved.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_recovery_clear(round_id: &str, c: &Coin) -> Result<()> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    zcash_voting::recovery::clear(&db, &round_id).await?;
    Ok(())
}

/// Resets process-local vote-tree cache and clears unsigned delegation setup
/// fields for a round (the fork's recovery when a restart after
/// `build_governance_pczt` persisted `pczt_sighash` makes re-setup refuse to
/// overwrite it). Submitted bundles, imported capabilities, and bundles with
/// persisted Keystone signatures are preserved.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_reset_session_state(round_id: &str, c: &Coin) -> Result<()> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    zcash_voting::precompute::reset_voting_session_state(&db, &round_id).await?;
    Ok(())
}

/// Returns the persisted ballot intents for a round, sorted by proposal id.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_ballot_intents(round_id: &str, c: &Coin) -> Result<Vec<VotingBallotIntent>> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &mut *connection, &wallet_id).await?;
    let intents = db.ballot_intents(&mut *connection, &round_id).await?;
    Ok(intents.into_iter().map(Into::into).collect())
}

// ---------------------------------------------------------------------------
// Vote-chain HTTP
// ---------------------------------------------------------------------------

/// Generic vote-chain HTTP response: status code + raw JSON body.
///
/// 404 means "not found" (e.g. a transaction that is not confirmed yet) and
/// 422 means a deterministic chain rejection whose body is a `VotingTxResult`.
/// Only network failures surface as `Err`.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingChainResponse {
    pub status_code: u16,
    pub body: String,
}

/// Voting traffic honors the external-proxy setting (transport 3) only; it is
/// never routed through the Tor/Nym transports in v1.
fn votechain_proxy(c: &Coin) -> String {
    if c.transport == 3 {
        c.proxy.clone()
    } else {
        String::new()
    }
}

/// Lists rounds from the vote server (`{ "rounds": [...] }`).
#[cfg_attr(feature = "flutter", frb)]
pub async fn votechain_list_rounds(base_url: &str, c: &Coin) -> Result<VotingChainResponse> {
    let base_url = base_url.to_string();
    let proxy = votechain_proxy(c);
    let (status_code, body) = crate::net::votechain::list_rounds(&base_url, &proxy).await?;
    Ok(VotingChainResponse { status_code, body })
}

/// Fetches one round's status (`{ "round": ... }` envelope).
#[cfg_attr(feature = "flutter", frb)]
pub async fn votechain_round_status(
    base_url: &str,
    round_id: &str,
    c: &Coin,
) -> Result<VotingChainResponse> {
    let base_url = base_url.to_string();
    let round_id = round_id.to_string();
    let proxy = votechain_proxy(c);
    let (status_code, body) =
        crate::net::votechain::round_status(&base_url, &round_id, &proxy).await?;
    Ok(VotingChainResponse { status_code, body })
}

/// Fetches the round tally envelope.
#[cfg_attr(feature = "flutter", frb)]
pub async fn votechain_round_tally(
    base_url: &str,
    round_id: &str,
    c: &Coin,
) -> Result<VotingChainResponse> {
    let base_url = base_url.to_string();
    let round_id = round_id.to_string();
    let proxy = votechain_proxy(c);
    let (status_code, body) =
        crate::net::votechain::round_tally(&base_url, &round_id, &proxy).await?;
    Ok(VotingChainResponse { status_code, body })
}

/// Broadcasts a delegation transaction to the vote chain.
#[cfg_attr(feature = "flutter", frb)]
pub async fn votechain_submit_delegation(
    base_url: &str,
    submission_json: &str,
    c: &Coin,
) -> Result<VotingChainResponse> {
    let base_url = base_url.to_string();
    let submission_json = submission_json.to_string();
    let proxy = votechain_proxy(c);
    let (status_code, body) =
        crate::net::votechain::submit_delegation(&base_url, &submission_json, &proxy).await?;
    Ok(VotingChainResponse { status_code, body })
}

/// Broadcasts a vote commitment transaction to the vote chain.
#[cfg_attr(feature = "flutter", frb)]
pub async fn votechain_submit_vote(
    base_url: &str,
    submission_json: &str,
    c: &Coin,
) -> Result<VotingChainResponse> {
    let base_url = base_url.to_string();
    let submission_json = submission_json.to_string();
    let proxy = votechain_proxy(c);
    let (status_code, body) =
        crate::net::votechain::submit_vote_commitment(&base_url, &submission_json, &proxy).await?;
    Ok(VotingChainResponse { status_code, body })
}

/// Fetches the on-chain confirmation for a transaction; 404 = not confirmed.
#[cfg_attr(feature = "flutter", frb)]
pub async fn votechain_tx_confirmation(
    base_url: &str,
    tx_hash: &str,
    c: &Coin,
) -> Result<VotingChainResponse> {
    let base_url = base_url.to_string();
    let tx_hash = tx_hash.to_string();
    let proxy = votechain_proxy(c);
    let (status_code, body) =
        crate::net::votechain::tx_confirmation(&base_url, &tx_hash, &proxy).await?;
    Ok(VotingChainResponse { status_code, body })
}

/// Posts one encrypted share to a helper server.
#[cfg_attr(feature = "flutter", frb)]
pub async fn votechain_submit_share(
    server_url: &str,
    payload_json: &str,
    c: &Coin,
) -> Result<VotingChainResponse> {
    let server_url = server_url.to_string();
    let payload_json = payload_json.to_string();
    let proxy = votechain_proxy(c);
    let (status_code, body) =
        crate::net::votechain::submit_share(&server_url, &payload_json, &proxy).await?;
    Ok(VotingChainResponse { status_code, body })
}

/// Resends a previously generated share to a helper server (same endpoint as
/// the initial submission).
#[cfg_attr(feature = "flutter", frb)]
pub async fn votechain_resubmit_share(
    server_url: &str,
    payload_json: &str,
    c: &Coin,
) -> Result<VotingChainResponse> {
    let server_url = server_url.to_string();
    let payload_json = payload_json.to_string();
    let proxy = votechain_proxy(c);
    let (status_code, body) =
        crate::net::votechain::submit_share(&server_url, &payload_json, &proxy).await?;
    Ok(VotingChainResponse { status_code, body })
}

/// Checks whether a helper has confirmed a share identified by its nullifier.
#[cfg_attr(feature = "flutter", frb)]
pub async fn votechain_share_status(
    server_url: &str,
    round_id: &str,
    share_id: &str,
    c: &Coin,
) -> Result<VotingChainResponse> {
    let server_url = server_url.to_string();
    let round_id = round_id.to_string();
    let share_id = share_id.to_string();
    let proxy = votechain_proxy(c);
    let (status_code, body) =
        crate::net::votechain::share_status(&server_url, &round_id, &share_id, &proxy).await?;
    Ok(VotingChainResponse { status_code, body })
}

#[cfg(test)]
mod tests {
    use super::*;
    use zcash_voting::config::{
        AuthenticatedRound, ConfigCondition, ConfigConditionKind, ConfigSwitchKind, PirLayout,
        ResolvedVotingConfig, ServiceEndpoint, SupportedVersions,
    };
    use zcash_voting::phases::{DelegationPhase, SharePhase, VotePhase};
    use zcash_voting::prelude::{
        DelegationConfirmation, DelegationSetup, DelegationSubmission, SharePayload,
        SignedVoteCommitments, VanWitness, VoteConfirmation, VoteSubmission,
    };
    use zcash_voting::session::{
        CompletedVoteChoice, CompletedVoteDisplay, DelegationRecoveryWork,
        DelegationRecoveryWorkKind, DelegationStatus, RoundPlanAction,
    };
    use zcash_voting::share_policy::ShareTrackingSummary;
    use zcash_voting::vote::{SignedVoteCommitment, VoteCommitStage};
    use zcash_voting::WireEncryptedShare;

    fn assert_round_trips<T>(v: T)
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let json = serde_json::to_string(&v).unwrap();
        let back: T = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }

    fn coin(transport: u8, proxy: &str) -> Coin {
        Coin {
            coin: 0,
            account: 0,
            db_filepath: String::new(),
            url: String::new(),
            server_type: 0,
            transport,
            proxy: proxy.to_string(),
        }
    }

    #[test]
    fn voting_pir_layout_from_and_to_fork_round_trip() {
        let fork = PirLayout {
            pir_depth: 5,
            tier0_layers: 2,
            tier1_layers: 3,
            poly_len: 2048,
        };
        let mirror = VotingPirLayout::from(fork);
        assert_eq!(
            mirror,
            VotingPirLayout {
                pir_depth: 5,
                tier0_layers: 2,
                tier1_layers: 3,
                poly_len: 2048,
            }
        );
        assert_eq!(mirror.to_fork(), fork);
    }

    #[test]
    fn voting_delegation_setup_from_maps_all_fields() {
        let fork = DelegationSetup {
            pczt_bytes: vec![1, 2],
            pczt_sighash: [3u8; 32],
            rk: [4u8; 32],
            action_index: 65536,
            action_bytes: vec![5],
            tx1_effects: vec![6],
        };
        assert_eq!(
            VotingDelegationSetup::from(fork),
            VotingDelegationSetup {
                pczt_bytes: vec![1, 2],
                pczt_sighash: vec![3u8; 32],
                rk: vec![4u8; 32],
                action_index: 65536,
                action_bytes: vec![5],
                tx1_effects: vec![6],
            }
        );
    }

    #[test]
    fn voting_delegation_submission_from_maps_all_fields() {
        let fork = DelegationSubmission {
            proof: vec![1],
            rk: [2u8; 32],
            nf_signed: [3u8; 32],
            cmx_new: [4u8; 32],
            gov_comm: [5u8; 32],
            gov_nullifiers: [[6u8; 32], [7u8; 32], [8u8; 32], [9u8; 32], [10u8; 32]],
            alpha: [11u8; 32],
            vote_round_id: "round-1".to_string(),
            spend_auth_sig: [12u8; 64],
            sighash: [13u8; 32],
            tx1_effects: vec![14],
        };
        assert_eq!(
            VotingDelegationSubmission::from(fork),
            VotingDelegationSubmission {
                proof: vec![1],
                rk: vec![2u8; 32],
                nf_signed: vec![3u8; 32],
                cmx_new: vec![4u8; 32],
                gov_comm: vec![5u8; 32],
                gov_nullifiers: vec![
                    vec![6u8; 32],
                    vec![7u8; 32],
                    vec![8u8; 32],
                    vec![9u8; 32],
                    vec![10u8; 32],
                ],
                alpha: vec![11u8; 32],
                vote_round_id: "round-1".to_string(),
                spend_auth_sig: vec![12u8; 64],
                sighash: vec![13u8; 32],
                tx1_effects: vec![14],
            }
        );
    }

    #[test]
    fn voting_delegation_confirmation_from_maps_all_fields() {
        let fork = DelegationConfirmation {
            tx_hash: "0xabc".to_string(),
            van_leaf_position: 9,
        };
        assert_eq!(
            VotingDelegationConfirmation::from(fork),
            VotingDelegationConfirmation {
                tx_hash: "0xabc".to_string(),
                van_leaf_position: 9,
            }
        );
    }

    #[test]
    fn voting_vote_confirmation_from_maps_all_fields() {
        let fork = VoteConfirmation {
            tx_hash: "0xdef".to_string(),
            van_leaf_position: 9,
            vc_tree_position: 7,
        };
        assert_eq!(
            VotingVoteConfirmation::from(fork),
            VotingVoteConfirmation {
                tx_hash: "0xdef".to_string(),
                van_leaf_position: 9,
                vc_tree_position: 7,
            }
        );
    }

    #[test]
    fn voting_van_witness_from_maps_all_fields() {
        let fork = VanWitness {
            auth_path: vec![vec![1], vec![2, 3]],
            position: 4,
            anchor_height: 5,
        };
        assert_eq!(
            VotingVanWitness::from(fork),
            VotingVanWitness {
                auth_path: vec![vec![1], vec![2, 3]],
                position: 4,
                anchor_height: 5,
            }
        );
    }

    #[test]
    fn voting_signed_vote_commitments_from_maps_all_fields() {
        let fork = SignedVoteCommitments {
            bundle_index: 3,
            commitments: vec![
                SignedVoteCommitment {
                    proposal_id: 1,
                    choice: 2,
                    vote_round_id: "r".to_string(),
                    van_nullifier: [3u8; 32],
                    vote_authority_note_new: [4u8; 32],
                    vote_commitment: [5u8; 32],
                    proof: vec![6],
                    encrypted_shares: vec![],
                    share_payloads: vec![],
                    anchor_height: 7,
                    shares_hash: [8u8; 32],
                    share_comms: vec![],
                    r_vpk: [9u8; 32],
                    vote_auth_sig: [10u8; 64],
                    commitment_bundle_json: "{}".to_string(),
                },
                SignedVoteCommitment {
                    proposal_id: 11,
                    choice: 12,
                    vote_round_id: "r2".to_string(),
                    van_nullifier: [13u8; 32],
                    vote_authority_note_new: [14u8; 32],
                    vote_commitment: [15u8; 32],
                    proof: vec![16],
                    encrypted_shares: vec![],
                    share_payloads: vec![],
                    anchor_height: 17,
                    shares_hash: [18u8; 32],
                    share_comms: vec![],
                    r_vpk: [19u8; 32],
                    vote_auth_sig: [20u8; 64],
                    commitment_bundle_json: "{\"x\":1}".to_string(),
                },
            ],
        };
        let mirror = VotingVoteCommitments::from(fork);
        assert_eq!(mirror.bundle_index, 3);
        assert_eq!(mirror.commitments.len(), 2);
        assert_eq!(
            mirror.commitments[0],
            VotingSignedVoteCommitment {
                proposal_id: 1,
                choice: 2,
                vote_round_id: "r".to_string(),
                van_nullifier: vec![3u8; 32],
                vote_authority_note_new: vec![4u8; 32],
                vote_commitment: vec![5u8; 32],
                proof: vec![6],
                anchor_height: 7,
                r_vpk: vec![9u8; 32],
                vote_auth_sig: vec![10u8; 64],
                commitment_bundle_json: "{}".to_string(),
            }
        );
        assert_eq!(mirror.commitments[1].commitment_bundle_json, "{\"x\":1}");

        // Empty edge.
        let empty = VotingVoteCommitments::from(SignedVoteCommitments {
            bundle_index: 4,
            commitments: vec![],
        });
        assert!(empty.commitments.is_empty());
    }

    #[test]
    fn voting_encrypted_share_from_maps_all_fields() {
        let fork = WireEncryptedShare {
            c1: vec![1],
            c2: vec![2],
            share_index: 3,
        };
        assert_eq!(
            VotingEncryptedShare::from(&fork),
            VotingEncryptedShare {
                c1: vec![1],
                c2: vec![2],
                share_index: 3,
            }
        );
    }

    #[test]
    fn voting_share_payload_from_maps_all_fields() {
        let fork = SharePayload {
            shares_hash: vec![1],
            proposal_id: 2,
            vote_decision: 3,
            enc_share: WireEncryptedShare {
                c1: vec![4],
                c2: vec![5],
                share_index: 6,
            },
            tree_position: 7,
            all_enc_shares: vec![WireEncryptedShare {
                c1: vec![8],
                c2: vec![9],
                share_index: 10,
            }],
            share_comms: vec![vec![11]],
            primary_blind: vec![12],
        };
        assert_eq!(
            VotingSharePayload::from(&fork),
            VotingSharePayload {
                shares_hash: vec![1],
                proposal_id: 2,
                vote_decision: 3,
                enc_share: VotingEncryptedShare {
                    c1: vec![4],
                    c2: vec![5],
                    share_index: 6,
                },
                tree_position: 7,
                all_enc_shares: vec![VotingEncryptedShare {
                    c1: vec![8],
                    c2: vec![9],
                    share_index: 10,
                }],
                share_comms: vec![vec![11]],
                primary_blind: vec![12],
            }
        );
    }

    #[test]
    fn voting_vote_submission_from_maps_all_fields() {
        let fork = VoteSubmission {
            vote_round_id: "r".to_string(),
            proposal_id: 1,
            van_nullifier: [2u8; 32],
            vote_authority_note_new: [3u8; 32],
            vote_commitment: [4u8; 32],
            proof: vec![5],
            r_vpk: [6u8; 32],
            vote_auth_sig: [7u8; 64],
            anchor_height: 8,
        };
        assert_eq!(
            VotingVoteSubmission::from(fork),
            VotingVoteSubmission {
                vote_round_id: "r".to_string(),
                proposal_id: 1,
                van_nullifier: vec![2u8; 32],
                vote_authority_note_new: vec![3u8; 32],
                vote_commitment: vec![4u8; 32],
                proof: vec![5],
                r_vpk: vec![6u8; 32],
                vote_auth_sig: vec![7u8; 64],
                anchor_height: 8,
            }
        );
    }

    #[test]
    fn delegation_progress_maps_every_variant() {
        let cases = [
            (
                DelegationProgress::SelectingNotes,
                VotingDelegationProgress::SelectingNotes,
            ),
            (
                DelegationProgress::PcztBuilding,
                VotingDelegationProgress::PcztBuilding,
            ),
            (
                DelegationProgress::PcztBuilt,
                VotingDelegationProgress::PcztBuilt,
            ),
            (
                DelegationProgress::ProofStarting,
                VotingDelegationProgress::ProofStarting,
            ),
            (
                DelegationProgress::ProofProgress(0.5),
                VotingDelegationProgress::ProofProgress { progress: 0.5 },
            ),
            (
                DelegationProgress::ProofComplete,
                VotingDelegationProgress::ProofComplete,
            ),
            (
                DelegationProgress::SigningPayload,
                VotingDelegationProgress::SigningPayload,
            ),
            (
                DelegationProgress::PayloadReady,
                VotingDelegationProgress::PayloadReady,
            ),
        ];
        for (fork, expected) in cases {
            assert_eq!(VotingDelegationProgress::from(fork), expected);
        }
    }

    #[test]
    fn vote_commit_stage_maps_every_variant() {
        let cases = [
            (
                VoteCommitStage::ProofStarting {
                    proposal_id: 1,
                    bundle_index: 2,
                },
                VotingVoteCommitStage::ProofStarting {
                    proposal_id: 1,
                    bundle_index: 2,
                },
            ),
            (
                VoteCommitStage::ProofProgress {
                    proposal_id: 1,
                    bundle_index: 2,
                    progress: 0.25,
                },
                VotingVoteCommitStage::ProofProgress {
                    proposal_id: 1,
                    bundle_index: 2,
                    progress: 0.25,
                },
            ),
            (
                VoteCommitStage::SharePayloadsBuilding {
                    proposal_id: 1,
                    bundle_index: 2,
                },
                VotingVoteCommitStage::SharePayloadsBuilding {
                    proposal_id: 1,
                    bundle_index: 2,
                },
            ),
            (
                VoteCommitStage::Signing {
                    proposal_id: 1,
                    bundle_index: 2,
                },
                VotingVoteCommitStage::Signing {
                    proposal_id: 1,
                    bundle_index: 2,
                },
            ),
        ];
        for (fork, expected) in cases {
            assert_eq!(VotingVoteCommitStage::from(fork), expected);
        }
    }

    #[test]
    fn round_info_from_maps_all_fields_and_network_string() {
        for (network, expected_network) in [
            (VotingNetwork::Mainnet, "mainnet"),
            (VotingNetwork::Testnet, "testnet"),
            (VotingNetwork::Regtest, "regtest"),
        ] {
            let fork = ForkRoundInfo {
                round_id: "r1".to_string(),
                network,
                snapshot_height: 100,
                hotkey_address: Some("addr".to_string()),
                eligible_weight: Some(50_000),
                bundle_count: 2,
                created_at: 123,
            };
            let mirror = VotingRoundInfo::from(fork);
            assert_eq!(mirror.round_id, "r1");
            assert_eq!(mirror.network, expected_network);
            assert_eq!(mirror.snapshot_height, 100);
            assert_eq!(mirror.hotkey_address.as_deref(), Some("addr"));
            assert_eq!(mirror.eligible_weight_zatoshi, Some(50_000));
            assert_eq!(mirror.bundle_count, 2);
            assert_eq!(mirror.created_at, 123);
        }

        let fork = ForkRoundInfo {
            round_id: "r2".to_string(),
            network: VotingNetwork::Mainnet,
            snapshot_height: 0,
            hotkey_address: None,
            eligible_weight: None,
            bundle_count: 0,
            created_at: 0,
        };
        let mirror = VotingRoundInfo::from(fork);
        assert_eq!(mirror.hotkey_address, None);
        assert_eq!(mirror.eligible_weight_zatoshi, None);
    }

    #[test]
    fn next_step_maps_every_variant() {
        fn assert_step(
            step: NextStep,
            kind: &str,
            bundle: u32,
            proposal: u32,
            choice: u32,
            share: u32,
        ) {
            let mirror = VotingNextStep::from(step);
            assert_eq!(mirror.kind, kind);
            assert_eq!(mirror.bundle_index, bundle);
            assert_eq!(mirror.proposal_id, proposal);
            assert_eq!(mirror.choice, choice);
            assert_eq!(mirror.share_index, share);
        }

        assert_step(NextStep::Delegate { bundle_index: 1 }, "delegate", 1, 0, 0, 0);
        assert_step(
            NextStep::PollDelegation { bundle_index: 2 },
            "poll_delegation",
            2,
            0,
            0,
            0,
        );
        assert_step(
            NextStep::CastVote {
                bundle_index: 3,
                proposal_id: 4,
                choice: 5,
            },
            "cast_vote",
            3,
            4,
            5,
            0,
        );
        assert_step(
            NextStep::SubmitVote {
                bundle_index: 6,
                proposal_id: 7,
            },
            "submit_vote",
            6,
            7,
            0,
            0,
        );
        assert_step(
            NextStep::PollVote {
                bundle_index: 8,
                proposal_id: 9,
            },
            "poll_vote",
            8,
            9,
            0,
            0,
        );
        assert_step(
            NextStep::SubmitShares {
                bundle_index: 10,
                proposal_id: 11,
                share_index: 12,
            },
            "submit_shares",
            10,
            11,
            0,
            12,
        );
        assert_step(
            NextStep::ConfirmShare {
                bundle_index: 13,
                proposal_id: 14,
                share_index: 15,
            },
            "confirm_share",
            13,
            14,
            0,
            15,
        );
    }

    #[test]
    fn round_plan_from_maps_all_fields() {
        let fork = ForkRoundPlan {
            round_id: "r1".to_string(),
            pending_recovery: true,
            next_steps: vec![
                NextStep::Delegate { bundle_index: 1 },
                NextStep::PollVote {
                    bundle_index: 2,
                    proposal_id: 3,
                },
            ],
            open_proposals: vec![3, 4],
            all_decided: false,
            delegation_statuses: vec![DelegationStatus {
                bundle_index: 1,
                phase: DelegationPhase::Proved,
                tx_hash: Some("t1".to_string()),
            }],
            blocking_recovery: true,
            blocking_share_work: false,
            hotkey_bound: true,
            completed_vote_artifact: true,
            completed_for_display: true,
            completed_vote_display: Some(CompletedVoteDisplay {
                choices: vec![
                    CompletedVoteChoice {
                        proposal_id: 3,
                        choice: Some(1),
                    },
                    CompletedVoteChoice {
                        proposal_id: 4,
                        choice: None,
                    },
                ],
                voted_at: Some(42),
            }),
            needs_draft_setup: false,
            primary_action: RoundPlanAction::Done,
            recovered_delegation_work: vec![DelegationRecoveryWork {
                kind: DelegationRecoveryWorkKind::PollDelegation,
                bundle_index: 1,
                phase: DelegationPhase::Submitted,
                tx_hash: Some("t1".to_string()),
            }],
            recovered_vote_work: vec![],
        };
        let mirror = VotingRoundPlan::from(fork);
        assert_eq!(mirror.round_id, "r1");
        assert!(mirror.pending_recovery);
        assert_eq!(mirror.next_steps.len(), 2);
        assert_eq!(
            mirror.next_steps[0],
            VotingNextStep {
                kind: "delegate".to_string(),
                bundle_index: 1,
                proposal_id: 0,
                choice: 0,
                share_index: 0,
            }
        );
        assert_eq!(mirror.open_proposals, vec![3, 4]);
        assert!(!mirror.all_decided);
        assert_eq!(
            mirror.delegation_statuses,
            vec![VotingDelegationStatus {
                bundle_index: 1,
                phase: "proved".to_string(),
                tx_hash: Some("t1".to_string()),
            }]
        );
        assert!(mirror.blocking_recovery);
        assert!(!mirror.blocking_share_work);
        assert!(mirror.hotkey_bound);
        assert!(mirror.completed_vote_artifact);
        assert!(mirror.completed_for_display);
        assert!(!mirror.needs_draft_setup);
        assert_eq!(mirror.primary_action, "done");
        let display = mirror.completed_vote_display.unwrap();
        assert_eq!(
            display.choices,
            vec![
                VotingCompletedVoteChoice {
                    proposal_id: 3,
                    choice: Some(1),
                },
                VotingCompletedVoteChoice {
                    proposal_id: 4,
                    choice: None,
                },
            ]
        );
        assert_eq!(display.voted_at, Some(42));
    }

    #[test]
    fn delegation_recovery_from_maps_all_fields() {
        let fork = ForkDelegationRecovery {
            bundle_index: 1,
            phase: DelegationPhase::Confirmed,
            tx_hash: None,
            van_leaf_position: Some(2),
        };
        let mirror = VotingDelegationRecovery::from(fork);
        assert_eq!(mirror.bundle_index, 1);
        assert_eq!(mirror.phase, "confirmed");
        assert_eq!(mirror.workflow_phase, "confirmed");
        assert_eq!(mirror.tx_hash, None);
        assert_eq!(mirror.van_leaf_position, Some(2));
    }

    #[test]
    fn vote_recovery_from_maps_all_fields() {
        let fork = ForkVoteRecovery {
            bundle_index: 1,
            proposal_id: 2,
            choice: 3,
            phase: VotePhase::Submitted,
            tx_hash: Some("t".to_string()),
            vc_tree_position: Some(9),
            has_commitment_bundle: true,
        };
        let mirror = VotingVoteRecovery::from(fork);
        assert_eq!(mirror.bundle_index, 1);
        assert_eq!(mirror.proposal_id, 2);
        assert_eq!(mirror.choice, 3);
        assert_eq!(mirror.phase, "submitted");
        assert_eq!(mirror.workflow_phase, "submitted_vote");
        assert_eq!(mirror.tx_hash.as_deref(), Some("t"));
        assert_eq!(mirror.vc_tree_position, Some(9));
        assert!(mirror.has_commitment_bundle);
    }

    #[test]
    fn share_workflow_from_maps_all_fields() {
        let fork = ForkShareWorkflow {
            bundle_index: 1,
            proposal_id: 2,
            share_index: 3,
            phase: SharePhase::Confirmed,
        };
        let mirror = VotingShareWorkflow::from(fork);
        assert_eq!(mirror.bundle_index, 1);
        assert_eq!(mirror.proposal_id, 2);
        assert_eq!(mirror.share_index, 3);
        assert_eq!(mirror.phase, "confirmed");
    }

    #[test]
    fn share_delegation_record_from_maps_all_fields() {
        let fork = ForkShareDelegationRecord {
            round_id: "r".to_string(),
            bundle_index: 1,
            proposal_id: 2,
            share_index: 3,
            sent_to_urls: vec!["u1".to_string()],
            nullifier: vec![1, 2, 3],
            confirmed: true,
            submit_at: 4,
            created_at: 5,
        };
        assert_eq!(
            VotingShareDelegationRecord::from(fork),
            VotingShareDelegationRecord {
                round_id: "r".to_string(),
                bundle_index: 1,
                proposal_id: 2,
                share_index: 3,
                sent_to_urls: vec!["u1".to_string()],
                nullifier: vec![1, 2, 3],
                confirmed: true,
                submit_at: 4,
                created_at: 5,
            }
        );
    }

    #[test]
    fn round_recovery_from_maps_all_fields() {
        let fork = ForkRoundRecovery {
            round_id: "r".to_string(),
            bundle_count: 2,
            delegation: vec![ForkDelegationRecovery {
                bundle_index: 1,
                phase: DelegationPhase::Proved,
                tx_hash: None,
                van_leaf_position: None,
            }],
            votes: vec![ForkVoteRecovery {
                bundle_index: 1,
                proposal_id: 2,
                choice: 3,
                phase: VotePhase::Committed,
                tx_hash: None,
                vc_tree_position: None,
                has_commitment_bundle: true,
            }],
            commitment_bundles: vec![],
            shares: vec![ForkShareWorkflow {
                bundle_index: 1,
                proposal_id: 2,
                share_index: 3,
                phase: SharePhase::Submitted,
            }],
            share_delegations: vec![
                ForkShareDelegationRecord {
                    round_id: "r".to_string(),
                    bundle_index: 1,
                    proposal_id: 2,
                    share_index: 3,
                    sent_to_urls: vec!["u1".to_string()],
                    nullifier: vec![1],
                    confirmed: false,
                    submit_at: 4,
                    created_at: 5,
                },
                ForkShareDelegationRecord {
                    round_id: "r".to_string(),
                    bundle_index: 1,
                    proposal_id: 2,
                    share_index: 4,
                    sent_to_urls: vec![],
                    nullifier: vec![2],
                    confirmed: true,
                    submit_at: 6,
                    created_at: 7,
                },
            ],
            unconfirmed_share_delegations: vec![ForkShareDelegationRecord {
                round_id: "r".to_string(),
                bundle_index: 1,
                proposal_id: 2,
                share_index: 5,
                sent_to_urls: vec![],
                nullifier: vec![3],
                confirmed: false,
                submit_at: 8,
                created_at: 9,
            }],
        };
        let mirror = VotingRoundRecovery::from(fork);
        assert_eq!(mirror.round_id, "r");
        assert_eq!(mirror.bundle_count, 2);
        assert_eq!(mirror.delegation.len(), 1);
        assert_eq!(mirror.votes.len(), 1);
        assert_eq!(mirror.shares.len(), 1);
        assert_eq!(mirror.share_delegations.len(), 2);
        assert_eq!(mirror.unconfirmed_share_delegations.len(), 1);
        assert_eq!(mirror.votes[0].phase, "committed");
        assert_eq!(mirror.shares[0].phase, "submitted");
    }

    #[test]
    fn ballot_intent_from_maps_choice_and_skipped() {
        assert_eq!(
            VotingBallotIntent::from((7u32, Decision::Choice(3))),
            VotingBallotIntent {
                proposal_id: 7,
                skipped: false,
                choice: Some(3),
            }
        );
        assert_eq!(
            VotingBallotIntent::from((7u32, Decision::Skipped)),
            VotingBallotIntent {
                proposal_id: 7,
                skipped: true,
                choice: None,
            }
        );
    }

    #[test]
    fn share_tracking_summary_from_maps_all_fields() {
        let fork = ShareTrackingSummary {
            total: 10,
            confirmed: 6,
            waiting: 2,
            ready: 1,
            overdue: 1,
        };
        assert_eq!(
            VotingShareTrackingSummary::from(fork),
            VotingShareTrackingSummary {
                total: 10,
                confirmed: 6,
                waiting: 2,
                ready: 1,
                overdue: 1,
            }
        );
    }

    #[test]
    fn fork_network_string_maps_each_network() {
        assert_eq!(fork_network_string(VotingNetwork::Mainnet), "mainnet");
        assert_eq!(fork_network_string(VotingNetwork::Testnet), "testnet");
        assert_eq!(fork_network_string(VotingNetwork::Regtest), "regtest");
    }

    #[test]
    fn config_switch_kind_string_maps_each_kind() {
        assert_eq!(config_switch_kind_string(ConfigSwitchKind::Unchanged), "unchanged");
        assert_eq!(config_switch_kind_string(ConfigSwitchKind::InitialLoad), "initial_load");
        assert_eq!(
            config_switch_kind_string(ConfigSwitchKind::SameChainServiceUpdate),
            "same_chain_service_update"
        );
        assert_eq!(
            config_switch_kind_string(ConfigSwitchKind::NewChainOrRound),
            "new_chain_or_round"
        );
        assert_eq!(
            config_switch_kind_string(ConfigSwitchKind::ProtocolChanged),
            "protocol_changed"
        );
    }

    #[test]
    fn votechain_proxy_uses_external_proxy_only_for_transport_three() {
        assert_eq!(
            votechain_proxy(&coin(3, "socks5://127.0.0.1:1080")),
            "socks5://127.0.0.1:1080"
        );
        assert_eq!(votechain_proxy(&coin(3, "")), "");
        assert_eq!(votechain_proxy(&coin(0, "socks5://x")), "");
        assert_eq!(votechain_proxy(&coin(1, "socks5://x")), "");
        assert_eq!(votechain_proxy(&coin(2, "socks5://x")), "");
    }

    fn resolved_config(pir_layout: PirLayout) -> ResolvedVotingConfig {
        ResolvedVotingConfig {
            source_fingerprint: "sf".to_string(),
            trusted_key_fingerprint: "tf".to_string(),
            dynamic_config_fingerprint: "df".to_string(),
            vote_servers: vec![
                ServiceEndpoint {
                    url: "https://v1".to_string(),
                    label: "vote".to_string(),
                },
                ServiceEndpoint {
                    url: "https://v2".to_string(),
                    label: "vote2".to_string(),
                },
            ],
            pir_endpoints: vec![ServiceEndpoint {
                url: "https://p".to_string(),
                label: "pir".to_string(),
            }],
            pir_layout,
            supported_versions: SupportedVersions {
                pir: vec!["1".to_string()],
                vote_protocol: "v1".to_string(),
                tally: "t1".to_string(),
                vote_server: "s1".to_string(),
            },
            authenticated_rounds: vec![
                AuthenticatedRound {
                    round_id: "r1".to_string(),
                    ea_pk: vec![1, 2],
                },
                AuthenticatedRound {
                    round_id: "r2".to_string(),
                    ea_pk: vec![3],
                },
            ],
            skipped_round_ids: vec!["r9".to_string()],
            conditions: vec![ConfigCondition {
                kind: ConfigConditionKind::VersionsSupported,
                status: true,
                message: "m".to_string(),
            }],
        }
    }

    #[test]
    fn voting_config_from_resolved_maps_all_fields() {
        let layout = PirLayout {
            pir_depth: 4,
            tier0_layers: 2,
            tier1_layers: 3,
            poly_len: 2048,
        };
        let config = VotingConfig::from_resolved(
            "https://src".to_string(),
            &resolved_config(layout),
            ConfigSwitchKind::NewChainOrRound,
        );
        assert_eq!(config.source, "https://src");
        assert_eq!(config.source_fingerprint, "sf");
        assert_eq!(config.trusted_key_fingerprint, "tf");
        assert_eq!(config.switch_kind, "new_chain_or_round");
        assert_eq!(
            config.vote_servers,
            vec![
                VotingServiceEndpoint {
                    url: "https://v1".to_string(),
                    label: "vote".to_string(),
                },
                VotingServiceEndpoint {
                    url: "https://v2".to_string(),
                    label: "vote2".to_string(),
                },
            ]
        );
        assert_eq!(
            config.pir_servers,
            vec![VotingServiceEndpoint {
                url: "https://p".to_string(),
                label: "pir".to_string(),
            }]
        );
        assert_eq!(
            config.pir_layout,
            Some(VotingPirLayout {
                pir_depth: 4,
                tier0_layers: 2,
                tier1_layers: 3,
                poly_len: 2048,
            })
        );
        assert_eq!(
            config.rounds,
            vec![
                VotingConfigRound {
                    round_id: "r1".to_string(),
                    ea_pk: vec![1, 2],
                },
                VotingConfigRound {
                    round_id: "r2".to_string(),
                    ea_pk: vec![3],
                },
            ]
        );
    }

    #[test]
    fn voting_config_from_resolved_maps_unknown_pir_layout_to_none() {
        let config = VotingConfig::from_resolved(
            "https://src".to_string(),
            &resolved_config(PirLayout::UNKNOWN),
            ConfigSwitchKind::InitialLoad,
        );
        assert_eq!(config.pir_layout, None);
        assert_eq!(config.switch_kind, "initial_load");
    }

    #[test]
    fn share_delivery_parses_dart_wire_json() {
        let json = r#"{"share_index":1,"sent_to_urls":["https://h1","https://h2"],"submit_at":42,"confirmed":true}"#;
        let parsed: VotingShareDelivery = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            VotingShareDelivery {
                share_index: 1,
                sent_to_urls: vec!["https://h1".to_string(), "https://h2".to_string()],
                submit_at: 42,
                confirmed: true,
            }
        );
    }

    #[test]
    fn progress_and_stage_enums_serde_round_trip() {
        assert_round_trips(VotingDelegationProgress::SelectingNotes);
        assert_round_trips(VotingDelegationProgress::ProofProgress { progress: 0.25 });
        assert_round_trips(VotingVoteCommitStage::Signing {
            proposal_id: 1,
            bundle_index: 2,
        });
        assert_round_trips(VotingVoteCommitStage::ProofProgress {
            proposal_id: 3,
            bundle_index: 4,
            progress: 0.5,
        });
    }

    #[test]
    fn leaf_flow_structs_serde_round_trip() {
        assert_round_trips(VotingDelegationSetup {
            pczt_bytes: vec![1],
            pczt_sighash: vec![2u8; 32],
            rk: vec![3u8; 32],
            action_index: 4,
            action_bytes: vec![5],
            tx1_effects: vec![6],
        });
        assert_round_trips(VotingDelegationSubmission {
            proof: vec![1],
            rk: vec![2u8; 32],
            nf_signed: vec![3u8; 32],
            cmx_new: vec![4u8; 32],
            gov_comm: vec![5u8; 32],
            gov_nullifiers: vec![vec![6u8; 32], vec![7u8; 32]],
            alpha: vec![8u8; 32],
            vote_round_id: "r".to_string(),
            spend_auth_sig: vec![9u8; 64],
            sighash: vec![10u8; 32],
            tx1_effects: vec![11],
        });
        assert_round_trips(VotingDelegationConfirmation {
            tx_hash: "0x1".to_string(),
            van_leaf_position: 1,
        });
        assert_round_trips(VotingVoteConfirmation {
            tx_hash: "0x2".to_string(),
            van_leaf_position: 2,
            vc_tree_position: 3,
        });
        assert_round_trips(VotingVanWitness {
            auth_path: vec![vec![1], vec![2, 3]],
            position: 4,
            anchor_height: 5,
        });
        assert_round_trips(VotingVoteSubmission {
            vote_round_id: "r".to_string(),
            proposal_id: 1,
            van_nullifier: vec![2u8; 32],
            vote_authority_note_new: vec![3u8; 32],
            vote_commitment: vec![4u8; 32],
            proof: vec![5],
            r_vpk: vec![6u8; 32],
            vote_auth_sig: vec![7u8; 64],
            anchor_height: 8,
        });
        assert_round_trips(VotingEncryptedShare {
            c1: vec![1],
            c2: vec![2],
            share_index: 3,
        });
    }

    #[test]
    fn nested_and_config_structs_serde_round_trip() {
        assert_round_trips(VotingSharePayload {
            shares_hash: vec![1],
            proposal_id: 2,
            vote_decision: 3,
            enc_share: VotingEncryptedShare {
                c1: vec![4],
                c2: vec![5],
                share_index: 6,
            },
            tree_position: 7,
            all_enc_shares: vec![VotingEncryptedShare {
                c1: vec![8],
                c2: vec![9],
                share_index: 10,
            }],
            share_comms: vec![vec![11]],
            primary_blind: vec![12],
        });
        assert_round_trips(VotingShareDelivery {
            share_index: 1,
            sent_to_urls: vec!["https://h1".to_string()],
            submit_at: 42,
            confirmed: false,
        });
        assert_round_trips(VotingRoundInfo {
            round_id: "r".to_string(),
            network: "regtest".to_string(),
            snapshot_height: 100,
            hotkey_address: None,
            eligible_weight_zatoshi: Some(50_000),
            bundle_count: 2,
            created_at: 123,
        });
        assert_round_trips(VotingBallotIntent {
            proposal_id: 1,
            skipped: false,
            choice: Some(2),
        });
        assert_round_trips(VotingShareTrackingSummary {
            total: 10,
            confirmed: 6,
            waiting: 2,
            ready: 1,
            overdue: 1,
        });
        assert_round_trips(VotingConfig {
            source: "https://src".to_string(),
            source_fingerprint: "sf".to_string(),
            trusted_key_fingerprint: "tf".to_string(),
            switch_kind: "initial_load".to_string(),
            vote_servers: vec![VotingServiceEndpoint {
                url: "https://v".to_string(),
                label: "vote".to_string(),
            }],
            pir_servers: vec![],
            pir_layout: Some(VotingPirLayout {
                pir_depth: 4,
                tier0_layers: 2,
                tier1_layers: 3,
                poly_len: 2048,
            }),
            rounds: vec![VotingConfigRound {
                round_id: "r1".to_string(),
                ea_pk: vec![1, 2],
            }],
        });
        assert_round_trips(VotingDelegationBuild {
            submission: VotingDelegationSubmission {
                proof: vec![1],
                rk: vec![2u8; 32],
                nf_signed: vec![3u8; 32],
                cmx_new: vec![4u8; 32],
                gov_comm: vec![5u8; 32],
                gov_nullifiers: vec![],
                alpha: vec![6u8; 32],
                vote_round_id: "r".to_string(),
                spend_auth_sig: vec![7u8; 64],
                sighash: vec![8u8; 32],
                tx1_effects: vec![9],
            },
            wire_json: "{}".to_string(),
        });
        assert_round_trips(VotingVotePayloads {
            submission: VotingVoteSubmission {
                vote_round_id: "r".to_string(),
                proposal_id: 1,
                van_nullifier: vec![2u8; 32],
                vote_authority_note_new: vec![3u8; 32],
                vote_commitment: vec![4u8; 32],
                proof: vec![5],
                r_vpk: vec![6u8; 32],
                vote_auth_sig: vec![7u8; 64],
                anchor_height: 8,
            },
            share_payloads: vec![],
        });
        assert_round_trips(VotingRoundPlan {
            round_id: "r".to_string(),
            pending_recovery: false,
            next_steps: vec![],
            open_proposals: vec![],
            all_decided: true,
            delegation_statuses: vec![],
            blocking_recovery: false,
            blocking_share_work: false,
            hotkey_bound: false,
            completed_vote_artifact: false,
            completed_for_display: false,
            completed_vote_display: None,
            needs_draft_setup: false,
            primary_action: "done".to_string(),
        });
        assert_round_trips(VotingRoundRecovery {
            round_id: "r".to_string(),
            bundle_count: 0,
            delegation: vec![],
            votes: vec![],
            shares: vec![],
            share_delegations: vec![],
            unconfirmed_share_delegations: vec![],
        });
        assert_round_trips(VotingSharePlan {
            summary: VotingShareTrackingSummary {
                total: 1,
                confirmed: 0,
                waiting: 1,
                ready: 0,
                overdue: 0,
            },
            next_tracking_delay_secs: Some(30),
            last_moment: false,
            submissions: vec![VotingSharePlanItem {
                submit_at: 100,
                target_count: 1,
                target_servers: vec!["https://h".to_string()],
            }],
        });
    }
}
