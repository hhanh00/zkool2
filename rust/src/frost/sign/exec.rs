//! Task execution: the side effects, and nothing else.
//!
//! Every decision has already been made by the time control reaches here.
//! Each function carries out exactly one named effect and returns either
//! `Ok(SendResult::Sent)` or `Ok(SendResult::WaitingForFunds)` when the
//! effect involves a Zcash publish.

use std::collections::BTreeMap;

use anyhow::{Context as _, Result};
use bincode::config;
use ed25519_dalek::SigningKey;
use ff::PrimeField as _;
use ff_legacy::PrimeField as _;
use frost_rerandomized::{aggregate, sign, RandomizedParams};
use halo2_proofs::pasta::Fq;
use pczt::{
    roles::{
        low_level_signer::Signer as LowLevelSigner, prover::Prover,
        spend_finalizer::SpendFinalizer, tx_extractor::TransactionExtractor,
    },
    Pczt,
};
use rand_core_legacy::OsRng;
use reddsa_legacy::frost::redpallas::{
    frost::{
        keys::{KeyPackage, PublicKeyPackage, VerifyingShare},
        round1::{SigningCommitments, SigningNonces},
        round2::SignatureShare,
        SigningPackage,
    },
    round1::commit,
    Identifier, Randomizer,
};
use sqlx::{sqlite::SqliteRow, Connection as _, Row, SqliteConnection};
use tracing::info;

use crate::{
    account::get_orchard_vk,
    api::coin::Network,
    db::put_prop,
    frost::{
        dkg::delete_frost_state,
        protocol::{
            ensure_broadcast_account_from_sk, ensure_mailbox_from_seed, group_sk,
            publish_expired, resend_publish, send_staged, session_broadcast_sk,
            stage_publish, SIGN_LAST_TXID_PROP,
        },
        FrostSigMessage, P,
    },
    pay::{plan::get_orchard_pk, plan::get_sapling_prover},
    Client,
};

use super::pubkey::rebuild_public_key_package;
use super::state::{
    load_pczt_pkg, SigPackagePart, SignState, COORDINATOR_ADDRESS_PROP, VERIFYING_SHARE_ROUND,
};
use super::{COMMITMENT_PREFIX, COORDINATOR_PREFIX, SIGPACKAGE_PREFIX, SIGSHARE_PREFIX};

/// Result of an exec function that involves a Zcash publish.
pub enum SendResult {
    Sent,
    WaitingForFunds,
}

// ── EnsureMailbox ─────────────────────────────────────────────────────────────

/// Create the private mailbox account if it does not exist yet, and the
/// session's broadcast account. Neither needs the participant addresses:
/// the mailbox comes from its recorded seed, the broadcast account from
/// the group's `sk` and the transaction being signed. Both are deleted
/// with the session, and the lookback covers a member that joins after
/// the coordinator by far more than the transaction's expiry allows.
pub async fn ensure_mailbox(
    network: &Network,
    connection: &mut SqliteConnection,
    state: &SignState,
    height: u32,
) -> Result<()> {
    let birth_height = height.saturating_sub(10000) + 1;
    ensure_mailbox_from_seed(network, connection, state.account, birth_height).await?;
    let sk = group_sk(network, connection, state.account).await?;
    let session_sk = session_broadcast_sk(network, &sk, &state.sighash)?;
    ensure_broadcast_account_from_sk(network, connection, &session_sk, birth_height).await?;
    Ok(())
}

// ── AnnounceCoordinator ───────────────────────────────────────────────────────

