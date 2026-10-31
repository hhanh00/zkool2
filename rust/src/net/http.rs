//! Shared HTTP plumbing: the wallet's standard reqwest client, the Tor
//! request primitive, the [`Route`] that applies the selected transport to
//! a GET, and one retry policy for idempotent GETs.

use anyhow::{anyhow, Result};
use http_body_util::{BodyExt, Full};
use hyper::{
    body::Bytes,
    rt::{Read, ReadBufCursor, Write},
    Method, Request, Uri,
};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::{
    client::legacy::{
        connect::{Connected, Connection},
        Client as HyperClient,
    },
    rt::{TokioExecutor, TokioIo},
};
use rand::RngExt;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::sleep;
use tower::service_fn;

/// A bounded response returned by the Arti-backed HTTP client.
///
/// This is intentionally owned: the Tor stream can be released immediately
/// after the complete response has been read, and callers never accidentally
/// fall back to a direct reqwest connection while consuming its body.
pub struct TorHttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
    pub content_type: Option<String>,
}

/// Gives Arti's async stream the connection metadata Hyper's pooled client
/// requires. I/O is delegated unchanged to `TokioIo`.
struct TorConnection(TokioIo<arti_client::DataStream>);

impl Connection for TorConnection {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

impl Read for TorConnection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: ReadBufCursor<'_>,
    ) -> Poll<std::io::Result<()>> {
        // `TorConnection` is a transparent newtype and never moves its inner
        // pinned I/O object after it has been pinned.
        unsafe { self.map_unchecked_mut(|connection| &mut connection.0) }.poll_read(cx, buf)
    }
}

impl Write for TorConnection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        unsafe { self.map_unchecked_mut(|connection| &mut connection.0) }.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        unsafe { self.map_unchecked_mut(|connection| &mut connection.0) }.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        unsafe { self.map_unchecked_mut(|connection| &mut connection.0) }.poll_shutdown(cx)
    }
}

/// Performs a regular HTTP(S) GET over an isolated Arti circuit.
///
/// It is the transport primitive used when a caller has selected Tor. It
/// keeps DNS resolution inside Tor and applies the deadline to connect,
/// request, and body read.
pub async fn tor_request(
    method: Method,
    url: &str,
    body: Vec<u8>,
    headers: &[(String, String)],
    timeout: Duration,
    max_response_bytes: usize,
) -> Result<TorHttpResponse> {
    let uri: Uri = url.parse()?;
    let host = uri
        .host()
        .ok_or_else(|| anyhow!("HTTP URL has no host"))?
        .to_owned();
    let port = uri.port_u16().unwrap_or_else(|| {
        if uri.scheme_str() == Some("https") {
            443
        } else {
            80
        }
    });
    let connector = service_fn(move |_uri: Uri| {
        let host = host.clone();
        async move {
            let tor = crate::api::coin::get_tor_client().await.lock().await;
            let stream = tor.isolated_client().connect((host.as_str(), port)).await?;
            Ok::<_, anyhow::Error>(TorConnection(TokioIo::new(stream)))
        }
    });
    let https = HttpsConnectorBuilder::new()
        .with_native_roots()?
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .wrap_connector(connector);
    let client: HyperClient<_, Full<Bytes>> =
        HyperClient::builder(TokioExecutor::new()).build(https);
    let mut request = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        request = request.header(name, value);
    }
    let request = request.body(Full::new(Bytes::from(body)))?;
    let response = tokio::time::timeout(timeout, client.request(request))
        .await
        .map_err(|_| anyhow!("Tor HTTP request timed out"))??;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(hyper::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    if response
        .headers()
        .get(hyper::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|length| length > max_response_bytes)
    {
        return Err(anyhow!("Tor HTTP response exceeds the body limit"));
    }
    let mut response_body = response.into_body();
    let mut response_bytes = Vec::new();
    while let Some(frame) = tokio::time::timeout(timeout, response_body.frame())
        .await
        .map_err(|_| anyhow!("Tor HTTP response body timed out"))?
    {
        let frame = frame?;
        if let Some(chunk) = frame.data_ref() {
            if response_bytes.len().saturating_add(chunk.len()) > max_response_bytes {
                return Err(anyhow!("Tor HTTP response exceeds the body limit"));
            }
            response_bytes.extend_from_slice(chunk);
        }
    }
    Ok(TorHttpResponse {
        status,
        body: response_bytes,
        content_type,
    })
}

/// Performs a regular HTTP(S) GET over an isolated Arti circuit.
pub async fn tor_get(url: &str, timeout: Duration) -> Result<TorHttpResponse> {
    tor_request(Method::GET, url, Vec::new(), &[], timeout, usize::MAX).await
}

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

