use anyhow::Result;
use flutter_rust_bridge::{DartFnFuture};
use tonic::async_trait;

use crate::vault::VaultIO;

pub struct DartVaultIO {
    append: Box<dyn Fn(Vec<u8>) -> DartFnFuture<Result<()>> + Send + Sync>,
}

impl DartVaultIO {
    pub fn new(
        append: impl Fn(Vec<u8>) -> DartFnFuture<Result<()>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            append: Box::new(append),
        }
    }
}

impl std::fmt::Debug for DartVaultIO {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[async_trait]
impl VaultIO for DartVaultIO {
    async fn append(&self, entry_bytes: Vec<u8>) -> Result<()> {
        (self.append)(entry_bytes).await
    }
}

impl super::Vault<DartVaultIO> {
    pub fn new(io_handler: DartVaultIO) -> Self {
        Self {
            io_handler,
        }
    }
}