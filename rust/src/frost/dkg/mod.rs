use std::collections::BTreeMap;

use anyhow::{Context, Result};
use ed25519_dalek::{SigningKey, VerifyingKey, SECRET_KEY_LENGTH};
use rand_core_legacy::OsRng;
use reddsa_legacy::frost::redpallas::{
    keys::dkg::{self, round1, round2},
    Identifier,
};
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};
use tracing::info;

use crate::{
    api::{
        coin::Network,
        frost::{get_funding_account, DKGParams},
    },
    db::delete_account,
    frost::{
        keys::{CURRENT_DKG_VERSION, DKG_VERSION_1},
        Broadcast, FrostBytes, PerPeer, Round,
    },
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

// ── Round-0 package ──────────────────────────────────────────────────────────

/// What a participant broadcasts in round 0: its ed25519 message key and
/// the DKG version it will derive the group key under.
///
/// Wire form is `verifying_key (32) || dkg_version (1)`. A dkg-1 build sent
/// the bare 32-byte key, so a 32-byte payload decodes as version 1; a dkg-1
/// build cannot decode the 33-byte payload and never leaves round 0, which
/// is the intended failure when builds are mixed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round0Pkg {
    pub verifying_key: VerifyingKey,
    pub dkg_version: u8,
}

impl FrostBytes for Round0Pkg {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = self.verifying_key.as_bytes().to_vec();
        bytes.push(self.dkg_version);
        Ok(bytes)
    }

    fn from_bytes(data: &[u8]) -> Result<Self> {
        let dkg_version = match data.len() {
            32 => DKG_VERSION_1,
            33 => data[32],
            n => anyhow::bail!("Invalid round-0 package length: expected 32 or 33, got {n}"),
        };
        let verifying_key = VerifyingKey::from_bytes(data[..32].try_into()?)?;
        Ok(Self {
            verifying_key,
            dkg_version,
        })
    }
}

#[cfg(test)]
mod round0_tests {
    use super::*;

    fn pkg(dkg_version: u8) -> Round0Pkg {
        Round0Pkg {
            verifying_key: SigningKey::generate(&mut OsRng).verifying_key(),
            dkg_version,
        }
    }

    /// The versioned payload round-trips; a dkg-1 build's bare key decodes
    /// as version 1; anything else is rejected.
    #[test]
    fn round0_package_wire_form() {
        let p = pkg(2);
        let bytes = p.to_bytes().unwrap();
        assert_eq!(bytes.len(), 33);
        assert_eq!(Round0Pkg::from_bytes(&bytes).unwrap(), p);

        let legacy = Round0Pkg::from_bytes(p.verifying_key.as_bytes()).unwrap();
        assert_eq!(legacy.dkg_version, DKG_VERSION_1);
        assert_eq!(legacy.verifying_key, p.verifying_key);

        assert!(Round0Pkg::from_bytes(&bytes[..31]).is_err());
        assert!(Round0Pkg::from_bytes(&[bytes.clone(), vec![0]].concat()).is_err());
    }

    #[test]
    fn collect_fails_on_a_peer_version_mismatch() {
        let init = DkgInit {
            self_id: 1,
            n: 3,
            t: 2,
            dkg_version: 2,
        };
        let sk = SigningKey::generate(&mut OsRng);
        let peers = vec![(2, pkg(2)), (3, pkg(1))];
        let err = match DkgRound0::collect(init, sk.clone(), peers) {
            Err(e) => e,
            Ok(_) => panic!("a mismatched peer must fail collect"),
        };
        assert_eq!(
            err.downcast::<DkgVersionMismatch>().unwrap(),
            DkgVersionMismatch {
                peer: 3,
                ours: 2,
                theirs: 1
            }
        );

        // Our own package is skipped whatever it says; matching peers pass.
        let peers = vec![(1, pkg(7)), (2, pkg(2)), (3, pkg(2))];
        let s0 = DkgRound0::collect(init, sk, peers).unwrap();
        assert_eq!(s0.peer_verifying_keys.len(), 2);
    }
}

/// A peer runs a different DKG generation: every participant would derive
/// a different group address, so the DKG stops instead of completing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("participant {peer} runs DKG version {theirs}; this wallet runs version {ours}")]
pub struct DkgVersionMismatch {
    pub peer: u8,
    pub ours: u8,
    pub theirs: u8,
}

