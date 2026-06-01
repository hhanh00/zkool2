use std::sync::LazyLock;

use anyhow::{anyhow, Result};
use bip32::PrivateKey;
use bip39::Mnemonic;
use nonempty::NonEmpty;
use orchard::{
    circuit::ProvingKey,
    flavor::OrchardZSA,
    issuance::{
        auth::{IssueAuthKey, ZSASchnorr},
        compute_asset_desc_hash,
    },
    keys::SpendAuthorizingKey,
    note::AssetBase,
    value::NoteValue,
};
use pczt::{
    roles::{
        creator::Creator, io_finalizer::IoFinalizer, issuer::Issuer, prover::Prover,
        signer::Signer, spend_finalizer::SpendFinalizer,
        tx_extractor::TransactionExtractor, updater::Updater,
    },
};
use rand_core::OsRng;
use ripemd::Ripemd160;
use secp256k1::{PublicKey, SecretKey};
use sha2::{Digest, Sha256};
use sqlx::{sqlite::SqliteRow, Row};
use tracing::{debug, info};
use zcash_primitives::transaction::{
    builder::{BuildConfig, Builder},
    fees::zip317::FeeRule,
    zsa_builder::ZsaBuilder,
};
use zcash_protocol::{
    consensus::{BlockHeight, NetworkType, Parameters, ZIP212_GRACE_PERIOD},
    memo::MemoBytes,
    value::Zatoshis,
};
use zcash_transparent::{
    address::TransparentAddress,
    builder::{SpendInfo, TransparentInputInfo},
    bundle::{OutPoint, TxOut},
    pczt::Bip32Derivation,
};

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

use crate::{
    account::{derive_transparent_sk, get_account_seed, get_orchard_note, get_orchard_sk, get_orchard_vk},
    api::coin::Coin,
    db::{get_account_dindex, select_account_transparent},
    pay::plan::fetch_unspent_notes_grouped_by_pool,
    warp::hasher::{empty_roots, OrchardHasher, SaplingHasher},
};

/// Proving key for OrchardZSA (needed for ZSA-enabled proofs).
pub static ORCHARD_ZSA_PK: LazyLock<ProvingKey> = LazyLock::new(ProvingKey::build::<OrchardZSA>);

