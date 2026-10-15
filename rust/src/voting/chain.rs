//! Vote-chain, vote-tree, PIR, and helper HTTP transports routed through zkool.
//!
//! The voting crate asks the host for its own HTTP mechanism so privacy
//! routing can fail closed. This module supplies one route implementation the
//! four host transport traits share: direct connections and externally
//! configured proxies use the wallet's standard reqwest setup, Tor uses the
//! wallet's Arti HTTP adapter, and Nym deliberately fails closed until it has
//! a native HTTP adapter.
//!
//! Transport selectors: `0` direct, `1` Tor, `2` Nym (fails closed), `3`
//! external proxy.

use std::time::Duration;

use zcash_voting::{
    ChainHttpRequest, ChainHttpResponse, ChainPostDispatch, ChainTransport, ChainTransportError,
    ChainTransportFuture, HelperFuture, HelperResponse, HelperTransport, HelperTransportError,
    MAX_CHAIN_HTTP_RESPONSE_BYTES, MAX_HELPER_RESPONSE_BYTES,
};

/// PIR and vote-tree responses are multi-megabyte: the SDK's built-in
/// transport caps both at 8 MiB (its private `MAX_PIR_RESPONSE_BYTES` and
/// `MAX_TREE_RESPONSE_BYTES`) so a server cannot force unbounded allocation
/// before the client validates the negotiated geometry. Mirror those caps;
/// the 256 KiB chain limit must not leak into these routes.
const MAX_PIR_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
const MAX_TREE_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

/// The route failure of one request, classified by where it happened relative
/// to the network boundary. Each adapter maps this onto its own protocol's
/// error type: the chain client treats everything past `BeforeSend` as
/// possibly dispatched, while the helper protocol keeps a refused connection
/// and a failed body as definite, retryable failures.
enum RouteFailure {
    /// The request never left: unsupported route, route construction, or
    /// request building failed.
    BeforeSend(anyhow::Error),
    /// The route timed the request out.
    Timeout,
    /// The TCP connection could not be established (direct and proxy routes
    /// only; Tor reports every post-construction failure as ambiguous).
    ConnectFailed(anyhow::Error),
    /// Response headers arrived but the body could not be read, or exceeded
    /// the route's body cap.
    AfterHeaders(anyhow::Error),
    /// Anything else: the request may have crossed the network boundary.
    Ambiguous(anyhow::Error),
}

impl RouteFailure {
    fn chain_error(&self) -> ChainTransportError {
        match self {
            RouteFailure::BeforeSend(error) => {
                ChainTransportError::definitely_unsent(error.to_string())
            }
            RouteFailure::Timeout => {
                ChainTransportError::possibly_dispatched("request timed out")
            }
            RouteFailure::ConnectFailed(error) => {
                ChainTransportError::possibly_dispatched(format!("connection failed: {error}"))
            }
            RouteFailure::AfterHeaders(error) | RouteFailure::Ambiguous(error) => {
                ChainTransportError::possibly_dispatched(format!(
                    "request ended before response: {error}"
                ))
            }
        }
    }

    fn helper_error(&self) -> HelperTransportError {
        match self {
            RouteFailure::BeforeSend(error) => HelperTransportError::Transport(error.to_string()),
            RouteFailure::Timeout => HelperTransportError::Timeout,
            RouteFailure::ConnectFailed(error) => {
                HelperTransportError::Transport(format!("helper connection failed: {error}"))
            }
            RouteFailure::AfterHeaders(error) => HelperTransportError::Response(error.to_string()),
            RouteFailure::Ambiguous(error) => HelperTransportError::Ambiguous(error.to_string()),
        }
    }
}

fn route_kind(transport: u8) -> Result<(), RouteFailure> {
    match transport {
        0 | 1 | 3 => Ok(()),
        2 => Err(RouteFailure::BeforeSend(anyhow::anyhow!(
            "Nym transport is unavailable"
        ))),
        _ => Err(RouteFailure::BeforeSend(anyhow::anyhow!(
            "unsupported transport"
        ))),
    }
}

fn content_type(headers: &[(String, String)]) -> Option<String> {
    headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.clone())
}

