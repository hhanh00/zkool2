use std::collections::BTreeMap;

use anyhow::{Context, Result};
use bincode::{config, Decode, Encode};
use bip39::Mnemonic;
use futures::StreamExt as _;
use group::ff::PrimeField as _;
use orchard::keys::{FullViewingKey, Scope};
use pczt::Pczt;
use rand_core::{CryptoRng, RngCore};
use reddsa::frost::redpallas::{
    frost::{round1::SigningCommitments, SigningPackage},
    keys::{
        dkg::{self, round1, round2},
        EvenY, KeyPackage, PublicKeyPackage,
    },
    Identifier, PallasBlake2b512,
};
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use tonic::Request;
use tracing::info;
use zcash_keys::address::UnifiedAddress;
use zcash_primitives::transaction::{
    sighash::SignableInput, sighash_v5::v5_signature_hash, txid::TxIdDigester,
};
use zcash_protocol::{consensus::Network, memo::Memo};

use crate::{
    account::{get_birth_height, get_orchard_vk},
    api::{
        account::{new_account, NewAccount},
        frost::{DKGPackage, DKGState, DKGStatus, FrostSignParams},
        pay::PcztPackage,
    },
    db::{delete_account, init_account_orchard, store_account_metadata, store_account_orchard_vk},
    lwd::ChainSpec,
    pay::{
        plan::{extract_transaction, plan_transaction, sign_transaction},
        pool::ALL_POOLS,
        Recipient,
    },
    Client,
};

pub type PK1Map = BTreeMap<Identifier, round1::Package>;
pub type PK2Map = BTreeMap<Identifier, round2::Package>;

impl DKGState {
    pub fn seed(&self) -> Option<Mnemonic> {
        let mut state = blake2b_simd::Params::new()
            .hash_length(32)
            .personal(b"Zcash__FROST_DKG")
            .to_state();
        for a in self.package.addresses.iter() {
            if a.is_empty() {
                return None;
            }
            state.update(a.as_bytes());
        }
        let hash = state.finalize();
        let m = Mnemonic::from_entropy(hash.as_ref()).expect("Failed to create mnemonic from hash");
        Some(m)
    }

    pub async fn get_broadcast_account(&self, connection: &SqlitePool, seed: &str) -> Result<u32> {
        let r: Option<(u32,)> = sqlx::query_as("SELECT id_account FROM accounts WHERE seed = ?1")
            .bind(seed)
            .fetch_optional(connection)
            .await?;
        if let Some((account,)) = r {
            info!("Broadcast account already exists");
            return Ok(account);
        }

        let birth_height = get_birth_height(connection, self.package.mailbox_account).await?;
        let na = NewAccount {
            name: format!("{}-frost-broadcast", self.package.name),
            icon: None,
            restore: true,
            key: seed.to_string(),
            passphrase: None,
            fingerprint: None,
            aindex: 0,
            birth: Some(birth_height),
            use_internal: false,
            internal: true,
        };
        let broadcast_account = new_account(&na).await?;

        Ok(broadcast_account)
    }

    pub async fn get_broadcast_address(
        &self,
        network: &Network,
        connection: &SqlitePool,
        broadcast_account: u32,
    ) -> Result<String> {
        let fvk = get_orchard_vk(connection, broadcast_account)
            .await?
            .unwrap();
        let address = fvk.address_at(0u64, Scope::External);
        let ua = UnifiedAddress::from_receivers(Some(address), None, None).unwrap();
        let broadcast_address = ua.encode(network);
        Ok(broadcast_address)
    }

    pub async fn get_sk1(&self, connection: &SqlitePool) -> Result<Option<round1::SecretPackage>> {
        let sk1 = sqlx::query("SELECT value FROM props WHERE key = 'frost-sk1'")
            .map(|row: SqliteRow| {
                let sk1b: Vec<u8> = row.get(0);
                let (sk1, _) = bincode::serde::decode_from_slice::<round1::SecretPackage, _>(
                    &sk1b,
                    config::standard(),
                )
                .expect("Failed to decode SecretPackage");
                sk1
            })
            .fetch_optional(connection)
            .await?;
        info!("sk1: {:?}", sk1);
        Ok(sk1)
    }

