use anyhow::Result;
use sapling_crypto::keys::FullViewingKey;
use zcash_transparent::address::TransparentAddress;
use sqlx::SqliteConnection;

use crate::{api::{coin::{Coin, Network}, pay::{PcztPackage, SigningEvent}}};
#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))] {
        pub(crate) async fn get_hw_fvk(_network: &Network, ledger_code: u32, aindex: u32) -> Result<FullViewingKey> {
            assert_eq!(ledger_code, crate::db::LEDGER_CODE);
            let ledger = crate::ledger::connect_ledger().await?;
            let fvk = crate::ledger::fvk::get_fvk(&ledger, aindex).await?;
            Ok(fvk)
        }

        pub(crate) async fn get_hw_sapling_address(
            network: &Network,
            aindex: u32,
        ) -> Result<String> {
            let ledger = crate::ledger::connect_ledger().await?;
            let address = crate::ledger::fvk::get_hw_sapling_address(&ledger, network, aindex).await?;
            Ok(address)
        }

        pub(crate) async fn get_hw_transparent_address(
            network: &Network,
            aindex: u32,
            scope: u32,
            dindex: u32,
        ) -> Result<(Vec<u8>, TransparentAddress)> {
            let ledger = crate::ledger::connect_ledger().await?;
            let (pk, address) = crate::ledger::fvk::get_hw_transparent_address(&ledger, network, aindex, scope, dindex).await?;
            Ok((pk, address))
        }

        pub(crate) async fn get_hw_next_diversifier_address(
            network: &Network,
            aindex: u32,
            dindex: u32,
        ) -> Result<(u32, String)> {
            let ledger = crate::ledger::connect_ledger().await?;
            let (dindex, address) = crate::ledger::fvk::get_hw_next_diversifier_address(&ledger, network, aindex, dindex).await?;
            Ok((dindex, address))
        }

        pub(crate) async fn show_sapling_address(network: &Network, connection: &mut SqliteConnection, account: u32) -> Result<String> {
            let address = crate::ledger::fvk::show_sapling_address(network, connection, account).await?;
            Ok(address)
        }

        pub(crate) async fn show_transparent_address(network: &Network, connection: &mut SqliteConnection, account: u32) -> Result<String> {
            let address = crate::ledger::fvk::show_transparent_address(network, connection, account).await?;
            Ok(address)
        }

        #[cfg(feature = "flutter")]
        pub async fn sign_ledger_transaction(
            sink: StreamSink<SigningEvent>,
            package: PcztPackage,
            c: &Coin,
        ) -> Result<()> {
            let connection = c.get_connection().await?;
            crate::ledger::builder::sign_ledger_transaction(c.network(), sink, connection, c.account, package).await?;
            Ok(())
        }
    }
    else {
#[macro_export]
        macro_rules! no_ledger {
            () => {
                Err(anyhow::anyhow!("Ledger not supported on this platform"))
            }
        }

        pub(crate) async fn get_hw_fvk(_network: &Network, _ledger_code: u32, _aindex: u32) -> Result<FullViewingKey> {
            no_ledger!()
        }

        pub(crate) async fn get_hw_sapling_address(
            _network: &Network,
            _aindex: u32,
        ) -> Result<String> {
            no_ledger!()
        }

        pub(crate) async fn get_hw_transparent_address(
            _network: &Network,
            _aindex: u32,
            _scope: u32,
            _dindex: u32,
        ) -> Result<(Vec<u8>, TransparentAddress)> {
            no_ledger!()
        }

        pub(crate) async fn get_hw_next_diversifier_address(
            _network: &Network,
            _aindex: u32,
            _dindex: u32,
        ) -> Result<(u32, String)> {
            no_ledger!()
        }

        pub(crate) async fn show_sapling_address(_network: &Network, _connection: &mut SqliteConnection, _account: u32) -> Result<String> {
            no_ledger!()
        }

        pub(crate) async fn show_transparent_address(_network: &Network, _connection: &mut SqliteConnection, _account: u32) -> Result<String> {
            no_ledger!()
        }

        pub async fn sign_ledger_transaction(
            _sink: StreamSink<SigningEvent>,
            _package: PcztPackage,
            _c: &Coin,
        ) -> Result<()> {
            no_ledger!()
        }
    }
}