/// Coordinator: broadcast our mailbox address for this session, as the
/// session's first message. Members send their commitments to this
/// address, so it has to be known before anything else, which is why it
/// is its own memo rather than a field of a later message. Recorded as
/// our own coordinator address in the same transaction, which is what
/// makes the task done.
pub async fn announce_coordinator(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    state: &SignState,
) -> Result<SendResult> {
    let broadcast_address = state
        .broadcast_address
        .as_deref()
        .context("broadcast address not set")?;
    let address = state
        .self_mailbox_address
        .as_deref()
        .context("our mailbox address not set")?;

    let mut tx = connection.begin().await?;
    let signing_key = load_signing_key(&mut *tx, state.account).await?;
    let mut msg = FrostSigMessage {
        sighash: state.sighash.clone().try_into().unwrap(),
        from_id: state.coordinator as u16,
        idx: 0,
        data: address.as_bytes().to_vec(),
        signature: None,
        verifying_share: None,
    };
    if let Some(ref key) = signing_key {
        msg.sign(key)?;
    }
    put_prop(&mut *tx, COORDINATOR_ADDRESS_PROP, address).await?;
    let recipients = [(broadcast_address, msg.encode_with_prefix(COORDINATOR_PREFIX)?)];
    let staged = match stage_publish(network, &mut tx, state.funding_account, client, &recipients, SIGN_LAST_TXID_PROP).await {
        Ok(staged) => staged,
        Err(e) if crate::pay::plan::is_no_feasible_selection(&e) => {
            return Ok(SendResult::WaitingForFunds);
        }
        Err(e) => return Err(e),
    };
    tx.commit().await?;
    send_staged(client, &staged).await?;
    Ok(SendResult::Sent)
}

// ── SendCommitments (Phase 1) ─────────────────────────────────────────────────