    pub async fn get_pk1(
        &self,
        connection: &SqlitePool,
        broadcast_account: u32,
    ) -> Result<Option<PK1Map>> {
        let mut pkg1map: PK1Map = BTreeMap::new();
        let mut pk1s = sqlx::query("SELECT memo_bytes FROM memos WHERE account = ?")
            .bind(broadcast_account)
            .map(|row: SqliteRow| {
                let memo_bytes: Vec<u8> = row.get(0);
                info!("memo bytes: {}", hex::encode(&memo_bytes[0..32]));
                let memo = Memo::from_bytes(&memo_bytes);
                if let Ok(memo) = memo {
                    match memo {
                        Memo::Arbitrary(pk1b) => {
                            if let Ok((pk1, _)) =
                                bincode::serde::decode_from_slice::<DKGPackage, _>(
                                    &pk1b[..],
                                    config::standard(),
                                )
                                .context("Failed to decode DKGPackage")
                            {
                                return Some(pk1);
                            }
                        }
                        _ => (),
                    }
                }
                None
            })
            .fetch(connection);
        while let Some(pk1) = pk1s.next().await {
            if let Some(pk1) = pk1? {
                let id = pk1.from_id;
                if id == self.package.id as u16 {
                    continue;
                }
                let pkg = round1::Package::deserialize(&pk1.payload)
                    .context("Failed to decode round1::Package")?;
                pkg1map.insert(id.try_into().unwrap(), pkg);
            }
        }

        if pkg1map.len() + 1 == self.package.n as usize {
            info!("All pk1 received");
            Ok(Some(pkg1map))
        } else {
            info!("Waiting for pk1s: {}/{}", pkg1map.len(), self.package.n - 1);
            Ok(None)
        }
    }

    pub async fn get_pk2(
        &self,
        connection: &SqlitePool,
        mailbox_account: u32,
    ) -> Result<Option<PK2Map>> {
        let mut pkg2map: PK2Map = BTreeMap::new();
        let mut pk2s = sqlx::query("SELECT memo_bytes FROM memos WHERE account = ?")
            .bind(mailbox_account)
            .map(|row: SqliteRow| {
                let memo_bytes: Vec<u8> = row.get(0);
                info!("memo bytes: {}", hex::encode(&memo_bytes[0..32]));
                let memo = Memo::from_bytes(&memo_bytes);
                if let Ok(memo) = memo {
                    match memo {
                        Memo::Arbitrary(pk2b) => {
                            if let Ok((pk2, _)) =
                                bincode::serde::decode_from_slice::<DKGPackage, _>(
                                    &pk2b[..],
                                    config::standard(),
                                )
                                .context("Failed to decode DKGPackage")
                            {
                                return Some(pk2);
                            }
                        }
                        _ => (),
                    }
                }
                None
            })
            .fetch(connection);
        while let Some(pk2) = pk2s.next().await {
            if let Some(pk2) = pk2? {
                let id = pk2.from_id;
                if id == self.package.id as u16 {
                    continue;
                }
                info!("pk2 id: {} -> {}", id, hex::encode(&pk2.payload[0..32]));
                let pkg = round2::Package::deserialize(&pk2.payload)
                    .context("Failed to decode round2::Package")?;
                pkg2map.insert(id.try_into().unwrap(), pkg);
            }
        }

        if pkg2map.len() + 1 == self.package.n as usize {
            info!("All pk2 received");
            Ok(Some(pkg2map))
        } else {
            info!("Waiting for pk2s: {}/{}", pkg2map.len(), self.package.n - 1);
            Ok(None)
        }
    }

