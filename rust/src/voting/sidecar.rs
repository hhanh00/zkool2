//! Async adapter over the voting crate's blocking sidecar API.
//!
//! `zcash_voting` is synchronous and owns its own rusqlite sidecar database
//! next to the wallet file. Every call therefore runs on a blocking thread:
//! the crate's HTTP transport drives its own Tokio runtime internally, and
//! `Runtime::block_on` panics inside an async task but is fine on a
//! `spawn_blocking` thread.
//!
//! zkool never lends the crate a `WalletDb`. It supplies its own notes,
//! keys and Merkle witnesses, so only the caller-supplied half of the crate's
//! API is used here.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use zcash_voting::delegate::{ensure_round_context, DelegationLwdInputs};
use zcash_voting::prelude::{
    bundle_notes_for_index_for_round, BundlePolicy, DelegationKeys, NoteInfo,
    PreparedDelegationBundle, VotingDb, VotingError, WitnessData,
};

/// One wallet's handle on the voting sidecar database.
///
/// `VotingDb` is `Send + Sync` (its connection sits behind a mutex), so the
/// handle is cheap to clone and safe to move onto blocking threads. The crate
/// keeps one connection per sidecar path, so cloning this does not open a
/// second connection.
#[derive(Clone)]
pub struct VotingSidecar {
    db: Arc<VotingDb>,
}

impl VotingSidecar {
    /// Opens the sidecar beside `wallet_db_path` and binds it to `wallet_id`.
    ///
    /// The crate derives the sidecar filename from the wallet path and runs
    /// migrations on first open, both of which touch the filesystem, so this
    /// runs off the async executor like every other call here.
    pub async fn open(wallet_db_path: PathBuf, wallet_id: String) -> Result<Self> {
        let db = tokio::task::spawn_blocking(move || {
            VotingDb::open_wallet_sidecar(&wallet_db_path, &wallet_id)
        })
        .await??;
        Ok(Self { db })
    }

    /// Runs one blocking voting-crate call on a blocking thread.
    ///
    /// This is the only way this module touches the crate: the closure takes
    /// the database by reference, so callers cannot accidentally hold the
    /// handle across an await point while the connection mutex is locked.
    pub async fn run<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&VotingDb) -> Result<T, VotingError> + Send + 'static,
        T: Send + 'static,
    {
        let db = Arc::clone(&self.db);
        Ok(tokio::task::spawn_blocking(move || f(&db)).await??)
    }

    /// Stores Merkle inclusion witnesses zkool computed from its own note
    /// commitment tree.
    ///
    /// This replaces the crate's `ensure_witnesses`/`note_witnesses` helpers,
    /// which read the wallet through `zcash_client_sqlite`. The crate still
    /// validates each witness against the stored round params, so a witness
    /// built against the wrong anchor is rejected here rather than at proof
    /// time.
    pub async fn store_witnesses(
        &self,
        round_id: String,
        bundle_index: u32,
        witnesses: Vec<WitnessData>,
    ) -> Result<()> {
        self.run(move |db| db.store_witnesses(&round_id, bundle_index, &witnesses))
            .await
    }

    /// Prepares one delegation bundle from caller-supplied notes and witnesses.
    ///
    /// Mirrors the crate's own `prepare_delegation_bundle` with the two
    /// wallet-database steps replaced: the scanned height and note selection
    /// come from zkool, and the witnesses are stored directly instead of being
    /// regenerated from a `WalletDb`.
    #[allow(clippy::too_many_arguments)]
    pub async fn prepare_delegation_bundle(
        &self,
        lwd: DelegationLwdInputs,
        session_json: Option<String>,
        round_note_infos: Vec<NoteInfo>,
        delegation_keys: DelegationKeys,
        witnesses: Vec<WitnessData>,
        bundle_index: u32,
        bundle_policy: BundlePolicy,
    ) -> Result<PreparedDelegationBundle> {
        self.run(move |db| {
            let DelegationLwdInputs {
                round_params,
                resolved_round_name,
                anchor_tree_state_bytes,
                branch_id_provider,
                network,
            } = lwd;

            ensure_round_context(
                db,
                network,
                &round_params,
                &resolved_round_name,
                session_json.as_deref(),
            )?;
            // `None`, not `session_json`: the context call above already
            // applied it, and the crate's own prepare path passes `None` here.
            db.ensure_round(network, &round_params, None)?;

            let round_id = round_params.vote_round_id.clone();
            let layout = db.ensure_bundles_with_skipped_suffix_with_policy(
                &round_id,
                &round_note_infos,
                bundle_policy,
            )?;
            let bundle_note_infos = bundle_notes_for_index_for_round(
                &round_note_infos,
                &layout,
                bundle_index,
                db,
                &round_id,
            )?;

            // Witnesses must land before any proof step reads them; the crate
            // validates them against the round params stored above.
            db.store_witnesses(&round_id, bundle_index, &witnesses)?;

            Ok(PreparedDelegationBundle {
                round_id,
                round_params,
                bundle_index,
                layout,
                bundle_note_infos,
                delegation_keys,
                branch_id_provider,
                anchor_tree_state_bytes,
                network,
                round_name: resolved_round_name,
            })
        })
        .await
    }
}