pub async fn send_commitments(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    state: &SignState,
) -> Result<SendResult> {
    let coordinator_address = state
        .coordinator_address
        .as_deref()
        .context("coordinator address not set")?;

    let mut tx = connection.begin().await?;
    let signing_key = load_signing_key(&mut *tx, state.account).await?;
    let key_package = load_key_package(&mut *tx, state.account).await?;
    // Our verifying share rides along, so a coordinator without a stored
    // public key package can rebuild one from the signing set.
    let verifying_share = key_package.verifying_share().serialize()?;
    let mut recipients = vec![];

    for idx in 0..state.nsigs {
        let (nonces, commitments) = commit(key_package.signing_share(), &mut OsRng);
        let nonces = nonces.serialize()?;
        sqlx::query(
            "INSERT INTO frost_signatures(account, sighash, idx, nonce) VALUES (?, ?, ?, ?)",
        )
        .bind(state.account)
        .bind(&state.sighash)
        .bind(idx)
        .bind(&nonces)
        .execute(&mut *tx)
        .await?;

        let commitments = commitments.serialize()?;
        sqlx::query(
            "INSERT INTO frost_commitments
             (account, sighash, idx, from_id, commitment) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(state.account)
        .bind(&state.sighash)
        .bind(idx)
        .bind(state.self_id_u8)
        .bind(&commitments)
        .execute(&mut *tx)
        .await?;

        let mut msg = FrostSigMessage {
            sighash: state.sighash.clone().try_into().unwrap(),
            from_id: state.self_id_u8 as u16,
            idx,
            data: commitments,
            signature: None,
            verifying_share: Some(verifying_share.clone()),
        };
        if let Some(ref key) = signing_key {
            msg.sign(key)?;
        }
        recipients.push((
            coordinator_address.to_string(),
            msg.encode_with_prefix(COMMITMENT_PREFIX)?,
        ));
    }

    // Coordinator does not send to itself — its rows are already in the DB.
    let mut staged = None;
    if !state.is_coordinator() {
        let refs: Vec<(&str, Vec<u8>)> =
            recipients.iter().map(|(a, d)| (a.as_str(), d.clone())).collect();
        staged = match stage_publish(network, &mut tx, state.funding_account, client, &refs, SIGN_LAST_TXID_PROP).await {
            Ok(staged) => Some(staged),
            Err(e) if crate::pay::plan::is_no_feasible_selection(&e) => {
                return Ok(SendResult::WaitingForFunds);
            }
            Err(e) => return Err(e),
        };
    }
    tx.commit().await?;
    if let Some(staged) = staged {
        send_staged(client, &staged).await?;
    }
    Ok(SendResult::Sent)
}

// ── MakeSigpackages (Phase 2, coordinator) ────────────────────────────────────

pub async fn make_sigpackages(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    state: &SignState,
) -> Result<SendResult> {
    let broadcast_address = state
        .broadcast_address
        .as_deref()
        .context("broadcast address not set")?;

    let pczt_pkg = load_pczt_pkg(&mut *connection).await?;
    let pczt = Pczt::parse(&pczt_pkg.pczt).map_err(|e| anyhow::anyhow!("invalid PCZT: {e:?}"))?;
    check_indices(&pczt_pkg)?;

    let commitments_vec = load_commitments(&mut *connection, state.account, &state.sighash, state.nsigs).await?;

    let mut tx = connection.begin().await?;
    let signing_key = load_signing_key(&mut *tx, state.account).await?;
    let mut recipients = vec![];

    for (idx, commitment_map) in commitments_vec.iter().enumerate() {
        let sigpackage = SigningPackage::new(commitment_map.clone(), &state.sighash);

        // Extract randomizer (alpha) from the PCZT for this input. A package
        // whose index names no action ends the pass with an error, not a
        // panic that stops the driver (#1482).
        let signer = LowLevelSigner::new(pczt.clone());
        let mut alpha: Option<Fq> = None;
        let action_index = if idx < pczt_pkg.orchard_indices.len() {
            let action_index = pczt_pkg.orchard_indices[idx];
            signer
                .sign_orchard_with(|_, bundle, _| {
                    alpha = bundle
                        .actions()
                        .get(action_index)
                        .and_then(|action| *action.spend().alpha());
                    Ok::<_, pczt::roles::low_level_signer::OrchardParseError>(())
                })
                .map_err(|e| anyhow::anyhow!("invalid Orchard bundle: {e:?}"))?;
            action_index
        } else {
            let action_index = pczt_pkg.ironwood_indices[idx - pczt_pkg.orchard_indices.len()];
            signer
                .sign_ironwood_with(|_, bundle, _| {
                    alpha = bundle
                        .actions()
                        .get(action_index)
                        .and_then(|action| *action.spend().alpha());
                    Ok::<_, pczt::roles::low_level_signer::OrchardParseError>(())
                })
                .map_err(|e| anyhow::anyhow!("invalid Ironwood bundle: {e:?}"))?;
            action_index
        };
        let alpha = alpha.with_context(|| {
            format!("input {idx}: action {action_index} has no spend randomizer in the PCZT")
        })?;
        let randomizer = Randomizer::from_scalar(
            pasta_curves_legacy::Fq::from_repr(alpha.to_repr())
                .expect("same canonical Pallas scalar encoding"),
        );

        let sigpackage_bytes = sigpackage.serialize()?;
        sqlx::query(
            "UPDATE frost_signatures SET sigpackage = ?1, randomizer = ?2
             WHERE account = ?3 AND sighash = ?4 AND idx = ?5",
        )
        .bind(&sigpackage_bytes)
        .bind(randomizer.serialize())
        .bind(state.account)
        .bind(&state.sighash)
        .bind(idx as u32)
        .execute(&mut *tx)
        .await?;

        // One memo per signer: the whole package stops fitting a memo past
        // three signers, a part never grows. Members rebuild the package
        // from the parts (`assemble_sigpackages`).
        for (id, commitments) in commitment_map {
            let part = SigPackagePart {
                signer_id: participant_id(id),
                commitment: commitments.serialize()?,
                randomizer: randomizer.serialize(),
            };
            let mut msg = FrostSigMessage {
                sighash: state.sighash.clone().try_into().unwrap(),
                from_id: state.coordinator as u16,
                idx: idx as u32,
                data: bincode::encode_to_vec(&part, config::legacy())?,
                signature: None,
                verifying_share: None,
            };
            if let Some(ref key) = signing_key {
                msg.sign(key)?;
            }
            recipients.push((
                broadcast_address.to_string(),
                msg.encode_with_prefix(SIGPACKAGE_PREFIX)?,
            ));
        }
    }

    let refs: Vec<(&str, Vec<u8>)> =
        recipients.iter().map(|(a, d)| (a.as_str(), d.clone())).collect();
    let staged = match stage_publish(network, &mut tx, state.funding_account, client, &refs, SIGN_LAST_TXID_PROP).await {
        Ok(staged) => staged,
        Err(e) if crate::pay::plan::is_no_feasible_selection(&e) => {
            return Ok(SendResult::WaitingForFunds);
        }
        Err(e) => return Err(e),
    };
    tx.commit().await?;
    send_staged(client, &staged).await?;
    Ok(SendResult::Sent)
}

// ── SendSigshares (Phase 3) ───────────────────────────────────────────────────

pub async fn send_sigshares(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    state: &SignState,
) -> Result<SendResult> {
    let coordinator_address = state
        .coordinator_address
        .as_deref()
        .context("coordinator address not set")?;

    let spkg = load_key_package(&mut *connection, state.account).await?;
    let nonces = load_nonces(&mut *connection, state.account, &state.sighash).await?;
    let sigpackages = load_sigpackages(&mut *connection, state.account, &state.sighash).await?;

    let mut tx = connection.begin().await?;
    let signing_key = load_signing_key(&mut *tx, state.account).await?;
    let mut recipients = vec![];

    for (idx, ((sigpackage, randomizer), nonce)) in
        sigpackages.iter().zip(nonces.iter()).enumerate()
    {
        let share =
            sign(sigpackage, nonce, &spkg, *randomizer).context("Failed to produce sigshare")?;
        let share_bytes = share.serialize();

        sqlx::query(
            "UPDATE frost_signatures SET sigshare = ?1
             WHERE account = ?2 AND sighash = ?3 AND idx = ?4",
        )
        .bind(&share_bytes)
        .bind(state.account)
        .bind(&state.sighash)
        .bind(idx as u32)
        .execute(&mut *tx)
        .await?;

        let mut msg = FrostSigMessage {
            sighash: state.sighash.clone().try_into().unwrap(),
            from_id: state.self_id_u8 as u16,
            idx: idx as u32,
            data: share_bytes,
            signature: None,
            verifying_share: None,
        };
        if let Some(ref key) = signing_key {
            msg.sign(key)?;
        }
        recipients.push((coordinator_address.to_string(), msg.encode_with_prefix(SIGSHARE_PREFIX)?));
    }

    let mut staged = None;
    if !state.is_coordinator() {
        let refs: Vec<(&str, Vec<u8>)> =
            recipients.iter().map(|(a, d)| (a.as_str(), d.clone())).collect();
        staged = match stage_publish(network, &mut tx, state.funding_account, client, &refs, SIGN_LAST_TXID_PROP).await {
            Ok(staged) => Some(staged),
            Err(e) if crate::pay::plan::is_no_feasible_selection(&e) => {
                return Ok(SendResult::WaitingForFunds);
            }
            Err(e) => return Err(e),
        };
    }
    tx.commit().await?;
    if let Some(staged) = staged {
        send_staged(client, &staged).await?;
    }
    Ok(SendResult::Sent)
}

// ── WaitOwnPublish ────────────────────────────────────────────────────────────

/// What a pass found when it met its own unmined message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnPublish {
    /// Past the message's expiry: the marker is cleared and its notes
    /// unlocked, so the planner sends the message afresh.
    Expired,
    /// Resent; the pass waits for the block that mines it.
    Waiting,
}