    pub async fn run_phase1<R: RngCore + CryptoRng>(
        &self,
        network: &Network,
        connection: &SqlitePool,
        client: &mut Client,
        broadcast_address: &str,
        rng: R,
    ) -> Result<round1::SecretPackage> {
        let id = self.package.id as u16;
        let (sk1, pk1) = dkg::part1::<R>(
            id.try_into().unwrap(),
            self.package.n as u16,
            self.package.t as u16,
            rng,
        )?;
        let pk1 = DKGPackage {
            from_id: id,
            payload: pk1.serialize()?,
        };

        let sk1b = bincode::serde::encode_to_vec(&sk1, config::standard())?;
        let pk1 = bincode::serde::encode_to_vec(&pk1, config::standard())?;
        let height = client
            .get_latest_block(Request::new(ChainSpec {}))
            .await?
            .into_inner()
            .height;
        let pczt = plan_transaction(
            network,
            connection,
            client,
            self.package.funding_account,
            ALL_POOLS,
            std::slice::from_ref(&Recipient {
                address: broadcast_address.to_string(),
                amount: 0,
                pools: None,
                user_memo: None,
                memo_bytes: Some(to_arb_memo(&pk1)),
            }),
            false,
        )
        .await?;
        let pczt = sign_transaction(connection, self.package.funding_account, &pczt).await?;
        let txb = extract_transaction(&pczt).await?;
        let txid = crate::pay::send(client, height as u32, &txb).await?;
        info!("Broadcasted transaction: {txid}");

        sqlx::query(
            r#"INSERT INTO props(key, value) VALUES ('frost-sk1', $1) ON CONFLICT(key) DO NOTHING"#,
        )
        .bind(&sk1b)
        .execute(connection)
        .await?;

        sqlx::query(r#"INSERT INTO props(key, value) VALUES ($1, $2) ON CONFLICT(key) DO NOTHING"#)
            .bind(format!("frost-pk1-{id}"))
            .bind(&pk1)
            .execute(connection)
            .await?;

        Ok(sk1)
    }

    pub async fn get_sk2(&self, connection: &SqlitePool) -> Result<Option<round2::SecretPackage>> {
        let sk2 = sqlx::query("SELECT value FROM props WHERE key = 'frost-sk2'")
            .map(|row: SqliteRow| {
                let sk2b: Vec<u8> = row.get(0);
                let (sk2, _) = bincode::serde::decode_from_slice::<round2::SecretPackage, _>(
                    &sk2b,
                    config::standard(),
                )
                .expect("Failed to decode SecretPackage");
                sk2
            })
            .fetch_optional(connection)
            .await?;
        Ok(sk2)
    }

    pub async fn run_phase2(
        &self,
        network: &Network,
        connection: &SqlitePool,
        client: &mut Client,
        sk1: &round1::SecretPackage,
        pk1s: &BTreeMap<Identifier, round1::Package>,
    ) -> Result<round2::SecretPackage> {
        let (sk2, pk2s) = dkg::part2(sk1.clone(), pk1s)?;
        info!("Frost secret key 2: {:?}", sk2);

        let sk2b = bincode::serde::encode_to_vec(&sk2, config::standard())?;
        let height = client
            .get_latest_block(Request::new(ChainSpec {}))
            .await?
            .into_inner()
            .height;
        let mut recipients = vec![];
        for (id, pk2) in pk2s.into_iter() {
            let e = id.to_scalar().to_repr();
            let id = e[0] as u16;

            let pk2b = bincode::serde::encode_to_vec(&pk2, config::standard())?;
            sqlx::query(
                r#"INSERT INTO props(key, value) VALUES ($1, $2) ON CONFLICT(key) DO NOTHING"#,
            )
            .bind(format!("frost-pk2-{}-{id}", self.package.id))
            .bind(&pk2b)
            .execute(connection)
            .await?;

            let pk2p = DKGPackage {
                from_id: self.package.id as u16,
                payload: pk2b.clone(),
            };
            let pk2b = bincode::serde::encode_to_vec(&pk2p, config::standard())?;

            let recipient = Recipient {
                address: self.package.addresses[id as usize - 1].to_string(),
                amount: 0,
                pools: None,
                user_memo: None,
                memo_bytes: Some(to_arb_memo(&pk2b)),
            };
            recipients.push(recipient);
        }

        let pczt = plan_transaction(
            network,
            connection,
            client,
            self.package.funding_account,
            ALL_POOLS,
            &recipients,
            false,
        )
        .await?;
        let pczt = sign_transaction(connection, self.package.funding_account, &pczt).await?;
        let txb = extract_transaction(&pczt).await?;
        let txid = crate::pay::send(client, height as u32, &txb).await?;
        info!("Broadcasted transaction: {txid}");

        sqlx::query(
            r#"INSERT INTO props(key, value) VALUES ('frost-sk2', ?) ON CONFLICT(key) DO NOTHING"#,
        )
        .bind(&sk2b)
        .execute(connection)
        .await?;

        Ok(sk2)
    }

    pub async fn finalize(
        &self,
        connection: &SqlitePool,
        fvk: &FullViewingKey,
        mailbox_account: u32,
        broadcast_account: u32,
        sk1: &round1::SecretPackage,
        sk2: &round2::SecretPackage,
        sk: &KeyPackage,
        pk: &PublicKeyPackage,
        pk1s: &BTreeMap<Identifier, round1::Package>,
        pk2s: &BTreeMap<Identifier, round2::Package>,
    ) -> Result<()> {
        let r = sqlx::query("SELECT 1 FROM props WHERE key = 'frost'")
            .fetch_optional(connection)
            .await?;
        if r.is_none() {
            info!("Frost already finalized");
            return Ok(());
        }

        let package = &self.package;
        let birth = get_birth_height(connection, package.mailbox_account).await?;
        let account =
            store_account_metadata(connection, &package.name, &None, &None, birth, false, false)
                .await?;
        init_account_orchard(connection, account, birth).await?;
        store_account_orchard_vk(connection, account, fvk).await?;

        let (seed,): (String,) = sqlx::query_as("SELECT seed FROM accounts WHERE id_account = ?1")
            .bind(package.mailbox_account)
            .fetch_one(connection)
            .await?;

        sqlx::query(
            r#"INSERT INTO dkg_params(account, id, n, t, seed, birth_height)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT DO NOTHING"#,
        )
        .bind(account)
        .bind(package.id)
        .bind(package.n)
        .bind(package.t)
        .bind(&seed)
        .bind(birth)
        .execute(connection)
        .await?;

        for (i, address) in package.addresses.iter().enumerate() {
            sqlx::query(
                r#"INSERT INTO dkg_packages(account, public, round, from_id, data)
                VALUES (?, TRUE, 0, ?, ?) ON CONFLICT DO NOTHING"#,
            )
            .bind(account)
            .bind(i as u32 + 1)
            .bind(address)
            .execute(connection)
            .await?;
        }

        for round in 1..=2 {
            let sk = if round == 1 {
                sk1.serialize()?
            } else {
                sk2.serialize()?
            };
            sqlx::query(
                r#"INSERT INTO dkg_packages(account, public, round, from_id, data)
                VALUES (?, FALSE, ?, ?, ?) ON CONFLICT DO NOTHING"#,
            )
            .bind(account)
            .bind(round as u32)
            .bind(package.id)
            .bind(&sk)
            .execute(connection)
            .await?;

            for id in 1..=package.n {
                if id == package.id {
                    continue;
                }

                let other_id: Identifier = (id as u16).try_into().unwrap();
                let pk = if round == 1 {
                    let pk1 = pk1s.get(&other_id).unwrap();
                    pk1.serialize()?
                } else {
                    let pk2 = pk2s.get(&other_id).unwrap();
                    pk2.serialize()?
                };

                sqlx::query(
                    r#"INSERT INTO dkg_packages(account, public, round, from_id, data)
                    VALUES (?, TRUE, ?, ?, ?) ON CONFLICT DO NOTHING"#,
                )
                .bind(account)
                .bind(round)
                .bind(id)
                .bind(&pk)
                .execute(connection)
                .await?;
            }
        }
        sqlx::query(
            r#"INSERT INTO dkg_packages(account, public, round, from_id, data)
            VALUES (?, FALSE, 3, ?, ?) ON CONFLICT DO NOTHING"#,
        )
        .bind(account)
        .bind(package.id)
        .bind(&sk.serialize()?)
        .execute(connection)
        .await?;
        sqlx::query(
            r#"INSERT INTO dkg_packages(account, public, round, from_id, data)
            VALUES (?, TRUE, 3, ?, ?) ON CONFLICT DO NOTHING"#,
        )
        .bind(account)
        .bind(package.id)
        .bind(&pk.serialize()?)
        .execute(connection)
        .await?;

        let frost_keys = sqlx::query("SELECT key FROM props WHERE key LIKE 'frost%'")
            .map(|row: SqliteRow| {
                let key: String = row.get(0);
                key
            })
            .fetch_all(connection)
            .await?;
        for key in frost_keys {
            sqlx::query("DELETE FROM props WHERE key = ?1")
                .bind(key)
                .execute(connection)
                .await?;
        }

        delete_account(connection, mailbox_account).await?;
        delete_account(connection, broadcast_account).await?;

        Ok(())
    }

    pub async fn process<R: CryptoRng + RngCore>(
        &self,
        network: &Network,
        connection: &SqlitePool,
        client: &mut Client,
        mut rng: R,
    ) -> Result<DKGStatus> {
        let Some(seed) = self.seed() else {
            return Ok(DKGStatus::WaitAddresses);
        };
        let seed = seed.to_string();
        let broadcast_account = self.get_broadcast_account(connection, &seed).await?;
        let broadcast_address = self
            .get_broadcast_address(network, connection, broadcast_account)
            .await?;
        let sk1 = match self.get_sk1(connection).await? {
            Some(sk1) => sk1,
            None => {
                self.run_phase1(network, connection, client, &broadcast_address, &mut rng)
                    .await?
            }
        };
        let Some(pk1s) = self.get_pk1(connection, broadcast_account).await? else {
            return Ok(DKGStatus::WaitRound1Pkg);
        };
        let sk2 = match self.get_sk2(connection).await? {
            Some(sk2) => sk2,
            None => {
                self.run_phase2(network, connection, client, &sk1, &pk1s)
                    .await?
            }
        };
        let Some(pk2s) = self
            .get_pk2(connection, self.package.mailbox_account)
            .await?
        else {
            return Ok(DKGStatus::WaitRound2Pkg);
        };

        let (sk, pk) = dkg::part3(&sk2, &pk1s, &pk2s).map_err(anyhow::Error::new)?;

        let fvk = get_orchard_vk(connection, broadcast_account)
            .await?
            .expect("broadcast account vk not found");
        let mut fvkb = fvk.to_bytes();
        let pk = pk.into_even_y(None);
        let vk = pk.verifying_key();

        let pkb = vk.serialize().expect("pk serialize");
        fvkb[0..32].copy_from_slice(&pkb);
        let fvk = FullViewingKey::from_bytes(&fvkb).expect("Failed to create shared FVK");
        let address = fvk.address_at(0u64, Scope::External);
        let ua = UnifiedAddress::from_receivers(Some(address), None, None).unwrap();
        let sua = ua.encode(network);
        info!("Shared address: {sua}");

        self.finalize(
            connection,
            &fvk,
            self.package.mailbox_account,
            broadcast_account,
            &sk1,
            &sk2,
            &sk,
            &pk,
            &pk1s,
            &pk2s,
        )
        .await?;

        Ok(DKGStatus::SharedAddress(sua))
    }
}

pub async fn run<R: RngCore + CryptoRng>(
    network: &Network,
    connection: &SqlitePool,
    account: u32,
    client: &mut Client,
    mut rng: R,
) -> Result<()> {
    let pczt = sqlx::query("SELECT value FROM props WHERE key = 'frost_pczt'")
        .map(|row: SqliteRow| {
            let value: Vec<u8> = row.get(0);
            let pczt: PcztPackage = bincode::decode_from_slice(&value, config::legacy())
                .unwrap()
                .0;
            pczt
        })
        .fetch_one(connection)
        .await
        .context("Fetch pczt failed")?;
    let params = sqlx::query("SELECT value FROM props WHERE key = 'frost_sign_params'")
        .map(|row: SqliteRow| {
            let value: String = row.get(0);
            let params: FrostSignParams =
                serde_json::from_str(&value).expect("Failed to decode FrostSignParams");
            params
        })
        .fetch_one(connection)
        .await
        .context("Fetch params failed")?;
    let FrostSignParams {
        coordinator,
        funding_account,
    } = params;
    let (coordinator_address,): (String,) = sqlx::query_as(
        "SELECT data FROM dkg_packages WHERE account = ?1
        AND public = 1 AND round = 0 AND from_id = ?2",
    )
    .bind(account)
    .bind(coordinator)
    .fetch_one(connection)
    .await
    .context("Fetch coordinator address failed")?;

    let s = pczt.n_spends;
    if s[0] != 0 {
        anyhow::bail!("PCZT cannot have transparent inputs");
    }
    if s[1] != 0 {
        anyhow::bail!("PCZT cannot have sapling inputs");
    }
    let n_sigs = s[2];

    let (id,): (u8,) = sqlx::query_as("SELECT id FROM dkg_params WHERE account = ?1")
        .bind(account)
        .fetch_one(connection)
        .await
        .context("Fetch dkg id failed")?;

    if id == coordinator {
        run_frost_coordinator(
            network,
            connection,
            client,
            id,
            account,
            funding_account,
            n_sigs,
            &mut rng,
        )
        .await?;
    } else {
        run_frost_participant(
            network,
            connection,
            client,
            id,
            account,
            funding_account,
            n_sigs,
            coordinator_address,
            &mut rng,
        )
        .await?;
    }

    Ok(())
}

pub async fn run_frost_coordinator<R: RngCore + CryptoRng>(
    network: &Network,
    connection: &SqlitePool,
    client: &mut Client,
    id: u8,
    account: u32,
    funding_account: u32,
    n_sigs: usize,
    rng: &mut R,
) -> Result<()> {
    let height = client
        .get_latest_block(Request::new(ChainSpec {}))
        .await?
        .into_inner()
        .height as u32;

    let sk = sqlx::query(
        "SELECT data FROM dkg_packages WHERE account = ?1
        AND public = 0 AND round = 3",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let sk: Vec<u8> = row.get(0);
        let sk: KeyPackage = KeyPackage::deserialize(&sk).expect("Failed to decode KeyPackage");
        sk
    })
    .fetch_one(connection)
    .await
    .context("Fetch keypage failed")?;

