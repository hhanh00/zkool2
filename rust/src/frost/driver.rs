//! The protocol driver: the one task that advances a DKG or a signing
//! session for a wallet.
//!
//! A pass is an observe/plan/execute step over whichever protocol is in
//! progress. Running two passes at once is unsafe: each observes the wallet
//! before the other commits, both plan the same round, and both spend the
//! same funding note. So every reason to step goes through this task's
//! mailbox and passes run one after another:
//!
//! - the driver polls the chain tip itself and runs a pass when the height
//!   changes, so it depends on no other task;
//! - a client asking for a step (the server's `doDkg`, `frostSign`) sends a
//!   request and awaits the pass's result;
//! - a cancel (the app's cancel buttons, the server's `dkgCancel`,
//!   `frostCancel`) is queued the same way and runs after the pass in
//!   flight, so it never deletes a session a pass is still working from.
//!   Meanwhile it raises a flag the pass checks before each step: the step
//!   under way finishes, a broadcast included, and no other starts.
//!
//! Requests that arrive while a pass runs are coalesced: they all wait on
//! the single pass that follows, which observes the state the running pass
//! committed. A pass takes the wallet's sync session for its whole duration:
//! it syncs the funding and frost accounts, then steps the protocol, still
//! under the session, so the step's writes never overlap a sync the app
//! starts meanwhile (SQLite serves one writer; the loser sees "database is
//! locked"). When the session is busy the pass is retried on the next tick
//! rather than waiting for another block.
//!
//! What a pass did is reported through a [`Sink`] of [`ProtocolStatus`]: the
//! app renders it, the headless server passes `()`. The same driver serves
//! both; the app runs it for as long as a protocol page is open.

use std::{
    future::Future,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::{anyhow, Result};
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};

use crate::{
    api::{
        coin::Coin,
        frost::{get_funding_account, sync_frost_accounts, ProtocolStatus, SigningStatus},
    },
    frost::{dkg::in_dkg, sign::in_sign},
    Sink,
};

/// Which session a cancel ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cancel {
    Sign,
    Dkg,
}

/// A reason to wake the driver.
enum Request {
    /// Run a pass. `reply` is set when the sender wants the result.
    Step {
        reply: Option<oneshot::Sender<Result<()>>>,
    },
    /// End a session, after the pass in flight.
    Cancel {
        kind: Cancel,
        reply: oneshot::Sender<Result<()>>,
    },
}

/// What a pass found: it ran, or it could not sync and wants another go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pass {
    Done,
    /// The wallet's sync session was busy; nothing was observed.
    SyncBusy,
}

/// Handle to the driver task. Dropping every handle ends the task once the
/// pass in flight, if any, has finished.
#[derive(Clone)]
pub struct Driver {
    requests: mpsc::Sender<Request>,
    /// Cancels sent and not yet run; a pass stops before its next step
    /// while there is one.
    cancelling: Arc<AtomicUsize>,
}

impl Driver {
    /// Spawn the driver for `coin`, polling the chain tip every `poll` and
    /// reporting passes to `status`. Needs a tokio runtime.
    pub fn start<S>(coin: Coin, poll: Duration, status: S) -> Self
    where
        S: Sink<ProtocolStatus> + Send + Sync + 'static,
    {
        let (requests, inbox) = mpsc::channel(16);
        let cancelling = Arc::new(AtomicUsize::new(0));
        let tip = {
            let coin = coin.clone();
            move || {
                let coin = coin.clone();
                async move { coin.client().await?.latest_height().await }
            }
        };
        let cancel = {
            let coin = coin.clone();
            move |kind| {
                let coin = coin.clone();
                async move { cancel_session(&coin, kind).await }
            }
        };
        let pass = {
            let cancelling = cancelling.clone();
            move || {
                let coin = coin.clone();
                let status = status.clone();
                let cancelling = cancelling.clone();
                async move {
                    let stopping = || cancelling.load(Ordering::SeqCst) > 0;
                    let result = pass(&coin, &status, &stopping).await;
                    // A failed pass is transient (the next block retries);
                    // the app shows it as a warning, the server logs it. It
                    // goes as a status value, not a stream error: a
                    // flutter_rust_bridge stream ends at its first error,
                    // which would leave the page frozen while the driver
                    // carries on.
                    if let Err(e) = &result {
                        status.send(ProtocolStatus::Warning(format!("{e:#}"))).await;
                    }
                    result
                }
            }
        };
        tokio::spawn(run(inbox, poll, tip, pass, cancel, cancelling.clone()));
        Self {
            requests,
            cancelling,
        }
    }

