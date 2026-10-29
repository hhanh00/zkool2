use std::{collections::HashMap, sync::LazyLock, time::Duration};

use futures::{future::Either, stream, Stream, StreamExt};
use juniper::{graphql_subscription, FieldError, FieldResult};
use tokio::sync::mpsc::{self, error::TrySendError};
use tokio::sync::{mpsc::Sender, Mutex};
use tokio_stream::wrappers::ReceiverStream;

use crate::graphql::{
    data::{Event, EventStream},
    {check_auth, now, Context},
};

pub struct Subscription {}

/// Live subscriptions one account may hold at once: a client cannot grow
/// the index between the events that would find its dead ones.
pub const MAX_SUBSCRIPTIONS_PER_ACCOUNT: usize = 32;

#[graphql_subscription(context = Context)]
impl Subscription {
    pub async fn events(id_account: i32, context: &Context) -> FieldResult<EventStream> {
        check_auth(context, id_account, false)?;
        let events = subscribe(id_account).await?;
        Ok(match &context.auth {
            // The token was checked at the handshake; the subscription
            // outlives it, so it ends when the token does.
            Some(auth) => Box::pin(until_exp(events, auth.exp)),
            None => Box::pin(events),
        })
    }
}

/// Register a subscription to the account's events: its sender joins
/// `SUBS` and leaves it on its own when the stream is dropped.
async fn subscribe(id_account: i32) -> FieldResult<ReceiverStream<FieldResult<Event>>> {
    let (tx, rx) = mpsc::channel::<FieldResult<Event>>(10);
    {
        let mut subs = SUBS.lock().await;
        let e = subs.entry(id_account).or_default();
        e.retain(|s| !s.is_closed());
        if e.len() >= MAX_SUBSCRIPTIONS_PER_ACCOUNT {
            return Err(FieldError::new(
                "too many subscriptions for this account",
                juniper::Value::Null,
            ));
        }
        e.push(tx.clone());
    }
    // The stream is the only receiver: `closed` fires when juniper drops
    // it, when the client unsubscribes, the socket goes or the token
    // expires.
    tokio::spawn(async move {
        tx.closed().await;
        let mut subs = SUBS.lock().await;
        if let Some(e) = subs.get_mut(&id_account) {
            e.retain(|s| !s.same_channel(&tx));
            if e.is_empty() {
                subs.remove(&id_account);
            }
        }
    });
    Ok(ReceiverStream::new(rx))
}

/// Deliver `event` to every subscriber in `subs`, without waiting: one
/// that stopped reading (its channel is full) misses this event rather
/// than holding up the others, and one that is gone is dropped here, in
/// case its watcher has not run yet.
pub fn fan_out(subs: &mut Vec<Sender<FieldResult<Event>>>, event: &Event) {
    subs.retain(|s| match s.try_send(Ok(event.clone())) {
        Ok(()) => true,
        Err(TrySendError::Full(_)) => {
            tracing::debug!("subscriber is not reading; event skipped");
            true
        }
        Err(TrySendError::Closed(_)) => false,
    });
}

#[cfg(test)]
mod fan_out_tests {
    use super::*;

    fn event(height: i32) -> Event {
        Event {
            height,
            ..Event::default()
        }
    }

    #[tokio::test]
    async fn a_gone_subscriber_is_dropped_a_slow_one_skips_and_a_live_one_receives() {
        let (gone, rx_gone) = mpsc::channel(1);
        drop(rx_gone);
        let (slow, mut rx_slow) = mpsc::channel(1);
        slow.try_send(Ok(event(1))).unwrap(); // full
        let (live, mut rx_live) = mpsc::channel(1);
        let mut subs = vec![gone, slow.clone(), live.clone()];

        fan_out(&mut subs, &event(2));

        assert_eq!(subs.len(), 2, "the gone subscriber is dropped");
        assert!(subs.iter().any(|s| s.same_channel(&slow)));
        assert!(subs.iter().any(|s| s.same_channel(&live)));
        assert_eq!(rx_live.try_recv().unwrap().unwrap().height, 2);
        assert_eq!(rx_slow.try_recv().unwrap().unwrap().height, 1, "only the old item");
        assert!(rx_slow.try_recv().is_err(), "the event was skipped, not queued");
    }

    #[tokio::test]
    async fn dropping_the_stream_removes_its_sender_without_an_event() {
        let account = 1_001;
        let first = subscribe(account).await.unwrap();
        let second = subscribe(account).await.unwrap();
        assert_eq!(SUBS.lock().await[&account].len(), 2);

        drop(first);
        tokio::task::yield_now().await;
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(SUBS.lock().await[&account].len(), 1);

        drop(second);
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(!SUBS.lock().await.contains_key(&account), "an empty list goes too");
    }

    #[tokio::test]
    async fn an_account_cannot_hold_more_than_the_cap() {
        let account = 1_002;
        let mut streams = Vec::new();
        for _ in 0..MAX_SUBSCRIPTIONS_PER_ACCOUNT {
            streams.push(subscribe(account).await.unwrap());
        }
        let err = subscribe(account).await.err().expect("refused");
        assert_eq!(err.message(), "too many subscriptions for this account");
        // Dropped ones free their slots even before an event prunes them.
        streams.clear();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(subscribe(account).await.is_ok());
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
