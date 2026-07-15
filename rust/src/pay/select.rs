use std::{cmp::max, collections::HashMap};

pub const NUM_POOLS: usize = 4;

// ---------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub pool: u8,
    pub amount: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Output {
    pub pool: u8,
    pub amount: u64,
}

#[derive(Debug, Clone, Default)]
struct PoolSolution {
    note_indices: Vec<usize>, // indices into notes_by_pool[p]
    sum: u64,
}

#[derive(Debug)]
pub struct Selection {
    pub inputs: Vec<Note>,
    /// Per-pool indices into the original notes-by-pool arrays.
    /// `per_pool_indices[p]` lists the indices of selected notes within pool `p`.
    pub per_pool_indices: [Vec<usize>; NUM_POOLS],
    pub change_pool: u8,
    pub change_amount: u64,
    pub fee: u64,
    pub turnstile: u64,
}

// ---------------------------------------------------------------------
// Fee formula (as given)
// ---------------------------------------------------------------------

fn pool_fee(n_i: u64, out_i: u64, pool: usize) -> u64 {
    let m = max(n_i, out_i);
    if pool == 0 {
        m
    } else if m == 1 {
        2 // shielded padding
    } else {
        m
    }
}

#[allow(dead_code)]
fn total_fee(n: &[u64; NUM_POOLS], out_count: &[u64; NUM_POOLS], f_unit: u64) -> u64 {
    (0..NUM_POOLS)
        .map(|i| pool_fee(n[i], out_count[i], i))
        .sum::<u64>()
        * f_unit
}

// ---------------------------------------------------------------------
// Grouping helpers
// ---------------------------------------------------------------------

fn group_notes_by_pool(notes: &[Note]) -> [Vec<u64>; NUM_POOLS] {
    let mut grouped: [Vec<u64>; NUM_POOLS] = Default::default();
    for n in notes {
        grouped[n.pool as usize].push(n.amount);
    }
    grouped
}

fn out_sum_and_count(outputs: &[Output]) -> ([u64; NUM_POOLS], [u64; NUM_POOLS]) {
    let mut sum = [0u64; NUM_POOLS];
    let mut count = [0u64; NUM_POOLS];
    for o in outputs {
        sum[o.pool as usize] += o.amount;
        count[o.pool as usize] += 1;
    }
    (sum, count)
}

/// Standard 0/1 knapsack-style subset-sum DP, but the "table" is a sparse
/// map from achievable-sum -> notes-used, not a dense array indexed by sum.
/// Cost is driven by the NUMBER of distinct reachable sums (bounded by 2^k,
/// and pruned to stay within [0, target+slack]), not by the sum magnitude.
fn closest_subset_sum(notes: &[u64], target: u64, slack: u64) -> Option<(u64, Vec<usize>)> {
    if target == 0 {
        return Some((0, vec![]));
    }
    let cap = target + slack;

    // dp: sum -> subset of indices achieving it
    let mut dp: HashMap<u64, Vec<usize>> = HashMap::new();
    dp.insert(0, vec![]);

    for (idx, &amt) in notes.iter().enumerate() {
        if amt > cap {
            continue; // this note alone overshoots the window, never useful here
        }
        // snapshot existing keys before mutating (0/1 knapsack: each note used once)
        let existing: Vec<(u64, Vec<usize>)> =
            dp.iter().map(|(&s, v)| (s, v.clone())).collect();

        for (s, path) in existing {
            let ns = s + amt;
            if ns <= cap && !dp.contains_key(&ns) {
                let mut np = path;
                np.push(idx);
                dp.insert(ns, np);
            }
        }
    }

    // smallest reachable sum >= target
    dp.iter()
        .filter(|&(&s, _)| s >= target)
        .min_by_key(|&(&s, _)| s)
        .map(|(&s, path)| (s, path.clone()))
        .or_else(|| {
            // nothing covers target: return largest reachable sum below it
            dp.iter().max_by_key(|&(&s, _)| s).map(|(&s, path)| (s, path.clone()))
        })
}

// ---------------------------------------------------------------------
// Solver B: monotonic greedy scan with the pool's OWN fee folded in.
// Only valid for the pool absorbing change, whose target depends on fee.
// net(n) = gross(n) - F * max(0, n - out_i) is guaranteed non-decreasing
// because every note's amount > F (given precondition).
// ---------------------------------------------------------------------

