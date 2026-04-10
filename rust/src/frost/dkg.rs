use std::collections::BTreeMap;

use anyhow::{Context, Result};
use orchard::keys::{FullViewingKey, Scope};
use rand_core::OsRng;
use reddsa::frost::redpallas::{
    frost::keys::{KeyPackage, PublicKeyPackage},
    keys::dkg::{self, round1, round2},
    keys::EvenY,
    Identifier,
};
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};
use tracing::info;
use zcash_keys::address::UnifiedAddress;

use crate::{
    account::{get_account_seed, get_orchard_vk},
    api::{
        coin::Network,
        frost::{get_funding_account, DKGParams, DKGStatus},
        sync::SYNCING,
    },
    db::{delete_account, init_account_orchard, store_account_metadata, store_account_orchard_vk},
    frost::{Broadcast, FrostBytes, PerPeer, Round, RouteCtx},
    Client, Sink,
};

pub use super::protocol::{
    get_addresses, get_coordinator_broadcast_account, get_mailbox_account, publish, run_round,
};

// ── State types ──────────────────────────────────────────────────────────────

/// Seed data for the first DKG round.
pub struct DkgInit {
    pub self_id: u8,
    pub n: u8,
    pub t: u8,
}

/// State after round 1 completes: our secret + all peers' round-1 packages.
pub struct DkgState1 {
    pub init: DkgInit,
    pub spkg1: round1::SecretPackage,
    pub ppkg1s: BTreeMap<Identifier, round1::Package>,
}

/// State after round 2 completes: carries forward everything needed for part3.
pub struct DkgState2 {
    pub state1: DkgState1,
    pub spkg2: round2::SecretPackage,
    pub ppkg2s: BTreeMap<Identifier, round2::Package>,
}

// ── DkgRound1 ────────────────────────────────────────────────────────────────

pub struct DkgRound1;

impl Round for DkgRound1 {
    type Input = DkgInit;
    type Output = DkgState1;
    type Secret = round1::SecretPackage;
    type Outgoing = Broadcast<round1::Package>;
    type Public = round1::Package;

    const PREFIX: [u8; 4] = *b"DK11";

    /// Need all other participants' packages.
    fn threshold(_n: u8, t: u8) -> usize {
        t as usize
    }

    fn produce(input: &DkgInit) -> Result<(round1::SecretPackage, Broadcast<round1::Package>)> {
        info!("DKG: calling dkg::part1 (self_id={}, n={}, t={})", input.self_id, input.n, input.t);
        let (spkg1, ppkg1) = dkg::part1(
            (input.self_id as u16).try_into()?,
            input.n as u16,
            input.t as u16,
            OsRng,
        )?;
        info!("DKG: dkg::part1 completed successfully");
        Ok((spkg1, Broadcast(ppkg1)))
    }

    fn collect(
        input: DkgInit,
        spkg1: round1::SecretPackage,
        peers: Vec<(u8, round1::Package)>,
    ) -> Result<DkgState1> {
        let ppkg1s = peers
            .into_iter()
            .filter(|(id, _)| *id != input.self_id)  // Skip our own package
            .map(|(id, pkg)| Ok(((id as u16).try_into()?, pkg)))
            .collect::<Result<_>>()?;
        Ok(DkgState1 { init: input, spkg1, ppkg1s })
    }

    async fn load_secret(conn: &mut SqliteConnection, account: u32) -> Result<Option<round1::SecretPackage>> {
        sqlx::query_as::<_, (Vec<u8>,)>("SELECT spkg1 FROM dkg_state WHERE account = ? AND spkg1 IS NOT NULL")
            .bind(account)
            .fetch_optional(&mut *conn)
            .await?
            .map(|(b,)| round1::SecretPackage::from_bytes(&b))
            .transpose()
    }

