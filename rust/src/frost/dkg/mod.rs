use std::collections::BTreeMap;

use anyhow::{Context, Result};
use ed25519_dalek::{SigningKey, VerifyingKey, SECRET_KEY_LENGTH};
use rand_core::OsRng;
use reddsa::frost::redpallas::{
    keys::dkg::{self, round1, round2},
    Identifier,
};
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};
use tracing::info;

use crate::{
    api::{coin::Network, frost::{get_funding_account, DKGParams}},
    db::delete_account,
    frost::{Broadcast, FrostBytes, PerPeer, Round},
    Client,
};

pub mod exec;
pub mod plan;
pub mod state;
pub mod step;
pub mod task;

pub use super::protocol::{
    get_addresses, get_coordinator_broadcast_account, get_mailbox_account, publish,
};

// ── FrostBytes for ed25519 types ─────────────────────────────────────────────

impl FrostBytes for SigningKey {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.to_bytes().to_vec())
    }

    fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() != SECRET_KEY_LENGTH {
            anyhow::bail!(
                "Invalid SigningKey length: expected {}, got {}",
                SECRET_KEY_LENGTH,
                data.len()
            );
        }
        let arr: [u8; SECRET_KEY_LENGTH] = data[..SECRET_KEY_LENGTH]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to convert slice to array"))?;
        Ok(SigningKey::from_bytes(&arr))
    }
}

impl FrostBytes for VerifyingKey {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.as_bytes().to_vec())
    }

    fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() != 32 {
            anyhow::bail!(
                "Invalid VerifyingKey length: expected 32, got {}",
                data.len()
            );
        }
        let arr: [u8; 32] = data
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to convert slice to array"))?;
        VerifyingKey::from_bytes(&arr).map_err(|e| anyhow::anyhow!("Invalid VerifyingKey: {}", e))
    }
}

// ── State types ──────────────────────────────────────────────────────────────

/// Seed data for the first DKG round.
#[derive(Clone, Copy, Debug)]
pub struct DkgInit {
    pub self_id: u8,
    pub n: u8,
    pub t: u8,
}

/// State after round 0 completes: our signing keypair + all peers' public keys.
#[derive(Clone)]
pub struct DkgState0 {
    pub init: DkgInit,
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
    pub peer_verifying_keys: BTreeMap<u8, VerifyingKey>,
}

/// State after round 1 completes: our secret + all peers' round-1 packages.
#[derive(Clone)]
pub struct DkgState1 {
    pub state0: DkgState0,
    pub spkg1: round1::SecretPackage,
    pub ppkg1s: BTreeMap<Identifier, round1::Package>,
}

/// State after round 2 completes: carries forward everything needed for part3.
#[derive(Clone)]
pub struct DkgState2 {
    pub state1: DkgState1,
    pub spkg2: round2::SecretPackage,
    pub ppkg2s: BTreeMap<Identifier, round2::Package>,
}

// ── DkgRound0 ────────────────────────────────────────────────────────────────

pub struct DkgRound0;

impl Round for DkgRound0 {
    type Input = DkgInit;
    type Output = DkgState0;
    type Secret = SigningKey;
    type Outgoing = Broadcast<VerifyingKey>;
    type Public = VerifyingKey;

    const PREFIX: [u8; 4] = *b"DK00";

    fn produce(input: &DkgInit) -> Result<(SigningKey, Broadcast<VerifyingKey>)> {
        info!(
            "DKG Round0: generating signing keypair (self_id={}, n={}, t={})",
            input.self_id, input.n, input.t
        );
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        info!(
            "DKG Round0: signing keypair generated, public key: {}",
            hex::encode(verifying_key.as_bytes())
        );
        Ok((signing_key, Broadcast(verifying_key.clone())))
    }

    fn collect(
        input: DkgInit,
        signing_key: SigningKey,
        peers: Vec<(u8, VerifyingKey)>,
    ) -> Result<DkgState0> {
        let verifying_key = signing_key.verifying_key().clone();
        let peer_verifying_keys: BTreeMap<u8, VerifyingKey> = peers
            .into_iter()
            .filter(|(id, _)| *id != input.self_id) // Skip our own key
            .map(|(id, pk)| (id, pk))
            .collect();
        info!(
            "DKG Round0: collected {} peer verifying keys",
            peer_verifying_keys.len()
        );
        Ok(DkgState0 {
            init: input,
            signing_key,
            verifying_key,
            peer_verifying_keys,
        })
    }

