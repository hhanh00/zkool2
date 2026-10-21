use super::state::SignState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskKind {
    Time,
    Effect,
    Terminal,
}

/// One step of the signing protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignTask {
    /// Private mailbox account does not exist yet.
    EnsureMailbox,
    /// Coordinator: broadcast our mailbox address for this session, so that
    /// members send to it without a stored address for us.
    AnnounceCoordinator,
    /// Non-coordinator: the coordinator's address for this session has not
    /// arrived yet.
    WaitCoordinator,
    /// Our previous message is not mined yet: the next one would need its
    /// change, so it waits for the block that mines it.
    WaitOwnPublish,
    /// Generate and send commitments to the coordinator (Phase 1).
    SendCommitments,
    /// Coordinator: not enough commitments have arrived yet.
    WaitCommitments,
    /// Coordinator: build and broadcast signing packages (Phase 2).
    MakeSigpackages,
    /// Non-coordinator: waiting for signing packages from the coordinator.
    WaitSigpackages,
    /// Generate and send signature shares to the coordinator (Phase 3).
    SendSigshares,
    /// Coordinator: not enough signature shares have arrived yet.
    WaitSigshares,
    /// Coordinator: aggregate shares and broadcast the final transaction (Phase 4).
    Aggregate,
    /// Non-coordinator: its signature share is mined, so the coordinator can
    /// read it; nothing is left to do here. Ends the session.
    Done,
}

impl SignTask {
    pub fn kind(&self) -> TaskKind {
        match self {
            SignTask::WaitCoordinator
            | SignTask::WaitOwnPublish
            | SignTask::WaitCommitments
            | SignTask::WaitSigpackages
            | SignTask::WaitSigshares => TaskKind::Time,
            SignTask::EnsureMailbox
            | SignTask::AnnounceCoordinator
            | SignTask::SendCommitments
            | SignTask::MakeSigpackages
            | SignTask::SendSigshares
            | SignTask::Aggregate
            | SignTask::Done => TaskKind::Effect,
        }
    }

    /// Whether this task is still applicable against freshly observed state.
    pub fn is_satisfied_by(&self, s: &SignState) -> bool {
        match self {
            SignTask::WaitCoordinator
            | SignTask::WaitOwnPublish
            | SignTask::WaitCommitments
            | SignTask::WaitSigpackages
            | SignTask::WaitSigshares => true,
            SignTask::Done => !s.is_coordinator() && s.own_sigshares_stored && !s.publish_unmined,
            SignTask::EnsureMailbox => s.mailbox.is_none() || s.broadcast.is_none(),
            SignTask::AnnounceCoordinator => s.is_coordinator() && s.coordinator_address.is_none(),
            SignTask::SendCommitments => !s.own_commitments_stored,
            SignTask::MakeSigpackages => {
                s.is_coordinator()
                    && s.sigpackages_count < s.nsigs
                    && s.commitment_counts.iter().all(|&c| c >= s.t as usize)
            }
            SignTask::SendSigshares => !s.own_sigshares_stored,
            SignTask::Aggregate => {
                s.is_coordinator()
                    && s.own_sigshares_stored
                    && s.sigshare_counts.iter().all(|&c| c >= s.t as usize)
            }
        }
    }
}
