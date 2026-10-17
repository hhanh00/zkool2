use std::{collections::BTreeMap, str::FromStr};

use anyhow::{Context, Result};
use bincode::{config, Decode, Encode};
use bip39::Mnemonic;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use futures::TryStreamExt;
use orchard::keys::{FullViewingKey, Scope, SpendingKey};
use reddsa_legacy::frost::redpallas::{
    frost::{
        keys::{KeyPackage, PublicKeyPackage},
        round1::{SigningCommitments, SigningNonces},
        round2::SignatureShare,
        SigningPackage,
    },
    keys::dkg::{round1, round2},
    Identifier, PallasBlake2b512, Randomizer,
};
use sqlx::{sqlite::SqliteRow, Connection as _, Row, SqliteConnection};
use tracing::info;
use zcash_keys::{address::UnifiedAddress, keys::UnifiedSpendingKey};
use zcash_protocol::memo::Memo;
use zip32::AccountId;

use crate::{
    account::{get_account_seed, get_orchard_vk, new_account},
    api::{account::NewAccount, coin::Network},
    db::{init_account_orchard, store_account_metadata, store_account_orchard_vk},
    pay::{
        plan::{plan_transaction, PaymentRequest, PlanMode},
        sign::sign,
        pool::ALL_SHIELDED_POOLS,
        Recipient,
    },
    Client,
};

pub type P = PallasBlake2b512;

pub type PK1Map = BTreeMap<Identifier, round1::Package>;
pub type PK2Map = BTreeMap<Identifier, round2::Package>;

// ── FrostBytes ───────────────────────────────────────────────────────────────

/// Byte serialization for any type that participates in the FROST protocol.
/// Implemented for all DKG and SIGN package types.
pub trait FrostBytes: Sized {
    fn to_bytes(&self) -> Result<Vec<u8>>;
    fn from_bytes(data: &[u8]) -> Result<Self>;
}

// DKG types — all use the frost library's serialize/deserialize methods
impl FrostBytes for round1::SecretPackage {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        self.serialize().map_err(|e| anyhow::anyhow!("{e}"))
    }
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

impl FrostBytes for round2::SecretPackage {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        self.serialize().map_err(|e| anyhow::anyhow!("{e}"))
    }
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

impl FrostBytes for round1::Package {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        self.serialize().map_err(|e| anyhow::anyhow!("{e}"))
    }
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

impl FrostBytes for round2::Package {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        self.serialize().map_err(|e| anyhow::anyhow!("{e}"))
    }
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

impl FrostBytes for KeyPackage<P> {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        self.serialize().map_err(|e| anyhow::anyhow!("{e}"))
    }
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

impl FrostBytes for PublicKeyPackage<P> {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        self.serialize().map_err(|e| anyhow::anyhow!("{e}"))
    }
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

// SIGN types
impl FrostBytes for SigningNonces<P> {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        self.serialize().map_err(|e| anyhow::anyhow!("{e}"))
    }
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

impl FrostBytes for SigningCommitments<P> {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        self.serialize().map_err(|e| anyhow::anyhow!("{e}"))
    }
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

impl FrostBytes for SigningPackage<P> {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        self.serialize().map_err(|e| anyhow::anyhow!("{e}"))
    }
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

impl FrostBytes for SignatureShare<P> {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.serialize())
    }
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

impl FrostBytes for Randomizer {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.serialize().to_vec())
    }
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::deserialize(
            data.try_into()
                .map_err(|_| anyhow::anyhow!("bad randomizer length"))?,
        )
        .map_err(|e| anyhow::anyhow!("{e}"))
    }
}

impl<T: FrostBytes + bincode::Encode + bincode::Decode<()>> FrostBytes for Vec<T> {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::encode_to_vec(self, config::legacy()).map_err(|e| anyhow::anyhow!("{e}"))
    }
    fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::decode_from_slice(data, config::legacy())
            .map(|(v, _)| v)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}

/// One indexed slot — used by SIGN rounds to carry one per-action package.
/// A vec of these is the `Public` type for SIGN rounds (Option A for idx).
#[derive(Encode, Decode)]
pub struct Indexed {
    pub idx: u32,
    pub data: Vec<u8>, // inner package serialized via FrostBytes
}

