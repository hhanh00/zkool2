use std::collections::BTreeMap;

use anyhow::Result;
use bincode::{config, Decode, Encode};
use bip39::Mnemonic;
use futures::TryStreamExt;
use orchard::keys::{FullViewingKey, Scope};
use reddsa::frost::redpallas::{
    frost::{
        keys::{KeyPackage, PublicKeyPackage},
        round1::{SigningCommitments, SigningNonces},
        round2::SignatureShare,
        SigningPackage,
    },
    keys::dkg::{round1, round2},
    Identifier, PallasBlake2b512, Randomizer,
};
use sqlx::{sqlite::SqliteRow, Connection, Row, SqliteConnection};
use tracing::info;
use zcash_keys::address::UnifiedAddress;
use zcash_protocol::memo::Memo;

use crate::{
    account::{get_account_seed, get_orchard_vk, new_account},
    api::{account::NewAccount, coin::Network},
    pay::{
        plan::{extract_transaction, plan_transaction, sign_transaction},
        pool::ALL_POOLS,
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
    fn to_bytes(&self) -> Result<Vec<u8>> { self.serialize().map_err(|e| anyhow::anyhow!("{e}")) }
    fn from_bytes(data: &[u8]) -> Result<Self> { Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}")) }
}

impl FrostBytes for round2::SecretPackage {
    fn to_bytes(&self) -> Result<Vec<u8>> { self.serialize().map_err(|e| anyhow::anyhow!("{e}")) }
    fn from_bytes(data: &[u8]) -> Result<Self> { Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}")) }
}

impl FrostBytes for round1::Package {
    fn to_bytes(&self) -> Result<Vec<u8>> { self.serialize().map_err(|e| anyhow::anyhow!("{e}")) }
    fn from_bytes(data: &[u8]) -> Result<Self> { Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}")) }
}

impl FrostBytes for round2::Package {
    fn to_bytes(&self) -> Result<Vec<u8>> { self.serialize().map_err(|e| anyhow::anyhow!("{e}")) }
    fn from_bytes(data: &[u8]) -> Result<Self> { Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}")) }
}

impl FrostBytes for KeyPackage<P> {
    fn to_bytes(&self) -> Result<Vec<u8>> { self.serialize().map_err(|e| anyhow::anyhow!("{e}")) }
    fn from_bytes(data: &[u8]) -> Result<Self> { Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}")) }
}

impl FrostBytes for PublicKeyPackage<P> {
    fn to_bytes(&self) -> Result<Vec<u8>> { self.serialize().map_err(|e| anyhow::anyhow!("{e}")) }
    fn from_bytes(data: &[u8]) -> Result<Self> { Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}")) }
}

// SIGN types
impl FrostBytes for SigningNonces<P> {
    fn to_bytes(&self) -> Result<Vec<u8>> { self.serialize().map_err(|e| anyhow::anyhow!("{e}")) }
    fn from_bytes(data: &[u8]) -> Result<Self> { Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}")) }
}

impl FrostBytes for SigningCommitments<P> {
    fn to_bytes(&self) -> Result<Vec<u8>> { self.serialize().map_err(|e| anyhow::anyhow!("{e}")) }
    fn from_bytes(data: &[u8]) -> Result<Self> { Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}")) }
}

impl FrostBytes for SigningPackage<P> {
    fn to_bytes(&self) -> Result<Vec<u8>> { self.serialize().map_err(|e| anyhow::anyhow!("{e}")) }
    fn from_bytes(data: &[u8]) -> Result<Self> { Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}")) }
}

impl FrostBytes for SignatureShare<P> {
    fn to_bytes(&self) -> Result<Vec<u8>> { Ok(self.serialize()) }
    fn from_bytes(data: &[u8]) -> Result<Self> { Self::deserialize(data).map_err(|e| anyhow::anyhow!("{e}")) }
}

impl FrostBytes for Randomizer {
    fn to_bytes(&self) -> Result<Vec<u8>> { Ok(self.serialize().to_vec()) }
    fn from_bytes(data: &[u8]) -> Result<Self> { Self::deserialize(data.try_into().map_err(|_| anyhow::anyhow!("bad randomizer length"))?).map_err(|e| anyhow::anyhow!("{e}")) }
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
        Ok(Self { idx, data: value.to_bytes()? })
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
                let addr = ctx.peer_addresses.get(idx).ok_or_else(|| {
                    anyhow::anyhow!("no address for participant {}", from_id)
                })?;
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

impl FrostBytes for () {
    fn to_bytes(&self) -> Result<Vec<u8>> { Ok(vec![]) }
    fn from_bytes(_: &[u8]) -> Result<Self> { Ok(()) }
}

// ── Round ─────────────────────────────────────────────────────────────────────

/// One round of a FROST protocol (DKG or SIGN).
///
/// The type chain enforces that rounds are composed correctly at compile time:
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

    /// Minimum number of peer packages required to advance.
    fn threshold(n: u8, t: u8) -> usize;

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
    async fn load_secret(
        conn: &mut SqliteConnection,
        account: u32,
    ) -> Result<Option<Self::Secret>>;

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

// ── Generic round engine ─────────────────────────────────────────────────────

/// Decode memos from `mailbox_account`, filter by `R::PREFIX`, deserialize
/// each as a `FrostMessage`, and store the public package via `R::store_public`.
async fn decode_dkg_memos<R: Round>(
    conn: &mut SqliteConnection,
    account: u32,
    mailbox_account: u32,
) -> Result<()> {
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
        if let Ok(public) = R::Public::from_bytes(&msg.data) {
            R::store_public(conn, account, msg.from_id, &public).await?;
        }
    }
    Ok(())
}

/// Drive one round of a FROST protocol.
///
/// 1. Decodes incoming memos from `mailbox_account` and stores peer packages.
/// 2. If our own secret is not yet produced, calls `R::produce`, stores the
///    secret, and publishes outgoing packages to the network (in a DB tx).
/// 3. Checks whether enough peer packages have arrived (`R::threshold`).
/// 4. If ready, assembles and returns `R::Output` via `R::collect`.
///    Returns `None` when still waiting for peer messages.
#[allow(clippy::too_many_arguments)]
pub async fn run_round<R: Round>(
    conn: &mut SqliteConnection,
    account: u32,
    n: u8,
    t: u8,
    self_id: u8,
    funding_account: u32,
    mailbox_account: u32,
    input: R::Input,
    route_ctx: &RouteCtx,
    network: &Network,
    client: &mut Client,
    height: u32,
) -> Result<Option<R::Output>> {
    // 1. Decode incoming peer packages from memos
    decode_dkg_memos::<R>(conn, account, mailbox_account).await?;

    // 2. Produce our own secret + outgoing packages if not yet done
    if R::load_secret(conn, account).await?.is_none() {
        let (secret, outgoing) = R::produce(&input)?;
        let recipients_raw = outgoing.into_recipients(route_ctx)?;

        let mut tx = conn.begin().await?;
        R::store_secret(&mut *tx, account, &secret).await?;
        if !recipients_raw.is_empty() {
            let recipients: Vec<(String, Vec<u8>)> = recipients_raw
                .into_iter()
                .map(|(addr, data)| {
                    let msg = FrostMessage { from_id: self_id, data };
                    let bytes = msg.encode_with_prefix(&R::PREFIX)?;
                    Ok((addr, bytes))
                })
                .collect::<Result<_>>()?;
            let refs: Vec<(&str, Vec<u8>)> =
                recipients.iter().map(|(a, d)| (a.as_str(), d.clone())).collect();
            publish(network, &mut *tx, funding_account, client, height, &refs).await?;
        }
        tx.commit().await?;
    }

    // 3. Check if enough peer packages have arrived
    let peers = R::load_publics(conn, account).await?;
    if peers.len() < R::threshold(n, t) {
        return Ok(None);
    }

    // 4. Collect and advance to the next round
    let secret = R::load_secret(conn, account).await?.unwrap();
    Ok(Some(R::collect(input, secret, peers)?))
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
#[derive(Encode, Decode)]
pub struct FrostSigMessage {
    pub sighash: [u8; 32],
    pub from_id: u16,
    pub idx: u32,
    pub data: Vec<u8>,
}

impl FrostSigMessage {
    pub fn encode_with_prefix(&self, prefix: &[u8]) -> Result<Vec<u8>> {
        let mut data = vec![];
        data.extend_from_slice(prefix);
        bincode::encode_into_std_write(self, &mut data, config::legacy())?;
        Ok(data)
    }
}

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
    height: u32,
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
        })
        .collect::<Vec<_>>();
    let pczt = plan_transaction(
        network,
        connection,
        client,
        account,
        ALL_POOLS,
        &recipients,
        false,
        None,
        false,
        None,
    )
    .await
    .unwrap();
    let pczt = sign_transaction(connection, account, &pczt).await?;
    let txb = extract_transaction(&pczt).await?;
    let result = crate::pay::send(client, height, &txb).await?;
    hex::decode(&result)?;
    Ok(result)
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
                    ledger: false,
                };
                let mailbox_account = new_account(network, &mut *connection, &na).await?;
                let fvk = get_orchard_vk(&mut *connection, mailbox_account)
                    .await?
                    .expect("Mailbox account should have orchard");
                let address = fvk.address_at(0u64, Scope::External);
                let ua = UnifiedAddress::from_receivers(Some(address), None, None).unwrap();
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

    let mut state = blake2b_simd::Params::new()
        .hash_length(32)
        .personal(b"Zcash__FROST_DKG")
        .to_state();
    for a in addresses.iter() {
        state.update(a.as_bytes());
    }
    let hash = state.finalize();
    let m = Mnemonic::from_entropy(hash.as_ref()).expect("Failed to create mnemonic from hash");
    let seed = m.to_string();

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
                    ledger: false,
                };
                new_account(network, &mut *connection, &na).await?;
                // Loop again to retrieve the account
            }
            Some((account, xvk)) => {
                let fvk = FullViewingKey::from_bytes(&xvk.try_into().unwrap())
                    .expect("Failed to create shared FVK");
                let address = fvk.address_at(0u64, Scope::External);
                let ua = UnifiedAddress::from_receivers(Some(address), None, None).unwrap();
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
