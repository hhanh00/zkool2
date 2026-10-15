//! Sync planning: which height ranges to scan, and for which accounts.
//!
//! Accounts sit at different heights, so a sync is a sequence of ranges.
//! Each range starts where one more account joins in and ends just before
//! the next account's start; the last range ends at the chain tip. An
//! account takes part in every range from its own start onwards, so the
//! blocks shared by several accounts are downloaded once.
//!
//! This is a pure function of the observed account states and the tip: no
//! database, no network. The runner in `sync/mod.rs` executes the ranges in
//! order.

/// What the planner needs to know about one account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountSyncState {
    pub account: u32,
    /// First height the account has not synced yet.
    pub next_height: u32,
    pub use_internal: bool,
}

/// One scan: blocks `start..=end` for `accounts`, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncRange {
    pub start: u32,
    pub end: u32,
    /// `(account, use_internal)` for every account that needs this range.
    pub accounts: Vec<(u32, bool)>,
}

impl SyncRange {
    pub fn account_ids(&self) -> Vec<u32> {
        self.accounts.iter().map(|(a, _)| *a).collect()
    }
}

/// Plan the ranges to scan up to `tip`. An account already past the tip
/// has nothing to do and is left out; an empty plan means nothing to scan.
pub fn plan_sync(states: &[AccountSyncState], tip: u32) -> Vec<SyncRange> {
    let mut starts: Vec<u32> = states
        .iter()
        .map(|s| s.next_height)
        .filter(|&h| h <= tip)
        .collect();
    starts.sort_unstable();
    starts.dedup();

    starts
        .iter()
        .enumerate()
        .map(|(i, &start)| {
            let end = match starts.get(i + 1) {
                Some(next) => next - 1,
                None => tip,
            };
            let mut accounts: Vec<(u32, bool)> = states
                .iter()
                .filter(|s| s.next_height <= start)
                .map(|s| (s.account, s.use_internal))
                .collect();
            accounts.sort_unstable();
            SyncRange {
                start,
                end,
                accounts,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(account: u32, next_height: u32) -> AccountSyncState {
        AccountSyncState {
            account,
            next_height,
            use_internal: account % 2 == 0,
        }
    }

    #[test]
    fn accounts_at_the_same_height_share_one_range() {
        let plan = plan_sync(&[at(1, 100), at(2, 100)], 300);
        assert_eq!(
            plan,
            vec![SyncRange {
                start: 100,
                end: 300,
                accounts: vec![(1, false), (2, true)],
            }]
        );
    }

    #[test]
    fn a_later_account_joins_at_its_own_height() {
        let plan = plan_sync(&[at(2, 200), at(1, 100)], 300);
        assert_eq!(
            plan,
            vec![
                SyncRange {
                    start: 100,
                    end: 199,
                    accounts: vec![(1, false)],
                },
                SyncRange {
                    start: 200,
                    end: 300,
                    accounts: vec![(1, false), (2, true)],
                },
            ]
        );
        assert_eq!(plan[1].account_ids(), vec![1, 2]);
    }

    #[test]
    fn an_account_at_the_tip_scans_one_block() {
        let plan = plan_sync(&[at(1, 300)], 300);
        assert_eq!(plan.len(), 1);
        assert_eq!((plan[0].start, plan[0].end), (300, 300));
    }

    #[test]
    fn accounts_past_the_tip_are_left_out_and_others_still_run() {
        assert!(plan_sync(&[at(1, 301)], 300).is_empty());
        let plan = plan_sync(&[at(1, 305), at(2, 250)], 300);
        assert_eq!(
            plan,
            vec![SyncRange {
                start: 250,
                end: 300,
                accounts: vec![(2, true)],
            }]
        );
    }

    #[test]
    fn nothing_to_plan_without_accounts() {
        assert!(plan_sync(&[], 300).is_empty());
    }
}