impl Indexed {
    pub fn new<T: FrostBytes>(idx: u32, value: &T) -> Result<Self> {
        Ok(Self {
            idx,
            data: value.to_bytes()?,
        })
    }
    pub fn decode<T: FrostBytes>(&self) -> Result<T> {
        T::from_bytes(&self.data)
    }
}

/// A vec of `Indexed` is itself `FrostBytes` — the whole bundle is the unit
/// exchanged between participants in one SIGN round message.
impl FrostBytes for Vec<Indexed> {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::encode_to_vec(self, config::legacy()).map_err(|e| anyhow::anyhow!("{e}"))
    }
    fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::decode_from_slice(data, config::legacy())
            .map(|(v, _)| v)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}

// ── Dispatch ─────────────────────────────────────────────────────────────────

/// Addresses needed to route outgoing packages.
pub struct RouteCtx {
    /// Shared broadcast address (all participants monitor this).
    pub broadcast_address: String,
    /// Coordinator's private mailbox address.
    pub coordinator_address: String,
    /// Per-participant mailbox addresses, indexed by `from_id - 1`.
    pub peer_addresses: Vec<String>,
}

/// Encodes both the content (`P` vs `BTreeMap<Identifier, P>`) and the
/// routing strategy. The wrapper type alone determines the fan-out pattern.
pub trait Dispatch {
    type Public: FrostBytes;
    /// Produce `(address, serialized_bytes)` pairs ready for `publish()`.
    /// The engine will wrap each payload in the appropriate wire message.
    fn into_recipients(self, ctx: &RouteCtx) -> Result<Vec<(String, Vec<u8>)>>;
}

/// One package broadcast to the shared address (same bytes for every peer).
pub struct Broadcast<P>(pub P);
/// One package sent directly to the coordinator's mailbox.
pub struct ToCoordinator<P>(pub P);
/// One package per peer, each routed to that peer's private mailbox.
/// Keyed by `from_id` (1-based participant index), matching the DB column.
pub struct PerPeer<P>(pub BTreeMap<u8, P>);
/// Nothing is sent (local-only computation, e.g. DKG part3).
pub struct NoSend;

/// Multiple packages sent to the coordinator (e.g., commitments for multiple inputs).
pub struct ToCoordinatorMultiple<P>(pub Vec<P>);

impl<P: FrostBytes> Dispatch for Broadcast<P> {
    type Public = P;
    fn into_recipients(self, ctx: &RouteCtx) -> Result<Vec<(String, Vec<u8>)>> {
        Ok(vec![(ctx.broadcast_address.clone(), self.0.to_bytes()?)])
    }
}

impl<P: FrostBytes> Dispatch for ToCoordinator<P> {
    type Public = P;
    fn into_recipients(self, ctx: &RouteCtx) -> Result<Vec<(String, Vec<u8>)>> {
        Ok(vec![(ctx.coordinator_address.clone(), self.0.to_bytes()?)])
    }
}

impl<P: FrostBytes> Dispatch for PerPeer<P> {
    type Public = P;
    fn into_recipients(self, ctx: &RouteCtx) -> Result<Vec<(String, Vec<u8>)>> {
        self.0
            .into_iter()
            .map(|(from_id, pkg)| {
                let idx = from_id as usize - 1;
                let addr = ctx
                    .peer_addresses
                    .get(idx)
                    .ok_or_else(|| anyhow::anyhow!("no address for participant {}", from_id))?;
                Ok((addr.clone(), pkg.to_bytes()?))
            })
            .collect()
    }
}

impl Dispatch for NoSend {
    type Public = ();
    fn into_recipients(self, _ctx: &RouteCtx) -> Result<Vec<(String, Vec<u8>)>> {
        Ok(vec![])
    }
}

impl<P: FrostBytes + bincode::Encode + bincode::Decode<()>> Dispatch for ToCoordinatorMultiple<P> {
    type Public = Vec<P>;
    fn into_recipients(self, ctx: &RouteCtx) -> Result<Vec<(String, Vec<u8>)>> {
        let data = bincode::encode_to_vec(&self.0, config::legacy())?;
        Ok(vec![(ctx.coordinator_address.clone(), data)])
    }
}