/// The wait on our own message is not idle (#1565): the staged transaction
/// is resent, which repairs a broadcast lost to a crash or a dropped
/// mempool entry and is a no-op for a node that has it; and once the tip
/// passes its expiry the message is given up so the next pass rebuilds it.
pub async fn own_publish(
    connection: &mut SqliteConnection,
    client: &mut Client,
    state: &SignState,
    height: u32,
) -> Result<OwnPublish> {
    if publish_expired(connection, SIGN_LAST_TXID_PROP, state.funding_account, height).await? {
        return Ok(OwnPublish::Expired);
    }
    resend_publish(connection, client, SIGN_LAST_TXID_PROP).await?;
    Ok(OwnPublish::Waiting)
}

// ── Aggregate (Phase 4, coordinator) ─────────────────────────────────────────

/// Aggregate signature shares, prove, finalize, and broadcast the transaction.
/// Returns the transaction id.
pub async fn aggregate_and_send(
    _network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    state: &SignState,
) -> Result<String> {
    let sigpackages = load_sigpackages(&mut *connection, state.account, &state.sighash).await?;
    let sigsharess =
        load_all_sigshares(&mut *connection, state.account, &state.sighash, state.nsigs).await?;
    let signing_set: Vec<Identifier> = sigsharess
        .first()
        .map(|m| m.keys().copied().collect())
        .unwrap_or_default();
    let ppkg = match rebuild_pubkey_package(&mut *connection, state.account, &signing_set).await? {
        Some(ppkg) => ppkg,
        None => load_pubkey_package(&mut *connection, state.account)
            .await?
            .context("no public key package: a participant's verifying share is missing")?,
    };

    let pczt_pkg = load_pczt_pkg(&mut *connection).await?;
    let pczt = Pczt::parse(&pczt_pkg.pczt).map_err(|e| anyhow::anyhow!("invalid PCZT: {e:?}"))?;
    check_indices(&pczt_pkg)?;

    // Aggregate per input.
    let mut tx = connection.begin().await?;
    let mut signatures = vec![];
    for (idx, (sigshares, (sigpackage, randomizer))) in
        sigsharess.iter().zip(sigpackages.iter()).enumerate()
    {
        let randomized_params = RandomizedParams::from_randomizer(ppkg.verifying_key(), *randomizer);
        let signature = aggregate(sigpackage, sigshares, &ppkg, &randomized_params)?;
        let sig_bytes = signature.serialize()?;
        let sig_arr: [u8; 64] = sig_bytes
            .clone()
            .try_into()
            .map_err(|_| anyhow::anyhow!("input {idx}: the aggregated signature is not 64 bytes"))?;
        signatures.push(orchard::primitives::redpallas::Signature::from(sig_arr));
        sqlx::query(
            "UPDATE frost_signatures SET signature = ?1
             WHERE account = ?2 AND sighash = ?3 AND idx = ?4",
        )
        .bind(&sig_bytes)
        .bind(state.account)
        .bind(&state.sighash)
        .bind(idx as u32)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    // Apply signatures to PCZT.
    let orchard_count = pczt_pkg.orchard_indices.len();
    let (orchard_sigs, ironwood_sigs) = signatures.split_at(orchard_count);
    let mut signer = pczt::roles::signer::Signer::new(pczt)
        .map_err(|e| anyhow::anyhow!("invalid PCZT: {e:?}"))?;
    for (idx, sig) in orchard_sigs.iter().enumerate() {
        let action_index = pczt_pkg.orchard_indices[idx];
        signer
            .apply_orchard_signature(action_index, sig.clone())
            .map_err(|e| anyhow::anyhow!("Orchard action {action_index}: {e:?}"))?;
    }
    for (idx, sig) in ironwood_sigs.iter().enumerate() {
        let action_index = pczt_pkg.ironwood_indices[idx];
        signer
            .apply_ironwood_signature(action_index, sig.clone())
            .map_err(|e| anyhow::anyhow!("Ironwood action {action_index}: {e:?}"))?;
    }
    let pczt = signer.finish();

    // Prove, finalize, extract.
    let sapling_prover = get_sapling_prover().await?;
    let orchard_pk = get_orchard_pk(*pczt.global().consensus_branch_id())?;
    // A PCZT the session cannot finalize ends the pass with an error the
    // UI shows, not a panic that stops the driver (#1548).
    let pczt = Prover::new(pczt)
        .create_sapling_proofs(rand_core::UnwrapErr(rand::rngs::SysRng), sapling_prover, sapling_prover)
        .map_err(|e| anyhow::anyhow!("Sapling proofs: {e:?}"))?
        .create_orchard_proof(rand_core::UnwrapErr(rand::rngs::SysRng), orchard_pk)
        .map_err(|e| anyhow::anyhow!("Orchard proof: {e:?}"))?
        .create_ironwood_proof(rand_core::UnwrapErr(rand::rngs::SysRng), orchard_pk)
        .map_err(|e| anyhow::anyhow!("Ironwood proof: {e:?}"))?
        .finish();
    let pczt = SpendFinalizer::new(pczt)
        .finalize_spends()
        .map_err(|e| anyhow::anyhow!("finalizing spends: {e:?}"))?;
    let sapling_prover = get_sapling_prover().await?;
    let (svk, ovk) = sapling_prover.verifying_keys();
    let tx = TransactionExtractor::new(pczt)
        .with_sapling(&svk, &ovk)
        .extract(rand_core::UnwrapErr(rand::rngs::SysRng))
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    let txid = tx.txid();
    let mut tx_bytes = vec![];
    tx.write(&mut tx_bytes).unwrap();

    // The session stays until the transaction is out: a broadcast that
    // fails (the server unreachable, say) is retried by the next pass with
    // the same signatures, instead of ending the session with nothing sent.
    // A retry after a broadcast that did go through, but whose answer was
    // lost, would be refused as a duplicate; the scan has recorded the
    // transaction by then, so finish without sending again.
    if transaction_recorded(connection, state.account, txid.as_ref()).await? {
        info!("Transaction already recorded: {txid}");
        delete_frost_state(&mut *connection).await?;
        return Ok(txid.to_string());
    }
    let txid =
        crate::pay::broadcast_under_session(connection, client, state.account, &tx_bytes, None)
            .await?;
    info!("Transaction sent: {txid}");
    delete_frost_state(&mut *connection).await?;
    Ok(txid)
}

/// Whether the account's scan has recorded [txid], in either byte order.
async fn transaction_recorded(
    connection: &mut SqliteConnection,
    account: u32,
    txid: &[u8],
) -> Result<bool> {
    let reversed: Vec<u8> = txid.iter().rev().copied().collect();
    let (known,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM transactions WHERE account = ? AND (txid = ? OR txid = ?))",
    )
    .bind(account)
    .bind(txid)
    .bind(&reversed)
    .fetch_one(&mut *connection)
    .await?;
    Ok(known)
}

