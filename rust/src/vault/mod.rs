use anyhow::Result;

// #[cfg(flutter)]
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
                // Use _new_password to derive new MP vault
            }
            (Some(_old_password), Some(_bytes)) => {
                // Existing vault
                // Decrypt current vault with old_password
                // Encrypt with new password
            }
            _ => unreachable!()
        }
        Ok(vec![])
    }
}

pub use dart::DartVaultIO;
use tonic::async_trait;