impl FrostBytes for () {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(vec![])
    }
    fn from_bytes(_: &[u8]) -> Result<Self> {
        Ok(())
    }
}

// ── Round ─────────────────────────────────────────────────────────────────────

/// One round of a FROST protocol (DKG or SIGN).
///
/// The type chain enforces that rounds are composed correctly at compile time:
#[allow(async_fn_in_trait)]
/// `Round2::Input = Round1::Output`, `Round3::Input = Round2::Output`, etc.
///
/// - `Secret` is stored locally and never transmitted.
/// - `Outgoing` (a `Dispatch` impl) carries the public package(s) to the network.
/// - `Public` (= `Outgoing::Public`) is what peers send *us* — decoded from memos.
pub trait Round {
    /// Output of the previous round (or a seed struct for the first round).
    type Input;
    /// Input to the next round, assembled by `collect`.
    type Output;

    /// Locally stored secret for this round; never transmitted.
    type Secret: FrostBytes;
    /// Dispatch wrapper — encodes both the public payload type and its routing.
    /// Must satisfy `Outgoing::Public = Self::Public`.
    type Outgoing: Dispatch<Public = Self::Public>;
    /// Public package type received from peers (same underlying type as outgoing).
    type Public: FrostBytes;

    /// 4-byte memo prefix that identifies messages for this round.
    const PREFIX: [u8; 4];

    /// Pure: given the previous round's output, compute our secret and
    /// the outgoing package(s).
    fn produce(input: &Self::Input) -> Result<(Self::Secret, Self::Outgoing)>;

    /// Pure: once `threshold` peer packages are available, assemble the output
    /// for the next round. Receives `input` so earlier round data can be
    /// threaded through without re-fetching from the DB.
    fn collect(
        input: Self::Input,
        secret: Self::Secret,
        peers: Vec<(u8, Self::Public)>,
    ) -> Result<Self::Output>;

    // ── DB operations (each impl maps to concrete columns / tables) ──────────

    /// Load our own secret for this round, if already computed.
    async fn load_secret(conn: &mut SqliteConnection, account: u32)
        -> Result<Option<Self::Secret>>;

    /// Persist our secret for this round.
    async fn store_secret(
        conn: &mut SqliteConnection,
        account: u32,
        secret: &Self::Secret,
    ) -> Result<()>;

    /// Persist a public package received from `from_id`.
    async fn store_public(
        conn: &mut SqliteConnection,
        account: u32,
        from_id: u8,
        public: &Self::Public,
    ) -> Result<()>;

    /// Load all received public packages for this round.
    async fn load_publics(
        conn: &mut SqliteConnection,
        account: u32,
    ) -> Result<Vec<(u8, Self::Public)>>;
}

// ── Memo ingest ──────────────────────────────────────────────────────────────

/// Decode memos from `mailbox_account`, filter by `R::PREFIX`, deserialize
/// each as a `FrostMessage`, and store the public package via `R::store_public`.
///
/// Idempotent: `dkg_peers`' primary key `(account, round, from_id)` dedups,
/// so re-ingesting the same memos is a no-op.
pub async fn decode_dkg_memos<R: Round>(
    conn: &mut SqliteConnection,
    account: u32,
    mailbox_account: u32,
) -> Result<()> {
    let (participant_count,) =
        sqlx::query_as::<_, (u8,)>("SELECT n FROM dkg_params WHERE account = ?")
            .bind(account)
            .fetch_one(&mut *conn)
            .await?;
    let pkgs = sqlx::query("SELECT memo_bytes FROM memos WHERE account = ?")
        .bind(mailbox_account)
        .map(|row: SqliteRow| {
            let memo_bytes: Vec<u8> = row.get(0);
            if let Ok(Memo::Arbitrary(pkg_bytes)) = Memo::from_bytes(&memo_bytes) {
                if pkg_bytes.len() < 4 || pkg_bytes[0..4] != R::PREFIX {
                    return None;
                }
                bincode::decode_from_slice::<FrostMessage, _>(&pkg_bytes[4..], config::legacy())
                    .ok()
                    .map(|(msg, _)| msg)
            } else {
                None
            }
        })
        .fetch_all(&mut *conn)
        .await?;

    for msg in pkgs.into_iter().flatten() {
        if !(1..=participant_count).contains(&msg.from_id) {
            info!(
                "ignoring DKG memo with invalid participant id {}",
                msg.from_id
            );
            continue;
        }
        if let Ok(public) = R::Public::from_bytes(&msg.data) {
            R::store_public(conn, account, msg.from_id, &public).await?;
        }
    }
    Ok(())
}

