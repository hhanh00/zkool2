use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("InvalidPoolMask. Mask must have at least one pool")]
    InvalidPoolMask,
    #[error("Multiple recipients, but recipient pays fee")]
    TooManyRecipients,
    #[error("Recipient pays fee, but amount is not enough")]
    RecipientNotEnoughAmount,
    #[error("Not enough funds")]
    NotEnoughFunds,
    #[error("No Signing Key")]
    NoSigningKey,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
