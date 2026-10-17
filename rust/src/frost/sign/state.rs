//! A snapshot of everything the signing protocol needs to decide what to do.
//!
//! `observe_sign` is the protocol's single reader of state. Like DKG's
//! `observe`, it first ingests peer packages that have arrived in memos since
//! the last step (idempotent — primary keys dedup), then queries the resulting
//! DB rows for the status flags `next_sign_task` reads.

use anyhow::Result;
use bincode::{config, Decode, Encode};
use reddsa_legacy::frost::redpallas::frost::{
    keys::VerifyingShare, round1::SigningCommitments, round2::SignatureShare, SigningPackage,
};
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};
use tracing::info;
use zcash_protocol::memo::Memo;

use crate::{
    api::{coin::Network, frost::FrostSignParams, pay::PcztPackage},
    db::{get_prop, put_prop},
    frost::{
        dkg::{get_dkg_params, load_peer_verifying_key},
        protocol::{group_sk, lookup_broadcast_account_from_sk, mailbox_address},
        FrostSigMessage, P,
    },
};

use super::{COMMITMENT_PREFIX, COORDINATOR_PREFIX, SIGPACKAGE_PREFIX, SIGSHARE_PREFIX};

// ── SignState ─────────────────────────────────────────────────────────────────

/// The wallet's signing state at one instant, as the planner sees it.
#[derive(Clone)]
pub struct SignState {
    pub account: u32,
    pub funding_account: u32,
    /// Our participant id from dkg_params (u8 for coordinator comparison).
    pub self_id_u8: u8,
    pub coordinator: u8,
    pub n: u8,
    pub t: u8,
    pub sighash: Vec<u8>,
    pub nsigs: u32,
    pub birth_height: u32,

    pub mailbox: Option<u32>,
    /// Our own mailbox address; the coordinator sends it with the signing
    /// packages so that members need no stored address for it.
    pub self_mailbox_address: Option<String>,
    /// The coordinator's private mailbox address for this session, from its
    /// announcement (our own when we coordinate). None until it arrives.
    pub coordinator_address: Option<String>,
    pub broadcast: Option<u32>,
    pub broadcast_address: Option<String>,

    // Phase 1: commitments
    /// True when our own commitment row exists in frost_commitments.
    pub own_commitments_stored: bool,
    /// commitment_counts[idx] = how many distinct participants have a commitment row for input idx.
    pub commitment_counts: Vec<usize>,

    // Phase 2: signing packages
    /// How many inputs have a signing package stored in frost_signatures.
    pub sigpackages_count: u32,

    // Phase 3: signature shares
    /// True when our sigshare row exists in frost_signatures.
    pub own_sigshares_stored: bool,

    // Phase 4: aggregation (coordinator)
    /// sigshare_counts[idx] = how many participants have a sigshare in frost_commitments for input idx.
    pub sigshare_counts: Vec<usize>,
}

impl SignState {
    pub fn is_coordinator(&self) -> bool {
        self.self_id_u8 == self.coordinator
    }
}

// ── observe_sign ──────────────────────────────────────────────────────────────

