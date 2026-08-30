//! Pure decision logic for the migration pipeline.
//!
//! Nothing here performs I/O. Every function is a total function of its
//! arguments, so what the migration decides to do can be unit-tested without
//! a wallet, a chain, or a clock.

use crate::pay::fee::FeeManager;

use super::state::{MigrationState, NoteRef};
use super::task::{MigrationTask, Pacing};
use super::{decompose_to_sd, next_anchor_bucket_height, MIN_SD, SD_FEE_PAD};

/// Plan the standard-denomination outputs of one O→O split transaction.
///
/// `total` is the summed value of the preselected non-SD inputs, `num_inputs`
/// how many notes they are. Returns sparse `(denom, count)` pairs, largest
/// denomination first.
///
/// An empty result means no split is worth broadcasting — either `total` is
/// below [`MIN_SD`], or the fee cannot be covered by the change left over
/// after denominating. Callers should treat `plan_split(..).is_empty()` as
/// the authoritative "is a split available?" predicate rather than
/// re-deriving one, so that what is displayed and what is executed cannot
/// disagree.
///
/// The plan is exact rather than an estimate: the [`FeeManager`] mirrors what
/// `plan_transaction` builds for a migration self-send — one Orchard input per
/// note, one output per denomination, plus a change output — so `fee()` is the
/// fee the transaction will actually pay.
pub fn plan_split(total: u64, num_inputs: u64) -> Vec<(u64, u8)> {
    // Below this, splitting costs more in fees than the value it recovers.
    if total < MIN_SD {
        return Vec::new();
    }

    let (mut digits, mut remainder) = decompose_to_sd(total);
    let mut num_outputs: u64 = digits.iter().map(|&(_, c)| c as u64).sum();

    let mut fm = FeeManager {
        migration: true,
        ..FeeManager::default()
    };
    for _ in 0..num_inputs {
        fm.add_input(2);
    }
    for _ in 0..num_outputs {
        fm.add_output(2);
    }
    fm.add_output(2); // change output

    // While the fee exceeds the change it must come out of, drop one unit of
    // the lowest denomination (last, since they are sorted largest-first).
    // Each trim moves `denom` into the change *and* removes one action from
    // the fee, so this converges in a pass or two.
    while num_outputs > 0 && fm.fee() > remainder {
        let Some((denom, count)) = digits.last_mut() else {
            break;
        };
        *count -= 1;
        remainder += *denom;
        num_outputs -= 1;
        fm.remove_output(2);
        if *count == 0 {
            digits.pop();
        }
    }

    digits
}

/// Total value of a split plan's outputs.
pub fn planned_value(digits: &[(u64, u8)]) -> u64 {
    digits.iter().map(|&(d, c)| d * c as u64).sum()
}

/// Decide the migration's next step.
///
/// Pure: a total function of observed wallet state and driver pacing. This is
/// the single authority on what the migration will do — the UI phase, the
/// loop's termination, and the action actually executed all derive from it,
/// so they cannot contradict each other. Previously a separate predicate
/// guessed at the same question and could disagree, which is how a wallet
/// could be told "splitting" forever while no split was ever built.
pub fn next_task(s: &MigrationState, p: &Pacing) -> MigrationTask {
    let split = plan_split(s.split_input_total(), s.capped_non_sd().len() as u64);

    // Terminal first: a finished migration reports completion immediately
    // rather than sleeping through one more delay.
    if split.is_empty() && s.orchard_sd.is_empty() {
        return MigrationTask::Done;
    }

    // Pace the cycle, and never act twice in the same block.
    let acted_this_block = p.last_action_height.is_some_and(|h| s.tip_height <= h);
    if !p.delay_served || acted_this_block {
        return MigrationTask::WaitDelay {
            ms: p.sampled_delay_ms,
        };
    }

    if !split.is_empty() {
        return MigrationTask::Split {
            inputs: s.capped_non_sd().iter().map(|n| n.id).collect(),
            outputs: split,
        };
    }

    // O→I. Every migrating wallet prepares against the same shared anchor, so
    // the transaction may only be built once the wallet is synced onto a
    // bucket boundary and the note is witnessed there.
    match boundary_state(s.tip_height, s.checkpoint_height, s.anchor_bucket_size) {
        BoundaryState::At { anchor } => match pick_sd(&s.sd_spendable_at(anchor)) {
            Some(note) => MigrationTask::Migrate {
                note: note.id,
                // The note's value embeds SD_FEE_PAD to pay for this hop; the
                // Ironwood output is the pure denomination.
                amount: note.value - SD_FEE_PAD,
                anchor,
            },
            // On a boundary, but no SD note is witnessed there. Aim past it.
            None => MigrationTask::WaitBoundary {
                target: next_anchor_bucket_height(anchor.saturating_add(1), s.anchor_bucket_size),
            },
        },
        // Not on a boundary. Migration does not sync itself: it waits for the
        // wallet's autosync to land the checkpoint on one.
        BoundaryState::Waiting { target } => MigrationTask::WaitBoundary { target },
    }
}

