use std::{collections::BTreeMap, iter};

use anyhow::{Context, Result};
use bincode::{config, Decode, Encode};
use bip39::Mnemonic;
use db::{get_addresses, get_coordinator_broadcast_account, get_mailbox_account};
use futures::{Stream, StreamExt as _};
use group::ff::PrimeField as _;
use orchard::keys::{FullViewingKey, Scope};
use pczt::Pczt;
use rand_core::{CryptoRng, OsRng, RngCore};
use reddsa::frost::redpallas::{
    frost::{
        keys::dkg::round1::SecretPackage,
        round1::{SigningCommitments, SigningNonces},
        round2::SignatureShare,
        SigningPackage,
    },
    keys::{
        dkg::{round1, round2},
        EvenY, KeyPackage, PublicKeyPackage,
    },
    Identifier, PallasBlake2b512,
};
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use tonic::Request;
use tracing::info;
use zcash_keys::address::{self, UnifiedAddress};
use zcash_primitives::transaction::{
    sighash::SignableInput, sighash_v5::v5_signature_hash, txid::TxIdDigester,
};
use zcash_protocol::{consensus::Network, memo::Memo};

use crate::{
    account::{get_birth_height, get_orchard_vk},
    api::{
        account::{new_account, NewAccount},
        frost::{DKGParams, DKGStatus},
        pay::PcztPackage,
    },
    db::{delete_account, init_account_orchard, store_account_metadata, store_account_orchard_vk},
    lwd::ChainSpec,
    pay::{
        plan::{extract_transaction, plan_transaction, sign_transaction},
        pool::ALL_POOLS,
        send, Recipient,
    },
    Client,
};

pub type P = PallasBlake2b512;

pub mod db;
pub mod dkg;
pub mod sign;

pub type PK1Map = BTreeMap<Identifier, round1::Package>;
pub type PK2Map = BTreeMap<Identifier, round2::Package>;

