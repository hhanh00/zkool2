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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwKind {
    Software = 0,
    Zondax = 1,
    Official = 2,
}

impl HwKind {
    pub fn from_hw(hw: u8) -> Self {
        match hw {
            1 => HwKind::Zondax,
            2 => HwKind::Official,
            _ => HwKind::Software,
        }
    }

    pub fn is_ledger(self) -> bool {
        self != HwKind::Software
    }
}

pub mod mock;

cfg_if::cfg_if! {
    if #[cfg(feature="ledger")] {
        pub mod transport;
        pub mod builder;
        pub mod fvk;
        pub mod hashers;
        pub mod nano;

        #[cfg(test)]
        mod tests;
    }
}

#[async_trait]
pub trait LedgerApp: Send + Sync {
    fn kind(&self) -> HwKind;

    async fn get_transparent_pubkey(
        &self,
        _network: &Network,
        _aindex: u32,
        _scope: u32,
        _dindex: u32,
    ) -> Result<(Vec<u8>, TransparentAddress)> {
        anyhow::bail!("not supported by this Ledger app")
    }

    async fn next_diversifier_address(
        &self,
        _network: &Network,
        _aindex: u32,
        _dindex: u32,
    ) -> Result<(u32, String)> {
        anyhow::bail!("not supported by this Ledger app")
    }

    async fn show_transparent_address(
        &self,
        _network: &Network,
        _connection: &mut SqliteConnection,
        _account: u32,
    ) -> Result<String> {
        anyhow::bail!("not supported by this Ledger app")
    }

    async fn import_sapling_fvk(
        &self,
        _network: &Network,
        _aindex: u32,
    ) -> Result<FullViewingKey> {
        anyhow::bail!("not supported by this Ledger app")
    }

    async fn get_sapling_address(&self, _network: &Network, _aindex: u32) -> Result<String> {
        anyhow::bail!("not supported by this Ledger app")
    }

    async fn show_sapling_address(
        &self,
        _network: &Network,
        _connection: &mut SqliteConnection,
        _account: u32,
    ) -> Result<String> {
        anyhow::bail!("not supported by this Ledger app")
    }

    async fn sign_pczt(
        &self,
        _sink: StreamSink<SigningEvent>,
        _package: PcztPackage,
        _c: &Coin,
    ) -> Result<()> {
        anyhow::bail!("not supported by this Ledger app")
    }
}
