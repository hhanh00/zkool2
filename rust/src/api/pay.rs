use anyhow::Result;
use bincode::{
    config::{Configuration, Limit, LittleEndian, Varint},
    Decode, Encode,
};

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

/// The ZIP 317 marginal fee, in zatoshis per logical action, that a
/// transaction sent now pays: the wallet's `fee_per_action` override if set,
/// else the ZIP 2009 schedule for the wallet's network at the block after
/// `height`, the current chain tip. One ZEC note below this is dust.
#[cfg_attr(feature = "flutter", frb)]
pub async fn fee_per_action(height: u32, c: &Coin) -> Result<u64> {
    let mut connection = c.get_connection().await?;
    crate::pay::fee::resolve_cost_per_action(&c.network(), &mut connection, height.saturating_add(1))
        .await
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

/// The largest PCZT package the wallet decodes from outside: a file, an
/// animated QR, or a zkool_graphql request. A real package weighs a few KB
/// to a few hundred KB. bincode allocates a byte vector at the length its
/// prefix claims before reading it, so without a limit a ten-byte input can
/// ask for gigabytes (#1540).
pub const PCZT_MAX_BYTES: usize = 8 << 20;

/// bincode config for `PcztPackage` transport: the `standard()` encoding,
/// so the bytes are interchangeable with zkool_graphql, whose `prepareSend`
/// / `frostSign` use the same config, bounded by [`PCZT_MAX_BYTES`]. The two
/// used to disagree (`legacy()` here), which made it impossible for an app
/// user and a zkool_graphql user to co-sign a FROST transaction.
#[cfg_attr(feature = "flutter", frb(ignore))]
pub fn pczt_config() -> Configuration<LittleEndian, Varint, Limit<PCZT_MAX_BYTES>> {
    bincode::config::standard().with_limit::<PCZT_MAX_BYTES>()
}

#[cfg_attr(feature = "flutter", frb)]
/// Serialize a PCZT for transport between participants.
pub fn pack_transaction(pczt: &PcztPackage) -> Result<Vec<u8>> {
    let pkg = bincode::encode_to_vec(pczt, pczt_config())?;
    Ok(pkg)
}

#[cfg_attr(feature = "flutter", frb)]
/// Deserialize a PCZT package received from outside. A length prefix above
/// [`PCZT_MAX_BYTES`] is an error before anything is allocated.
pub fn unpack_transaction(bytes: &[u8]) -> Result<PcztPackage> {
    let (pkg, _) = bincode::decode_from_slice(bytes, pczt_config())?;
    Ok(pkg)
}

/// Broadcast a signed package: extract, post, lock the spent notes and
/// record the package's price and category. Returns the txid; a rejection
/// is an error.
#[cfg_attr(feature = "flutter", frb)]
pub async fn broadcast(package: &PcztPackage, account: u32, c: &Coin) -> Result<String> {
    let tx_bytes = crate::pay::plan::extract_transaction(package).await?;
    send(&tx_bytes, Some(crate::pay::pending_of(package)), account, c).await
}

/// Broadcast an already extracted transaction (an issuance, for instance).
#[cfg_attr(feature = "flutter", frb)]
pub async fn broadcast_bytes(tx_bytes: &[u8], account: u32, c: &Coin) -> Result<String> {
    send(tx_bytes, None, account, c).await
}

/// Post outside the sync session, then queue the writes that follow an
/// acceptance through the sync driver. A rejection is an error; once the
/// node has accepted, a failure to queue or apply the writes is logged and
/// the txid still returned (see [`crate::pay::bookkeeping_failed`]).
async fn send(
    tx_bytes: &[u8],
    pending: Option<crate::pay::Pending>,
    account: u32,
    c: &Coin,
) -> Result<String> {
    let mut client = c.client().await?;
    let sent = crate::pay::post(&mut client, tx_bytes).await?;
    let response = sent.response.clone();
    let recorded = async {
        c.driver()?
            .apply(
                c,
                crate::sync::driver::Apply::Sent {
                    account,
                    sent,
                    tx_bytes: tx_bytes.to_vec(),
                    pending,
                },
            )
            .await
    }
    .await;
    if let Err(error) = recorded {
        crate::pay::bookkeeping_failed(&response, &error);
    }
    Ok(response)
}

#[cfg_attr(feature = "flutter", frb(sync))]
pub fn to_plan(package: &PcztPackage, c: &Coin) -> Result<TxPlan> {
    TxPlan::from_package(&c.network(), package)
}

#[cfg_attr(feature = "flutter", frb(sync))]
pub fn parse_payment_uri(uri: &str) -> Option<Vec<Recipient>> {
    crate::pay::prepare::parse_payment_uri(uri).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package() -> PcztPackage {
        PcztPackage {
            pczt: vec![1, 2, 3],
            n_spends: [1, 0, 2, 0],
            sapling_indices: vec![],
            orchard_indices: vec![0, 1],
            ironwood_indices: vec![],
            can_sign: true,
            can_broadcast: false,
            price: Some(1.5),
            category: Some(3),
            is_issuance: false,
        }
    }

    #[test]
    fn a_package_round_trips_through_the_limited_config() {
        let packed = pack_transaction(&package()).unwrap();
        let unpacked = unpack_transaction(&packed).unwrap();
        assert_eq!(unpacked.pczt, vec![1, 2, 3]);
        assert_eq!(unpacked.orchard_indices, vec![0, 1]);
        assert_eq!(unpacked.n_spends, [1, 0, 2, 0]);
    }

    #[test]
    fn a_length_prefix_above_the_limit_is_an_error_before_allocating() {
        // `pczt` is the first field; its varint length says 16 GB.
        let mut bytes = vec![0xFD];
        bytes.extend_from_slice(&(16u64 << 30).to_le_bytes());
        bytes.push(0);
        let err = unpack_transaction(&bytes).unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("limit"),
            "expected a limit error, got: {err}"
        );
    }
}
