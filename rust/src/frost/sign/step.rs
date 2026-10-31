//! One signing pass, without waiting.
//!
//! Mirrors the DKG step: plan, double-observe, execute. A pass advances every
//! ready effect until it hits a time-task or a terminal. The caller syncs
//! first if fresh memo data matters; this never synchronizes itself.

use anyhow::{bail, Result};
use sqlx::SqliteConnection;
use tracing::info;

use crate::{api::coin::Network, frost::dkg::delete_frost_state, Client};

use super::{
    exec::{self, SendResult},
    plan::next_sign_task,
    state::observe_sign,
    task::{SignTask, TaskKind},
};

/// What one signing pass did.
pub struct SignStepOutcome {
    /// Tasks executed this pass (last entry is the wait/terminal that stopped it).
    pub executed: Vec<SignTask>,
    /// A publish could not be funded; the pass stopped early. The caller
    /// retries on the next block.
    pub waiting_for_funds: bool,
    /// The transaction id, once aggregation completed.
    pub txid: Option<String>,
    /// When the pass stopped waiting on the others: what is awaited, how
    /// many are in, how many are needed.
    pub progress: Option<(String, u8, u8)>,
}

/// Run one signing pass for the account stored in `frost_sign_params`.
pub async fn sign_step(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    height: u32,
    stopping: &(dyn Fn() -> bool + Send + Sync),
) -> Result<Option<SignStepOutcome>> {
    let Some(_state) = observe_sign(network, connection).await? else {
        return Ok(None); // no signing session in progress
    };

    let mut executed = Vec::new();

    // Effects per pass are bounded; the cap turns any loop into an error.
    for _ in 0..16 {
        // The session can be reset or cancelled between observes; that ends
        // the pass quietly, like a pass that found no session at all.
        let Some(state) = observe_sign(network, connection).await? else {
            return Ok(None);
        };
        let task = match next_sign_task(&state) {
            Ok(task) => task,
            Err(e) => {
                // The protocol must abort (participants disagreed on the
                // signing set). Clear the session so the error is reported
                // once rather than on every later pass.
                delete_frost_state(connection).await?;
                return Err(e);
            }
        };

        // Our own unmined message is resent while it is waited on, and
        // given up past its expiry, in which case the pass replans.
        if task == SignTask::WaitOwnPublish
            && exec::own_publish(connection, client, &state, height).await? == exec::OwnPublish::Expired
        {
            info!("sign: the last message expired unmined; replanning");
            continue;
        }

        if !matches!(task.kind(), TaskKind::Effect) {
            let least = |counts: &[usize]| counts.iter().copied().min().unwrap_or(0) as u8;
            let progress = match task {
                SignTask::WaitCommitments => {
                    Some(("Commitments".to_string(), least(&state.commitment_counts), state.t))
                }
                SignTask::WaitSigpackages => Some((
                    "Signing package parts".to_string(),
                    state.sigpackages_count as u8,
                    state.nsigs as u8,
                )),
                SignTask::WaitSigshares => {
                    Some(("Signature shares".to_string(), least(&state.sigshare_counts), state.t))
                }
                _ => None,
            };
            executed.push(task);
            return Ok(Some(SignStepOutcome {
                executed,
                waiting_for_funds: false,
                txid: None,
                progress,
            }));
        }

        // Double-observe: re-validate before touching anything.
        let Some(fresh) = observe_sign(network, connection).await? else {
            return Ok(None);
        };
        if !task.is_satisfied_by(&fresh) {
            info!("sign task {task:?} no longer applies; replanning");
            continue;
        }
        // A cancel is waiting for this pass: it ends here, quietly, like a
        // session reset between observes.
        if stopping() {
            info!("sign: cancelled; stopping before {task:?}");
            return Ok(None);
        }

        match task {
            SignTask::EnsureMailbox => {
                exec::ensure_mailbox(network, connection, &fresh, height).await?;
            }

            SignTask::AnnounceCoordinator => {
                match exec::announce_coordinator(network, connection, client, &fresh).await? {
                    SendResult::Sent => {}
                    SendResult::WaitingForFunds => {
                        return Ok(Some(SignStepOutcome {
                            executed,
                            waiting_for_funds: true,
                            txid: None,
                        progress: None,
                        }));
                    }
                }
            }

            SignTask::SendCommitments => {
                match exec::send_commitments(network, connection, client, &fresh).await? {
                    SendResult::Sent => {}
                    SendResult::WaitingForFunds => {
                        return Ok(Some(SignStepOutcome {
                            executed,
                            waiting_for_funds: true,
                            txid: None,
                        progress: None,
                        }));
                    }
                }
            }

            SignTask::MakeSigpackages => {
                match exec::make_sigpackages(network, connection, client, &fresh).await? {
                    SendResult::Sent => {}
                    SendResult::WaitingForFunds => {
                        return Ok(Some(SignStepOutcome {
                            executed,
                            waiting_for_funds: true,
                            txid: None,
                        progress: None,
                        }));
                    }
                }
            }

            SignTask::SendSigshares => {
                match exec::send_sigshares(network, connection, client, &fresh).await? {
                    SendResult::Sent => {}
                    SendResult::WaitingForFunds => {
                        return Ok(Some(SignStepOutcome {
                            executed,
                            waiting_for_funds: true,
                            txid: None,
                        progress: None,
                        }));
                    }
                }
            }

            SignTask::Aggregate => {
                let txid =
                    exec::aggregate_and_send(network, connection, client, &fresh).await?;
                executed.push(task);
                return Ok(Some(SignStepOutcome {
                    executed,
                    waiting_for_funds: false,
                    txid: Some(txid),
                progress: None,
                }));
            }

            SignTask::Done => {
                // The signer's part is over: without this, the session rows
                // would outlive the page, and the account's send button would
                // keep leading back into a finished session.
                delete_frost_state(connection).await?;
                executed.push(task);
                return Ok(Some(SignStepOutcome {
                    executed,
                    waiting_for_funds: false,
                    txid: None,
                    progress: None,
                }));
            }

            SignTask::WaitCoordinator
            | SignTask::WaitOwnPublish
            | SignTask::WaitCommitments
            | SignTask::WaitSigpackages
            | SignTask::WaitSigshares => {
                unreachable!("non-effect task handled above")
            }
        }
        executed.push(task);
    }
    bail!("sign pass executed too many effects without reaching a wait; aborting")
}
