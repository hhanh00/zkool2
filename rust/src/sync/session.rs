//! One sync at a time per wallet, and the way to stop it.
//!
//! The wallet handle owns a [`SyncSession`]. A sync starts by taking the
//! session with [`SyncSession::try_begin`], which refuses while another run
//! is live, and holds the returned [`SyncRun`] for its whole duration; the
//! run is the proof that the caller holds the lock, and the only source of
//! the cancellation token the scan loops listen on. The token is sticky:
//! a stage that starts after the cancel sees it at once, so a cancel sent
//! between two stages is never lost (#1571). Dropping the run releases the
//! session.
//!
//! A write of the chain log outside the sync itself (a rewind, the details
//! of a transaction, a note the user wrote) takes a [`Held`] connection,
//! which only a run hands out: the function's signature says it must be
//! called under the session, instead of its caller having to know.

use std::{
    ops::{Deref, DerefMut},
    sync::Mutex,
};

use anyhow::{bail, Result};
use sqlx::SqliteConnection;
use tokio_util::sync::CancellationToken;

#[derive(Default)]
pub struct SyncSession {
    lock: tokio::sync::Mutex<()>,
    /// The live run's cancellation token, while there is one.
    cancel: Mutex<Option<CancellationToken>>,
}

/// A live sync run: holds the session until dropped.
pub struct SyncRun<'a> {
    _guard: tokio::sync::MutexGuard<'a, ()>,
    session: &'a SyncSession,
    /// Cancelled by [`SyncSession::cancel`]; every stage holds a clone.
    token: CancellationToken,
}

impl SyncSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a run, or `None` while another one is live.
    pub fn try_begin(&self) -> Option<SyncRun<'_>> {
        let guard = self.lock.try_lock().ok()?;
        Some(self.run(guard))
    }

    /// Start a run once the live one, if any, has ended.
    pub async fn begin(&self) -> SyncRun<'_> {
        let guard = self.lock.lock().await;
        self.run(guard)
    }

    fn run<'a>(&'a self, guard: tokio::sync::MutexGuard<'a, ()>) -> SyncRun<'a> {
        let token = CancellationToken::new();
        *self.cancel.lock().unwrap() = Some(token.clone());
        SyncRun {
            _guard: guard,
            session: self,
            token,
        }
    }

    /// Ask the live run to stop. Returns whether there was one.
    pub fn cancel(&self) -> bool {
        match self.cancel.lock().unwrap().as_ref() {
            Some(token) => {
                token.cancel();
                true
            }
            None => false,
        }
    }

    pub fn is_running(&self) -> bool {
        self.cancel.lock().unwrap().is_some()
    }
}

/// A connection to write the chain log through, obtainable only from a
/// [`SyncRun`]: a function that takes one is called under the session.
/// Derefs to the connection, so the row helpers underneath take it as is.
pub struct Held<'a>(&'a mut SqliteConnection);

impl Deref for Held<'_> {
    type Target = SqliteConnection;
    fn deref(&self) -> &SqliteConnection {
        self.0
    }
}

impl DerefMut for Held<'_> {
    fn deref_mut(&mut self) -> &mut SqliteConnection {
        self.0
    }
}

impl SyncRun<'_> {
    /// The run's cancellation token, for a stage to check between steps and
    /// to race its long waits against.
    pub fn cancellation(&self) -> CancellationToken {
        self.token.clone()
    }

    /// `connection`, marked as held under this run.
    pub fn held<'a>(&'a self, connection: &'a mut SqliteConnection) -> Held<'a> {
        Held(connection)
    }

    /// Whether a cancel was requested at any point during the run.
    pub fn was_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    /// Fails the run if a cancel was requested: a stage calls it before it
    /// starts work it would otherwise do in full.
    pub fn check_cancelled(&self) -> Result<()> {
        if self.was_cancelled() {
            bail!("Sync canceled");
        }
        Ok(())
    }
}

impl Drop for SyncRun<'_> {
    fn drop(&mut self) {
        *self.session.cancel.lock().unwrap() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn only_one_run_at_a_time() {
        let session = SyncSession::new();
        assert!(!session.is_running());
        let first = session.try_begin().unwrap();
        assert!(session.is_running());
        assert!(session.try_begin().is_none());
        drop(first);
        assert!(!session.is_running());
        assert!(session.try_begin().is_some());
    }

    #[tokio::test]
    async fn cancel_reaches_a_live_run_and_is_a_noop_without_one() {
        let session = SyncSession::new();
        assert!(!session.cancel());

        let run = session.try_begin().unwrap();
        let listener = run.cancellation();
        assert!(!run.was_cancelled());
        assert!(run.check_cancelled().is_ok());
        assert!(session.cancel());
        assert!(listener.is_cancelled());
        assert!(run.was_cancelled());
        drop(run);
        assert!(!session.cancel());
    }

    /// #1571: a stage that takes its token after the cancel still sees it,
    /// which a broadcast receiver subscribed late did not.
    #[tokio::test]
    async fn a_cancel_is_seen_by_a_stage_that_starts_after_it() {
        let session = SyncSession::new();
        let run = session.try_begin().unwrap();
        assert!(session.cancel());
        let late = run.cancellation();
        assert!(late.is_cancelled());
        late.cancelled().await;
        let m = run.check_cancelled().unwrap_err().to_string();
        assert_eq!(m, "Sync canceled");
    }
}