/// Sends one request over the route, capping the response body.
///
/// `dispatch` is marked immediately before the request is released to the
/// network stack: after the client and request objects are fully built, so a
/// failure before the marker is definitely unsent, and everything after it is
/// conservatively treated as possibly dispatched.
#[allow(clippy::too_many_arguments)]
async fn route_request(
    transport: u8,
    proxy: &str,
    method: hyper::Method,
    url: &str,
    body: Vec<u8>,
    headers: &[(String, String)],
    timeout: Duration,
    max_response_bytes: usize,
    dispatch: Option<&ChainPostDispatch>,
) -> Result<zcash_voting::TransportResponse, RouteFailure> {
    route_kind(transport)?;
    if transport == 1 {
        if let Some(dispatch) = dispatch {
            dispatch.mark_possible();
        }
        return crate::net::http::tor_request(
            method, url, body, headers, timeout, max_response_bytes,
        )
        .await
        .map(|response| zcash_voting::TransportResponse {
            status: response.status,
            headers: response
                .content_type
                .map(|value| vec![("content-type".to_owned(), value)])
                .unwrap_or_default(),
            body: response.body,
        })
        .map_err(|error| {
            if error.to_string().contains("timed out") {
                RouteFailure::Timeout
            } else {
                // A failure reported by Hyper after route construction may
                // have reached the server. Conservatively ambiguous.
                RouteFailure::Ambiguous(error)
            }
        });
    }

    let proxy = if transport == 3 { proxy } else { "" };
    let client = crate::net::http::client(proxy, timeout).map_err(RouteFailure::BeforeSend)?;
    let mut request = client.request(method, url).body(body);
    for (name, value) in headers {
        request = request.header(name, value);
    }
    let request = request
        .build()
        .map_err(|error| RouteFailure::BeforeSend(anyhow::anyhow!("request build failed: {error}")))?;
    if let Some(dispatch) = dispatch {
        dispatch.mark_possible();
    }
    let response = match tokio::time::timeout(timeout, client.execute(request)).await {
        Err(_) => return Err(RouteFailure::Timeout),
        Ok(Err(error)) if error.is_timeout() => return Err(RouteFailure::Timeout),
        Ok(Err(error)) if error.is_connect() => {
            return Err(RouteFailure::ConnectFailed(anyhow::anyhow!(error)));
        }
        Ok(Err(error)) => return Err(RouteFailure::Ambiguous(anyhow::anyhow!(error))),
        Ok(Ok(response)) => response,
    };
    let status = response.status().as_u16();
    let response_headers = response
        .headers()
        .iter()
        .filter_map(|(name, value)| value.to_str().ok().map(|value| (name.as_str().to_owned(), value.to_owned())))
        .collect::<Vec<_>>();
    if response
        .content_length()
        .is_some_and(|length| length as usize > max_response_bytes)
    {
        return Err(RouteFailure::AfterHeaders(anyhow::anyhow!(
            "response exceeds the body limit"
        )));
    }
    let mut body = Vec::new();
    let mut stream = response.bytes_stream();
    use futures::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| {
            RouteFailure::AfterHeaders(anyhow::anyhow!("response body read failed: {error}"))
        })?;
        if body.len().saturating_add(chunk.len()) > max_response_bytes {
            return Err(RouteFailure::AfterHeaders(anyhow::anyhow!(
                "response exceeds the body limit"
            )));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(zcash_voting::TransportResponse {
        status,
        headers: response_headers,
        body,
    })
}

/// Routes vote-chain HTTP requests through zkool's selected transport.
///
/// The chain client's POSTs broadcast transactions, so the transport marks
/// [`ChainPostDispatch`] at the network boundary: a failure before the marker
/// is definitely unsent and safe to redispatch, while anything after it must
/// go through the SDK's recovery instead of a blind resend.
#[derive(Clone, Debug)]
pub struct ZkoolChainTransport {
    transport: u8,
    proxy: String,
}

impl ZkoolChainTransport {
    pub fn new(transport: u8, proxy: impl Into<String>) -> Self {
        Self {
            transport,
            proxy: proxy.into(),
        }
    }

    async fn route(
        &self,
        method: hyper::Method,
        request: ChainHttpRequest,
        body: Vec<u8>,
        dispatch: Option<&ChainPostDispatch>,
    ) -> Result<ChainHttpResponse, ChainTransportError> {
        let result = route_request(
            self.transport,
            &self.proxy,
            method,
            request.url(),
            body,
            request.headers(),
            request.timeout(),
            request.max_response_bytes().min(MAX_CHAIN_HTTP_RESPONSE_BYTES),
            dispatch,
        )
        .await;
        match result {
            Ok(response) => Ok(ChainHttpResponse::new(
                response.status,
                response.body,
                content_type(&response.headers),
                response.headers,
            )),
            Err(failure) => Err(failure.chain_error()),
        }
    }
}

impl ChainTransport for ZkoolChainTransport {
    fn chain_get<'a>(&'a self, request: ChainHttpRequest) -> ChainTransportFuture<'a> {
        Box::pin(async move {
            self.route(hyper::Method::GET, request, Vec::new(), None)
                .await
        })
    }

    fn chain_post_json<'a>(
        &'a self,
        request: ChainHttpRequest,
        json: Vec<u8>,
    ) -> ChainTransportFuture<'a> {
        Box::pin(async move {
            self.route(hyper::Method::POST, request, json, None).await
        })
    }

    fn chain_post_json_with_dispatch<'a>(
        &'a self,
        request: ChainHttpRequest,
        json: Vec<u8>,
        dispatch: ChainPostDispatch,
    ) -> ChainTransportFuture<'a> {
        Box::pin(async move {
            self.route(hyper::Method::POST, request, json, Some(&dispatch))
                .await
        })
    }
}

