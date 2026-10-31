use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::frost::driver::Driver;
use crate::graphql::jwt::Claims;
use crate::{
    api::{coin::Coin, voting_drive::VotingDrive},
    graphql::query::{new_tx_loader, TxLoader},
};
use juniper::{FieldError, FieldResult};

pub mod data;
pub mod frost;
pub mod jwt;
pub mod mutation;
pub mod query;
pub mod swaps;
pub mod subs;
pub mod voting;

#[derive(Clone)]
pub struct Context {
    pub coin: Coin,
    pub auth: Option<Claims>,
    pub tx_loader: TxLoader,
    /// The one task that advances DKG and signing; see `frost::driver`.
    pub driver: Driver,
    /// The submission runs this server started, by round id and account:
    /// two accounts can take part in the same round, each with its own
    /// run. A stopped run stays until the next submission, so
    /// `votingSubmissionStatus` can read how it ended; the sidecar answers
    /// for it afterwards.
    pub voting_drives: Arc<tokio::sync::Mutex<HashMap<(String, u32), VotingDrive>>>,
    /// FROST and DKG are offered. A server with token authentication is
    /// multi-tenant, and a FROST session is wallet-wide, so it refuses
    /// them (#1529); the single-operator server keeps them.
    pub frost_enabled: bool,
}

impl Context {
    /// Build the server's context and start its protocol driver, which
    /// polls the chain tip at the same cadence as the app's block watcher.
    /// Needs a tokio runtime.
    pub fn new(coin: Coin) -> Self {
        let pool = coin.get_pool().unwrap();
        let driver = Driver::start(coin.clone(), Duration::from_secs(10), ());
        Self {
            coin,
            tx_loader: new_tx_loader(pool),
            auth: None,
            driver,
            voting_drives: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            frost_enabled: true,
        }
    }
}

impl juniper::Context for Context {}

/// Seconds since the epoch, as a JWT's `exp` counts them.
pub(crate) fn now() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as usize)
        .unwrap_or(0)
}

/// The token's claims are checked on every operation, expiry included: a
/// WebSocket presents its token once, at the handshake, and then outlives
/// it, so `exp` is enforced here rather than by the connection.
pub fn check_auth(context: &Context, id_account: i32, check_write: bool) -> FieldResult<()> {
    authorize(context.auth.as_ref(), id_account, check_write, now())
}

pub fn check_admin_auth(context: &Context) -> FieldResult<()> {
    authorize_admin(context.auth.as_ref(), now())
}

fn authorize(auth: Option<&Claims>, id_account: i32, check_write: bool, now: usize) -> FieldResult<()> {
    if let Some(auth) = auth {
        unexpired(auth, now)?;
        if auth.sub == 0 {
            return Ok(()); // has admin rights
        }
        if auth.sub == id_account as u32 && (!check_write || auth.write) {
            return Ok(());
        }
        return Err(FieldError::new("Unauthorized", juniper::Value::Null));
    }
    Ok(())
}

fn authorize_admin(auth: Option<&Claims>, now: usize) -> FieldResult<()> {
    if let Some(auth) = auth {
        unexpired(auth, now)?;
        if auth.sub != 0 {
            return Err(FieldError::new("Unauthorized", juniper::Value::Null));
        }
    }
    Ok(())
}

/// Distinct from "Unauthorized": the client should re-authenticate, not
/// ask for less.
fn unexpired(auth: &Claims, now: usize) -> FieldResult<()> {
    if now >= auth.exp {
        return Err(FieldError::new("Token expired", juniper::Value::Null));
    }
    Ok(())
}

#[cfg(test)]
mod auth_tests {
    use super::*;

    fn claims(sub: u32, write: bool, exp: usize) -> Claims {
        Claims { exp, sub, write }
    }

    fn message(r: FieldResult<()>) -> String {
        r.err().map(|e| e.message().to_string()).unwrap_or_default()
    }

    #[test]
    fn an_expired_token_is_refused_whatever_its_scope() {
        let admin = claims(0, true, 100);
        let writer = claims(7, true, 100);
        assert_eq!(message(authorize(Some(&admin), 7, true, 100)), "Token expired");
        assert_eq!(message(authorize(Some(&writer), 7, false, 101)), "Token expired");
        assert_eq!(message(authorize_admin(Some(&admin), 100)), "Token expired");
    }

    #[test]
    fn an_unexpired_token_keeps_its_scope() {
        let writer = claims(7, true, 100);
        let reader = claims(7, false, 100);
        assert!(authorize(Some(&writer), 7, true, 99).is_ok());
        assert!(authorize(Some(&reader), 7, false, 99).is_ok());
        assert_eq!(message(authorize(Some(&reader), 7, true, 99)), "Unauthorized");
        assert_eq!(message(authorize(Some(&reader), 8, false, 99)), "Unauthorized");
        assert!(authorize(Some(&claims(0, false, 100)), 8, true, 99).is_ok());
        assert_eq!(message(authorize_admin(Some(&reader), 99)), "Unauthorized");
    }

    #[test]
    fn without_a_token_everything_is_allowed_as_before() {
        assert!(authorize(None, 7, true, usize::MAX).is_ok());
        assert!(authorize_admin(None, usize::MAX).is_ok());
    }
}
