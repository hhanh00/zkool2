//! FRB wrappers for the shielded voting flow (ZIP 262 delegation + cast votes).
//!
//! The fork's `zcash_voting` types are not FRB-visible, so this module defines
//! JSON-serializable mirror structs and converts at the boundary. State
//! transitions follow the plan: prepare → setup → sign/prove/submit → confirm,
//! then van witness → commit → payloads → record execution → confirm.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use zcash_voting::prelude::{BundlePolicy, NoopProgressReporter, TxEvent};
use zcash_voting::VotingRoundParams;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

use crate::{api::coin::Coin, voting};

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
    fn to_fork(self) -> zcash_voting::config::PirLayout {
        zcash_voting::config::PirLayout {
            pir_depth: self.pir_depth,
            tier0_layers: self.tier0_layers,
            tier1_layers: self.tier1_layers,
            poly_len: self.poly_len,
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
/// witnesses are rooted at the snapshot's Ironwood `nc_root`.
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

    let submission = voting::prove_and_submit_delegation(
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
