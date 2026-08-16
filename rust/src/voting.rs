//! Zcash shielded voting (ZIP 262): delegation and vote casting.
//!
//! This module integrates the patched `zcash_voting` fork with zkool's own
//! wallet database. The fork's `VotingDb` is embedded in zkool's SQLCipher
//! pool (`voting_*` tables) and all note/witness/seed material comes from
//! zkool's own storage — no `zcash_client_sqlite::WalletDb` is involved.
//!
//! The delegation bundle is broadcast to the vote chain (never to the Zcash
//! network); `zcash_voting::confirmation` turns chain events back into voting
//! DB state before votes can be cast.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{anyhow, ensure, Context as _, Result};
use bip39::Mnemonic;
use halo2_proofs::pasta::group::ff::PrimeField as _;
use rand_core::OsRng;
use sqlx::{Row as _, SqliteConnection, SqlitePool};
use zip32::AccountId;

use zcash_keys::keys::{UnifiedFullViewingKey, UnifiedSpendingKey};
use zcash_voting::prelude::{
    confirm_delegation_submission, confirm_vote_submission, generate_random_voting_hotkey,
    BundlePolicy, CommittedVote, DelegationConfirmation, DelegationKeys, DelegationProgress,
    DelegationProgressReporter, DelegationSigningRequest, DelegationSubmission, DraftVote,
    NoopProgressReporter, NoteInfo, PrepareDelegationBundleWithInputsParams,
    PreparedDelegationBundle, PreparedSigner, SharePayload, SignedVoteCommitments, TxEvent,
    VanWitness, VoteCommitStageReporter, VoteConfirmation, VoteSigner, VoteSubmission, VotingDb,
    VotingHotkey, WitnessData,
};
use zcash_voting::{
    minimum_voting_eligibility_for_notes, Network as VotingNetwork, VotingRoundParams,
};

use crate::api::coin::Network as WalletNetwork;
use crate::warp::hasher::{empty_roots, OrchardHasher};

/// Prop key holding the hex-encoded voting hotkey stored secret.
pub const VOTING_HOTKEY_PROP: &str = "voting_hotkey_secret";

/// Maps zkool's wallet network to the voting crate's network selector.
pub fn voting_network(network: &WalletNetwork) -> Result<VotingNetwork> {
    match network {
        WalletNetwork::Main => Ok(VotingNetwork::Mainnet),
        WalletNetwork::Test => Ok(VotingNetwork::Testnet),
        WalletNetwork::Regtest(_) => Ok(VotingNetwork::Regtest),
        WalletNetwork::ZsaRegtest(_) => {
            Err(anyhow!("voting is not supported on the ZSA network"))
        }
    }
}

/// Opens (or reuses) the voting database embedded in the wallet's SQLCipher pool.
///
/// Migrations are additive and idempotent; all voting state is scoped to
/// `wallet_id` (the hex-encoded ZIP-32 seed fingerprint).
pub async fn open_voting_db(pool: SqlitePool, wallet_id: &str) -> Result<VotingDb> {
    let db = VotingDb::from_pool(pool).await?;
    db.set_wallet_id(wallet_id);
    Ok(db)
}

/// Wallet identifier for voting state: hex-encoded ZIP-32 seed fingerprint.
pub async fn voting_wallet_id(connection: &mut SqliteConnection, account: u32) -> Result<String> {
    let fingerprint = crate::db::get_account_fingerprint(connection, account)
        .await?
        .ok_or_else(|| anyhow!("account {account} has no seed fingerprint"))?;
    ensure!(
        fingerprint.len() == 32,
        "seed fingerprint must be 32 bytes, got {}",
        fingerprint.len()
    );
    Ok(hex::encode(fingerprint))
}

/// The ZIP-32 seed bytes for an account, derived from its stored mnemonic.
pub async fn account_seed(connection: &mut SqliteConnection, account: u32) -> Result<Vec<u8>> {
    let seed = crate::account::get_account_seed(connection, account)
        .await?
        .ok_or_else(|| anyhow!("account {account} has no mnemonic seed"))?;
    let mnemonic = Mnemonic::from_str(&seed.mnemonic)?;
    Ok(mnemonic.to_seed(&seed.phrase).to_vec())
}

/// Generates a fresh app-owned voting hotkey and persists its stored secret
/// (hex) in the wallet props table.
pub async fn voting_hotkey_create(
    connection: &mut SqliteConnection,
    network: VotingNetwork,
) -> Result<VotingHotkey> {
    ensure!(
        crate::db::get_prop(connection, VOTING_HOTKEY_PROP)
            .await?
            .is_none(),
        "voting hotkey already exists for this wallet"
    );
    let hotkey = generate_random_voting_hotkey(network)?;
    crate::db::put_prop(
        connection,
        VOTING_HOTKEY_PROP,
        &hex::encode(hotkey.stored_secret()),
    )
    .await?;
    Ok(hotkey)
}

