//! A snapshot of everything the DKG needs in order to decide what to do.
//!
//! `observe` is the DKG's single reader of protocol state. Unlike the note
//! migration's observation, it is not purely read-only: it first ingests the
//! peer packages that have arrived in memos since the last step. That ingest
//! is idempotent — `dkg_peers`' primary key `(account, round, from_id)` dedups
//! — and it is what makes peer progress visible: in a multi-party protocol,
//! the state that decides the next task lives partly in other wallets'
//! messages.

use anyhow::{bail, Context, Result};
use bincode::{config, Decode, Encode};
use sqlx::{Row, SqliteConnection};

use crate::api::{coin::Network, frost::DKGParams};
use crate::frost::protocol::{decode_dkg_memos, get_addresses, lookup_broadcast_account};

use super::{DkgRound0, DkgRound1, DkgRound2, DkgInit, DkgState0, DkgState1, DkgState2, Round};

/// Outgoing bytes staged before the broadcast, cleared after it. Routing is
/// frozen at stage time so a retry sends exactly what was planned. The crash
/// window between `send` and clearing the marker only ever costs one
/// duplicate memo transmission — the identical bytes, deduplicated by every
/// peer's primary key — never a secret/package mismatch.
#[derive(Encode, Decode)]
pub struct PendingPublish {
    pub round: u8,
    pub recipients: Vec<(String, Vec<u8>)>,
}

impl PendingPublish {
    pub fn encode(&self) -> Result<Vec<u8>> {
        Ok(bincode::encode_to_vec(self, config::legacy())?)
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        Ok(bincode::decode_from_slice(data, config::legacy())?.0)
    }
}

/// Where one round stands: our secret, the peers we have heard from, and
/// whether an outgoing package is staged but not yet confirmed sent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundState {
    pub secret_present: bool,
    /// Distinct packages from participants other than us.
    pub others: u8,
    /// Round whose outgoing bytes are staged in `dkg_state.pending_publish`.
    pub pending: Option<u8>,
}

/// The wallet's DKG state at one instant, as the planner sees it.
#[derive(Clone)]
pub struct DkgState {
    pub funding_account: u32,
    pub params: DKGParams,
    /// Participant addresses, slot `id - 1` empty when not yet exchanged.
    pub addresses: Vec<String>,
    pub mailbox: Option<u32>,
    pub broadcast: Option<u32>,
    pub broadcast_address: Option<String>,
    pub rounds: [RoundState; 3],
    pub key_pkg_present: bool,
    /// The shared frost account, once created (props `dkg_frost_account`).
    pub frost_account: Option<u32>,
    /// True when `dkg_params` was found under the frost account: the finalize
    /// rekey already happened and only cleanup remains.
    pub rekeyed: bool,
    // Reconstructed round inputs (pure `collect` chains), consumed by exec.
    pub state0: Option<DkgState0>,
    pub state1: Option<DkgState1>,
    pub state2: Option<DkgState2>,
}

impl DkgState {
    pub fn addresses_complete(&self) -> bool {
        self.addresses.len() == self.params.n as usize
            && self.addresses.iter().all(|a| !a.is_empty())
    }

    /// The round with staged-but-unsent outgoing bytes, if any.
    pub fn pending_round(&self) -> Option<u8> {
        self.rounds
            .iter()
            .position(|r| r.pending.is_some())
            .map(|r| r as u8)
    }
}