#[derive(Encode, Decode)]
pub struct FrostMessage {
    pub from_id: u16,
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

/*
pub async fn run<R: RngCore + CryptoRng>(
    network: &Network,
    connection: &SqlitePool,
    account: u32,
    client: &mut Client,
    mut rng: R,
) -> Result<()> {
    let height = client
        .get_latest_block(Request::new(ChainSpec {}))
        .await?
        .into_inner()
        .height as u32;

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
    info!("Fetch coordinator: {account} {coordinator}");
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

    let (id, t): (u8, u8) = sqlx::query_as("SELECT id, t FROM dkg_params WHERE account = ?1")
        .bind(account)
        .fetch_one(connection)
        .await
        .context("Fetch dkg id failed")?;

    if id == coordinator {
        run_frost_coordinator(
            network,
            connection,
            height,
            client,
            pczt,
            id,
            account,
            funding_account,
            t as usize,
            n_sigs,
            &mut rng,
        )
        .await?;
    } else {
        run_frost_participant(
            network,
            connection,
            account,
            height,
            client,
            pczt,
            id,
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
    height: u32,
    client: &mut Client,
    pczt: PcztPackage,
    id: u8,
    account: u32,
    funding_account: u32,
    t: usize,
    n_sigs: usize,
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

    let pk = sqlx::query(
        "SELECT data FROM dkg_packages WHERE account = ?1
        AND public = 1 AND round = 3",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let pk: Vec<u8> = row.get(0);
        let pk = PublicKeyPackage::deserialize(&pk).expect("Failed to decode KeyPackage");
        pk
    })
    .fetch_one(connection)
    .await
    .context("Fetch public key pkg failed")?;

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

    let mut signing_commitments = vec![];
    for i in 0..n_sigs {
        let mut signing_commitment =
            BTreeMap::<Identifier, SigningCommitments<PallasBlake2b512>>::new();
        let r = sqlx::query(
            "SELECT nonces, commitments FROM frost_signature WHERE account = ?1 AND idx = ?2",
        )
        .bind(account)
        .bind(i as u32)
        .map(|row: SqliteRow| {
            let nonces: Vec<u8> = row.get(0);
            let commitments: Vec<u8> = row.get(1);
            let nonces = SigningNonces::<PallasBlake2b512>::deserialize(&nonces)
                .expect("Failed to decode SigningNonces");
            let commitments = SigningCommitments::<PallasBlake2b512>::deserialize(&commitments)
                .expect("Failed to decode SigningCommitments");
            (nonces, commitments)
        })
        .fetch_optional(connection)
        .await
        .context("Fetch signing commitments failed")?;

        let (_nonces, commitments) = match r {
            Some((nonces, commitments)) => (nonces, commitments),
            None => {
                let (nonces, commitments) =
                    reddsa::frost::redpallas::frost::round1::commit(sk.signing_share(), &mut *rng);
                sqlx::query("INSERT OR REPLACE INTO frost_signature(account, idx, nonces, commitments) VALUES (?, ?, ?, ?)")
                .bind(account)
                .bind(i as u32)
                .bind(&nonces.serialize()?)
                .bind(&commitments.serialize()?)
                .execute(connection).await?;
                (nonces, commitments)
            }
        };

        signing_commitment.insert(Identifier::try_from(id as u16).unwrap(), commitments);
        signing_commitments.push(signing_commitment);
    }

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
                        if pk1b.len() < 4 || pk1b[0..4] != *b"COM2" {
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
    if signing_commitments[0].len() < t {
        info!(
            "Waiting for signing commitments: {}/{}",
            signing_commitments[0].len(),
            t
        );
        return Ok(());
    }

    let pczt = Pczt::parse(&pczt.pczt).expect("Failed to decode pczt");
    let sighash = get_sighash(pczt.clone());

    let (_broadcast_account, broadcast_address) =
        get_coordinator_broadcast_account(network, connection, account, height).await?;

    let mut recipients = vec![];
    let mut signing_packages = vec![];
    for (idx, signing_commitments) in signing_commitments.iter().enumerate() {
        let signing_package = SigningPackage::new(signing_commitments.clone(), &sighash);
        signing_packages.push(signing_package.clone());

        let nonces =
            sqlx::query("SELECT nonces FROM frost_signature WHERE account = ?1 AND idx = ?2")
                .bind(account)
                .bind(idx as u32)
                .map(|row: SqliteRow| {
                    let nonces: Vec<u8> = row.get(0);
                    let nonces = SigningNonces::<PallasBlake2b512>::deserialize(&nonces)
                        .expect("Failed to decode SigningNonces");
                    nonces
                })
                .fetch_one(connection)
                .await
                .context("Fetch signing nonces failed")?;

        // let's sign our share
        let signature_share =
            reddsa::frost::redpallas::frost::round2::sign(&signing_package, &nonces, &sk)
                .context("Failed to sign")?;
        sqlx::query("UPDATE frost_signature SET signature = ?1 WHERE account = ?2 AND idx = ?3")
            .bind(&signature_share.serialize())
            .bind(account)
            .bind(idx as u32)
            .execute(connection)
            .await?;

        let mut sp = vec![];
        sp.extend_from_slice(b"SPK3");
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
    let has_spkg_txid =
        sqlx::query("SELECT 1 FROM props WHERE key = 'frost_coordinator_spkg_txid'")
            .fetch_optional(connection)
            .await?
            .is_some();

    if !has_spkg_txid {
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

        let txid = crate::pay::send(client, height, &txb).await?;
        info!("Broadcasted transaction for signing packages: {txid}");

        sqlx::query("INSERT INTO props(key, value) VALUES ('frost_coordinator_spkg_txid', ?1) ON CONFLICT(key) DO NOTHING")
        .bind(&txid).execute(connection).await?;
    } else {
        info!("Already sent signing packages");
    }

    let mut signature_shares_stream =
        sqlx::query("SELECT memo_bytes FROM memos WHERE account = ?1")
            .bind(mailbox_account)
            .map(|row: SqliteRow| {
                let memo_bytes: Vec<u8> = row.get(0);
                info!("memo bytes: {}", hex::encode(&memo_bytes[0..32]));
                let memo = Memo::from_bytes(&memo_bytes);
                if let Ok(memo) = memo {
                    match memo {
                        Memo::Arbitrary(spkgb) => {
                            if spkgb.len() < 4 || spkgb[0..4] != *b"SSH1" {
                                return None;
                            }
                            if let Ok((signature_share, _)) =
                                bincode::decode_from_slice::<SigShare, _>(
                                    &spkgb[4..],
                                    config::legacy(),
                                )
                                .context("Failed to decode SignatureShare")
                            {
                                return Some(signature_share);
                            }
                        }
                        _ => (),
                    }
                }
                None
            })
            .fetch(connection);

    let mut signature_shares_vec =
        iter::repeat_with(|| BTreeMap::<Identifier, SignatureShare<PallasBlake2b512>>::new())
            .take(n_sigs)
            .collect::<Vec<_>>();

    while let Some(sig_share) = signature_shares_stream.next().await {
        if let Some(sig_share) = sig_share? {
            let signature_share =
                SignatureShare::<PallasBlake2b512>::deserialize(&sig_share.signature)
                    .context("Failed to decode SignatureShare")?;
            // BUG: Share should be an array of shares
            info!("Adding signature share 0 from {}", sig_share.from_id);
            signature_shares_vec[0].insert(
                Identifier::try_from(sig_share.from_id).unwrap(),
                signature_share,
            );
        }
    }
    let coordinatore_sigs =
        sqlx::query("SELECT idx, signature FROM frost_signature WHERE account = ?1")
            .bind(account)
            .map(|row: SqliteRow| {
                let idx: u32 = row.get(0);
                let signature: Vec<u8> = row.get(1);
                let signature = SignatureShare::<PallasBlake2b512>::deserialize(&signature)
                    .expect("Failed to decode SignatureShare");
                (idx, signature)
            })
            .fetch_all(connection)
            .await?;

    for (idx, signature) in coordinatore_sigs {
        signature_shares_vec[idx as usize]
            .insert(Identifier::try_from(id as u16).unwrap(), signature);
    }

    for idx in 0..n_sigs {
        let signature_shares = &signature_shares_vec[idx];
        if signature_shares.len() < t {
            info!(
                "Waiting for signature shares: {}/{}",
                signature_shares.len(),
                t
            );
            return Ok(());
        }
        info!("Got signature shares");
        let signature = reddsa::frost::redpallas::frost::aggregate(
            &signing_packages[idx],
            &signature_shares,
            &pk,
        )
        .context("Failed to aggregate signature shares")?;

        info!("Signature: {}", hex::encode(signature.serialize().unwrap()));
    }

    Ok(())
}

fn get_sighash(pczt: Pczt) -> Vec<u8> {
    let tx = pczt.into_effects().unwrap();
    let txid_parts = tx.digest(TxIdDigester);
    let shielded_sighash = v5_signature_hash(&tx, &SignableInput::Shielded, &txid_parts);
    let sighash = shielded_sighash.as_bytes();
    info!("sighash: {}", hex::encode(sighash));
    sighash.to_vec()
}

#[derive(Encode, Decode)]
pub struct SigningPkg {
    pub idx: u32,
    pub spkg: Vec<u8>,
}

pub async fn run_frost_participant<R: RngCore + CryptoRng>(
    network: &Network,
    connection: &SqlitePool,
    account: u32,
    height: u32,
    client: &mut Client,
    pczt: PcztPackage,
    id: u8,
    funding_account: u32,
    n_actions: usize,
    coordinator_address: String,
    rng: &mut R,
) -> Result<()> {
    let (broadcast_account, _) =
        get_coordinator_broadcast_account(network, connection, account, height).await?;

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
        commitments_buffer.extend_from_slice(b"COM2");
        bincode::encode_into_std_write(&commitments, &mut commitments_buffer, config::legacy())?;
        let pczt = plan_transaction(
            network,
            connection,
            client,
            funding_account,
            ALL_POOLS,
            std::slice::from_ref(&Recipient {
                address: coordinator_address.clone(),
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
    } else {
        info!("Already sent commitments");
    }

    // get signing packages
    let mut signing_packages = sqlx::query("SELECT memo_bytes FROM memos WHERE account = ?1")
        .bind(broadcast_account)
        .map(|row: SqliteRow| {
            let memo_bytes: Vec<u8> = row.get(0);
            info!("memo bytes: {}", hex::encode(&memo_bytes[0..32]));
            let memo = Memo::from_bytes(&memo_bytes);
            if let Ok(memo) = memo {
                match memo {
                    Memo::Arbitrary(spkgb) => {
                        if spkgb.len() < 4 || spkgb[0..4] != *b"SPK3" {
                            return None;
                        }
                        if let Ok((spkg, _)) = bincode::decode_from_slice::<SigningPkg, _>(
                            &spkgb[4..],
                            config::legacy(),
                        )
                        .context("Failed to decode SigningPkg")
                        {
                            return Some(spkg);
                        }
                    }
                    _ => (),
                }
            }
            None
        })
        .fetch(connection);

    let mut spkgs = vec![];
    while let Some(spkg) = signing_packages.next().await {
        if let Some(spkg) = spkg? {
            let signing_package = SigningPackage::<PallasBlake2b512>::deserialize(&spkg.spkg)
                .context("Failed to decode SigningPackage")?;
            let idx = spkg.idx as usize;
            info!("SigningPackage: {idx} for {}", id);
            spkgs.push((idx, signing_package));
        }
    }

    let pczt = Pczt::parse(&pczt.pczt).expect("Failed to decode pczt");
    let sighash = get_sighash(pczt.clone());
    info!("sighash: {}", hex::encode(&sighash));

    let key_package = sqlx::query(
        "SELECT data FROM dkg_packages WHERE account = ?1 AND public = 0 AND round = 3",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let data: Vec<u8> = row.get(0);
        let kp = KeyPackage::deserialize(&data).expect("Failed to decode PublicKeyPackage");
        kp
    })
    .fetch_one(connection)
    .await?;

    for (idx, spkg) in spkgs.iter() {
        let message = spkg.message();
        info!("Signing message: {}", hex::encode(message));
        if message != &sighash {
            anyhow::bail!("Invalid signing message");
        }
        let nonces =
            sqlx::query("SELECT nonces FROM frost_signature WHERE account = ?1 AND idx = ?2")
                .bind(account)
                .bind(*idx as u32)
                .map(|row: SqliteRow| {
                    let nonces: Vec<u8> = row.get(0);
                    let nonces = SigningNonces::<PallasBlake2b512>::deserialize(&nonces)
                        .expect("Failed to decode SigningNonces");
                    nonces
                })
                .fetch_one(connection)
                .await?;

        let signature_share =
            reddsa::frost::redpallas::frost::round2::sign(spkg, &nonces, &key_package)
                .context("Failed to sign")?;
        info!(
            "Signature share: {}",
            hex::encode(&signature_share.serialize())
        );

        let signature =
            sqlx::query("SELECT signature FROM frost_signature WHERE account = ?1 AND idx = ?2")
                .bind(account)
                .bind(*idx as u32)
                .map(|row: SqliteRow| {
                    let signature: Option<Vec<u8>> = row.get(0);
                    let signature = signature.map(|signature| {
                        reddsa::frost::redpallas::frost::round2::SignatureShare::<
                                PallasBlake2b512,
                            >::deserialize(&signature)
                            .expect("Failed to decode SignatureShare")
                    });
                    signature
                })
                .fetch_one(connection)
                .await?;
        if signature.is_none() {
            sqlx::query(
                "UPDATE frost_signature SET signature = ?1 WHERE account = ?2 AND idx = ?3",
            )
            .bind(&signature_share.serialize())
            .bind(account)
            .bind(*idx as u32)
            .execute(connection)
            .await?;
            info!("Signature share stored");

            let mut signature_share_buffer = vec![];
            signature_share_buffer.extend_from_slice(b"SSH1");
            let signature_share = SigShare {
                from_id: id as u16,
                signature: signature_share.serialize(),
            };
            bincode::encode_into_std_write(
                &signature_share,
                &mut signature_share_buffer,
                config::legacy(),
            )?;

            let recipient = Recipient {
                address: coordinator_address.clone(),
                amount: 0,
                pools: None,
                user_memo: None,
                memo_bytes: Some(to_arb_memo(&signature_share_buffer)),
            };
            let pczt = plan_transaction(
                network,
                connection,
                client,
                funding_account,
                ALL_POOLS,
                std::slice::from_ref(&recipient),
                false,
            )
            .await?;
            let pczt = sign_transaction(connection, funding_account, &pczt).await?;
            let txb = extract_transaction(&pczt).await?;
            let txid = send(client, height, &txb).await?;
            info!("Broadcasted signature share: {txid}");
        }
    }
    Ok(())
}

#[derive(Encode, Decode)]
pub struct Commitments {
    pub from_id: u16,
    pub commitments: Vec<Vec<u8>>,
}

#[derive(Encode, Decode)]
pub struct SigShare {
    pub from_id: u16,
    pub signature: Vec<u8>,
}
*/

pub fn to_arb_memo(pk1: &[u8]) -> Vec<u8> {
    let mut memo_bytes = vec![0xFF];
    memo_bytes.extend_from_slice(&pk1);
    memo_bytes
}
