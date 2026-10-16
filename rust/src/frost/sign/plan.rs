//! Pure decision logic for the signing pipeline.
//!
//! Nothing here performs I/O. Every function is a total function of its
//! arguments, so what signing decides to do next can be unit-tested without a
//! wallet, a chain, or other participants.

use anyhow::Result;

use super::state::SignState;
use super::task::SignTask;

/// Decide the signing protocol's next step.
///
/// Returns `Err` when the protocol must abort (e.g. too many commitments
/// received — participants failed to agree on the signing set out of band).
pub fn next_sign_task(s: &SignState) -> Result<SignTask> {
    // Both internal accounts are needed: the mailbox receives commitments
    // and shares, the broadcast account carries the signing packages.
    if s.mailbox.is_none() || s.broadcast.is_none() {
        return Ok(SignTask::EnsureMailbox);
    }

    // ── Phase 1: commitments ──────────────────────────────────────────────────
    if !s.own_commitments_stored {
        return Ok(SignTask::SendCommitments);
    }

    // ── Phase 2: signing packages ─────────────────────────────────────────────
    if s.sigpackages_count < s.nsigs {
        if s.is_coordinator() {
            for (idx, &count) in s.commitment_counts.iter().enumerate() {
                if count > s.t as usize {
                    anyhow::bail!(
                        "Signing aborted: received {} commitments for input {idx}, \
                         expected exactly {}. Participants must agree on the signing \
                         set out of band",
                        count,
                        s.t
                    );
                }
            }
            return if s.commitment_counts.iter().any(|&c| c < s.t as usize) {
                Ok(SignTask::WaitCommitments)
            } else {
                Ok(SignTask::MakeSigpackages)
            };
        } else {
            return Ok(SignTask::WaitSigpackages);
        }
    }

    // ── Phase 3: signature shares ─────────────────────────────────────────────
    if !s.own_sigshares_stored {
        return Ok(SignTask::SendSigshares);
    }

    // ── Phase 4: aggregation (coordinator only) ────────────────────────────────
    if !s.is_coordinator() {
        return Ok(SignTask::Done);
    }

    if s.sigshare_counts.iter().any(|&c| c < s.t as usize) {
        return Ok(SignTask::WaitSigshares);
    }
    Ok(SignTask::Aggregate)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(is_coord: bool) -> SignState {
        SignState {
            account: 1,
            funding_account: 2,
            self_id_u8: if is_coord { 1 } else { 2 },
            coordinator: 1,
            n: 3,
            t: 2,
            sighash: vec![0u8; 32],
            nsigs: 2,
            birth_height: 0,
            mailbox: Some(10),
            coordinator_address: Some("addr1".into()),
            broadcast: Some(11),
            broadcast_address: Some("addr2".into()),
            own_commitments_stored: false,
            commitment_counts: vec![0, 0],
            sigpackages_count: 0,
            own_sigshares_stored: false,
            sigshare_counts: vec![0, 0],
        }
    }

    #[test]
    fn ensures_mailbox_first() {
        let mut s = state(false);
        s.mailbox = None;
        assert_eq!(next_sign_task(&s).unwrap(), SignTask::EnsureMailbox);
    }

    #[test]
    fn ensures_the_broadcast_account_too() {
        // The mailbox exists but the broadcast account does not (a partial
        // EnsureMailbox, or the account was deleted): repair it before
        // anything else, for the coordinator and the others alike.
        for is_coord in [true, false] {
            let mut s = state(is_coord);
            s.broadcast = None;
            s.broadcast_address = None;
            assert_eq!(next_sign_task(&s).unwrap(), SignTask::EnsureMailbox);
            assert!(SignTask::EnsureMailbox.is_satisfied_by(&s));
        }
        let s = state(true);
        assert!(!SignTask::EnsureMailbox.is_satisfied_by(&s));
    }

    #[test]
    fn sends_commitments_when_not_stored() {
        assert_eq!(next_sign_task(&state(false)).unwrap(), SignTask::SendCommitments);
        assert_eq!(next_sign_task(&state(true)).unwrap(), SignTask::SendCommitments);
    }

    #[test]
    fn non_coordinator_waits_for_sigpackages() {
        let mut s = state(false);
        s.own_commitments_stored = true;
        assert_eq!(next_sign_task(&s).unwrap(), SignTask::WaitSigpackages);
    }

    #[test]
    fn coordinator_waits_when_not_enough_commitments() {
        let mut s = state(true);
        s.own_commitments_stored = true;
        s.commitment_counts = vec![1, 2]; // input 0 has only 1 of 2 needed
        assert_eq!(next_sign_task(&s).unwrap(), SignTask::WaitCommitments);
    }

    #[test]
    fn coordinator_makes_sigpackages_when_exactly_t_commitments() {
        let mut s = state(true);
        s.own_commitments_stored = true;
        s.commitment_counts = vec![2, 2];
        assert_eq!(next_sign_task(&s).unwrap(), SignTask::MakeSigpackages);
    }

    #[test]
    fn coordinator_aborts_on_excess_commitments() {
        let mut s = state(true);
        s.own_commitments_stored = true;
        s.commitment_counts = vec![3, 2]; // 3 > t=2 for input 0
        assert!(next_sign_task(&s).is_err());
    }

    #[test]
    fn sends_sigshares_once_sigpackages_received() {
        let mut s = state(false);
        s.own_commitments_stored = true;
        s.sigpackages_count = 2;
        assert_eq!(next_sign_task(&s).unwrap(), SignTask::SendSigshares);
    }

    #[test]
    fn non_coordinator_done_after_sigshares() {
        let mut s = state(false);
        s.own_commitments_stored = true;
        s.sigpackages_count = 2;
        s.own_sigshares_stored = true;
        assert_eq!(next_sign_task(&s).unwrap(), SignTask::Done);
    }

    #[test]
    fn coordinator_waits_for_sigshares() {
        let mut s = state(true);
        s.own_commitments_stored = true;
        s.sigpackages_count = 2;
        s.own_sigshares_stored = true;
        s.sigshare_counts = vec![1, 2]; // input 0 only has 1 of 2
        assert_eq!(next_sign_task(&s).unwrap(), SignTask::WaitSigshares);
    }

    #[test]
    fn coordinator_aggregates_when_all_sigshares_ready() {
        let mut s = state(true);
        s.own_commitments_stored = true;
        s.sigpackages_count = 2;
        s.own_sigshares_stored = true;
        s.sigshare_counts = vec![2, 2];
        assert_eq!(next_sign_task(&s).unwrap(), SignTask::Aggregate);
    }
}
