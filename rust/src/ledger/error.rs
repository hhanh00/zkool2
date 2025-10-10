use hidapi::HidError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Ledger Error {0}: {1}")]
    Generic(u16, String),
    #[error("No Device Found. Is it connected and unlocked?")]
    NotFound,
    #[error("Error Executing Instruction {1}: {0}")]
    Execute(u16, u8),
    #[error("Protocol Error: {0}")]
    Protocol(String),
    #[error(transparent)]
    Hid(#[from] HidError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    ZEMU(#[from] ledger_transport_zemu::LedgerZemuError),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
