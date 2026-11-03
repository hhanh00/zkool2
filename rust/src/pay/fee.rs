use anyhow::Result;
use sqlx::SqliteConnection;
use zcash_primitives::transaction::fees::zip317::{
    FeeRule, GRACE_ACTIONS, P2PKH_STANDARD_INPUT_SIZE, P2PKH_STANDARD_OUTPUT_SIZE,
};
use zcash_protocol::value::Zatoshis;

use crate::api::coin::Network;

/// ZIP 317 marginal fee before ZIP 2009: 5000 zatoshis per logical action.
pub const LEGACY_COST_PER_ACTION: u64 = 5000;

/// ZIP 2009 marginal fee: 1000 zatoshis per logical action.
pub const REDUCED_COST_PER_ACTION: u64 = 1000;

/// Mainnet height from which wallets adopt the ZIP 2009 fee
/// (`FEE_REDUCTION_HEIGHT` in ZIP 317 revision 2).
pub const MAINNET_FEE_REDUCTION_HEIGHT: u32 = 3_590_000;

/// Testnet height from which wallets adopt the ZIP 2009 fee: the Testnet
/// NU7 activation height.
pub const TESTNET_FEE_REDUCTION_HEIGHT: u32 = 4_465_026;

/// Wallet property holding the user's fee-per-action override, in zatoshis.
/// Absent, empty or zero means automatic.
pub const FEE_OVERRIDE_PROP: &str = "fee_per_action";

/// The ZIP 317 marginal fee, in zatoshis per logical action, for a
/// transaction expected to land at `target_height`.
///
/// `override_fee` wins when set: it is the user's setting, and later the
/// value a node reports. Otherwise ZIP 2009's schedule applies: Mainnet and
/// Testnet switch from 5000 to 1000 at their reduction heights, regtest is
/// 1000 at every height like zebra's test-network rule, and the ZSA regtest
/// stays at 5000 because its node fork predates the reduction.
///
/// This is the single place the fee per action is decided: the planner,
/// the builder's fee rule, the dust floors and the migration denominations
/// all read it, so they cannot disagree.
pub fn cost_per_action(network: &Network, target_height: u32, override_fee: Option<u64>) -> u64 {
    if let Some(fee) = override_fee.filter(|fee| *fee > 0) {
        return fee;
    }
    let reduction_height = match network {
        Network::Main => MAINNET_FEE_REDUCTION_HEIGHT,
        Network::Test => TESTNET_FEE_REDUCTION_HEIGHT,
        Network::Regtest(_) => 0,
        Network::ZsaRegtest(_) => return LEGACY_COST_PER_ACTION,
    };
    if target_height >= reduction_height {
        REDUCED_COST_PER_ACTION
    } else {
        LEGACY_COST_PER_ACTION
    }
}

/// The user's fee-per-action override from the wallet properties, if any.
pub async fn fee_override(connection: &mut SqliteConnection) -> Result<Option<u64>> {
    Ok(crate::db::get_prop(connection, FEE_OVERRIDE_PROP)
        .await?
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|fee| *fee > 0))
}

/// [`cost_per_action`] for `target_height`, honouring the wallet's override.
pub async fn resolve_cost_per_action(
    network: &Network,
    connection: &mut SqliteConnection,
    target_height: u32,
) -> Result<u64> {
    let override_fee = fee_override(connection).await?;
    Ok(cost_per_action(network, target_height, override_fee))
}

/// The fee rule the builder checks a plan against: ZIP 317 with the marginal
/// fee the plan was priced at. The builder requires the plan's fee to match
/// this rule exactly, so it must be built from the same `cost_per_action`.
pub fn fee_rule(cost_per_action: u64) -> FeeRule {
    FeeRule::non_standard(
        Zatoshis::const_from_u64(cost_per_action),
        GRACE_ACTIONS,
        P2PKH_STANDARD_INPUT_SIZE,
        P2PKH_STANDARD_OUTPUT_SIZE,
    )
    .expect("standard P2PKH sizes are non-zero")
}

/// ZIP 317 size of a P2PKH input spending a compressed-key coin: the standard
/// size that both the planner and the builder price such an input at.
pub const P2PKH_INPUT_SIZE: u64 = 150;