    async fn load_secret(conn: &mut SqliteConnection, account: u32) -> Result<Option<SigningKey>> {
        let result = sqlx::query_as::<_, (Vec<u8>,)>("SELECT signing_keypair FROM dkg_state WHERE account = ? AND signing_keypair IS NOT NULL")
            .bind(account)
            .fetch_optional(&mut *conn)
            .await?;
        match result {
            Some((b,)) => {
                if b.len() != SECRET_KEY_LENGTH {
                    anyhow::bail!(
                        "Invalid SigningKey length in DB: expected {}, got {}",
                        SECRET_KEY_LENGTH,
                        b.len()
                    );
                }
                let arr: [u8; SECRET_KEY_LENGTH] = b
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Failed to convert DB bytes to array"))?;
                Ok(Some(SigningKey::from_bytes(&arr)))
            }
            None => Ok(None),
        }
    }

    async fn store_secret(conn: &mut SqliteConnection, account: u32, s: &SigningKey) -> Result<()> {
        sqlx::query(
            "INSERT INTO dkg_state(account, signing_keypair) VALUES(?1, ?2)
            ON CONFLICT(account) DO UPDATE SET signing_keypair = excluded.signing_keypair",
        )
        .bind(account)
        .bind(s.to_bytes().to_vec())
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    async fn store_public(
        conn: &mut SqliteConnection,
        account: u32,
        from_id: u8,
        p: &VerifyingKey,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO dkg_peers(account, round, from_id, data) VALUES(?1, 0, ?2, ?3)
            ON CONFLICT DO NOTHING",
        )
        .bind(account)
        .bind(from_id)
        .bind(p.as_bytes().to_vec())
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    async fn load_publics(
        conn: &mut SqliteConnection,
        account: u32,
    ) -> Result<Vec<(u8, VerifyingKey)>> {
        let rows =
            sqlx::query("SELECT from_id, data FROM dkg_peers WHERE account = ? AND round = 0")
                .bind(account)
                .map(|row: SqliteRow| (row.get::<u8, _>(0), row.get::<Vec<u8>, _>(1)))
                .fetch_all(&mut *conn)
                .await?;

        let mut result = Vec::new();
        for (id, data) in rows {
            if data.len() != 32 {
                anyhow::bail!(
                    "Invalid VerifyingKey length for participant {}: expected 32, got {}",
                    id,
                    data.len()
                );
            }
            let arr: [u8; 32] = data.try_into().map_err(|_| {
                anyhow::anyhow!(
                    "Failed to convert VerifyingKey bytes for participant {}",
                    id
                )
            })?;
            let vk = VerifyingKey::from_bytes(&arr).map_err(|e| {
                anyhow::anyhow!("Invalid VerifyingKey for participant {}: {}", id, e)
            })?;
            result.push((id, vk));
        }
        Ok(result)
    }
}

// ── DkgRound1 ────────────────────────────────────────────────────────────────

pub struct DkgRound1;

impl Round for DkgRound1 {
    type Input = DkgState0;
    type Output = DkgState1;
    type Secret = round1::SecretPackage;
    type Outgoing = Broadcast<round1::Package>;
    type Public = round1::Package;

    const PREFIX: [u8; 4] = *b"DK11";

    fn produce(input: &DkgState0) -> Result<(round1::SecretPackage, Broadcast<round1::Package>)> {
        info!(
            "DKG: calling dkg::part1 (self_id={}, n={}, t={})",
            input.init.self_id, input.init.n, input.init.t
        );
        let (spkg1, ppkg1) = dkg::part1(
            (input.init.self_id as u16).try_into()?,
            input.init.n as u16,
            input.init.t as u16,
            OsRng,
        )?;
        info!("DKG: dkg::part1 completed successfully");
        Ok((spkg1, Broadcast(ppkg1)))
    }

    fn collect(
        input: DkgState0,
        spkg1: round1::SecretPackage,
        peers: Vec<(u8, round1::Package)>,
    ) -> Result<DkgState1> {
        let ppkg1s = peers
            .into_iter()
            .filter(|(id, _)| *id != input.init.self_id) // Skip our own package
            .map(|(id, pkg)| Ok(((id as u16).try_into()?, pkg)))
            .collect::<Result<_>>()?;
        Ok(DkgState1 {
            state0: input,
            spkg1,
            ppkg1s,
        })
    }

    async fn load_secret(
        conn: &mut SqliteConnection,
        account: u32,
    ) -> Result<Option<round1::SecretPackage>> {
        sqlx::query_as::<_, (Vec<u8>,)>(
            "SELECT spkg1 FROM dkg_state WHERE account = ? AND spkg1 IS NOT NULL",
        )
        .bind(account)
        .fetch_optional(&mut *conn)
        .await?
        .map(|(b,)| round1::SecretPackage::from_bytes(&b))
        .transpose()
    }

