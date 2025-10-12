use anyhow::Result;
use zcash_primitives::legacy::TransparentAddress;

use crate::{coin::Network, frb_generated::StreamSink};

pub async fn get_hw_next_diversifier_address(
    _network: &Network,
    _aindex: u32,
    _dindex: u32,
) -> Result<(u32, String)> {
    Err(anyhow::anyhow!("Ledger not supported on this platform"))
}

pub async fn get_hw_transparent_address(
    _network: &Network,
    _aindex: u32,
    _scope: u32,
    _dindex: u32,
) -> Result<(Vec<u8>, TransparentAddress)> {
    Err(anyhow::anyhow!("Ledger not supported on this platform"))
}

pub async fn sign_ledger_transaction() -> Result<()> {
    Err(anyhow::anyhow!("Ledger not supported on this platform"))
}