    async fn store_secret(conn: &mut SqliteConnection, account: u32, s: &round1::SecretPackage) -> Result<()> {
        sqlx::query(
            "INSERT INTO dkg_state(account, spkg1) VALUES(?1, ?2)
            ON CONFLICT(account) DO UPDATE SET spkg1 = excluded.spkg1",
        )
        .bind(account)
        .bind(s.to_bytes()?)
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    async fn store_public(conn: &mut SqliteConnection, account: u32, from_id: u8, p: &round1::Package) -> Result<()> {
        sqlx::query(
            "INSERT INTO dkg_peers(account, round, from_id, data) VALUES(?1, 1, ?2, ?3)
            ON CONFLICT DO NOTHING",
        )
        .bind(account)
        .bind(from_id)
        .bind(p.to_bytes()?)
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    async fn load_publics(conn: &mut SqliteConnection, account: u32) -> Result<Vec<(u8, round1::Package)>> {
        sqlx::query("SELECT from_id, data FROM dkg_peers WHERE account = ? AND round = 1")
            .bind(account)
            .map(|row: SqliteRow| (row.get::<u8, _>(0), row.get::<Vec<u8>, _>(1)))
            .fetch_all(&mut *conn)
            .await?
            .into_iter()
            .map(|(id, data)| Ok((id, round1::Package::from_bytes(&data)?)))
            .collect()
    }
}

// ── DkgRound2 ────────────────────────────────────────────────────────────────

pub struct DkgRound2;

impl Round for DkgRound2 {
    type Input = DkgState1;
    type Output = DkgState2;
    type Secret = round2::SecretPackage;
    type Outgoing = PerPeer<round2::Package>;
    type Public = round2::Package;

    const PREFIX: [u8; 4] = *b"DK21";

    /// Need all other participants' packages.
    fn threshold(_n: u8, t: u8) -> usize {
        t as usize
    }

    fn produce(input: &DkgState1) -> Result<(round2::SecretPackage, PerPeer<round2::Package>)> {
        // part2 takes spkg1 by value — clone since input is borrowed
        info!("DKG: calling dkg::part2 (self_id={}, n={}, t={})", input.init.self_id, input.init.n, input.init.t);
        info!("DKG: have {} peer packages for part2", input.ppkg1s.len());
        let (spkg2, ppkg2s) = dkg::part2(input.spkg1.clone(), &input.ppkg1s)?;
        info!("DKG: dkg::part2 completed successfully");
        // Convert BTreeMap<Identifier, Package> → BTreeMap<u8, Package>
        let per_peer: BTreeMap<u8, round2::Package> = (1u8..=input.init.n)
            .filter_map(|i| {
                let id: Identifier = (i as u16).try_into().ok()?;
                let pkg = ppkg2s.get(&id)?.clone();
                Some((i, pkg))
            })
            .collect();
        Ok((spkg2, PerPeer(per_peer)))
    }

    fn collect(
        state1: DkgState1,
        spkg2: round2::SecretPackage,
        peers: Vec<(u8, round2::Package)>,
    ) -> Result<DkgState2> {
        let ppkg2s = peers
            .into_iter()
            .filter(|(id, _)| *id != state1.init.self_id)  // Skip our own package
            .map(|(id, pkg)| Ok(((id as u16).try_into()?, pkg)))
            .collect::<Result<_>>()?;
        Ok(DkgState2 { state1, spkg2, ppkg2s })
    }

    async fn load_secret(conn: &mut SqliteConnection, account: u32) -> Result<Option<round2::SecretPackage>> {
        sqlx::query_as::<_, (Vec<u8>,)>("SELECT spkg2 FROM dkg_state WHERE account = ? AND spkg2 IS NOT NULL")
            .bind(account)
            .fetch_optional(&mut *conn)
            .await?
            .map(|(b,)| round2::SecretPackage::from_bytes(&b))
            .transpose()
    }

    async fn store_secret(conn: &mut SqliteConnection, account: u32, s: &round2::SecretPackage) -> Result<()> {
        sqlx::query(
            "INSERT INTO dkg_state(account, spkg2) VALUES(?1, ?2)
            ON CONFLICT(account) DO UPDATE SET spkg2 = excluded.spkg2",
        )
        .bind(account)
        .bind(s.to_bytes()?)
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    async fn store_public(conn: &mut SqliteConnection, account: u32, from_id: u8, p: &round2::Package) -> Result<()> {
        sqlx::query(
            "INSERT INTO dkg_peers(account, round, from_id, data) VALUES(?1, 2, ?2, ?3)
            ON CONFLICT DO NOTHING",
        )
        .bind(account)
        .bind(from_id)
        .bind(p.to_bytes()?)
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    async fn load_publics(conn: &mut SqliteConnection, account: u32) -> Result<Vec<(u8, round2::Package)>> {
        sqlx::query("SELECT from_id, data FROM dkg_peers WHERE account = ? AND round = 2")
            .bind(account)
            .map(|row: SqliteRow| (row.get::<u8, _>(0), row.get::<Vec<u8>, _>(1)))
            .fetch_all(&mut *conn)
            .await?
            .into_iter()
            .map(|(id, data)| Ok((id, round2::Package::from_bytes(&data)?)))
            .collect()
    }
}

// ── DKG helpers ──────────────────────────────────────────────────────────────

pub async fn set_dkg_params(
    _network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    name: &str,
    id: u8,
    n: u8,
    t: u8,
    funding_account: u32,
) -> Result<()> {
    let height = client.latest_height().await?;
    let birth_height = height.saturating_sub(10000) + 1;
    tracing::info!("birth_height {birth_height}");

    sqlx::query(
        "INSERT INTO dkg_params(account, id, n, t, seed, birth_height, name) VALUES (?, ?, ?, ?, '', ?, ?)",
    )
    .bind(funding_account)
    .bind(id)
    .bind(n)
    .bind(t)
    .bind(birth_height)
    .bind(name)
    .execute(&mut *connection)
    .await?;
    sqlx::query("INSERT INTO props(key, value) VALUES ('dkg_account', ?1)")
        .bind(funding_account)
        .execute(&mut *connection)
        .await?;

    Ok(())
}

pub async fn set_dkg_address(
    connection: &mut SqliteConnection,
    account: u32,
    id: u8,
    my_id: u8,
    address: &str,
) -> Result<()> {
    if id == my_id {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO dkg_addresses(account, from_id, address)
        VALUES (?, ?, ?) ON CONFLICT DO UPDATE SET address = excluded.address",
    )
    .bind(account)
    .bind(id)
    .bind(address)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

pub async fn is_dkg_ready(
    connection: &mut SqliteConnection,
    account: u32,
    n: u8,
) -> Result<bool> {
    let addresses = get_addresses(&mut *connection, account, n).await?;
    Ok(addresses.iter().all(|a| !a.is_empty()))
}

pub async fn get_dkg_params(connection: &mut SqliteConnection, account: u32) -> Result<DKGParams> {
    sqlx::query("SELECT id, n, t, birth_height FROM dkg_params WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| DKGParams {
            id: row.get(0),
            n: row.get(1),
            t: row.get(2),
            birth_height: row.get(3),
        })
        .fetch_one(&mut *connection)
        .await
        .context("Fetch dkg_params")
}

pub async fn in_dkg(connection: &mut SqliteConnection) -> Result<bool> {
    let exists = sqlx::query("SELECT 1 FROM props WHERE key LIKE 'dkg_%'")
        .fetch_optional(&mut *connection)
        .await?;
    if exists.is_none() {
        return Ok(false);
    }
    let account = get_funding_account(&mut *connection).await?;
    let (n,) = sqlx::query_as::<_, (u32,)>("SELECT n FROM dkg_params WHERE account = ?1")
        .bind(account)
        .fetch_optional(&mut *connection)
        .await?
        .unwrap_or_default();
    if n == 0 {
        return Ok(false);
    }
    let (n_addresses,): (u32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM dkg_addresses WHERE account = ?1",
    )
    .bind(account)
    .fetch_one(&mut *connection)
    .await?;
    Ok(n_addresses == n)
}

pub async fn cancel_dkg(connection: &mut SqliteConnection, account: u32) -> Result<()> {
    sqlx::query("DELETE FROM dkg_state WHERE account = ?").bind(account).execute(&mut *connection).await?;
    sqlx::query("DELETE FROM dkg_peers WHERE account = ?").bind(account).execute(&mut *connection).await?;
    sqlx::query("DELETE FROM dkg_addresses WHERE account = ?").bind(account).execute(&mut *connection).await?;
    sqlx::query("DELETE FROM dkg_params WHERE account = ?").bind(account).execute(&mut *connection).await?;
    sqlx::query("DELETE FROM props WHERE key LIKE 'dkg_%'").execute(&mut *connection).await?;
    delete_frost_state(&mut *connection).await
}

pub async fn delete_frost_state(connection: &mut SqliteConnection) -> Result<()> {
    info!("delete_frost_state");
    sqlx::query("DELETE FROM frost_signatures").execute(&mut *connection).await?;
    sqlx::query("DELETE FROM frost_commitments").execute(&mut *connection).await?;
    sqlx::query("DELETE FROM props WHERE key LIKE 'frost_%'").execute(&mut *connection).await?;
    sqlx::query("DELETE FROM props WHERE key LIKE 'dkg_%'").execute(&mut *connection).await?;
    let frost_accounts = sqlx::query_as::<_, (u32,)>(
        "SELECT id_account FROM accounts WHERE name LIKE 'frost-%' AND internal = 1",
    )
    .fetch_all(&mut *connection)
    .await?;
    for (frost_account,) in frost_accounts {
        delete_account(&mut *connection, frost_account).await?;
    }
    Ok(())
}

// ── Main orchestrator ────────────────────────────────────────────────────────

#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;

#[cfg(feature = "flutter")]
pub async fn do_dkg(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    client: &mut Client,
    height: u32,
    status: StreamSink<DKGStatus>,
) -> Result<()> {
    do_dkg_impl(network, connection, account, client, height, status).await
}

pub async fn do_dkg_impl(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    client: &mut Client,
    height: u32,
    status: impl Sink<DKGStatus>,
) -> Result<()> {
    info!("dkg: {account}");

    let guard = SYNCING.try_lock();
    if guard.is_err() {
        return Ok(());
    }

    let DKGParams { id: self_id, n, t, birth_height } = get_dkg_params(connection, account).await?;

    let (mailbox_account, _) =
        get_mailbox_account(network, connection, account, self_id, birth_height).await?;
    let (broadcast_account, broadcast_address) =
        get_coordinator_broadcast_account(network, connection, account, birth_height).await?;

    let addresses = get_addresses(connection, account, n).await?;
    let route_ctx = RouteCtx {
        broadcast_address: broadcast_address.clone(),
        coordinator_address: broadcast_address.clone(), // unused in DKG
        peer_addresses: addresses,
    };

    // ── Round 1: everyone broadcasts one package to the shared address ────────
    let init = DkgInit { self_id, n, t };
    let Some(state1) = run_round::<DkgRound1>(
        connection,
        account,
        n, t, self_id,
        account,            // funding_account
        broadcast_account,  // incoming memos arrive at broadcast
        init,
        &route_ctx,
        network,
        client,
        height,
    )
    .await?
    else {
        status.send(DKGStatus::WaitRound1Pkg).await;
        return Ok(());
    };
    info!("Round 1 complete");

    // ── Round 2: each sends a unique package to every peer's mailbox ──────────
    let Some(state2) = run_round::<DkgRound2>(
        connection,
        account,
        n, t, self_id,
        account,           // funding_account
        mailbox_account,   // incoming memos arrive at our private mailbox
        state1,
        &route_ctx,
        network,
        client,
        height,
    )
    .await?
    else {
        status.send(DKGStatus::WaitRound2Pkg).await;
        return Ok(());
    };
    info!("Round 2 complete");

    // ── Round 3: local only — derive the shared key ───────────────────────────
    let key_pkg = sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT key_pkg FROM dkg_state WHERE account = ? AND key_pkg IS NOT NULL",
    )
    .bind(account)
    .fetch_optional(&mut *connection)
    .await?;

    let (key_pkg, pub_key_pkg) = if let Some((data,)) = key_pkg {
        // Already computed on a previous block — reload from DB
        let kp = KeyPackage::<_>::from_bytes(&data)?;
        let (pp_data,) = sqlx::query_as::<_, (Vec<u8>,)>(
            "SELECT data FROM dkg_peers WHERE account = ? AND round = 3 LIMIT 1",
        )
        .bind(account)
        .fetch_one(&mut *connection)
        .await?;
        (kp, PublicKeyPackage::from_bytes(&pp_data)?)
    } else {
        info!("DKG: calling dkg::part3 (self_id={}, n={}, t={})", self_id, n, t);
        let (kp, pp) = dkg::part3(&state2.spkg2, &state2.state1.ppkg1s, &state2.ppkg2s)?;
        info!("DKG: dkg::part3 completed successfully");
        sqlx::query(
            "UPDATE dkg_state SET key_pkg = ?1 WHERE account = ?2",
        )
        .bind(kp.to_bytes()?)
        .bind(account)
        .execute(&mut *connection)
        .await?;
        sqlx::query(
            "INSERT INTO dkg_peers(account, round, from_id, data) VALUES(?1, 3, ?2, ?3)
            ON CONFLICT DO NOTHING",
        )
        .bind(account)
        .bind(self_id)
        .bind(pp.to_bytes()?)
        .execute(&mut *connection)
        .await?;
        (kp, pp)
    };

    // Build the shared Orchard address by replacing the spend-auth key in the
    // broadcast account's FVK with the FROST group public key.
    let fvk = get_orchard_vk(connection, broadcast_account)
        .await?
        .expect("broadcast account vk not found");
    let mut fvkb = fvk.to_bytes();
    let pub_key_pkg = pub_key_pkg.into_even_y(None);
    let vk = pub_key_pkg.verifying_key();
    let pkb = vk.serialize().expect("pk serialize");
    fvkb[0..32].copy_from_slice(&pkb);
    let shared_fvk = FullViewingKey::from_bytes(&fvkb).expect("Failed to create shared FVK");
    let address = shared_fvk.address_at(0u64, Scope::External);
    let ua = UnifiedAddress::from_receivers(Some(address), None, None).unwrap();
    let sua = ua.encode(network);
    info!("Shared address: {sua}");

    let (name,) = sqlx::query_as::<_, (String,)>("SELECT name FROM dkg_params WHERE account = ?")
        .bind(account)
        .fetch_one(&mut *connection)
        .await?;
    let frost_account =
        store_account_metadata(connection, &name, &None, &None, height, false, false).await?;
    init_account_orchard(network, connection, frost_account, height).await?;
    store_account_orchard_vk(connection, frost_account, &shared_fvk).await?;

    dkg_finalize(connection, account, frost_account, mailbox_account, broadcast_account).await?;

    // Store key material under the frost account
    sqlx::query(
        "UPDATE dkg_state SET key_pkg = ?1 WHERE account = ?2",
    )
    .bind(key_pkg.to_bytes()?)
    .bind(frost_account)
    .execute(&mut *connection)
    .await?;

    status.send(DKGStatus::SharedAddress(sua)).await;

    cancel_dkg(connection, account).await?;
    Ok(())
}

async fn dkg_finalize(
    connection: &mut SqliteConnection,
    account: u32,
    frost_account: u32,
    mailbox_account: u32,
    broadcast_account: u32,
) -> Result<()> {
    sqlx::query("UPDATE dkg_params SET account = ?1 WHERE account = ?2")
        .bind(frost_account).bind(account).execute(&mut *connection).await?;
    sqlx::query("UPDATE dkg_state SET account = ?1 WHERE account = ?2")
        .bind(frost_account).bind(account).execute(&mut *connection).await?;
    sqlx::query("UPDATE dkg_peers SET account = ?1 WHERE account = ?2")
        .bind(frost_account).bind(account).execute(&mut *connection).await?;
    sqlx::query("UPDATE dkg_addresses SET account = ?1 WHERE account = ?2")
        .bind(frost_account).bind(account).execute(&mut *connection).await?;
    sqlx::query("DELETE FROM props WHERE key LIKE 'dkg_%'")
        .execute(&mut *connection).await?;
    let seed = get_account_seed(&mut *connection, mailbox_account)
        .await?
        .expect("mailbox seed not found")
        .mnemonic;
    sqlx::query("UPDATE dkg_params SET seed = ?1 WHERE account = ?2")
        .bind(seed).bind(frost_account).execute(&mut *connection).await?;
    delete_account(&mut *connection, mailbox_account).await?;
    delete_account(&mut *connection, broadcast_account).await?;
    Ok(())
}