/// ZIP 317 size of a P2PKH input spending an uncompressed-key coin: its exact
/// size with a maximal signature (36 prevout + 1 + 74 signature push + 66 key
/// push + 4 sequence), as the builder prices it. Only 31 more than the
/// standard size because that one rounds the compressed input's 149 bytes up.
pub const P2PKH_UNCOMPRESSED_INPUT_SIZE: u64 = 181;

/// Whether a P2PKH `scriptSig` reveals an uncompressed (65-byte) public key.
pub fn script_sig_pushes_uncompressed_key(script_sig: &zcash_transparent::address::Script) -> bool {
    use zcash_script::{opcode::PossiblyBad, Opcode};
    script_sig.0.parse().last().is_some_and(
        |op| matches!(op, Ok(PossiblyBad::Good(Opcode::PushValue(pv))) if pv.value().len() == 65),
    )
}

/// ZIP-317 creation cost per new asset, in equivalent logical actions.
/// Zebra currently uses 0; this matches the upstream `FeeRule::standard()`
/// constant but may diverge on networks where zebra configures it differently.
pub const CREATION_COST: u64 = 0;

#[derive(Clone, Debug)]
pub struct FeeManager {
    /// Marginal fee, in zatoshis per logical action: see [`cost_per_action`].
    pub cost_per_action: u64,
    pub num_inputs: [u8; 4],
    pub num_outputs: [u8; 4],
    /// Total ZIP 317 size of the transparent inputs, in bytes. ZIP 317 prices
    /// transparent inputs by size (`ceil(t_in_size / 150)` actions), not by
    /// count: a coin held by an uncompressed key takes a bigger `scriptSig`.
    pub t_in_size: u64,
    /// Logical actions contributed by the issue bundle (ZIP-317).
    /// Equal to `total_issue_note_count + CREATION_COST * asset_creation_count`.
    pub issuance_actions: u64,
    /// In migration mode (O→O self-sends), the Orchard V3 bundle disables
    /// cross-address transfers, so actions are `spends + outputs` instead
    /// of `max(spends, outputs)`.
    pub migration: bool,
}

impl FeeManager {
    /// An empty fee manager pricing actions at `cost_per_action`.
    pub fn new(cost_per_action: u64) -> Self {
        Self {
            cost_per_action,
            num_inputs: [0; 4],
            num_outputs: [0; 4],
            t_in_size: 0,
            issuance_actions: 0,
            migration: false,
        }
    }

    // Add an input; a transparent one spends a compressed-key coin.
    pub fn add_input(&mut self, pool: u8) {
        if pool == 0 {
            self.add_transparent_input(false);
        } else {
            self.num_inputs[pool as usize] += 1;
        }
    }

    // Add a transparent input, priced by the key encoding its coin commits to.
    pub fn add_transparent_input(&mut self, uncompressed: bool) {
        self.num_inputs[0] += 1;
        self.t_in_size += if uncompressed {
            P2PKH_UNCOMPRESSED_INPUT_SIZE
        } else {
            P2PKH_INPUT_SIZE
        };
    }

    // Add an output
    pub fn add_output(&mut self, pool: u8) {
        self.num_outputs[pool as usize] += 1;
    }

    // Remove an output
    pub fn remove_output(&mut self, pool: u8) {
        self.num_outputs[pool as usize] -= 1;
    }

    /// Add logical actions from the issue bundle.
    /// `issue_note_count` — total number of issue notes across all issue actions.
    /// `asset_creation_count` — number of new assets being created (first issuance).
    pub fn add_issuance_actions(&mut self, issue_note_count: u64, asset_creation_count: u64) {
        self.issuance_actions += issue_note_count + CREATION_COST * asset_creation_count;
    }

