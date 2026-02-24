use serde::{Deserialize, Serialize};
use warp::reject::Reject;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Claims {
    pub exp: usize,
    pub sub: u32,
}

#[derive(Debug)]
pub struct AuthError;

impl Reject for AuthError {}
