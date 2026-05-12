use serde::{Deserialize, Serialize};
use warp::reject::Reject;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Claims {
    pub exp: usize,
    pub sub: u32,
    pub write: bool,
}

#[derive(Debug)]
pub struct AuthError;

impl Reject for AuthError {}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Authentication failed")
    }
}

impl std::error::Error for AuthError {}
