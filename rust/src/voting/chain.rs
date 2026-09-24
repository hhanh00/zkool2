//! Vote-chain, vote-tree, and PIR HTTP transports routed through zkool.
//!
//! The voting crate asks the host for its own HTTP mechanism so privacy
//! routing can fail closed. This module supplies one route implementation the
//! three host transport traits share: direct connections and externally
//! configured proxies use the wallet's standard reqwest setup, Tor uses the
//! wallet's Arti HTTP adapter, and Nym deliberately fails closed until it has
//! a native HTTP adapter.
//!
//! Transport selectors mirror [`crate::voting::sidecar::ZkoolHelperTransport`]:
//! `0` direct, `1` Tor, `2` Nym (fails closed), `3` external proxy.

use std::time::Duration;

use zcash_voting::{
    ChainHttpRequest, ChainHttpResponse, ChainPostDispatch, ChainTransport, ChainTransportError,
    ChainTransportFuture, MAX_CHAIN_HTTP_RESPONSE_BYTES,
};

/// The route failure of one request, before or after the network boundary.
enum RouteFailure {
    /// The request never left: route construction or request building failed.
    BeforeSend(anyhow::Error),
    /// The route timed the request out.
    Timeout,
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
            RouteFailure::Ambiguous(error) => ChainTransportError::possibly_dispatched(
                format!("request ended before response: {error}"),
            ),
        }
    }
}

fn route_kind(transport: u8) -> Result<(), ChainTransportError> {
    match transport {
        0 | 1 | 3 => Ok(()),
        2 => Err(ChainTransportError::definitely_unsent(
            "Nym chain transport is unavailable",
        )),
        _ => Err(ChainTransportError::definitely_unsent(
            "unsupported chain transport",
        )),
    }
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
            headers: Vec::new(),
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
            return Err(RouteFailure::Ambiguous(anyhow::anyhow!(
                "connection failed: {error}"
            )));
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
        return Err(RouteFailure::Ambiguous(anyhow::anyhow!(
            "response exceeds the body limit"
        )));
    }
    let mut body = Vec::new();
    let mut stream = response.bytes_stream();
    use futures::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| {
            RouteFailure::Ambiguous(anyhow::anyhow!("response body read failed: {error}"))
        })?;
        if body.len().saturating_add(chunk.len()) > max_response_bytes {
            return Err(RouteFailure::Ambiguous(anyhow::anyhow!(
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
        route_kind(self.transport)?;
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
                response
                    .headers
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
                    .map(|(_, value)| value.clone()),
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
        route_kind(self.transport).map_err(|error| anyhow::anyhow!(error.to_string()))?;
        route_request(
            self.transport,
            &self.proxy,
            method,
            url,
            body,
            &[],
            Duration::from_secs(30),
            MAX_CHAIN_HTTP_RESPONSE_BYTES,
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
        let result = route_kind(self.transport).map_err(|error| {
            vote_commitment_tree_client::transport::TransportError::Request(error.to_string())
        });
        result?;
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
                MAX_CHAIN_HTTP_RESPONSE_BYTES,
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

#[cfg(test)]
mod tests {
    use super::route_kind;

    #[test]
    fn nym_route_fails_closed() {
        let error = route_kind(2).unwrap_err();
        assert!(error.is_definitely_unsent(), "{error}");
    }

    #[test]
    fn unsupported_route_fails_closed() {
        assert!(route_kind(7).is_err());
        assert!(route_kind(0).is_ok());
        assert!(route_kind(1).is_ok());
        assert!(route_kind(3).is_ok());
    }
}
