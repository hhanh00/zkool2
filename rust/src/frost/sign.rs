use std::{collections::BTreeMap};

use anyhow::{Context as _, Result};
use bincode::{
    config::{self, legacy},
    Decode, Encode,
};
use frost_rerandomized::{aggregate, sign, RandomizedParams};
use futures::StreamExt as _;
use halo2_proofs::pasta::Fq;
use pczt::{roles::{low_level_signer::Signer, prover::Prover, spend_finalizer::SpendFinalizer, tx_extractor::TransactionExtractor}, Pczt};
use rand_core::OsRng;
use reddsa::frost::redpallas::{
    frost::{
        keys::{KeyPackage, PublicKeyPackage},
        round1::{SigningCommitments, SigningNonces},
        round2::SignatureShare,
        SigningPackage,
    },
    round1::commit,
    Identifier, Randomizer,
};
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use tracing::info;
use zcash_primitives::transaction::{
    sighash::SignableInput, sighash_v5::v5_signature_hash, txid::TxIdDigester,
};
use zcash_protocol::{consensus::Network, memo::Memo};

use crate::{
    api::{
        frost::{FrostSignParams, SigningStatus},
        pay::PcztPackage, sync::SYNCING,
    }, frb_generated::StreamSink, frost::{
        db::{get_coordinator_broadcast_account, get_mailbox_account},
        dkg::{delete_frost_accounts, get_dkg_params, publish},
    }, pay::{plan::ORCHARD_PK, send}, Client
};

use super::{FrostSigMessage, P};

type CommitmentMap = BTreeMap<Identifier, SigningCommitments<P>>;
type SignatureMap = BTreeMap<Identifier, SignatureShare<P>>;

const COMMITMENT_PREFIX: &[u8] = b"CMT5";
const SIGPACKAGE_PREFIX: &[u8] = b"SPK4";
const SIGSHARE_PREFIX: &[u8] = b"SSH2";

pub async fn init_sign(
    connection: &SqlitePool,
    _account: u32,
    funding_account: u32,
    coordinator: u16,
    pczt: &PcztPackage,
) -> Result<()> {
    let pczt = bincode::encode_to_vec(pczt, config::legacy()).unwrap();
    sqlx::query("INSERT INTO props(key, value) VALUES ('frost_pczt', ?) ON CONFLICT DO NOTHING")
        .bind(&pczt)
        .execute(connection)
        .await?;
    let params = FrostSignParams {
        coordinator,
        funding_account,
    };
    let params = serde_json::to_string(&params).unwrap();
    sqlx::query(
        "INSERT INTO props(key, value) VALUES ('frost_sign_params', ?) ON CONFLICT DO NOTHING",
    )
    .bind(&params)
    .execute(connection)
    .await?;
    Ok(())
}

