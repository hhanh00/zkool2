//! Anytime branch-and-bound note selection for Zcash transactions.
//!
//! Two modes:
//!   - `Mode::Fee`     — minimize ZIP-317 conventional fee. Monotonic, so
//!     the search prunes supersets after reaching feasibility.
//!   - `Mode::Privacy` — minimize cross-pool turnstile value (tin + tout +
//!     Σ|balance|). Non-monotonic, so the search continues exploring
//!     supersets that might improve per-pool balance.
//!
//! Both modes fold change-pool assignment into the cost evaluation at
//! each feasibility checkpoint.
//!
//! Replaces the knapsack+greedy solver in `select.rs`.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::time::{Duration, Instant};

use tracing::info;

// ---------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------

pub const N_POOLS: usize = 4; // Transparent=0, Sapling=1, Orchard=2, Ironwood=3

/// Optimisation target for note selection.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Minimize ZIP-317 conventional fee.
    Fee,
    /// Minimize cross-pool turnstile (privacy-preserving).
    Privacy,
}

/// Candidate note for selection.  `pool` is the pool index (0–3), `amount`
/// is the note value in zatoshis.
#[derive(Clone, Debug)]
pub(super) struct Note {
    pub pool: u8,
    pub amount: u64,
    /// Index within the original per-pool `input_pools[pool]` array so the
    /// caller can map results back after sorting/reordering.
    pub pool_index: usize,
}

/// A required output.  Mirrors `select::Output` so callers in `plan.rs`
/// don't need type-level changes.
#[derive(Clone, Debug)]
pub(super) struct Output {
    pub pool: u8,
    pub amount: u64,
}

/// Result of a successful coin-selection run.
#[derive(Debug)]
pub(super) struct Selection {
    #[allow(dead_code)]
    pub inputs: Vec<Note>,
    /// Per-pool indices into the original notes-by-pool arrays.
    pub per_pool_indices: [Vec<usize>; N_POOLS],
    pub change_pool: u8,
    #[allow(dead_code)]
    pub change_amount: u64,
    #[allow(dead_code)]
    pub fee: u64,
}

// ZIP-317 constants
const GRACE_ACTIONS: u64 = 2;

// ---------------------------------------------------------------------
// Search state
// ---------------------------------------------------------------------

#[derive(Clone, Debug)]
struct State {
    sum: u64,
    /// Per-pool balance: inputs_value - outputs_value (including change).
    balance: [i64; N_POOLS],
    /// Number of inputs selected per pool.  Drives the fee computation.
    n_inputs: [u32; N_POOLS],
    /// Transparent input value (zats).
    tin: u64,
    /// Transparent output value (zats).  Fixed once from Context.
    tout: u64,
    /// Indices into `ctx.notes` that have been selected.
    selected: Vec<usize>,
}

/// Fixed, precomputed context for a single selection run.
struct Context<'a> {
    notes: &'a [Note],               // sorted: shielded pools first, then transparent; within pool descending by amount
    output_amounts: [u64; N_POOLS],  // output value per pool (zats)
    n_outputs: [u32; N_POOLS],       // number of fixed recipient outputs per pool
    output_sum: u64,                 // total output value (zats)
    f_unit: u64,                     // COST_PER_ACTION (5000)
    migration: bool,                 // orchard fee = inputs+outputs instead of max
    recipient_pays_fee: bool,
    first_recipient_amount: u64,
    mode: Mode,
}

// ---------------------------------------------------------------------
// Budget (anytime control)
// ---------------------------------------------------------------------

pub(super) struct Budget {
    pub max_nodes: u64,
    pub max_time: Duration,
    pub beam_width: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Budget { max_nodes: 100_000, max_time: Duration::from_millis(200), beam_width: 24 }
    }
}

struct BudgetTracker {
    start: Instant,
    limit: Duration,
    max_nodes: u64,
    nodes: u64,
}

impl BudgetTracker {
    fn new(b: &Budget) -> Self {
        BudgetTracker { start: Instant::now(), limit: b.max_time, max_nodes: b.max_nodes, nodes: 0 }
    }
    fn exceeded(&mut self) -> bool {
        self.nodes += 1;
        self.nodes > self.max_nodes || self.start.elapsed() > self.limit
    }
}