/// How a GET leaves the wallet: the selected transport applied to HTTP.
///
/// The selectors are the wallet's: `0` direct, `1` Tor, `2` Nym, `3` the
/// external proxy. Nym has no HTTP transport yet and fails closed, so a
/// request never goes out in the clear under a transport the user chose
/// for privacy. One route serves every request of an operation.
pub enum Route {
    /// A reqwest client: direct, or through the external proxy.
    Direct(reqwest::Client),
    /// One isolated Arti circuit per request. Redirects are not followed.
    Tor {
        timeout: Duration,
        max_response_bytes: usize,
    },
    /// Every request fails before it is sent.
    Nym,
}

impl Route {
    /// The route for `transport`, with `proxy` for transport 3 and `timeout`
    /// applied to each request. Bodies are not capped; see [`Route::capped`].
    pub fn new(transport: u8, proxy: &str, timeout: Duration) -> Result<Self> {
        match transport {
            0 => Ok(Self::Direct(client("", timeout)?)),
            1 => Ok(Self::Tor {
                timeout,
                max_response_bytes: usize::MAX,
            }),
            2 => Ok(Self::Nym),
            3 => Ok(Self::Direct(client(proxy, timeout)?)),
            other => Err(anyhow!("unsupported transport {other}")),
        }
    }

    /// Caps the body of a Tor response, which is read whole into memory.
    /// The reqwest route reads whole bodies uncapped as it always has.
    pub fn capped(self, max_response_bytes: usize) -> Self {
        match self {
            Self::Tor { timeout, .. } => Self::Tor {
                timeout,
                max_response_bytes,
            },
            other => other,
        }
    }

    /// One GET; a response outside 2xx is a failure, as `error_for_status`
    /// makes it on the reqwest route.
    async fn get_once(&self, url: &str) -> Result<HttpResponse> {
        match self {
            Self::Direct(client) => {
                let response = client.get(url).send().await?.error_for_status()?;
                let status = response.status();
                let headers = response.headers().clone();
                let body = response.bytes().await?;
                Ok(HttpResponse {
                    status,
                    headers,
                    body,
                })
            }
            Self::Tor {
                timeout,
                max_response_bytes,
            } => {
                let response = tor_request(
                    Method::GET,
                    url,
                    Vec::new(),
                    &[],
                    *timeout,
                    *max_response_bytes,
                )
                .await?;
                let status = reqwest::StatusCode::from_u16(response.status)?;
                if !status.is_success() {
                    return Err(anyhow!("HTTP status {status}"));
                }
                let mut headers = reqwest::header::HeaderMap::new();
                if let Some(content_type) = response.content_type {
                    headers.insert(reqwest::header::CONTENT_TYPE, content_type.parse()?);
                }
                Ok(HttpResponse {
                    status,
                    headers,
                    body: Bytes::from(response.body),
                })
            }
            Self::Nym => Err(anyhow!("Nym transport is unavailable")),
        }
    }
}

/// Builds the wallet's reqwest client: user agent, per-request timeout, and
/// an optional proxy URL ("http://…", "socks5h://…"; empty = direct).
///
/// `ZKOOL_EXTRA_ROOT_CERT_PATH` may name one PEM CA certificate. It is useful
/// for an explicitly configured private service, such as the local HTTPS
/// fixture used by the voting E2E test; normal wallet operation leaves it unset.
pub fn client(proxy: &str, timeout: Duration) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .user_agent("zkool/1.0")
        .timeout(timeout);
    if let Ok(path) = std::env::var("ZKOOL_EXTRA_ROOT_CERT_PATH") {
        let pem = std::fs::read(&path)
            .map_err(|error| anyhow!("read extra root certificate {path}: {error}"))?;
        let certificate = reqwest::Certificate::from_pem(&pem)
            .map_err(|error| anyhow!("parse extra root certificate {path}: {error}"))?;
        builder = builder.add_root_certificate(certificate);
    }
    if !proxy.is_empty() {
        builder = builder.proxy(reqwest::Proxy::all(proxy)?);
    }
    Ok(builder.build()?)
}

/// A GET response with its body already read: the read happens inside
/// [`http_get`]'s loop, so a body cut short by a reset or a timeout is one
/// failed attempt like any other, and the next URL is tried (#1434).
#[derive(Debug)]
pub struct HttpResponse {
    pub status: reqwest::StatusCode,
    pub headers: reqwest::header::HeaderMap,
    pub body: Bytes,
}

impl HttpResponse {
    /// The body parsed as JSON.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_json::from_slice(&self.body)?)
    }

    /// The body as UTF-8 text.
    pub fn text(&self) -> Result<String> {
        Ok(String::from_utf8(self.body.to_vec())?)
    }

    /// The body.
    pub fn bytes(&self) -> Bytes {
        self.body.clone()
    }
}

