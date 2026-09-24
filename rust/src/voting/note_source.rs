//! Account-bound wallet facts for the synchronous delegation pipeline.

use anyhow::{anyhow, ensure, Context as _, Result};
use sqlx::{Row as _, SqliteConnection};
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_voting::{
    delegation_pipeline::{DelegationNote, DelegationNoteSource},
    types::{NoteInfo, VotingError, WitnessData},
};

use crate::api::coin::Network as WalletNetwork;
use crate::warp::hasher::{empty_roots, OrchardHasher};

/// Immutable notes and witnesses loaded for one account and snapshot.
///
/// Construct a fresh source after switching accounts. No wallet or voting DB
/// handles are retained, and the callback performs no I/O or eligibility policy.
pub struct ZkoolNoteSource {
    account: u32,
    snapshot_height: u64,
    notes: Vec<DelegationNote>,
}

impl ZkoolNoteSource {
    /// Resolve wallet facts before entering the blocking voting pipeline.
    pub async fn load(
        network: &WalletNetwork,
        connection: &mut SqliteConnection,
        client: &mut crate::Client,
        account: u32,
        snapshot_height: u64,
    ) -> Result<Self, VotingError> {
        let height = u32::try_from(snapshot_height).map_err(|_| VotingError::InvalidInput {
            message: format!("snapshot height {snapshot_height} does not fit u32"),
        })?;
        let notes = load_notes(network, connection, client, account, height)
            .await
            .map_err(|error| VotingError::Storage {
                message: format!("load voting notes for account {account}: {error:#}"),
            })?;
        Ok(Self {
            account,
            snapshot_height,
            notes,
        })
    }

    pub fn account(&self) -> u32 {
        self.account
    }
}

impl DelegationNoteSource for ZkoolNoteSource {
    fn notes_and_witnesses(
        &self,
        snapshot_height: u64,
    ) -> Result<Vec<DelegationNote>, VotingError> {
        if snapshot_height != self.snapshot_height {
            return Err(VotingError::InvalidInput {
                message: format!(
                    "note source for account {} is bound to snapshot {}, requested {snapshot_height}",
                    self.account, self.snapshot_height,
                ),
            });
        }
        Ok(self.notes.clone())
    }
}

async fn load_notes(
    network: &WalletNetwork,
    connection: &mut SqliteConnection,
    client: &mut crate::Client,
    account: u32,
    snapshot_height: u32,
) -> Result<Vec<DelegationNote>> {
    // Sync guard: the wallet must be synced through the round snapshot height.
    let h = crate::sync::get_db_height(connection, account).await?;
    ensure!(
        h.height >= snapshot_height,
        "wallet is not synced to the round snapshot height {snapshot_height} (current {})",
        h.height
    );

    // Select unspent, unlocked Ironwood (pool 3) ZEC notes that existed at
    // the snapshot height: notes created
    // after the snapshot sit past the anchor edge and cannot be rewound.
    let notes = unspent_ironwood_notes(connection, account, snapshot_height).await?;
    if notes.is_empty() {
        return Ok(Vec::new());
    }

    // The finalized snapshot determines the witness root.
    let (_, _, ironwood_frontier) =
        crate::sync::get_tree_state(network, client, snapshot_height).await?;
    let edge = ironwood_frontier.to_edge(&OrchardHasher::default());
    let ufvk = unified_full_viewing_key(network, connection, account).await?;
    load_snapshot_notes(network, connection, notes, h.height, &ufvk, &edge).await
}

