use std::time::Duration;

use crate::frost::driver::Driver;
use crate::graphql::jwt::Claims;
use crate::{
    api::coin::Coin,
    graphql::query::{new_tx_loader, TxLoader},
};
use juniper::{FieldError, FieldResult};

pub mod data;
pub mod frost;
pub mod jwt;
pub mod mutation;
pub mod query;
pub mod swaps;
pub mod subs;
pub mod voting;

#[derive(Clone)]
pub struct Context {
    pub coin: Coin,
    pub auth: Option<Claims>,
    pub tx_loader: TxLoader,
    /// The one task that advances DKG and signing; see `frost::driver`.
    pub driver: Driver,
}

impl Context {
    /// Build the server's context and start its protocol driver, which
    /// polls the chain tip at the same cadence as the app's block watcher.
    /// Needs a tokio runtime.
    pub fn new(coin: Coin) -> Self {
        let pool = coin.get_pool().unwrap();
        let driver = Driver::start(coin.clone(), Duration::from_secs(10));
        Self {
            coin,
            tx_loader: new_tx_loader(pool),
            auth: None,
            driver,
        }
    }
}

impl juniper::Context for Context {}

pub fn check_auth(context: &Context, id_account: i32, check_write: bool) -> FieldResult<()> {
    if let Some(auth) = &context.auth {
        if auth.sub == 0 {
            return Ok(()); // has admin rights
        }
        if auth.sub == id_account as u32 && (!check_write || auth.write) {
            return Ok(());
        }
        return Err(FieldError::new("Unauthorized", juniper::Value::Null));
    }
    Ok(())
}

pub fn check_admin_auth(context: &Context) -> FieldResult<()> {
    if let Some(auth) = &context.auth {
        if auth.sub != 0 {
            return Err(FieldError::new("Unauthorized", juniper::Value::Null));
        }
    }
    Ok(())
}