/// The package names each input's action once. A duplicate would sign one
/// action twice and leave another unsigned, failing later and less clearly
/// (#1482).
fn check_indices(pczt_pkg: &crate::api::pay::PcztPackage) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    for (pool, indices) in [
        ("Orchard", &pczt_pkg.orchard_indices),
        ("Ironwood", &pczt_pkg.ironwood_indices),
    ] {
        for index in indices {
            if !seen.insert((pool, *index)) {
                anyhow::bail!("the signing package names {pool} action {index} twice");
            }
        }
    }
    Ok(())
}

// ── DB helpers ────────────────────────────────────────────────────────────────

async fn load_signing_key(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<SigningKey>> {
    let result = sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT signing_keypair FROM dkg_state
         WHERE account = ? AND signing_keypair IS NOT NULL",
    )
    .bind(account)
    .fetch_optional(&mut *connection)
    .await?;
    match result {
        Some((b,)) => {
            // A row of the wrong length is an error naming the account, as
            // the DKG loader reports it, not a panic (#1495).
            let arr: [u8; ed25519_dalek::SECRET_KEY_LENGTH] = b.try_into().map_err(|b: Vec<u8>| {
                anyhow::anyhow!(
                    "account {account}: invalid signing key length in the DKG state, expected {} bytes, got {}",
                    ed25519_dalek::SECRET_KEY_LENGTH,
                    b.len()
                )
            })?;
            Ok(Some(SigningKey::from_bytes(&arr)))
        }
        None => Ok(None),
    }
}

