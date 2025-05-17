use anyhow::Result;
use bincode::{config::legacy, Decode, Encode};

use crate::pay::{plan::plan_transaction, Recipient, TxPlan};
use flutter_rust_bridge::frb;

#[frb]
pub async fn prepare(
    src_pools: u8,
    recipients: &[Recipient],
    recipient_pays_fee: bool,
) -> Result<PcztPackage> {
    let c = crate::get_coin!();
    let account = c.account;
    let network = &c.network;
    let connection = c.get_pool();
    let mut client = c.client().await?;

    plan_transaction(
        network,
        connection,
        &mut client,
        account,
        src_pools,
        recipients,
        recipient_pays_fee,
    )
    .await
}

#[frb]
pub async fn sign_transaction(pczt: &PcztPackage) -> Result<PcztPackage> {
    let c = crate::get_coin!();
    let account = c.account;
    let connection = c.get_pool();

    let tx = crate::pay::plan::sign_transaction(connection, account, pczt).await?;

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
    pub puri: String,
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

#[frb(sync)]
pub fn parse_payment_uri(uri: &str) -> Option<Vec<Recipient>> {
    crate::pay::prepare::parse_payment_uri(uri).ok()
}
