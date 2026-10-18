//! Signing a planned transaction: with the wallet's own keys, or on the
//! Ledger app the account is paired with. Callers say which account signs;
//! this module decides who does it. Either way the result has every spend
//! signed, proved and finalised, ready to extract and broadcast.

use anyhow::{anyhow, Result};
use bip32::PrivateKey as _;
use orchard::keys::SpendAuthorizingKey;
use pczt::{
    roles::{
        prover::Prover, signer::Signer as PcztSigner, spend_finalizer::SpendFinalizer,
        updater::Updater,
    },
    Pczt,
};
use rand::rngs::SysRng;
use rand_core::UnwrapErr;
use secp256k1::SecretKey;
use sqlx::{Row, SqliteConnection};
use tracing::{info, span, Level};

use crate::{
    account::{derive_transparent_sk, get_orchard_sk, get_sapling_sk},
    api::{
        coin::Network,
        pay::{PcztPackage, SigningEvent},
    },
    db::{get_account_dindex, get_account_hw, select_account_transparent},
    keys::{sapling_pgk_for_scope, sapling_ssk_for_scope},
    ledger::{HwKind, LedgerHandle},
    pay::{
        error::Error,
        plan::{get_orchard_pk, get_sapling_prover, IRONWOOD_PK},
    },
    Sink,
};

/// Who signs for an account.
enum Signer {
    /// The wallet holds the spending keys.
    Software,
    /// The account is paired with a Ledger app, which signs on the device.
    Ledger(LedgerHandle),
}

impl Signer {
    fn for_hw(hw: u8) -> Self {
        if HwKind::from_hw(hw).is_ledger() {
            Signer::Ledger(LedgerHandle::from_hw(hw))
        } else {
            Signer::Software
        }
    }
}

/// Sign `package` for `account` with whichever signer the account uses.
/// A Ledger app reports its progress through `sink`; the software signer
/// is silent.
pub async fn sign<S>(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    package: PcztPackage,
    sink: &S,
) -> Result<PcztPackage>
where
    S: Sink<SigningEvent> + Sync,
{
    let hw = get_account_hw(connection, account).await?;
    match Signer::for_hw(hw) {
        Signer::Software => sign_transaction(connection, account, &package).await,
        Signer::Ledger(app) => {
            app.sign_pczt(*network, sink, connection, account, package)
                .await
        }
    }
}

