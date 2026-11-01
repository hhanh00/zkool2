//! Signing a planned transaction: with the wallet's own keys, or on the
//! Ledger app the account is paired with. Callers say which account signs;
//! this module decides who does it. Either way the result has every spend
//! signed, proved and finalised, ready to extract and broadcast.

use std::collections::{BTreeMap, HashSet};

use anyhow::{anyhow, bail, Result};
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

    let package = pczt;
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
    } = package;
    let pczt = Pczt::parse(pczt)
        .map_err(|error| anyhow!("failed to parse PCZT for signing: {error:?}"))?;
    validate_package(package, &pczt)?;
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

    // The scope of each Sapling spend, read from the validated package
    // before the updater takes the PCZT.
    let sapling_scopes = sapling_indices
        .iter()
        .map(|bundle_index| {
            scope_field(
                pczt.sapling().spends()[*bundle_index].proprietary(),
                "Sapling spend",
                *bundle_index,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let sapling_keys = if sapling_indices.is_empty() {
        None
    } else {
        let ssk = ssk.as_ref().ok_or(Error::NoSigningKey)?;
        Some((
            ssk.expsk().proof_generation_key(),
            sapling_ssk_for_scope(1u8, ssk)?
                .expsk()
                .proof_generation_key(),
        ))
    };
    let pczt = match sapling_keys {
        Some((pgk, internal_pgk)) => Updater::new(pczt)
            .update_sapling_with(|mut u| {
                for (bundle_index, scope) in sapling_indices.iter().zip(&sapling_scopes) {
                    u.update_spend_with(*bundle_index, |mut u| {
                        u.set_proof_generation_key(sapling_pgk_for_scope(
                            *scope,
                            pgk.clone(),
                            internal_pgk.clone(),
                        ))?;
                        Ok(())
                    })?;
                }
                Ok(())
            })
            .map_err(|error| anyhow!("failed to set the Sapling proof keys: {error:?}"))?
            .finish(),
        None => pczt,
    };
    info!("Updated");

    let mut signer = PcztSigner::new(pczt.clone())
        .map_err(|error| anyhow!("failed to start the PCZT signer: {error:?}"))?;
    let tbundle = pczt.transparent();
    for index in 0..n_spends[0] {
        info!("signing transparent {index}");
        let inp = &tbundle.inputs()[index];
        let scope = scope_field(inp.proprietary(), "transparent input", index)?;
        let dindex = u32_field(inp.proprietary(), "dindex", "transparent input", index)?;
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
                <[u8; 32]>::try_from(sk)
                    .ok()
                    .and_then(|sk| SecretKey::from_bytes(&sk).ok())
            }
            // Or directly from the private key
            None => {
                let address = address_field(inp.proprietary(), index)?;
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

        // Sign the sighash ourselves and append the signature; the signer
        // recovers the pubkey from the input's hash160 preimage.
        let sighash = signer
            .transparent_sighash(index)
            .map_err(|e| anyhow!("failed to compute the sighash of transparent input {index}: {e:?}"))?;
        let msg = secp256k1::Message::from_digest(sighash);
        let sig = secp256k1::ecdsa::sign(msg, &sk);
        signer
            .append_transparent_signature(index, sig)
            .map_err(|e| anyhow!("failed to sign transparent input {index}: {e:?}"))?;
    }
    for (index, bundle_index) in sapling_indices.iter().enumerate() {
        info!("signing sapling {index}");
        let scope = sapling_scopes[index];
        let ssk = ssk
            .as_ref()
            .map(|ssk| sapling_ssk_for_scope(scope, ssk))
            .transpose()?;
        let Some(sk) = ssk.as_ref().map(|sk| sk.expsk().ask()) else {
            return Err(Error::NoSigningKey.into());
        };
        signer
            .sign_sapling(UnwrapErr(SysRng), *bundle_index, sk)
            .map_err(|e| {
                anyhow!("failed to sign Sapling spend {bundle_index} (selected spend {index}): {e:?}")
            })?;
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
            .map_err(|e| {
                anyhow!("failed to sign Ironwood action {bundle_index} (selected spend {index}): {e:?}")
            })?;
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

/// Check a package's own fields against the PCZT it carries, before any
/// signer trusts them (#1480). The planner always writes them consistently;
/// a package loaded from a file or an animated QR, posted to zkool_graphql
/// or received from a FROST coordinator may not. Every mismatch is an error
/// naming the field, so a hand-built package fails instead of panicking.
pub fn validate_package(package: &PcztPackage, pczt: &Pczt) -> Result<()> {
    let inputs = pczt.transparent().inputs();
    if package.n_spends[0] > inputs.len() {
        bail!(
            "the package names {} transparent spends but the PCZT has {} inputs",
            package.n_spends[0],
            inputs.len()
        );
    }
    check_indices(
        "sapling_indices",
        &package.sapling_indices,
        pczt.sapling().spends().len(),
        Some(package.n_spends[1]),
    )?;
    check_indices(
        "orchard_indices",
        &package.orchard_indices,
        pczt.orchard().actions().len(),
        None,
    )?;
    check_indices(
        "ironwood_indices",
        &package.ironwood_indices,
        pczt.ironwood().actions().len(),
        Some(package.n_spends[3]),
    )?;
    for (index, input) in inputs.iter().take(package.n_spends[0]).enumerate() {
        scope_field(input.proprietary(), "transparent input", index)?;
        let dindex = u32_field(input.proprietary(), "dindex", "transparent input", index)?;
        if dindex & (1 << 31) != 0 {
            bail!("transparent input {index}: child index {dindex} is hardened");
        }
        address_field(input.proprietary(), index)?;
    }
    for bundle_index in &package.sapling_indices {
        scope_field(
            pczt.sapling().spends()[*bundle_index].proprietary(),
            "Sapling spend",
            *bundle_index,
        )?;
    }
    Ok(())
}

/// Every index names an entry of its bundle, once; `count` is what the
/// package's spend count says there should be.
fn check_indices(name: &str, indices: &[usize], len: usize, count: Option<usize>) -> Result<()> {
    if let Some(count) = count {
        if indices.len() != count {
            bail!("{name} has {} entries but the package counts {count} spends", indices.len());
        }
    }
    let mut seen = HashSet::new();
    for &index in indices {
        if index >= len {
            bail!("{name} names entry {index} but the PCZT bundle has {len}");
        }
        if !seen.insert(index) {
            bail!("{name} names entry {index} twice");
        }
    }
    Ok(())
}

/// A 4-byte little-endian proprietary field.
fn u32_field(
    proprietary: &BTreeMap<String, Vec<u8>>,
    field: &str,
    what: &str,
    index: usize,
) -> Result<u32> {
    let bytes = proprietary
        .get(field)
        .ok_or_else(|| anyhow!("{what} {index} has no `{field}` field"))?;
    let bytes: [u8; 4] = bytes.as_slice().try_into().map_err(|_| {
        anyhow!("{what} {index}: `{field}` is {} bytes, expected 4", bytes.len())
    })?;
    Ok(u32::from_le_bytes(bytes))
}

/// The `scope` field: 0 (external) or 1 (internal).
fn scope_field(proprietary: &BTreeMap<String, Vec<u8>>, what: &str, index: usize) -> Result<u32> {
    let scope = u32_field(proprietary, "scope", what, index)?;
    if scope > 1 {
        bail!("{what} {index}: scope {scope} is neither external (0) nor internal (1)");
    }
    Ok(scope)
}

/// The `address` field of a transparent input, as the encoded address.
fn address_field(proprietary: &BTreeMap<String, Vec<u8>>, index: usize) -> Result<String> {
    let bytes = proprietary
        .get("address")
        .filter(|bytes| !bytes.is_empty())
        .ok_or_else(|| anyhow!("transparent input {index} has no `address` field"))?;
    String::from_utf8(bytes.clone())
        .map_err(|_| anyhow!("transparent input {index}: `address` is not UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_schema;
    use sqlx::Connection;

    mod validation {
        use super::super::*;
        use crate::pay::plan::tests::assembly::transparent_package;

        fn parsed(package: &PcztPackage) -> Pczt {
            Pczt::parse(&package.pczt).unwrap()
        }

        fn error(package: &PcztPackage) -> String {
            validate_package(package, &parsed(package))
                .expect_err("the package should be refused")
                .to_string()
        }

        /// The same package with one transparent proprietary field rewritten.
        fn with_field(package: &PcztPackage, field: &str, value: Vec<u8>) -> PcztPackage {
            let pczt = Updater::new(parsed(package))
                .update_transparent_with(|mut u| {
                    u.update_input_with(0, |mut u| {
                        u.set_proprietary(field.to_string(), value);
                        Ok(())
                    })
                })
                .unwrap()
                .finish();
            PcztPackage {
                pczt: pczt.serialize().unwrap(),
                ..package.clone()
            }
        }

        #[test]
        fn a_planner_built_package_is_valid() {
            let package = transparent_package();
            validate_package(&package, &parsed(&package)).unwrap();
        }

        #[test]
        fn a_spend_count_above_the_inputs_is_refused() {
            let mut package = transparent_package();
            package.n_spends[0] = 2;
            assert!(error(&package).contains("2 transparent spends"), "{}", error(&package));
        }

        #[test]
        fn an_index_past_its_bundle_is_refused() {
            let mut package = transparent_package();
            package.sapling_indices = vec![0];
            package.n_spends[1] = 1;
            assert!(error(&package).contains("sapling_indices names entry 0"), "{}", error(&package));
        }

        #[test]
        fn an_index_count_that_disagrees_with_the_spend_count_is_refused() {
            let mut package = transparent_package();
            package.n_spends[3] = 1;
            assert!(error(&package).contains("ironwood_indices has 0 entries"), "{}", error(&package));
        }

        #[test]
        fn a_duplicate_index_is_refused() {
            let err = check_indices("orchard_indices", &[1, 1], 3, None).unwrap_err();
            assert!(err.to_string().contains("entry 1 twice"), "{err}");
            check_indices("orchard_indices", &[0, 2], 3, None).unwrap();
        }

        #[test]
        fn a_scope_other_than_external_or_internal_is_refused() {
            let package = with_field(&transparent_package(), "scope", 2u32.to_le_bytes().to_vec());
            assert!(error(&package).contains("scope 2"), "{}", error(&package));
        }

        #[test]
        fn a_hardened_child_index_is_refused() {
            let package = with_field(
                &transparent_package(),
                "dindex",
                (1u32 << 31 | 3).to_le_bytes().to_vec(),
            );
            assert!(error(&package).contains("is hardened"), "{}", error(&package));
        }

        #[test]
        fn a_malformed_field_is_refused() {
            let package = with_field(&transparent_package(), "dindex", vec![1, 2]);
            assert!(error(&package).contains("`dindex` is 2 bytes"), "{}", error(&package));
            let package = with_field(&transparent_package(), "address", vec![]);
            assert!(error(&package).contains("no `address` field"), "{}", error(&package));
            let empty = BTreeMap::new();
            let err = u32_field(&empty, "scope", "transparent input", 0).unwrap_err();
            assert!(err.to_string().contains("no `scope` field"), "{err}");
        }

        #[tokio::test]
        async fn unparseable_bytes_fail_extraction_instead_of_panicking() {
            let package = PcztPackage {
                pczt: vec![1, 2, 3],
                ..transparent_package()
            };
            let err = crate::pay::plan::extract_transaction(&package)
                .await
                .unwrap_err();
            assert!(err.to_string().contains("parse"), "{err}");
        }
    }

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
