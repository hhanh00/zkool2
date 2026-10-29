//! One sync at a time per wallet, and the way to stop it.
//!
//! The wallet handle owns a [`SyncSession`]. A sync starts by taking the
//! session with [`SyncSession::try_begin`], which refuses while another run
//! is live, and holds the returned [`SyncRun`] for its whole duration; the
//! run is the proof that the caller holds the lock, and the only source of
//! the cancellation receivers the scan loops listen on. Dropping the run
//! releases the session.
//!
//! A write of the chain log outside the sync itself (a rewind, the details
//! of a transaction, a note the user wrote) takes a [`Held`] connection,
//! which only a run hands out: the function's signature says it must be
//! called under the session, instead of its caller having to know.

use std::{
    ops::{Deref, DerefMut},
    sync::Mutex,
};

use sqlx::SqliteConnection;
use tokio::sync::broadcast;

#[derive(Default)]
pub struct SyncSession {
    lock: tokio::sync::Mutex<()>,
    /// The live run's cancel sender, while there is one.
    cancel: Mutex<Option<broadcast::Sender<()>>>,
}

/// A live sync run: holds the session until dropped.
pub struct SyncRun<'a> {
    _guard: tokio::sync::MutexGuard<'a, ()>,
    session: &'a SyncSession,
    sender: broadcast::Sender<()>,
    /// Subscribed at the start, so a cancel sent at any point of the run is
    /// still there to read when the run ends.
    receiver: broadcast::Receiver<()>,
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
        let (sender, receiver) = broadcast::channel(16);
        *self.cancel.lock().unwrap() = Some(sender.clone());
        SyncRun {
            _guard: guard,
            session: self,
            sender,
            receiver,
        }
    }

    /// Ask the live run to stop. Returns whether there was one.
    pub fn cancel(&self) -> bool {
        match self.cancel.lock().unwrap().as_ref() {
            Some(sender) => sender.send(()).is_ok(),
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
    /// A receiver that fires when the run is cancelled.
    pub fn cancellation(&self) -> broadcast::Receiver<()> {
        self.sender.subscribe()
    }

    /// `connection`, marked as held under this run.
    pub fn held<'a>(&'a self, connection: &'a mut SqliteConnection) -> Held<'a> {
        Held(connection)
    }

    /// Whether a cancel was requested at any point during the run.
    pub fn was_cancelled(&mut self) -> bool {
        self.receiver.try_recv().is_ok()
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

        let mut run = session.try_begin().unwrap();
        let mut listener = run.cancellation();
        assert!(!run.was_cancelled());
        assert!(session.cancel());
        assert!(listener.try_recv().is_ok());
        assert!(run.was_cancelled());
        drop(run);
        assert!(!session.cancel());
    }
}