    // Return the current amount of fees
    pub fn fee(&self) -> u64 {
        let t = self
            .t_in_size
            .div_ceil(P2PKH_INPUT_SIZE)
            .max(self.num_outputs[0] as u64);
        let s = if self.num_inputs[1] > 0 || self.num_outputs[1] > 0 {
            // if any sapling, # bundle outputs = max(2, # outputs)
            // if any input, # bundle inputs = max(1, # inputs)
            // # logical sapling = max(# bundle in, bundle out) =
            // max(2, # inputs, # outputs)
            // I O -> BI BO -> L
            // 0 0 -> 0  0  -> 0
            // 1 0 -> 1  2  -> 2
            // 0 1 -> 0  2  -> 2
            // 1 1 -> 1  2  -> 2
            // 2 1 -> 2  1  -> 2
            // etc.
            //
            // basically it is max(# inputs, # outputs, 2) unless there
            // is no input or output
            self.num_inputs[1].max(self.num_outputs[1]).max(2)
        } else {
            0
        };
        let o = if self.num_inputs[2] > 0 || self.num_outputs[2] > 0 {
            if self.migration {
                (self.num_inputs[2] as u64 + self.num_outputs[2] as u64).max(2)
            } else {
                self.num_inputs[2].max(self.num_outputs[2]).max(2) as u64
            }
        } else {
            0
        };
        let i = if self.num_inputs[3] > 0 || self.num_outputs[3] > 0 {
            // Ironwood has same padding as Orchard: min 2 actions
            self.num_inputs[3].max(self.num_outputs[3]).max(2)
        } else {
            0
        };
        let f = (t + s as u64 + o + i as u64).max(2); // minimum 2 logical actions
                                                             // Issuance actions are counted by the builder as orchard actions,
                                                             // so we don't add them separately here. The issuance counts are
                                                             // informational for logging only.
        f as u64 * self.cost_per_action
    }

    #[allow(dead_code)]
    fn min_actions_padding(a: u8) -> u8 {
        if a == 0 {
            0
        } else {
            a.max(2)
        }
    }
}

