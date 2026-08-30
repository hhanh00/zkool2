//! The unit of DKG work.
//!
//! A task is plain data: it names an action without performing it, so the
//! same value can be planned, displayed, re-validated against fresh state,
//! and only then executed. This is the discipline the note migration
//! ([`crate::migrate::task`]) established for a single wallet, extended to a
//! multi-party protocol where progress depends on other participants'
//! packages arriving.

use super::state::DkgState;

/// What kind of work a task represents. A driver uses this to decide how to
/// run it: a wait ends the pass, an effect carries a precondition that is
/// re-checked against fresh state before it runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskKind {
    /// Nothing to do but wait — the pass stops here.
    Time,
    /// Touches the wallet database or the chain.
    Effect,
    /// Nothing left to do.
    Terminal,
}

/// One step of the DKG.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DkgTask {
    /// Not all n participant addresses are known yet.
    WaitAddresses,
    /// Create the private mailbox and shared broadcast accounts if missing.
    EnsureAccounts,
    /// Publish our outgoing package for `round` (0..3): produce it if our
    /// secret is not stored yet, then send whatever bytes are staged.
    PublishRound { round: u8 },
    /// Peer packages for `round` are still incoming. The all-n discipline:
    /// every participant's package must arrive before the round advances, so
    /// each wallet materializes the same complete key set.
    WaitRound { round: u8 },
    /// dkg::part3 — derive our key package and the group public key package.
    FinalizeKey,
    /// Create the shared frost account holding the group spending key.
    CreateFrostAccount,
    /// Rekey the dkg_* rows onto the frost account and tear down the helper
    /// accounts. The last effect of the protocol.
    CompleteFinalize,
    /// No further progress is possible.
    Done,
}

impl DkgTask {
    pub fn kind(&self) -> TaskKind {
        match self {
            DkgTask::WaitAddresses | DkgTask::WaitRound { .. } => TaskKind::Time,
            DkgTask::EnsureAccounts
            | DkgTask::PublishRound { .. }
            | DkgTask::FinalizeKey
            | DkgTask::CreateFrostAccount
            | DkgTask::CompleteFinalize => TaskKind::Effect,
            DkgTask::Done => TaskKind::Terminal,
        }
    }

    /// Whether this task is still valid against freshly observed state.
    ///
    /// A task is planned from one snapshot and executed against another: a
    /// sync lands, a peer's package arrives, another driver — the app and the
    /// GraphQL server can drive the same wallet — advanced the protocol.
    /// Re-checking here turns a stale plan into a re-plan rather than a
    /// repeated publish.
    pub fn is_satisfied_by(&self, s: &DkgState) -> bool {
        match self {
            DkgTask::WaitAddresses | DkgTask::WaitRound { .. } | DkgTask::Done => true,
            DkgTask::EnsureAccounts => s.mailbox.is_none() || s.broadcast.is_none(),
            DkgTask::PublishRound { round } => {
                let rs = &s.rounds[*round as usize];
                !rs.secret_present || rs.pending == Some(*round)
            }
            DkgTask::FinalizeKey => !s.key_pkg_present,
            DkgTask::CreateFrostAccount => s.frost_account.is_none(),
            DkgTask::CompleteFinalize => true,
        }
    }
}
