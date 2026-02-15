use anyhow::Result;
use sapling_crypto::keys::FullViewingKey;
use sqlx::SqliteConnection;
use tonic::async_trait;
use zcash_transparent::address::TransparentAddress;

use crate::{api::{coin::{Coin, Network}, pay::{PcztPackage, SigningEvent}}, frb_generated::StreamSink, ledger::HWAPI};

pub struct NanoLedger {}

#[async_trait]
impl HWAPI for NanoLedger {
    async fn get_hw_fvk(&self, _network: &Network, aindex: u32) -> Result<FullViewingKey> {
        let ledger = crate::ledger::transport::connect_ledger().await?;
        let fvk = crate::ledger::fvk::get_fvk(&ledger, aindex).await?;
        Ok(fvk)
    }
    async fn get_hw_sapling_address(&self, network: &Network, aindex: u32) -> Result<String> {
        let ledger = crate::ledger::transport::connect_ledger().await?;
        let address = crate::ledger::fvk::get_hw_sapling_address(&ledger, network, aindex).await?;
        Ok(address)
    }
    async fn get_hw_transparent_address(
        &self,
        network: &Network,
        aindex: u32,
        scope: u32,
        dindex: u32,
    ) -> Result<(Vec<u8>, TransparentAddress)> {
        let ledger = crate::ledger::transport::connect_ledger().await?;
        let (pk, address) =
            crate::ledger::fvk::get_hw_transparent_address(&ledger, network, aindex, scope, dindex)
                .await?;
        Ok((pk, address))
    }

    async fn get_hw_next_diversifier_address(
        &self,
        network: &Network,
        aindex: u32,
        dindex: u32,
    ) -> Result<(u32, String)> {
        let ledger = crate::ledger::transport::connect_ledger().await?;
        let (dindex, address) =
            crate::ledger::fvk::get_hw_next_diversifier_address(&ledger, network, aindex, dindex)
                .await?;
        Ok((dindex, address))
    }

    async fn show_sapling_address(
        &self,
        network: &Network,
        connection: &mut SqliteConnection,
        account: u32,
    ) -> Result<String> {
        let address =
            crate::ledger::fvk::show_sapling_address(network, connection, account).await?;
        Ok(address)
    }

    async fn show_transparent_address(
        &self,
        network: &Network,
        connection: &mut SqliteConnection,
        account: u32,
    ) -> Result<String> {
        let address =
            crate::ledger::fvk::show_transparent_address(network, connection, account).await?;
        Ok(address)
    }

    async fn sign_ledger_transaction(
        &self,
        sink: StreamSink<SigningEvent>,
        package: PcztPackage,
        c: &Coin,
    ) -> Result<()> {
        let connection = c.get_connection().await?;
        crate::ledger::builder::sign_ledger_transaction(
            c.network(),
            sink,
            connection,
            c.account,
            package,
        )
        .await?;
        Ok(())
    }
}
