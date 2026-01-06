use std::{collections::HashMap, sync::LazyLock};

use juniper::{graphql_subscription, FieldResult};
use tokio::sync::mpsc;
use tokio::sync::{mpsc::Sender, Mutex};
use tokio_stream::wrappers::ReceiverStream;

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

#[allow(clippy::type_complexity)]
pub static SUBS: LazyLock<Mutex<HashMap<i32, Vec<Sender<FieldResult<Event>>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