/// Wire message used during DKG rounds.
#[derive(Encode, Decode)]
pub struct FrostMessage {
    pub from_id: u8,
    pub data: Vec<u8>,
}

impl FrostMessage {
    pub fn encode_with_prefix(&self, prefix: &[u8]) -> Result<Vec<u8>> {
        let mut data = vec![];
        data.extend_from_slice(prefix);
        bincode::encode_into_std_write(self, &mut data, config::legacy())?;
        Ok(data)
    }
}

/// Wire message used during the signing rounds.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct FrostSigMessage {
    pub sighash: [u8; 32],
    pub from_id: u16,
    pub idx: u32,
    pub data: Vec<u8>,
    /// Optional ed25519 signature by the sender's round-0 message key. If
    /// present it covers every other field, the routing ones included. A
    /// member that has no round-0 key (a dkg-0 group, a restored member)
    /// sends unsigned messages, which are accepted.
    pub signature: Option<[u8; 64]>,
    /// On commitments: the sender's FROST verifying share, so that the
    /// coordinator can rebuild the public key package without having
    /// stored one. The coordinator's mailbox address is not a field here:
    /// it is the session's first message (`data` under the CRD1 prefix),
    /// since members need it before they can send commitments.
    pub verifying_share: Option<Vec<u8>>,
}

impl FrostSigMessage {
    pub fn encode_with_prefix(&self, prefix: &[u8]) -> Result<Vec<u8>> {
        let mut data = vec![];
        data.extend_from_slice(prefix);
        bincode::encode_into_std_write(self, &mut data, config::legacy())?;
        Ok(data)
    }

    /// What the signature covers: the message's own wire encoding with the
    /// signature cleared. Every variable-length field is length-prefixed
    /// and every optional one is tagged, so no two messages share a
    /// payload; a bare concatenation would let bytes move between `data`
    /// and the verifying share under the same signature.
    fn signed_payload(&self) -> Result<Vec<u8>> {
        let unsigned = Self {
            signature: None,
            ..self.clone()
        };
        Ok(bincode::encode_to_vec(&unsigned, config::legacy())?)
    }

    pub fn sign(&mut self, signing_key: &SigningKey) -> Result<()> {
        let signature = Signer::sign(signing_key, &self.signed_payload()?);
        self.signature = Some(signature.to_bytes());
        Ok(())
    }

    /// True when the message is unsigned, or signed by `verifying_key`.
    pub fn verify(&self, verifying_key: &VerifyingKey) -> bool {
        let Some(signature) = self.signature else {
            return true; // no round-0 key on the sender's side
        };
        let Ok(payload) = self.signed_payload() else {
            return false;
        };
        verifying_key
            .verify_strict(&payload, &Signature::from_bytes(&signature))
            .is_ok()
    }
}

#[cfg(test)]
mod sig_message_tests {
    use super::*;
    use rand_core_legacy::OsRng;

    #[test]
    fn the_signature_covers_the_verifying_share() {
        let key = SigningKey::generate(&mut OsRng);
        let mut msg = FrostSigMessage {
            sighash: [7u8; 32],
            from_id: 2,
            idx: 1,
            data: vec![1, 2, 3],
            signature: None,
            verifying_share: Some(vec![4, 5, 6]),
        };
        assert!(msg.verify(&key.verifying_key()), "unsigned is accepted");
        msg.sign(&key).unwrap();
        assert!(msg.verify(&key.verifying_key()));

        let bytes = msg.encode_with_prefix(b"CMT6").unwrap();
        let (decoded, _) =
            bincode::decode_from_slice::<FrostSigMessage, _>(&bytes[4..], config::legacy())
                .unwrap();
        assert_eq!(decoded, msg);

        let mut forged = msg.clone();
        forged.data = vec![3, 2, 1];
        assert!(!forged.verify(&key.verifying_key()));
        let mut forged = msg.clone();
        forged.verifying_share = Some(vec![6, 5, 4]);
        assert!(!forged.verify(&key.verifying_key()));
        assert!(!msg.verify(&SigningKey::generate(&mut OsRng).verifying_key()));
    }

