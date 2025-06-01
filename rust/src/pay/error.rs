use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("InvalidPoolMask. Mask must have at least one pool")]
    InvalidPoolMask,
    #[error("Not enough funds, {0} more ZEC required")]
    NotEnoughFunds(String),
    #[error("No Signing Key")]
    NoSigningKey,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
