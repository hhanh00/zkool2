//! Shared HTTP plumbing: the wallet's standard reqwest client and one retry
//! policy for idempotent GETs.

use anyhow::{anyhow, Result};
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

/// Retry policy for idempotent GETs: up to [RetryPolicy::attempts] total
/// tries, sleeping `base_delay * rate^(n-1)` before every try `n`. The
/// rate must be >= 1.0.
#[derive(Clone, Copy, Debug)]
pub struct RetryPolicy {
    pub attempts: u32,
    /// Choose a random first URL before rotating through the list.
    pub randomize_first: bool,
    pub base_delay: Duration,
    pub rate: f64,
}

impl RetryPolicy {
    /// Sets whether to choose a random first URL.
    pub fn randomize_first(mut self, randomize_first: bool) -> Self {
        self.randomize_first = randomize_first;
        self
    }
}

impl Default for RetryPolicy {
    /// Three tries, waiting 500 ms, 1 s, then 2 s before each try.
    fn default() -> Self {
        Self {
            attempts: 3,
            randomize_first: true,
            base_delay: Duration::from_millis(500),
            rate: 2.0,
        }
    }
}

/// Resolves the external proxy URL for the selected transport.
/// Transport 3 uses the configured proxy; other transports return an empty URL.
pub fn proxy_url(transport: u8, proxy: &str) -> &str {
    if transport == 3 {
        proxy
    } else {
        ""
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

/// GETs candidate URLs in round-robin order, advancing after each transport
/// failure or HTTP 4xx/5xx response and wrapping after the last URL.
/// [RetryPolicy::attempts] limits total requests across all URLs. Each call
/// starts at the first URL unless `policy.randomize_first` selects a random starting
/// index. Backoff and the last-error behavior are unchanged.
pub async fn http_get<S: AsRef<str>>(
    client: &reqwest::Client,
    urls: &[S],
    policy: RetryPolicy,
) -> Result<reqwest::Response> {
    if urls.is_empty() {
        return Err(anyhow!("GET requires at least one URL"));
    }
    let start = if policy.randomize_first {
        rand::thread_rng().gen_range(0, urls.len())
    } else {
        0
    };
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 1..=policy.attempts.max(1) {
        let url = urls[(start + (attempt - 1) as usize) % urls.len()].as_ref();
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

#[cfg(test)]
mod tests {
    use super::{http_get, proxy_url, RetryPolicy};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn immediate_policy(attempts: u32) -> RetryPolicy {
        RetryPolicy {
            attempts,
            randomize_first: false,
            base_delay: Duration::ZERO,
            rate: 1.0,
        }
    }

    #[tokio::test]
    async fn rejects_empty_url_list() {
        let client = reqwest::Client::new();
        let error = http_get(&client, &[] as &[&str], immediate_policy(3))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("at least one URL"));
    }

    #[tokio::test]
    async fn rotates_and_wraps_after_failed_requests() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let urls = [format!("{base}/first"), format!("{base}/second")];
        let server = tokio::spawn(async move {
            for (path, status) in [
                ("first", "503 Unavailable"),
                ("second", "500 Error"),
                ("first", "200 OK"),
            ] {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    request.push(socket.read_u8().await.unwrap());
                }
                assert!(request.starts_with(format!("GET /{path} HTTP/1.1\r\n").as_bytes()));
                socket
                    .write_all(
                        format!(
                            "HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                        )
                        .as_bytes(),
                    )
                    .await
                    .unwrap();
            }
        });
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let response = tokio::time::timeout(
            Duration::from_secs(5),
            http_get(&client, &urls, immediate_policy(3)),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn randomized_start_preserves_round_robin_order() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let urls: Vec<_> = (0..3).map(|i| format!("{base}/{i}")).collect();
        let server = tokio::spawn(async move {
            let mut indices = Vec::new();
            for attempt in 0..4 {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    request.push(socket.read_u8().await.unwrap());
                }
                let request = String::from_utf8(request).unwrap();
                let index: usize = request
                    .split_whitespace()
                    .nth(1)
                    .unwrap()
                    .trim_start_matches('/')
                    .parse()
                    .unwrap();
                indices.push(index);
                let status = if attempt == 3 {
                    "200 OK"
                } else {
                    "503 Unavailable"
                };
                socket
                    .write_all(
                        format!(
                            "HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                        )
                        .as_bytes(),
                    )
                    .await
                    .unwrap();
            }
            for pair in indices.windows(2) {
                assert_eq!(pair[1], (pair[0] + 1) % 3);
            }
        });
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        tokio::time::timeout(
            Duration::from_secs(5),
            http_get(
                &client,
                &urls,
                RetryPolicy {
                    randomize_first: true,
                    ..immediate_policy(4)
                },
            ),
        )
        .await
        .unwrap()
        .unwrap();
        server.await.unwrap();
    }

    #[tokio::test]
    async fn advances_after_transport_failure_and_returns_last_error() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                request.push(socket.read_u8().await.unwrap());
            }
            socket
                .write_all(
                    b"HTTP/1.1 503 Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await
                .unwrap();
        });
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let error = tokio::time::timeout(
            Duration::from_secs(5),
            http_get(&client, &["invalid-url", &url], immediate_policy(2)),
        )
        .await
        .unwrap()
        .unwrap_err();
        assert!(error.to_string().contains(&url));
        assert!(error.to_string().contains("503"));
        server.await.unwrap();
    }

    #[test]
    fn external_proxy_only_for_transport_three() {
        assert_eq!(
            proxy_url(3, "socks5://127.0.0.1:1080"),
            "socks5://127.0.0.1:1080"
        );
        assert_eq!(proxy_url(3, ""), "");
        for transport in 0..3 {
            assert_eq!(proxy_url(transport, "socks5://x"), "");
        }
    }
}