/// Read protocol state. Returns `None` when no signing session is in progress.
///
/// Like DKG's `observe`, this is not purely read-only: it ingests peer
/// packages from memos first. All ingest operations are idempotent.
pub async fn observe_sign(
    network: &Network,
    connection: &mut SqliteConnection,
) -> Result<Option<SignState>> {
    // Guard: is there a signing session?
    let exists = sqlx::query("SELECT 1 FROM props WHERE key = 'frost_pczt'")
        .fetch_optional(&mut *connection)
        .await?;
    if exists.is_none() {
        return Ok(None);
    }

    let params = load_sign_params(&mut *connection).await?;
    let account = params.account;

    let pczt_pkg = load_pczt_pkg(&mut *connection).await?;
    let pczt = pczt::Pczt::parse(&pczt_pkg.pczt).expect("Failed to parse PCZT");
    let sighash = get_sighash(pczt);
    let nsigs = (pczt_pkg.orchard_indices.len() + pczt_pkg.ironwood_indices.len()) as u32;

    let dkg_params = get_dkg_params(&mut *connection, account).await?;
    let birth_height = dkg_params.birth_height;
    let self_id_u8 = dkg_params.id;

    // Read-only account lookups (creation is an exec effect). The broadcast
    // account is the one `sk` builds: the same account the DKG's address
    // hash produced, found without the participant addresses.
    let mailbox = mailbox_account_id(&mut *connection, account).await?;
    let self_mailbox_address = match mailbox {
        Some(mailbox) => Some(mailbox_address(network, &mut *connection, mailbox).await?),
        None => None,
    };
    let sk = group_sk(network, &mut *connection, account).await?;
    let broadcast_lookup =
        lookup_broadcast_account_from_sk(network, &mut *connection, &sk).await?;
    let (broadcast, broadcast_address) = match broadcast_lookup {
        Some((id, addr)) => (Some(id), Some(addr)),
        None => (None, None),
    };

    // Ingest memos (idempotent).
    if let Some(mb) = mailbox {
        ingest_commitment_memos(
            &mut *connection,
            account,
            mb,
            &sighash,
            nsigs,
            dkg_params.n as u16,
        )
        .await?;
        ingest_sigshare_memos(
            &mut *connection,
            account,
            mb,
            &sighash,
            nsigs,
            dkg_params.n as u16,
        )
        .await?;
    }
    if let Some(bc) = broadcast {
        ingest_coordinator_memos(&mut *connection, account, bc, &sighash, params.coordinator as u16)
            .await?;
        ingest_sigpackage_memos(
            &mut *connection,
            account,
            bc,
            &sighash,
            nsigs,
            params.coordinator as u16,
        )
        .await?;
    }

    // For the coordinator: copy own sigshare from frost_signatures into frost_commitments
    // so sigshare_counts is accurate. Idempotent UPDATE — no-op when sigshare not set yet.
    if self_id_u8 == params.coordinator {
        for idx in 0..nsigs {
            sqlx::query(
                "UPDATE frost_commitments SET sigshare =
                (SELECT sigshare FROM frost_signatures
                 WHERE account = ?1 AND sighash = ?2 AND idx = ?3)
                WHERE account = ?1 AND sighash = ?2 AND idx = ?3 AND from_id = ?4",
            )
            .bind(account)
            .bind(&sighash)
            .bind(idx)
            .bind(self_id_u8)
            .execute(&mut *connection)
            .await?;
        }
    }

    // The coordinator's address, after the ingest that may have just stored it.
    let coordinator_address = coordinator_address_opt(&mut *connection).await?;

    // Phase 1: own commitment.
    let own_commitments_stored = sqlx::query(
        "SELECT 1 FROM frost_commitments
         WHERE account = ? AND sighash = ? AND from_id = ? AND commitment IS NOT NULL LIMIT 1",
    )
    .bind(account)
    .bind(&sighash)
    .bind(self_id_u8)
    .fetch_optional(&mut *connection)
    .await?
    .is_some();

    // Per-input commitment counts.
    let mut commitment_counts = vec![0usize; nsigs as usize];
    for idx in 0..nsigs {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM frost_commitments
             WHERE account = ? AND sighash = ? AND idx = ? AND commitment IS NOT NULL",
        )
        .bind(account)
        .bind(&sighash)
        .bind(idx)
        .fetch_one(&mut *connection)
        .await?;
        commitment_counts[idx as usize] = count as usize;
    }

    // Phase 2: sigpackage count.
    let (sigpackages_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM frost_signatures
         WHERE account = ? AND sighash = ? AND sigpackage IS NOT NULL",
    )
    .bind(account)
    .bind(&sighash)
    .fetch_one(&mut *connection)
    .await?;
    let sigpackages_count = sigpackages_count as u32;

    // Phase 3: own sigshare.
    let own_sigshares_stored = sqlx::query(
        "SELECT 1 FROM frost_signatures
         WHERE account = ? AND sighash = ? AND sigshare IS NOT NULL LIMIT 1",
    )
    .bind(account)
    .bind(&sighash)
    .fetch_optional(&mut *connection)
    .await?
    .is_some();

    // Phase 4: per-input sigshare counts (coordinator only, but harmless to query for all).
    let mut sigshare_counts = vec![0usize; nsigs as usize];
    for idx in 0..nsigs {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM frost_commitments
             WHERE account = ? AND sighash = ? AND idx = ? AND sigshare IS NOT NULL",
        )
        .bind(account)
        .bind(&sighash)
        .bind(idx)
        .fetch_one(&mut *connection)
        .await?;
        sigshare_counts[idx as usize] = count as usize;
    }

    Ok(Some(SignState {
        account,
        funding_account: params.funding_account,
        self_id_u8,
        coordinator: params.coordinator,
        n: dkg_params.n,
        t: dkg_params.t,
        sighash,
        nsigs,
        birth_height,
        mailbox,
        self_mailbox_address,
        coordinator_address,
        broadcast,
        broadcast_address,
        own_commitments_stored,
        commitment_counts,
        sigpackages_count,
        own_sigshares_stored,
        sigshare_counts,
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub(super) async fn load_sign_params(connection: &mut SqliteConnection) -> Result<FrostSignParams> {
    let (value,): (String,) =
        sqlx::query_as("SELECT value FROM props WHERE key = 'frost_sign_params'")
            .fetch_one(&mut *connection)
            .await?;
    Ok(serde_json::from_str(&value)?)
}

pub(super) async fn load_pczt_pkg(connection: &mut SqliteConnection) -> Result<PcztPackage> {
    let (value,): (Vec<u8>,) = sqlx::query_as("SELECT value FROM props WHERE key = 'frost_pczt'")
        .fetch_one(&mut *connection)
        .await?;
    let (pkg, _) = bincode::decode_from_slice(&value, config::legacy())?;
    Ok(pkg)
}

pub(super) fn get_sighash(pczt: pczt::Pczt) -> Vec<u8> {
    use zcash_primitives::transaction::{
        sighash::SignableInput, sighash_v5::v5_signature_hash, sighash_v6::v6_signature_hash,
        txid::TxIdDigester, TxVersion,
    };
    let tx = pczt.into_effects().unwrap();
    let txid_parts = tx.digest(TxIdDigester);
    let hash = match tx.version() {
        TxVersion::V6 => v6_signature_hash(&tx, &SignableInput::Shielded, &txid_parts),
        _ => v5_signature_hash(&tx, &SignableInput::Shielded, &txid_parts),
    };
    hash.as_bytes().to_vec()
}

async fn mailbox_account_id(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<u32>> {
    let seed: Option<(String,)> =
        sqlx::query_as("SELECT seed FROM dkg_params WHERE account = ?")
            .bind(account)
            .fetch_optional(&mut *connection)
            .await?;
    let Some((seed,)) = seed else {
        return Ok(None);
    };
    if seed.is_empty() {
        return Ok(None);
    }
    let r: Option<(u32,)> =
        sqlx::query_as("SELECT id_account FROM accounts WHERE seed = ?1")
            .bind(&seed)
            .fetch_optional(&mut *connection)
            .await?;
    Ok(r.map(|(a,)| a))
}

/// Props key holding the coordinator's mailbox address for this session:
/// what its announcement carried, or our own address when we coordinate.
/// Under the `frost_` prefix so that it lives and dies with the session.
/// Stored DKG addresses are never consulted: a restored coordinator has a
/// fresh mailbox that nobody has on file.
pub(super) const COORDINATOR_ADDRESS_PROP: &str = "frost_coordinator_address";

async fn coordinator_address_opt(connection: &mut SqliteConnection) -> Result<Option<String>> {
    get_prop(&mut *connection, COORDINATOR_ADDRESS_PROP).await
}

// ── Memo ingestion ────────────────────────────────────────────────────────────

/// Decode and verify FrostSigMessages from a mailbox, filter by prefix, then
/// call `store` for each valid message.
async fn decode_frost_memos(
    connection: &mut SqliteConnection,
    account: u32,
    mailbox_account: u32,
    prefix: &[u8],
    store: impl AsyncFn(&mut SqliteConnection, &FrostSigMessage) -> Result<()>,
) -> Result<()>
{
    let rows = sqlx::query("SELECT memo_bytes FROM memos WHERE account = ?")
        .bind(mailbox_account)
        .map(|row: SqliteRow| row.get::<Vec<u8>, _>(0))
        .fetch_all(&mut *connection)
        .await?;

    let mut messages = vec![];
    for memo_bytes in rows {
        let Ok(Memo::Arbitrary(pkg_bytes)) = Memo::from_bytes(&memo_bytes) else {
            continue;
        };
        if pkg_bytes.len() < 4 || pkg_bytes[0..4] != *prefix {
            continue;
        }
        if let Ok((msg, _)) = bincode::decode_from_slice::<FrostSigMessage, _>(
            &pkg_bytes[4..],
            config::legacy(),
        ) {
            messages.push(msg);
        }
    }

    for msg in messages {
        let Ok(from_id) = u8::try_from(msg.from_id) else {
            continue;
        };
        if let Some(vk) = load_peer_verifying_key(connection, account, from_id).await? {
            if !msg.verify(&vk) {
                info!(
                    "decode_frost_memos: rejecting message from {} — invalid signature",
                    msg.from_id
                );
                continue;
            }
        }
        store(connection, &msg).await?;
    }
    Ok(())
}

/// Round number under which a participant's verifying share is kept in
/// `dkg_peers`, after the DKG's own rounds 0..3. Learned from commitments
/// messages; a restored coordinator rebuilds the public key package from
/// these rows.
pub(crate) const VERIFYING_SHARE_ROUND: u8 = 4;

async fn ingest_commitment_memos(
    connection: &mut SqliteConnection,
    account: u32,
    mailbox_account: u32,
    sighash: &[u8],
    nsigs: u32,
    participant_count: u16,
) -> Result<()> {
    let sighash = sighash.to_vec();
    decode_frost_memos(
        connection,
        account,
        mailbox_account,
        COMMITMENT_PREFIX,
        async move |conn: &mut SqliteConnection, pkg: &FrostSigMessage| {
            if pkg.sighash.as_slice() != sighash.as_slice()
                || pkg.idx >= nsigs
                || !(1..=participant_count).contains(&pkg.from_id)
                || SigningCommitments::<P>::deserialize(&pkg.data).is_err()
            {
                return Ok(());
            }
            sqlx::query(
                "INSERT INTO frost_commitments
                 (account, sighash, idx, from_id, commitment)
                 VALUES (?, ?, ?, ?, ?) ON CONFLICT DO NOTHING",
            )
            .bind(account)
            .bind(sighash.as_slice())
            .bind(pkg.idx)
            .bind(pkg.from_id)
            .bind(pkg.data.as_slice())
            .execute(&mut *conn)
            .await?;
            if let Some(share) = pkg
                .verifying_share
                .as_deref()
                .filter(|s| VerifyingShare::<P>::deserialize(s).is_ok())
            {
                sqlx::query(
                    "INSERT INTO dkg_peers(account, round, from_id, data)
                     VALUES (?1, ?2, ?3, ?4) ON CONFLICT DO NOTHING",
                )
                .bind(account)
                .bind(VERIFYING_SHARE_ROUND)
                .bind(pkg.from_id as u8)
                .bind(share)
                .execute(&mut *conn)
                .await?;
            }
            Ok(())
        },
    )
    .await
}

#[derive(Encode, Decode)]
pub(super) struct RandomizedSigPackage {
    pub sigpackage: Vec<u8>,
    pub randomizer: Vec<u8>,
}

/// The coordinator's mailbox address for this session, from the
/// announcement it broadcasts at the start.
async fn ingest_coordinator_memos(
    connection: &mut SqliteConnection,
    account: u32,
    broadcast_account: u32,
    sighash: &[u8],
    coordinator: u16,
) -> Result<()> {
    let sighash = sighash.to_vec();
    decode_frost_memos(
        connection,
        account,
        broadcast_account,
        COORDINATOR_PREFIX,
        async move |conn: &mut SqliteConnection, pkg: &FrostSigMessage| {
            if pkg.sighash.as_slice() != sighash.as_slice() || pkg.from_id != coordinator {
                return Ok(());
            }
            let Ok(address) = std::str::from_utf8(&pkg.data) else {
                return Ok(());
            };
            put_prop(&mut *conn, COORDINATOR_ADDRESS_PROP, address).await
        },
    )
    .await
}

async fn ingest_sigpackage_memos(
    connection: &mut SqliteConnection,
    account: u32,
    broadcast_account: u32,
    sighash: &[u8],
    nsigs: u32,
    coordinator: u16,
) -> Result<()> {
    let sighash = sighash.to_vec();
    decode_frost_memos(
        connection,
        account,
        broadcast_account,
        SIGPACKAGE_PREFIX,
        async move |conn: &mut SqliteConnection, pkg: &FrostSigMessage| {
            if pkg.sighash.as_slice() != sighash.as_slice()
                || pkg.idx >= nsigs
                || pkg.from_id != coordinator
            {
                return Ok(());
            }
            let Ok((rsp, _)) = bincode::decode_from_slice::<RandomizedSigPackage, _>(
                &pkg.data,
                config::legacy(),
            ) else {
                return Ok(());
            };
            if SigningPackage::<P>::deserialize(&rsp.sigpackage).is_err()
                || reddsa_legacy::frost::redpallas::Randomizer::deserialize(&rsp.randomizer)
                    .is_err()
            {
                return Ok(());
            }
            sqlx::query(
                "UPDATE frost_signatures
                 SET sigpackage = ?1, randomizer = ?2
                 WHERE account = ?3 AND sighash = ?4 AND idx = ?5",
            )
            .bind(rsp.sigpackage.as_slice())
            .bind(rsp.randomizer.as_slice())
            .bind(account)
            .bind(sighash.as_slice())
            .bind(pkg.idx)
            .execute(&mut *conn)
            .await?;
            Ok(())
        },
    )
    .await
}

async fn ingest_sigshare_memos(
    connection: &mut SqliteConnection,
    account: u32,
    mailbox_account: u32,
    sighash: &[u8],
    nsigs: u32,
    participant_count: u16,
) -> Result<()> {
    let sighash = sighash.to_vec();
    decode_frost_memos(
        connection,
        account,
        mailbox_account,
        SIGSHARE_PREFIX,
        async move |conn: &mut SqliteConnection, pkg: &FrostSigMessage| {
            if pkg.sighash.as_slice() != sighash.as_slice()
                || pkg.idx >= nsigs
                || !(1..=participant_count).contains(&pkg.from_id)
                || SignatureShare::<P>::deserialize(&pkg.data).is_err()
            {
                return Ok(());
            }
            sqlx::query(
                "UPDATE frost_commitments SET sigshare = ?1
                 WHERE account = ?2 AND sighash = ?3 AND idx = ?4 AND from_id = ?5",
            )
            .bind(pkg.data.as_slice())
            .bind(account)
            .bind(sighash.as_slice())
            .bind(pkg.idx)
            .bind(pkg.from_id)
            .execute(&mut *conn)
            .await?;
            Ok(())
        },
    )
    .await
}