/// Routes the PIR client's HTTP requests through zkool's selected transport.
#[derive(Clone, Debug)]
pub struct ZkoolPirTransport {
    transport: u8,
    proxy: String,
}

impl ZkoolPirTransport {
    pub fn new(transport: u8, proxy: impl Into<String>) -> Self {
        Self {
            transport,
            proxy: proxy.into(),
        }
    }

    async fn route(
        &self,
        method: hyper::Method,
        url: &str,
        body: Vec<u8>,
    ) -> anyhow::Result<zcash_voting::TransportResponse> {
        route_request(
            self.transport,
            &self.proxy,
            method,
            url,
            body,
            &[],
            Duration::from_secs(30),
            MAX_PIR_RESPONSE_BYTES,
            None,
        )
        .await
        .map_err(|failure| anyhow::anyhow!(failure.chain_error().to_string()))
    }
}

impl zcash_voting::Transport for ZkoolPirTransport {
    fn get<'a>(&'a self, url: &'a str) -> zcash_voting::TransportFuture<'a> {
        Box::pin(self.route(hyper::Method::GET, url, Vec::new()))
    }

    fn post<'a>(&'a self, url: &'a str, body: Vec<u8>) -> zcash_voting::TransportFuture<'a> {
        Box::pin(self.route(hyper::Method::POST, url, body))
    }
}

/// Blocking GET-only transport for the vote commitment tree client.
///
/// The tree client's transport trait is synchronous while zkool's HTTP stack
/// is async, so GETs run on a dedicated current-thread runtime that any
/// caller context (async worker or blocking thread) can park on.
#[derive(Clone, Debug)]
pub struct ZkoolTreeTransport {
    transport: u8,
    proxy: String,
}

impl ZkoolTreeTransport {
    pub fn new(transport: u8, proxy: impl Into<String>) -> Self {
        Self {
            transport,
            proxy: proxy.into(),
        }
    }
}

static TREE_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
    std::sync::LazyLock::new(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tree sync runtime")
    });

impl vote_commitment_tree_client::transport::Transport for ZkoolTreeTransport {
    fn get(
        &self,
        url: &str,
    ) -> Result<
        vote_commitment_tree_client::transport::TransportResponse,
        vote_commitment_tree_client::transport::TransportError,
    > {
        let proxy = self.proxy.clone();
        let url = url.to_owned();
        TREE_RUNTIME
            .block_on(route_request(
                self.transport,
                &proxy,
                hyper::Method::GET,
                &url,
                Vec::new(),
                &[],
                Duration::from_secs(30),
                MAX_TREE_RESPONSE_BYTES,
                None,
            ))
            .map(|response| vote_commitment_tree_client::transport::TransportResponse {
                status: response.status,
                body: response.body,
            })
            .map_err(|failure| {
                vote_commitment_tree_client::transport::TransportError::Request(
                    failure.chain_error().to_string(),
                )
            })
    }
}

/// Routes voting-helper HTTP requests through zkool's selected transport.
///
/// The helper protocol retries a share submission only after a *definite*
/// failure, so this adapter keeps the finer classification: a refused
/// connection is `Transport` (never sent) and a body that fails after the
/// headers arrived is `Response`, where the chain adapter above would report
/// both as possibly dispatched.
#[derive(Clone, Debug)]
pub struct ZkoolHelperTransport {
    transport: u8,
    proxy: String,
}

impl ZkoolHelperTransport {
    pub fn new(transport: u8, proxy: impl Into<String>) -> Self {
        Self {
            transport,
            proxy: proxy.into(),
        }
    }

    async fn route(
        &self,
        method: hyper::Method,
        url: &str,
        body: Vec<u8>,
        headers: &[(String, String)],
        timeout: Duration,
    ) -> Result<HelperResponse, HelperTransportError> {
        let response = route_request(
            self.transport,
            &self.proxy,
            method,
            url,
            body,
            headers,
            timeout,
            MAX_HELPER_RESPONSE_BYTES,
            None,
        )
        .await
        .map_err(|failure| failure.helper_error())?;
        Ok(HelperResponse::new(
            response.status,
            response.body,
            content_type(&response.headers),
        ))
    }
}

impl HelperTransport for ZkoolHelperTransport {
    fn get<'a>(&'a self, url: &'a str, timeout: Duration) -> HelperFuture<'a> {
        Box::pin(self.route(hyper::Method::GET, url, Vec::new(), &[], timeout))
    }

