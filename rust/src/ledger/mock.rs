use anyhow::Result;
use sqlx::SqliteConnection;
use tonic::async_trait;
use zcash_transparent::address::TransparentAddress;

use crate::{
    api::{
        coin::{Coin, Network},
        pay::{PcztPackage, SigningEvent},
    },
    frb_generated::StreamSink,
    ledger::{HwKind, LedgerApp},
};

/// Placeholder device for accounts that cannot perform device operations:
/// software accounts, builds without the `ledger` feature, and the Official
/// Ledger app whose device protocol is not implemented yet.
pub struct StubLedger {
    kind: HwKind,
    error: &'static str,
}

impl StubLedger {
    pub fn software() -> Self {
        Self {
            kind: HwKind::Software,
            error: "account is not a hardware wallet",
        }
    }

    pub fn no_support() -> Self {
        Self {
            kind: HwKind::Software,
            error: "this build has no Ledger support",
        }
    }

    pub fn official() -> Self {
        Self {
            kind: HwKind::Official,
            error: "not implemented yet for the Official Ledger app",
        }
    }
}

#[async_trait]
impl LedgerApp for StubLedger {
    fn kind(&self) -> HwKind {
        self.kind
    }

    async fn get_transparent_pubkey(
        &self,
        _network: &Network,
        _aindex: u32,
        _scope: u32,
        _dindex: u32,
    ) -> Result<(Vec<u8>, TransparentAddress)> {
        anyhow::bail!("{}", self.error)
    }

    async fn next_diversifier_address(
        &self,
        _network: &Network,
        _aindex: u32,
        _dindex: u32,
    ) -> Result<(u32, String)> {
        anyhow::bail!("{}", self.error)
    }

    async fn show_transparent_address(
        &self,
        _network: &Network,
        _connection: &mut SqliteConnection,
        _account: u32,
    ) -> Result<String> {
        anyhow::bail!("{}", self.error)
    }

    async fn show_sapling_address(
        &self,
        _network: &Network,
        _connection: &mut SqliteConnection,
        _account: u32,
    ) -> Result<String> {
        anyhow::bail!("{}", self.error)
    }

    async fn sign_pczt(&self, _sink: StreamSink<SigningEvent>, _package: PcztPackage, _c: &Coin) -> Result<()> {
        anyhow::bail!("{}", self.error)
    }
}