/// Loads the persisted voting hotkey from the wallet props table.
pub async fn voting_hotkey_load(
    connection: &mut SqliteConnection,
    network: VotingNetwork,
) -> Result<VotingHotkey> {
    let secret = crate::db::get_prop(connection, VOTING_HOTKEY_PROP)
        .await?
        .ok_or_else(|| anyhow!("no voting hotkey; create one first"))?;
    let secret = hex::decode(secret)?;
    Ok(VotingHotkey::from_stored_secret(&secret, network)?)
}

/// Persists the round inputs needed to re-run [`prepare_delegation_bundle`]
/// after a restart (the prepared-bundle cache is process-local only).
pub async fn save_round_config(
    connection: &mut SqliteConnection,
    round_id: &str,
    round_params_json: &str,
    round_name: &str,
    max_real_notes_per_bundle: Option<u32>,
    lightwalletd_url: &str,
) -> Result<()> {
    crate::db::put_prop(
        connection,
        &format!("voting_round_params:{round_id}"),
        round_params_json,
    )
    .await?;
    crate::db::put_prop(
        connection,
        &format!("voting_round_name:{round_id}"),
        round_name,
    )
    .await?;
    crate::db::put_prop(
        connection,
        &format!("voting_round_lwd:{round_id}"),
        lightwalletd_url,
    )
    .await?;
    if let Some(n) = max_real_notes_per_bundle {
        crate::db::put_prop(
            connection,
            &format!("voting_round_bundle_policy:{round_id}"),
            &n.to_string(),
        )
        .await?;
    }
    Ok(())
}

/// Loads the round config persisted by [`save_round_config`].
pub async fn load_round_config(
    connection: &mut SqliteConnection,
    round_id: &str,
) -> Result<(String, String, Option<u32>, String)> {
    let params = crate::db::get_prop(connection, &format!("voting_round_params:{round_id}"))
        .await?
        .ok_or_else(|| {
            anyhow!("no saved voting config for round {round_id}; run delegation_prepare first")
        })?;
    let name = crate::db::get_prop(connection, &format!("voting_round_name:{round_id}"))
        .await?
        .ok_or_else(|| anyhow!("no saved voting round name for round {round_id}"))?;
    let lwd = crate::db::get_prop(connection, &format!("voting_round_lwd:{round_id}"))
        .await?
        .ok_or_else(|| anyhow!("no saved lightwalletd URL for round {round_id}"))?;
    let policy = crate::db::get_prop(
        connection,
        &format!("voting_round_bundle_policy:{round_id}"),
    )
    .await?
    .and_then(|s| s.parse::<u32>().ok());
    Ok((params, name, policy, lwd))
}

/// Persists the PIR layout used to build a round's delegation payload, so the
/// proving step can resume after a restart without re-reading the chain config.
pub async fn save_pir_layout(
    connection: &mut SqliteConnection,
    round_id: &str,
    pir_layout: &zcash_voting::config::PirLayout,
) -> Result<()> {
    let json = serde_json::to_string(pir_layout)?;
    crate::db::put_prop(connection, &format!("voting_round_pir:{round_id}"), &json).await
}

/// Loads the PIR layout persisted by [`save_pir_layout`].
pub async fn load_pir_layout(
    connection: &mut SqliteConnection,
    round_id: &str,
) -> Result<Option<zcash_voting::config::PirLayout>> {
    let Some(json) =
        crate::db::get_prop(connection, &format!("voting_round_pir:{round_id}")).await?
    else {
        return Ok(None);
    };
    Ok(Some(serde_json::from_str(&json)?))
}

/// Resolves lightwalletd-derived delegation inputs for a voting round.
pub async fn gather_lwd_inputs(
    lightwalletd_url: &str,
    network: VotingNetwork,
    round_params: &VotingRoundParams,
    round_name: &str,
) -> Result<zcash_voting::delegate::DelegationLwdInputs> {
    Ok(zcash_voting::delegate::gather_delegation_lwd_inputs(
        zcash_voting::delegate::ResolveDelegationLwdParams {
            lightwalletd_url,
            network,
            round_params: round_params.clone(),
            round_name,
        },
    )
    .await?)
}

/// Caller-selected note and witness material for one voting round.
pub struct RoundInputs {
    pub note_infos: Vec<NoteInfo>,
    pub witnesses: Vec<WitnessData>,
}

