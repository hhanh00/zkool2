use anyhow::{Result, anyhow};

// #[cfg(flutter)]
mod crypto;
mod dart;

#[async_trait]
pub trait VaultIO {
    /// Append a serialized log entry to the log file.
    async fn append(&self, entry_bytes: Vec<u8>) -> Result<()>;

    /// Read the full log file as raw bytes. Only called at recovery time, not during normal operation.
    async fn read_log(&self) -> Result<Vec<u8>>;
}

#[derive(Debug)]
pub struct Vault<IO: VaultIO> {
    pub(crate) io_handler: IO,
}

impl<IO: VaultIO> Vault<IO> {
    pub async fn set_master_password(
        old_password: Option<String>,
        _new_password: String,
        old_bytes: Option<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        match (old_password, old_bytes) {
            (None, None) => {
                // New vault
                return crypto::derive_master_key(&_new_password);
            }
            (Some(_old_password), Some(_bytes)) => {
                // Existing vault
                // Decrypt current vault with old_password
                // Encrypt with new password
            }
            _ => unreachable!(),
        }
        Ok(vec![])
    }

    pub async fn store_account(
        &self,
        name: String,
        seed: String,
        aindex: u32,
        _use_internal: bool,
        birth_height: u32,
        pk: Vec<u8>,
    ) -> Result<()> {
        let mnemonic = bip39::Mnemonic::parse_normalized(&seed)?;
        let (entropy_arr, entropy_len) = mnemonic.to_entropy_array();
        let mut entropy = [0u8; 32];
        entropy.copy_from_slice(&entropy_arr[..entropy_len]);

        let xpk = x25519_dalek::PublicKey::from(<[u8; 32]>::try_from(pk).map_err(|_| anyhow::anyhow!("Invalid pk length"))?);

        let account = crypto::AccountPayload { name, entropy, aindex, use_internal: _use_internal, birth_height };
        let entry_bytes = crypto::encrypt_account(account, xpk)?;

        self.io_handler.append(entry_bytes).await?;
        Ok(())
    }
}

pub use dart::DartVaultIO;
use tonic::async_trait;