fn solve_with_folded_fee(
    notes: &[u64],
    out_i: u64,
    required_net: u64,
    f_unit: u64,
    pool: usize,
) -> Option<(u64, u64, Vec<usize>)> {
    let mut order: Vec<usize> = (0..notes.len()).collect();
    order.sort_unstable_by(|&a, &b| notes[b].cmp(&notes[a])); // largest first

    let mut gross = 0u64;
    for (pos, &idx) in order.iter().enumerate() {
        let n = (pos + 1) as u64;
        gross += notes[idx];
        let penalty = pool_fee(n, out_i, pool) * f_unit;
        let net = gross.saturating_sub(penalty);
        if net >= required_net {
            return Some((n, gross, order[..=pos].to_vec()));
        }
    }
    None // this pool alone can't cover it
}

// ---------------------------------------------------------------------
// Per-trial bookkeeping
// ---------------------------------------------------------------------

struct TrialResult {
    change_pool: u8,
    per_pool: [PoolSolution; NUM_POOLS],
    fee: u64,
    change: u64,
    turnstile: u64,
}

// ---------------------------------------------------------------------
// Main selection algorithm
// ---------------------------------------------------------------------

pub fn select_notes(notes: &[Note], outputs: &[Output], f_unit: u64, slack: u64) -> Option<Selection> {
    let notes_by_pool = group_notes_by_pool(notes);
    let (out_sum, out_count) = out_sum_and_count(outputs);
    let a_o: u64 = out_sum.iter().sum();

    // Step 1 — plain solve for shielded pools 1..3, cached & reused across trials.
    let mut plain: [PoolSolution; NUM_POOLS] = Default::default();
    for p in 1..NUM_POOLS {
        if let Some((sum, idx)) = closest_subset_sum(&notes_by_pool[p], out_sum[p], slack) {
            plain[p] = PoolSolution { note_indices: idx, sum };
        }
    }

    // Step 2 — try each shielded pool as the change absorber.
    let mut best: Option<TrialResult> = None;

    for change_pool in 0u8..NUM_POOLS as u8 {
        let cp = change_pool as usize;

        let mut n_counts = [0u64; NUM_POOLS]; // pool 0 stays at 0 in this trial
        for p in 1..NUM_POOLS {
            if p != cp {
                n_counts[p] = plain[p].note_indices.len() as u64;
            }
        }

        let fixed_fee: u64 = (0..NUM_POOLS)
            .filter(|&p| p != cp)
            .map(|p| pool_fee(n_counts[p], out_count[p], p))
            .sum::<u64>()
            * f_unit;

        let covered_by_others: u64 = (1..NUM_POOLS)
            .filter(|&p| p != cp)
            .map(|p| plain[p].sum)
            .sum();

        let required = (a_o + fixed_fee).saturating_sub(covered_by_others);

        let Some((n_cp, gross_cp, idx_cp)) =
            solve_with_folded_fee(&notes_by_pool[cp], out_count[cp], required, f_unit, cp)
        else {
            continue; // this pool can't cover it even using all its notes
        };

        let fee_cp = pool_fee(n_cp, out_count[cp], cp);
        let fee = fixed_fee + fee_cp * f_unit;
        let total_input = covered_by_others + gross_cp;
        let change = total_input.saturating_sub(a_o + fee);

        let mut per_pool: [PoolSolution; NUM_POOLS] = Default::default();
        per_pool[cp] = PoolSolution { note_indices: idx_cp.clone(), sum: gross_cp };
        for p in 1..NUM_POOLS {
            if p != cp {
                per_pool[p] = plain[p].clone();
            }
        }

        let t0 = if cp == 0 { gross_cp + out_sum[0] } else { out_sum[0] };
        let t_shielded: u64 = (1..NUM_POOLS)
            .map(|p| {
                let target = if p == cp { out_sum[p] + change } else { out_sum[p] };
                (per_pool[p].sum as i64 - target as i64).unsigned_abs()
            })
            .sum();
        let turnstile = t0 + t_shielded;

        let candidate = TrialResult { change_pool, per_pool, fee, change, turnstile };

        if best.as_ref().map_or(true, |b| candidate.turnstile < b.turnstile) {
            best = Some(candidate);
        }
    }

    // Step 3 — fallback to pool 0 if shielded notes alone can't cover the spend.
    if best.is_none() {
        best = fallback_with_pool0(&notes_by_pool, &out_sum, &out_count, a_o, f_unit);
    }

    best.map(|t| finalize_selection(t, &notes_by_pool))
}

