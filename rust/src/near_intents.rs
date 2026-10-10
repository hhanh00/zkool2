//! Direct 1Click integration. Amounts are integer strings in asset base units.
//! This module never signs or broadcasts wallet transactions and adds no app fee.
use anyhow::{bail, ensure, Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;

use crate::{api::coin::Coin, net::http};

const BASE_URL: &str = "https://1click.chaindefuser.com";
pub const ZEC_ASSET: &str = "nep141:zec.omft.near";
const MAX_RESPONSE: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "graphql", derive(juniper::GraphQLObject))]
pub struct SwapAsset {
    pub asset_id: String,
    pub decimals: i32,
    pub blockchain: String,
    pub symbol: String,
    pub price: Option<String>,
    pub contract_address: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "graphql", derive(juniper::GraphQLEnum))]
pub enum SwapType {
    ExactInput,
    ExactOutput,
}

/// Source/destination fields are shared so inbound swaps can reuse the client later.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "graphql", derive(juniper::GraphQLInputObject))]
pub struct SwapRequest {
    pub dry: bool,
    pub swap_type: SwapType,
    pub origin_asset: String,
    pub destination_asset: String,
    /// Base units of source for EXACT_INPUT, destination for EXACT_OUTPUT.
    pub amount: String,
    /// Basis points: 100 = 1%.
    pub slippage_tolerance: i32,
    pub recipient: String,
    pub refund_to: String,
    /// RFC 3339 expiry, validated by 1Click.
    pub deadline: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "graphql", derive(juniper::GraphQLObject))]
pub struct SwapQuote {
    pub deposit_address: Option<String>,
    pub deposit_memo: Option<String>,
    pub amount_in: String,
    pub amount_out: String,
    pub min_amount_in: Option<String>,
    pub min_amount_out: Option<String>,
    pub amount_in_formatted: Option<String>,
    pub amount_out_formatted: Option<String>,
    pub amount_in_usd: Option<String>,
    pub amount_out_usd: Option<String>,
    pub deadline: Option<String>,
    pub time_estimate: Option<i32>,
}

/// Keep the complete provider response, including its signature, for persistence.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "graphql", derive(juniper::GraphQLObject))]
pub struct SwapQuoteResponse {
    pub quote: SwapQuote,
    pub raw_response: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "graphql", derive(juniper::GraphQLObject))]
pub struct SwapTransaction {
    pub hash: String,
    pub explorer_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "graphql", derive(juniper::GraphQLObject))]
pub struct SwapDetails {
    pub amount_in: Option<String>,
    pub amount_out: Option<String>,
    pub refunded_amount: Option<String>,
    #[serde(default)]
    pub origin_chain_tx_hashes: Vec<SwapTransaction>,
    #[serde(default)]
    pub destination_chain_tx_hashes: Vec<SwapTransaction>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "graphql", derive(juniper::GraphQLObject))]
pub struct SwapStatus {
    /// Preserve unknown provider states instead of reporting a false failure.
    pub status: String,
    pub swap_details: Option<SwapDetails>,
    pub raw_response: String,
}

fn quote_payload(request: &SwapRequest) -> Result<serde_json::Value> {
    ensure!(
        !request.amount.is_empty()
            && request.amount.bytes().all(|b| b.is_ascii_digit())
            && request.amount.bytes().any(|b| b != b'0'),
        "Amount must be a positive integer in base units"
    );
    ensure!(
        (0..=10000).contains(&request.slippage_tolerance),
        "Invalid slippage tolerance"
    );
    for value in [
        &request.origin_asset,
        &request.destination_asset,
        &request.recipient,
        &request.refund_to,
        &request.deadline,
    ] {
        ensure!(!value.trim().is_empty(), "Missing swap parameter");
    }
    ensure!(
        request.origin_asset != request.destination_asset,
        "Source and destination must differ"
    );
    let mut payload = serde_json::to_value(request)?;
    payload["depositType"] = "ORIGIN_CHAIN".into();
    payload["recipientType"] = "DESTINATION_CHAIN".into();
    payload["refundType"] = "ORIGIN_CHAIN".into();
    payload["depositMode"] = "SIMPLE".into();
    Ok(payload)
}

/// No automatic POST retries: a timed-out quote may already have been created.
async fn request(c: &Coin, method: hyper::Method, url: &str, body: Vec<u8>) -> Result<String> {
    let timeout = Duration::from_secs(30);
    let headers = [("Content-Type".to_owned(), "application/json".to_owned())];
    let (status, bytes) = if c.transport == 1 {
        let response =
            http::tor_request(method, url, body, &headers, timeout, MAX_RESPONSE).await?;
        (response.status, response.body)
    } else {
        ensure!(
            c.transport == 0 || c.transport == 3,
            "1Click HTTP does not support the selected transport"
        );
        ensure!(
            c.transport != 3 || !c.proxy.is_empty(),
            "External proxy is not configured"
        );
        let client = http::client(http::proxy_url(c.transport, &c.proxy), timeout)?;
        let mut response = client
            .request(method, url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;
        let status = response.status().as_u16();
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            ensure!(
                bytes.len() + chunk.len() <= MAX_RESPONSE,
                "1Click response is too large"
            );
            bytes.extend_from_slice(&chunk);
        }
        (status, bytes)
    };
    if !(200..300).contains(&status) {
        // Do not echo provider bodies: they may contain recipient/refund addresses.
        bail!("1Click request failed with HTTP {status}");
    }
    String::from_utf8(bytes).context("Invalid 1Click response encoding")
}

