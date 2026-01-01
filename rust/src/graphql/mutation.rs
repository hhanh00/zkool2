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

#[derive(GraphQLInputObject)]
pub struct UpdateAccount {
    pub name: Option<String>,
    pub birth: Option<i32>,
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

    async fn edit_account(
        id_account: i32,
        update_account: UpdateAccount,
        context: &Context,
    ) -> FieldResult<bool> {
        let ua = crate::api::account::AccountUpdate {
            coin: 0,
            id: id_account as u32,
            name: update_account.name,
            icon: None,
            birth: update_account.birth.map(|v| v as u32),
            folder: 0,
            hidden: None,
            enabled: None,
        };
        crate::api::account::update_account(&ua, &context.coin).await?;
        Ok(true)
    }

    async fn delete_account(id_account: i32, context: &Context) -> FieldResult<bool> {
        crate::api::account::delete_account(id_account as u32, &context.coin).await?;
        Ok(true)
    }

    async fn reset_account(id_account: i32, context: &Context) -> FieldResult<bool> {
        crate::api::account::reset_sync(id_account as u32, &context.coin).await?;
        Ok(true)
    }

    async fn synchronize(id_accounts: Vec<i32>, context: &Context) -> FieldResult<bool> {
        let id_accounts = id_accounts.into_iter().map(|v| v as u32).collect();
        let current_height = crate::api::network::get_current_height(&context.coin).await?;
        crate::api::sync::synchronize_impl(
            (),
            id_accounts,
            current_height,
            100_000,
            40,
            10_000,
            &context.coin,
        )
        .await?;
        Ok(true)
    }
}
