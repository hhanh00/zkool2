use anyhow::Result;
use sapling_crypto::keys::FullViewingKey;
use sqlx::SqliteConnection;
use zcash_transparent::address::TransparentAddress;

use tonic::async_trait;

use crate::api::coin::{Coin, Network};
use crate::api::pay::{PcztPackage, SigningEvent};
use crate::frb_generated::StreamSink;

pub mod error;
pub type LedgerError = error::Error;
pub type LedgerResult<T> = std::result::Result<T, LedgerError>;

#[async_trait]
pub trait HWAPI {
    async fn get_hw_fvk(
        &self,
        network: &Network,
        aindex: u32,
    ) -> Result<FullViewingKey>;
    async fn get_hw_sapling_address(&self, network: &Network, aindex: u32) -> Result<String>;
    async fn get_hw_transparent_address(
        &self,
        network: &Network,
        aindex: u32,
        scope: u32,
        dindex: u32,
    ) -> Result<(Vec<u8>, TransparentAddress)>;

    async fn get_hw_next_diversifier_address(
        &self,
        network: &Network,
        aindex: u32,
        dindex: u32,
    ) -> Result<(u32, String)>;
    async fn show_sapling_address(
        &self,
        network: &Network,
        connection: &mut SqliteConnection,
        account: u32,
    ) -> Result<String>;
    async fn show_transparent_address(
        &self,
        network: &Network,
        connection: &mut SqliteConnection,
        account: u32,
    ) -> Result<String>;
    async fn sign_ledger_transaction(
        &self,
        sink: StreamSink<SigningEvent>,
        package: PcztPackage,
        c: &Coin,
    ) -> Result<()>;
}

pub mod mock;

cfg_if::cfg_if! {
    if #[cfg(feature="ledger")] {
        pub mod transport;
        pub mod builder;
        pub mod fvk;
        pub mod hashers;
        pub mod nano;
        // pub mod legacy;

        #[cfg(test)]
        mod tests;
    }
}

