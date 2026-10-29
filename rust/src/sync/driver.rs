//! The sync driver: the one task through which the wallet's chain log is
//! written outside a sync.
//!
//! A sync holds the wallet's [`SyncSession`](super::session::SyncSession)
//! for its whole run, so two syncs never overlap. The other writers of the
//! chain log did not take the session: a rewind trimmed rows under a sync
//! that was still scanning them, and the app's detail pass wrote through
//! row ids it had read before a rewind deleted the rows and a later sync
//! handed the ids to other transactions. So every such write goes through
//! this task's mailbox, and runs under the session, after whatever the
//! driver was doing:
//!
//! - a [`Request::Sync`] is a sync, as the app and the server ask for one;
//! - a [`Request::Rewind`] trims an account, or resets it to its birth. A
//!   sync in flight is cancelled first: it resumes from its last checkpoint
//!   next time, and the user who asked for the rewind is not kept waiting;
//! - a [`Request::Apply`] is a short, database-only mutation prepared
//!   outside the session: the details the app's background pass decrypted,
//!   or a note the user wrote on a transaction.
//!
//! Requests run one at a time, in the order they arrived. Each carries the
//! caller's wallet handle: the handle is rebuilt by every settings change
//! (server, transport), so the task keeps none of its own. The FROST
//! protocol driver takes the same session directly and is serialized by it,
//! not by this mailbox: it sees the session busy and retries on its next
//! tick.

use std::{collections::VecDeque, future::Future};

use anyhow::{anyhow, Result};
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};

use crate::{
    api::{
        coin::Coin,
        sync::{SyncEvent, SyncOutcome},
    },
    memo::DecryptedTx,
};

/// Transactions the app's detail pass decrypts before queueing one
/// [`Apply::TxDetails`]: enough to amortize the queueing, few enough that
/// a rewind between batches loses little.
pub const DETAILS_BATCH: usize = 100;

/// How far a rewind goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rewind {
    /// To the last checkpoint at or before this height.
    To(u32),
    /// To the account's birth height.
    Reset,
}

/// A mutation of the chain log prepared outside the session.
#[derive(Debug)]
pub enum Apply {
    /// Details decrypted by the app's background pass. Each one is stored
    /// against the transaction as it is found then: one rewound meanwhile
    /// is dropped, and the sync that records it again leaves it for the
    /// next pass.
    TxDetails { account: u32, txs: Vec<DecryptedTx> },
    /// The user's note on a transaction, or none to clear it.
    UserMemo {
        account: u32,
        txid: Vec<u8>,
        memo: Option<String>,
    },
}

/// What a sync is asked to do, as the app and the server pass it.
#[derive(Clone, Debug)]
pub struct SyncRequest {
    pub accounts: Vec<u32>,
    pub current_height: u32,
    pub actions_per_sync: u32,
    pub transparent_limit: u32,
    pub checkpoint_age: u32,
    /// Skip the detail pass; the app fetches details afterwards.
    pub skip_details: bool,
}

enum Request {
    Sync {
        coin: Coin,
        request: SyncRequest,
        progress: mpsc::Sender<SyncEvent>,
        reply: oneshot::Sender<Result<SyncOutcome>>,
    },
    Rewind {
        coin: Coin,
        account: u32,
        rewind: Rewind,
        reply: oneshot::Sender<Result<()>>,
    },
    Apply {
        coin: Coin,
        apply: Apply,
        reply: oneshot::Sender<Result<()>>,
    },
}

/// Handle to the driver task. Every clone of the wallet handle holds one;
/// the task ends once the last is dropped and its request in flight, if
/// any, has finished.
#[derive(Clone)]
pub struct SyncDriver {
    requests: mpsc::Sender<Request>,
}

impl SyncDriver {
    /// Spawn the driver. Needs a tokio runtime.
    pub fn start() -> Self {
        let (requests, inbox) = mpsc::channel(16);
        tokio::spawn(run(inbox, WalletOps));
        Self { requests }
    }

    /// Sync `coin` as `request` says, after whatever the driver is doing,
    /// with progress to `progress`; returns how the sync ended.
    pub async fn sync(
        &self,
        coin: &Coin,
        request: SyncRequest,
        progress: mpsc::Sender<SyncEvent>,
    ) -> Result<SyncOutcome> {
        let (reply, result) = oneshot::channel();
        self.send(Request::Sync {
            coin: coin.clone(),
            request,
            progress,
            reply,
        })
        .await?;
        Self::answer(result).await
    }

    /// Rewind `account` of `coin`, cancelling a sync in flight first.
    pub async fn rewind(&self, coin: &Coin, account: u32, rewind: Rewind) -> Result<()> {
        let (reply, result) = oneshot::channel();
        self.send(Request::Rewind {
            coin: coin.clone(),
            account,
            rewind,
            reply,
        })
        .await?;
        Self::answer(result).await
    }

    /// Apply a prepared mutation to `coin` after whatever the driver is doing.
    pub async fn apply(&self, coin: &Coin, apply: Apply) -> Result<()> {
        let (reply, result) = oneshot::channel();
        self.send(Request::Apply {
            coin: coin.clone(),
            apply,
            reply,
        })
        .await?;
        Self::answer(result).await
    }

