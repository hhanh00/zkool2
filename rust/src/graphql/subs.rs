use std::{collections::HashMap, sync::LazyLock, time::Duration};

use futures::{future::Either, stream, Stream, StreamExt};
use juniper::{graphql_subscription, FieldResult};
use tokio::sync::mpsc;
use tokio::sync::{mpsc::Sender, Mutex};
use tokio_stream::wrappers::ReceiverStream;

use crate::graphql::{
    data::{Event, EventStream},
    {check_auth, now, Context},
};

pub struct Subscription {}

#[graphql_subscription(context = Context)]
impl Subscription {
    pub async fn events(id_account: i32, context: &Context) -> FieldResult<EventStream> {
        check_auth(context, id_account, false)?;
        let (tx, rx) = mpsc::channel::<FieldResult<Event>>(10);
        let mut subs = SUBS.lock().await;
        let e = subs.entry(id_account).or_insert_with(Vec::new);
        e.push(tx);

        let events = ReceiverStream::new(rx);
        Ok(match &context.auth {
            // The token was checked at the handshake; the subscription
            // outlives it, so it ends when the token does.
            Some(auth) => Box::pin(until_exp(events, auth.exp)),
            None => Box::pin(events),
        })
    }
}

/// `stream`, ended once the clock reaches `exp` (seconds since the epoch);
/// empty from the start when it already has.
pub fn until_exp<S: Stream>(stream: S, exp: usize) -> impl Stream<Item = S::Item> {
    let now = now();
    if exp <= now {
        // `take_until` would still yield what the source has ready.
        return Either::Left(stream::empty());
    }
    Either::Right(stream.take_until(tokio::time::sleep(Duration::from_secs((exp - now) as u64))))
}

#[cfg(test)]
mod expiry_tests {
    use super::*;

    #[tokio::test]
    async fn the_stream_ends_at_the_token_expiry() {
        // Items before the deadline go through; then nothing more, even
        // though the source never ends.
        let source = stream::iter([1, 2]).chain(stream::pending());
        let mut bounded = Box::pin(until_exp(source, now() + 1));
        assert_eq!(bounded.next().await, Some(1));
        assert_eq!(bounded.next().await, Some(2));
        let end = tokio::time::timeout(Duration::from_secs(3), bounded.next()).await;
        assert_eq!(end, Ok(None), "ended by the deadline, not the source");
    }

    #[tokio::test]
    async fn an_already_expired_token_gets_an_empty_stream() {
        let mut bounded = Box::pin(until_exp(stream::iter([1]), now().saturating_sub(1)));
        assert_eq!(bounded.next().await, None);
    }
}

#[allow(clippy::type_complexity)]
pub static SUBS: LazyLock<Mutex<HashMap<i32, Vec<Sender<FieldResult<Event>>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
