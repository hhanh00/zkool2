use crate::{api::coin::Coin, graphql::query::{TxLoader, new_tx_loader}};

pub mod data;
pub mod frost;
pub mod query;
pub mod mutation;
pub mod subs;

#[derive(Clone)]
pub struct Context {
    pub coin: Coin,
    pub tx_loader: TxLoader,
}

impl Context {
    pub fn new(coin: Coin) -> Self {
        let pool = coin.get_pool().unwrap();
        Self {
            coin,
            tx_loader: new_tx_loader(pool),
        }
    }
}

impl juniper::Context for Context {}
