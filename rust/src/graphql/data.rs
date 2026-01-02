use juniper::GraphQLObject;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;

#[derive(GraphQLObject)]
#[graphql(description = "A Zcash wallet account")]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub seed: Option<String>,
    pub passphrase: Option<String>,
    pub aindex: i32,
    pub dindex: i32,
    pub birth: i32,
}

#[derive(GraphQLObject)]
pub struct Transaction {
    pub id: i32,
    pub txid: String,
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

#[derive(GraphQLObject)]
pub struct Note {
    pub height: i32,
    pub pool: i32,
    pub tx: i32,
    pub value: BigDecimal,
}
