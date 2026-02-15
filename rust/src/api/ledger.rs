use anyhow::Result;
use sapling_crypto::keys::FullViewingKey;
use sqlx::SqliteConnection;
use zcash_transparent::address::TransparentAddress;

use crate::api::{
    coin::{Coin, Network},
    pay::{PcztPackage, SigningEvent},
};
#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;

pub(crate) async fn get_hw_transparent_address(
    network: &Network,
    aindex: u32,
    scope: u32,
    dindex: u32,
) -> Result<(Vec<u8>, TransparentAddress)> {
}

pub(crate) async fn get_hw_next_diversifier_address(
    network: &Network,
    aindex: u32,
    dindex: u32,
) -> Result<(u32, String)> {
}

pub(crate) async fn show_sapling_address(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<String> {
}

pub(crate) async fn show_transparent_address(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<String> {
}

