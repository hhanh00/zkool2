use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("InvalidPoolMask. Mask must have at least one pool")]
    InvalidPoolMask,
    #[error("Not enough funds, {0} more ZEC required")]
    NotEnoughFunds(String),
    /// No spendable notes could be selected. In the FROST rounds this is a
    /// transient "the change we just spent has not been mined yet" condition,
    /// so it is a distinct variant the callers can recognize and treat as a
    /// wait rather than a hard failure.
    #[error("No feasible note selection found")]
    NoFeasibleSelection,
    #[error("No Signing Key")]
    NoSigningKey,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