pub async fn do_sign(
    network: &Network,
    connection: &SqlitePool,
    account: u32,
    client: &mut Client,
    height: u32,
    status: StreamSink<SigningStatus>,
) -> Result<()> {
    let Some(pczt_pkg) = get_pczt(connection).await? else {
        return Ok(()); // No signing in progress
    };

    let guard = SYNCING.try_lock();
    if guard.is_err() {
        info!("Signing in progress");
        return Ok(());
    }

    let birth_height = height - 10000;
    let params = get_sign_params(connection).await?;
    let coordinator_address =
        get_coordinator_address(connection, account, params.coordinator).await?;
    let dkg_params = get_dkg_params(connection, account).await?;
    let (spkg, ppkg) = get_keys(connection, account).await?;
    let pczt = Pczt::parse(&pczt_pkg.pczt).expect("Failed to parse PCZT");
    let sighash = get_sighash(pczt.clone());
    let nsigs = pczt_pkg.orchard_indices.len() as u32;

    // Create a mailbox account if it doesn't exist
    let (mailbox_account, _mailbox_address) = get_mailbox_account(
        network,
        connection,
        account,
        params.coordinator,
        birth_height,
    )
    .await?;

    // Parse commitment memos and store them
    // commitments are privately received by the coordinator
    // the participants will not get anything
    process_memos(
        connection,
        account,
        mailbox_account,
        COMMITMENT_PREFIX,
        async move |connection: &SqlitePool, account, pkg: &FrostSigMessage| {
            sqlx::query("INSERT INTO frost_commitments(account, sighash, idx, from_id, commitment) VALUES (?, ?, ?, ?, ?) ON CONFLICT DO NOTHING")
                .bind(account)
                .bind(pkg.sighash.as_slice())
                .bind(pkg.idx)
                .bind(pkg.from_id)
                .bind(&pkg.data)
                .execute(connection)
                .await?;
            Ok(())
        },
    ).await?;

    let (broadcast_account, broadcast_address) =
        get_coordinator_broadcast_account(network, connection, account, birth_height).await?;

    info!("Processing commitments for account {}", account);
    let commitments_vec = loop {
        let commitments_vec = get_commitments(connection, account, &sighash, nsigs).await?;
        if !commitments_vec[0].is_empty() {
            break commitments_vec; // we have published our commitments
        }
        let mut tx = connection.begin().await?;
        let mut recipients = vec![];
        for idx in 0..nsigs {
            let (nonces, commitments) = commit(spkg.signing_share(), &mut OsRng);
            // store nonces and commitments
            // nonces go to the frost_signatures table
            let nonces = nonces.serialize()?;
            sqlx::query(
                "INSERT INTO frost_signatures(account, sighash, idx, nonce) VALUES (?, ?, ?, ?)",
            )
            .bind(account)
            .bind(&sighash)
            .bind(idx)
            .bind(&nonces)
            .execute(&mut *tx)
            .await?;
            // commitments go to the frost_commitments table
            let commitments = commitments.serialize()?;
            sqlx::query("INSERT INTO frost_commitments(account, sighash, idx, from_id, commitment) VALUES (?, ?, ?, ?, ?)")
                .bind(account)
                .bind(&sighash)
                .bind(idx)
                .bind(dkg_params.id)
                .bind(&commitments)
                .execute(&mut *tx)
                .await?;
            let message = FrostSigMessage {
                sighash: sighash.clone().try_into().unwrap(),
                from_id: dkg_params.id as u16,
                idx,
                data: commitments,
            };
            let memo_bytes = message.encode_with_prefix(COMMITMENT_PREFIX)?;
            recipients.push((coordinator_address.as_str(), memo_bytes));
        }
        // send the commitments to the coordinator
        // we send all the commitments in one zcash transaction
        // the coordinator does not need to send a message to itself,
        //   (the commitments are already in the database)
        if dkg_params.id as u16 != params.coordinator {
            status.add(SigningStatus::SendingCommitment).map_err(anyhow::Error::msg)?;
            let txid = publish(
                network,
                connection,
                params.funding_account,
                client,
                height,
                &recipients,
            )
            .await?;
            info!("Published commitment transaction: {}", txid);
        }
        tx.commit().await?;
    };
    info!("Commitments: {:?}", commitments_vec);

    // process sigpackages
    // there is one sigpackage per signature
    //
    // this is for the participants other than the coordinator
    // the coordinator will produce the sigpackages
    process_memos(
        connection,
        account,
        broadcast_account,
        SIGPACKAGE_PREFIX,
        async move |connection: &SqlitePool, account, pkg: &FrostSigMessage| {
            let randomized_sigpackage: RandomizedSigPackage =
                bincode::decode_from_slice(&pkg.data, config::legacy()).unwrap().0;
            sqlx::query("UPDATE frost_signatures SET sigpackage = ?1, randomizer = ?2 WHERE account = ?3 AND sighash = ?4 AND idx = ?5")
                .bind(&randomized_sigpackage.sigpackage)
                .bind(&randomized_sigpackage.randomizer)
                .bind(account)
                .bind(pkg.sighash.as_slice())
                .bind(pkg.idx)
                .execute(connection)
                .await?;
            Ok(())
        },
    ).await?;

    let sigpackages = loop {
        let sigpackages = get_sigpackages(connection, account, &sighash).await?;
        if sigpackages.len() == nsigs as usize {
            break sigpackages; // we have all sigpackages
        }

        // we are not the coordinator, and we haven't received all the sigpackages
        if dkg_params.id as u16 != params.coordinator {
            info!("Waiting for sigpackages");
            status.add(SigningStatus::WaitingForSigningPackage).map_err(anyhow::Error::msg)?;
            return Ok(());
        }

        // we are the coordinator, let's try to make the sigpackages
        let mut tx = connection.begin().await?;
        let mut recipients = vec![];

        for (idx, c) in commitments_vec.iter().enumerate() {
            // each sigpackage needs t commitments
            // if we don't have them, bail out
            if c.len() != dkg_params.t as usize {
                info!(
                    "Not enough commitments for input {idx}: {}/{}",
                    c.len(),
                    dkg_params.t
                );
                status.add(SigningStatus::WaitingForCommitments).map_err(anyhow::Error::msg)?;
                return Ok(());
            }
            // build the sigpackage for this input and store it
            // note that it will be kept in the database only if we succesfully sent it out
            // because of the db transaction
            let sigpackage = SigningPackage::new(c.clone(), &sighash);

            // get the randomizer from the pczt
            let action_index = pczt_pkg.orchard_indices[idx as usize];
            let signer = Signer::new(pczt.clone());
            let mut alpha = Fq::zero();
            signer
                .sign_orchard_with(|_pczt, bundle, _| {
                    let a = &bundle.actions()[action_index];
                    let spend = a.spend();
                    alpha = spend.alpha().expect("Failed to get alpha");
                    Ok::<_, orchard::pczt::ParseError>(())
                })
                .unwrap();

            let randomizer = Randomizer::from_scalar(alpha);
            let sigpackage = sigpackage.serialize()?;
            sqlx::query(
                "UPDATE frost_signatures SET sigpackage = ?1, randomizer = ?2 WHERE account = ?3 AND sighash = ?4 AND idx = ?5",
            )
            .bind(&sigpackage)
            .bind(&randomizer.serialize())
            .bind(account)
            .bind(&sighash)
            .bind(idx as u32)
            .execute(&mut *tx)
            .await?;

            // build the randomized package
            let randomized_sigpackage = RandomizedSigPackage {
                sigpackage: sigpackage.clone(),
                randomizer: randomizer.serialize(),
            };
            let randomized_sigpackage =
                bincode::encode_to_vec(&randomized_sigpackage, config::legacy())?;

            let message = FrostSigMessage {
                sighash: sighash.clone().try_into().unwrap(),
                from_id: params.coordinator as u16,
                idx: idx as u32,
                data: randomized_sigpackage,
            };
            let memo_bytes = message.encode_with_prefix(SIGPACKAGE_PREFIX)?;
            // broadcast the sigpackage to all participants
            recipients.push((broadcast_address.as_str(), memo_bytes));
        }
        // we send all the sigpackages in one zcash transaction
        // with one output/memo per input/signature needed
        status.add(SigningStatus::SendingSigningPackage).map_err(anyhow::Error::msg)?;
        let txid = publish(
            network,
            connection,
            params.funding_account,
            client,
            height,
            &recipients,
        )
        .await?;
        info!("Published sigpackages transaction: {}", txid);
        // we got all the sigshares, commit them
        tx.commit().await?;
    };

    info!("Completed sigpackages: {:?}", sigpackages);

    let nonces = get_nonces(connection, account, &sighash).await?;

    let sigshares = loop {
        // get the sigshares from the database
        // if we have them all, we have already signed the sigpackages and we are done
        let sigshares = get_sigshares(connection, account, &sighash).await?;
        if !sigshares.is_empty() {
            break sigshares; // we have all sigshares, it's all or none
        }

        // same as above
        // we start a database transaction to make sure we don't store
        // the sigshares if we fail to send them
        let mut tx = connection.begin().await?;
        let mut recipients = vec![];
        for (idx, ((signing_package, randomizer), nonces)) in
            sigpackages.iter().zip(nonces.iter()).enumerate()
        {
            let signature_share = sign(&signing_package, nonces, &spkg, randomizer.clone())
                .context("Failed to sign")?;
            let signature_share = signature_share.serialize();

            sqlx::query(
                "UPDATE frost_signatures SET sigshare = ?1 WHERE account = ?2 AND sighash = ?3 AND idx = ?4",
            )
            .bind(&signature_share)
            .bind(account)
            .bind(&sighash)
            .bind(idx as u32)
            .execute(&mut *tx)
            .await?;

            let message = FrostSigMessage {
                sighash: sighash.clone().try_into().unwrap(),
                from_id: dkg_params.id as u16,
                idx: idx as u32,
                data: signature_share,
            };
            let memo_bytes = message.encode_with_prefix(SIGSHARE_PREFIX)?;
            // send the sigshare to the coordinator
            recipients.push((coordinator_address.as_str(), memo_bytes));
        }

        if dkg_params.id as u16 != params.coordinator {
            status.add(SigningStatus::SendingSignatureShare).map_err(anyhow::Error::msg)?;
            let txid = publish(
                network,
                connection,
                params.funding_account,
                client,
                height,
                &recipients,
            )
            .await?;

            info!("Published sigshares transaction: {}", txid);
        }
        tx.commit().await?;
    };

    info!("Signature shares: {:?}", sigshares);

    // copy our own sigshares to the commitments table
    for idx in 0..nsigs {
        sqlx::query(
            "UPDATE frost_commitments SET sigshare =
            (SELECT sigshare FROM frost_signatures WHERE account = ?1 AND sighash = ?2 AND idx = ?3)
            WHERE account = ?1 AND sighash = ?2 AND idx = ?3 AND from_id = ?4",
        )
        .bind(account)
        .bind(&sighash)
        .bind(idx as u32)
        .bind(dkg_params.id)
        .execute(connection)
        .await?;
    }

    // add sigshares from the mailbox
    process_memos(
        connection,
        account,
        mailbox_account,
        SIGSHARE_PREFIX,
        async move |connection: &SqlitePool, account, pkg: &FrostSigMessage| {
            sqlx::query("UPDATE frost_commitments SET sigshare = ?1 WHERE account = ?2 AND sighash = ?3 AND idx = ?4 AND from_id = ?5")
                .bind(&pkg.data)
                .bind(account)
                .bind(pkg.sighash.as_slice())
                .bind(pkg.idx)
                .bind(pkg.from_id)
                .execute(connection)
                .await?;
            Ok(())
        },
    ).await?;

    // final step: aggregate the sigshares
    // this is only done by the coordinator
    if dkg_params.id as u16 == params.coordinator {
        let mut tx = connection.begin().await?;
        let sigsharess = get_all_sigshares(connection, account, &sighash, nsigs).await?;
        let mut signatures = vec![];
        for (idx, (sigshares, (sigpackage, randomizer))) in
            sigsharess.iter().zip(sigpackages.iter()).enumerate()
        {
            if sigshares.len() != dkg_params.t as usize {
                info!(
                    "Not enough sigshares for input {}: {}/{}",
                    idx,
                    sigshares.len(),
                    dkg_params.t
                );
                status.add(SigningStatus::WaitingForSignatureShares).map_err(anyhow::Error::msg)?;
                return Ok(());
            }
            let randomized_params =
                RandomizedParams::from_randomizer(ppkg.verifying_key(), randomizer.clone());
            let signature = aggregate(sigpackage, sigshares, &ppkg, &randomized_params)?;
            let signature = signature.serialize()?;
            let signature_bytes: [u8; 64] = signature.clone().try_into().unwrap();
            let orchard_signature =
                orchard::primitives::redpallas::Signature::from(signature_bytes);
            signatures.push(orchard_signature);

            sqlx::query("UPDATE frost_signatures SET signature = ?1 WHERE account = ?2 AND sighash = ?3 AND idx = ?4")
            .bind(&signature)
            .bind(account)
            .bind(&sighash)
            .bind(idx as u32)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        info!("Signature completed");

        status.add(SigningStatus::PreparingTransaction).map_err(anyhow::Error::msg)?;
        let signer = Signer::new(pczt);
        let signer = signer
            .sign_orchard_with(|_pczt, bundle, _| {
                for (idx, signature) in signatures.into_iter().enumerate() {
                    let action_index = pczt_pkg.orchard_indices[idx as usize];
                    let action = &mut bundle.actions_mut()[action_index];
                    action.spend_auth_sig(signature);
                    // How do we update the spend_auth_sig?
                    // a[0].spend().spend_auth_sig = Some(signature.clone());
                }
                Ok::<_, orchard::pczt::ParseError>(())
            })
            .unwrap();
        let pczt = signer.finish();
        info!("Signed");

        let pczt = Prover::new(pczt)
            .create_orchard_proof(&ORCHARD_PK)
            .unwrap()
            .finish();
        info!("Proved");

        let pczt = SpendFinalizer::new(pczt).finalize_spends().unwrap();
        info!("Spend Finalized");

        let tx_extractor = TransactionExtractor::new(pczt);
        let tx = tx_extractor.extract().map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let mut tx_bytes = vec![];
        tx.write(&mut tx_bytes).unwrap();
        info!("Transaction Len: {}", tx_bytes.len());

        status.add(SigningStatus::SendingTransaction).map_err(anyhow::Error::msg)?;
        let txid = send(client, height, &tx_bytes).await?;
        info!("Transaction sent: {}", txid);
        status.add(SigningStatus::TransactionSent(txid)).map_err(anyhow::Error::msg)?;
    }

    sqlx::query("DELETE FROM props WHERE key LIKE 'frost_%'")
        .execute(connection)
        .await?;
    delete_frost_accounts(connection).await?;

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

async fn get_pczt(connection: &SqlitePool) -> Result<Option<PcztPackage>> {
    let pczt = sqlx::query("SELECT value FROM props WHERE key = 'frost_pczt'")
        .map(|row: SqliteRow| {
            let value: Vec<u8> = row.get(0);
            let pczt: PcztPackage = bincode::decode_from_slice(&value, legacy()).unwrap().0;
            pczt
        })
        .fetch_optional(connection)
        .await?;
    Ok(pczt)
}

async fn get_sign_params(connection: &SqlitePool) -> Result<FrostSignParams> {
    let params = sqlx::query(
        "SELECT value FROM props WHERE
        key = 'frost_sign_params'",
    )
    .map(|row: SqliteRow| {
        let value: String = row.get(0);
        let frost: FrostSignParams = serde_json::from_str(&value).unwrap();
        frost
    })
    .fetch_one(connection)
    .await?;
    Ok(params)
}

async fn get_coordinator_address(
    connection: &SqlitePool,
    account: u32,
    coordinator: u16,
) -> Result<String> {
    let (address,) = sqlx::query_as::<_, (String,)>(
        "SELECT data FROM dkg_packages WHERE
        account = ? AND round = 0 AND public = 1 AND from_id = ?",
    )
    .bind(account)
    .bind(coordinator)
    .fetch_one(connection)
    .await?;
    Ok(address)
}

async fn get_keys(
    connection: &SqlitePool,
    account: u32,
) -> Result<(KeyPackage<P>, PublicKeyPackage<P>)> {
    let (data,) = sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT data FROM dkg_packages WHERE account = ? AND public = 0 AND round = 3",
    )
    .bind(account)
    .fetch_one(connection)
    .await?;
    let spkg = KeyPackage::<P>::deserialize(&data)?;

    let (data,) = sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT data FROM dkg_packages WHERE account = ? AND public = 1 AND round = 3",
    )
    .bind(account)
    .fetch_one(connection)
    .await?;
    let ppkg = PublicKeyPackage::<P>::deserialize(&data)?;

    Ok((spkg, ppkg))
}