/// Loads eligible Ironwood notes and snapshot-rooted witnesses from the wallet DB.
///
/// Replicates the sync guard and rewind logic of the send path: the wallet
/// must be synced through the round snapshot height, and each note's witness
/// is rewound to the snapshot-height Ironwood frontier whose root must match
/// the round's `nc_root`.
pub async fn load_round_inputs(
    network: &WalletNetwork,
    connection: &mut SqliteConnection,
    client: &mut crate::Client,
    account: u32,
    snapshot_height: u32,
    nc_root: &[u8],
) -> Result<RoundInputs> {
    // Sync guard: the wallet must be synced through the round snapshot height.
    let h = crate::sync::get_db_height(connection, account).await?;
    ensure!(
        h.height >= snapshot_height,
        "wallet is not synced to the round snapshot height {snapshot_height} (current {})",
        h.height
    );

    // Select unspent, unlocked Ironwood (pool 3) ZEC notes that existed at
    // the snapshot height (mirrors eligible_voting_weight): notes created
    // after the snapshot sit past the anchor edge and cannot be rewound.
    let notes = unspent_ironwood_notes(connection, account, snapshot_height).await?;
    ensure!(
        !notes.is_empty(),
        "no unspent Ironwood notes available for voting"
    );

    // Anchor the witnesses at the snapshot-height Ironwood frontier. The fork
    // enforces witness.root == nc_root anyway; fail fast here with a clear error.
    let (_, _, ironwood_frontier) =
        crate::sync::get_tree_state(network, client, snapshot_height).await?;
    let edge = ironwood_frontier.to_edge(&OrchardHasher::default());
    let anchor_root = edge.root(&OrchardHasher::default());
    ensure!(
        anchor_root.as_slice() == nc_root,
        "Ironwood anchor root does not match round nc_root"
    );
    let edge = edge.to_auth_path(&OrchardHasher::default());
    let ero = empty_roots(&OrchardHasher::default());

    let ovk = crate::account::get_orchard_vk(connection, account)
        .await?
        .ok_or_else(|| anyhow!("account {account} has no Orchard viewing key"))?;
    let ufvk = unified_full_viewing_key(network, connection, account).await?;

    let mut note_infos = Vec::with_capacity(notes.len());
    let mut witnesses = Vec::with_capacity(notes.len());
    for (id, scope) in notes {
        let (note, merkle_path) = crate::account::get_orchard_note(
            connection,
            id,
            h.height,
            &ovk,
            &edge,
            &ero,
            orchard::NoteVersion::V3,
            Some(edge.1),
        )
        .await
        .with_context(|| format!("load Ironwood note {id}"))?;
        let position = merkle_path.position() as u64;
        let scope = match scope {
            0 => orchard::keys::Scope::External,
            1 => orchard::keys::Scope::Internal,
            _ => return Err(anyhow!("unexpected note scope {scope}")),
        };
        let info = NoteInfo::from_orchard_note(&note, position, scope, &ufvk, network)?;
        let auth_path = merkle_path
            .auth_path()
            .iter()
            .map(|sibling| sibling.to_bytes().to_vec())
            .collect();
        witnesses.push(WitnessData {
            note_commitment: info.commitment.clone(),
            position,
            root: anchor_root.to_vec(),
            auth_path,
        });
        note_infos.push(info);
    }

    Ok(RoundInputs {
        note_infos,
        witnesses,
    })
}

/// Unspent, unlocked Ironwood (pool 3) ZEC notes that existed at or before
/// `snapshot_height`, as `(note_id, scope)`.
async fn unspent_ironwood_notes(
    connection: &mut SqliteConnection,
    account: u32,
    snapshot_height: u32,
) -> Result<Vec<(u32, u8)>> {
    let notes = sqlx::query(
        "SELECT a.id_note, a.scope
         FROM notes a
         LEFT JOIN spends b ON a.id_note = b.id_note
         LEFT JOIN assets ast ON a.id_asset = ast.id_asset
         WHERE b.id_note IS NULL AND a.account = ?
           AND a.pool = 3 AND a.locked = 0
           AND a.height <= ?
           AND COALESCE(ast.asset_base, X'0000000000000000000000000000000000000000000000000000000000000000') = X'0000000000000000000000000000000000000000000000000000000000000000'",
    )
    .bind(account)
    .bind(snapshot_height)
    .map(|row: sqlx::sqlite::SqliteRow| {
        let id: u32 = row.get(0);
        let scope: Option<u8> = row.get(1);
        (id, scope.unwrap_or(0))
    })
    .fetch_all(&mut *connection)
    .await?;

    Ok(notes)
}

