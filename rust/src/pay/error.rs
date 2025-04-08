use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("InvalidPoolMask. Mask must have at least one pool")]
    InvalidPoolMask,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