/// Issue a new ZSA (Zcash Shielded Asset) via the PCZT workflow.
///
/// This creates, signs, and extracts a full issuance transaction. The issuance
/// key is derived from the wallet's BIP-39 seed (ZIP-32 purpose 227).
///
/// # Parameters
/// - `asset_name`: Human-readable asset description (e.g. "WETH"). Must be non-empty UTF-8.
/// - `amount`: Raw units to issue (passed to `NoteValue::from_raw`).
/// - `first_issuance`: Whether this is the first issuance of this asset — adds a zero-value
///   reference note per ZIP-227.
/// - `finalize`: Whether to finalize the asset, preventing any future issuance of this type.
/// - `c`: Wallet state.
///
/// # Returns
/// Serialized transaction bytes, ready for broadcast via `api::pay::broadcast_transaction`.
#[cfg_attr(feature = "flutter", frb)]
pub async fn issue_asset(
    asset_name: String,
    amount: u64,
    first_issuance: bool,
    finalize: bool,
    id_account: u32,
    c: &Coin,
) -> Result<Vec<u8>> {
    let network = &c.network();
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    let account = id_account;

    // ── 1. Derive issuance key from wallet seed ──────────────────────────
    let seed_info = get_account_seed(&mut connection, account)
        .await?
        .ok_or_else(|| anyhow!("No seed for account {account}"))?;
    let mnemonic = Mnemonic::parse(seed_info.mnemonic)?;
    let seed = mnemonic.to_seed(&seed_info.phrase);

    let coin_type = match network.network_type() {
        NetworkType::Main => 133u32,
        _ => 1u32,
    };
    let isk = IssueAuthKey::<ZSASchnorr>::from_zip32_seed(&seed, coin_type, 0)
        .map_err(|e| anyhow!("Failed to derive issue auth key: {e:?}"))?;

    // ── 2. Get orchard keys and address ──────────────────────────────────
    let ovk = get_orchard_vk(&mut connection, account)
        .await?
        .ok_or_else(|| anyhow!("No orchard key for account {account}"))?;
    let osk = get_orchard_sk(&mut connection, account)
        .await?
        .ok_or_else(|| anyhow!("No orchard spending key for account {account}"))?;
    let osak = SpendAuthorizingKey::from(&osk);

    let dindex = get_account_dindex(&mut *connection, account).await?;
    let oaddress = ovk.address_at(dindex, orchard::keys::Scope::External);

    // ── 3. Compute asset description hash ────────────────────────────────
    let name_bytes = asset_name.as_bytes().to_vec();
    let non_empty = NonEmpty::from_slice(&name_bytes)
        .ok_or_else(|| anyhow!("Asset name cannot be empty"))?;
    let desc_hash = compute_asset_desc_hash(&non_empty);

    // ── 4. Select an orchard note (required for the first nullifier used
    //      in ZIP-227 rho derivation) ─────────────────────────────────────
    let unspent = fetch_unspent_notes_grouped_by_pool(&mut connection, account).await?;
    let height = client.latest_height().await?;

    // Find an orchard note with sufficient value for fees.
    // Est. fee: 2 actions × 5k + 2 outputs × 5k ≈ 20k zats.
    let est_fee: u64 = 10_000;
    let orchard_note = unspent
        .iter()
        .find(|n| n.pool == 2 && n.height <= height && n.amount >= est_fee)
        .ok_or_else(|| {
            anyhow!(
                "No unspent orchard note with ≥ {est_fee} zats available. \
                 An orchard note is required for issuance (provides the nullifier \
                 for rho derivation per ZIP-227). Shield some ZEC first."
            )
        })?;

    let orchard_note_id = orchard_note.id;

    // ── 5. Optionally select a transparent UTXO for extra fee coverage ───
    // Skip the orchard note already selected above to avoid double-spend.
    let transparent_note = unspent
        .iter()
        .find(|n| n.pool == 0 && n.height <= height && n.amount >= est_fee && n.id != orchard_note_id);

    // ── 6. Get tree state and anchors ────────────────────────────────────
    let h = crate::sync::get_db_height(&mut connection, account).await?;
    let (ts, to) = crate::sync::get_tree_state(network, &mut client, h.height).await?;
    let es = ts.to_edge(&SaplingHasher::default());
    let eo = to.to_edge(&OrchardHasher::default());
    let sapling_anchor = es.root(&SaplingHasher::default());
    let orchard_anchor = eo.root(&OrchardHasher::default());
    let eo_auth = eo.to_auth_path(&OrchardHasher::default());
    let ero = empty_roots(&OrchardHasher::default());

    let current_height = client.latest_height().await?;
    let target_height = current_height
        + if network.network_type() == NetworkType::Regtest {
            ZIP212_GRACE_PERIOD
        } else {
            0
        };

    // ── 7. Build the base transaction ────────────────────────────────────
    let mut builder = Builder::new(
        network,
        BlockHeight::from_u32(target_height),
        BuildConfig::Standard {
            sapling_anchor: sapling_crypto::Anchor::from_bytes(sapling_anchor).into_option(),
            orchard_anchor: orchard::Anchor::from_bytes(orchard_anchor).into_option(),
        },
    );

    // -- orchard spend (provides the nullifier + partially pays fee) --
    let (onote, merkle_path) = get_orchard_note(
        &mut connection,
        orchard_note_id,
        h.height,
        &ovk,
        &eo_auth,
        &ero,
    )
    .await?;

    builder
        .add_orchard_spend::<std::convert::Infallible>(ovk.clone(), onote, merkle_path)?;

    // -- transparent input (if available) --
    let mut tsk_dindex = vec![];
    let mut n_spends: [usize; 3] = [0, 0, 0];
    n_spends[2] = 1; // 1 orchard spend

    if let Some(tnote) = transparent_note {
        let row = sqlx::query(
            "SELECT nullifier, t.pk, t.scope, t.dindex, t.address, t.uncompressed \
             FROM notes \
             JOIN transparent_address_accounts t ON notes.taddress = t.id_taddress \
             WHERE id_note = ?",
        )
        .bind(tnote.id)
        .fetch_one(&mut *connection)
        .await?;

        let nf: Vec<u8> = row.get(0);
        let pk: Vec<u8> = row.get(1);
        let scope: u32 = row.get(2);
        let dindex_t: u32 = row.get(3);
        let taddress: String = row.get(4);
        let uncompressed: bool = row.get(5);

        let pubkey = PublicKey::from_slice(&pk)
            .map_err(|e| anyhow!("Invalid transparent pubkey: {e}"))?;
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&nf[0..32]);
        let n = u32::from_le_bytes(nf[32..36].try_into().unwrap());
        let utxo = OutPoint::new(hash, n);
        let pk_bytes = if uncompressed {
            pubkey.serialize_uncompressed().to_vec()
        } else {
            pubkey.serialize().to_vec()
        };
        let pkh: [u8; 20] = Ripemd160::digest(Sha256::digest(&pk_bytes)).into();
        let addr = TransparentAddress::PublicKeyHash(pkh);
        let coin =
            TxOut::new(Zatoshis::from_u64(tnote.amount).unwrap(), addr.script().into());

        builder.add_transparent_input(
            TransparentInputInfo::from_parts(utxo, coin, SpendInfo::P2pkh { pubkey, uncompressed })
                .map_err(|e| anyhow!("{e}"))?,
        );

        tsk_dindex.push((pubkey, scope, dindex_t, taddress, uncompressed));
        n_spends[0] = 1;
    }

    // -- orchard ZEC change output (send change back to self) --
    let orchard_zec_change = orchard_note.amount.saturating_sub(est_fee);
    if orchard_zec_change > 0 {
        builder.add_orchard_output::<std::convert::Infallible>(
            Some(ovk.to_ovk(orchard::keys::Scope::External)),
            oaddress,
            Zatoshis::from_u64(orchard_zec_change).unwrap(),
            AssetBase::zatoshi(),
            MemoBytes::empty(),
        )?;
    }

    // -- transparent change output (if we have a transparent input) --
    if let Some(tnote) = transparent_note {
        let tkeys = select_account_transparent(&mut connection, account, dindex).await?;
        if let Some(xvk) = tkeys.xvk.as_ref() {
            let (_pk_bytes, change_taddr) =
                crate::account::derive_transparent_address(xvk, 0, dindex, false)?;
            let transparent_change = tnote.amount.saturating_sub(est_fee);
            if transparent_change > 0 {
                builder.add_transparent_output(
                    &change_taddr,
                    Zatoshis::from_u64(transparent_change).unwrap(),
                )?;
            }
        }
    }

    // ── 8. Build PCZT parts ─────────────────────────────────────────────
    info!("Building PCZT with ZSA issuance");
    let pczt_result = builder.build_for_pczt(
        OsRng,
        &FeeRule::standard(),
        |asset: &AssetBase| *asset != AssetBase::zatoshi(),
    )?;
    let orchard_meta = &pczt_result.orchard_meta;

    // ── 9. Build ZsaBuilder with issuance outputs ────────────────────────
    let mut zsa_builder = ZsaBuilder::new(isk.clone());
    zsa_builder
        .add_issue_output(
            desc_hash,
            oaddress,
            NoteValue::from_raw(amount),
            first_issuance,
            &mut OsRng,
        )
        .map_err(|e| anyhow!("Failed to add issue output: {e:?}"))?;

    if finalize {
        zsa_builder
            .finalize_asset(&desc_hash)
            .map_err(|e| anyhow!("Failed to finalize asset: {e:?}"))?;
    }

    // ── 10. Run the PCZT pipeline ────────────────────────────────────────
    // a. Creator
    let pczt = Creator::build_from_parts(pczt_result.pczt_parts)
        .ok_or_else(|| anyhow!("Creator failed: empty PCZT parts"))?;

    // -- update transparent metadata (bip32 derivation paths) --
    let updater = Updater::new(pczt);
    let updater = updater
        .update_transparent_with(|mut u| {
            for (i, (pubkey, scope, dindex_t, taddress, uncompressed)) in
                tsk_dindex.iter().enumerate()
            {
                u.update_input_with(i, |mut u| {
                    let derivation_path = vec![*scope, *dindex_t];
                    let path =
                        Bip32Derivation::parse([0u8; 32], derivation_path).unwrap();
                    u.set_bip32_derivation(pubkey.serialize().to_vec(), path);
                    u.set_proprietary("scope".to_string(), scope.to_le_bytes().to_vec());
                    u.set_proprietary("dindex".to_string(), dindex_t.to_le_bytes().to_vec());
                    u.set_proprietary("address".to_string(), taddress.as_bytes().to_vec());
                    u.set_proprietary("uncompressed".to_string(), vec![*uncompressed as u8]);
                    // Set hash160 preimage for the signer
                    let pk_bytes = if *uncompressed {
                        pubkey.serialize_uncompressed().to_vec()
                    } else {
                        pubkey.serialize().to_vec()
                    };
                    u.set_hash160_preimage(pk_bytes);
                    Ok(())
                })?;
            }
            Ok(())
        })
        .map_err(|e| anyhow!("Failed to update transparent metadata: {e:?}"))?;

    // -- update orchard metadata (user address for output) --
    let updater = updater
        .update_orchard_with(|mut u| {
            // The first orchard action (index 0) is the spend; outputs start after.
            // We only have one change output at action index 1 (after the spend).
            let output_action_index = orchard_meta
                .output_action_index(0)
                .unwrap();
            u.update_action_with(output_action_index, |mut u| {
                let encoded = oaddress.to_raw_address_bytes();
                u.set_output_user_address(hex::encode(encoded));
                Ok(())
            })?;
            Ok(())
        })
        .map_err(|e| anyhow!("Failed to update orchard metadata: {e:?}"))?;

    let pczt = updater.finish();

    // b. Issuer phase 1 — build awaiting sighash (derives rho from first nullifier)
    let pczt = Issuer::new(pczt)
        .build_awaiting_sighash(zsa_builder, OsRng)
        .map_err(|e| anyhow!("Issuer (phase 1) failed: {e:?}"))?;

    // c. IoFinalizer — computes the shielded sighash
    let pczt = IoFinalizer::new(pczt)
        .finalize_io()
        .map_err(|e| anyhow!("IoFinalizer failed: {e:?}"))?;

    // d. Issuer phase 2 — sign the issuance bundle
    let pczt = Issuer::new(pczt)
        .sign(&isk)
        .map_err(|e| anyhow!("Issuer (phase 2/sign) failed: {e:?}"))?;

    // e. Prover — create the orchard ZSA proof
    let pczt = Prover::new(pczt)
        .create_orchard_proof(&ORCHARD_ZSA_PK)
        .map_err(|e| anyhow!("Prover (orchard) failed: {e:?}"))?
        .finish();

    // f. Signer — sign transparent and orchard inputs
    let mut signer = Signer::new(pczt.clone())
        .map_err(|e| anyhow!("Signer creation failed: {e:?}"))?;

    // Sign transparent inputs
    let dindex_p = get_account_dindex(&mut *connection, account).await?;
    let tkeys = select_account_transparent(&mut connection, account, dindex_p).await?;
    let tbundle = pczt.transparent();
    for index in 0..n_spends[0] {
        let inp = &tbundle.inputs()[index];
        let scope = u32::from_le_bytes(
            inp.proprietary()["scope"].clone().try_into().unwrap(),
        );
        let dindex_t = u32::from_le_bytes(
            inp.proprietary()["dindex"].clone().try_into().unwrap(),
        );

        let sk = match tkeys.xsk.as_ref() {
            Some(tsk) => {
                let sk_bytes = derive_transparent_sk(tsk, scope, dindex_t)?;
                SecretKey::from_bytes(&sk_bytes.try_into().unwrap()).ok()
            }
            None => {
                let address =
                    String::from_utf8(inp.proprietary()["address"].clone())?;
                sqlx::query(
                    "SELECT sk FROM transparent_address_accounts \
                     WHERE account = ?1 AND address = ?2",
                )
                .bind(account)
                .bind(&address)
                .map(|r: SqliteRow| {
                    let sk: Vec<u8> = r.get(0);
                    SecretKey::from_bytes(&sk.try_into().unwrap()).unwrap()
                })
                .fetch_optional(&mut *connection)
                .await?
            }
        };
        let sk = sk.ok_or_else(|| anyhow!("No signing key for transparent input {index}"))?;

        let sighash = signer
            .transparent_sighash(index)
            .map_err(|e| anyhow!("Failed to get transparent sighash: {e:?}"))?;
        let secp = secp256k1::Secp256k1::new();
        let msg = secp256k1::Message::from_digest(sighash);
        let sig = secp.sign_ecdsa(&msg, &sk);
        signer
            .append_transparent_signature(index, sig)
            .map_err(|e| anyhow!("Failed to append transparent signature: {e:?}"))?;
    }

    // Sign orchard spends
    let orchard_spend_action_index = orchard_meta
        .spend_action_index(0)
        .ok_or_else(|| anyhow!("No orchard spend action index"))?;
    signer
        .sign_orchard(orchard_spend_action_index, &osak)
        .map_err(|e| anyhow!("Failed to sign orchard spend: {e:?}"))?;

    let pczt = signer.finish();

    // g. SpendFinalizer
    let pczt = SpendFinalizer::new(pczt)
        .finalize_spends()
        .map_err(|e| anyhow!("SpendFinalizer failed: {e:?}"))?;

    // h. TransactionExtractor — extract the final transaction
    let tx = TransactionExtractor::new(pczt)
        .extract()
        .map_err(|e| anyhow!("TransactionExtractor failed: {e:?}"))?;

    let mut tx_bytes = vec![];
    tx.write(&mut tx_bytes)
        .map_err(|e| anyhow!("Failed to serialize transaction: {e}"))?;

    // Pre-insert the asset with its name so that when the sync handler later
    // encounters this asset via compact block, the name is already set.
    // INSERT OR IGNORE ensures no duplicate if the row already exists.
    let ik = orchard::issuance::auth::IssueValidatingKey::from(&isk);
    let asset_id = orchard::note::AssetId::new_v0(&ik, &desc_hash);
    let asset_base_bytes = AssetBase::custom(&asset_id).to_bytes().to_vec();
    sqlx::query(
        "INSERT OR IGNORE INTO assets(asset_desc_hash, ik, asset_base, asset_name, finalized, first_seen_height)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(desc_hash.to_vec())
    .bind(ik.encode())
    .bind(&asset_base_bytes)
    .bind(&asset_name)
    .bind(finalize)
    .bind(0_i64)     // first_seen_height — updated by sync to the actual block height
    .execute(&mut *connection)
    .await?;

    info!(
        "Issued asset \"{asset_name}\" (amount={amount}, first={first_issuance}, finalize={finalize}), \
         tx size={} bytes",
        tx_bytes.len()
    );
    debug!("{}", hex::encode(&tx_bytes));

    Ok(tx_bytes)
}
