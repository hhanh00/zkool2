//! FRB wrappers for the shielded voting flow (ZIP 262 delegation + cast votes).
//!
//! The fork's `zcash_voting` types are not FRB-visible, so this module defines
//! JSON-serializable mirror structs and converts at the boundary. State
//! transitions follow the plan: prepare → setup → sign/prove/submit → confirm,
//! then van witness → commit → payloads → record execution → confirm.

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
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;

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

    let progress = DelegationProgressBridge::new(move |p| {
        let _ = sink.add(p.into());
    });
    let (submission, wire_json) = voting::prove_and_submit_delegation_with_progress(
        c.get_pool()?,
        &wallet_id,
        &prepared,
        &seed,
        pczt_bytes,
        pir_layout.to_fork(),
        &pir_server_url,
        &progress,
    )
    .await?;
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
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
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
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
    Ok(db.get_delegation_tx_hash(&round_id, bundle_index).await?)
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
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
    db.set_ballot_intent(&round_id, proposal_id, decision, num_options)
        .await?;
    Ok(())
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

    let stages = VoteCommitStageBridge::new(move |s| {
        let _ = sink.add(s.into());
    });
    let commitments = voting::commit_votes_with_progress(
        c.get_pool()?,
        &wallet_id,
        &round_id,
        bundle_index,
        &drafts,
        &witness,
        &hotkey,
        &stages,
    )
    .await?;
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
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
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
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
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
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
    Ok(zcash_voting::share::unconfirmed(&db, &round_id)
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
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
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
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
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
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
    Ok(zcash_voting::precompute::sync_vote_tree(&db, &round_id, &vote_node_url).await?)
}

/// Computes the share tracking plan for a round: summary counts, next poll
/// delay, last-moment flag, and freshly planned submissions (with local
/// entropy) for the unconfirmed shares.
#[cfg_attr(feature = "flutter", frb)]
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
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
    let shares = zcash_voting::share::unconfirmed(&db, &round_id).await?;

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
        _ => "unchanged".to_string(),
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
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
    let rounds = db.rounds().await?;
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
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
    let plan = zcash_voting::session::resume_plan(&db, &round_id, &proposal_ids).await?;
    Ok(plan.into())
}

/// Returns the full read-only recovery snapshot for a round.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_recovery(round_id: &str, c: &Coin) -> Result<VotingRoundRecovery> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
    let snapshot = zcash_voting::recovery::round_snapshot(&db, &round_id).await?;
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
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
    zcash_voting::recovery::clear(&db, &round_id).await?;
    Ok(())
}

/// Returns the persisted ballot intents for a round, sorted by proposal id.
#[cfg_attr(feature = "flutter", frb)]
pub async fn voting_ballot_intents(round_id: &str, c: &Coin) -> Result<Vec<VotingBallotIntent>> {
    let account = c.account;
    let round_id = round_id.to_string();
    let mut connection = c.get_connection().await?;
    let wallet_id = voting::voting_wallet_id(&mut connection, account).await?;
    let db = voting::open_voting_db(c.get_pool()?, &wallet_id).await?;
    let intents = db.ballot_intents(&round_id).await?;
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