    /// Field boundaries are part of what is signed: the same bytes split
    /// differently between fields, or an absent field made present and
    /// empty, is a different message.
    #[test]
    fn moving_bytes_between_fields_breaks_the_signature() {
        let key = SigningKey::generate(&mut OsRng);
        let mut msg = FrostSigMessage {
            sighash: [7u8; 32],
            from_id: 2,
            idx: 1,
            data: vec![1, 2, 3, 4, 5, 6],
            signature: None,
            verifying_share: None,
        };
        msg.sign(&key).unwrap();

        let mut shifted = msg.clone();
        shifted.data = vec![1, 2, 3];
        shifted.verifying_share = Some(vec![4, 5, 6]);
        assert!(!shifted.verify(&key.verifying_key()));

        let mut empty_share = msg.clone();
        empty_share.verifying_share = Some(vec![]);
        assert!(!empty_share.verify(&key.verifying_key()));
    }
}

/// The broadcast account's Orchard spending key, from the mnemonic the
/// participant addresses hash to: the `sk` of ZIP 2005, shared by every
/// member since the first DKG. ZIP 32 makes it depend on the network.
pub fn broadcast_sk(network: &Network, addresses: &[String]) -> Result<SpendingKey> {
    let mnemonic = Mnemonic::from_str(&broadcast_seed(addresses))?;
    let seed = mnemonic.to_seed("");
    let usk = UnifiedSpendingKey::from_seed(network, &seed, AccountId::ZERO)?;
    Ok(usk.orchard().clone())
}

