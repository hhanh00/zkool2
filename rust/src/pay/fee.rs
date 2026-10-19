pub const COST_PER_ACTION: u64 = 5000;

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

#[derive(Clone, Default, Debug)]
pub struct FeeManager {
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
        f as u64 * COST_PER_ACTION
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
            let mut fm = FeeManager::default();
            for _ in 0..n {
                fm.add_transparent_input(true);
            }
            fm.add_output(0);
            fm.fee()
        };
        assert_eq!(fee(1), 2 * COST_PER_ACTION);
        assert_eq!(fee(2), 3 * COST_PER_ACTION);
        assert_eq!(fee(5), 7 * COST_PER_ACTION);

        // Compressed-key inputs stay one action each.
        let mut fm = FeeManager::default();
        for _ in 0..3 {
            fm.add_input(0);
        }
        fm.add_output(0);
        assert_eq!(fm.fee(), 3 * COST_PER_ACTION);
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
