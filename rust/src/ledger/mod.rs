use anyhow::Result;
use sapling_crypto::keys::FullViewingKey;
use sqlx::SqliteConnection;
use zcash_transparent::address::TransparentAddress;

use tonic::async_trait;

use crate::api::coin::Network;
use crate::api::pay::{PcztPackage, SigningEvent};
use crate::Sink;

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
        pub mod official;
        pub mod official_sign;

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

    async fn get_ufvk(&self, _network: &Network, _aindex: u32) -> Result<String> {
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
}

/// Concrete enum dispatch for hardware-wallet operations.
///
/// All address-import and signing operations funnel through a single
/// `LedgerHandle` value so adding a new wallet app only requires a new
/// variant here rather than separate dispatch points in `get_ledger` and
/// `sign_ledger_pczt`.
pub enum LedgerHandle {
    Stub(mock::StubLedger),
    #[cfg(feature = "ledger")]
    Zondax(nano::ZondaxApp),
    #[cfg(feature = "ledger")]
    Official(official::OfficialApp),
}

impl LedgerHandle {
    pub fn from_hw(hw: u8) -> Self {
        match HwKind::from_hw(hw) {
            HwKind::Zondax => {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "ledger")] {
                        Self::Zondax(nano::ZondaxApp {})
                    } else {
                        Self::Stub(mock::StubLedger::no_support())
                    }
                }
            }
            HwKind::Official => {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "ledger")] {
                        Self::Official(official::OfficialApp {})
                    } else {
                        Self::Stub(mock::StubLedger::official())
                    }
                }
            }
            HwKind::Software => Self::Stub(mock::StubLedger::software()),
        }
    }

    pub fn kind(&self) -> HwKind {
        match self {
            Self::Stub(s) => s.kind(),
            #[cfg(feature = "ledger")]
            Self::Zondax(z) => z.kind(),
            #[cfg(feature = "ledger")]
            Self::Official(o) => o.kind(),
        }
    }

    pub async fn get_transparent_pubkey(
        &self,
        network: &Network,
        aindex: u32,
        scope: u32,
        dindex: u32,
    ) -> Result<(Vec<u8>, TransparentAddress)> {
        match self {
            Self::Stub(s) => s.get_transparent_pubkey(network, aindex, scope, dindex).await,
            #[cfg(feature = "ledger")]
            Self::Zondax(z) => z.get_transparent_pubkey(network, aindex, scope, dindex).await,
            #[cfg(feature = "ledger")]
            Self::Official(o) => o.get_transparent_pubkey(network, aindex, scope, dindex).await,
        }
    }

    pub async fn next_diversifier_address(
        &self,
        network: &Network,
        aindex: u32,
        dindex: u32,
    ) -> Result<(u32, String)> {
        match self {
            Self::Stub(s) => s.next_diversifier_address(network, aindex, dindex).await,
            #[cfg(feature = "ledger")]
            Self::Zondax(z) => z.next_diversifier_address(network, aindex, dindex).await,
            #[cfg(feature = "ledger")]
            Self::Official(o) => o.next_diversifier_address(network, aindex, dindex).await,
        }
    }

    pub async fn show_transparent_address(
        &self,
        network: &Network,
        connection: &mut SqliteConnection,
        account: u32,
    ) -> Result<String> {
        match self {
            Self::Stub(s) => s.show_transparent_address(network, connection, account).await,
            #[cfg(feature = "ledger")]
            Self::Zondax(z) => z.show_transparent_address(network, connection, account).await,
            #[cfg(feature = "ledger")]
            Self::Official(o) => o.show_transparent_address(network, connection, account).await,
        }
    }

    pub async fn import_sapling_fvk(
        &self,
        network: &Network,
        aindex: u32,
    ) -> Result<FullViewingKey> {
        match self {
            Self::Stub(s) => s.import_sapling_fvk(network, aindex).await,
            #[cfg(feature = "ledger")]
            Self::Zondax(z) => z.import_sapling_fvk(network, aindex).await,
            #[cfg(feature = "ledger")]
            Self::Official(o) => o.import_sapling_fvk(network, aindex).await,
        }
    }

    pub async fn get_sapling_address(&self, network: &Network, aindex: u32) -> Result<String> {
        match self {
            Self::Stub(s) => s.get_sapling_address(network, aindex).await,
            #[cfg(feature = "ledger")]
            Self::Zondax(z) => z.get_sapling_address(network, aindex).await,
            #[cfg(feature = "ledger")]
            Self::Official(o) => o.get_sapling_address(network, aindex).await,
        }
    }

    pub async fn get_ufvk(&self, network: &Network, aindex: u32) -> Result<String> {
        match self {
            Self::Stub(s) => s.get_ufvk(network, aindex).await,
            #[cfg(feature = "ledger")]
            Self::Zondax(z) => z.get_ufvk(network, aindex).await,
            #[cfg(feature = "ledger")]
            Self::Official(o) => o.get_ufvk(network, aindex).await,
        }
    }

    pub async fn show_sapling_address(
        &self,
        network: &Network,
        connection: &mut SqliteConnection,
        account: u32,
    ) -> Result<String> {
        match self {
            Self::Stub(s) => s.show_sapling_address(network, connection, account).await,
            #[cfg(feature = "ledger")]
            Self::Zondax(z) => z.show_sapling_address(network, connection, account).await,
            #[cfg(feature = "ledger")]
            Self::Official(o) => o.show_sapling_address(network, connection, account).await,
        }
    }

    #[cfg_attr(not(feature = "ledger"), allow(unused_variables))]
    pub async fn sign_pczt<S>(
        &self,
        network: Network,
        sink: &S,
        connection: &mut SqliteConnection,
        account: u32,
        package: PcztPackage,
    ) -> Result<PcztPackage>
    where
        S: Sink<SigningEvent> + Sync,
    {
        match self {
            Self::Stub(_) => anyhow::bail!("signing not supported: {}", self.kind().error_msg()),
            #[cfg(feature = "ledger")]
            Self::Zondax(z) => z.sign_pczt(network, sink, connection, account, package).await,
            #[cfg(feature = "ledger")]
            Self::Official(o) => o.sign_pczt(network, sink, connection, account, package).await,
        }
    }
}

impl HwKind {
    fn error_msg(self) -> &'static str {
        match self {
            HwKind::Software => "account is not a hardware wallet",
            HwKind::Zondax | HwKind::Official => "this build has no Ledger support",
        }
    }
}