/// Build paired notes using the current wallet witnesses and the snapshot edge.
/// Kept separate from the tree-state fetch so historical rewinds can be tested
/// against an in-memory wallet without a lightwalletd server.
async fn load_snapshot_notes(
    network: &WalletNetwork,
    connection: &mut SqliteConnection,
    notes: Vec<(u32, u8)>,
    witness_height: u32,
    ufvk: &UnifiedFullViewingKey,
    edge: &crate::warp::Edge,
) -> Result<Vec<DelegationNote>> {
    let anchor_root = edge.root(&OrchardHasher::default());
    let edge = edge.to_auth_path(&OrchardHasher::default());
    let ero = empty_roots(&OrchardHasher::default());
    let ovk = ufvk
        .orchard()
        .ok_or_else(|| anyhow!("account has no Orchard viewing key"))?;

    let mut paired = Vec::with_capacity(notes.len());
    for (id, scope) in notes {
        let (note, merkle_path) = crate::account::get_orchard_note(
            connection,
            id,
            witness_height,
            ovk,
            &edge,
            &ero,
            orchard::NoteVersion::V3,
            Some(edge.1),
        )
        .await
        .with_context(|| format!("load Ironwood note {id}"))?;
        let position = merkle_path.position() as u64;
        let scope = match scope {
            0 => orchard::keys::Scope::External,
            1 => orchard::keys::Scope::Internal,
            _ => return Err(anyhow!("unexpected note scope {scope}")),
        };
        let info = NoteInfo::from_orchard_note(&note, position, scope, ufvk, network)?;
        let auth_path = merkle_path
            .auth_path()
            .iter()
            .map(|sibling| sibling.to_bytes().to_vec())
            .collect();
        ensure!(
            merkle_path.root(note.commitment().into()).to_bytes() == anchor_root,
            "Ironwood note {id} witness does not match snapshot root"
        );
        let witness = WitnessData {
            note_commitment: info.commitment.clone(),
            position,
            root: anchor_root.to_vec(),
            auth_path,
        };
        paired.push(DelegationNote { info, witness });
    }

    Ok(paired)
}

/// Unspent, unlocked Ironwood (pool 3) ZEC notes that existed at or before
/// `snapshot_height`, as `(note_id, scope)`.
async fn unspent_ironwood_notes(
    connection: &mut SqliteConnection,
    account: u32,
    snapshot_height: u32,
) -> Result<Vec<(u32, u8)>> {
    let notes = sqlx::query(
        "SELECT a.id_note, a.scope
         FROM notes a
         LEFT JOIN spends b ON a.id_note = b.id_note
         LEFT JOIN assets ast ON a.id_asset = ast.id_asset
         WHERE b.id_note IS NULL AND a.account = ?
           AND a.pool = 3 AND a.locked = 0
           AND a.height <= ?
           AND COALESCE(ast.asset_base, X'0000000000000000000000000000000000000000000000000000000000000000') = X'0000000000000000000000000000000000000000000000000000000000000000'
         ORDER BY a.id_note",
    )
    .bind(account)
    .bind(snapshot_height)
    .map(|row: sqlx::sqlite::SqliteRow| {
        let id: u32 = row.get(0);
        let scope: Option<u8> = row.get(1);
        (id, scope.unwrap_or(0))
    })
    .fetch_all(&mut *connection)
    .await?;

    Ok(notes)
}