/// Computes the quantized voting weight for the account's currently-unspent
/// Ironwood ZEC notes at `snapshot_height`, using the same canonical bundle
/// planning as the delegation prepare step (sort by value, greedy chunking,
/// per-bundle ballot quantization) — but from the local DB only: no tree
/// state, no witnesses.
///
/// Notes received after the snapshot are excluded, mirroring the prepare-time
/// rejection (their witness cannot anchor at the snapshot tree).
pub async fn eligible_voting_weight(
    connection: &mut SqliteConnection,
    account: u32,
    snapshot_height: u32,
) -> Result<u64> {
    let rows = sqlx::query(
        "SELECT a.value, a.position, a.cmx, a.nullifier, a.scope, a.diversifier, a.rcm, a.rho
         FROM notes a
         LEFT JOIN spends b ON a.id_note = b.id_note
         LEFT JOIN assets ast ON a.id_asset = ast.id_asset
         WHERE b.id_note IS NULL AND a.account = ?
           AND a.pool = 3 AND a.locked = 0
           AND a.height <= ?
           AND COALESCE(ast.asset_base, X'0000000000000000000000000000000000000000000000000000000000000000') = X'0000000000000000000000000000000000000000000000000000000000000000'",
    )
    .bind(account)
    .bind(snapshot_height)
    .map(|row: sqlx::sqlite::SqliteRow| {
        (
            row.get::<i64, _>(0) as u64,   // value
            row.get::<u32, _>(1),          // position
            row.get::<Vec<u8>, _>(2),      // cmx
            row.get::<Vec<u8>, _>(3),      // nullifier
            row.get::<Option<u8>, _>(4),   // scope
            row.get::<Vec<u8>, _>(5),      // diversifier
            row.get::<Vec<u8>, _>(6),      // rcm (rseed)
            row.get::<Vec<u8>, _>(7),      // rho
        )
    })
    .fetch_all(&mut *connection)
    .await?;

    let notes: Vec<NoteInfo> = rows
        .into_iter()
        .map(
            |(value, position, cmx, nullifier, scope, diversifier, rcm, rho)| NoteInfo {
                commitment: cmx,
                nullifier,
                value,
                position: position as u64,
                diversifier,
                rho,
                rseed: rcm,
                scope: scope.unwrap_or(0) as u32,
                ufvk_str: String::new(),
            },
        )
        .collect();

    Ok(
        minimum_voting_eligibility_for_notes(&notes, BundlePolicy::default())
            .map(|e| e.eligible_weight)
            .unwrap_or(0),
    )
}