/// Sign `pczt` with the account's spending keys, prove and finalise it.
async fn sign_transaction(
    connection: &mut SqliteConnection,
    account: u32,
    pczt: &PcztPackage,
) -> Result<PcztPackage> {
    let span = span!(Level::INFO, "transaction");

    let PcztPackage {
        pczt,
        n_spends,
        sapling_indices,
        orchard_indices,
        ironwood_indices,
        price,
        category,
        is_issuance,
        ..
    } = pczt;
    let pczt = Pczt::parse(pczt)
        .map_err(|error| anyhow!("failed to parse PCZT for signing: {error:?}"))?;
    let orchard_pk = get_orchard_pk(*pczt.global().consensus_branch_id())?;

    let dindex = get_account_dindex(connection, account).await?;
    let tkeys = select_account_transparent(connection, account, dindex).await?;
    let tsk = tkeys.xsk;
    let ssk = get_sapling_sk(connection, account).await?;
    let osk = get_orchard_sk(connection, account).await?;
    let osak = osk.map(|osk| SpendAuthorizingKey::from(&osk));

    // A viewing-key account can plan a transaction for an offline signer but
    // cannot sign it here: refuse before the proof keys are unwrapped below.
    if !sapling_indices.is_empty() && ssk.is_none() {
        return Err(Error::NoSigningKey.into());
    }
    if (!orchard_indices.is_empty() || !ironwood_indices.is_empty()) && osak.is_none() {
        return Err(Error::NoSigningKey.into());
    }

    let updater = Updater::new(pczt);
    let pgk = ssk.as_ref().map(|ssk| ssk.expsk().proof_generation_key());
    let internal_pgk = ssk
        .clone()
        .map(|ssk| sapling_ssk_for_scope(1u8, &ssk).map(|key| key.expsk().proof_generation_key()))
        .transpose()?;
    let updater = updater
        .update_sapling_with(|mut u| {
            for bundle_index in sapling_indices.iter() {
                let spend = &u.bundle().spends()[*bundle_index];
                let scope =
                    u32::from_le_bytes(spend.proprietary()["scope"].clone().try_into().unwrap());
                u.update_spend_with(*bundle_index, |mut u| {
                    u.set_proof_generation_key(sapling_pgk_for_scope(
                        scope,
                        pgk.clone().expect("proof_generation_key"),
                        internal_pgk.clone().expect("internal_proof_generation_key"),
                    ))
                    .unwrap();

                    Ok(())
                })
                .unwrap();
            }
            Ok(())
        })
        .unwrap();
    let pczt = updater.finish();
    info!("Updated");

    let mut signer = PcztSigner::new(pczt.clone()).unwrap();
    let tbundle = pczt.transparent();
    let sbundle = pczt.sapling();
    for index in 0..n_spends[0] {
        info!("signing transparent {index}");
        let inp = &tbundle.inputs()[index];
        let scope = u32::from_le_bytes(inp.proprietary()["scope"].clone().try_into().unwrap());
        let dindex = u32::from_le_bytes(inp.proprietary()["dindex"].clone().try_into().unwrap());
        // Check if "uncompressed" flag exists in proprietary, default to false (compressed)
        let uncompressed_flag = if let Some(val) = inp.proprietary().get("uncompressed") {
            if !val.is_empty() {
                val[0] != 0
            } else {
                info!(
                    "Invalid uncompressed flag length: {}, defaulting to compressed",
                    val.len()
                );
                false
            }
        } else {
            info!("No 'uncompressed' proprietary field found, defaulting to compressed");
            false
        };
        info!(
            "Signing transparent input {}: scope={}, dindex={}, uncompressed={}",
            index, scope, dindex, uncompressed_flag
        );

        // Get the signing key
        let sk = match tsk.as_ref() {
            // From the derivation path if we have the xsk
            Some(tsk) => {
                let sk = derive_transparent_sk(tsk, scope, dindex)?;
                SecretKey::from_bytes(&sk.try_into().unwrap()).ok()
            }
            // Or directly from the private key
            None => {
                let address = String::from_utf8(inp.proprietary()["address"].clone())?;
                sqlx::query(
                    "SELECT sk FROM transparent_address_accounts
                    WHERE account = ?1 AND address = ?2",
                )
                .bind(account)
                .bind(&address)
                .map(|r| {
                    // A watch-only key (xpub, bare public key) stores no secret:
                    // the column is NULL or empty, and the account cannot sign.
                    let sk: Option<Vec<u8>> = r.get(0);
                    sk.and_then(|sk| <[u8; 32]>::try_from(sk).ok())
                        .and_then(|sk| SecretKey::from_bytes(&sk).ok())
                })
                .fetch_optional(&mut *connection)
                .await?
                .flatten()
            }
        };
        let sk = sk.ok_or(Error::NoSigningKey)?;

        // Derive pubkey from secret key to check
        let derived_pubkey = secp256k1::PublicKey::from_secret_key(&sk);
        let derived_compressed = derived_pubkey.serialize();
        let derived_uncompressed = derived_pubkey.serialize_uncompressed();
        let hash_compressed = zcash_transparent::util::hash160::hash(&derived_compressed);
        let hash_uncompressed = zcash_transparent::util::hash160::hash(&derived_uncompressed);
        info!(
            "Derived pubkey (compressed): hash={}, len={}",
            hex::encode(hash_compressed),
            derived_compressed.len()
        );
        info!(
            "Derived pubkey (uncompressed): hash={}, len={}",
            hex::encode(hash_uncompressed),
            65
        );

        // Get the sighash and sign manually
        let sighash = signer.transparent_sighash(index).unwrap();
        let msg = secp256k1::Message::from_digest(sighash);
        let sig = secp256k1::ecdsa::sign(msg, &sk);

        // Append the signature - the pubkey will be retrieved from hash160_preimages
        info!("Appending signature for input {}", index);
        match signer.append_transparent_signature(index, sig) {
            Ok(_) => info!("Successfully appended signature"),
            Err(e) => info!("Failed to append signature: {:?}", e),
        }
    }
    for (index, bundle_index) in sapling_indices.iter().enumerate() {
        info!("signing sapling {index}");
        let spend = &sbundle.spends()[*bundle_index];
        let scope = u32::from_le_bytes(spend.proprietary()["scope"].clone().try_into().unwrap());
        let ssk = ssk
            .as_ref()
            .map(|ssk| sapling_ssk_for_scope(scope, ssk))
            .transpose()?;
        let Some(sk) = ssk.as_ref().map(|sk| sk.expsk().ask()) else {
            return Err(Error::NoSigningKey.into());
        };
        signer
            .sign_sapling(UnwrapErr(SysRng), *bundle_index, sk)
            .unwrap();
    }
    for (index, bundle_index) in orchard_indices.iter().enumerate() {
        info!("signing orchard {index}");
        let Some(osak) = osak.as_ref() else {
            return Err(Error::NoSigningKey.into());
        };
        signer
            .sign_orchard(UnwrapErr(SysRng), *bundle_index, osak)
            .map_err(|e| {
                anyhow!(
                    "failed to sign Orchard action {bundle_index} (selected spend {index}): {e:?}"
                )
            })?;
    }
    for (index, bundle_index) in ironwood_indices.iter().enumerate() {
        info!("signing ironwood {index}");
        let Some(osak) = osak.as_ref() else {
            return Err(Error::NoSigningKey.into());
        };
        signer
            .sign_ironwood(UnwrapErr(SysRng), *bundle_index, osak)
            .unwrap();
    }
    let pczt = signer.finish();

    span.in_scope(|| {
        info!("Adding Proofs to PCZT");
    });
    let sapling_prover = get_sapling_prover().await?;

    let pczt = Prover::new(pczt)
        .create_sapling_proofs(UnwrapErr(SysRng), sapling_prover, sapling_prover)
        .map_err(|error| anyhow!("failed to create Sapling proofs: {error:?}"))?
        .create_orchard_proof(UnwrapErr(SysRng), orchard_pk)
        .map_err(|error| anyhow!("failed to create Orchard proof: {error:?}"))?
        .create_ironwood_proof(UnwrapErr(SysRng), &IRONWOOD_PK)
        .map_err(|error| anyhow!("failed to create Ironwood proof: {error:?}"))?
        .finish();
    info!("Proved");

    let pczt = SpendFinalizer::new(pczt)
        .finalize_spends()
        .map_err(|error| anyhow!("failed to finalize PCZT spends: {error:?}"))?;
    info!("Spend Finalized");

    Ok(PcztPackage {
        pczt: pczt
            .serialize()
            .map_err(|error| anyhow!("failed to serialize signed PCZT: {error:?}"))?,
        n_spends: *n_spends,
        sapling_indices: sapling_indices.clone(),
        orchard_indices: orchard_indices.clone(),
        ironwood_indices: ironwood_indices.clone(),
        can_sign: true,
        can_broadcast: true,
        price: *price,
        category: *category,
        is_issuance: *is_issuance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_schema;
    use sqlx::Connection;

    fn kind(hw: u8) -> &'static str {
        match Signer::for_hw(hw) {
            Signer::Software => "software",
            Signer::Ledger(_) => "ledger",
        }
    }

    #[test]
    fn the_hardware_kind_picks_the_signer() {
        assert_eq!(kind(HwKind::Software as u8), "software");
        assert_eq!(kind(HwKind::Zondax as u8), "ledger");
        assert_eq!(kind(HwKind::Official as u8), "ledger");
    }

    async fn wallet(hw: u8) -> Result<SqliteConnection> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        sqlx::query(
            "INSERT INTO accounts(id_account, name, aindex, dindex, def_dindex, birth, position,
             use_internal, hidden, saved, hw) VALUES (1, 'a', 0, 0, 0, 0, 0, FALSE, FALSE, FALSE, ?)",
        )
        .bind(hw)
        .execute(&mut db)
        .await?;
        Ok(db)
    }

    /// The signer is observable through the seam by how it fails on an
    /// empty package: the software signer tries to parse it, a Ledger-kind
    /// account without Ledger support is refused before that.
    async fn failure(hw: u8) -> String {
        let mut db = wallet(hw).await.unwrap();
        let package = PcztPackage {
            pczt: vec![],
            n_spends: [0; 4],
            sapling_indices: vec![],
            orchard_indices: vec![],
            ironwood_indices: vec![],
            can_sign: true,
            can_broadcast: false,
            price: None,
            category: None,
            is_issuance: false,
        };
        sign(&Network::Main, &mut db, 1, package, &())
            .await
            .expect_err("an empty package cannot be signed")
            .to_string()
    }

    #[tokio::test]
    async fn a_software_account_signs_with_the_wallet_keys() {
        assert!(failure(HwKind::Software as u8).await.contains("parse PCZT"));
    }

    #[cfg(not(feature = "ledger"))]
    #[tokio::test]
    async fn a_ledger_account_signs_on_the_device() {
        assert!(failure(HwKind::Zondax as u8)
            .await
            .contains("signing not supported"));
        assert!(failure(HwKind::Official as u8)
            .await
            .contains("signing not supported"));
    }
}