/// GETs candidate URLs over `route` in round-robin order, advancing after
/// each transport failure, HTTP 4xx/5xx response or failed body read, and
/// wrapping after the last URL. [RetryPolicy::attempts] limits total
/// requests across all URLs. Each call starts at the first URL unless
/// `policy.randomize_first` selects a random starting index. The body is
/// read before a response is returned, so every caller's read is covered by
/// the retries. A route that fails closed fails here without a retry.
pub async fn http_get<S: AsRef<str>>(
    route: &Route,
    urls: &[S],
    policy: RetryPolicy,
) -> Result<HttpResponse> {
    if urls.is_empty() {
        return Err(anyhow!("GET requires at least one URL"));
    }
    if matches!(route, Route::Nym) {
        return route.get_once("").await;
    }
    let start = if policy.randomize_first {
        rand::rng().random_range(0..urls.len())
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
        match route.get_once(url).await {
            Ok(response) => return Ok(response),
            Err(e) => last_err = Some(anyhow!("GET {url}: {e}")),
        }
    }
    Err(last_err.expect("attempts is at least 1"))
}

#[cfg(test)]
mod tests {
    use super::{http_get, proxy_url, tor_get, RetryPolicy, Route};
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
        let route = Route::Direct(reqwest::Client::new());
        let error = http_get(&route, &[] as &[&str], immediate_policy(3))
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
        let route = Route::Direct(reqwest::Client::builder().no_proxy().build().unwrap());
        let response = tokio::time::timeout(
            Duration::from_secs(5),
            http_get(&route, &urls, immediate_policy(3)),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(response.status, reqwest::StatusCode::OK);
        server.await.unwrap();
    }

    /// V12 F-19 (#1434): a 200 whose body is cut short is one failed attempt,
    /// and the next URL is requested.
    #[tokio::test]
    async fn advances_after_a_body_cut_short() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let urls = [format!("{base}/short"), format!("{base}/whole")];
        let server = tokio::spawn(async move {
            let mut paths = Vec::new();
            for _ in 0..2 {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    request.push(socket.read_u8().await.unwrap());
                }
                let request = String::from_utf8(request).unwrap();
                let path = request.split_whitespace().nth(1).unwrap().to_string();
                let response: &[u8] = if path == "/short" {
                    // 128 bytes announced, a few sent, then the connection closes.
                    b"HTTP/1.1 200 OK\r\nContent-Length: 128\r\nConnection: close\r\n\r\nabc"
                } else {
                    b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nwhole"
                };
                socket.write_all(response).await.unwrap();
                socket.shutdown().await.unwrap();
                paths.push(path);
            }
            paths
        });
        let route = Route::Direct(reqwest::Client::builder().no_proxy().build().unwrap());
        let response = tokio::time::timeout(
            Duration::from_secs(5),
            http_get(&route, &urls, immediate_policy(2)),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(response.text().unwrap(), "whole");
        assert_eq!(server.await.unwrap(), ["/short", "/whole"]);
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
        let route = Route::Direct(reqwest::Client::builder().no_proxy().build().unwrap());
        tokio::time::timeout(
            Duration::from_secs(5),
            http_get(
                &route,
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
        let route = Route::Direct(reqwest::Client::builder().no_proxy().build().unwrap());
        let error = tokio::time::timeout(
            Duration::from_secs(5),
            http_get(&route, &["invalid-url", &url], immediate_policy(2)),
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

    #[tokio::test]
    async fn tor_get_rejects_an_invalid_url_before_opening_a_route() {
        assert!(tor_get("not a URL", Duration::from_secs(1)).await.is_err());
    }

    /// Nym fails closed before any request and without the retry delays:
    /// the listener never sees a connection.
    #[tokio::test]
    async fn nym_route_fails_closed_without_a_request() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let route = Route::new(2, "", Duration::from_secs(1)).unwrap();
        let error = tokio::time::timeout(
            Duration::from_millis(200),
            http_get(&route, &[&url], RetryPolicy::default()),
        )
        .await
        .unwrap()
        .unwrap_err();
        assert!(error.to_string().contains("Nym transport is unavailable"));
        assert!(tokio::time::timeout(Duration::from_millis(50), listener.accept())
            .await
            .is_err());
    }

    #[tokio::test]
    async fn tor_route_rejects_an_invalid_url_before_opening_a_circuit() {
        let route = Route::new(1, "", Duration::from_secs(1)).unwrap();
        assert!(http_get(&route, &["not a URL"], immediate_policy(1))
            .await
            .is_err());
    }

    #[test]
    fn routes_follow_the_transport_selector() {
        assert!(matches!(
            Route::new(0, "socks5://x", Duration::from_secs(1)).unwrap(),
            Route::Direct(_)
        ));
        assert!(matches!(
            Route::new(1, "", Duration::from_secs(1)).unwrap().capped(8),
            Route::Tor {
                max_response_bytes: 8,
                ..
            }
        ));
        assert!(matches!(
            Route::new(3, "socks5://127.0.0.1:1080", Duration::from_secs(1)).unwrap(),
            Route::Direct(_)
        ));
        assert!(Route::new(4, "", Duration::from_secs(1)).is_err());
    }
}