    /// Run a pass (after any pass in flight) and return its result.
    pub async fn step(&self) -> Result<()> {
        let (reply, result) = oneshot::channel();
        self.requests
            .send(Request::Step { reply: Some(reply) })
            .await
            .map_err(|_| anyhow!("the protocol driver has stopped"))?;
        result
            .await
            .map_err(|_| anyhow!("the protocol driver dropped the request"))?
    }

    /// End the `kind` session once the step in flight, if any, has finished.
    pub async fn cancel(&self, kind: Cancel) -> Result<()> {
        let (reply, result) = oneshot::channel();
        // Raised before sending, so the pass in flight sees it; the driver
        // lowers it once the cancel has run.
        self.cancelling.fetch_add(1, Ordering::SeqCst);
        if self
            .requests
            .send(Request::Cancel { kind, reply })
            .await
            .is_err()
        {
            self.cancelling.fetch_sub(1, Ordering::SeqCst);
            return Err(anyhow!("the protocol driver has stopped"));
        }
        result
            .await
            .map_err(|_| anyhow!("the protocol driver dropped the request"))?
    }
}

/// Delete the `kind` session. Only safe with no pass running: the driver
/// calls it between passes, and the app when it runs no driver.
pub(crate) async fn cancel_session(coin: &Coin, kind: Cancel) -> Result<()> {
    let mut connection = coin.get_connection().await?;
    match kind {
        Cancel::Sign => crate::frost::sign::reset_sign(&mut connection).await,
        Cancel::Dkg => {
            let account = get_funding_account(&mut connection).await?;
            crate::frost::dkg::cancel_dkg(&mut connection, account).await
        }
    }
}

/// The driver loop. `tip` reads the chain height; `pass` runs one step;
/// `cancel` ends a session, and lowers `cancelling` once it has.
/// Generic so the queueing can be tested without a wallet or a chain.
async fn run<T, TF, P, PF, C, CF>(
    mut inbox: mpsc::Receiver<Request>,
    poll: Duration,
    tip: T,
    pass: P,
    cancel: C,
    cancelling: Arc<AtomicUsize>,
) where
    T: Fn() -> TF,
    TF: Future<Output = Result<u32>>,
    P: Fn() -> PF,
    PF: Future<Output = Result<Pass>>,
    C: Fn(Cancel) -> CF,
    CF: Future<Output = Result<()>>,
{
    let mut ticks = tokio::time::interval(poll);
    ticks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut last_height = None;
    // Set when the last pass could not sync: the next tick steps again even
    // if no block arrived meanwhile.
    let mut retry = false;

    loop {
        // Wait for a reason to step: a request, a tick that finds a new
        // block, or a tick after a pass that asked to be retried.
        let mut requests = vec![];
        tokio::select! {
            request = inbox.recv() => match request {
                Some(request) => requests.push(request),
                None => return, // every handle dropped
            },
            _ = ticks.tick() => match tip().await {
                Ok(height) if Some(height) != last_height => {
                    last_height = Some(height);
                    info!("protocol driver: new block {height}");
                }
                Ok(_) if retry => info!("protocol driver: retrying after a busy sync"),
                Ok(_) => continue,
                Err(e) => {
                    error!("protocol driver: cannot read the chain tip: {e:#}");
                    continue;
                }
            },
        }

        // Coalesce whatever queued up meanwhile into this one pass.
        while let Ok(request) = inbox.try_recv() {
            requests.push(request);
        }
        let mut waiters = vec![];
        let mut cancels = vec![];
        for request in requests {
            match request {
                Request::Step { reply } => waiters.extend(reply),
                Request::Cancel { kind, reply } => cancels.push((kind, reply)),
            }
        }

        // A cancel runs instead of the pass: any pass after it would step a
        // session that is gone, so the steps queued with it are answered
        // without one.
        if !cancels.is_empty() {
            for (kind, reply) in cancels {
                info!("protocol driver: cancelling the {kind:?} session");
                let result = cancel(kind).await;
                cancelling.fetch_sub(1, Ordering::SeqCst);
                if let Err(e) = &result {
                    error!("protocol driver: cancel failed: {e:#}");
                }
                let _ = reply.send(result);
            }
            for waiter in waiters {
                let _ = waiter.send(Ok(()));
            }
            retry = false;
            continue;
        }

        let result = pass().await;
        retry = matches!(result, Ok(Pass::SyncBusy));
        if let Err(e) = &result {
            error!("protocol driver: pass failed: {e:#}");
        }
        for waiter in waiters {
            let _ = waiter.send(result.as_ref().map(|_| ()).map_err(|e| anyhow!("{e:#}")));
        }
    }
}

/// Forward a signing pass's statuses into the protocol stream.
#[derive(Clone)]
struct SignStatus<S>(S);

impl<S: Sink<ProtocolStatus> + Send + Sync> Sink<SigningStatus> for SignStatus<S> {
    async fn send(&self, value: SigningStatus) {
        self.0.send(ProtocolStatus::Sign(value)).await
    }
    async fn send_error(&self, error: anyhow::Error) {
        self.0.send_error(error).await
    }
}

