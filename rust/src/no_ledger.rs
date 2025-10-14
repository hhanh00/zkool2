use anyhow::Result;
use zcash_primitives::legacy::TransparentAddress;
use sqlx::SqliteConnection;

use crate::{coin::Network, frb_generated::StreamSink};

#[macro_export]
macro_rules! no_ledger {
    () => {
        Err(anyhow::anyhow!("Ledger not supported on this platform"))
    }
}

pub async fn get_hw_next_diversifier_address(
    _network: &Network,
    _aindex: u32,
    _dindex: u32,
) -> Result<(u32, String)> {
    no_ledger!()
}

pub async fn get_hw_sapling_address(
    network: &Network,
    aindex: u32,
) -> Result<String> {
    no_ledger!()
}

pub async fn get_hw_transparent_address(
    _network: &Network,
    _aindex: u32,
    _scope: u32,
    _dindex: u32,
) -> Result<(Vec<u8>, TransparentAddress)> {
    no_ledger!()
}

pub async fn sign_ledger_transaction() -> Result<()> {
    no_ledger!()
}

pub async fn show_sapling_address(network: &Network, connection: &mut SqliteConnection, account: u32) -> Result<String> {
    no_ledger!()
}

pub async fn show_transparent_address(network: &Network, connection: &mut SqliteConnection, account: u32) -> Result<String> {
    no_ledger!()
}
