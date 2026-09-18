//! Shared HTTP plumbing: the wallet's standard reqwest client and one retry
//! policy for idempotent GETs.

use anyhow::{anyhow, Result};
use std::time::Duration;
use tokio::time::sleep;

/// Retry policy for idempotent GETs: up to [RetryPolicy::attempts] total
/// tries, sleeping `base_delay * rate^(n-1)` before every try `n`. The
/// rate must be >= 1.0.
#[derive(Clone, Copy, Debug)]
pub struct RetryPolicy {
    pub attempts: u32,
    pub base_delay: Duration,
    pub rate: f64,
}

impl Default for RetryPolicy {
    /// Three tries, waiting 500 ms, 1 s, then 2 s before each try.
    fn default() -> Self {
        Self {
            attempts: 3,
            base_delay: Duration::from_millis(500),
            rate: 2.0,
        }
    }
}

/// Builds the wallet's reqwest client: user agent, per-request timeout, and
/// an optional proxy URL ("http://…", "socks5h://…"; empty = direct).
pub fn client(proxy: &str, timeout: Duration) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .user_agent("zkool/1.0")
        .timeout(timeout);
    if !proxy.is_empty() {
        builder = builder.proxy(reqwest::Proxy::all(proxy)?);
    }
    Ok(builder.build()?)
}

/// GETs `url` with `client`, retrying transport failures and HTTP 4xx/5xx
/// responses with exponential backoff per [RetryPolicy]. The last error is
/// returned after the final attempt.
pub async fn http_get(
    client: &reqwest::Client,
    url: &str,
    policy: RetryPolicy,
) -> Result<reqwest::Response> {
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 1..=policy.attempts.max(1) {
        sleep(
            policy
                .base_delay
                .mul_f64(policy.rate.powi((attempt - 1) as i32)),
        )
        .await;
        match client
            .get(url)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
        {
            Ok(response) => return Ok(response),
            Err(e) => last_err = Some(anyhow!("GET {url}: {e}")),
        }
    }
    Err(last_err.expect("attempts is at least 1"))
}