    async fn send(&self, request: Request) -> Result<()> {
        self.requests
            .send(request)
            .await
            .map_err(|_| anyhow!("the sync driver has stopped"))
    }

    async fn answer<T>(result: oneshot::Receiver<Result<T>>) -> Result<T> {
        result
            .await
            .map_err(|_| anyhow!("the sync driver dropped the request"))?
    }
}

/// What the driver does with each request; a trait so the loop can be
/// tested without a wallet or a chain.
trait Ops {
    fn sync(
        &self,
        coin: Coin,
        request: SyncRequest,
        progress: mpsc::Sender<SyncEvent>,
    ) -> impl Future<Output = Result<SyncOutcome>> + Send;
    fn rewind(
        &self,
        coin: Coin,
        account: u32,
        rewind: Rewind,
    ) -> impl Future<Output = Result<()>> + Send;
    fn apply(&self, coin: Coin, apply: Apply) -> impl Future<Output = Result<()>> + Send;
    /// Ask `coin`'s sync in flight to stop.
    fn cancel_sync(&self, coin: &Coin);
}

/// The driver loop. A sync runs while the inbox is watched: a rewind that
/// arrives meanwhile cancels it, and every arrival is kept, in order, for
/// after it.
async fn run<O: Ops>(mut inbox: mpsc::Receiver<Request>, ops: O) {
    let mut pending = VecDeque::new();
    let mut closed = false;
    loop {
        let request = match pending.pop_front() {
            Some(request) => request,
            None if closed => return,
            None => match inbox.recv().await {
                Some(request) => request,
                None => return, // every handle dropped
            },
        };
        match request {
            Request::Sync {
                coin,
                request,
                progress,
                reply,
            } => {
                let sync = ops.sync(coin.clone(), request, progress);
                tokio::pin!(sync);
                let result = loop {
                    tokio::select! {
                        result = &mut sync => break result,
                        next = inbox.recv(), if !closed => match next {
                            Some(next) => {
                                if matches!(next, Request::Rewind { .. }) {
                                    info!("sync driver: a rewind is waiting; cancelling the sync");
                                    ops.cancel_sync(&coin);
                                }
                                pending.push_back(next);
                            }
                            None => closed = true,
                        },
                    }
                };
                if let Err(e) = &result {
                    error!("sync driver: sync failed: {e:#}");
                }
                let _ = reply.send(result);
            }
            Request::Rewind {
                coin,
                account,
                rewind,
                reply,
            } => {
                info!("sync driver: rewinding account {account} {rewind:?}");
                let result = ops.rewind(coin, account, rewind).await;
                if let Err(e) = &result {
                    error!("sync driver: rewind failed: {e:#}");
                }
                let _ = reply.send(result);
            }
            Request::Apply { coin, apply, reply } => {
                let result = ops.apply(coin, apply).await;
                if let Err(e) = &result {
                    error!("sync driver: apply failed: {e:#}");
                }
                let _ = reply.send(result);
            }
        }
    }
}

/// The real operations, each under the wallet's session.
struct WalletOps;

impl Ops for WalletOps {
    async fn sync(
        &self,
        coin: Coin,
        request: SyncRequest,
        progress: mpsc::Sender<SyncEvent>,
    ) -> Result<SyncOutcome> {
        let c = &coin;
        let mut run = c.sync.begin().await;
        super::synchronize_within(
            &mut run,
            progress,
            request.accounts,
            request.current_height,
            request.actions_per_sync,
            request.transparent_limit,
            request.checkpoint_age,
            request.skip_details,
            c,
        )
        .await
    }

    async fn rewind(&self, coin: Coin, account: u32, rewind: Rewind) -> Result<()> {
        let c = &coin;
        let run = c.sync.begin().await;
        let mut connection = c.get_connection().await?;
        let mut held = run.held(&mut connection);
        match rewind {
            Rewind::To(height) => {
                super::rewind_sync(&c.network(), &mut held, account, height).await
            }
            Rewind::Reset => crate::account::reset_sync(&c.network(), &mut held, account).await,
        }
    }

    async fn apply(&self, coin: Coin, apply: Apply) -> Result<()> {
        let c = &coin;
        let run = c.sync.begin().await;
        let mut connection = c.get_connection().await?;
        let mut held = run.held(&mut connection);
        match apply {
            Apply::TxDetails { account, txs } => {
                for tx in &txs {
                    crate::memo::apply_tx_details(&mut held, account, tx).await?;
                }
                Ok(())
            }
            Apply::UserMemo {
                account,
                txid,
                memo,
            } => crate::db::set_user_memo(&mut held, account, &txid, memo).await,
        }
    }

    fn cancel_sync(&self, coin: &Coin) {
        coin.sync.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    };
    use std::time::Duration;
    use tokio::sync::Notify;

    /// Operations that record what ran, with syncs that block until released.
    #[derive(Default)]
    struct Script {
        log: Mutex<Vec<String>>,
        release: Notify,
        cancels: AtomicU32,
    }