/// Read protocol state. Peer packages are ingested from memos first; the
/// rest is a handful of queries over the dkg_* tables plus the two helper
/// accounts, and the pure `collect` chain that rebuilds the round inputs.
pub async fn observe(
    network: &Network,
    connection: &mut SqliteConnection,
    funding_account: u32,
) -> Result<DkgState> {
    // Resolve which account's rows we are on: the funding account until the
    // finalize rekey, the frost account after it. props `dkg_frost_account`
    // bridges the gap so a crash inside finalize resumes instead of failing.
    let frost_account: Option<u32> = sqlx::query_as::<_, (String,)>(
        "SELECT value FROM props WHERE key = 'dkg_frost_account'",
    )
    .fetch_optional(&mut *connection)
    .await?
    .and_then(|(v,)| v.parse().ok());

    let (params, params_account, rekeyed) = match get_params_opt(connection, funding_account).await?
    {
        Some(p) => (p, funding_account, false),
        None => match frost_account {
            Some(fa) => match get_params_opt(connection, fa).await? {
                Some(p) => (p, fa, true),
                None => bail!("dkg_params not found for account {funding_account}"),
            },
            None => bail!("dkg_params not found for account {funding_account}"),
        },
    };

    let n = params.n;
    let addresses = get_addresses(connection, params_account, n).await?;
    let addresses_complete = addresses.len() == n as usize && addresses.iter().all(|a| !a.is_empty());

    // Helper accounts, looked up read-only — creating them is an effect
    // (EnsureAccounts), and observe has no side effects beyond memo ingest.
    let mailbox = mailbox_account_id(connection, params_account).await?;
    let broadcast_lookup = if addresses_complete {
        lookup_broadcast_account(network, connection, params_account).await?
    } else {
        None
    };
    let (broadcast, broadcast_address) = match broadcast_lookup {
        Some((id, addr)) => (Some(id), Some(addr)),
        None => (None, None),
    };

    // Ingest arrived peer packages. Rounds 0 and 1 are broadcast to the
    // shared address, round 2 to our private mailbox; ingest whichever
    // accounts exist (a missing account has nothing to read yet).
    if let Some(broadcast_id) = broadcast {
        decode_dkg_memos::<DkgRound0>(connection, params_account, broadcast_id).await?;
        decode_dkg_memos::<DkgRound1>(connection, params_account, broadcast_id).await?;
    }
    if let Some(mailbox_id) = mailbox {
        decode_dkg_memos::<DkgRound2>(connection, params_account, mailbox_id).await?;
    }

    let pending: Option<PendingPublish> =
        sqlx::query_as::<_, (Vec<u8>,)>(
            "SELECT pending_publish FROM dkg_state WHERE account = ? AND pending_publish IS NOT NULL",
        )
        .bind(params_account)
        .fetch_optional(&mut *connection)
        .await?
        .map(|(b,)| PendingPublish::decode(&b))
        .transpose()?;

    let mut rounds = [RoundState { secret_present: false, others: 0, pending: None }; 3];
    for (round, rs) in rounds.iter_mut().enumerate() {
        let round = round as u8;
        rs.secret_present = match round {
            0 => <DkgRound0 as Round>::load_secret(connection, params_account).await?.is_some(),
            1 => <DkgRound1 as Round>::load_secret(connection, params_account).await?.is_some(),
            _ => <DkgRound2 as Round>::load_secret(connection, params_account).await?.is_some(),
        };
        rs.others = sqlx::query_as::<_, (u32,)>(
            "SELECT COUNT(*) FROM dkg_peers WHERE account = ? AND round = ? AND from_id != ?",
        )
        .bind(params_account)
        .bind(round)
        .bind(params.id)
        .fetch_one(&mut *connection)
        .await?
        .0 as u8;
        rs.pending = pending
            .as_ref()
            .filter(|p| p.round == round)
            .map(|p| p.round);
    }

    let key_pkg_present = sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT key_pkg FROM dkg_state WHERE account = ? AND key_pkg IS NOT NULL",
    )
    .bind(params_account)
    .fetch_optional(&mut *connection)
    .await?
    .is_some();

    // Rebuild the round inputs by chaining the pure `collect` functions over
    // stored secrets and peer packages — the same data `do_dkg_impl` used to
    // thread through `run_round` return values, re-derived from the DB each
    // step instead.
    let init = DkgInit {
        self_id: params.id,
        n,
        t: params.t,
    };
    let need_peers = |rs: &RoundState| rs.secret_present && rs.others >= n.saturating_sub(1);
    let state0 = if need_peers(&rounds[0]) {
        let secret = <DkgRound0 as Round>::load_secret(connection, params_account)
            .await?
            .unwrap();
        let peers = <DkgRound0 as Round>::load_publics(connection, params_account).await?;
        Some(DkgRound0::collect(init, secret, peers)?)
    } else {
        None
    };
    let state1 = if let (Some(s0), true) = (&state0, need_peers(&rounds[1])) {
        let secret = <DkgRound1 as Round>::load_secret(connection, params_account)
            .await?
            .unwrap();
        let peers = <DkgRound1 as Round>::load_publics(connection, params_account).await?;
        Some(DkgRound1::collect(s0.clone(), secret, peers)?)
    } else {
        None
    };
    let state2 = if let (Some(s1), true) = (&state1, need_peers(&rounds[2])) {
        let secret = <DkgRound2 as Round>::load_secret(connection, params_account)
            .await?
            .unwrap();
        let peers = <DkgRound2 as Round>::load_publics(connection, params_account).await?;
        Some(DkgRound2::collect(s1.clone(), secret, peers)?)
    } else {
        None
    };

    Ok(DkgState {
        funding_account,
        params,
        addresses,
        mailbox,
        broadcast,
        broadcast_address,
        rounds,
        key_pkg_present,
        frost_account,
        rekeyed,
        state0,
        state1,
        state2,
    })
}

/// The dkg_params row, if present. Unlike [`super::get_dkg_params`], missing
/// rows are a normal outcome: they mean the finalize rekey already moved the
/// row to the frost account.
async fn get_params_opt(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<DKGParams>> {
    sqlx::query("SELECT id, n, t, birth_height FROM dkg_params WHERE account = ?")
        .bind(account)
        .map(|row: sqlx::sqlite::SqliteRow| DKGParams {
            id: row.get(0),
            n: row.get(1),
            t: row.get(2),
            birth_height: row.get(3),
        })
        .fetch_optional(&mut *connection)
        .await
        .context("Fetch dkg_params")
}

/// The private mailbox account, found by the seed stored in dkg_params when
/// it was created. None when the mailbox does not exist yet.
async fn mailbox_account_id(
    connection: &mut SqliteConnection,
    params_account: u32,
) -> Result<Option<u32>> {
    let seed: Option<(String,)> =
        sqlx::query_as("SELECT seed FROM dkg_params WHERE account = ?")
            .bind(params_account)
            .fetch_optional(&mut *connection)
            .await?;
    let Some((seed,)) = seed else {
        return Ok(None);
    };
    if seed.is_empty() {
        return Ok(None);
    }
    let r: Option<(u32,)> = sqlx::query_as("SELECT id_account FROM accounts WHERE seed = ?1")
        .bind(&seed)
        .fetch_optional(&mut *connection)
        .await?;
    Ok(r.map(|(a,)| a))
}
