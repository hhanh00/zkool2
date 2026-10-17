use std::pin::Pin;

use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use futures::Stream;
use juniper::{FieldResult, GraphQLEnum, GraphQLObject};

#[derive(Clone, Debug)]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub seed: Option<String>,
    pub passphrase: Option<String>,
    pub aindex: i32,
    pub dindex: i32,
    pub birth: i32,
    pub height: i32,
    pub balance: BigDecimal,
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

impl From<crate::api::account::Tx> for Transaction {
    fn from(tx: crate::api::account::Tx) -> Self {
        let mut txid = tx.txid;
        txid.reverse();
        Self {
            id: tx.id as i32,
            txid: hex::encode(&txid),
            account: tx.account as i32,
            height: tx.height as i32,
            time: chrono::DateTime::from_timestamp(tx.time as i64, 0)
                .unwrap_or_default()
                .naive_local(),
            value: crate::graphql::query::zats_to_zec(tx.value),
            fee: crate::graphql::query::zats_to_zec(tx.fee),
        }
    }
}

#[derive(GraphQLObject)]
pub struct Balance {
    pub height: Option<i32>,
    pub transparent: BigDecimal,
    pub sapling: BigDecimal,
    pub orchard: BigDecimal,
    pub ironwood: BigDecimal,
    pub total: BigDecimal,
}

#[derive(GraphQLObject)]
pub struct Addresses {
    pub ua: Option<String>,
    pub transparent: Option<String>,
    pub sapling: Option<String>,
    pub orchard: Option<String>,
    pub ironwood: Option<String>,
    pub diversifier_index: BigDecimal,
}

impl From<crate::api::account::Addresses> for Addresses {
    fn from(a: crate::api::account::Addresses) -> Self {
        Self {
            ua: a.ua,
            transparent: a.taddr,
            sapling: a.saddr,
            // Ironwood shares Orchard's keys and therefore its address.
            ironwood: a.oaddr.clone(),
            orchard: a.oaddr,
            diversifier_index: BigDecimal::from(a.diversifier_index),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Note {
    pub id: i32,
    pub height: i32,
    pub pool: i32,
    pub tx: i32,
    pub value: BigDecimal,
    pub scope: i32,
    pub diversifier: String,
    pub diversifier_index: Option<BigDecimal>,
    pub address: String,
    pub memo: Option<String>,
    pub id_asset: Option<i32>,
    pub asset_base: Option<String>,
}

#[derive(GraphQLObject)]
pub struct UnconfirmedTx {
    pub txid: String,
    pub value: BigDecimal,
    pub notes: Vec<UnconfirmedNote>,
}

#[derive(Clone, Default, GraphQLObject)]
pub struct UnconfirmedNote {
    pub pool: i32,
    pub scope: i32,
    pub value: BigDecimal,
    pub diversifier: String,
    pub diversifier_index: Option<BigDecimal>,
    pub address: Option<String>,
    pub memo: Option<String>,
}

#[derive(Clone, GraphQLEnum)]
pub enum DKGStatus {
    Waiting,
    Round1,
    Round2,
    Completed,
}

#[derive(Clone, GraphQLObject, Default)]
pub struct Event {
    pub r#type: EventType,
    pub height: i32,
    pub txid: String,
    pub value: BigDecimal,
    pub dkg_account: i32,
    pub notes: Vec<UnconfirmedNote>,
}

#[derive(Clone, GraphQLEnum, Default)]
pub enum EventType {
    #[default]
    Block,
    Tx,
    DKG,
}

#[derive(GraphQLObject)]
pub struct AssetInfo {
    pub id_asset: i32,
    pub asset_desc_hash: String,
    pub asset_name: Option<String>,
    pub ik: String,
    pub asset_base: String,
    pub finalized: bool,
    pub first_seen_height: i32,
    pub balance: BigDecimal,
}

/// A frost account restored from a `zkfrost` backup string.
#[derive(GraphQLObject)]
pub struct FrostRestoreResult {
    pub id_account: i32,
    /// 0 for a legacy group, 2 for a ZIP 2005 one.
    pub dkg_version: i32,
}

#[derive(GraphQLObject)]
pub struct MigrationEvent {
    /// "SplitComplete" | "MigrateComplete" | "Complete" | "NothingToDo" | "Error"
    pub event: String,
    /// Fee in zats for SplitComplete / MigrateComplete
    pub fee: Option<i32>,
    /// Error message for Error variant
    pub message: Option<String>,
}

pub type EventStream = Pin<Box<dyn Stream<Item = FieldResult<Event>> + Send>>;