async fn process_memos(
    connection: &SqlitePool,
    account: u32,
    mailbox_account: u32,
    prefix: &[u8],
    fn_store: impl AsyncFn(&SqlitePool, u32, &FrostSigMessage) -> Result<()>,
) -> Result<()> {
    let mut pkgs = sqlx::query("SELECT memo_bytes FROM memos WHERE account = ?")
        .bind(mailbox_account)
        .map(|row: SqliteRow| {
            let memo_bytes: Vec<u8> = row.get(0);
            let memo = Memo::from_bytes(&memo_bytes);
            if let Ok(memo) = memo {
                match memo {
                    Memo::Arbitrary(pkg_bytes) => {
                        if pkg_bytes.len() < 4 || pkg_bytes[0..4] != *prefix {
                            return None;
                        }
                        if let Ok((pkg, _)) = bincode::decode_from_slice::<FrostSigMessage, _>(
                            &pkg_bytes[4..],
                            config::legacy(),
                        )
                        .context("Failed to decode FrostMessage")
                        {
                            return Some(pkg);
                        }
                    }
                    _ => (),
                }
            }
            None
        })
        .fetch(connection);
    while let Some(pkg) = pkgs.next().await {
        if let Some(pkg) = pkg? {
            fn_store(connection, account, &pkg).await?;
        }
    }

    Ok(())
}