async fn load_key_package(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<KeyPackage<P>> {
    let (data,): (Vec<u8>,) =
        sqlx::query_as("SELECT key_pkg FROM dkg_state WHERE account = ?")
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;
    Ok(KeyPackage::<P>::deserialize(&data)?)
}

/// The public key package the DKG stored, if this device ran the DKG.
async fn load_pubkey_package(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<PublicKeyPackage<P>>> {
    let row: Option<(Vec<u8>,)> = sqlx::query_as(
        "SELECT data FROM dkg_peers WHERE account = ? AND round = 3 LIMIT 1",
    )
    .bind(account)
    .fetch_optional(&mut *connection)
    .await?;
    row.map(|(data,)| Ok(PublicKeyPackage::<P>::deserialize(&data)?))
        .transpose()
}

/// The public key package rebuilt from the signing set's verifying shares:
/// ours from the key package, the others' from the rows their commitments
/// messages filled. None when a share is missing (a participant on a build
/// that does not send it); an error when the shares do not interpolate to
/// the group key.
async fn rebuild_pubkey_package(
    connection: &mut SqliteConnection,
    account: u32,
    signing_set: &[Identifier],
) -> Result<Option<PublicKeyPackage<P>>> {
    let key_package = load_key_package(&mut *connection, account).await?;
    let fvk = get_orchard_vk(&mut *connection, account)
        .await?
        .context("frost account has no Orchard key")?;
    let ak: [u8; 32] = fvk.to_bytes()[..32].try_into().unwrap();

    let mut shares = BTreeMap::new();
    for id in signing_set {
        if id == key_package.identifier() {
            shares.insert(*id, *key_package.verifying_share());
            continue;
        }
        let from_id = participant_id(id) as u8;
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT data FROM dkg_peers WHERE account = ? AND round = ? AND from_id = ?",
        )
        .bind(account)
        .bind(VERIFYING_SHARE_ROUND)
        .bind(from_id)
        .fetch_optional(&mut *connection)
        .await?;
        let Some((data,)) = row else {
            return Ok(None);
        };
        shares.insert(*id, VerifyingShare::<P>::deserialize(&data)?);
    }
    rebuild_public_key_package(shares, &ak).map(Some)
}

async fn load_nonces(
    connection: &mut SqliteConnection,
    account: u32,
    sighash: &[u8],
) -> Result<Vec<SigningNonces<P>>> {
    let rows = sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT nonce FROM frost_signatures WHERE account = ? AND sighash = ? ORDER BY idx",
    )
    .bind(account)
    .bind(sighash)
    .fetch_all(&mut *connection)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(n,)| SigningNonces::<P>::deserialize(&n).expect("deserialize nonce"))
        .collect())
}