fn decode<T: DeserializeOwned>(response: &str) -> Result<T> {
    serde_json::from_str(response).context("Invalid 1Click response")
}

pub async fn assets(c: &Coin) -> Result<Vec<SwapAsset>> {
    decode(
        &request(
            c,
            hyper::Method::GET,
            &format!("{BASE_URL}/v0/tokens"),
            vec![],
        )
        .await?,
    )
}

pub async fn quote(request_data: SwapRequest, c: &Coin) -> Result<SwapQuoteResponse> {
    // First version exposes ZEC out only. The request model supports future ZEC in.
    ensure!(
        request_data.origin_asset == ZEC_ASSET,
        "Only native ZEC outgoing swaps are supported"
    );
    use zcash_protocol::consensus::{NetworkType, Parameters};
    ensure!(
        c.network().network_type() == NetworkType::Main,
        "1Click requires Zcash mainnet"
    );
    crate::openalias::try_validate_zcash_address(&request_data.refund_to, NetworkType::Main)?;
    let payload = quote_payload(&request_data)?;
    let raw_response = request(
        c,
        hyper::Method::POST,
        &format!("{BASE_URL}/v0/quote"),
        serde_json::to_vec(&payload)?,
    )
    .await?;
    #[derive(Deserialize)]
    struct Response {
        quote: SwapQuote,
    }
    let response: Response = decode(&raw_response)?;
    if !request_data.dry {
        let address = response
            .quote
            .deposit_address
            .as_deref()
            .context("1Click did not return a deposit address")?;
        crate::openalias::try_validate_zcash_address(address, NetworkType::Main)?;
        ensure!(
            response.quote.deposit_memo.is_none(),
            "ZEC deposit unexpectedly requires a memo"
        );
    }
    Ok(SwapQuoteResponse {
        quote: response.quote,
        raw_response,
    })
}

pub async fn status(
    deposit_address: &str,
    deposit_memo: Option<&str>,
    c: &Coin,
) -> Result<SwapStatus> {
    ensure!(
        !deposit_address.trim().is_empty(),
        "Missing deposit address"
    );
    let mut url = reqwest::Url::parse(&format!("{BASE_URL}/v0/status"))?;
    url.query_pairs_mut()
        .append_pair("depositAddress", deposit_address);
    if let Some(memo) = deposit_memo {
        url.query_pairs_mut().append_pair("depositMemo", memo);
    }
    let raw_response = request(c, hyper::Method::GET, url.as_str(), vec![]).await?;
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        status: String,
        swap_details: Option<SwapDetails>,
    }
    let response: Response = decode(&raw_response)?;
    Ok(SwapStatus {
        status: response.status,
        swap_details: response.swap_details,
        raw_response,
    })
}

/// Notifies 1Click about an already broadcast deposit; this does not send funds.
pub async fn submit_deposit(deposit_address: &str, tx_hash: &str, c: &Coin) -> Result<()> {
    ensure!(
        !deposit_address.trim().is_empty(),
        "Missing deposit address"
    );
    ensure!(
        tx_hash.len() == 64 && tx_hash.bytes().all(|b| b.is_ascii_hexdigit()),
        "Invalid Zcash transaction hash"
    );
    request(
        c,
        hyper::Method::POST,
        &format!("{BASE_URL}/v0/deposit/submit"),
        serde_json::to_vec(
            &serde_json::json!({"depositAddress": deposit_address, "txHash": tx_hash}),
        )?,
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> SwapRequest {
        SwapRequest {
            dry: true,
            swap_type: SwapType::ExactOutput,
            origin_asset: ZEC_ASSET.into(),
            destination_asset: "usdt".into(),
            amount: "600000000".into(),
            slippage_tolerance: 100,
            recipient: "recipient".into(),
            refund_to: "refund".into(),
            deadline: "2026-10-10T12:00:00Z".into(),
        }
    }
    #[test]
    fn exact_output_preserves_base_units_and_omits_fees() {
        let payload = quote_payload(&sample()).unwrap();
        assert_eq!(payload["amount"], "600000000");
        assert_eq!(payload["swapType"], "EXACT_OUTPUT");
        assert_eq!(payload["depositType"], "ORIGIN_CHAIN");
        assert!(payload.get("appFees").is_none());
    }
    #[test]
    fn rejects_fractional_zero_and_negative_amounts() {
        for amount in ["", "0", "000", "-1", "1.2", "1e8"] {
            let mut input = sample();
            input.amount = amount.into();
            assert!(quote_payload(&input).is_err());
        }
    }
    #[test]
    fn preview_quote_without_deposit_address_deserializes() {
        let quote: SwapQuote =
            decode(r#"{"amountIn":"123","amountOut":"600000000","newField":true}"#).unwrap();
        assert!(quote.deposit_address.is_none());
    }
}
