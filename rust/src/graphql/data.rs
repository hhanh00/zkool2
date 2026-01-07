use std::pin::Pin;

use futures::Stream;
use juniper::{FieldResult, GraphQLEnum, GraphQLObject};
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;

#[derive(Clone, Debug)]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub seed: Option<String>,
    pub passphrase: Option<String>,
    pub aindex: i32,
    pub dindex: i32,
    pub birth: i32,
}

#[derive(Clone, Debug)]
pub struct Transaction {
    pub id: i32,
    pub txid: String,
    pub account: i32,
    pub height: i32,
    pub time: NaiveDateTime,
    pub value: BigDecimal,
    pub fee: BigDecimal,
}

#[derive(GraphQLObject)]
pub struct Balance {
    pub height: Option<i32>,
    pub transparent: BigDecimal,
    pub sapling: BigDecimal,
    pub orchard: BigDecimal,
}

#[derive(GraphQLObject)]
pub struct Addresses {
    pub ua: Option<String>,
    pub transparent: Option<String>,
    pub sapling: Option<String>,
    pub orchard: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Note {
    pub height: i32,
    pub pool: i32,
    pub tx: i32,
    pub value: BigDecimal,
}

#[derive(GraphQLObject)]
pub struct UnconfirmedTx {
    pub txid: String,
    pub value: BigDecimal,
}

#[derive(Clone, GraphQLObject, Default)]
pub struct Event {
    pub r#type: EventType,
    pub height: i32,
    pub txid: String,
}

#[derive(Clone, GraphQLEnum, Default)]
pub enum EventType {
    #[default] Block,
    Tx,
}

pub type EventStream = Pin<Box<dyn Stream<Item = FieldResult<Event>> + Send>>;
