//! One DKG pass, without waiting.
//!
//! The shared entry point behind both front ends: `api::frost` exposes it to
//! Flutter and the GraphQL server calls it directly. A pass executes every
//! effect the planner names until it hits a wait or the protocol completes —
//! the same multi-round advance per invocation the sequential orchestrator
//! performed, one precondition-checked effect at a time. The caller syncs
//! first if fresh memo data matters; like [`crate::migrate::step::step_once`],
//! the step itself never synchronizes.

use anyhow::{bail, Context, Result};
use sqlx::SqliteConnection;

use crate::{api::coin::Network, Client};

use super::{
    exec,
    plan::next_task,
    state::observe,
    task::{DkgTask, TaskKind},
};

/// What one pass did. `executed` ends with the wait/terminal task the pass
/// stopped on — planned but not executed — so a front end can show where
/// things stand without re-observing.
pub struct DkgStepOutcome {
    pub executed: Vec<DkgTask>,
    /// The shared address, once the frost account exists.
    pub shared_address: Option<String>,
    /// A publish could not be funded yet: the note we spent for the previous
    /// round is locked and its change is not mined, so there is nothing to
    /// spend. Transient — the staged `pending_publish` is kept and the caller
    /// retries on the next block. Surfaced as a warning, never an error.
    pub waiting_for_funds: bool,
}

/// Run one pass for `account`.
///
/// The account is explicit rather than read from `c.account`: a caller holding
/// a process-wide coin — the GraphQL server does — must step the wallet it was
/// asked about, not whichever one the coin happens to name.
pub async fn dkg_step(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    height: u32,
    account: u32,
) -> Result<DkgStepOutcome> {
    let mut executed = Vec::new();
    let mut shared_address = None;

    // Effects per pass are bounded (three publishes plus three finalize
    // stages); the cap turns a would-be hot loop into an error.
    for _ in 0..16 {
        let state = observe(network, connection, account).await?;
        let task = next_task(&state);

        if !matches!(task.kind(), TaskKind::Effect) {
            executed.push(task);
            return Ok(DkgStepOutcome {
                executed,
                shared_address,
                waiting_for_funds: false,
            });
        }

        // Re-validate before touching anything. The snapshot the task was
        // planned from is an observe old, and a concurrent step — the app and
        // the GraphQL server can drive the same wallet — may have moved the
        // protocol underneath it. A stale task is re-planned, never executed.
        let fresh = observe(network, connection, account).await?;
        if !task.is_satisfied_by(&fresh) {
            tracing::info!("DKG task {task:?} no longer applies; replanning");
            continue;
        }

        match task {
            DkgTask::EnsureAccounts => {
                exec::ensure_accounts(network, connection, account, &fresh.params).await?;
            }
            DkgTask::PublishRound { round } => {
                match exec::publish_round(
                    network, connection, client, account, height, round, &fresh,
                )
                .await
                {
                    Ok(()) => {}
                    // No spendable note yet: the previous round's change is not
                    // mined. `stage()` already committed the secret and the
                    // pending bytes, so bail out of the pass as a wait — the
                    // caller retries on the next block and the resend goes
                    // through once the change confirms.
                    Err(e) if crate::pay::plan::is_no_feasible_selection(&e) => {
                        tracing::warn!(
                            "DKG round {round} publish deferred: no spendable note yet; \
                             will retry on the next block"
                        );
                        return Ok(DkgStepOutcome {
                            executed,
                            shared_address,
                            waiting_for_funds: true,
                        });
                    }
                    Err(e) => return Err(e),
                }
            }
            DkgTask::FinalizeKey => {
                exec::finalize_key(connection, account, &fresh).await?;
            }
            DkgTask::CreateFrostAccount => {
                let broadcast = fresh.broadcast.context("broadcast account missing")?;
                shared_address = Some(
                    exec::create_frost_account(network, connection, account, broadcast, height)
                        .await?,
                );
            }
            DkgTask::CompleteFinalize => {
                let frost_account = fresh.frost_account.context("frost account missing")?;
                exec::complete_finalize(
                    connection,
                    account,
                    frost_account,
                    fresh.mailbox,
                    fresh.broadcast,
                )
                .await?;
                // The rekey moved the protocol rows and the cleanup deleted
                // the props, so this is the pass's last act: report the shared
                // address from the frost account's stored group key.
                if shared_address.is_none() {
                    shared_address =
                        Some(exec::shared_address(network, connection, frost_account).await?);
                }
                executed.push(task);
                return Ok(DkgStepOutcome {
                    executed,
                    shared_address,
                    waiting_for_funds: false,
                });
            }
            DkgTask::WaitAddresses | DkgTask::WaitRound { .. } | DkgTask::Done => {
                unreachable!("non-effect task handled above")
            }
        }
        executed.push(task);
    }
    bail!("DKG pass executed too many effects without reaching a wait; aborting")
}
