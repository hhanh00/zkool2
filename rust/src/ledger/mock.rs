use anyhow::Result;
use sapling_crypto::keys::FullViewingKey;
use sqlx::SqliteConnection;
use tonic::async_trait;
use zcash_transparent::address::TransparentAddress;

use crate::{api::{coin::{Coin, Network}, pay::{PcztPackage, SigningEvent}}, frb_generated::StreamSink, ledger::HWAPI};

#[async_trait]
impl HWAPI for () {
    async fn get_hw_fvk(
        &self,
        _network: &Network,
        _aindex: u32,
    ) -> Result<FullViewingKey> {
        unimplemented!()
    }
    async fn get_hw_sapling_address(&self, _network: &Network, _aindex: u32) -> Result<String> {
        unimplemented!()
    }
    async fn get_hw_transparent_address(
        &self,
        _network: &Network,
        _aindex: u32,
        _scope: u32,
        _dindex: u32,
    ) -> Result<(Vec<u8>, TransparentAddress)> {
        unimplemented!()
    }
    async fn get_hw_next_diversifier_address(
        &self,
        _network: &Network,
        _aindex: u32,
        _dindex: u32,
    ) -> Result<(u32, String)> {
        unimplemented!()
    }
    async fn show_sapling_address(
        &self,
        _network: &Network,
        _connection: &mut SqliteConnection,
        _account: u32,
    ) -> Result<String> {
        unimplemented!()
    }
    async fn show_transparent_address(
        &self,
        _network: &Network,
        _connection: &mut SqliteConnection,
        _account: u32,
    ) -> Result<String> {
        unimplemented!()
    }
    async fn sign_ledger_transaction(
        &self,
        _sink: StreamSink<SigningEvent>,
        _package: PcztPackage,
        _c: &Coin,
    ) -> Result<()> {
        unimplemented!()
    }
}

