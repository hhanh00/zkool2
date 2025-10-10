use anyhow::Result;
use sapling_crypto::keys::FullViewingKey;

use crate::{coin::Network, db::LEDGER_CODE, ledger::connect_ledger};

pub(crate) async fn get_hw_fvk(_network: &Network, ledger_code: u32, aindex: u32) -> Result<FullViewingKey> {
    assert_eq!(ledger_code, LEDGER_CODE);
    let ledger = connect_ledger().await?;
    let fvk = crate::ledger::fvk::get_fvk(&ledger, aindex).await?;
    Ok(fvk)
}