async fn get_nonces(
    connection: &SqlitePool,
    account: u32,
    sighash: &[u8],
) -> Result<Vec<SigningNonces<P>>> {
    let rs = sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT nonce FROM frost_signatures WHERE account = ? AND sighash = ?
        ORDER BY idx",
    )
    .bind(account)
    .bind(sighash)
    .fetch_all(connection)
    .await?;
    let nonces = rs
        .into_iter()
        .map(|(n,)| SigningNonces::<P>::deserialize(&n).expect("Failed to deserialize nonce"))
        .collect::<Vec<_>>();

    Ok(nonces)
}

async fn get_commitments(
    connection: &SqlitePool,
    account: u32,
    sighash: &[u8],
    nsigs: u32,
) -> Result<Vec<CommitmentMap>> {
    let mut commitments_maps = vec![];
    for i in 0..nsigs {
        let mut commitments_map = BTreeMap::<Identifier, SigningCommitments<P>>::new();
        let commitments = sqlx::query(
            "SELECT from_id, commitment FROM frost_commitments WHERE account = ? AND sighash = ? AND idx = ?"
        )
        .bind(account)
        .bind(sighash)
        .bind(i)
        .map(|row: SqliteRow| {
            let from_id: u16 = row.get(0);
            let commitment: Vec<u8> = row.get(1);
            (from_id, commitment)
        })
        .fetch_all(connection)
        .await?;
        info!(
            "Found {} commitments for sighash {}",
            commitments.len(),
            hex::encode(sighash)
        );

        for (from_id, commitment) in commitments {
            commitments_map.insert(
                from_id.try_into().unwrap(),
                SigningCommitments::<P>::deserialize(&commitment).unwrap(),
            );
        }
        commitments_maps.push(commitments_map);
    }

    Ok(commitments_maps)
}