    let r = sqlx::query_as::<_, (String,)>(
        "SELECT value FROM props WHERE key = 'frost_coordinator_mailbox'",
    )
    .fetch_optional(connection)
    .await?;
    let mailbox_account = match r {
        Some((mailbox_account,)) => str::parse::<u32>(&mailbox_account).expect("should be int"),
        None => {
            let (name,) = sqlx::query_as::<_, (String,)>(
                "SELECT name, seed FROM accounts WHERE id_account = ?1",
            )
            .bind(account)
            .fetch_one(connection)
            .await?;
            let (seed,): (String,) =
                sqlx::query_as("SELECT seed FROM dkg_params WHERE account = ?1")
                    .bind(account)
                    .fetch_one(connection)
                    .await?;

            // generate an internal account for receiving messages from the
            // other participants
            let na = NewAccount {
                name: format!("{}-frost", name),
                icon: None,
                restore: true,
                key: seed,
                passphrase: None,
                fingerprint: None,
                aindex: 0,
                birth: Some(height as u32),
                use_internal: false,
                internal: true,
            };
            let mailbox_account = new_account(&na).await?;
            sqlx::query(
                "INSERT INTO props(key, value) VALUES ('frost_coordinator_mailbox', ?1) ON CONFLICT(key) DO NOTHING",
            ).bind(mailbox_account).execute(connection).await?;
            mailbox_account
        }
    };
    info!("Coordinator mailbox account: {mailbox_account}");