fn fallback_with_pool0(
    notes_by_pool: &[Vec<u64>; NUM_POOLS],
    out_sum: &[u64; NUM_POOLS],
    out_count: &[u64; NUM_POOLS],
    a_o: u64,
    f_unit: u64,
) -> Option<TrialResult> {
    // Use every available shielded note (best effort), then cover the rest from pool 0.
    let mut per_pool: [PoolSolution; NUM_POOLS] = Default::default();
    let mut covered = 0u64;
    for p in 1..NUM_POOLS {
        let idx: Vec<usize> = (0..notes_by_pool[p].len()).collect();
        let sum: u64 = notes_by_pool[p].iter().sum();
        covered += sum;
        per_pool[p] = PoolSolution { note_indices: idx, sum };
    }

    let fixed_fee: u64 = (1..NUM_POOLS)
        .map(|p| pool_fee(per_pool[p].note_indices.len() as u64, out_count[p], p))
        .sum::<u64>()
        * f_unit;

    let required = (a_o + fixed_fee).saturating_sub(covered);

    let (n0, gross0, idx0) =
        solve_with_folded_fee(&notes_by_pool[0], out_count[0], required, f_unit, 0)?;

    per_pool[0] = PoolSolution { note_indices: idx0, sum: gross0 };
    let fee0 = pool_fee(n0, out_count[0], 0);
    let fee = fixed_fee + fee0 * f_unit;

    let total_input = covered + gross0;
    let change = total_input.saturating_sub(a_o + fee);

    let t0 = gross0 + out_sum[0]; // pool 0 used: additive, no cancellation

    // pick whichever shielded pool best absorbs the change
    let mut best_change_pool = 0u8;
    let mut best_shielded_t = u64::MAX;
    for p in 0..NUM_POOLS {
        let t: u64 = (1..NUM_POOLS)
            .map(|q| {
                let target = if q == p { out_sum[q] + change } else { out_sum[q] };
                (per_pool[q].sum as i64 - target as i64).unsigned_abs()
            })
            .sum();
        if t < best_shielded_t {
            best_shielded_t = t;
            best_change_pool = p as u8;
        }
    }

    Some(TrialResult {
        change_pool: best_change_pool,
        per_pool,
        fee,
        change,
        turnstile: t0 + best_shielded_t,
    })
}

fn finalize_selection(trial: TrialResult, notes_by_pool: &[Vec<u64>; NUM_POOLS]) -> Selection {
    let mut inputs = Vec::new();
    let mut per_pool_indices: [Vec<usize>; NUM_POOLS] = Default::default();
    for p in 0..NUM_POOLS {
        for &idx in &trial.per_pool[p].note_indices {
            inputs.push(Note { pool: p as u8, amount: notes_by_pool[p][idx] });
        }
        per_pool_indices[p] = trial.per_pool[p].note_indices.clone();
    }
    Selection {
        inputs,
        per_pool_indices,
        change_pool: trial.change_pool,
        change_amount: trial.change,
        fee: trial.fee,
        turnstile: trial.turnstile,
    }
}

// ---------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_notes_basic() {
        let f_unit = 5u64;
        let slack = 50u64; // DP search window for the plain (non-change) pools

        let notes = vec![
            Note { pool: 1, amount: 120 },
            Note { pool: 1, amount: 80 },
            Note { pool: 1, amount: 30 },
            Note { pool: 2, amount: 200 },
            Note { pool: 2, amount: 15 },
            Note { pool: 3, amount: 60 },
            Note { pool: 0, amount: 500 }, // last resort only
        ];

        let outputs = vec![
            Output { pool: 1, amount: 150 },
            Output { pool: 2, amount: 100 },
        ];

        let sel = select_notes(&notes, &outputs, f_unit, slack)
            .expect("should find a feasible selection");

        // Total input >= total output + fee
        let total_input: u64 = sel.inputs.iter().map(|n| n.amount).sum();
        let total_output: u64 = outputs.iter().map(|o| o.amount).sum();
        assert!(
            total_input >= total_output + sel.fee,
            "total input {} should cover outputs {} + fee {}",
            total_input,
            total_output,
            sel.fee
        );

        // Change = total input - total output - fee
        assert_eq!(
            sel.change_amount,
            total_input - total_output - sel.fee,
            "change amount should balance"
        );

        // Fee should be positive (non-zero outputs incur fees)
        assert!(sel.fee > 0, "fee should be positive for non-empty outputs");

        // Inputs should not be empty
        assert!(!sel.inputs.is_empty(), "should select at least one input");

        // The pool 0 note (500) should NOT be used unless necessary;
        // with sufficient shielded notes, the fallback shouldn't trigger.
        let used_pool0 = sel.inputs.iter().any(|n| n.pool == 0);
        assert!(
            !used_pool0,
            "pool 0 (transparent) should not be used when shielded notes suffice"
        );
    }
}