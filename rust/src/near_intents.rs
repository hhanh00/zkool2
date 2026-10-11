//! Direct 1Click integration. Amounts are integer strings in asset base units.
//! This module never signs or broadcasts wallet transactions and adds no app fee.
use anyhow::{bail, ensure, Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;

use crate::net::http;

/// HTTP routing settings supplied by the API layer.
pub struct Transport {
    pub mode: u8,
    pub proxy: String,
}

const BASE_URL: &str = "https://1click.chaindefuser.com";
pub const ZEC_ASSET: &str = "nep141:zec.omft.near";
const MAX_RESPONSE: usize = 4 * 1024 * 1024;

/// Persisted quote and latest provider snapshot. Amounts are base-unit strings;
/// timestamps are Unix seconds. Listing never contacts the provider.
#[derive(Clone, Debug)]
pub struct SavedSwap {
    pub id_swap: i64,
    pub account: u32,
    pub origin_asset: String,
    pub destination_asset: String,
    pub swap_type: String,
    pub amount: String,
    pub slippage_tolerance: i32,
    pub recipient: String,
    pub refund_to: String,
    pub deadline: String,
    pub amount_in: String,
    pub amount_out: String,
    pub min_amount_in: Option<String>,
    pub min_amount_out: Option<String>,
    pub deposit_address: String,
    pub deposit_memo: Option<String>,
    pub deposit_tx_hash: Option<String>,
    pub deposit_submitted_at: Option<i64>,
    pub quote_response: String,
    pub status: Option<String>,
    pub status_response: Option<String>,
    pub last_checked_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Expire local unfunded swaps one day after the request deadline, never the deposit
/// address lifetime returned in quote.deadline.
async fn expire_swaps(connection: &mut sqlx::SqliteConnection, account: u32) -> Result<()> {
    sqlx::query(
        "UPDATE swaps SET status = 'EXPIRED', completed_at = unixepoch(), updated_at = unixepoch()
        WHERE account = ? AND completed_at IS NULL AND deposit_tx_hash IS NULL
        AND (status IS NULL OR status = 'PENDING_DEPOSIT')
        AND julianday(COALESCE(
            CASE WHEN json_valid(quote_response) THEN json_extract(quote_response, '$.quoteRequest.deadline') END,
            deadline), '+1 day') <= julianday('now')",
    ).bind(account).execute(connection).await?;
    Ok(())
}

pub(crate) async fn read_swaps(
    connection: &mut sqlx::SqliteConnection,
    account: u32,
    pending_only: bool,
) -> Result<Vec<SavedSwap>> {
    use sqlx::Row;
    expire_swaps(connection, account).await?;
    let rows = sqlx::query(
        "SELECT * FROM swaps WHERE account = ?
        AND (NOT ? OR completed_at IS NULL) ORDER BY created_at DESC, id_swap DESC",
    )
    .bind(account)
    .bind(pending_only)
    .fetch_all(connection)
    .await?;
    rows.into_iter()
        .map(|row| {
            let quote_response: String = row.try_get("quote_response")?;
            // Older rows stored the address lifetime instead of the requested
            // swap deadline. Recover the request from the provider snapshot.
            let deadline = serde_json::from_str::<serde_json::Value>(&quote_response)
                .ok()
                .and_then(|response| {
                    response["quoteRequest"]["deadline"]
                        .as_str()
                        .map(str::to_owned)
                })
                .unwrap_or(row.try_get("deadline")?);
            Ok(SavedSwap {
                id_swap: row.try_get("id_swap")?,
                account: row.try_get("account")?,
                origin_asset: row.try_get("origin_asset")?,
                destination_asset: row.try_get("destination_asset")?,
                swap_type: row.try_get("swap_type")?,
                amount: row.try_get("amount")?,
                slippage_tolerance: row.try_get("slippage_tolerance")?,
                recipient: row.try_get("recipient")?,
                refund_to: row.try_get("refund_to")?,
                deadline,
                amount_in: row.try_get("amount_in")?,
                amount_out: row.try_get("amount_out")?,
                min_amount_in: row.try_get("min_amount_in")?,
                min_amount_out: row.try_get("min_amount_out")?,
                deposit_address: row.try_get("deposit_address")?,
                deposit_memo: row.try_get("deposit_memo")?,
                deposit_tx_hash: row.try_get("deposit_tx_hash")?,
                deposit_submitted_at: row.try_get("deposit_submitted_at")?,
                quote_response,
                status: row.try_get("status")?,
                status_response: row.try_get("status_response")?,
                last_checked_at: row.try_get("last_checked_at")?,
                completed_at: row.try_get("completed_at")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })
        })
        .collect()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "graphql", derive(juniper::GraphQLObject))]
pub struct SwapAsset {
    pub asset_id: String,
    pub decimals: i32,
    pub blockchain: String,
    pub symbol: String,
    #[serde(default, deserialize_with = "deserialize_price")]
    pub price: Option<String>,
    pub contract_address: Option<String>,
}

// Provider prices may be JSON numbers or strings. Keep the public API string
// representation; transfer amounts still require exact integer strings.
fn deserialize_price<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error> {
    match Option::<serde_json::Value>::deserialize(deserializer)? {
        None => Ok(None),
        Some(serde_json::Value::String(value)) => Ok(Some(value)),
        Some(serde_json::Value::Number(value)) => Ok(Some(value.to_string())),
        Some(_) => Err(serde::de::Error::custom(
            "Expected a numeric or string price",
        )),
    }
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
async fn request(
    transport: &Transport,
    method: hyper::Method,
    url: &str,
    body: Vec<u8>,
) -> Result<String> {
    let timeout = Duration::from_secs(30);
    let headers = [("Content-Type".to_owned(), "application/json".to_owned())];
    let (status, bytes) = if transport.mode == 1 {
        let response =
            http::tor_request(method, url, body, &headers, timeout, MAX_RESPONSE).await?;
        (response.status, response.body)
    } else {
        ensure!(
            transport.mode == 0 || transport.mode == 3,
            "1Click HTTP does not support the selected transport"
        );
        ensure!(
            transport.mode != 3 || !transport.proxy.is_empty(),
            "External proxy is not configured"
        );
        let client = http::client(http::proxy_url(transport.mode, &transport.proxy), timeout)?;
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
        bail!("{}", response_error(status, &bytes));
    }
    String::from_utf8(bytes).context("Invalid 1Click response encoding")
}

fn response_error(status: u16, bytes: &[u8]) -> String {
    let body = String::from_utf8_lossy(bytes);
    let body = body.trim();
    let json = serde_json::from_str::<serde_json::Value>(body).ok();
    let detail = json.as_ref().and_then(|value| {
        let message = value.get("message").or_else(|| value.get("error"))?;
        match message {
            serde_json::Value::String(message) if !message.trim().is_empty() => {
                Some(message.clone())
            }
            serde_json::Value::Array(messages) => {
                let messages = messages
                    .iter()
                    .filter_map(|message| message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ");
                if messages.is_empty() {
                    None
                } else {
                    Some(messages)
                }
            }
            _ => None,
        }
    });
    let detail = detail.as_deref().unwrap_or(body);
    if detail.is_empty() {
        format!("1Click request failed with HTTP {status} (empty response body)")
    } else {
        format!("1Click request failed with HTTP {status}: {detail}")
    }
}

fn decode<T: DeserializeOwned>(response: &str) -> Result<T> {
    serde_json::from_str(response).context("Invalid 1Click response")
}

pub async fn assets(transport: &Transport) -> Result<Vec<SwapAsset>> {
    decode(
        &request(
            transport,
            hyper::Method::GET,
            &format!("{BASE_URL}/v0/tokens"),
            vec![],
        )
        .await?,
    )
}

fn validate_zcash_side(request: &SwapRequest) -> Result<bool> {
    let outgoing = request.origin_asset == ZEC_ASSET;
    ensure!(
        outgoing || request.destination_asset == ZEC_ASSET,
        "Swaps must send or receive native ZEC"
    );
    use zcash_protocol::consensus::NetworkType;
    let zcash_address = if outgoing {
        &request.refund_to
    } else {
        &request.recipient
    };
    crate::openalias::try_validate_zcash_address(zcash_address, NetworkType::Main)?;
    Ok(outgoing)
}

pub async fn quote(request_data: SwapRequest, transport: &Transport) -> Result<SwapQuoteResponse> {
    let outgoing = validate_zcash_side(&request_data)?;
    let payload = quote_payload(&request_data)?;
    let raw_response = request(
        transport,
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
        ensure!(!address.trim().is_empty(), "Missing deposit address");
        if outgoing {
            crate::openalias::try_validate_zcash_address(
                address,
                zcash_protocol::consensus::NetworkType::Main,
            )?;
            ensure!(
                response.quote.deposit_memo.is_none(),
                "ZEC deposit unexpectedly requires a memo"
            );
        }
    }
    Ok(SwapQuoteResponse {
        quote: response.quote,
        raw_response,
    })
}

pub async fn status(
    deposit_address: &str,
    deposit_memo: Option<&str>,
    transport: &Transport,
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
    let raw_response = request(transport, hyper::Method::GET, url.as_str(), vec![]).await?;
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

/// Read the snapshot used to guard against concurrent refreshes.
pub(crate) async fn swap_status_snapshot(
    connection: &mut sqlx::SqliteConnection,
    id_swap: i64,
    account: u32,
) -> Result<(String, Option<String>, Option<String>)> {
    sqlx::query_as(
        "SELECT deposit_address, deposit_memo, status_response
        FROM swaps WHERE id_swap = ? AND account = ?",
    )
    .bind(id_swap)
    .bind(account)
    .fetch_optional(connection)
    .await?
    .context("Swap not found for the current account")
}

pub(crate) async fn save_swap(
    connection: &mut sqlx::SqliteConnection,
    account: u32,
    request: &SwapRequest,
    response: &SwapQuoteResponse,
) -> Result<SavedSwap> {
    ensure!(!request.dry, "Preview quotes cannot be saved as swaps");
    let quote = &response.quote;
    let address = quote
        .deposit_address
        .as_deref()
        .context("Missing deposit address")?;
    let result = sqlx::query(
        "INSERT INTO swaps(account, origin_asset, destination_asset, swap_type,
        amount, slippage_tolerance, recipient, refund_to, deadline, amount_in, amount_out,
        min_amount_in, min_amount_out, deposit_address, deposit_memo, quote_response)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(account)
    .bind(&request.origin_asset)
    .bind(&request.destination_asset)
    .bind(match request.swap_type {
        SwapType::ExactInput => "EXACT_INPUT",
        SwapType::ExactOutput => "EXACT_OUTPUT",
    })
    .bind(&request.amount)
    .bind(request.slippage_tolerance)
    .bind(&request.recipient)
    .bind(&request.refund_to)
    .bind(&request.deadline)
    .bind(&quote.amount_in)
    .bind(&quote.amount_out)
    .bind(&quote.min_amount_in)
    .bind(&quote.min_amount_out)
    .bind(address)
    .bind(&quote.deposit_memo)
    .bind(&response.raw_response)
    .execute(&mut *connection)
    .await?;
    read_swaps(connection, account, false)
        .await?
        .into_iter()
        .find(|swap| swap.id_swap == result.last_insert_rowid())
        .context("Saved swap not found")
}

pub(crate) async fn persist_swap_status(
    connection: &mut sqlx::SqliteConnection,
    id_swap: i64,
    account: u32,
    deposit_address: &str,
    deposit_memo: Option<&str>,
    previous_response: Option<&str>,
    current: &SwapStatus,
) -> Result<()> {
    // INCOMPLETE_DEPOSIT may still need refund tracking. New provider states
    // also stay open rather than silently dropping out of the pending index.
    let completed = matches!(current.status.as_str(), "SUCCESS" | "REFUNDED" | "FAILED");
    let result = sqlx::query(
        "UPDATE swaps SET status = ?, status_response = ?,
        last_checked_at = unixepoch(), updated_at = unixepoch(),
        completed_at = CASE WHEN ? THEN COALESCE(completed_at, unixepoch()) ELSE NULL END
        WHERE id_swap = ? AND account = ? AND deposit_address = ?
        AND deposit_memo IS ? AND status_response IS ?",
    )
    .bind(&current.status)
    .bind(&current.raw_response)
    .bind(completed)
    .bind(id_swap)
    .bind(account)
    .bind(deposit_address)
    .bind(deposit_memo)
    .bind(previous_response)
    .execute(&mut *connection)
    .await?;
    // Compare the snapshot read before HTTP, so a slower concurrent refresh
    // cannot overwrite a newer response (or a deleted/replaced row).
    ensure!(
        result.rows_affected() == 1,
        "Swap changed during status refresh; retry"
    );
    expire_swaps(connection, account).await?;
    Ok(())
}

/// Notifies 1Click about an already broadcast deposit; this does not send funds.
pub async fn submit_deposit(
    deposit_address: &str,
    tx_hash: &str,
    transport: &Transport,
) -> Result<()> {
    ensure!(
        !deposit_address.trim().is_empty(),
        "Missing deposit address"
    );
    ensure!(
        tx_hash.len() == 64 && tx_hash.bytes().all(|b| b.is_ascii_hexdigit()),
        "Invalid Zcash transaction hash"
    );
    request(
        transport,
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
    use sqlx::Connection;

    #[tokio::test]
    async fn expired_swaps_use_request_deadline_and_preserve_funded_swaps() -> Result<()> {
        let mut db = sqlx::SqliteConnection::connect("sqlite::memory:").await?;
        sqlx::raw_sql("CREATE TABLE accounts(id_account INTEGER PRIMARY KEY); INSERT INTO accounts VALUES (1), (2);")
            .execute(&mut db).await?;
        sqlx::raw_sql(include_str!("db/swaps.sql"))
            .execute(&mut db)
            .await?;
        for id in 1..=7 {
            let mut request = sample();
            request.dry = false;
            let response = SwapQuoteResponse {
                quote: decode(&format!(
                    r#"{{"amountIn":"100","amountOut":"200","depositAddress":"deposit-{id}"}}"#
                ))?,
                raw_response: if id == 1 {
                    r#"{"quoteRequest":{"deadline":"2000-01-01T00:00:00Z"},"quote":{"deadline":"2099-01-01T00:00:00Z"}}"#.into()
                } else {
                    "{}".into()
                },
            };
            save_swap(&mut db, if id == 7 { 2 } else { 1 }, &request, &response).await?;
        }
        sqlx::query("UPDATE swaps SET deadline = '2000-01-01T00:00:00Z', status = 'PENDING_DEPOSIT' WHERE id_swap IN (2, 3, 4, 5, 7)").execute(&mut db).await?;
        sqlx::query("UPDATE swaps SET deposit_tx_hash = ? WHERE id_swap = 3")
            .bind("a".repeat(64))
            .execute(&mut db)
            .await?;
        sqlx::query("UPDATE swaps SET status = 'PROCESSING' WHERE id_swap = 4")
            .execute(&mut db)
            .await?;
        sqlx::query("UPDATE swaps SET status = 'SUCCESS', completed_at = 123 WHERE id_swap = 5")
            .execute(&mut db)
            .await?;
        // Still within the one-day grace period.
        sqlx::query("UPDATE swaps SET deadline = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-12 hours') WHERE id_swap = 6").execute(&mut db).await?;
        let swaps = read_swaps(&mut db, 1, false).await?;
        for id in [1, 2] {
            let swap = swaps.iter().find(|s| s.id_swap == id).unwrap();
            assert_eq!(swap.status.as_deref(), Some("EXPIRED"));
            assert!(swap.completed_at.is_some());
        }
        let pending = read_swaps(&mut db, 1, true).await?;
        assert_eq!(pending.len(), 3); // funded, processing, within grace period
        let other: Option<String> =
            sqlx::query_scalar("SELECT status FROM swaps WHERE id_swap = 7")
                .fetch_one(&mut db)
                .await?;
        assert_eq!(other.as_deref(), Some("PENDING_DEPOSIT"));
        assert_eq!(
            swaps.iter().find(|s| s.id_swap == 5).unwrap().completed_at,
            Some(123)
        );
        Ok(())
    }

    #[test]
    fn provider_errors_preserve_reasons() {
        assert_eq!(
            response_error(400, br#"{"message":"Amount is too low","statusCode":400}"#),
            "1Click request failed with HTTP 400: Amount is too low"
        );
        assert_eq!(
            response_error(
                400,
                br#"{"message":["Invalid recipient","Invalid refund address"]}"#
            ),
            "1Click request failed with HTTP 400: Invalid recipient; Invalid refund address"
        );
        assert_eq!(
            response_error(400, br#"{"error":"No route available"}"#),
            "1Click request failed with HTTP 400: No route available"
        );
        assert_eq!(
            response_error(502, b" upstream unavailable "),
            "1Click request failed with HTTP 502: upstream unavailable"
        );
        assert_eq!(
            response_error(400, br#"{"reason":"Unknown format"}"#),
            "1Click request failed with HTTP 400: {\"reason\":\"Unknown format\"}"
        );
        assert_eq!(
            response_error(400, b""),
            "1Click request failed with HTTP 400 (empty response body)"
        );
    }

    #[tokio::test]
    async fn confirmed_quote_persists_request_and_provider_snapshot() -> Result<()> {
        let mut db = sqlx::SqliteConnection::connect("sqlite::memory:").await?;
        sqlx::raw_sql(
            "CREATE TABLE accounts(id_account INTEGER PRIMARY KEY);
            INSERT INTO accounts VALUES (1);",
        )
        .execute(&mut db)
        .await?;
        sqlx::raw_sql(include_str!("db/swaps.sql"))
            .execute(&mut db)
            .await?;
        let mut request = sample();
        let raw = r#"{"quote":{"amountIn":"123456789","amountOut":"600000000","depositAddress":"deposit","deadline":"2026-10-13T12:00:00Z","minAmountOut":"590000000"},"signature":"provider-signature"}"#;
        let response = SwapQuoteResponse {
            quote: decode(&serde_json::from_str::<serde_json::Value>(raw)?["quote"].to_string())?,
            raw_response: raw.into(),
        };
        assert!(save_swap(&mut db, 1, &request, &response).await.is_err());
        request.dry = false;
        let saved = save_swap(&mut db, 1, &request, &response).await?;
        assert_eq!(saved.account, 1);
        assert_eq!(saved.swap_type, "EXACT_OUTPUT");
        assert_eq!(saved.amount, request.amount);
        assert_eq!(saved.deadline, request.deadline);
        assert_eq!(saved.amount_in, "123456789");
        assert_eq!(saved.amount_out, "600000000");
        assert_eq!(saved.min_amount_out.as_deref(), Some("590000000"));
        assert_eq!(saved.recipient, request.recipient);
        assert_eq!(saved.refund_to, request.refund_to);
        assert_eq!(saved.quote_response, raw);
        assert!(saved.status.is_none() && saved.deposit_tx_hash.is_none());
        assert!(save_swap(&mut db, 1, &request, &response).await.is_err());
        let mut legacy_response: serde_json::Value = serde_json::from_str(raw)?;
        legacy_response["quoteRequest"] = serde_json::json!({"deadline": request.deadline});
        sqlx::query("UPDATE swaps SET deadline = ?, quote_response = ? WHERE id_swap = ?")
            .bind("2026-10-13T12:00:00Z")
            .bind(legacy_response.to_string())
            .bind(saved.id_swap)
            .execute(&mut db)
            .await?;
        assert_eq!(
            read_swaps(&mut db, 1, false).await?[0].deadline,
            request.deadline
        );
        Ok(())
    }

    #[test]
    fn token_prices_accept_numbers_strings_null_and_missing() {
        for (price, expected) in [
            (Some(serde_json::json!(5.27)), Some("5.27")),
            (Some(serde_json::json!(1)), Some("1")),
            (Some(serde_json::json!("5.27")), Some("5.27")),
            (Some(serde_json::Value::Null), None),
            (None, None),
        ] {
            let mut token = serde_json::json!({"assetId": "zec", "decimals": 8,
                "blockchain": "zec", "symbol": "ZEC"});
            if let Some(price) = price {
                token["price"] = price;
            }
            let assets: Vec<SwapAsset> = decode(&serde_json::json!([token]).to_string()).unwrap();
            assert_eq!(assets[0].price.as_deref(), expected);
        }
        let token =
            r#"[{"assetId":"zec","decimals":8,"blockchain":"zec","symbol":"ZEC","price":true}]"#;
        assert!(decode::<Vec<SwapAsset>>(token).is_err());
        assert!(decode::<SwapQuote>(r#"{"amountIn":5.27,"amountOut":"1"}"#).is_err());
    }

    #[tokio::test]
    async fn list_saved_swaps_filters_account_and_pending_in_stable_order() -> Result<()> {
        let mut db = sqlx::SqliteConnection::connect("sqlite::memory:").await?;
        sqlx::raw_sql(
            "CREATE TABLE accounts(id_account INTEGER PRIMARY KEY);
            INSERT INTO accounts VALUES (1), (2);",
        )
        .execute(&mut db)
        .await?;
        sqlx::raw_sql(include_str!("db/swaps.sql"))
            .execute(&mut db)
            .await?;
        for (id, account, created, completed) in [
            (1, 1, 10, None),
            (2, 1, 20, Some(30)),
            (3, 1, 20, None),
            (4, 2, 40, None),
        ] {
            sqlx::query(
                "INSERT INTO swaps(id_swap, account, origin_asset, destination_asset,
                swap_type, amount, slippage_tolerance, recipient, refund_to, deadline,
                amount_in, amount_out, deposit_address, quote_response, created_at, completed_at)
                VALUES (?, ?, 'zec', 'eth', 'EXACT_INPUT', '100', 100, 'recipient',
                'refund', 'deadline', '100', '123456789012345678901234567890', ?, '{}', ?, ?)",
            )
            .bind(id)
            .bind(account)
            .bind(format!("deposit-{id}"))
            .bind(created)
            .bind(completed)
            .execute(&mut db)
            .await?;
        }
        let all = read_swaps(&mut db, 1, false).await?;
        assert_eq!(
            all.iter().map(|s| s.id_swap).collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
        assert_eq!(all[0].amount_out, "123456789012345678901234567890");
        assert!(all[0].status.is_none() && all[0].deposit_memo.is_none());
        let pending = read_swaps(&mut db, 1, true).await?;
        assert_eq!(
            pending.iter().map(|s| s.id_swap).collect::<Vec<_>>(),
            vec![3, 1]
        );
        assert_eq!(read_swaps(&mut db, 2, false).await?[0].id_swap, 4);
        assert!(read_swaps(&mut db, 99, false).await?.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn saved_status_tracks_completion_and_rejects_stale_or_wrong_account_updates(
    ) -> Result<()> {
        let mut db = sqlx::SqliteConnection::connect("sqlite::memory:").await?;
        sqlx::raw_sql(
            "CREATE TABLE accounts(id_account INTEGER PRIMARY KEY);
            INSERT INTO accounts VALUES (1), (2);",
        )
        .execute(&mut db)
        .await?;
        sqlx::raw_sql(include_str!("db/swaps.sql"))
            .execute(&mut db)
            .await?;
        sqlx::query(
            "INSERT INTO swaps(id_swap, account, origin_asset, destination_asset,
            swap_type, amount, slippage_tolerance, recipient, refund_to, deadline,
            amount_in, amount_out, deposit_address, deposit_memo, quote_response, updated_at)
            VALUES (1, 1, 'zec', 'eth', 'EXACT_INPUT', '100', 100, 'recipient',
            'refund', 'deadline', '100', '200', 'deposit', 'memo', 'original quote', 1)",
        )
        .execute(&mut db)
        .await?;
        let mut previous: Option<String> = None;
        for state in [
            "KNOWN_DEPOSIT_TX",
            "PENDING_DEPOSIT",
            "INCOMPLETE_DEPOSIT",
            "PROCESSING",
            "NEW_STATE",
            "SUCCESS",
            "REFUNDED",
            "FAILED",
        ] {
            let raw = serde_json::json!({"status": state, "swapDetails": {
                "amountOut": "190", "destinationChainTxHashes": [{"hash": "destination"}]
            }})
            .to_string();
            let current = SwapStatus {
                status: state.into(),
                swap_details: None,
                raw_response: raw.clone(),
            };
            assert!(persist_swap_status(
                &mut db,
                1,
                2,
                "deposit",
                Some("memo"),
                previous.as_deref(),
                &current
            )
            .await
            .is_err());
            assert!(persist_swap_status(
                &mut db,
                1,
                1,
                "deposit",
                None,
                previous.as_deref(),
                &current
            )
            .await
            .is_err());
            persist_swap_status(
                &mut db,
                1,
                1,
                "deposit",
                Some("memo"),
                previous.as_deref(),
                &current,
            )
            .await?;
            let row: (String, String, Option<i64>, i64, i64, String, String) = sqlx::query_as(
                "SELECT status, status_response, completed_at, last_checked_at,
                updated_at, amount_out, quote_response FROM swaps WHERE id_swap = 1",
            )
            .fetch_one(&mut db)
            .await?;
            assert_eq!(row.0, state);
            assert_eq!(row.1, raw);
            assert_eq!(
                row.2.is_some(),
                matches!(state, "SUCCESS" | "REFUNDED" | "FAILED")
            );
            assert!(row.3 > 1 && row.4 >= row.3);
            assert_eq!(row.5, "200");
            assert_eq!(row.6, "original quote");
            // An older in-flight result cannot replace the new snapshot.
            assert!(persist_swap_status(
                &mut db,
                1,
                1,
                "deposit",
                Some("memo"),
                previous.as_deref(),
                &current
            )
            .await
            .is_err());
            previous = Some(raw);
        }
        // Polling an already finished swap retains its first completion time.
        sqlx::query("UPDATE swaps SET completed_at = 123 WHERE id_swap = 1")
            .execute(&mut db)
            .await?;
        let current = SwapStatus {
            status: "FAILED".into(),
            swap_details: None,
            raw_response: previous.clone().unwrap(),
        };
        persist_swap_status(
            &mut db,
            1,
            1,
            "deposit",
            Some("memo"),
            previous.as_deref(),
            &current,
        )
        .await?;
        let (completed,): (i64,) =
            sqlx::query_as("SELECT completed_at FROM swaps WHERE id_swap = 1")
                .fetch_one(&mut db)
                .await?;
        assert_eq!(completed, 123);
        assert!(
            persist_swap_status(&mut db, 99, 1, "deposit", Some("memo"), None, &current)
                .await
                .is_err()
        );
        Ok(())
    }

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
            deadline: "2099-10-10T12:00:00Z".into(),
        }
    }
    #[test]
    fn validates_zcash_refund_for_outgoing_and_recipient_for_incoming() {
        let address = "t1VmmGiyjVNeCjxDZzg7vZmd99WyzVby9yC";
        let mut request = sample();
        request.refund_to = address.into();
        assert!(validate_zcash_side(&request).unwrap());
        request.origin_asset = "usdt".into();
        request.destination_asset = ZEC_ASSET.into();
        request.refund_to = "external-refund".into();
        request.recipient = address.into();
        assert!(!validate_zcash_side(&request).unwrap());
        let payload = quote_payload(&request).unwrap();
        assert_eq!(payload["originAsset"], "usdt");
        assert_eq!(payload["destinationAsset"], ZEC_ASSET);
        assert_eq!(payload["refundTo"], "external-refund");
        request.recipient = "invalid".into();
        assert!(validate_zcash_side(&request).is_err());
        request.recipient = address.into();
        request.destination_asset = "eth".into();
        assert!(validate_zcash_side(&request).is_err());
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