    // Check messages in mailbox
    let mut messages = sqlx::query("SELECT memo_bytes FROM memos WHERE account = ?1")
        .bind(mailbox_account)
        .map(|row: SqliteRow| {
            let memo_bytes: Vec<u8> = row.get(0);
            info!("memo bytes: {}", hex::encode(&memo_bytes[0..32]));
            let memo = Memo::from_bytes(&memo_bytes);
            if let Ok(memo) = memo {
                match memo {
                    Memo::Arbitrary(pk1b) => {
                        if pk1b.len() < 4 || pk1b[0..4] != *b"COM1" {
                            return None;
                        }
                        if let Ok((commitments, _)) = bincode::decode_from_slice::<Commitments, _>(
                            &pk1b[4..],
                            config::legacy(),
                        )
                        .context("Failed to decode DKGPackage")
                        {
                            return Some(commitments);
                        }
                    }
                    _ => (),
                }
            }
            None
        })
        .fetch(connection);
    let mut signing_commitments = vec![];
    let mut noncess = vec![];
    for _ in 0..n_sigs {
        let mut signing_commitment =
            BTreeMap::<Identifier, SigningCommitments<PallasBlake2b512>>::new();
        let (nonces, commitments) =
            reddsa::frost::redpallas::frost::round1::commit(sk.signing_share(), &mut *rng);
        noncess.push(nonces);
        signing_commitment.insert(Identifier::try_from(id as u16).unwrap(), commitments);
        signing_commitments
            .push(BTreeMap::<Identifier, SigningCommitments<PallasBlake2b512>>::new());
    }