// ── State types ──────────────────────────────────────────────────────────────

/// Seed data for the first DKG round.
#[derive(Clone, Copy, Debug)]
pub struct DkgInit {
    pub self_id: u8,
    pub n: u8,
    pub t: u8,
    /// The generation this wallet derives the group key under.
    pub dkg_version: u8,
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
    type Outgoing = Broadcast<Round0Pkg>;
    type Public = Round0Pkg;

    const PREFIX: [u8; 4] = *b"DK00";

    fn produce(input: &DkgInit) -> Result<(SigningKey, Broadcast<Round0Pkg>)> {
        info!(
            "DKG Round0: generating signing keypair (self_id={}, n={}, t={}, version={})",
            input.self_id, input.n, input.t, input.dkg_version
        );
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        info!(
            "DKG Round0: signing keypair generated, public key: {}",
            hex::encode(verifying_key.as_bytes())
        );
        Ok((
            signing_key,
            Broadcast(Round0Pkg {
                verifying_key,
                dkg_version: input.dkg_version,
            }),
        ))
    }

    /// Fails with [`DkgVersionMismatch`] when a peer runs another generation.
    fn collect(
        input: DkgInit,
        signing_key: SigningKey,
        peers: Vec<(u8, Round0Pkg)>,
    ) -> Result<DkgState0> {
        let verifying_key = signing_key.verifying_key().clone();
        let mut peer_verifying_keys = BTreeMap::new();
        for (id, pkg) in peers {
            if id == input.self_id {
                continue; // Skip our own key
            }
            if pkg.dkg_version != input.dkg_version {
                return Err(DkgVersionMismatch {
                    peer: id,
                    ours: input.dkg_version,
                    theirs: pkg.dkg_version,
                }
                .into());
            }
            peer_verifying_keys.insert(id, pkg.verifying_key);
        }
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
        p: &Round0Pkg,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO dkg_peers(account, round, from_id, data) VALUES(?1, 0, ?2, ?3)
            ON CONFLICT DO NOTHING",
        )
        .bind(account)
        .bind(from_id)
        .bind(p.to_bytes()?)
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    /// Rows written by a dkg-1 build hold the bare 32-byte key and decode
    /// as version 1.
    async fn load_publics(
        conn: &mut SqliteConnection,
        account: u32,
    ) -> Result<Vec<(u8, Round0Pkg)>> {
        let rows =
            sqlx::query("SELECT from_id, data FROM dkg_peers WHERE account = ? AND round = 0")
                .bind(account)
                .map(|row: SqliteRow| (row.get::<u8, _>(0), row.get::<Vec<u8>, _>(1)))
                .fetch_all(&mut *conn)
                .await?;

        rows.into_iter()
            .map(|(id, data)| {
                Round0Pkg::from_bytes(&data)
                    .map(|pkg| (id, pkg))
                    .with_context(|| format!("round-0 package of participant {id}"))
            })
            .collect()
    }
}

/// The round-0 message key a peer published, from its stored package. None
/// when the peer sent nothing in round 0 (a dkg-0 group, or a restored
/// member), in which case its signing messages are unsigned and accepted.
pub async fn load_peer_verifying_key(
    connection: &mut SqliteConnection,
    account: u32,
    from_id: u8,
) -> Result<Option<VerifyingKey>> {
    let row = sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT data FROM dkg_peers WHERE account = ? AND round = 0 AND from_id = ?",
    )
    .bind(account)
    .bind(from_id)
    .fetch_optional(&mut *connection)
    .await?;
    row.map(|(data,)| {
        Round0Pkg::from_bytes(&data)
            .map(|pkg| pkg.verifying_key)
            .with_context(|| format!("round-0 package of participant {from_id}"))
    })
    .transpose()
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
        "INSERT INTO dkg_params(account, id, n, t, seed, birth_height, name, dkg_version)
        VALUES (?, ?, ?, ?, '', ?, ?, ?)",
    )
    .bind(funding_account)
    .bind(id)
    .bind(n)
    .bind(t)
    .bind(birth_height)
    .bind(name)
    .bind(CURRENT_DKG_VERSION)
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
