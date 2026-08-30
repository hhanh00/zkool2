//! Note migration: moving a wallet's ZEC out of Orchard and into Ironwood.
//!
//! The migration runs as a sequence of tasks. [`state`] observes the wallet,
//! [`plan`] decides what to do next (purely, from that observation alone),
//! [`task`] is the vocabulary of things that can be done, and [`exec`] does
//! them. This module holds only the denomination arithmetic they share.

pub mod exec;
pub mod plan;
pub mod state;
pub mod step;
pub mod task;

use crate::pay::fee::COST_PER_ACTION;

/// Minimum spendable chunk: 100 × COST_PER_ACTION (500,000 zats).
/// Below this threshold, non-SD notes are left alone — splitting them
/// would cost more in fees than the value recovered.
pub const MIN_SD: u64 = 100 * COST_PER_ACTION;

/// Maximum number of non-SD notes to split in a single transaction.
/// Caps transaction size to avoid oversized bundles that nodes reject.
pub(crate) const MAX_SPLIT_INPUTS: usize = 50;

/// Maximum migration anchor interval specified by the migration protocol.
pub const ANCHOR_BUCKET_SIZE: u32 = 144;

/// Zcash's target block spacing, used to scale the anchor interval to the
/// selected migration speed.
const TARGET_BLOCK_SPACING_MS: u64 = 75_000;

/// Fee padding embedded in each standard denomination.
/// Covers Orchard input + change (2 actions in sum mode) and Ironwood
/// output (2 actions, padded) = 4 × COST_PER_ACTION = 20,000 zats.
pub(crate) const SD_FEE_PAD: u64 = 4 * COST_PER_ACTION;

/// Decompose a total amount into standard denomination notes with embedded fees.
///
/// Each standard denomination is `10^k + P` where P = 2*COST_PER_ACTION:
/// 1_000_010_000, 100_010_000, …, 110_000.
///
/// Greedy from largest to smallest. Returns sparse `(denom, count)` pairs
/// and any leftover below the smallest denomination.
pub fn decompose_to_sd(total: u64) -> (Vec<(u64, u8)>, u64) {
    let p = SD_FEE_PAD;
    let d_min = 10u64.pow(5) + p; // 110_000
    let k_min = 5u32;
    let k_max = 16u32; // 10^16 + P covers 100M ZEC
    let mut result = Vec::new();
    let mut remainder = total;

    for k in (k_min..k_max).rev() {
        if remainder < d_min {
            break;
        }
        let d = 10u64.pow(k) + p;
        let count = (remainder / d) as u8;
        remainder %= d;
        if count > 0 {
            result.push((d, count));
        }
    }

    (result, remainder)
}

/// Check if a value is a fee-inclusive standard denomination:
/// `10^(i+5) + 2*COST_PER_ACTION` (110_000, 1_010_000, 10_010_000, …).
/// Check whether `value` is a standard denomination: exactly `10^(i+5)`, i ≥ 0.
/// Ironwood notes are minted at the pure denomination; Orchard SD notes include
/// an additional `SD_FEE_PAD`.
pub fn is_iw_sd(value: u64) -> bool {
    value >= 100_000 && value % 100_000 == 0 && {
        let mut x = value / 100_000;
        while x % 10 == 0 {
            x /= 10;
        }
        x == 1
    }
}

pub fn is_sd(value: u64) -> bool {
    value > SD_FEE_PAD && is_iw_sd(value - SD_FEE_PAD)
}

/// Scale the anchor interval to the selected migration speed.
pub(crate) fn migration_anchor_bucket_size(mean_delay_ms: u64) -> u32 {
    let blocks =
        mean_delay_ms.saturating_add(TARGET_BLOCK_SPACING_MS - 1) / TARGET_BLOCK_SPACING_MS;
    u32::try_from(blocks)
        .unwrap_or(u32::MAX)
        .clamp(1, ANCHOR_BUCKET_SIZE)
}

