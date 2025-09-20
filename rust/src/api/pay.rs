use anyhow::Result;
use bincode::{config::legacy, Decode, Encode};

use crate::pay::{plan::plan_transaction, Recipient, TxPlan};
use flutter_rust_bridge::frb;

pub enum DustChangePolicy {
    Discard,
    SendToRecipient,
}

pub struct PaymentOptions {
    pub src_pools: u8,
    pub recipient_pays_fee: bool,
    pub smart_transparent: bool,
    pub dust_change_policy: DustChangePolicy,
    pub category: Option<u32>,
}

#[frb]
pub async fn build_puri(recipients: &[Recipient]) -> Result<String> {
    crate::pay::plan::build_puri(recipients).await
}

#[frb]
pub async fn prepare(recipients: &[Recipient], options: PaymentOptions) -> Result<PcztPackage> {
    let c = crate::get_coin!();
    let account = c.account;
    let network = &c.network;
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;

    plan_transaction(
        network,
        &mut *connection,
        &mut client,
        account,
        options.src_pools,
        recipients,
        options.recipient_pays_fee,
        options.smart_transparent,
        options.dust_change_policy,
        options.category,
    )
    .await
}

#[frb]
pub async fn sign_transaction(pczt: &PcztPackage) -> Result<PcztPackage> {
    let c = crate::get_coin!();
    let account = c.account;
    let mut connection = c.get_connection().await?;

    let tx = crate::pay::plan::sign_transaction(&mut *connection, account, pczt).await?;

    Ok(tx)
}

#[frb]
pub async fn extract_transaction(package: &PcztPackage) -> Result<Vec<u8>> {
    crate::pay::plan::extract_transaction(package).await
}

#[frb(dart_metadata = ("freezed"))]
#[derive(Encode, Decode)]
pub struct PcztPackage {
    pub pczt: Vec<u8>,
    pub n_spends: [usize; 3],
    pub sapling_indices: Vec<usize>,
    pub orchard_indices: Vec<usize>,
    pub can_sign: bool,
    pub can_broadcast: bool,
    pub price: Option<f64>,
    pub category: Option<u32>,
}

#[frb]
pub fn pack_transaction(pczt: &PcztPackage) -> Result<Vec<u8>> {
    let pkg = bincode::encode_to_vec(pczt, legacy())?;
    Ok(pkg)
}

#[frb]
pub fn unpack_transaction(bytes: &[u8]) -> Result<PcztPackage> {
    let (pkg, _) = bincode::decode_from_slice(bytes, legacy())?;
    Ok(pkg)
}

#[frb]
pub async fn broadcast_transaction(height: u32, tx_bytes: &[u8]) -> Result<String> {
    let c = crate::get_coin!();
    let mut client = c.client().await?;

    let tx = crate::pay::send(&mut client, height, tx_bytes).await?;
    Ok(tx)
}

#[frb(sync)]
pub fn to_plan(package: &PcztPackage) -> Result<TxPlan> {
    let c = crate::get_coin!();
    TxPlan::from_package(&c.network, package)
}

#[frb]
pub async fn send(height: u32, data: &[u8]) -> Result<String> {
    let c = crate::get_coin!();
    let mut client = c.client().await?;

    let tx = crate::pay::send(&mut client, height, data).await?;
    Ok(tx)
}

#[frb]
pub async fn store_pending_tx(height: u32, txid: &[u8],
    price: Option<f64>, category: Option<u32>) -> Result<()> {
    let c = crate::get_coin!();
    let mut connection = c.get_connection().await?;
    crate::db::store_pending_tx(&mut connection, c.account, height, txid, price, category).await?;

    Ok(())
}

#[frb(sync)]
pub fn parse_payment_uri(uri: &str) -> Option<Vec<Recipient>> {
    crate::pay::prepare::parse_payment_uri(uri).ok()
}