async fn get_sigpackages(
    connection: &SqlitePool,
    account: u32,
    sighash: &[u8],
) -> Result<Vec<(SigningPackage<P>, Randomizer)>> {
    let randomized_sigpackages =
        sqlx::query("SELECT sigpackage, randomizer FROM frost_signatures WHERE account = ? AND sighash = ? AND sigpackage IS NOT NULL")
            .bind(account)
            .bind(sighash)
            .map(|row| {
                let sigpackage: Vec<u8> = row.get(0);
                let randomizer: Vec<u8> = row.get(1);
                let sigpackage = SigningPackage::<P>::deserialize(&sigpackage).unwrap();
                let randomizer = Randomizer::deserialize(&randomizer).unwrap();
                (sigpackage, randomizer)
            })
            .fetch_all(connection)
            .await?;

    Ok(randomized_sigpackages)
}

async fn get_sigshares(
    connection: &SqlitePool,
    account: u32,
    sighash: &[u8],
) -> Result<Vec<SignatureShare<P>>> {
    let sigshares = sqlx::query("SELECT sigshare FROM frost_signatures WHERE account = ? AND sighash = ? AND sigshare IS NOT NULL")
        .bind(account)
        .bind(sighash)
        .map(|row| {
            let sigshare: Vec<u8> = row.get(0);
            SignatureShare::<P>::deserialize(&sigshare).unwrap()
        })
        .fetch_all(connection)
        .await?;

    Ok(sigshares)
}

