use std::time::Duration;
use std::{collections::HashMap, sync::LazyLock};

use juniper::{graphql_subscription, FieldResult};
use tokio::sync::mpsc;
use tokio::sync::{mpsc::Sender, Mutex};
use tokio_stream::wrappers::ReceiverStream;

use crate::graphql::data::EventType;
use crate::graphql::{
    data::{Event, EventStream},
    Context,
};

pub struct Subscription {}

#[graphql_subscription(context = Context)]
impl Subscription {
    pub async fn events(id_account: i32) -> EventStream {
        let (tx, rx) = mpsc::channel::<FieldResult<Event>>(10);
        let mut subs = SUBS.lock().await;
        let e = subs.entry(id_account).or_insert_with(Vec::new);
        e.push(tx);

        Box::pin(ReceiverStream::new(rx))
    }
}

pub async fn test_event_pub(id_account: i32) {
    let mut height = 0;
    loop {
        let subs = SUBS.lock().await;
        if let Some(ss) = subs.get(&id_account) {
            for s in ss {
                let _ = s.send(Ok(Event {
                    r#type: EventType::Block,
                    height,
                    txid: "".to_string(),
                }))
                .await;
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        height += 1;
    }
}

#[allow(clippy::type_complexity)]
pub static SUBS: LazyLock<Mutex<HashMap<i32, Vec<Sender<FieldResult<Event>>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
