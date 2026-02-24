use crate::{api::coin::Coin, graphql::query::{TxLoader, new_tx_loader}};
use crate::graphql::jwt::Claims;
use juniper::{FieldResult, FieldError};

pub mod jwt;
pub mod data;
pub mod frost;
pub mod query;
pub mod mutation;
pub mod subs;

#[derive(Clone)]
pub struct Context {
    pub coin: Coin,
    pub auth: Option<Claims>,
    pub tx_loader: TxLoader,
}

impl Context {
    pub fn new(coin: Coin) -> Self {
        let pool = coin.get_pool().unwrap();
        Self {
            coin,
            tx_loader: new_tx_loader(pool),
            auth: None,
        }
    }
}

impl juniper::Context for Context {}

pub fn check_auth(context: &Context, id_account: i32) -> FieldResult<()> {
    if let Some(auth) = &context.auth {
        if auth.sub != 0 && auth.sub != id_account as u32 {
            return Err(FieldError::new("Unauthorized", juniper::Value::Null))
        }
    }
    Ok(())
}

pub fn check_admin_auth(context: &Context) -> FieldResult<()> {
    if let Some(auth) = &context.auth {
        if auth.sub != 0 {
            return Err(FieldError::new("Unauthorized", juniper::Value::Null))
        }
    }
    Ok(())
}
