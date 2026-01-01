use crate::graphql::Context;
use juniper::{graphql_object, FieldResult, GraphQLInputObject};

pub struct Mutation {}

#[derive(GraphQLInputObject)]
pub struct NewAccount {
    pub name: String,
    pub key: String,
    pub passphrase: Option<String>,
    pub aindex: i32,
    pub birth: Option<i32>,
    pub pools: Option<i32>,
    pub use_internal: bool,
}

#[graphql_object]
#[graphql(
    context = Context,
)]
impl Mutation {
    async fn create_account(new_account: NewAccount, context: &Context) -> FieldResult<i32> {
        let na = crate::api::account::NewAccount {
            name: new_account.name,
            restore: false,
            key: new_account.key,
            passphrase: new_account.passphrase,
            fingerprint: None,
            icon: None,
            aindex: new_account.aindex as u32,
            birth: new_account.birth.map(|v| v as u32),
            pools: new_account.pools.map(|v| v as u8),
            use_internal: new_account.use_internal,
            folder: String::new(),
            internal: false,
            ledger: false,
        };
        let id_account = crate::api::account::new_account(&na, &context.coin).await?;
        Ok(id_account as i32)
    }
}