async fn get_all_sigshares(
    connection: &SqlitePool,
    account: u32,
    sighash: &[u8],
    nsigs: u32,
) -> Result<Vec<SignatureMap>> {
    let mut sigshare_maps = vec![];
    for i in 0..nsigs {
        let mut map = SignatureMap::new();
        let sigshares = sqlx::query("SELECT from_id, sigshare FROM frost_commitments WHERE account = ?1 AND sighash = ?2 AND idx = ?3 AND sigshare IS NOT NULL")
            .bind(account)
            .bind(sighash)
            .bind(i)
            .map(|row| {
                let from_id: u16 = row.get(0);
                let id: Identifier = from_id.try_into().unwrap();
                let sigshare: Vec<u8> = row.get(1);
                let sigshare = SignatureShare::<P>::deserialize(&sigshare).unwrap();
                (id, sigshare)
            })
            .fetch_all(connection)
            .await?;
        for (id, sigshare) in sigshares {
            map.insert(id, sigshare);
        }
        sigshare_maps.push(map);
    }
    Ok(sigshare_maps)
}

#[derive(Encode, Decode)]
struct RandomizedSigPackage {
    sigpackage: Vec<u8>,
    randomizer: Vec<u8>,
}

pub async fn is_signing_in_progress(connection: &SqlitePool) -> Result<bool> {
    let exists = sqlx::query_as::<_, (bool, )>("SELECT TRUE FROM props WHERE key = 'frost_pczt'")
    .fetch_optional(connection).await?;

    Ok(exists.is_some())
}