    async fn store_secret(
        conn: &mut SqliteConnection,
        account: u32,
        s: &round1::SecretPackage,
    ) -> Result<()> {
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

    async fn store_public(
        conn: &mut SqliteConnection,
        account: u32,
        from_id: u8,
        p: &round1::Package,
    ) -> Result<()> {
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

    async fn load_publics(
        conn: &mut SqliteConnection,
        account: u32,
    ) -> Result<Vec<(u8, round1::Package)>> {
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

    fn produce(input: &DkgState1) -> Result<(round2::SecretPackage, PerPeer<round2::Package>)> {
        // part2 takes spkg1 by value — clone since input is borrowed
        info!(
            "DKG: calling dkg::part2 (self_id={}, n={}, t={})",
            input.state0.init.self_id, input.state0.init.n, input.state0.init.t
        );
        info!("DKG: have {} peer packages for part2", input.ppkg1s.len());
        let (spkg2, ppkg2s) = dkg::part2(input.spkg1.clone(), &input.ppkg1s)?;
        info!("DKG: dkg::part2 completed successfully");
        // Convert BTreeMap<Identifier, Package> → BTreeMap<u8, Package>
        let per_peer: BTreeMap<u8, round2::Package> = (1u8..=input.state0.init.n)
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
            .filter(|(id, _)| *id != state1.state0.init.self_id) // Skip our own package
            .map(|(id, pkg)| Ok(((id as u16).try_into()?, pkg)))
            .collect::<Result<_>>()?;
        Ok(DkgState2 {
            state1,
            spkg2,
            ppkg2s,
        })
    }

    async fn load_secret(
        conn: &mut SqliteConnection,
        account: u32,
    ) -> Result<Option<round2::SecretPackage>> {
        sqlx::query_as::<_, (Vec<u8>,)>(
            "SELECT spkg2 FROM dkg_state WHERE account = ? AND spkg2 IS NOT NULL",
        )
        .bind(account)
        .fetch_optional(&mut *conn)
        .await?
        .map(|(b,)| round2::SecretPackage::from_bytes(&b))
        .transpose()
    }

    async fn store_secret(
        conn: &mut SqliteConnection,
        account: u32,
        s: &round2::SecretPackage,
    ) -> Result<()> {
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

    async fn store_public(
        conn: &mut SqliteConnection,
        account: u32,
        from_id: u8,
        p: &round2::Package,
    ) -> Result<()> {
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

    async fn load_publics(
        conn: &mut SqliteConnection,
        account: u32,
    ) -> Result<Vec<(u8, round2::Package)>> {
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

pub async fn is_dkg_ready(connection: &mut SqliteConnection, account: u32, n: u8) -> Result<bool> {
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
    let (n_addresses,): (u32,) =
        sqlx::query_as("SELECT COUNT(*) FROM dkg_addresses WHERE account = ?1")
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;
    Ok(n_addresses == n)
}

pub async fn cancel_dkg(connection: &mut SqliteConnection, account: u32) -> Result<()> {
    sqlx::query("DELETE FROM dkg_state WHERE account = ?")
        .bind(account)
        .execute(&mut *connection)
        .await?;
    sqlx::query("DELETE FROM dkg_peers WHERE account = ?")
        .bind(account)
        .execute(&mut *connection)
        .await?;
    sqlx::query("DELETE FROM dkg_addresses WHERE account = ?")
        .bind(account)
        .execute(&mut *connection)
        .await?;
    sqlx::query("DELETE FROM dkg_params WHERE account = ?")
        .bind(account)
        .execute(&mut *connection)
        .await?;
    sqlx::query("DELETE FROM props WHERE key LIKE 'dkg_%'")
        .execute(&mut *connection)
        .await?;
    delete_frost_state(&mut *connection).await
}

pub async fn delete_frost_state(connection: &mut SqliteConnection) -> Result<()> {
    info!("delete_frost_state");
    sqlx::query("DELETE FROM frost_signatures")
        .execute(&mut *connection)
        .await?;
    sqlx::query("DELETE FROM frost_commitments")
        .execute(&mut *connection)
        .await?;
    sqlx::query("DELETE FROM props WHERE key LIKE 'frost_%'")
        .execute(&mut *connection)
        .await?;
    sqlx::query("DELETE FROM props WHERE key LIKE 'dkg_%'")
        .execute(&mut *connection)
        .await?;
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