/// Pick which SD note to migrate: the largest commitment, which is a
/// deterministic but chain-ordered choice rather than a predictable one.
fn pick_sd<'a>(spendable: &[&'a NoteRef]) -> Option<&'a NoteRef> {
    spendable
        .iter()
        .max_by(|a, b| {
            let a_cmx = a.cmx.as_deref().unwrap_or(&[]);
            let b_cmx = b.cmx.as_deref().unwrap_or(&[]);
            a_cmx.cmp(b_cmx)
        })
        .copied()
}

/// Where the wallet stands relative to the anchor boundary an O→I needs.
///
/// Two outcomes, deliberately: either the wallet is sitting on a boundary and
/// the transaction can be prepared, or it is not and the caller is told which
/// height it is waiting for. Nothing blocks here and no state is held across
/// the wait — the caller comes back and asks again, so a missed boundary, a
/// sync landing late, or a restart all resolve by re-asking rather than by
/// unwinding a loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryState {
    /// The wallet is synced onto `anchor`; an O→I may be prepared here.
    At { anchor: u32 },
    /// Not there. Wait for `target` to become the chain tip.
    Waiting { target: u32 },
}

/// The next boundary the wallet could land on.
///
/// Takes the later of the tip and the wallet so the target is never a
/// boundary already behind us.
pub fn initial_boundary(tip: u32, wallet_height: u32, bucket_size: u32) -> u32 {
    next_anchor_bucket_height(tip.max(wallet_height), bucket_size)
}

/// Classify the wallet against the anchor discipline.
pub fn boundary_state(tip: u32, checkpoint: u32, bucket_size: u32) -> BoundaryState {
    if checkpoint.is_multiple_of(bucket_size) {
        BoundaryState::At { anchor: checkpoint }
    } else {
        BoundaryState::Waiting {
            target: initial_boundary(tip, checkpoint, bucket_size),
        }
    }
}