/// One pass over whichever protocol is in progress; nothing when none is.
async fn pass<S>(
    coin: &Coin,
    status: &S,
    stopping: &(dyn Fn() -> bool + Send + Sync),
) -> Result<Pass>
where
    S: Sink<ProtocolStatus> + Send + Sync,
{
    let mut connection = coin.get_connection().await?;
    let in_dkg = in_dkg(&mut connection).await?;
    let in_sign = in_sign(&mut connection).await?;
    if !in_dkg && !in_sign {
        return Ok(Pass::Done);
    }

    let mut client = coin.client().await?;
    let height = client.latest_height().await?;
    let account = get_funding_account(&mut connection).await?;
    info!("protocol driver: pass at {height}, funding account {account}");
    // Held until the end of the pass, through the step.
    let Some(mut run) = coin.sync.try_begin() else {
        info!("protocol driver: sync busy; will retry");
        return Ok(Pass::SyncBusy);
    };
    let Some(height) =
        sync_frost_accounts(&mut run, coin, &mut connection, account, height).await?
    else {
        info!("protocol driver: sync cancelled; will retry");
        return Ok(Pass::SyncBusy);
    };

    if in_dkg {
        let outcome = crate::frost::dkg::step::dkg_step(
            &coin.network(),
            &mut connection,
            &mut client,
            height,
            account,
            stopping,
        )
        .await?;
        for s in outcome.statuses() {
            status.send(ProtocolStatus::Dkg(s)).await;
        }
    }
    if in_sign {
        crate::frost::sign::do_sign_impl(
            &coin.network(),
            &mut connection,
            &mut client,
            height,
            SignStatus(status.clone()),
            stopping,
        )
        .await?;
    }
    drop(run);
    Ok(Pass::Done)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    };
    use tokio::sync::Notify;

    /// A pass that counts its runs, blocks until released, and returns the
    /// next scripted outcome.
    struct Gate {
        runs: AtomicU32,
        release: Notify,
        outcomes: Mutex<Vec<Pass>>,
        /// The cancels run, in order.
        cancels: Mutex<Vec<Cancel>>,
    }

    fn gate() -> Arc<Gate> {
        Arc::new(Gate {
            runs: AtomicU32::new(0),
            release: Notify::new(),
            outcomes: Mutex::new(vec![]),
            cancels: Mutex::new(vec![]),
        })
    }

    fn driver(gate: Arc<Gate>, heights: Arc<AtomicU32>, poll: Duration) -> Driver {
        let (requests, inbox) = mpsc::channel(16);
        let cancelling = Arc::new(AtomicUsize::new(0));
        let cancel = {
            let gate = gate.clone();
            move |kind| {
                gate.cancels.lock().unwrap().push(kind);
                async { Ok(()) }
            }
        };
        let tip = move || {
            let heights = heights.clone();
            async move { Ok(heights.load(Ordering::SeqCst)) }
        };
        let pass = move || {
            let gate = gate.clone();
            async move {
                gate.runs.fetch_add(1, Ordering::SeqCst);
                gate.release.notified().await;
                let next = gate.outcomes.lock().unwrap().pop().unwrap_or(Pass::Done);
                Ok(next)
            }
        };
        tokio::spawn(run(inbox, poll, tip, pass, cancel, cancelling.clone()));
        Driver {
            requests,
            cancelling,
        }
    }

    fn cancelling(driver: &Driver) -> bool {
        driver.cancelling.load(Ordering::SeqCst) > 0
    }

    #[tokio::test]
    async fn a_cancel_waits_for_the_pass_in_flight_and_flags_it_meanwhile() {
        let gate = gate();
        let driver = driver(
            gate.clone(),
            Arc::new(AtomicU32::new(0)),
            Duration::from_secs(3600),
        );
        settle().await;
        assert_eq!(gate.runs.load(Ordering::SeqCst), 1, "startup pass running");

        let cancel = tokio::spawn({
            let driver = driver.clone();
            async move { driver.cancel(Cancel::Sign).await }
        });
        settle().await;
        assert!(cancelling(&driver), "the pass sees the cancel coming");
        assert!(gate.cancels.lock().unwrap().is_empty(), "not while a pass runs");

        gate.release.notify_one();
        cancel.await.unwrap().unwrap();
        assert_eq!(*gate.cancels.lock().unwrap(), [Cancel::Sign]);
        assert!(!cancelling(&driver), "lowered once the cancel has run");
        settle().await;
        assert_eq!(gate.runs.load(Ordering::SeqCst), 1, "no pass after it");
    }

    #[tokio::test]
    async fn steps_queued_with_a_cancel_are_answered_without_a_pass() {
        let gate = gate();
        let driver = driver(
            gate.clone(),
            Arc::new(AtomicU32::new(0)),
            Duration::from_secs(3600),
        );
        settle().await;
        assert_eq!(gate.runs.load(Ordering::SeqCst), 1, "startup pass running");

        let step = tokio::spawn({
            let driver = driver.clone();
            async move { driver.step().await }
        });
        let cancel = tokio::spawn({
            let driver = driver.clone();
            async move { driver.cancel(Cancel::Dkg).await }
        });
        settle().await;

        gate.release.notify_one();
        cancel.await.unwrap().unwrap();
        step.await.unwrap().unwrap();
        assert_eq!(*gate.cancels.lock().unwrap(), [Cancel::Dkg]);
        settle().await;
        assert_eq!(gate.runs.load(Ordering::SeqCst), 1, "the pass was skipped");
    }

    async fn settle() {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn requests_during_a_pass_share_one_following_pass() {
        let gate = gate();
        let driver = driver(
            gate.clone(),
            Arc::new(AtomicU32::new(0)),
            Duration::from_secs(3600),
        );

        // The first tick after start is a new block: a session interrupted
        // by a restart resumes without waiting for a request.
        settle().await;
        assert_eq!(gate.runs.load(Ordering::SeqCst), 1, "startup pass");
        gate.release.notify_one();
        settle().await;

        let first = tokio::spawn({
            let driver = driver.clone();
            async move { driver.step().await }
        });
        settle().await;
        assert_eq!(
            gate.runs.load(Ordering::SeqCst),
            2,
            "the request's pass is running"
        );

        // Three more requests arrive while it runs.
        let queued: Vec<_> = (0..3)
            .map(|_| {
                let driver = driver.clone();
                tokio::spawn(async move { driver.step().await })
            })
            .collect();
        settle().await;
        assert_eq!(
            gate.runs.load(Ordering::SeqCst),
            2,
            "nothing overlaps the running pass"
        );

        gate.release.notify_one();
        first.await.unwrap().unwrap();
        settle().await;
        assert_eq!(
            gate.runs.load(Ordering::SeqCst),
            3,
            "the queued requests share one pass"
        );

        gate.release.notify_one();
        for q in queued {
            q.await.unwrap().unwrap();
        }
        settle().await;
        assert_eq!(
            gate.runs.load(Ordering::SeqCst),
            3,
            "no pass runs for nothing"
        );
    }

    #[tokio::test]
    async fn a_new_block_runs_a_pass_and_the_same_block_does_not() {
        let gate = gate();
        let heights = Arc::new(AtomicU32::new(100));
        let _driver = driver(gate.clone(), heights.clone(), Duration::from_millis(10));

        settle().await;
        assert_eq!(
            gate.runs.load(Ordering::SeqCst),
            1,
            "the first tip seen is a new block"
        );
        gate.release.notify_one();
        settle().await;
        assert_eq!(
            gate.runs.load(Ordering::SeqCst),
            1,
            "ticks at the same height do nothing"
        );

        heights.store(101, Ordering::SeqCst);
        settle().await;
        assert_eq!(
            gate.runs.load(Ordering::SeqCst),
            2,
            "a new height runs a pass"
        );
        gate.release.notify_one();
    }

    #[tokio::test]
    async fn a_busy_sync_is_retried_on_the_next_tick_without_a_new_block() {
        let gate = gate();
        gate.outcomes.lock().unwrap().push(Pass::SyncBusy);
        let _driver = driver(
            gate.clone(),
            Arc::new(AtomicU32::new(100)),
            Duration::from_millis(10),
        );

        settle().await;
        assert_eq!(gate.runs.load(Ordering::SeqCst), 1, "startup pass");
        gate.release.notify_one(); // ends with SyncBusy
        settle().await;
        assert_eq!(
            gate.runs.load(Ordering::SeqCst),
            2,
            "retried at the same height"
        );
        gate.release.notify_one(); // ends with Done
        settle().await;
        assert_eq!(
            gate.runs.load(Ordering::SeqCst),
            2,
            "a completed pass stops the retries"
        );
    }

    #[tokio::test]
    async fn dropping_the_last_handle_stops_the_driver() {
        let gate = gate();
        let driver = driver(
            gate.clone(),
            Arc::new(AtomicU32::new(100)),
            Duration::from_millis(10),
        );
        settle().await;
        gate.release.notify_one();
        settle().await;
        let runs = gate.runs.load(Ordering::SeqCst);

        drop(driver);
        settle().await;
        gate.release.notify_one();
        settle().await;
        assert_eq!(
            gate.runs.load(Ordering::SeqCst),
            runs,
            "no pass after the handle is gone"
        );
    }
}
