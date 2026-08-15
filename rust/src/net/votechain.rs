//! Minimal REST client for the vote-sdk `/shielded-vote/v1` API surface.
//!
//! Chain-facing calls use the configured vote server base URL; helper-server
//! share calls take an explicit server URL because foreground submission and
//! recovery may target different helper subsets over time.
//!
//! JSON envelopes are returned as raw bodies — the vote-sdk schema is still
//! evolving and the Dart UI parses leniently (mirroring vizor's client).
//! HTTP status is preserved for 2xx, 404, and 422 (422 = deterministic chain
//! rejection whose body is a `VotingTxResult`), so the UI can distinguish a
//! rejection from a transport error; only network failures produce `Err`.

use anyhow::{anyhow, Result};
use std::time::Duration;

/// Returns the status code and body of a GET, without erroring on 404.
async fn get(base_url: &str, path: &str, proxy: &str) -> Result<(u16, String)> {
    let url = endpoint(base_url, path)?;
    let response = client(proxy, Duration::from_secs(15))?
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("vote chain GET {url}: {e}"))?;
    let status = response.status().as_u16();
    let body = response.text().await?;
    if status != 404 && (status < 200 || status >= 300) {
        return Err(anyhow!("vote chain GET {url}: HTTP {status}: {body}"));
    }
    Ok((status, body))
}

/// Returns the status code and body of a POST, without erroring on 422
/// (deterministic chain rejection).
async fn post(base_url: &str, path: &str, body_json: &str, proxy: &str) -> Result<(u16, String)> {
    let url = endpoint(base_url, path)?;
    let response = client(proxy, Duration::from_secs(60))?
        .post(&url)
        .header("content-type", "application/json")
        .body(body_json.to_string())
        .send()
        .await
        .map_err(|e| anyhow!("vote chain POST {url}: {e}"))?;
    let status = response.status().as_u16();
    let body = response.text().await?;
    if status != 422 && (status < 200 || status >= 300) {
        return Err(anyhow!("vote chain POST {url}: HTTP {status}: {body}"));
    }
    Ok((status, body))
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
pub async fn list_rounds(base_url: &str, proxy: &str) -> Result<(u16, String)> {
    get(base_url, "rounds", proxy).await
}

/// Fetches one round's status (`{ "round": ... }` envelope).
pub async fn round_status(base_url: &str, round_id: &str, proxy: &str) -> Result<(u16, String)> {
    get(base_url, &format!("round/{round_id}"), proxy).await
}

/// Fetches the round tally envelope (`tally-results`).
pub async fn round_tally(base_url: &str, round_id: &str, proxy: &str) -> Result<(u16, String)> {
    get(base_url, &format!("tally-results/{round_id}"), proxy).await
}

/// Broadcasts a delegation transaction to the vote chain.
pub async fn submit_delegation(
    base_url: &str,
    submission_json: &str,
    proxy: &str,
) -> Result<(u16, String)> {
    post(base_url, "delegate-vote", submission_json, proxy).await
}

/// Broadcasts a vote commitment transaction to the vote chain.
pub async fn submit_vote_commitment(
    base_url: &str,
    commitment_json: &str,
    proxy: &str,
) -> Result<(u16, String)> {
    post(base_url, "cast-vote", commitment_json, proxy).await
}

/// Fetches the on-chain confirmation for a transaction; 404 = not confirmed.
pub async fn tx_confirmation(
    base_url: &str,
    tx_hash: &str,
    proxy: &str,
) -> Result<(u16, String)> {
    get(base_url, &format!("tx/{tx_hash}"), proxy).await
}

/// Posts one encrypted share to a helper server. The payload must already
/// carry the `vote_round_id` field required by the helper API.
pub async fn submit_share(
    server_url: &str,
    payload_json: &str,
    proxy: &str,
) -> Result<(u16, String)> {
    post(server_url, "shares", payload_json, proxy).await
}

/// Checks whether a helper has confirmed a share identified by its nullifier.
pub async fn share_status(
    server_url: &str,
    round_id: &str,
    share_id: &str,
    proxy: &str,
) -> Result<(u16, String)> {
    get(server_url, &format!("share-status/{round_id}/{share_id}"), proxy).await
}

/// Fetches raw bytes from an arbitrary URL (voting config blobs).
pub async fn fetch_bytes(url: &str, proxy: &str) -> Result<Vec<u8>> {
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
