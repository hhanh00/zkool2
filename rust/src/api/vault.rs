use anyhow::Result;

use flutter_rust_bridge::{DartFnFuture, frb};

use crate::vault::{DartVaultIO, Vault};

#[frb]
pub fn init_vault(
    append: impl Fn(Vec<u8>) -> DartFnFuture<Result<()>> + Send + Sync + 'static,
) -> Result<DartVault> {
    let io_handler = DartVaultIO::new(append);
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
    pub async fn register_device(
        &self,
        init_bytes: Vec<u8>,
        master_password: String,
        device_id_str: String,
        prf_output: Vec<u8>,
    ) -> Result<()> {
        let prf = <[u8; 32]>::try_from(prf_output)
            .map_err(|_| anyhow::anyhow!("Invalid PRF output length, expected 32 bytes"))?;
        self.0.register_device(init_bytes, master_password, device_id_str, prf).await
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

    #[frb]
    pub fn recover(&self, vault_bytes: Vec<u8>, master_password: String) -> Result<Vec<RestoredAccount>> {
        let accounts = crate::vault::crypto::recover(&vault_bytes, &master_password)?;
        Ok(accounts)
    }

    #[frb]
    pub fn recover_with_prf(
        &self,
        vault_bytes: Vec<u8>,
        device_id_str: String,
        prf_output: Vec<u8>,
    ) -> Result<Vec<RestoredAccount>> {
        let prf = <[u8; 32]>::try_from(prf_output)
            .map_err(|_| anyhow::anyhow!("Invalid PRF output length, expected 32 bytes"))?;
        let accounts = crate::vault::crypto::recover_with_prf(&vault_bytes, &device_id_str, prf)?;
        Ok(accounts)
    }
}

pub struct RestoredAccount {
    pub name: String,
    pub seed: String,
    pub aindex: u32,
    pub use_internal: bool,
    pub birth_height: u32,
}