/// The group's `sk`: stored in `dkg_params` (dkg-2 accounts, restored
/// accounts), or rederived from the stored addresses and cached there.
pub async fn group_sk(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<SpendingKey> {
    let (stored, n): (Option<Vec<u8>>, u8) =
        sqlx::query_as("SELECT sk, n FROM dkg_params WHERE account = ?")
            .bind(account)
            .fetch_one(&mut *connection)
            .await
            .context("Fetch dkg_params")?;
    if let Some(sk) = stored {
        let sk: [u8; 32] = sk
            .try_into()
            .map_err(|_| anyhow::anyhow!("stored sk is not 32 bytes"))?;
        return Option::from(SpendingKey::from_bytes(sk)).context("stored sk is invalid");
    }
    let addresses = get_addresses(&mut *connection, account, n).await?;
    anyhow::ensure!(
        addresses.iter().all(|a| !a.is_empty()),
        "the group spending key is not stored and the participant addresses are incomplete"
    );
    let sk = broadcast_sk(network, &addresses)?;
    sqlx::query("UPDATE dkg_params SET sk = ?1 WHERE account = ?2")
        .bind(sk.to_bytes().to_vec())
        .bind(account)
        .execute(&mut *connection)
        .await?;
    Ok(sk)
}

/// A unified address carrying only the Orchard receiver at index 0: the
/// form every frost helper address takes on chain.
pub fn orchard_only_address(network: &Network, fvk: &FullViewingKey) -> String {
    let address = fvk.address_at(0u64, Scope::External);
    UnifiedAddress::from_receivers(Some(address), None, None, None, None)
        .unwrap()
        .encode(network)
}

/// The broadcast account built from `sk`, without creating it: a view-only
/// internal account whose Orchard key is `sk`'s. It is the account the
/// address hash produces at DKG time, so the on-chain address is unchanged.
pub async fn lookup_broadcast_account_from_sk(
    network: &Network,
    connection: &mut SqliteConnection,
    sk: &SpendingKey,
) -> Result<Option<(u32, String)>> {
    let fvk = FullViewingKey::from(sk);
    let r: Option<(u32,)> = sqlx::query_as(
        "SELECT a.id_account FROM accounts a
        JOIN orchard_accounts o ON a.id_account = o.account
        WHERE o.xvk = ?1 AND a.internal = 1",
    )
    .bind(fvk.to_bytes().to_vec())
    .fetch_optional(&mut *connection)
    .await?;
    Ok(r.map(|(id,)| (id, orchard_only_address(network, &fvk))))
}

/// The broadcast account built from `sk`, created if missing (see
/// [`lookup_broadcast_account_from_sk`]).
pub async fn ensure_broadcast_account_from_sk(
    network: &Network,
    connection: &mut SqliteConnection,
    sk: &SpendingKey,
    birth_height: u32,
) -> Result<(u32, String)> {
    if let Some(found) = lookup_broadcast_account_from_sk(network, connection, sk).await? {
        return Ok(found);
    }
    let fvk = FullViewingKey::from(sk);
    let mut tx = connection.begin().await?;
    let account =
        store_account_metadata(&mut *tx, "frost-broadcast", &None, &None, birth_height, false, true)
            .await?;
    init_account_orchard(network, &mut *tx, account, birth_height).await?;
    store_account_orchard_vk(&mut *tx, account, &fvk).await?;
    tx.commit().await?;
    let address = orchard_only_address(network, &fvk);
    info!("Broadcast address: {address}");
    Ok((account, address))
}

/// The private mailbox account for signing, from the seed recorded in
/// `dkg_params`; created with a fresh seed when none is recorded (a
/// restored member). Needs no participant addresses. Returns the account
/// and its address.
pub async fn ensure_mailbox_from_seed(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    birth_height: u32,
) -> Result<(u32, String)> {
    let (seed,): (String,) = sqlx::query_as("SELECT seed FROM dkg_params WHERE account = ?1")
        .bind(account)
        .fetch_one(&mut *connection)
        .await
        .context("Fetch dkg_params")?;
    let existing: Option<(u32,)> = if seed.is_empty() {
        None
    } else {
        sqlx::query_as("SELECT id_account FROM accounts WHERE seed = ?1")
            .bind(&seed)
            .fetch_optional(&mut *connection)
            .await?
    };
    let mailbox = match existing {
        Some((mailbox,)) => mailbox,
        None => {
            info!("Creating mailbox account");
            let na = NewAccount {
                name: "frost-mailbox".to_string(),
                icon: None,
                restore: true,
                key: seed.clone(),
                passphrase: None,
                fingerprint: None,
                aindex: 0,
                birth: Some(birth_height),
                folder: "".to_string(),
                pools: None,
                use_internal: false,
                internal: true,
                hw: 0,
            };
            let mailbox = new_account(network, &mut *connection, &na).await?;
            if seed.is_empty() {
                let seed = get_account_seed(&mut *connection, mailbox)
                    .await?
                    .context("mailbox seed not found")?;
                sqlx::query("UPDATE dkg_params SET seed = ?1 WHERE account = ?2")
                    .bind(&seed.mnemonic)
                    .bind(account)
                    .execute(&mut *connection)
                    .await?;
            }
            mailbox
        }
    };
    let address = mailbox_address(network, connection, mailbox).await?;
    Ok((mailbox, address))
}

/// The Orchard-only unified address of a helper account.
pub async fn mailbox_address(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<String> {
    let fvk = get_orchard_vk(&mut *connection, account)
        .await?
        .context("mailbox account has no Orchard key")?;
    Ok(orchard_only_address(network, &fvk))
}

/// How many bytes an arbitrary memo carries: the 512-byte memo field less
/// the 0xFF marker `to_arb_memo` prepends. Every protocol message, prefix
/// included, has to fit in this.
pub const ARB_MEMO_CAPACITY: usize = 511;

/// Wrap raw bytes as a Zcash arbitrary memo (0xFF prefix).
pub fn to_arb_memo(data: &[u8]) -> Vec<u8> {
    let mut memo_bytes = vec![0xFF];
    memo_bytes.extend_from_slice(data);
    memo_bytes
}

/// Send a Zcash transaction carrying the given `(address, memo)` recipients,
/// funded from `account`. Returns the transaction ID hex string.
pub async fn publish(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    client: &mut Client,
    recipients: &[(&str, Vec<u8>)],
) -> Result<String, anyhow::Error> {
    let recipients = recipients
        .iter()
        .map(|(address, data)| Recipient {
            address: address.to_string(),
            amount: 0,
            pools: None,
            user_memo: None,
            memo_bytes: Some(to_arb_memo(data)),
            price: None,
            asset_base: [0u8; 32].to_vec(),
            asset_name: None,
        })
        .collect::<Vec<_>>();
    let request = PaymentRequest {
        recipients,
        src_pools: ALL_SHIELDED_POOLS,
        ..Default::default()
    };
    let pczt = plan_transaction(
        network,
        connection,
        client,
        account,
        &request,
        &PlanMode::Standard,
    )
    .await
    .context("plan_transaction in DKG publish")?;
    let pczt = sign(network, connection, account, pczt, &())
        .await
        .context("sign_transaction in DKG publish")?;
    // Broadcasting locks the funding notes this transaction spent so the
    // next round does not re-select them before the spend is mined.
    crate::pay::broadcast(connection, client, account, &pczt)
        .await
        .context("broadcast in DKG publish")
}

/// Get (and create if needed) the private mailbox address for
/// communication between all participants in the group
///
pub async fn get_mailbox_account(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    self_id: u8,
    birth_height: u32,
) -> Result<(u32, String)> {
    let mut retry = 0;
    let (account, mailbox_address) = loop {
        if retry > 1 {
            anyhow::bail!("Failed to create mailbox account");
        }

        // seed or empty string if not set
        let seed = sqlx::query_as::<_, (String,)>("SELECT seed FROM dkg_params WHERE account = ?1")
            .bind(account)
            .fetch_optional(&mut *connection)
            .await?
            .map(|row| row.0)
            .unwrap_or_default();

        let address = sqlx::query_as::<_, (String,)>(
            "SELECT address FROM dkg_addresses WHERE account = ? AND from_id = ?",
        )
        .bind(account)
        .bind(self_id)
        .fetch_optional(&mut *connection)
        .await?
        .map(|a| a.0);
        let mailbox_account = if !seed.is_empty() {
            sqlx::query_as::<_, (u32,)>("SELECT id_account FROM accounts WHERE seed = ?1")
                .bind(&seed)
                .fetch_optional(&mut *connection)
                .await?
        } else {
            None
        };

        match (address, mailbox_account) {
            (Some(mailbox_address), Some((mailbox_account,))) => {
                break (mailbox_account, mailbox_address);
            }
            (_, None) => {
                info!("Creating mailbox account");
                // The account does not exist, create it with a random seed
                let na = NewAccount {
                    name: "frost-mailbox".to_string(),
                    icon: None,
                    restore: true,
                    key: seed.clone(),
                    passphrase: None,
                    fingerprint: None,
                    aindex: 0,
                    birth: Some(birth_height),
                    folder: "".to_string(),
                    pools: None,
                    use_internal: false,
                    internal: true,
                    hw: 0,
                };
                let mailbox_account = new_account(network, &mut *connection, &na).await?;
                let fvk = get_orchard_vk(&mut *connection, mailbox_account)
                    .await?
                    .expect("Mailbox account should have orchard");
                let address = fvk.address_at(0u64, Scope::External);
                let ua =
                    UnifiedAddress::from_receivers(Some(address), None, None, None, None).unwrap();
                let ua = ua.encode(network);
                sqlx::query(
                    "INSERT INTO dkg_addresses (account, from_id, address)
                    VALUES (?1, ?2, ?3) ON CONFLICT DO NOTHING",
                )
                .bind(account)
                .bind(self_id)
                .bind(ua)
                .execute(&mut *connection)
                .await?;
                let seed = get_account_seed(&mut *connection, mailbox_account)
                    .await?
                    .expect("Seed should be set");
                sqlx::query("UPDATE dkg_params SET seed = ?1 WHERE account = ?2")
                    .bind(&seed.mnemonic)
                    .bind(account)
                    .execute(&mut *connection)
                    .await?;
            }
            _ => unreachable!(),
        }
        retry += 1;
    };

    Ok((account, mailbox_address))
}

/// Seed of the shared broadcast account: a deterministic function of the
/// participant addresses, so every participant derives the same account.
fn broadcast_seed(addresses: &[String]) -> String {
    let mut state = blake2b_simd::Params::new()
        .hash_length(32)
        .personal(b"Zcash__FROST_DKG")
        .to_state();
    for a in addresses.iter() {
        state.update(a.as_bytes());
    }
    let hash = state.finalize();
    let m = Mnemonic::from_entropy(hash.as_ref()).expect("Failed to create mnemonic from hash");
    m.to_string()
}

/// Look up the shared broadcast account without creating it. Returns the
/// account id and its encoded unified address, or `None` when the account
/// does not exist or any participant address is still missing.
pub async fn lookup_broadcast_account(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<(u32, String)>> {
    let addresses = sqlx::query_as::<_, (String,)>(
        "SELECT address FROM dkg_addresses WHERE account = ?1 ORDER BY from_id",
    )
    .bind(account)
    .fetch_all(&mut *connection)
    .await?;
    let addresses = addresses.into_iter().map(|row| row.0).collect::<Vec<_>>();
    if addresses.iter().any(|a| a.is_empty()) {
        return Ok(None);
    }
    let seed = broadcast_seed(&addresses);

    let r = sqlx::query_as::<_, (u32, Vec<u8>)>(
        "SELECT a.id_account, o.xvk FROM accounts a
        JOIN orchard_accounts o ON a.id_account = o.account
        WHERE seed = ?1",
    )
    .bind(&seed)
    .fetch_optional(&mut *connection)
    .await?;
    match r {
        None => Ok(None),
        Some((account, xvk)) => {
            let fvk = FullViewingKey::from_bytes(&xvk.try_into().unwrap())
                .expect("Failed to create shared FVK");
            let address = fvk.address_at(0u64, Scope::External);
            let ua = UnifiedAddress::from_receivers(Some(address), None, None, None, None).unwrap();
            Ok(Some((account, ua.encode(network))))
        }
    }
}

/// Get (and create if needed) the shared broadcast address for
/// communication between all participants in the group.
/// It is derived from the hash of the participant private mailbox addresses.
///
pub async fn get_coordinator_broadcast_account(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<(u32, String)> {
    let addresses = sqlx::query_as::<_, (String,)>(
        "SELECT address FROM dkg_addresses WHERE account = ?1 ORDER BY from_id",
    )
    .bind(account)
    .fetch_all(&mut *connection)
    .await?;
    let addresses = addresses.into_iter().map(|row| row.0).collect::<Vec<_>>();
    let seed = broadcast_seed(&addresses);

    let (account, broadcast_address) = loop {
        // Check if the account already exists
        let r = sqlx::query_as::<_, (u32, Vec<u8>)>(
            "SELECT a.id_account, o.xvk FROM accounts a
            JOIN orchard_accounts o ON a.id_account = o.account
            WHERE seed = ?1",
        )
        .bind(&seed)
        .fetch_optional(&mut *connection)
        .await?;

        match r {
            None => {
                // The account does not exist, create it
                let na = NewAccount {
                    name: "frost-broadcast".to_string(),
                    icon: None,
                    restore: true,
                    key: seed.to_string(),
                    passphrase: None,
                    fingerprint: None,
                    aindex: 0,
                    birth: Some(height),
                    folder: "".to_string(),
                    pools: None,
                    use_internal: false,
                    internal: true,
                    hw: 0,
                };
                new_account(network, &mut *connection, &na).await?;
                // Loop again to retrieve the account
            }
            Some((account, xvk)) => {
                let fvk = FullViewingKey::from_bytes(&xvk.try_into().unwrap())
                    .expect("Failed to create shared FVK");
                let address = fvk.address_at(0u64, Scope::External);
                let ua =
                    UnifiedAddress::from_receivers(Some(address), None, None, None, None).unwrap();
                let broadcast_address = ua.encode(network);
                info!("Broadcast address: {broadcast_address}");
                break (account, broadcast_address);
            }
        }
    };

    Ok((account, broadcast_address))
}

pub async fn get_addresses(
    connection: &mut SqliteConnection,
    account: u32,
    n: u8,
) -> Result<Vec<String>> {
    let mut addresses = vec![String::new(); n as usize];
    let mut rs = sqlx::query_as::<_, (u8, String)>(
        "SELECT from_id, address FROM dkg_addresses WHERE account = ?1",
    )
    .bind(account)
    .fetch(connection);
    while let Some((from_id, address)) = rs.try_next().await? {
        addresses[(from_id - 1) as usize] = address;
    }

    Ok(addresses)
}