async fn unified_full_viewing_key(
    network: &WalletNetwork,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<UnifiedFullViewingKey> {
    let encoded = crate::key::get_account_ufvk(network, connection, account, 4).await?;
    UnifiedFullViewingKey::decode(network, &encoded)
        .map_err(|e| anyhow!("invalid account UFVK: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Connection as _;

    #[tokio::test]
    async fn selection_is_account_bound_and_does_not_filter_small_notes() {
        let mut db = SqliteConnection::connect(":memory:").await.unwrap();
        sqlx::raw_sql(
            "CREATE TABLE notes (
                id_note INTEGER, scope INTEGER, account INTEGER, pool INTEGER,
                locked INTEGER, height INTEGER, id_asset INTEGER, value INTEGER);
             CREATE TABLE spends (id_note INTEGER);
             CREATE TABLE assets (id_asset INTEGER, asset_base BLOB);
             INSERT INTO assets VALUES (1, zeroblob(32)), (2, X'01');
             INSERT INTO notes VALUES
                (1, NULL, 7, 3, 0, 100, NULL, 1),
                (2, 1,    7, 3, 0, 100, 1, 100000000),
                (3, 0,    8, 3, 0, 100, NULL, 1),
                (4, 0,    7, 3, 0, 101, NULL, 1),
                (5, 0,    7, 2, 0, 100, NULL, 1),
                (6, 0,    7, 3, 1, 100, NULL, 1),
                (7, 0,    7, 3, 0, 100, 2, 1),
                (8, 0,    7, 3, 0, 100, NULL, 1);
             INSERT INTO spends VALUES (8);",
        )
        .execute(&mut db)
        .await
        .unwrap();
        assert_eq!(
            unspent_ironwood_notes(&mut db, 7, 100).await.unwrap(),
            vec![(1, 0), (2, 1)]
        );
        assert_eq!(
            unspent_ironwood_notes(&mut db, 8, 100).await.unwrap(),
            vec![(3, 0)]
        );
        assert!(unspent_ironwood_notes(&mut db, 7, 99)
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn rewinds_tip_witness_to_snapshot_root() {
        use crate::warp::{Edge, Hasher as _, Witness};
        use orchard::{
            keys::Scope,
            note::{AssetBase, RandomSeed, Rho},
            value::NoteValue,
            Note, NoteVersion,
        };
        use zcash_keys::keys::UnifiedSpendingKey;

        let network = WalletNetwork::Test;
        let usk = UnifiedSpendingKey::from_seed(
            &network,
            &[0x42; 32],
            zip32::AccountId::try_from(0).unwrap(),
        )
        .unwrap();
        let ufvk = usk.to_unified_full_viewing_key();
        let fvk = ufvk.orchard().unwrap();
        let recipient = fvk.address_at(0u32, Scope::External);
        let rho = Rho::from_bytes(&[9; 32]).unwrap();
        let rseed = RandomSeed::from_bytes([7; 32], &rho).unwrap();
        let note = Note::from_parts(
            recipient,
            NoteValue::from_raw(1),
            AssetBase::zatoshi(),
            rho,
            rseed,
            NoteVersion::V3,
        )
        .unwrap();
        let cmx = orchard::note::ExtractedNoteCommitment::from(note.commitment()).to_bytes();
        let hasher = OrchardHasher::default();
        // The note is leaf 0. The snapshot contains three leaves; the tip
        // contains eight, so two right sibling subtrees must be rewound.
        let leaves = [
            cmx, [1; 32], [2; 32], [3; 32], [4; 32], [5; 32], [6; 32], [7; 32],
        ];
        let mut snapshot = Edge::default();
        for leaf in &leaves[..3] {
            snapshot.append(&hasher, *leaf);
        }
        let mut tip = Edge::default();
        for leaf in leaves {
            tip.append(&hasher, leaf);
        }
        let snapshot_root = snapshot.root(&hasher);
        let tip_root = tip.root(&hasher);
        assert_ne!(snapshot_root, tip_root);
        let mut witness = Witness {
            value: cmx,
            position: 0,
            ommers: Edge::default(),
            anchor: tip_root,
        };
        witness.ommers.0[0] = Some(leaves[1]);
        witness.ommers.0[1] = Some(hasher.combine(0, &leaves[2], &leaves[3]));
        witness.ommers.0[2] = Some(hasher.combine(
            1,
            &hasher.combine(0, &leaves[4], &leaves[5]),
            &hasher.combine(0, &leaves[6], &leaves[7]),
        ));
        let empty = empty_roots(&hasher);
        assert_eq!(
            witness
                .build_auth_path(&tip.to_auth_path(&hasher), &empty)
                .unwrap()
                .root(0, &cmx, &hasher),
            tip_root,
        );
        // This fixture actually needs a rewind: merely substituting the
        // snapshot edge cannot produce an authentication path.
        assert!(witness
            .build_auth_path(&snapshot.to_auth_path(&hasher), &empty)
            .is_err());

        let mut db = SqliteConnection::connect(":memory:").await.unwrap();
        sqlx::raw_sql(
            "CREATE TABLE notes (
                id_note INTEGER, scope INTEGER, position INTEGER, diversifier BLOB,
                value INTEGER, rcm BLOB, rho BLOB, id_asset INTEGER);
             CREATE TABLE witnesses (note INTEGER, height INTEGER, witness BLOB);
             CREATE TABLE assets (id_asset INTEGER, asset_base BLOB);",
        )
        .execute(&mut db)
        .await
        .unwrap();
        sqlx::query("INSERT INTO notes VALUES (1, 0, 0, ?, 1, ?, ?, NULL)")
            .bind(recipient.diversifier().as_array().as_slice())
            .bind(rseed.as_bytes().as_slice())
            .bind(rho.to_bytes().as_slice())
            .execute(&mut db)
            .await
            .unwrap();
        let encoded = bincode::encode_to_vec(&witness, bincode::config::legacy()).unwrap();
        // Only a tip-height witness exists, never a snapshot-height witness.
        sqlx::query("INSERT INTO witnesses VALUES (1, 110, ?)")
            .bind(&encoded)
            .execute(&mut db)
            .await
            .unwrap();
        let notes = load_snapshot_notes(&network, &mut db, vec![(1, 0)], 110, &ufvk, &snapshot)
            .await
            .unwrap();
        let source = ZkoolNoteSource {
            account: 7,
            snapshot_height: 100,
            notes,
        };
        let paired = source.notes_and_witnesses(100).unwrap();
        assert_eq!(paired.len(), 1);
        let pair = &paired[0];
        assert_eq!(pair.info.commitment, cmx);
        assert_eq!(pair.witness.note_commitment, cmx);
        assert_eq!(pair.witness.position, 0);
        assert_eq!(pair.witness.root, snapshot_root);
        assert_ne!(pair.witness.root, tip_root);
        let auth_path = pair
            .witness
            .auth_path
            .iter()
            .map(|sibling| {
                orchard::tree::MerkleHashOrchard::from_bytes(&sibling.clone().try_into().unwrap())
                    .unwrap()
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let path = orchard::tree::MerklePath::from_parts(0, auth_path);
        assert_eq!(
            path.root(note.commitment().into()).to_bytes(),
            snapshot_root
        );
        let stored: Vec<u8> = sqlx::query_scalar("SELECT witness FROM witnesses WHERE note = 1")
            .fetch_one(&mut db)
            .await
            .unwrap();
        assert_eq!(
            stored, encoded,
            "rewinding must not mutate the stored tip witness"
        );
    }

    #[tokio::test]
    async fn missing_or_corrupt_witness_returns_error() {
        let mut db = SqliteConnection::connect(":memory:").await.unwrap();
        sqlx::raw_sql(
            "CREATE TABLE notes (
                id_note INTEGER, scope INTEGER, position INTEGER, diversifier BLOB,
                value INTEGER, rcm BLOB, rho BLOB, id_asset INTEGER);
             CREATE TABLE witnesses (note INTEGER, height INTEGER, witness BLOB);
             CREATE TABLE assets (id_asset INTEGER, asset_base BLOB);
             INSERT INTO notes VALUES (1, 0, 0, zeroblob(11), 1, zeroblob(32), zeroblob(32), NULL);",
        ).execute(&mut db).await.unwrap();
        let sk = orchard::keys::SpendingKey::from_bytes([7; 32]).unwrap();
        let fvk = orchard::keys::FullViewingKey::from(&sk);
        let empty = empty_roots(&OrchardHasher::default());
        let edge = crate::warp::FragmentAuthPath(empty_roots(&OrchardHasher::default()), 0);
        let missing = crate::account::get_orchard_note(
            &mut db,
            1,
            100,
            &fvk,
            &edge,
            &empty,
            orchard::NoteVersion::V3,
            Some(0),
        )
        .await
        .err()
        .expect("missing witness must fail");
        assert!(format!("{missing:#}").contains("retrieve oinput"));
        sqlx::query("INSERT INTO witnesses VALUES (1, 100, X'FF')")
            .execute(&mut db)
            .await
            .unwrap();
        let corrupt = crate::account::get_orchard_note(
            &mut db,
            1,
            100,
            &fvk,
            &edge,
            &empty,
            orchard::NoteVersion::V3,
            Some(0),
        )
        .await
        .err()
        .expect("corrupt witness must fail");
        assert!(format!("{corrupt:#}").contains("decode Orchard witness"));
    }

    #[test]
    fn callback_rejects_other_snapshots_and_returns_owned_pairs() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ZkoolNoteSource>();
        let source = ZkoolNoteSource {
            account: 7,
            snapshot_height: 100,
            notes: vec![DelegationNote {
                info: NoteInfo {
                    commitment: vec![1; 32],
                    nullifier: vec![2; 32],
                    value: 1,
                    position: 3,
                    diversifier: vec![0; 11],
                    rho: vec![0; 32],
                    rseed: vec![0; 32],
                    scope: 0,
                    ufvk_str: String::new(),
                },
                witness: WitnessData {
                    note_commitment: vec![1; 32],
                    position: 3,
                    root: vec![4; 32],
                    auth_path: vec![vec![0; 32]; 32],
                },
            }],
        };
        assert_eq!(source.account(), 7);
        assert!(source.notes_and_witnesses(99).is_err());
        assert!(source.notes_and_witnesses(101).is_err());
        let mut first = source.notes_and_witnesses(100).unwrap();
        first[0].witness.root.clear();
        let second = source.notes_and_witnesses(100).unwrap();
        assert_eq!(second[0].witness.root, vec![4; 32]);
        assert_eq!(second[0].info.commitment, second[0].witness.note_commitment);
        assert_eq!(second[0].info.position, second[0].witness.position);
        assert_eq!(second[0].info.value, 1);
    }
}
