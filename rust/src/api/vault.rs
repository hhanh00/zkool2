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

    #[frb]
    pub async fn set_master_password(
        &self,
        old_password: Option<String>,
        new_password: String,
        old_bytes: Option<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        Vault::<DartVaultIO>::set_master_password(old_password, new_password, old_bytes).await
    }

    #[frb]
    pub async fn store_account(
        &self,
        name: String,
        seed: String,
        aindex: u32,
        use_internal: bool,
        birth_height: u32,
        pk: Vec<u8>,
    ) -> Result<()> {
        self.0.store_account(name, seed, aindex, use_internal, birth_height, pk).await
    }
}