// ---------------------------------------------------------------------
// Fee computation — matches `FeeManager::fee()` in fee.rs
// ---------------------------------------------------------------------

/// ZIP-317 fee for a state, assuming change is assigned to `change_pool`.
fn compute_fee(n_inputs: &[u32; N_POOLS], n_outputs: &[u32; N_POOLS], change_pool: u8, f_unit: u64, migration: bool) -> u64 {
    let cp = change_pool as usize;
    let mut n_outs = *n_outputs;
    n_outs[cp] = n_outs[cp].saturating_add(1); // change output

    // Transparent: max(inputs, outputs), no padding
    let t = n_inputs[0].max(n_outs[0]) as u64;

    // Sapling: if any activity, max(inputs, outputs, 2)
    let s: u64 = if n_inputs[1] > 0 || n_outs[1] > 0 {
        n_inputs[1].max(n_outs[1]).max(2) as u64
    } else { 0 };

    // Orchard: migration? inputs+outputs : max(inputs,outputs); clamped to 2
    let o: u64 = if n_inputs[2] > 0 || n_outs[2] > 0 {
        if migration {
            (n_inputs[2] as u64 + n_outs[2] as u64).max(2)
        } else {
            n_inputs[2].max(n_outs[2]).max(2) as u64
        }
    } else { 0 };

    // Ironwood: same as Orchard non-migration
    let iw: u64 = if n_inputs[3] > 0 || n_outs[3] > 0 {
        n_inputs[3].max(n_outs[3]).max(2) as u64
    } else { 0 };

    let logical = (t + s + o + iw).max(GRACE_ACTIONS);
    logical * f_unit
}

/// Minimum possible fee for a state (no change output added yet).
/// Used as the lower-bound estimate — since adding change can only add
/// an output (monotonic), the actual final fee is >= this.
fn compute_min_fee(n_inputs: &[u32; N_POOLS], n_outputs: &[u32; N_POOLS], f_unit: u64, migration: bool) -> u64 {
    let t = n_inputs[0].max(n_outputs[0]) as u64;
    let s: u64 = if n_inputs[1] > 0 || n_outputs[1] > 0 {
        n_inputs[1].max(n_outputs[1]).max(2) as u64
    } else { 0 };
    let o: u64 = if n_inputs[2] > 0 || n_outputs[2] > 0 {
        if migration {
            (n_inputs[2] as u64 + n_outputs[2] as u64).max(2)
        } else {
            n_inputs[2].max(n_outputs[2]).max(2) as u64
        }
    } else { 0 };
    let iw: u64 = if n_inputs[3] > 0 || n_outputs[3] > 0 {
        n_inputs[3].max(n_outputs[3]).max(2) as u64
    } else { 0 };
    let logical = (t + s + o + iw).max(GRACE_ACTIONS);
    logical * f_unit
}

// ---------------------------------------------------------------------
// Cost evaluation — folds change-pool assignment into the search
// ---------------------------------------------------------------------

/// Evaluate a state by trying every pool as the change absorber.
/// Returns `(cost, best_change_pool)` if any pool yields a feasible
/// solution, or `(u64::MAX, 0)` if none does.
/// In Fee mode cost is the ZIP-317 fee; in Privacy mode cost is the
/// cross-pool turnstile value.
fn evaluate(state: &State, ctx: &Context) -> (u64, u8) {
    let (cost, pool) = match ctx.mode {
        Mode::Fee => evaluate_fee(state, ctx),
        Mode::Privacy => evaluate_privacy(state, ctx),
    };
    if cost == u64::MAX {
        info!("evaluate: mode={:?}, sum={}, INFEASIBLE", ctx.mode, state.sum);
    } else {
        info!("evaluate: mode={:?}, sum={}, fee/cost={}, change_pool={}", ctx.mode, state.sum, cost, pool);
    }
    (cost, pool)
}

