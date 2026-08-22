//! Minimal REST client for the vote-sdk `/shielded-vote/v1` API surface.
//!
//! Chain-facing calls use the configured vote server base URL; helper-server
//! share calls take an explicit server URL because foreground submission and
//! recovery may target different helper subsets over time.
//!
//! JSON envelopes are returned as raw bodies — the vote-sdk schema is still
//! evolving and the Dart UI parses leniently (mirroring vizor's client).
//! Every completed HTTP response is an answer: the status code, body, and
//! `Retry-After` header are passed through unchanged (422 = deterministic
//! chain rejection whose body is a `VotingTxResult`; 404 = not found; 5xx =
//! server-side trouble the caller may fail over from). Only transport
//! failures produce `Err`.
//!
//! Transient failures retry in place: idempotent GETs retry on any transport
//! error, and POSTs retry only when the request never reached the server (a
//! connection error), so a resent submission cannot double-deliver. Failover
//! across the configured vote servers is the Dart caller's job.

use anyhow::{anyhow, Result};
use std::time::Duration;
use tokio::time::sleep;

/// Attempts for idempotent vote-chain reads. A gateway 500 or a dropped
/// connection must not fail a whole voting run.
const GET_ATTEMPTS: usize = 3;
/// Attempts for vote-chain writes. Only connection failures retry — the
/// request never reached the server — anything else (including a timeout
/// after the request was sent) is final for this server.
const POST_ATTEMPTS: usize = 2;
/// Backoff between in-place retry attempts.
const RETRY_DELAY: Duration = Duration::from_millis(500);

/// Returns the status code, body, and `Retry-After` of a GET. Every completed
/// HTTP response is an answer; only transport failures are `Err`.
async fn get(base_url: &str, path: &str, proxy: &str) -> Result<(u16, String, Option<u64>)> {
    let url = endpoint(base_url, path)?;
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 0..GET_ATTEMPTS {
        if attempt > 0 {
            sleep(RETRY_DELAY * attempt as u32).await;
        }
        match get_once(&url, proxy).await {
            Ok(response) => return Ok(response),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.expect("GET_ATTEMPTS is at least 1"))
}

/// One GET attempt; 404 and other non-2xx statuses are answers, not errors.
async fn get_once(url: &str, proxy: &str) -> Result<(u16, String, Option<u64>)> {
    let response = client(proxy, Duration::from_secs(15))?
        .get(url)
        .send()
        .await
        .map_err(|e| anyhow!("vote chain GET {url}: {e}"))?;
    let status = response.status().as_u16();
    let retry_after = retry_after_secs(response.headers());
    let body = response.text().await?;
    Ok((status, body, retry_after))
}

/// Returns the status code, body, and `Retry-After` of a POST. Every completed
/// HTTP response (including 422 deterministic rejections and 5xx) is an
/// answer; only transport failures are `Err`.
async fn post(
    base_url: &str,
    path: &str,
    body_json: &str,
    proxy: &str,
) -> Result<(u16, String, Option<u64>)> {
    let url = endpoint(base_url, path)?;
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 0..POST_ATTEMPTS {
        if attempt > 0 {
            sleep(RETRY_DELAY).await;
        }
        let send = client(proxy, Duration::from_secs(60))?
            .post(&url)
            .header("content-type", "application/json")
            .body(body_json.to_string())
            .send()
            .await;
        let response = match send {
            Ok(response) => response,
            Err(e) => {
                let err = anyhow!("vote chain POST {url}: {e}");
                if e.is_connect() {
                    last_err = Some(err);
                    continue;
                }
                return Err(err);
            }
        };
        let status = response.status().as_u16();
        let retry_after = retry_after_secs(response.headers());
        let body = response.text().await?;
        return Ok((status, body, retry_after));
    }
    Err(last_err.expect("POST_ATTEMPTS is at least 1"))
}

/// Parses a `Retry-After` header in seconds (the vote-sdk sends an integer
/// delay; HTTP-date values are ignored).
fn retry_after_secs(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|raw| raw.trim().parse::<u64>().ok())
}