/// Return the first migration anchor boundary at or above `height`.
pub(crate) fn next_anchor_bucket_height(height: u32, bucket_size: u32) -> u32 {
    let remainder = height % bucket_size;
    if remainder == 0 {
        height
    } else {
        height.saturating_add(bucket_size - remainder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sd() {
        // SD_FEE_PAD = 20_000, so SD = 10^k + 20_000
        assert!(!is_sd(10_001)); // not a multiple of 10,000
        assert!(!is_sd(20_001)); // (20001-20000) % 100000 = 1 ≠ 0
        assert!(is_sd(120_000)); // 10^5 + 20_000
        assert!(is_sd(1_020_000)); // 10^6 + 20_000
        assert!(is_sd(10_020_000)); // 10^7 + 20_000
        assert!(!is_sd(1_000_000)); // missing +base
        assert!(!is_sd(120_001)); // (120001-20000) % 100000 = 1 ≠ 0
                                  // Old P=10_000 values are no longer SD
        assert!(!is_sd(110_000));
        assert!(!is_sd(1_010_000));
    }

    #[test]
    fn test_next_anchor_bucket_height() {
        assert_eq!(next_anchor_bucket_height(0, 144), 0);
        assert_eq!(next_anchor_bucket_height(1, 144), 144);
        assert_eq!(next_anchor_bucket_height(143, 144), 144);
        assert_eq!(next_anchor_bucket_height(144, 144), 144);
        assert_eq!(next_anchor_bucket_height(145, 144), 288);
        assert_eq!(next_anchor_bucket_height(145, 4), 148);
    }

    #[test]
    fn test_migration_anchor_bucket_size() {
        assert_eq!(migration_anchor_bucket_size(60_000), 1);
        assert_eq!(migration_anchor_bucket_size(900_000), 12);
        assert_eq!(migration_anchor_bucket_size(3_600_000), 48);
        assert_eq!(migration_anchor_bucket_size(10_800_000), 144);
        assert_eq!(migration_anchor_bucket_size(u64::MAX), 144);
    }

    #[test]
    fn test_decompose_below_min_denom() {
        // Below d_min (120_000).
        let (pairs, leftover) = decompose_to_sd(10_000);
        assert!(pairs.is_empty());
        assert_eq!(leftover, 10_000);
    }

    #[test]
    fn test_decompose_zero() {
        let (pairs, leftover) = decompose_to_sd(0);
        assert!(pairs.is_empty());
        assert_eq!(leftover, 0);
    }

    #[test]
    fn test_decompose_exact_sd() {
        // 120_000 = 10^5 + 20_000.
        let (pairs, leftover) = decompose_to_sd(120_000);
        assert_eq!(pairs, vec![(120_000, 1)]);
        assert_eq!(leftover, 0);
    }

    #[test]
    fn test_decompose_multiple() {
        // 4 × 120_000 = 480_000, leftover 20_000.
        let (pairs, leftover) = decompose_to_sd(500_000);
        assert_eq!(pairs, vec![(120_000, 4)]);
        assert_eq!(leftover, 20_000);
    }

    #[test]
    fn test_decompose_two_positions() {
        // 1_140_000 → 1×1_020_000 + 1×120_000.
        let (pairs, leftover) = decompose_to_sd(1_140_000);
        assert_eq!(pairs, vec![(1_020_000, 1), (120_000, 1)]);
        assert_eq!(leftover, 0);
    }

    #[test]
    fn test_decompose_with_remainder() {
        // 130_000 → 1×120_000, leftover 10_000 (below d_min).
        let (pairs, leftover) = decompose_to_sd(130_000);
        assert_eq!(pairs, vec![(120_000, 1)]);
        assert_eq!(leftover, 10_000);
    }

    /// Round-trip invariant: sum(denom × count) + leftover ≡ original total.
    #[test]
    fn test_decompose_round_trip() {
        let cases = &[0, 10_000, 120_000, 500_000, 1_140_000, 130_000, 5_000_000];
        for &total in cases {
            let (pairs, leftover) = decompose_to_sd(total);
            let represented: u64 = pairs.iter().map(|&(d, c)| d * c as u64).sum();
            assert_eq!(
                represented + leftover,
                total,
                "round-trip failed for total={total}"
            );
        }
    }
}
