use anyhow::Result;
use bincode::{config::standard, Decode, Encode};

use crate::{
    api::coin::Coin,
    pay::{
        plan::{plan_transaction, PaymentRequest, PlanMode},
        Recipient, TxPlan,
    },
};
#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;
#[cfg(feature = "flutter")]
use crate::Sink;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

pub struct PaymentOptions {
    pub src_pools: u8,
    pub recipient_pays_fee: bool,
    pub smart_transparent: bool,
    pub category: Option<u32>,
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn build_puri(recipients: &[Recipient]) -> Result<String> {
    crate::pay::plan::build_puri(recipients).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn prepare(
    recipients: &[Recipient],
    options: PaymentOptions,
    account: u32,
    c: &Coin,
) -> Result<PcztPackage> {
    let network = &c.network();
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;

    let request = PaymentRequest {
        recipients: recipients.to_vec(),
        src_pools: options.src_pools,
        recipient_pays_fee: options.recipient_pays_fee,
        confirmations: 0,
        category: options.category,
    };
    let mode = if options.smart_transparent {
        PlanMode::ShieldOneAddress
    } else {
        PlanMode::Standard
    };
    plan_transaction(network, &mut *connection, &mut client, account, &request, &mode).await
}

/// Sign with whichever signer the account uses (wallet keys or its Ledger
/// app), without progress reports.
#[cfg_attr(feature = "flutter", frb)]
pub async fn sign_transaction(pczt: &PcztPackage, account: u32, c: &Coin) -> Result<PcztPackage> {
    let mut connection = c.get_connection().await?;
    crate::pay::sign::sign(&c.network(), &mut connection, account, pczt.clone(), &()).await
}

/// Sign with whichever signer the account uses, streaming `Progress` events
/// (a Ledger app's prompts) and finally the signed package as `Result`.
#[cfg(feature = "flutter")]
#[frb]
pub async fn sign_transaction_with_progress(
    sink: StreamSink<SigningEvent>,
    package: PcztPackage,
    account: u32,
    c: &Coin,
) -> Result<()> {
    let c = c.clone();
    tokio::spawn(async move {
        let signed = async {
            let mut connection = c.get_connection().await?;
            crate::pay::sign::sign(&c.network(), &mut connection, account, package, &sink).await
        }
        .await;
        match signed {
            Ok(package) => sink.send(SigningEvent::Result(package)).await,
            Err(e) => sink.send_error(e).await,
        }
    });
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
#[derive(Debug, Clone)]
pub enum SigningEvent {
    Progress(String),
    Result(PcztPackage),
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn extract_transaction(package: &PcztPackage) -> Result<Vec<u8>> {
    crate::pay::plan::extract_transaction(package).await
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Debug, Clone, Encode, Decode)]
pub struct PcztPackage {
    pub pczt: Vec<u8>,
    pub n_spends: [usize; 4],
    pub sapling_indices: Vec<usize>,
    pub orchard_indices: Vec<usize>,
    pub ironwood_indices: Vec<usize>,
    pub can_sign: bool,
    pub can_broadcast: bool,
    pub price: Option<f64>,
    pub category: Option<u32>,
    pub is_issuance: bool,
}

#[cfg_attr(feature = "flutter", frb)]
/// Serialize a PCZT for transport between participants.
///
/// Uses bincode's `standard()` config so the bytes are interchangeable with
/// zkool_graphql, whose `prepareSend` / `frostSign` use the same config. The
/// two used to disagree (`legacy()` here), which made it impossible for an app
/// user and a zkool_graphql user to co-sign a FROST transaction.
pub fn pack_transaction(pczt: &PcztPackage) -> Result<Vec<u8>> {
    let pkg = bincode::encode_to_vec(pczt, standard())?;
    Ok(pkg)
}

#[cfg_attr(feature = "flutter", frb)]
pub fn unpack_transaction(bytes: &[u8]) -> Result<PcztPackage> {
    let (pkg, _) = bincode::decode_from_slice(bytes, standard())?;
    Ok(pkg)
}

/// Broadcast a signed package: extract, post, lock the spent notes and
/// record the package's price and category. Returns the txid; a rejection
/// is an error.
#[cfg_attr(feature = "flutter", frb)]
pub async fn broadcast(package: &PcztPackage, account: u32, c: &Coin) -> Result<String> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    crate::pay::broadcast(&mut connection, &mut client, account, package).await
}

/// Broadcast an already extracted transaction (an issuance, for instance).
#[cfg_attr(feature = "flutter", frb)]
pub async fn broadcast_bytes(tx_bytes: &[u8], account: u32, c: &Coin) -> Result<String> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    crate::pay::broadcast_bytes(&mut connection, &mut client, account, tx_bytes, None).await
}

#[cfg_attr(feature = "flutter", frb(sync))]
pub fn to_plan(package: &PcztPackage, c: &Coin) -> Result<TxPlan> {
    TxPlan::from_package(&c.network(), package)
}

#[cfg_attr(feature = "flutter", frb(sync))]
pub fn parse_payment_uri(uri: &str) -> Option<Vec<Recipient>> {
    crate::pay::prepare::parse_payment_uri(uri).ok()
}