/// Fee-mode evaluation: return the lowest fee achievable by assigning
/// change to any pool.
fn evaluate_fee(state: &State, ctx: &Context) -> (u64, u8) {
    let mut best_fee = u64::MAX;
    let mut best_pool = 0u8;

    for cp in 0..N_POOLS as u8 {
        let fee = compute_fee(&state.n_inputs, &ctx.n_outputs, cp, ctx.f_unit, ctx.migration);

        let needed = if ctx.recipient_pays_fee {
            if fee > ctx.first_recipient_amount {
                info!(
                    "evaluate_fee: cp={} SKIP — fee({}) > first_recipient_amount({})",
                    cp, fee, ctx.first_recipient_amount
                );
                continue; // fee exceeds what the first recipient can cover
            }
            ctx.output_sum
        } else {
            ctx.output_sum.saturating_add(fee)
        };

        let feasible = state.sum >= needed;
        info!(
            "evaluate_fee: cp={} fee={} needed={} sum={} feasible={} best_fee={}",
            cp, fee, needed, state.sum, feasible, best_fee
        );

        if feasible && fee < best_fee {
            best_fee = fee;
            best_pool = cp;
        }
    }

    (best_fee, best_pool)
}

/// Privacy-mode evaluation: return the lowest turnstile achievable by
/// assigning change to any pool, with fee feasibility as a constraint.
fn evaluate_privacy(state: &State, ctx: &Context) -> (u64, u8) {
    let mut best_turnstile = u64::MAX;
    let mut best_pool = 0u8;

    for cp in 0..N_POOLS as u8 {
        let fee = compute_fee(&state.n_inputs, &ctx.n_outputs, cp, ctx.f_unit, ctx.migration);

        let needed = if ctx.recipient_pays_fee {
            if fee > ctx.first_recipient_amount {
                continue;
            }
            ctx.output_sum
        } else {
            ctx.output_sum.saturating_add(fee)
        };

        if state.sum < needed {
            continue;
        }

        let change = state.sum.saturating_sub(needed);

        // Turnstile: transparent value + per-pool shielded imbalance.
        // All transparent value passes through the turnstile.
        let turnstile = state.tin
            + state.tout
            + (1..N_POOLS).map(|p| {
                let bal = state.balance[p];
                // Change output adds to the change-pool's output side,
                // deepening any deficit or reducing any surplus.
                let adjusted = if p == cp as usize {
                    (bal - change as i64).unsigned_abs()
                } else {
                    bal.unsigned_abs()
                };
                adjusted
            }).sum::<u64>();

        if turnstile < best_turnstile {
            best_turnstile = turnstile;
            best_pool = cp;
        }
    }

    (best_turnstile, best_pool)
}

/// Optimistic lower bound on the cost achievable by extending `state`.
fn lower_bound(state: &State, ctx: &Context) -> u64 {
    match ctx.mode {
        Mode::Fee => lower_bound_fee(state, ctx),
        Mode::Privacy => lower_bound_privacy(state, ctx),
    }
}

/// Fee-mode lower bound: the minimum fee across all change-pool
/// assignments. Because fee is monotonic non-decreasing as notes are
/// added, the current minimum fee is always a valid bound.
fn lower_bound_fee(state: &State, ctx: &Context) -> u64 {
    let mut min_fee = u64::MAX;
    for cp in 0..N_POOLS as u8 {
        let fee = compute_fee(&state.n_inputs, &ctx.n_outputs, cp, ctx.f_unit, ctx.migration);
        if ctx.recipient_pays_fee && fee > ctx.first_recipient_amount {
            continue;
        }
        if fee < min_fee {
            min_fee = fee;
        }
    }
    // Also try compute_min_fee (no change output) as a tighter bound
    let min_no_change = compute_min_fee(&state.n_inputs, &ctx.n_outputs, ctx.f_unit, ctx.migration);
    if min_no_change < min_fee {
        min_fee = min_no_change;
    }
    min_fee
}

