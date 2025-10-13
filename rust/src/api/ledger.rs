use anyhow::Result;
use sapling_crypto::keys::FullViewingKey;

use crate::{coin::Network, db::LEDGER_CODE};

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))] {
        pub(crate) async fn get_hw_fvk(_network: &Network, ledger_code: u32, aindex: u32) -> Result<FullViewingKey> {
            assert_eq!(ledger_code, LEDGER_CODE);
            let ledger = crate::ledger::connect_ledger().await?;
            let fvk = crate::ledger::fvk::get_fvk(&ledger, aindex).await?;
            Ok(fvk)
        }
    }
    else {
        pub(crate) async fn get_hw_fvk(_network: &Network, ledger_code: u32, aindex: u32) -> Result<FullViewingKey> {
            crate::no_ledger!()
        }

        pub async fn show_ledger_sapling_address() -> Result<String> {
            Err(anyhow::anyhow!("Ledger is not supported on mobile devices"))
        }
    }
}
