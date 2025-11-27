use anyhow::Result;
use bincode::{config::legacy, Decode, Encode};

use crate::{api::coin::Coin, frb_generated::StreamSink, pay::{Recipient, TxPlan, plan::plan_transaction}};
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
pub async fn prepare(recipients: &[Recipient], options: PaymentOptions, c: &Coin) -> Result<PcztPackage> {
    let account = c.account;
    let network = &c.network();
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
pub async fn sign_transaction(pczt: &PcztPackage, c: &Coin) -> Result<PcztPackage> {
    let account = c.account;
    let mut connection = c.get_connection().await?;

    let tx = crate::pay::plan::sign_transaction(&mut *connection, account, pczt).await?;

    Ok(tx)
}

#[frb]
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
pub async fn sign_ledger_transaction(sink: StreamSink<SigningEvent>, pczt: PcztPackage, c: &Coin) -> Result<()> {
    let connection = c.get_connection().await?;
    crate::ledger::builder::sign_ledger_transaction(c.network(), sink, connection, c.account, pczt).await
}

#[frb]
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub async fn sign_ledger_transaction(sink: StreamSink<SigningEvent>, pczt: PcztPackage) -> Result<()> {
    crate::no_ledger::sign_ledger_transaction().await
}

pub enum SigningEvent {
    Progress(String),
    Result(PcztPackage),
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
pub async fn broadcast_transaction(height: u32, tx_bytes: &[u8], c: &Coin) -> Result<String> {
    let mut client = c.client().await?;

    let tx = crate::pay::send(&mut client, height, tx_bytes).await?;
    Ok(tx)
}

#[frb(sync)]
pub fn to_plan(package: &PcztPackage, c: &Coin) -> Result<TxPlan> {
    TxPlan::from_package(&c.network(), package)
}

#[frb]
pub async fn send(height: u32, data: &[u8], c: &Coin) -> Result<String> {
    let mut client = c.client().await?;

    let tx = crate::pay::send(&mut client, height, data).await?;
    Ok(tx)
}

#[frb]
pub async fn store_pending_tx(height: u32, txid: &[u8],
    price: Option<f64>, category: Option<u32>, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::store_pending_tx(&mut connection, c.account, height, txid, price, category).await?;

    Ok(())
}

#[frb(sync)]
pub fn parse_payment_uri(uri: &str) -> Option<Vec<Recipient>> {
    crate::pay::prepare::parse_payment_uri(uri).ok()
}
