use anyhow::{anyhow, Result};
use bip39::Mnemonic;
use nonempty::NonEmpty;
use orchard::{
    issuance::{
        auth::{IssueAuthKey, ZSASchnorr},
        compute_asset_desc_hash,
    },
    note::AssetBase,
};
use tracing::{debug, info};
use zcash_protocol::consensus::{NetworkType, Parameters};

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

use crate::{
    account::get_account_seed,
    api::coin::Coin,
    pay::{
        plan::{extract_transaction, plan_transaction, sign_transaction},
        pool::ALL_POOLS,
    },
};

/// Information needed to add an asset issuance to a transaction.
///
/// Pass this to `plan_transaction` to turn a normal send into an issuance.
/// For normal sends, pass `None`.
#[cfg_attr(feature = "flutter", frb(ignore))]
pub struct IssuanceInfo {
    pub asset_name: String,
    pub amount: u64,
    pub first_issuance: bool,
    pub finalize: bool,
    pub isk: IssueAuthKey<ZSASchnorr>,
    pub desc_hash: [u8; 32],
}

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

    // ── 2. Compute asset description hash ────────────────────────────────
    let name_bytes = asset_name.as_bytes().to_vec();
    let non_empty = NonEmpty::from_slice(&name_bytes)
        .ok_or_else(|| anyhow!("Asset name cannot be empty"))?;
    let desc_hash = compute_asset_desc_hash(&non_empty);

    // ── 3. Derive identifiers for DB pre-insert ──────────────────────────
    let ik = orchard::issuance::auth::IssueValidatingKey::from(&isk);
    let asset_id = orchard::note::AssetId::new_v0(&ik, &desc_hash);
    let asset_base_bytes = AssetBase::custom(&asset_id).to_bytes().to_vec();

    // ── 4. Build issuance info ───────────────────────────────────────────
    let issuance_info = IssuanceInfo {
        asset_name: asset_name.clone(),
        amount,
        first_issuance,
        finalize,
        isk,
        desc_hash,
    };

    // ── 5. Plan the transaction (note selection, fees, builder, PCZT) ────
    let pczt_package = plan_transaction(
        network,
        &mut *connection,
        &mut client,
        account,
        ALL_POOLS,
        &[],           // no recipients — issuance output goes to issuer
        false,         // recipient_pays_fee
        None,          // confirmations
        false,         // smart_transparent
        None,          // category
        Some(&issuance_info),
    )
    .await?;

    // ── 6. Sign ──────────────────────────────────────────────────────────
    let pczt_package = sign_transaction(&mut *connection, account, network, &pczt_package).await?;

    // ── 7. Extract ───────────────────────────────────────────────────────
    let tx_bytes = extract_transaction(&pczt_package).await?;

    // ── 8. Pre-insert asset into DB for sync name resolution ─────────────
    sqlx::query(
        "INSERT OR IGNORE INTO assets(asset_desc_hash, ik, asset_base, asset_name, finalized, first_seen_height)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(desc_hash.to_vec())
    .bind(ik.encode())
    .bind(&asset_base_bytes)
    .bind(&asset_name)
    .bind(finalize)
    .bind(0_i64)
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