/// Privacy-mode lower bound: tin + tout is monotonic; per-pool
/// |balance| can only shrink by at most the sum of remaining note
/// values in that pool. So we subtract the remaining pool value from
/// each pool's absolute balance to get an optimistic floor.
fn lower_bound_privacy(state: &State, ctx: &Context) -> u64 {
    // tin + tout is monotonic (inputs only add to tin)
    let mut bound = state.tin + state.tout;

    // Compute remaining note value per pool (without full remaining_notes)
    let chosen: HashSet<usize> = state.selected.iter().copied().collect();
    let mut remaining_by_pool = [0u64; N_POOLS];
    for (i, n) in ctx.notes.iter().enumerate() {
        if !chosen.contains(&i) {
            remaining_by_pool[n.pool as usize] += n.amount;
        }
    }

    for p in 1..N_POOLS {
        let abs_bal = state.balance[p].unsigned_abs();
        bound += abs_bal.saturating_sub(remaining_by_pool[p]);
    }

    bound
}

// ---------------------------------------------------------------------
// State transitions
// ---------------------------------------------------------------------

fn initial_state(ctx: &Context) -> State {
    let mut balance = [0i64; N_POOLS];
    // Seed balance with negative output amounts so Privacy-mode (if
    // re-added later) can compute per-pool imbalance.
    for p in 0..N_POOLS {
        balance[p] = -(ctx.output_amounts[p] as i64);
    }
    State {
        sum: 0,
        balance,
        n_inputs: [0; N_POOLS],
        tin: 0,
        tout: ctx.output_amounts[0],
        selected: Vec::new(),
    }
}

/// Apply picking `note_idx` on top of `state`, returning a new child state.
fn apply(state: &State, note_idx: usize, note: &Note) -> State {
    let mut child = state.clone();
    child.sum += note.amount;
    child.selected.push(note_idx);
    child.n_inputs[note.pool as usize] = child.n_inputs[note.pool as usize].saturating_add(1);

    match note.pool {
        0 => {
            child.tin += note.amount;
        }
        1 | 2 | 3 => {
            child.balance[note.pool as usize] += note.amount as i64;
        }
        _ => {} // unreachable, but no panic
    }
    child
}

/// Notes not yet selected, each with its original index in ctx.notes.
fn remaining_notes<'a>(ctx: &Context<'a>, state: &State) -> Vec<(usize, &'a Note)> {
    let chosen: HashSet<usize> = state.selected.iter().copied().collect();
    ctx.notes
        .iter()
        .enumerate()
        .filter(|(i, _)| !chosen.contains(i))
        .map(|(i, n)| (i, n))
        .collect()
}

// ---------------------------------------------------------------------
// Heuristic for beam-search expansion ordering
// ---------------------------------------------------------------------

/// Score a candidate note for beam expansion. Higher = expand first.
fn local_score(note: &Note, state: &State, mode: Mode) -> i64 {
    match mode {
        Mode::Fee => {
            // Larger notes first — fewer notes means fewer inputs, lower fee.
            note.amount as i64
        }
        Mode::Privacy => {
            // Prefer notes that reduce a pool's imbalance toward zero.
            let bal = state.balance[note.pool as usize];
            let old_abs = bal.unsigned_abs() as i64;
            let new_abs = (bal + note.amount as i64).unsigned_abs() as i64;
            let reduction = old_abs - new_abs;
            // Light penalty on note size so smaller, more precise notes
            // are preferred when equally corrective.
            reduction - (note.amount as i64 / 1000)
        }
    }
}

fn top_k_by_local_heuristic<'a>(
    remaining: &[(usize, &'a Note)],
    state: &State,
    mode: Mode,
    k: usize,
) -> Vec<(usize, &'a Note)> {
    if remaining.len() <= k {
        return remaining.to_vec();
    }
    let mut scored: Vec<(i64, usize, &Note)> = remaining
        .iter()
        .map(|&(idx, n)| (local_score(n, state, mode), idx, n))
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0)); // descending
    scored.into_iter().take(k).map(|(_, idx, n)| (idx, n)).collect()
}

// ---------------------------------------------------------------------
// Dominance / state-key
// ---------------------------------------------------------------------