async fn load_sigpackages(
    connection: &mut SqliteConnection,
    account: u32,
    sighash: &[u8],
) -> Result<Vec<(SigningPackage<P>, Randomizer)>> {
    sqlx::query(
        "SELECT sigpackage, randomizer FROM frost_signatures
         WHERE account = ? AND sighash = ? AND sigpackage IS NOT NULL ORDER BY idx",
    )
    .bind(account)
    .bind(sighash)
    .map(|row: SqliteRow| {
        let sp: Vec<u8> = row.get(0);
        let rnd: Vec<u8> = row.get(1);
        (
            SigningPackage::<P>::deserialize(&sp).unwrap(),
            Randomizer::deserialize(&rnd).unwrap(),
        )
    })
    .fetch_all(&mut *connection)
    .await
    .map_err(Into::into)
}

/// The participant id (`from_id`) an identifier was made from: the inverse
/// of `Identifier::try_from(u16)`, which encodes the id as a little-endian
/// scalar.
pub(super) fn participant_id(id: &Identifier) -> u16 {
    u16::from_le_bytes(id.serialize()[..2].try_into().unwrap())
}

type CommitmentMap = BTreeMap<Identifier, SigningCommitments<P>>;

/// Every commitment row of each input, as a map by signer identifier.
pub(super) async fn load_commitments(
    connection: &mut SqliteConnection,
    account: u32,
    sighash: &[u8],
    nsigs: u32,
) -> Result<Vec<CommitmentMap>> {
    let mut maps = vec![];
    for idx in 0..nsigs {
        let mut map = BTreeMap::new();
        let rows = sqlx::query(
            "SELECT from_id, commitment FROM frost_commitments
             WHERE account = ? AND sighash = ? AND idx = ?",
        )
        .bind(account)
        .bind(sighash)
        .bind(idx)
        .map(|row: SqliteRow| {
            let from_id: u16 = row.get(0);
            let commitment: Vec<u8> = row.get(1);
            (from_id, commitment)
        })
        .fetch_all(&mut *connection)
        .await?;
        for (from_id, commitment) in rows {
            map.insert(
                from_id.try_into().unwrap(),
                SigningCommitments::<P>::deserialize(&commitment).unwrap(),
            );
        }
        maps.push(map);
    }
    Ok(maps)
}