async fn unified_full_viewing_key(
    network: &WalletNetwork,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<UnifiedFullViewingKey> {
    let encoded = crate::key::get_account_ufvk(network, connection, account, 4).await?;
    UnifiedFullViewingKey::decode(network, &encoded)
        .map_err(|e| anyhow!("invalid account UFVK: {e}"))
}

/// Wallet keys plus the voting hotkey used to build delegation PCZTs.
pub struct VotingIdentity {
    pub delegation_keys: DelegationKeys,
    pub hotkey: VotingHotkey,
}

/// Loads the delegation keys and voting hotkey for an account.
///
/// `round_name` must be the resolved round name so the PCZT display metadata
/// matches the prepared round.
pub async fn load_voting_identity(
    connection: &mut SqliteConnection,
    account: u32,
    network: VotingNetwork,
    round_name: &str,
) -> Result<VotingIdentity> {
    let fingerprint = crate::db::get_account_fingerprint(connection, account)
        .await?
        .ok_or_else(|| anyhow!("account {account} has no seed fingerprint"))?;
    let seed_fingerprint: [u8; 32] = fingerprint
        .try_into()
        .map_err(|_| anyhow!("seed fingerprint must be 32 bytes"))?;
    let account_index = crate::db::get_account_aindex(connection, account).await?;
    let ovk = crate::account::get_orchard_vk(connection, account)
        .await?
        .ok_or_else(|| anyhow!("account {account} has no Orchard viewing key"))?;
    let fvk_bytes = ovk.to_bytes();
    let hotkey = voting_hotkey_load(connection, network).await?;
    let delegation_keys = DelegationKeys::with_voting_hotkey(
        fvk_bytes.to_vec(),
        &hotkey,
        seed_fingerprint,
        account_index,
        round_name.to_string(),
    )?;
    Ok(VotingIdentity {
        delegation_keys,
        hotkey,
    })
}

/// Prepares one delegation bundle from caller-supplied notes and witnesses.
pub async fn prepare_delegation_bundle(
    pool: SqlitePool,
    wallet_id: &str,
    lwd: zcash_voting::delegate::DelegationLwdInputs,
    session_json: Option<&str>,
    round_note_infos: Vec<NoteInfo>,
    delegation_keys: DelegationKeys,
    witnesses: Vec<WitnessData>,
    bundle_index: u32,
    bundle_policy: BundlePolicy,
) -> Result<PreparedDelegationBundle> {
    let db = open_voting_db(pool, wallet_id).await?;
    Ok(zcash_voting::delegate::prepare_delegation_bundle_with_inputs(
        &db,
        PrepareDelegationBundleWithInputsParams {
            lwd,
            session_json,
            round_note_infos,
            delegation_keys,
            witnesses,
            bundle_index,
            bundle_policy,
        },
    )
    .await?)
}

/// Signs a delegation signing request with the wallet's own ZIP-32 seed.
///
/// Mirrors the fork's wallet-example signer: verifies the request fingerprint
/// against the seed, derives the account SpendAuth key, randomizes it with the
/// stored alpha, and signs the PCZT sighash.
pub fn sign_delegation_request(
    seed: &[u8],
    request: DelegationSigningRequest,
) -> Result<([u8; 64], [u8; 32])> {
    let seed_fingerprint = zip32::fingerprint::SeedFingerprint::from_seed(seed)
        .ok_or_else(|| anyhow!("wallet seed length is not valid for ZIP-32"))?;
    ensure!(
        seed_fingerprint.to_bytes() == request.seed_fingerprint,
        "wallet seed fingerprint does not match delegation signing request"
    );

    let account = AccountId::try_from(request.account_index)
        .map_err(|_| anyhow!("invalid account_index {}", request.account_index))?;
    let usk = UnifiedSpendingKey::from_seed(&request.network, seed, account)?;
    let sk = *usk.orchard();
    let ask = orchard::keys::SpendAuthorizingKey::from(&sk);
    let alpha = Option::<halo2_proofs::pasta::pallas::Scalar>::from(
        halo2_proofs::pasta::pallas::Scalar::from_repr(request.alpha),
    )
    .ok_or_else(|| anyhow!("delegation alpha is not a valid Pallas scalar"))?;
    let rsk = ask.randomize(&alpha);
    let mut rng = OsRng;
    let sig = rsk.sign(&mut rng, &request.sighash);
    Ok(((&sig).into(), request.sighash))
}

/// Proves one prepared delegation bundle and assembles the chain-ready submission.
///
/// The proof is generated against the PIR server; the bundle's SpendAuth
/// signature comes from the wallet's own seed. `pczt_bytes` is the setup PCZT
/// (empty skips the sighash consistency check, mirroring the fork's Keystone path).
#[allow(clippy::too_many_arguments)]
pub async fn prove_and_submit_delegation(
    pool: SqlitePool,
    wallet_id: &str,
    prepared: &PreparedDelegationBundle,
    seed: &[u8],
    pczt_bytes: Vec<u8>,
    pir_layout: zcash_voting::config::PirLayout,
    pir_server_url: &str,
) -> Result<(DelegationSubmission, String)> {
    prove_and_submit_delegation_with_progress(
        pool,
        wallet_id,
        prepared,
        seed,
        pczt_bytes,
        pir_layout,
        pir_server_url,
        &NoopProgressReporter,
    )
    .await
}

/// [`prove_and_submit_delegation`] with a live progress reporter. The library
/// emits the PCZT/proof stages; the host emits `SigningPayload` and
/// `PayloadReady` bookends around signing and assembly. Returns the
/// submission together with its vote-chain wire JSON body.
#[allow(clippy::too_many_arguments)]
pub async fn prove_and_submit_delegation_with_progress(
    pool: SqlitePool,
    wallet_id: &str,
    prepared: &PreparedDelegationBundle,
    seed: &[u8],
    pczt_bytes: Vec<u8>,
    pir_layout: zcash_voting::config::PirLayout,
    pir_server_url: &str,
    progress: &dyn DelegationProgressReporter,
) -> Result<(DelegationSubmission, String)> {
    let db = open_voting_db(pool, wallet_id).await?;

    let _setup = prepared.setup(&db, progress).await?;
    let request = prepared.signing_request(&db).await?;
    progress.on_progress(DelegationProgress::SigningPayload);
    let (sig, sighash) = sign_delegation_request(seed, request)?;

    let pir_client = zcash_voting::connect_pir_blocking(
        pir_layout,
        pir_server_url,
        Arc::new(zcash_voting::HyperTransport::new()),
    )?;
    prepared.prove(&db, &pir_client, progress).await?;

    let bundle = prepared
        .signed_bundle(&db, pczt_bytes, PreparedSigner::signature(sig, sighash))
        .await?;
    progress.on_progress(DelegationProgress::PayloadReady);

    let wire = zcash_voting::wire::DelegationSubmissionWire::try_from(&bundle.submission)?;
    let wire_json = wire.to_json()?;
    Ok((bundle.submission, wire_json))
}

/// Records a confirmed delegation transaction (persists the bundle's public
/// VAN position — required before any vote can be committed).
pub async fn confirm_delegation(
    pool: SqlitePool,
    wallet_id: &str,
    round_id: &str,
    bundle_index: u32,
    tx_hash: &str,
    events: &[TxEvent],
) -> Result<DelegationConfirmation> {
    let db = open_voting_db(pool, wallet_id).await?;
    Ok(confirm_delegation_submission(&db, round_id, bundle_index, tx_hash, events).await?)
}

/// Records a confirmed cast-vote transaction.
pub async fn confirm_vote(
    pool: SqlitePool,
    wallet_id: &str,
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
    tx_hash: &str,
    events: &[TxEvent],
) -> Result<VoteConfirmation> {
    let db = open_voting_db(pool, wallet_id).await?;
    Ok(confirm_vote_submission(&db, round_id, bundle_index, proposal_id, tx_hash, events).await?)
}

/// Syncs the vote-authority-note tree and derives this bundle's VAN witness.
///
/// Requires a confirmed delegation (VAN position persisted).
pub async fn vote_van_witness(
    pool: SqlitePool,
    wallet_id: &str,
    round_id: &str,
    bundle_index: u32,
    vote_node_url: &str,
) -> Result<VanWitness> {
    let db = open_voting_db(pool, wallet_id).await?;
    let anchor_height = zcash_voting::prelude::sync_vote_tree(&db, round_id, vote_node_url).await?;
    Ok(zcash_voting::prelude::van_witness(&db, round_id, bundle_index, anchor_height).await?)
}

/// Builds, hotkey-signs, and persists signed vote commitments for a draft batch.
pub async fn commit_votes(
    pool: SqlitePool,
    wallet_id: &str,
    round_id: &str,
    bundle_index: u32,
    drafts: &[DraftVote],
    witness: &VanWitness,
    hotkey: &VotingHotkey,
) -> Result<SignedVoteCommitments> {
    commit_votes_with_progress(
        pool,
        wallet_id,
        round_id,
        bundle_index,
        drafts,
        witness,
        hotkey,
        &NoopProgressReporter,
    )
    .await
}

/// [`commit_votes`] with a live stage reporter (proof / share-payload /
/// signing stages per proposal).
#[allow(clippy::too_many_arguments)]
pub async fn commit_votes_with_progress(
    pool: SqlitePool,
    wallet_id: &str,
    round_id: &str,
    bundle_index: u32,
    drafts: &[DraftVote],
    witness: &VanWitness,
    hotkey: &VotingHotkey,
    stages: &dyn VoteCommitStageReporter,
) -> Result<SignedVoteCommitments> {
    let db = open_voting_db(pool, wallet_id).await?;
    Ok(zcash_voting::prelude::commit_batch(
        &db,
        round_id,
        bundle_index,
        drafts,
        witness,
        VoteSigner::hotkey(hotkey),
        stages,
    )
    .await?)
}

/// Reconstructs the chain-ready vote submission and helper-share payloads
/// for one committed vote.
pub async fn vote_payloads(
    pool: SqlitePool,
    wallet_id: &str,
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
) -> Result<(VoteSubmission, Vec<SharePayload>)> {
    let db = open_voting_db(pool, wallet_id).await?;
    let committed = CommittedVote::recover(&db, round_id, bundle_index, proposal_id).await?;
    Ok((
        committed.submission(&db).await?,
        committed.share_payloads().to_vec(),
    ))
}

/// Reconstructs the chain-ready wire JSON for a committed vote.
pub async fn vote_wire_json(
    pool: SqlitePool,
    wallet_id: &str,
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
) -> Result<String> {
    let db = open_voting_db(pool, wallet_id).await?;
    let committed = CommittedVote::recover(&db, round_id, bundle_index, proposal_id).await?;
    let signed = committed.signed_commitment(&db).await?;
    Ok(zcash_voting::wire::VoteCommitmentWire::try_from(&signed)?.to_json()?)
}

/// Reconstructs one helper-share payload as helper wire JSON from the
/// persisted commitment bundle.
pub async fn share_wire_json(
    pool: SqlitePool,
    wallet_id: &str,
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
    share_index: u32,
    vc_tree_position: Option<u64>,
    submit_at: u64,
) -> Result<String> {
    let db = open_voting_db(pool, wallet_id).await?;
    let bundle = zcash_voting::recovery::recoverable_commitment_bundle(
        &db,
        round_id,
        bundle_index,
        proposal_id,
    )
    .await?
    .ok_or_else(|| {
        anyhow!(
            "commitment bundle not found for round {round_id} bundle {bundle_index} proposal {proposal_id}"
        )
    })?;
    let position = vc_tree_position.unwrap_or(bundle.vc_tree_position);
    Ok(zcash_voting::share::recover_wire_json(
        &bundle.commitment_bundle_json,
        proposal_id,
        share_index,
        position,
        submit_at,
    )?)
}

/// Records successful vote-chain and helper-share submissions for one vote.
pub async fn record_vote_execution(
    pool: SqlitePool,
    wallet_id: &str,
    round_id: &str,
    bundle_index: u32,
    proposal_id: u32,
    vote_tx_hash: &str,
    vc_tree_position: u64,
    shares: &[(u32, Vec<String>, u64, bool)],
) -> Result<()> {
    let db = open_voting_db(pool, wallet_id).await?;
    let committed = CommittedVote::recover(&db, round_id, bundle_index, proposal_id).await?;
    committed.record_submission(&db, vote_tx_hash).await?;
    committed.record_vc_position(&db, vc_tree_position).await?;
    for (share_index, sent_to_urls, submit_at, confirmed) in shares {
        committed
            .record_share(&db, *share_index, sent_to_urls, *submit_at)
            .await?;
        if *confirmed {
            committed.confirm_share(&db, *share_index).await?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Prepared bundle cache
//
// PreparedDelegationBundle is plain data and the fork persists all durable
// round/bundle/witness state in the voting DB. The cache avoids re-running the
// wallet-side input gathering between FRB steps (single-wallet app).
// ---------------------------------------------------------------------------

static PREPARED_BUNDLES: OnceLock<Mutex<HashMap<String, PreparedDelegationBundle>>> =
    OnceLock::new();

fn bundle_cache_key(wallet_id: &str, round_id: &str, bundle_index: u32) -> String {
    format!("{wallet_id}:{round_id}:{bundle_index}")
}

pub fn cache_prepared_bundle(wallet_id: &str, prepared: PreparedDelegationBundle) {
    let mut cache = PREPARED_BUNDLES
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .expect("voting prepared bundle cache poisoned");
    let key = bundle_cache_key(wallet_id, &prepared.round_id, prepared.bundle_index);
    cache.insert(key, prepared);
}

pub fn load_prepared_bundle(
    wallet_id: &str,
    round_id: &str,
    bundle_index: u32,
) -> Result<PreparedDelegationBundle> {
    let cache = PREPARED_BUNDLES
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .expect("voting prepared bundle cache poisoned");
    cache
        .get(&bundle_cache_key(wallet_id, round_id, bundle_index))
        .cloned()
        .ok_or_else(|| {
            anyhow!("delegation bundle not prepared; run delegation_prepare first")
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::pasta::pallas::Scalar;
    use orchard::primitives::redpallas::{Signature, VerificationKey};
    use zcash_protocol::{
        consensus::{BlockHeight, OrchardMode},
        local_consensus::LocalNetwork,
    };
    use zip32::fingerprint::SeedFingerprint;

    /// Standard BIP-39 test vector; its 64-byte seed is valid for ZIP-32.
    fn test_seed() -> Vec<u8> {
        Mnemonic::parse(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .unwrap()
        .to_seed("")
        .to_vec()
    }

    fn local_network() -> LocalNetwork {
        LocalNetwork {
            overwinter: Some(BlockHeight::from_u32(1)),
            sapling: Some(BlockHeight::from_u32(1)),
            blossom: Some(BlockHeight::from_u32(1)),
            heartwood: Some(BlockHeight::from_u32(1)),
            canopy: Some(BlockHeight::from_u32(1)),
            nu5: Some(BlockHeight::from_u32(1)),
            nu6: Some(BlockHeight::from_u32(1)),
            nu6_1: Some(BlockHeight::from_u32(1)),
            nu6_2: Some(BlockHeight::from_u32(1)),
            nu6_3: Some(BlockHeight::from_u32(250)),
            nu7: None,
            orchard_mode: OrchardMode::Normal,
        }
    }

    fn signing_request(
        fingerprint: [u8; 32],
        account_index: u32,
        alpha: [u8; 32],
    ) -> DelegationSigningRequest {
        DelegationSigningRequest {
            account_index,
            network: VotingNetwork::Mainnet,
            seed_fingerprint: fingerprint,
            sighash: [7u8; 32],
            alpha,
        }
    }

    /// Rebuilds the signing key from the request the same way
    /// `sign_delegation_request` does, and verifies the returned signature.
    fn assert_valid_signature(
        request: &DelegationSigningRequest,
        seed: &[u8],
        sig_bytes: &[u8; 64],
    ) {
        let account = AccountId::try_from(request.account_index).unwrap();
        let usk = UnifiedSpendingKey::from_seed(&request.network, seed, account).unwrap();
        let sk = *usk.orchard();
        let ask = orchard::keys::SpendAuthorizingKey::from(&sk);
        let alpha = Scalar::from_repr(request.alpha).unwrap();
        let vk = VerificationKey::from(&ask.randomize(&alpha));
        let sig = Signature::from(*sig_bytes);
        vk.verify(&request.sighash, &sig).unwrap();
    }

    #[test]
    fn voting_network_maps_each_wallet_network_to_fork_network() {
        assert_eq!(
            voting_network(&WalletNetwork::Main).unwrap(),
            VotingNetwork::Mainnet
        );
        assert_eq!(
            voting_network(&WalletNetwork::Test).unwrap(),
            VotingNetwork::Testnet
        );
        assert_eq!(
            voting_network(&WalletNetwork::Regtest(local_network())).unwrap(),
            VotingNetwork::Regtest
        );
    }

    #[test]
    fn voting_network_rejects_zsa_regtest() {
        let err = voting_network(&WalletNetwork::ZsaRegtest(local_network())).unwrap_err();
        assert_eq!(err.to_string(), "voting is not supported on the ZSA network");
    }

    #[test]
    fn sign_delegation_request_rejects_invalid_seed_length() {
        let request = signing_request([0u8; 32], 0, [0u8; 32]);
        let err = sign_delegation_request(&[0xAAu8; 16], request).unwrap_err();
        assert_eq!(
            err.to_string(),
            "wallet seed length is not valid for ZIP-32"
        );
    }

    #[test]
    fn sign_delegation_request_rejects_seed_fingerprint_mismatch() {
        let seed = test_seed();
        let mut fingerprint = SeedFingerprint::from_seed(&seed).unwrap().to_bytes();
        fingerprint[0] ^= 0x01;
        let request = signing_request(fingerprint, 0, [0u8; 32]);
        let err = sign_delegation_request(&seed, request).unwrap_err();
        assert_eq!(
            err.to_string(),
            "wallet seed fingerprint does not match delegation signing request"
        );
    }

    #[test]
    fn sign_delegation_request_rejects_account_index_at_or_above_2_to_31() {
        let seed = test_seed();
        let fingerprint = SeedFingerprint::from_seed(&seed).unwrap().to_bytes();
        let request = signing_request(fingerprint, 1 << 31, [0u8; 32]);
        let err = sign_delegation_request(&seed, request).unwrap_err();
        assert_eq!(err.to_string(), "invalid account_index 2147483648");

        // One below the boundary passes the account-index check; the request
        // then fails on the intentionally non-canonical alpha, pinning the
        // exact boundary.
        let request = signing_request(fingerprint, (1 << 31) - 1, [0xFFu8; 32]);
        let err = sign_delegation_request(&seed, request).unwrap_err();
        assert_eq!(
            err.to_string(),
            "delegation alpha is not a valid Pallas scalar"
        );
    }

    #[test]
    fn sign_delegation_request_rejects_noncanonical_alpha() {
        let seed = test_seed();
        let fingerprint = SeedFingerprint::from_seed(&seed).unwrap().to_bytes();
        let request = signing_request(fingerprint, 0, [0xFFu8; 32]);
        let err = sign_delegation_request(&seed, request).unwrap_err();
        assert_eq!(
            err.to_string(),
            "delegation alpha is not a valid Pallas scalar"
        );
    }

    #[test]
    fn sign_delegation_request_signs_with_zero_alpha_and_returns_sighash() {
        let seed = test_seed();
        let fingerprint = SeedFingerprint::from_seed(&seed).unwrap().to_bytes();
        let request = signing_request(fingerprint, 0, [0u8; 32]);
        let (sig_bytes, sighash) = sign_delegation_request(&seed, request).unwrap();
        assert_eq!(sighash, [7u8; 32]);
        assert_valid_signature(&request, &seed, &sig_bytes);
    }

    #[test]
    fn sign_delegation_request_signs_with_nonzero_alpha_under_randomized_key() {
        let seed = test_seed();
        let fingerprint = SeedFingerprint::from_seed(&seed).unwrap().to_bytes();
        let mut alpha = [0u8; 32];
        alpha[0] = 7;
        let request = signing_request(fingerprint, 0, alpha);
        let (sig_bytes, _) = sign_delegation_request(&seed, request).unwrap();
        assert_valid_signature(&request, &seed, &sig_bytes);

        // Negative control: the *unrandomized* ak must not verify, proving
        // the alpha randomizer actually enters the signing key.
        let account = AccountId::try_from(request.account_index).unwrap();
        let usk = UnifiedSpendingKey::from_seed(&request.network, &seed, account).unwrap();
        let ask = orchard::keys::SpendAuthorizingKey::from(&*usk.orchard());
        let zero = Scalar::from_repr([0u8; 32]).unwrap();
        let vk_ak = VerificationKey::from(&ask.randomize(&zero));
        let sig = Signature::from(sig_bytes);
        assert!(vk_ak.verify(&request.sighash, &sig).is_err());
    }

    #[test]
    fn bundle_cache_key_formats_wallet_round_bundle() {
        assert_eq!(
            bundle_cache_key("wallet-abc", "round-42", 0),
            "wallet-abc:round-42:0"
        );
        assert_eq!(bundle_cache_key("w", "r", u32::MAX), "w:r:4294967295");
        assert_eq!(bundle_cache_key("", "", 0), "::0");
        assert_ne!(
            bundle_cache_key("a", "r", 1),
            bundle_cache_key("b", "r", 1),
            "different wallet ids must not share a cache key"
        );
    }
}