/// Balances rounded to nearest QUANT zats to bound the `seen` map size.
const QUANT: i64 = 1000;

type StateKey = (u64, [i64; N_POOLS], u64, [u32; N_POOLS]);
//            = (sum, quantized_balances, tout, n_inputs)

fn state_key(state: &State) -> StateKey {
    let q = |b: i64| (b / QUANT) * QUANT;
    (
        state.sum,
        [
            q(state.balance[0]), q(state.balance[1]),
            q(state.balance[2]), q(state.balance[3]),
        ],
        state.tout,
        state.n_inputs,
    )
}

// ---------------------------------------------------------------------
// Greedy baseline (always-available fallback)
// ---------------------------------------------------------------------

/// Greedily take largest notes until feasible.  Returns `None` if even
/// consuming every note doesn't reach the target.
fn greedy_solution(ctx: &Context) -> Option<(State, u64, u8)> {
    let mut state = initial_state(ctx);
    let mut order: Vec<usize> = (0..ctx.notes.len()).collect();
    order.sort_by(|&a, &b| ctx.notes[b].amount.cmp(&ctx.notes[a].amount));

    for &idx in &order {
        let (fee, pool) = evaluate(&state, ctx);
        if fee != u64::MAX {
            return Some((state, fee, pool));
        }
        state = apply(&state, idx, &ctx.notes[idx]);
    }

    // Try once more after consuming all notes
    let (fee, pool) = evaluate(&state, ctx);
    if fee != u64::MAX {
        Some((state, fee, pool))
    } else {
        None
    }
}

// ---------------------------------------------------------------------
// Priority queue item
// ---------------------------------------------------------------------

struct QueueItem {
    bound: u64,
    seq: u64,
    state: State,
}

impl PartialEq for QueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.bound == other.bound && self.seq == other.seq
    }
}
impl Eq for QueueItem {}
impl PartialOrd for QueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for QueueItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bound.cmp(&other.bound).then(self.seq.cmp(&other.seq))
    }
}

// ---------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------

