//! One migration step, without waiting.
//!
//! The shared entry point behind both front ends: `api::migrate` exposes it to
//! Flutter and the GraphQL server calls it directly. Neither the pacing delay
//! nor the boundary wait belongs here — a caller polls this and does its own
//! waiting between calls, which is what makes it safe to drive from a timer.

use anyhow::Result;

use crate::api::coin::Coin;

use super::{
    exec,
    plan::next_task,
    task::{MigrationTask, Pacing},
};

/// What one step did.
pub enum StepOutcome {
    Split {
        fee: u64,
    },
    Migrated {
        fee: u64,
    },
    /// Nothing left to migrate.
    Complete,
    /// The next action is a wait, which a single step does not perform.
    NothingToDo,
}

/// Run one step for `account`.
///
/// The account is explicit rather than read from `c.account`: a caller holding
/// a process-wide coin — the GraphQL server does — must step the wallet it was
/// asked about, not whichever one the coin happens to name.
///
/// Migration does not synchronize; this plans against the checkpoint the
/// wallet already has, so the caller syncs first if it wants a current one.
pub async fn step_once(c: &Coin, account: u32) -> Result<StepOutcome> {
    let c = &Coin {
        account,
        ..c.clone()
    };
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    let state = super::state::observe(
        &c.network(),
        &mut connection,
        &mut client,
        account,
        super::ANCHOR_BUCKET_SIZE,
    )
    .await?;
    drop(connection);

    // One step does one substantive thing: no pacing delay to serve.
    let pacing = Pacing {
        delay_served: true,
        ..Pacing::default()
    };

    Ok(match next_task(&state, &pacing) {
        MigrationTask::Done => StepOutcome::Complete,
        MigrationTask::Split { inputs, outputs } => StepOutcome::Split {
            fee: exec::split(c, &inputs, &outputs).await?,
        },
        MigrationTask::Migrate {
            note,
            amount,
            anchor,
        } => StepOutcome::Migrated {
            fee: exec::migrate(c, note, amount, anchor).await?,
        },
        // Waiting is the caller's business, not a single step's.
        MigrationTask::WaitDelay { .. } | MigrationTask::WaitBoundary { .. } => {
            StepOutcome::NothingToDo
        }
    })
}