    fn post_json<'a>(&'a self, url: &'a str, body: Vec<u8>, timeout: Duration) -> HelperFuture<'a> {
        self.post_json_with_headers(url, body, timeout, &[])
    }

    fn post_json_with_headers<'a>(
        &'a self,
        url: &'a str,
        body: Vec<u8>,
        timeout: Duration,
        headers: &'a [(String, String)],
    ) -> HelperFuture<'a> {
        Box::pin(async move {
            let mut request_headers =
                vec![("content-type".to_string(), "application/json".to_string())];
            request_headers.extend(headers.iter().cloned());
            self.route(hyper::Method::POST, url, body, &request_headers, timeout)
                .await
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use zcash_voting::{HelperTransport, HelperTransportError};

    use super::{route_kind, route_request, RouteFailure, ZkoolHelperTransport};

    #[test]
    fn nym_route_fails_closed() {
        let error = route_kind(2).unwrap_err().chain_error();
        assert!(error.is_definitely_unsent(), "{error}");
    }

    #[test]
    fn unsupported_route_fails_closed() {
        assert!(route_kind(7).is_err());
        assert!(route_kind(0).is_ok());
        assert!(route_kind(1).is_ok());
        assert!(route_kind(3).is_ok());
    }

    #[tokio::test]
    async fn nym_helper_route_fails_closed_as_a_definite_failure() {
        let error = ZkoolHelperTransport::new(2, "")
            .get("http://127.0.0.1:1/share", Duration::from_secs(1))
            .await
            .unwrap_err();
        assert!(matches!(error, HelperTransportError::Transport(_)), "{error}");
    }

    /// A URL whose port was just released, so a connection to it is refused.
    async fn refused_url() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        drop(listener);
        url
    }

    /// A refused connection never reached the server. The chain adapter still
    /// reports it as possibly dispatched, while the helper adapter reports a
    /// definite `Transport` failure so the share can be retried.
    #[tokio::test]
    async fn refused_connection_is_definite_for_helper_and_ambiguous_for_chain() {
        let url = refused_url().await;
        let result = route_request(
            0,
            "",
            hyper::Method::GET,
            &url,
            Vec::new(),
            &[],
            Duration::from_secs(5),
            1024,
            None,
        )
        .await;
        let Err(failure) = result else {
            panic!("a refused connection must fail");
        };
        assert!(matches!(failure, RouteFailure::ConnectFailed(_)));
        assert!(!failure.chain_error().is_definitely_unsent());
        assert!(matches!(
            failure.helper_error(),
            HelperTransportError::Transport(_)
        ));
    }

    /// Serves one response with the given content length and body.
    async fn serve_once(content_length: usize, body: Vec<u8>) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                request.push(socket.read_u8().await.unwrap());
            }
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
                 Content-Length: {content_length}\r\nConnection: close\r\n\r\n"
            );
            // The client may hang up once it sees the headers; that is fine.
            let _ = socket.write_all(head.as_bytes()).await;
            let _ = socket.write_all(&body).await;
        });
        url
    }

    /// Headers arrived, so the request was received: the helper reports a
    /// `Response` failure, not an ambiguous or retryable one.
    #[tokio::test]
    async fn oversized_helper_body_is_a_response_failure() {
        let url = serve_once(
            zcash_voting::MAX_HELPER_RESPONSE_BYTES + 1,
            vec![b'x'; zcash_voting::MAX_HELPER_RESPONSE_BYTES + 1],
        )
        .await;
        let error = ZkoolHelperTransport::new(0, "")
            .get(&url, Duration::from_secs(5))
            .await
            .unwrap_err();
        assert!(matches!(error, HelperTransportError::Response(_)), "{error}");
    }

    #[tokio::test]
    async fn helper_transport_returns_the_content_type() {
        let url = serve_once(2, b"{}".to_vec()).await;
        let response = ZkoolHelperTransport::new(0, "")
            .get(&url, Duration::from_secs(5))
            .await
            .expect("small helper response");
        assert_eq!(response.status(), 200);
        assert_eq!(response.body(), b"{}");
        assert_eq!(response.content_type(), Some("application/json"));
    }

    /// A real PIR response is multi-megabyte; the 256 KiB chain cap must not
    /// leak into the PIR route (regression: "response exceeds the body
    /// limit" against live PIR servers).
    #[tokio::test]
    async fn pir_transport_accepts_multi_megabyte_responses() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/pir", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                request.push(socket.read_u8().await.unwrap());
            }
            let body = vec![b'x'; 512 * 1024];
            socket
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
            socket.write_all(&body).await.unwrap();
        });
        let transport = super::ZkoolPirTransport::new(0, "");
        let response = zcash_voting::Transport::get(&transport, &url)
            .await
            .expect("large PIR response is accepted");
        assert_eq!(response.body.len(), 512 * 1024);
        server.await.unwrap();
    }
}
