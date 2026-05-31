use crate::lwd::{CompactIssuance, CompactIssueNote, CompactOrchardAction, CompactSaplingOutput, CompactSaplingSpend};

/// Unified output for Orchard sync — either a regular action (from the Orchard
/// bundle) or a synthesized issuance note (from the Issue bundle). Both share
/// the same Merkle tree (pool 2), same cmx-based matching, same nullifier
/// derivation. Only `try_decrypt` differs.
#[derive(Clone)]
pub enum OrchardOutput {
    Action(CompactOrchardAction),
    Issuance {
        note: CompactIssueNote,
        ik: Vec<u8>,
        asset_desc_hash: Vec<u8>,
        asset_base: Vec<u8>,
        cmx: [u8; 32],
        /// Pre-resolved account that owns this issuance's ik (None if no
        /// matching account, in which case `try_decrypt` skips it).
        owner: Option<u32>,
    },
}

/// Preprocessed transaction — carries both the original wire-format fields
/// (for Sapling and for Orchard spend/nullifier extraction) and the merged
/// `orchard_outputs` list that interleaves Orchard actions and issuance notes.
pub struct SyncTx {
    pub hash: Vec<u8>,
    /// Sapling spends (nullifiers).
    pub spends: Vec<CompactSaplingSpend>,
    /// Sapling outputs (cmu, epk, ciphertext).
    pub sapling_outputs: Vec<CompactSaplingOutput>,
    /// Original Orchard actions — used for spend/nullifier extraction via
    /// `extract_inputs`. Outputs go through `orchard_outputs` instead.
    pub orchard_actions: Vec<CompactOrchardAction>,
    /// Merged Orchard outputs: actions first, then issuance notes. Used by
    /// `extract_outputs` — cmxs are naturally interleaved per-tx.
    pub orchard_outputs: Vec<OrchardOutput>,
    /// Asset metadata for DB storage (sent as `WarpSyncMessage::Issuance`).
    pub issuances: Vec<CompactIssuance>,
}

/// Preprocessed block — replaces `CompactBlock` as the input to the sync engine.
/// Issuance notes have been merged into `orchard_outputs` per transaction.
pub struct SyncBlock {
    pub height: u64,
    pub hash: Vec<u8>,
    pub prev_hash: Vec<u8>,
    pub time: u32,
    pub vtx: Vec<SyncTx>,
}