    while let Some(commitments) = messages.next().await {
        if let Some(commitments) = commitments? {
            for (idx, c) in commitments.commitments.iter().enumerate() {
                let c = SigningCommitments::<PallasBlake2b512>::deserialize(c)
                    .context("Failed to decode SigningCommitments")?;
                signing_commitments[idx].insert(commitments.from_id.try_into().unwrap(), c);
                info!("SigningCommitments: {idx} from {}", commitments.from_id);
            }
        }
    }

    let pczt = sqlx::query("SELECT value FROM props WHERE key = 'frost_pczt'")
        .map(|row: SqliteRow| {
            let value: Vec<u8> = row.get(0);
            let pczt: PcztPackage = bincode::decode_from_slice(&value, config::legacy())
                .unwrap()
                .0;
            pczt
        })
        .fetch_one(connection)
        .await
        .context("Fetch pczt failed")?;
    // let orchard_indices = &pczt.orchard_indices;
    let pczt = Pczt::parse(&pczt.pczt).expect("Failed to decode pczt");
    let tx = pczt.into_effects().unwrap();
    let txid_parts = tx.digest(TxIdDigester);
    let shielded_sighash = v5_signature_hash(&tx, &SignableInput::Shielded, &txid_parts);
    let sighash = shielded_sighash.as_bytes();
    info!("sighash: {}", hex::encode(sighash));