    impl Ops for Arc<Script> {
        async fn sync(
            &self,
            _coin: Coin,
            request: SyncRequest,
            _progress: mpsc::Sender<SyncEvent>,
        ) -> Result<SyncOutcome> {
            self.log
                .lock()
                .unwrap()
                .push(format!("sync {}", request.current_height));
            self.release.notified().await;
            Ok(SyncOutcome::Synced {
                height: request.current_height,
            })
        }

        async fn rewind(&self, _coin: Coin, account: u32, rewind: Rewind) -> Result<()> {
            self.log
                .lock()
                .unwrap()
                .push(format!("rewind {account} {rewind:?}"));
            Ok(())
        }

        async fn apply(&self, _coin: Coin, apply: Apply) -> Result<()> {
            let what = match apply {
                Apply::TxDetails { account, txs } => format!("details {account} {}", txs.len()),
                Apply::UserMemo { account, txid, .. } => {
                    format!("memo {account} {}", hex::encode(txid))
                }
            };
            self.log.lock().unwrap().push(what);
            Ok(())
        }

        fn cancel_sync(&self, _coin: &Coin) {
            self.cancels.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn driver(script: Arc<Script>) -> SyncDriver {
        let (requests, inbox) = mpsc::channel(16);
        tokio::spawn(run(inbox, script));
        SyncDriver { requests }
    }

    fn coin() -> Coin {
        Coin::new(None)
    }

    fn request(height: u32) -> SyncRequest {
        SyncRequest {
            accounts: vec![1],
            current_height: height,
            actions_per_sync: 1,
            transparent_limit: 1,
            checkpoint_age: 1,
            skip_details: true,
        }
    }

    async fn settle() {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    fn log(script: &Script) -> Vec<String> {
        script.log.lock().unwrap().clone()
    }

    #[tokio::test]
    async fn a_rewind_cancels_the_sync_in_flight_and_runs_after_it() {
        let script = Arc::new(Script::default());
        let driver = driver(script.clone());

        let sync = tokio::spawn({
            let driver = driver.clone();
            async move { driver.sync(&coin(), request(100), mpsc::channel(1).0).await }
        });
        settle().await;
        assert_eq!(log(&script), ["sync 100"]);

        let rewind = tokio::spawn({
            let driver = driver.clone();
            async move { driver.rewind(&coin(), 1, Rewind::To(50)).await }
        });
        settle().await;
        assert_eq!(
            script.cancels.load(Ordering::SeqCst),
            1,
            "the sync is told to stop"
        );
        assert_eq!(log(&script), ["sync 100"], "the rewind waits for it");

        script.release.notify_one();
        sync.await.unwrap().unwrap();
        rewind.await.unwrap().unwrap();
        assert_eq!(log(&script), ["sync 100", "rewind 1 To(50)"]);
    }

    #[tokio::test]
    async fn requests_during_a_sync_run_after_it_in_order() {
        let script = Arc::new(Script::default());
        let driver = driver(script.clone());

        let first = tokio::spawn({
            let driver = driver.clone();
            async move { driver.sync(&coin(), request(100), mpsc::channel(1).0).await }
        });
        settle().await;

        let apply = tokio::spawn({
            let driver = driver.clone();
            async move {
                driver
                    .apply(
                        &coin(),
                        Apply::UserMemo {
                            account: 1,
                            txid: vec![7],
                            memo: None,
                        },
                    )
                    .await
            }
        });
        let second = tokio::spawn({
            let driver = driver.clone();
            async move { driver.sync(&coin(), request(101), mpsc::channel(1).0).await }
        });
        settle().await;
        assert_eq!(log(&script), ["sync 100"], "nothing overlaps the sync");
        assert_eq!(
            script.cancels.load(Ordering::SeqCst),
            0,
            "only a rewind cancels"
        );

        script.release.notify_one();
        first.await.unwrap().unwrap();
        apply.await.unwrap().unwrap();
        settle().await;
        assert_eq!(log(&script), ["sync 100", "memo 1 07", "sync 101"]);
        script.release.notify_one();
        assert_eq!(
            second.await.unwrap().unwrap(),
            SyncOutcome::Synced { height: 101 }
        );
    }

    #[tokio::test]
    async fn dropping_every_handle_ends_the_driver_after_its_work() {
        let script = Arc::new(Script::default());
        let driver = driver(script.clone());
        let sync = tokio::spawn({
            let driver = driver.clone();
            async move { driver.sync(&coin(), request(100), mpsc::channel(1).0).await }
        });
        settle().await;
        let apply = tokio::spawn({
            let driver = driver.clone();
            async move {
                driver
                    .apply(
                        &coin(),
                        Apply::TxDetails {
                            account: 1,
                            txs: vec![],
                        },
                    )
                    .await
            }
        });
        settle().await;
        drop(driver);

        script.release.notify_one();
        sync.await.unwrap().unwrap();
        apply.await.unwrap().unwrap();
        assert_eq!(log(&script), ["sync 100", "details 1 0"]);
    }
}
