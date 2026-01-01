use crate::api::coin::Coin;

pub mod data;
pub mod query;
pub mod mutation;

pub struct Context {
    pub coin: Coin,
}

impl Context {
    pub fn new(coin: Coin) -> Self {
        Self {
            coin,
        }
    }
}

impl juniper::Context for Context {}