    let addresses = sqlx::query_as::<_, (String,)>(
        r#"SELECT data FROM dkg_packages WHERE account = ?1 AND round = 0
        ORDER BY from_id"#,
    )
    .bind(account)
    .fetch_all(connection)
    .await?;
    let addresses = addresses.into_iter().map(|row| row.0).collect::<Vec<_>>();

    let (_broadcast_account, broadcast_address) = get_coordinator_broadcast_account(
        network,
        connection,
        height,
        &"coordinator-broadcast",
        &addresses,
    )
    .await?;

    let mut recipients = vec![];
    for (idx, signing_commitments) in signing_commitments.iter().enumerate() {
        let signing_package = SigningPackage::new(signing_commitments.clone(), sighash);
        let mut sp = vec![];
        sp.extend_from_slice(b"SPK1");
        let spkg = SigningPkg {
            idx: idx as u32,
            spkg: signing_package.serialize()?,
        };
        bincode::encode_into_std_write(&spkg, &mut sp, config::legacy())?;
        let memo_bytes = to_arb_memo(&sp);
        let recipient = Recipient {
            address: broadcast_address.to_string(),
            amount: 0,
            pools: None,
            user_memo: None,
            memo_bytes: Some(memo_bytes),
        };
        recipients.push(recipient);
    }
    let pczt = plan_transaction(
        network,
        connection,
        client,
        funding_account,
        ALL_POOLS,
        &recipients,
        false,
    )
    .await?;
    info!("Funding account: {funding_account}");
    let pczt = sign_transaction(connection, funding_account, &pczt).await?;
    let txb = extract_transaction(&pczt).await?;

    let has_spkg_txid =
        sqlx::query("SELECT 1 FROM props WHERE key = 'frost_coordinator_spkg_txid'")
            .fetch_optional(connection)
            .await?
            .is_some();

    if !has_spkg_txid {
        let txid = crate::pay::send(client, height, &txb).await?;
        info!("Broadcasted transaction for signing packages: {txid}");

        sqlx::query("INSERT INTO props(key, value) VALUES ('frost_coordinator_spkg_txid', ?1) ON CONFLICT(key) DO NOTHING")
        .bind(&txid).execute(connection).await?;
    }