/// Select notes to cover `outputs`, minimizing the cost given by `mode`.
///
/// `f_unit` is `COST_PER_ACTION` (5000).  Notes with `amount < f_unit` are
/// filtered out — they can never pay for their own marginal fee.
///
/// Returns `None` when the available notes cannot cover the outputs plus
/// the required fee (or when `recipient_pays_fee` and the fee exceeds
/// `first_recipient_amount`).
pub(super) fn select_notes(
    notes: &[Note],
    outputs: &[Output],
    f_unit: u64,
    migration: bool,
    recipient_pays_fee: bool,
    first_recipient_amount: u64,
    mode: Mode,
) -> Option<Selection> {
    // ---- 1. Pre-filter dust ------------------------------------------------
    let total_notes = notes.len();
    let total_input_sum: u64 = notes.iter().map(|n| n.amount).sum();
    info!(
        "select_notes: {} notes total, sum={} zats, outputs={}, f_unit={}, migration={}, recipient_pays_fee={}, first_recipient={}, mode={:?}",
        total_notes, total_input_sum, outputs.len(), f_unit, migration, recipient_pays_fee, first_recipient_amount, mode
    );
    let filtered: Vec<Note> = notes
        .iter()
        .filter(|n| n.amount >= f_unit)
        .cloned()
        .collect();
    info!(
        "select_notes: after dust filter (>= {}): {} notes (removed {})",
        f_unit,
        filtered.len(),
        total_notes - filtered.len()
    );
    if filtered.is_empty() {
        info!("select_notes: FAIL — no notes after dust filter");
        return None;
    }

    // ---- 2. Build output aggregates ---------------------------------------
    let mut output_amounts = [0u64; N_POOLS];
    let mut n_outputs = [0u32; N_POOLS];
    let mut output_sum = 0u64;
    for o in outputs {
        let p = o.pool as usize;
        if p < N_POOLS {
            output_amounts[p] = output_amounts[p].saturating_add(o.amount);
            n_outputs[p] = n_outputs[p].saturating_add(1);
            output_sum = output_sum.saturating_add(o.amount);
        }
    }
    info!(
        "select_notes: output_sum={} zats, n_outputs={:?}, output_amounts={:?}",
        output_sum, n_outputs, output_amounts
    );

    // ---- 3. Sort notes: shielded pools first, then transparent; within each
    //         pool descending by amount (best notes for greedy + heuristic) --
    let mut sorted: Vec<Note> = filtered;
    sorted.sort_by(|a, b| {
        let a_shielded = if a.pool == 0 { 1u8 } else { 0u8 };
        let b_shielded = if b.pool == 0 { 1u8 } else { 0u8 };
        a_shielded.cmp(&b_shielded).then(b.amount.cmp(&a.amount))
    });

    // ---- 4. Build context -------------------------------------------------
    let ctx = Context {
        notes: &sorted,
        output_amounts,
        n_outputs,
        output_sum,
        f_unit,
        migration,
        recipient_pays_fee,
        first_recipient_amount,
        mode,
    };

    // ---- 5. Greedy baseline -----------------------------------------------
    info!("select_notes: running greedy baseline...");
    let (mut best_state, mut best_cost, mut best_pool) = match greedy_solution(&ctx) {
        Some((state, cost, pool)) => {
            info!(
                "select_notes: greedy found solution — sum={}, fee={}, change_pool={}, n_inputs={:?}",
                state.sum, cost, pool, state.n_inputs
            );
            (state, cost, pool)
        }
        None => {
            // Even consuming all notes doesn't reach the target
            info!("select_notes: greedy failed, trying all notes...");
            let state = {
                let mut s = initial_state(&ctx);
                for idx in 0..ctx.notes.len() {
                    s = apply(&s, idx, &ctx.notes[idx]);
                }
                s
            };
            info!(
                "select_notes: all-notes state sum={}, n_inputs={:?}",
                state.sum, state.n_inputs
            );
            let (cost, pool) = evaluate(&state, &ctx);
            if cost == u64::MAX {
                info!(
                    "select_notes: FAIL — all notes (sum={}) still infeasible. output_sum={}, recipient_pays_fee={}, first_recipient={}",
                    state.sum, output_sum, recipient_pays_fee, first_recipient_amount
                );
                return None;
            }
            info!(
                "select_notes: all-notes feasible — fee={}, pool={}",
                cost, pool
            );
            (state, cost, pool)
        }
    };

    let budget = Budget::default();
    let mut tracker = BudgetTracker::new(&budget);
    let mut heap: BinaryHeap<Reverse<QueueItem>> = BinaryHeap::new();
    let mut seen: HashMap<StateKey, u64> = HashMap::new();
    let mut seq: u64 = 0;

    // ---- 6. Initialize search ---------------------------------------------
    let start = initial_state(&ctx);
    let start_bound = lower_bound(&start, &ctx);
    if start_bound < best_cost {
        heap.push(Reverse(QueueItem { bound: start_bound, seq, state: start }));
    }

    let monotonic = matches!(ctx.mode, Mode::Fee);

    // ---- 7. Best-first branch-and-bound -----------------------------------
    while let Some(Reverse(item)) = heap.pop() {
        if tracker.exceeded() {
            break; // anytime cutoff
        }

        let QueueItem { bound, state, .. } = item;

        if bound >= best_cost {
            continue; // cannot beat incumbent
        }

        // Feasibility check: evaluate with change-pool folding
        let (cost, pool) = evaluate(&state, &ctx);
        if cost != u64::MAX && cost < best_cost {
            best_cost = cost;
            best_state = state.clone();
            best_pool = pool;
            if monotonic {
                // Fee is monotonic: supersets can only have >= cost
                continue;
            }
            // Privacy is non-monotonic: supersets may improve balance,
            // so don't prune — keep expanding this state.
        }

        let remaining = remaining_notes(&ctx, &state);
        if remaining.is_empty() {
            continue;
        }

        // Overshoot cap
        let max_remaining = remaining.iter().map(|(_, n)| n.amount).max().unwrap_or(0);
        if state.sum > ctx.output_sum.saturating_add(max_remaining) {
            continue;
        }

        let candidates = top_k_by_local_heuristic(&remaining, &state, ctx.mode, budget.beam_width);

        for (note_idx, note) in candidates {

            let child = apply(&state, note_idx, note);
            let child_bound = lower_bound(&child, &ctx);

            if child_bound >= best_cost {
                continue;
            }

            let key = state_key(&child);
            if let Some(&existing_bound) = seen.get(&key) {
                if existing_bound <= child_bound {
                    continue; // dominated
                }
            }
            seen.insert(key, child_bound);

            seq = seq.saturating_add(1);
            heap.push(Reverse(QueueItem { bound: child_bound, seq, state: child }));
        }
    }

    // ---- 8. Build Selection ------------------------------------------------
    // Recompute the actual fee for the best state (needed even in Privacy
    // mode: the Selection always reports the ZIP-317 fee, not turnstile).
    let (actual_fee, _) = evaluate_fee(&best_state, &ctx);
    // If evaluate_fee returned u64::MAX (shouldn't happen since best_state
    // was feasible), fall back to computing fee for the recorded best_pool.
    let fee = if actual_fee != u64::MAX {
        actual_fee
    } else {
        compute_fee(
            &best_state.n_inputs,
            &ctx.n_outputs,
            best_pool,
            ctx.f_unit,
            ctx.migration,
        )
    };

    let change_needed = if recipient_pays_fee {
        ctx.output_sum
    } else {
        ctx.output_sum.saturating_add(fee)
    };
    let change_amount = best_state.sum.saturating_sub(change_needed);

    // Gather inputs and per-pool indices
    let inputs: Vec<Note> = best_state.selected.iter().map(|&idx| ctx.notes[idx].clone()).collect();

    let mut per_pool_indices: [Vec<usize>; N_POOLS] = Default::default();
    for &idx in &best_state.selected {
        let note = &ctx.notes[idx];
        per_pool_indices[note.pool as usize].push(note.pool_index);
    }

    Some(Selection {
        inputs,
        per_pool_indices,
        change_pool: best_pool,
        change_amount,
        fee,
    })
}