type SignatureMap = BTreeMap<Identifier, SignatureShare<P>>;

async fn load_all_sigshares(
    connection: &mut SqliteConnection,
    account: u32,
    sighash: &[u8],
    nsigs: u32,
) -> Result<Vec<SignatureMap>> {
    let mut maps = vec![];
    for idx in 0..nsigs {
        let mut map = BTreeMap::new();
        let rows = sqlx::query(
            "SELECT from_id, sigshare FROM frost_commitments
             WHERE account = ? AND sighash = ? AND idx = ? AND sigshare IS NOT NULL",
        )
        .bind(account)
        .bind(sighash)
        .bind(idx)
        .map(|row: SqliteRow| {
            let from_id: u16 = row.get(0);
            let sigshare: Vec<u8> = row.get(1);
            (from_id, sigshare)
        })
        .fetch_all(&mut *connection)
        .await?;
        for (from_id, sigshare) in rows {
            map.insert(
                from_id.try_into().unwrap(),
                SignatureShare::<P>::deserialize(&sigshare).unwrap(),
            );
        }
        maps.push(map);
    }
    Ok(maps)
}

#[cfg(test)]
mod guard_tests {
    use super::*;
    use crate::{api::pay::PcztPackage, db::create_schema};
    use sqlx::Connection;

    fn package(orchard: Vec<usize>, ironwood: Vec<usize>) -> PcztPackage {
        PcztPackage {
            pczt: vec![],
            n_spends: [0; 4],
            sapling_indices: vec![],
            orchard_indices: orchard,
            ironwood_indices: ironwood,
            can_sign: true,
            can_broadcast: true,
            price: None,
            category: None,
            is_issuance: false,
        }
    }

    /// #1482: a duplicate action index is refused.
    #[test]
    fn duplicate_indices_are_an_error() {
        assert!(check_indices(&package(vec![0, 1], vec![0])).is_ok());
        let m = check_indices(&package(vec![0, 0], vec![])).unwrap_err().to_string();
        assert!(m.contains("Orchard action 0 twice"), "{m}");
    }

    /// #1495: a signing key of the wrong length is an error, not a panic.
    #[tokio::test]
    async fn a_short_signing_key_is_an_error() {
        let mut db = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        create_schema(&mut db).await.unwrap();
        sqlx::query("INSERT INTO dkg_state(account, signing_keypair) VALUES (1, ?)")
            .bind(vec![7u8; 31])
            .execute(&mut db)
            .await
            .unwrap();
        let m = load_signing_key(&mut db, 1).await.unwrap_err().to_string();
        assert!(m.contains("account 1") && m.contains("got 31"), "{m}");
    }
}