impl std::fmt::Display for FeeManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fee: {}:{} {}:{} {}:{} {}:{} issuance:{}",
            self.num_inputs[0],
            self.num_outputs[0],
            self.num_inputs[1],
            self.num_outputs[1],
            self.num_inputs[2],
            self.num_outputs[2],
            self.num_inputs[3],
            self.num_outputs[3],
            self.issuance_actions,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::{PublicKey, SecretKey};
    use zcash_primitives::transaction::fees::transparent::{InputSize, InputView};
    use zcash_protocol::value::Zatoshis;
    use zcash_script::script::Code;
    use zcash_transparent::{
        address::{Script, TransparentAddress},
        builder::{SpendInfo, TransparentInputInfo},
        bundle::{OutPoint, TxOut},
        util::hash160,
    };

    /// The sizes the planner prices transparent inputs at must be the ones
    /// the builder prices them at: `build_for_pczt` checks the fee the plan
    /// paid against its own and refuses any difference as missing change.
    #[test]
    fn input_sizes_match_the_builder() {
        let sk = SecretKey::from_secret_bytes([1; 32]).unwrap();
        let pk = PublicKey::from_secret_key(&sk);
        let size = |pubkey_bytes: &[u8]| {
            let addr = TransparentAddress::PublicKeyHash(hash160::hash(pubkey_bytes));
            let coin = TxOut::new(Zatoshis::const_from_u64(10_000), addr.script().into());
            let input = TransparentInputInfo::from_parts(
                OutPoint::new([0; 32], 0),
                coin,
                SpendInfo::P2pkh { pubkey: pk },
            )
            .unwrap();
            match InputView::serialized_size(&input) {
                InputSize::Known(n) => n as u64,
                other => panic!("{other:?}"),
            }
        };
        assert_eq!(size(&pk.serialize()), P2PKH_INPUT_SIZE);
        assert_eq!(
            size(&pk.serialize_uncompressed()),
            P2PKH_UNCOMPRESSED_INPUT_SIZE
        );
    }

    /// ZIP 317 sums the input sizes before rounding: one uncompressed-key
    /// input costs two transparent actions, two cost three, five cost seven.
    #[test]
    fn uncompressed_inputs_are_priced_by_size() {
        let fee = |n: usize| {
            let mut fm = FeeManager::new(LEGACY_COST_PER_ACTION);
            for _ in 0..n {
                fm.add_transparent_input(true);
            }
            fm.add_output(0);
            fm.fee()
        };
        assert_eq!(fee(1), 2 * LEGACY_COST_PER_ACTION);
        assert_eq!(fee(2), 3 * LEGACY_COST_PER_ACTION);
        assert_eq!(fee(5), 7 * LEGACY_COST_PER_ACTION);

        // Compressed-key inputs stay one action each.
        let mut fm = FeeManager::new(LEGACY_COST_PER_ACTION);
        for _ in 0..3 {
            fm.add_input(0);
        }
        fm.add_output(0);
        assert_eq!(fm.fee(), 3 * LEGACY_COST_PER_ACTION);
    }

    /// The fee scales with the marginal fee: the same shape costs one fifth
    /// after ZIP 2009.
    #[test]
    fn fee_follows_the_marginal_fee() {
        let fee = |cost| {
            let mut fm = FeeManager::new(cost);
            fm.add_input(2);
            fm.add_output(2);
            fm.add_output(2);
            fm.fee()
        };
        // One Orchard spend and two outputs pad to two actions.
        assert_eq!(fee(LEGACY_COST_PER_ACTION), 10_000);
        assert_eq!(fee(REDUCED_COST_PER_ACTION), 2_000);
    }

    /// ZIP 2009: Mainnet and Testnet switch at their reduction heights,
    /// regtest is always reduced, the ZSA regtest never is, and a positive
    /// override wins everywhere.
    #[test]
    fn cost_per_action_follows_zip_2009() {
        let regtest = crate::api::coin::network_for_coin(2);
        let zsa = crate::api::coin::network_for_coin(3);
        let cases: &[(&Network, u32, u64)] = &[
            (&Network::Main, MAINNET_FEE_REDUCTION_HEIGHT - 1, LEGACY_COST_PER_ACTION),
            (&Network::Main, MAINNET_FEE_REDUCTION_HEIGHT, REDUCED_COST_PER_ACTION),
            (&Network::Test, TESTNET_FEE_REDUCTION_HEIGHT - 1, LEGACY_COST_PER_ACTION),
            (&Network::Test, TESTNET_FEE_REDUCTION_HEIGHT, REDUCED_COST_PER_ACTION),
            (&regtest, 0, REDUCED_COST_PER_ACTION),
            (&zsa, u32::MAX, LEGACY_COST_PER_ACTION),
        ];
        for &(network, height, expected) in cases {
            assert_eq!(cost_per_action(network, height, None), expected, "{network:?} @ {height}");
            assert_eq!(cost_per_action(network, height, Some(0)), expected, "zero override is automatic");
            assert_eq!(cost_per_action(network, height, Some(777)), 777, "override wins");
        }
    }

    /// The builder's rule prices a plan exactly as the fee manager does, at
    /// either marginal fee.
    #[test]
    fn fee_rule_matches_the_fee_manager() {
        use zcash_primitives::transaction::fees::FeeRule as _;
        for cost in [LEGACY_COST_PER_ACTION, REDUCED_COST_PER_ACTION] {
            let rule = fee_rule(cost);
            let required = rule
                .fee_required(
                    &Network::Main,
                    zcash_protocol::consensus::BlockHeight::from_u32(1),
                    std::iter::empty::<InputSize>(),
                    std::iter::empty::<usize>(),
                    0,
                    0,
                    2,
                    0,
                )
                .unwrap();
            let mut fm = FeeManager::new(cost);
            fm.add_input(2);
            fm.add_output(2);
            fm.add_output(2);
            assert_eq!(required.into_u64(), fm.fee());
        }
    }

    #[test]
    fn script_sig_reveals_the_key_encoding() {
        let script = |key_len: usize| {
            let sig = vec![0u8; 71];
            let mut bytes = vec![sig.len() as u8];
            bytes.extend(&sig);
            bytes.push(key_len as u8);
            bytes.extend(vec![2u8; key_len]);
            Script(Code(bytes))
        };
        assert!(!script_sig_pushes_uncompressed_key(&script(33)));
        assert!(script_sig_pushes_uncompressed_key(&script(65)));
    }
}