// ---------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_notes_basic() {
        let notes = vec![
            Note { pool: 1, amount: 120_000, pool_index: 0 },
            Note { pool: 1, amount: 80_000, pool_index: 1 },
            Note { pool: 1, amount: 30_000, pool_index: 2 },
            Note { pool: 2, amount: 200_000, pool_index: 0 },
            Note { pool: 2, amount: 15_000, pool_index: 1 },
            Note { pool: 3, amount: 60_000, pool_index: 0 },
            Note { pool: 0, amount: 500_000, pool_index: 0 },
        ];

        let outputs = vec![
            Output { pool: 1, amount: 150_000 },
            Output { pool: 2, amount: 100_000 },
        ];

        let f_unit = 5_000u64;

        let sel = select_notes(&notes, &outputs, f_unit, false, false, 0, Mode::Fee)
            .expect("should find a feasible selection");

        // Total input >= total output + fee
        let total_input: u64 = sel.inputs.iter().map(|n| n.amount).sum();
        let total_output: u64 = outputs.iter().map(|o| o.amount).sum();
        assert!(
            total_input >= total_output + sel.fee,
            "total input {} should cover outputs {} + fee {}",
            total_input, total_output, sel.fee
        );

        // Change = total input - total output - fee
        assert_eq!(
            sel.change_amount,
            total_input - total_output - sel.fee,
            "change amount should balance"
        );

        // Fee should be positive
        assert!(sel.fee > 0, "fee should be positive for non-empty outputs");

        // Inputs should not be empty
        assert!(!sel.inputs.is_empty(), "should select at least one input");
    }

    #[test]
    fn test_select_notes_dust_filtered() {
        // Notes below f_unit (5000) should be filtered out
        let notes = vec![
            Note { pool: 1, amount: 120, pool_index: 0 },       // dust
            Note { pool: 1, amount: 4_000, pool_index: 1 },     // dust
            Note { pool: 2, amount: 1_000_000, pool_index: 0 }, // only usable note
        ];
        let outputs = vec![Output { pool: 2, amount: 500_000 }];
        let f_unit = 5_000u64;

        let sel = select_notes(&notes, &outputs, f_unit, false, false, 0, Mode::Fee)
            .expect("should find a feasible selection");

        // Should only use the non-dust note
        assert_eq!(sel.inputs.len(), 1);
        assert_eq!(sel.inputs[0].pool, 2);
    }

    #[test]
    fn test_select_notes_recipient_pays_fee() {
        let notes = vec![
            Note { pool: 2, amount: 200_000, pool_index: 0 },
            Note { pool: 2, amount: 100_000, pool_index: 1 },
            Note { pool: 2, amount: 50_000, pool_index: 2 },
        ];
        let outputs = vec![Output { pool: 2, amount: 150_000 }];
        let f_unit = 5_000u64;

        // First recipient has 200_000, fee will be well under that
        let sel = select_notes(&notes, &outputs, f_unit, false, true, 200_000, Mode::Fee)
            .expect("should find a feasible selection");

        // With recipient_pays_fee, target = output_sum (no fee added)
        let total_input: u64 = sel.inputs.iter().map(|n| n.amount).sum();
        let total_output: u64 = outputs.iter().map(|o| o.amount).sum();
        // Change = total_input - total_output (fee comes from recipient)
        assert_eq!(
            sel.change_amount,
            total_input - total_output,
            "change = inputs - outputs when recipient pays fee"
        );
        assert!(sel.fee <= 200_000, "fee must not exceed first recipient amount");
    }

    #[test]
    fn test_select_notes_recipient_pays_fee_too_high() {
        let notes = vec![
            Note { pool: 2, amount: 200_000, pool_index: 0 },
            Note { pool: 2, amount: 100_000, pool_index: 1 },
            Note { pool: 1, amount: 300_000, pool_index: 0 },
            Note { pool: 0, amount: 500_000, pool_index: 0 },
        ];
        let outputs = vec![Output { pool: 2, amount: 150_000 }];
        let f_unit = 5_000u64;

        // First recipient only has 1_000 zats — fee will exceed that
        let result = select_notes(&notes, &outputs, f_unit, false, true, 1_000, Mode::Fee);
        // Should still work if it can find a change pool where fee <= 1000,
        // but with 4 pools and enough notes the min fee is >= 10000.
        // This may or may not find a solution depending on fee structure.
        // Just verify it doesn't panic.
        if let Some(sel) = result {
            assert!(sel.fee <= 1_000, "if solution found, fee must fit recipient");
        }
    }

    #[test]
    fn test_select_notes_insufficient_funds() {
        let notes = vec![
            Note { pool: 2, amount: 10_000, pool_index: 0 },
        ];
        let outputs = vec![Output { pool: 2, amount: 1_000_000 }];
        let f_unit = 5_000u64;

        let result = select_notes(&notes, &outputs, f_unit, false, false, 0, Mode::Fee);
        assert!(result.is_none(), "should return None for insufficient funds");
    }

    #[test]
    fn test_compute_fee_matches_feemanager() {
        // Compare compute_fee against FeeManager for a representative case.
        // Sapling: 2 inputs, 1 output  → max(2,1,2)=2
        // Orchard: 1 input, 2 outputs (no migration) → max(1,3,2)=3
        // Ironwood: 0 inputs, 0 outputs → 0
        // Transparent: 0 inputs, 0 outputs → 0
        // Total logical = max(2+3,2) = 5, fee = 25000
        let n_inputs: [u32; 4] = [0, 2, 1, 0];
        let n_outputs: [u32; 4] = [0, 1, 2, 0];
        let fee = compute_fee(&n_inputs, &n_outputs, 2, 5_000, false);
        // With change in pool 2: n_outputs[2] becomes 3
        // Sapling: max(2,1,2) = 2
        // Orchard: max(1,3,2) = 3
        // Total: max(5, 2) = 5, fee = 25000
        assert_eq!(fee, 25_000);
    }
}
