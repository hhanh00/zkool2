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

pub use dart::DartVaultIO;
use tonic::async_trait;