/// Builds a `/shielded-vote/v1/...` URL under `base_url`.
fn endpoint(base_url: &str, path: &str) -> Result<String> {
    let base = base_url.trim_end_matches('/');
    Ok(format!("{base}/shielded-vote/v1/{path}"))
}

fn client(proxy: &str, timeout: Duration) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .user_agent("zkool/1.0")
        .timeout(timeout);
    if !proxy.is_empty() {
        builder = builder.proxy(reqwest::Proxy::all(proxy)?);
    }
    Ok(builder.build()?)
}

/// Lists rounds from the vote server. Current vote-sdk returns
/// `{ "rounds": [...] }`; an empty `{}` means no rounds.
pub async fn list_rounds(base_url: &str, proxy: &str) -> Result<(u16, String, Option<u64>)> {
    get(base_url, "rounds", proxy).await
}

/// Fetches one round's status (`{ "round": ... }` envelope).
pub async fn round_status(
    base_url: &str,
    round_id: &str,
    proxy: &str,
) -> Result<(u16, String, Option<u64>)> {
    get(base_url, &format!("round/{round_id}"), proxy).await
}

/// Fetches the round tally envelope (`tally-results`).
pub async fn round_tally(
    base_url: &str,
    round_id: &str,
    proxy: &str,
) -> Result<(u16, String, Option<u64>)> {
    get(base_url, &format!("tally-results/{round_id}"), proxy).await
}

/// Broadcasts a delegation transaction to the vote chain.
pub async fn submit_delegation(
    base_url: &str,
    submission_json: &str,
    proxy: &str,
) -> Result<(u16, String, Option<u64>)> {
    post(base_url, "delegate-vote", submission_json, proxy).await
}

/// Broadcasts a vote commitment transaction to the vote chain.
pub async fn submit_vote_commitment(
    base_url: &str,
    commitment_json: &str,
    proxy: &str,
) -> Result<(u16, String, Option<u64>)> {
    post(base_url, "cast-vote", commitment_json, proxy).await
}

/// Fetches the on-chain confirmation for a transaction; 404 = not confirmed.
pub async fn tx_confirmation(
    base_url: &str,
    tx_hash: &str,
    proxy: &str,
) -> Result<(u16, String, Option<u64>)> {
    get(base_url, &format!("tx/{tx_hash}"), proxy).await
}

/// Posts one encrypted share to a helper server. The payload must already
/// carry the `vote_round_id` field required by the helper API.
pub async fn submit_share(
    server_url: &str,
    payload_json: &str,
    proxy: &str,
) -> Result<(u16, String, Option<u64>)> {
    post(server_url, "shares", payload_json, proxy).await
}

/// Checks whether a helper has confirmed a share identified by its nullifier.
pub async fn share_status(
    server_url: &str,
    round_id: &str,
    share_id: &str,
    proxy: &str,
) -> Result<(u16, String, Option<u64>)> {
    get(server_url, &format!("share-status/{round_id}/{share_id}"), proxy).await
}

/// Fetches raw bytes from an arbitrary URL (voting config blobs). Idempotent,
/// so it retries like [get].
pub async fn fetch_bytes(url: &str, proxy: &str) -> Result<Vec<u8>> {
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 0..GET_ATTEMPTS {
        if attempt > 0 {
            sleep(RETRY_DELAY * attempt as u32).await;
        }
        match fetch_bytes_once(url, proxy).await {
            Ok(bytes) => return Ok(bytes),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.expect("GET_ATTEMPTS is at least 1"))
}

/// One config-blob fetch attempt.
async fn fetch_bytes_once(url: &str, proxy: &str) -> Result<Vec<u8>> {
    let response = client(proxy, Duration::from_secs(15))?
        .get(url)
        .send()
        .await
        .map_err(|e| anyhow!("config fetch {url}: {e}"))?;
    let status = response.status().as_u16();
    if status < 200 || status >= 300 {
        return Err(anyhow!("config fetch {url}: HTTP {status}"));
    }
    Ok(response.bytes().await?.to_vec())
}
