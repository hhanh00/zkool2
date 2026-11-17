//! An idle timeout for gRPC calls (#1654).
//!
//! A lightwalletd that accepts a request and then never answers, or a
//! connection that dies without the kernel noticing, used to leave the
//! caller waiting forever. [`IdleTimeout`] wraps a channel and fails a call
//! with `DeadlineExceeded` once the server has been silent for longer than
//! the window while the caller waits on it: before the response headers,
//! or between two frames of the response body. A stream that keeps sending
//! never times out, however long it runs, and time the caller spends not
//! reading (a busy scan holding back a block stream) does not count.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use http_body::{Body, Frame, SizeHint};
use tokio::time::{Instant, Sleep};
use tonic::codegen::{http, Bytes, Service, StdError};

/// The window for the sync's calls over a direct connection.
pub const SYNC_IDLE: Duration = Duration::from_secs(60);
/// The window for the sync's calls over Tor, Nym or a proxy, where each
/// round trip takes longer.
pub const SYNC_IDLE_SLOW: Duration = Duration::from_secs(120);
/// The window for every other call. It is generous, so a slow but healthy
/// call (a broadcast, a long transparent history) is not cut off, and it
/// still turns a call that would hang forever into an error.
pub const DEFAULT_IDLE: Duration = Duration::from_secs(5 * 60);

/// Wraps a gRPC channel so that each call fails after `window` without
/// data from the server.
#[derive(Clone, Debug)]
pub struct IdleTimeout<S> {
    inner: S,
    window: Duration,
}

impl<S> IdleTimeout<S> {
    pub fn new(inner: S, window: Duration) -> Self {
        Self { inner, window }
    }
}

/// The error an idle call ends with. It is a `tonic::Status`, which tonic
/// finds in the error chain, so the caller sees `DeadlineExceeded` and a
/// message that names the method.
fn idle_error(method: &str, window: Duration) -> StdError {
    Box::new(tonic::Status::deadline_exceeded(format!(
        "no data for {window:?} on {method}"
    )))
}

/// True if `error` comes from an idle timeout, at any depth of its chain.
pub fn is_idle_timeout(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<tonic::Status>()
            .is_some_and(|status| status.code() == tonic::Code::DeadlineExceeded)
    })
}

impl<S, ReqBody, ResBody> Service<http::Request<ReqBody>> for IdleTimeout<S>
where
    S: Service<http::Request<ReqBody>, Response = http::Response<ResBody>>,
    S::Error: Into<StdError>,
    S::Future: Send + 'static,
    ResBody: Body<Data = Bytes> + Unpin + Send + 'static,
    ResBody::Error: Into<StdError>,
{
    type Response = http::Response<IdleBody<ResBody>>;
    type Error = StdError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, StdError>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: http::Request<ReqBody>) -> Self::Future {
        let window = self.window;
        let method = request.uri().path().to_string();
        let response = self.inner.call(request);
        Box::pin(async move {
            match tokio::time::timeout(window, response).await {
                Ok(Ok(response)) => Ok(response.map(|body| IdleBody::new(body, window, method))),
                Ok(Err(error)) => Err(error.into()),
                Err(_) => Err(idle_error(&method, window)),
            }
        })
    }
}

/// A response body that fails once the server has sent nothing for the
/// window while it is being read.
pub struct IdleBody<B> {
    inner: B,
    window: Duration,
    method: String,
    deadline: Pin<Box<Sleep>>,
    /// Set while the reader waits on the server. The deadline is armed when
    /// a wait starts, so the time between two reads is not counted.
    waiting: bool,
}

impl<B> IdleBody<B> {
    fn new(inner: B, window: Duration, method: String) -> Self {
        Self {
            inner,
            window,
            method,
            deadline: Box::pin(tokio::time::sleep(window)),
            waiting: false,
        }
    }
}

impl<B> Body for IdleBody<B>
where
    B: Body<Data = Bytes> + Unpin,
    B::Error: Into<StdError>,
{
    type Data = Bytes;
    type Error = StdError;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, StdError>>> {
        let this = &mut *self;
        match Pin::new(&mut this.inner).poll_frame(cx) {
            Poll::Ready(frame) => {
                this.waiting = false;
                Poll::Ready(frame.map(|frame| frame.map_err(Into::into)))
            }
            Poll::Pending => {
                if !this.waiting {
                    this.waiting = true;
                    let deadline = Instant::now() + this.window;
                    this.deadline.as_mut().reset(deadline);
                }
                match this.deadline.as_mut().poll(cx) {
                    Poll::Ready(()) => {
                        Poll::Ready(Some(Err(idle_error(&this.method, this.window))))
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}