    Ok(())
}

#[derive(Encode, Decode)]
pub struct SigningPkg {
    pub idx: u32,
    pub spkg: Vec<u8>,
}

pub async fn run_frost_participant<R: RngCore + CryptoRng>(
    network: &Network,
    connection: &SqlitePool,
    client: &mut Client,
    id: u8,
    account: u32,
    funding_account: u32,
    n_actions: usize,
    coordinator_address: String,
    rng: &mut R,
) -> Result<()> {
    let sk = sqlx::query(
        "SELECT data FROM dkg_packages WHERE account = ?1
        AND public = 0 AND round = 3",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let sk: Vec<u8> = row.get(0);
        let sk: KeyPackage = KeyPackage::deserialize(&sk).expect("Failed to decode KeyPackage");
        sk
    })
    .fetch_one(connection)
    .await
    .context("Fetch keypage failed")?;

    let has_commitments = sqlx::query("SELECT 1 FROM frost_signature WHERE account = ?1")
        .bind(account)
        .fetch_optional(connection)
        .await?
        .is_some();

    if !has_commitments {
        let mut commitbs = vec![];
        for i in 0..n_actions {
            let (nonces, commitments) =
                reddsa::frost::redpallas::frost::round1::commit(sk.signing_share(), &mut *rng);
            sqlx::query("INSERT OR REPLACE INTO frost_signature(account, idx, nonces, commitments) VALUES (?, ?, ?, ?)")
            .bind(account)
            .bind(i as u32)
            .bind(&nonces.serialize()?)
            .bind(&commitments.serialize()?)
            .execute(connection).await?;
            let commitb = commitments.serialize()?;
            commitbs.push(commitb);
        }
        let commitments = Commitments {
            from_id: id as u16,
            commitments: commitbs,
        };
        let mut commitments_buffer = vec![];
        commitments_buffer.extend_from_slice(b"COM1");
        bincode::encode_into_std_write(&commitments, &mut commitments_buffer, config::legacy())?;
        let pczt = plan_transaction(
            network,
            connection,
            client,
            funding_account,
            ALL_POOLS,
            std::slice::from_ref(&Recipient {
                address: coordinator_address,
                amount: 0,
                pools: None,
                user_memo: None,
                memo_bytes: Some(to_arb_memo(&commitments_buffer)),
            }),
            false,
        )
        .await?;
        let height = client
            .get_latest_block(Request::new(ChainSpec {}))
            .await?
            .into_inner()
            .height;
        let pczt = sign_transaction(connection, funding_account, &pczt).await?;
        let txb = extract_transaction(&pczt).await?;
        let txid = crate::pay::send(client, height as u32, &txb).await?;
        info!("Broadcasted transaction: {txid}");
    }

    Ok(())
}

#[derive(Encode, Decode)]
pub struct Commitments {
    pub from_id: u16,
    pub commitments: Vec<Vec<u8>>,
}

pub fn to_arb_memo(pk1: &[u8]) -> Vec<u8> {
    let mut memo_bytes = vec![0xFF];
    memo_bytes.extend_from_slice(&pk1);
    memo_bytes
}

pub async fn get_coordinator_broadcast_account(
    network: &Network,
    connection: &SqlitePool,
    height: u32,
    name: &str,
    addresses: &[String],
) -> Result<(u32, String)> {
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
        let r = sqlx::query_as::<_, (u32, Vec<u8>)>(
            "SELECT a.id_account, o.xvk FROM accounts a
        JOIN orchard_accounts o ON a.id_account = o.account
        WHERE seed = ?1",
        )
        .bind(&seed)
        .fetch_optional(connection)
        .await?;

        match r {
            None => {
                let na = NewAccount {
                    name: format!("{}-frost-broadcast", name),
                    icon: None,
                    restore: true,
                    key: seed.to_string(),
                    passphrase: None,
                    fingerprint: None,
                    aindex: 0,
                    birth: Some(height),
                    use_internal: false,
                    internal: true,
                };
                new_account(&na).await?;
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
