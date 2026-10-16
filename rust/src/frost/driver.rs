//! The protocol driver: the one task that advances a DKG or a signing
//! session in a server process.
//!
//! A pass is an observe/plan/execute step over whichever protocol is in
//! progress. Running two passes at once is unsafe: each observes the wallet
//! before the other commits, both plan the same round, and both spend the
//! same funding note. So every reason to step goes through this task's
//! mailbox and passes run one after another:
//!
//! - the driver polls the chain tip itself and runs a pass when the height
//!   changes, so it does not depend on the mempool task, which is optional;
//! - a client asking for a step (`doDkg`, `frostSign`) sends a request and
//!   awaits the pass's result.
//!
//! Requests that arrive while a pass runs are coalesced: they all wait on
//! the single pass that follows, which observes the state the running pass
//! committed. The Flutter app has its own driver with the same guard.

use std::{future::Future, time::Duration};

use anyhow::{anyhow, Result};
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};

use crate::{
    api::{coin::Coin, frost::get_funding_account},
    frost::{dkg::in_dkg, sign::in_sign},
};

/// A reason to run a pass. `reply` is set when the sender wants the result.
struct StepRequest {
    reply: Option<oneshot::Sender<Result<()>>>,
}

/// Handle to the driver task; cloned into every request context.
#[derive(Clone)]
pub struct Driver {
    requests: mpsc::Sender<StepRequest>,
}

impl Driver {
    /// Spawn the driver for `coin`, polling the chain tip every `poll`.
    /// Needs a tokio runtime.
    pub fn start(coin: Coin, poll: Duration) -> Self {
        let (requests, inbox) = mpsc::channel(16);
        let tip = {
            let coin = coin.clone();
            move || {
                let coin = coin.clone();
                async move { coin.client().await?.latest_height().await }
            }
        };
        let pass = move || {
            let coin = coin.clone();
            async move { pass(&coin).await }
        };
        tokio::spawn(run(inbox, poll, tip, pass));
        Self { requests }
    }

    /// Run a pass (after any pass in flight) and return its result.
    pub async fn step(&self) -> Result<()> {
        let (reply, result) = oneshot::channel();
        self.requests
            .send(StepRequest { reply: Some(reply) })
            .await
            .map_err(|_| anyhow!("the protocol driver has stopped"))?;
        result
            .await
            .map_err(|_| anyhow!("the protocol driver dropped the request"))?
    }
}

/// The driver loop. `tip` reads the chain height; `pass` runs one step.
/// Generic so the queueing can be tested without a wallet or a chain.
async fn run<T, TF, P, PF>(mut inbox: mpsc::Receiver<StepRequest>, poll: Duration, tip: T, pass: P)
where
    T: Fn() -> TF,
    TF: Future<Output = Result<u32>>,
    P: Fn() -> PF,
    PF: Future<Output = Result<()>>,
{
    let mut ticks = tokio::time::interval(poll);
    ticks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut last_height = None;

    loop {
        // Wait for a reason to step: a request, or a tick that finds a new
        // block. A tick that sees the same height is not a reason.
        let mut waiters = vec![];
        tokio::select! {
            request = inbox.recv() => match request {
                Some(request) => waiters.extend(request.reply),
                None => return, // every handle dropped
            },
            _ = ticks.tick() => match tip().await {
                Ok(height) if Some(height) != last_height => {
                    last_height = Some(height);
                    info!("protocol driver: new block {height}");
                }
                Ok(_) => continue,
                Err(e) => {
                    error!("protocol driver: cannot read the chain tip: {e:#}");
                    continue;
                }
            },
        }

        // Coalesce whatever queued up meanwhile into this one pass.
        while let Ok(request) = inbox.try_recv() {
            waiters.extend(request.reply);
        }

        let result = pass().await;
        if let Err(e) = &result {
            error!("protocol driver: pass failed: {e:#}");
        }
        for waiter in waiters {
            let _ = waiter.send(result.as_ref().map(|_| ()).map_err(|e| anyhow!("{e:#}")));
        }
    }
}

/// One pass over whichever protocol is in progress; nothing when none is.
async fn pass(coin: &Coin) -> Result<()> {
    let mut connection = coin.get_connection().await?;
    let in_dkg = in_dkg(&mut connection).await?;
    let in_sign = in_sign(&mut connection).await?;
    if !in_dkg && !in_sign {
        return Ok(());
    }

    let mut client = coin.client().await?;
    let height = client.latest_height().await?;
    let account = get_funding_account(&mut connection).await?;
    info!("protocol driver: pass at {height}, funding account {account}");
    let height =
        crate::api::frost::sync_frost_accounts(coin, &mut connection, account, height).await?;

    if in_dkg {
        crate::frost::dkg::step::dkg_step(
            &coin.network(),
            &mut connection,
            &mut client,
            height,
            account,
        )
        .await?;
    }
    if in_sign {
        crate::frost::sign::do_sign_impl(&coin.network(), &mut connection, &mut client, height, ())
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };
    use tokio::sync::Notify;

    /// A pass that counts its runs and blocks until released, so requests
    /// can pile up behind it.
    struct Gate {
        runs: AtomicU32,
        release: Notify,
    }

    fn driver(gate: Arc<Gate>, heights: Arc<AtomicU32>, poll: Duration) -> Driver {
        let (requests, inbox) = mpsc::channel(16);
        let tip = move || {
            let heights = heights.clone();
            async move { Ok(heights.load(Ordering::SeqCst)) }
        };
        let pass = move || {
            let gate = gate.clone();
            async move {
                gate.runs.fetch_add(1, Ordering::SeqCst);
                gate.release.notified().await;
                Ok(())
            }
        };
        tokio::spawn(run(inbox, poll, tip, pass));
        Driver { requests }
    }

    async fn settle() {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn requests_during_a_pass_share_one_following_pass() {
        let gate = Arc::new(Gate {
            runs: AtomicU32::new(0),
            release: Notify::new(),
        });
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
        let gate = Arc::new(Gate {
            runs: AtomicU32::new(0),
            release: Notify::new(),
        });
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
}
