//! Pure decision logic for the DKG pipeline.
//!
//! Nothing here performs I/O. Every function is a total function of its
//! arguments, so what the DKG decides to do can be unit-tested without a
//! wallet, a chain, or other participants.

use super::state::DkgState;
use super::task::DkgTask;

/// Decide the DKG's next step.
///
/// Pure: a total function of observed protocol state. Rounds advance only
/// when every participant's package has arrived — the all-n discipline that
/// keeps every wallet's materialized key set identical, so any t-subset is
/// valid when signing begins.
pub fn next_task(s: &DkgState) -> DkgTask {
    if !s.addresses_complete() {
        return DkgTask::WaitAddresses;
    }
    if s.mailbox.is_none() || s.broadcast.is_none() {
        return DkgTask::EnsureAccounts;
    }
    // A staged-but-unsent publish outranks everything: resending exactly
    // those bytes is the crash-recovery path, and every peer dedups them.
    if let Some(round) = s.pending_round() {
        return DkgTask::PublishRound { round };
    }
    for round in 0..3u8 {
        let rs = &s.rounds[round as usize];
        if !rs.secret_present {
            return DkgTask::PublishRound { round };
        }
        if rs.others < s.params.n.saturating_sub(1) {
            return DkgTask::WaitRound { round };
        }
    }
    if !s.key_pkg_present {
        return DkgTask::FinalizeKey;
    }
    if s.frost_account.is_none() {
        return DkgTask::CreateFrostAccount;
    }
    DkgTask::CompleteFinalize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::frost::DKGParams;
    use crate::frost::dkg::state::RoundState;

    fn params(n: u8) -> DKGParams {
        DKGParams {
            id: 1,
            n,
            t: 2,
            birth_height: 0,
        }
    }

    fn round(secret: bool, others: u8) -> RoundState {
        RoundState {
            secret_present: secret,
            others,
            pending: None,
        }
    }

    /// A wallet with all addresses exchanged, both helper accounts present.
    fn state(n: u8, rounds: [RoundState; 3]) -> DkgState {
        DkgState {
            funding_account: 1,
            params: params(n),
            addresses: vec!["a".to_string(); n as usize],
            mailbox: Some(100),
            broadcast: Some(101),
            broadcast_address: Some("b".into()),
            rounds,
            key_pkg_present: false,
            frost_account: None,
            rekeyed: false,
            state0: None,
            state1: None,
            state2: None,
        }
    }

    /// Rounds 0..=`last` have secrets and all n-1 peer packages.
    fn through(n: u8, last: u8) -> [RoundState; 3] {
        let mut rounds = [round(false, 0); 3];
        for (i, r) in rounds.iter_mut().enumerate() {
            if i <= last as usize {
                *r = round(true, n - 1);
            }
        }
        rounds
    }

    #[test]
    fn waits_for_all_addresses() {
        let mut s = state(3, through(3, 2));
        s.addresses[1] = String::new();
        assert_eq!(next_task(&s), DkgTask::WaitAddresses);
    }

    #[test]
    fn ensures_accounts_before_anything_else() {
        let mut s = state(3, through(3, 2));
        s.broadcast = None;
        s.broadcast_address = None;
        assert_eq!(next_task(&s), DkgTask::EnsureAccounts);
    }

    #[test]
    fn pending_publish_outranks_producing_the_next_round() {
        let mut s = state(3, through(3, 0));
        s.rounds[0].pending = Some(0);
        assert_eq!(next_task(&s), DkgTask::PublishRound { round: 0 });
    }

    /// Regression: rounds used to advance once t packages had arrived, which
    /// with t < n finalized each wallet on a nondeterministic subset of the
    /// key material. All n participants run the DKG, so every round waits
    /// for all n-1 peers.
    #[test]
    fn rounds_wait_for_all_participants() {
        let s = state(
            3,
            {
                let mut r = [round(false, 0); 3];
                r[0] = round(true, 1); // only 1 of 2 peers so far
                r
            },
        );
        assert_eq!(next_task(&s), DkgTask::WaitRound { round: 0 });
    }

    #[test]
    fn produces_rounds_in_order() {
        assert_eq!(
            next_task(&state(3, [round(false, 0); 3])),
            DkgTask::PublishRound { round: 0 }
        );
        assert_eq!(
            next_task(&state(3, through(3, 0))),
            DkgTask::PublishRound { round: 1 }
        );
        assert_eq!(
            next_task(&state(3, through(3, 1))),
            DkgTask::PublishRound { round: 2 }
        );
    }

    #[test]
    fn finalizes_in_stages() {
        let s = state(3, through(3, 2));
        assert_eq!(next_task(&s), DkgTask::FinalizeKey);
        let mut s2 = s.clone();
        s2.key_pkg_present = true;
        assert_eq!(next_task(&s2), DkgTask::CreateFrostAccount);
        let mut s3 = s2.clone();
        s3.frost_account = Some(7);
        assert_eq!(next_task(&s3), DkgTask::CompleteFinalize);
    }

    /// A wallet the rekey already moved to the frost account must resume
    /// with the cleanup, not start over.
    #[test]
    fn rekeyed_wallet_finishes_the_cleanup() {
        let mut s = state(3, through(3, 2));
        s.rekeyed = true;
        s.key_pkg_present = true;
        s.frost_account = Some(7);
        assert_eq!(next_task(&s), DkgTask::CompleteFinalize);
    }

    #[test]
    fn preconditions_reject_stale_plans() {
        let s = state(3, through(3, 0));

        // Round 1 not produced yet: staging is due.
        assert!(DkgTask::PublishRound { round: 1 }.is_satisfied_by(&s));
        // Published (secret stored, nothing staged): re-publishing is stale.
        let mut published = s.clone();
        published.rounds[1].secret_present = true;
        assert!(!DkgTask::PublishRound { round: 1 }.is_satisfied_by(&published));
        // Staged but unconfirmed: resending the staged bytes is the point.
        published.rounds[1].pending = Some(1);
        assert!(DkgTask::PublishRound { round: 1 }.is_satisfied_by(&published));

        assert!(DkgTask::FinalizeKey.is_satisfied_by(&s));
        let mut done_key = s.clone();
        done_key.key_pkg_present = true;
        assert!(!DkgTask::FinalizeKey.is_satisfied_by(&done_key));

        assert!(DkgTask::CreateFrostAccount.is_satisfied_by(&s));
        let mut acc = s.clone();
        acc.frost_account = Some(7);
        assert!(!DkgTask::CreateFrostAccount.is_satisfied_by(&acc));

        assert!(!DkgTask::EnsureAccounts.is_satisfied_by(&s));
        let mut noacc = s.clone();
        noacc.mailbox = None;
        assert!(DkgTask::EnsureAccounts.is_satisfied_by(&noacc));

        assert!(DkgTask::WaitRound { round: 0 }.is_satisfied_by(&s));
        assert!(DkgTask::WaitAddresses.is_satisfied_by(&s));
        assert!(DkgTask::CompleteFinalize.is_satisfied_by(&s));
    }
}
