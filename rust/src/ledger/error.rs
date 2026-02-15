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
    #[error("Transaction Too Complex, check the number of inputs/outputs")]
    TooComplex,
    #[error("Transaction has Orchard actions. Orchard is not supported")]
    HasOrchard,
    #[error("Invalid Output")]
    InvalidOut,
    #[cfg(feature = "ledger")]
    #[error(transparent)]
    Hid(#[from] hidapi::HidError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[cfg(feature = "zemu")]
    #[error(transparent)]
    ZEMU(#[from] ledger_transport_zemu::LedgerZemuError),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
