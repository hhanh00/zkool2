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
}

/// Run one signing pass for the account stored in `frost_sign_params`.
pub async fn sign_step(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    height: u32,
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

        if !matches!(task.kind(), TaskKind::Effect) {
            executed.push(task);
            return Ok(Some(SignStepOutcome {
                executed,
                waiting_for_funds: false,
                txid: None,
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
                }));
            }

            SignTask::WaitCoordinator
            | SignTask::WaitOwnPublish
            | SignTask::WaitCommitments
            | SignTask::WaitSigpackages
            | SignTask::WaitSigshares
            | SignTask::Done => {
                unreachable!("non-effect task handled above")
            }
        }
        executed.push(task);
    }
    bail!("sign pass executed too many effects without reaching a wait; aborting")
}
