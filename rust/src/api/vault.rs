use anyhow::Result;

use flutter_rust_bridge::{DartFnFuture, frb};

use crate::vault::{DartVaultIO, Vault};

#[frb]
pub fn init_vault(
    append: impl Fn(Vec<u8>) -> DartFnFuture<Result<()>> + Send + Sync + 'static,
    read_log: impl Fn() -> DartFnFuture<Result<Vec<u8>>> + Send + Sync + 'static,
) -> Result<DartVault> {
    let io_handler = DartVaultIO::new(append, read_log);
    let vault = Vault::new(io_handler);
    Ok(DartVault(vault))
}

#[frb(opaque)]
pub struct DartVault(Vault<DartVaultIO>);

#[frb]
impl DartVault {
    #[frb]
    pub async fn test(&self) {
        use crate::vault::VaultIO;
        self.0.io_handler.append(vec![1, 2, 3]).await.unwrap();
    }
}