/// The phase to show the user, derived from the same `plan_split` the
/// executor uses, so the display cannot claim work that will not happen.
pub fn phase(s: &MigrationState) -> &'static str {
    if !plan_split(s.split_input_total(), s.capped_non_sd().len() as u64).is_empty() {
        "splitting"
    } else if !s.orchard_sd.is_empty() {
        "migrating"
    } else {
        "complete"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Value of one output count in a plan, 0 if the denomination is absent.
    fn count_of(digits: &[(u64, u8)], denom: u64) -> u8 {
        digits
            .iter()
            .find(|&&(d, _)| d == denom)
            .map(|&(_, c)| c)
            .unwrap_or(0)
    }

    fn note(id: u32, value: u64, height: u32) -> NoteRef {
        NoteRef {
            id,
            height,
            value,
            cmx: Some(vec![id as u8; 32]),
            has_checkpoint: true,
        }
    }

    /// A wallet with `non_sd` unsplit notes and `sd` denominated ones, synced
    /// to a checkpoint sitting exactly on an anchor boundary.
    fn state(non_sd: &[u64], sd: &[u64]) -> MigrationState {
        let mut id = 0;
        let mut mk = |values: &[u64]| {
            values
                .iter()
                .map(|&v| {
                    id += 1;
                    note(id, v, 100)
                })
                .collect::<Vec<_>>()
        };
        MigrationState {
            tip_height: 288,
            checkpoint_height: 288,
            anchor_bucket_size: 144,
            orchard_non_sd: mk(non_sd),
            orchard_sd: mk(sd),
            ironwood_sd_count: 0,
            own_address: "addr".into(),
        }
    }

    /// Pacing for a cycle whose delay and sync are already behind it.
    fn ready() -> Pacing {
        Pacing {
            sampled_delay_ms: 1000,
            last_action_height: None,
            delay_served: true,
        }
    }

    #[test]
    fn test_initial_boundary_never_aims_behind_the_wallet() {
        assert_eq!(initial_boundary(200, 100, 144), 288);
        // Wallet ahead of the observed tip: aim past the wallet, not the tip.
        assert_eq!(initial_boundary(100, 200, 144), 288);
        // Already exactly on one.
        assert_eq!(initial_boundary(288, 288, 144), 288);
    }

    #[test]
    fn test_boundary_state_two_outcomes() {
        assert_eq!(
            boundary_state(288, 288, 144),
            BoundaryState::At { anchor: 288 },
        );
        assert_eq!(
            boundary_state(200, 150, 144),
            BoundaryState::Waiting { target: 288 },
        );
    }

    /// A boundary that went by unobserved must never be prepared against:
    /// its tree state is no longer the one other wallets are using.
    #[test]
    fn test_boundary_never_targets_a_passed_height() {
        for bucket in [1u32, 4, 12, 144] {
            for tip in [1u32, 143, 145, 289, 100_001] {
                for checkpoint in [0u32, 1, 143, 288] {
                    if let BoundaryState::Waiting { target } =
                        boundary_state(tip, checkpoint, bucket)
                    {
                        assert!(target.is_multiple_of(bucket), "bucket {bucket}");
                        assert!(target >= tip, "target {target} behind tip {tip}");
                        assert!(target > checkpoint, "target {target} behind wallet");
                    }
                }
            }
        }
    }

    #[test]
    fn test_next_task_done_on_empty_wallet() {
        assert_eq!(next_task(&state(&[], &[]), &ready()), MigrationTask::Done);
    }

    #[test]
    fn test_next_task_done_skips_the_delay() {
        // A finished migration reports completion immediately; it does not
        // sleep one more time first.
        let idle = Pacing {
            delay_served: false,
            ..ready()
        };
        assert_eq!(next_task(&state(&[], &[]), &idle), MigrationTask::Done);
    }

    #[test]
    fn test_next_task_dust_below_threshold_is_done() {
        // Sub-MIN_SD dust is deliberately left in Orchard rather than being
        // split at a loss.
        assert_eq!(
            next_task(&state(&[100_000], &[]), &ready()),
            MigrationTask::Done
        );
    }

    #[test]
    fn test_next_task_delays_first() {
        let idle = Pacing {
            delay_served: false,
            ..ready()
        };
        assert_eq!(
            next_task(&state(&[550_000], &[]), &idle),
            MigrationTask::WaitDelay { ms: 1000 },
        );
    }

    #[test]
    fn test_next_task_does_not_act_twice_in_one_block() {
        let s = state(&[550_000], &[]);
        let paced = Pacing {
            last_action_height: Some(s.tip_height),
            ..ready()
        };
        assert_eq!(next_task(&s, &paced), MigrationTask::WaitDelay { ms: 1000 },);
    }

    /// Regression: the reported stall. A non-SD total in the dead zone must
    /// yield a Split, not an unactionable state the runner spins on.
    #[test]
    fn test_next_task_splits_in_the_dead_zone() {
        for total in [500_000u64, 550_000, 619_999] {
            match next_task(&state(&[total], &[]), &ready()) {
                MigrationTask::Split { inputs, outputs } => {
                    assert_eq!(inputs, vec![1]);
                    assert!(!outputs.is_empty(), "total {total}");
                }
                other => panic!("total {total} planned {other:?}, expected a split"),
            }
        }
    }

    /// Migration never syncs: a split is planned against whatever checkpoint
    /// the wallet's autosync has reached.
    #[test]
    fn test_next_task_splits_without_syncing() {
        let mut s = state(&[550_000], &[]);
        s.checkpoint_height = 200;
        assert!(matches!(
            next_task(&s, &ready()),
            MigrationTask::Split { .. },
        ));
    }

    #[test]
    fn test_next_task_migrates_sd_notes() {
        let s = state(&[], &[120_000, 1_020_000]);
        match next_task(&s, &ready()) {
            MigrationTask::Migrate {
                note,
                amount,
                anchor,
            } => {
                // Largest cmx wins; both notes carry SD_FEE_PAD for this hop.
                assert_eq!(note, 2);
                assert_eq!(amount, 1_020_000 - SD_FEE_PAD);
                assert_eq!(anchor, 288);
            }
            other => panic!("expected a migrate, got {other:?}"),
        }
    }

    #[test]
    fn test_next_task_splitting_takes_priority_over_migrating() {
        // Splits are O→O and need no anchor, so they run first.
        assert!(matches!(
            next_task(&state(&[550_000], &[120_000]), &ready()),
            MigrationTask::Split { .. },
        ));
    }

    #[test]
    fn test_next_task_waits_for_boundary_off_anchor() {
        let mut s = state(&[], &[120_000]);
        s.checkpoint_height = 300; // 300 % 144 != 0
        assert_eq!(
            next_task(&s, &ready()),
            MigrationTask::WaitBoundary { target: 432 },
        );
    }

    /// With a one-block bucket every height is a boundary, so the old
    /// `ensure!` guarding this was vacuous and an O→I could be prepared off
    /// a checkpoint the runner never waited for. The precondition now carries
    /// that discipline instead.
    #[test]
    fn test_next_task_unit_bucket_is_always_on_boundary() {
        let mut s = state(&[], &[120_000]);
        s.anchor_bucket_size = 1;
        s.checkpoint_height = 301;
        s.orchard_sd[0].height = 300;
        assert!(matches!(
            next_task(&s, &ready()),
            MigrationTask::Migrate { .. }
        ));
    }

    #[test]
    fn test_next_task_waits_when_no_sd_note_is_witnessed() {
        let mut s = state(&[], &[120_000]);
        s.orchard_sd[0].has_checkpoint = false;
        assert_eq!(
            next_task(&s, &ready()),
            MigrationTask::WaitBoundary { target: 432 },
        );
    }

    #[test]
    fn test_phase_agrees_with_next_task() {
        // The display and the executor read the same plan.
        let cases = [
            (state(&[550_000], &[]), "splitting"),
            (state(&[], &[120_000]), "migrating"),
            (state(&[], &[]), "complete"),
            (state(&[100_000], &[]), "complete"),
        ];
        for (s, expected) in cases {
            assert_eq!(phase(&s), expected);
            let terminal = matches!(next_task(&s, &ready()), MigrationTask::Done);
            assert_eq!(terminal, expected == "complete", "phase {expected}");
        }
    }

    #[test]
    fn test_precondition_rejects_a_spent_input() {
        let s = state(&[550_000], &[]);
        let task = next_task(&s, &ready());
        assert!(task.is_satisfied_by(&s));
        // The split is broadcast and locks its inputs, so they leave the
        // snapshot; replanning must not reuse them.
        let after = state(&[], &[]);
        assert!(!task.is_satisfied_by(&after));
    }

    #[test]
    fn test_precondition_rejects_a_moved_anchor() {
        let s = state(&[], &[120_000]);
        let task = next_task(&s, &ready());
        assert!(task.is_satisfied_by(&s));
        let mut moved = s.clone();
        moved.checkpoint_height = 432; // synced past the planned anchor
        assert!(!task.is_satisfied_by(&moved));
    }

    #[test]
    fn test_plan_split_below_min_sd() {
        // Not worth splitting, however it would decompose.
        assert!(plan_split(MIN_SD - 1, 1).is_empty());
        assert!(plan_split(400_000, 1).is_empty());
        assert!(plan_split(0, 0).is_empty());
    }

    /// Regression: a non-SD total anywhere in [MIN_SD, MIN_SD + 120_000) used
    /// to plan zero outputs, because a flat MIN_SD fee reserve was subtracted
    /// before decomposing. With no outputs no split was broadcast, no note was
    /// spent, and the migration loop span forever on an unchanged wallet.
    #[test]
    fn test_plan_split_dead_zone_still_splits() {
        for total in [500_000u64, 510_000, 550_000, 599_999, 619_999] {
            let digits = plan_split(total, 1);
            assert!(
                !digits.is_empty(),
                "total {total} planned no outputs — dead zone regression",
            );
            assert!(planned_value(&digits) <= total);
        }
    }

    #[test]
    fn test_plan_split_reported_case() {
        // The user's log: `SD split: [(120000, 4)]`, one input note.
        // fee = (1 input + 4 outputs + 1 change) × 5000 = 30_000, and the
        // change is 550_000 - 480_000 = 70_000, so nothing is trimmed.
        let digits = plan_split(550_000, 1);
        assert_eq!(digits, vec![(120_000, 4)]);
        assert_eq!(planned_value(&digits), 480_000);
    }

    #[test]
    fn test_plan_split_trims_when_fee_exceeds_change() {
        // 50 dust inputs summing to 550_000: the fee starts at
        // (50 + 4 + 1) × 5000 = 275_000 against 70_000 of change, so outputs
        // are trimmed until it fits rather than the split being abandoned.
        let digits = plan_split(550_000, 50);
        assert!(!digits.is_empty(), "should still plan a smaller split");
        assert!(count_of(&digits, 120_000) < 4, "should have trimmed");
        assert!(planned_value(&digits) <= 550_000);
    }

    #[test]
    fn test_plan_split_change_covers_fee() {
        // Whatever it plans, the change must cover the fee it will pay.
        for num_inputs in [1u64, 2, 7, 50] {
            for total in [500_000u64, 620_000, 1_000_000, 12_345_678, 100_000_000] {
                let digits = plan_split(total, num_inputs);
                let num_outputs: u64 = digits.iter().map(|&(_, c)| c as u64).sum();
                let remainder = total - planned_value(&digits);

                let mut fm = FeeManager {
                    migration: true,
                    ..FeeManager::default()
                };
                for _ in 0..num_inputs {
                    fm.add_input(2);
                }
                for _ in 0..num_outputs {
                    fm.add_output(2);
                }
                fm.add_output(2);

                if !digits.is_empty() {
                    assert!(
                        fm.fee() <= remainder,
                        "total {total} / {num_inputs} inputs: fee {} > change {remainder}",
                        fm.fee(),
                    );
                }
            }
        }
    }

    #[test]
    fn test_plan_split_outputs_are_sd() {
        let digits = plan_split(12_345_678, 3);
        assert!(!digits.is_empty());
        for &(denom, count) in &digits {
            assert!(super::super::is_sd(denom), "{denom} is not a denomination");
            assert!(count > 0, "zero-count denominations should be dropped");
        }
    }

    #[test]
    fn test_plan_split_largest_first() {
        let digits = plan_split(100_000_000, 1);
        let denoms: Vec<u64> = digits.iter().map(|&(d, _)| d).collect();
        let mut sorted = denoms.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        assert_eq!(denoms, sorted, "trim assumes largest-first ordering");
    }
}
