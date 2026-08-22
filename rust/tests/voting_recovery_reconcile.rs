//! Recovery-reconciliation tests for the voting fork: recording chain
//! confirmations whose evidence came from a commitment-tree scan instead of
//! tx events (no tx hash), and locating leaves in the commitment tree.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use vote_commitment_tree::MemoryTreeServer;
use zcash_voting::prelude::{
    delegation_van_commitment, find_leaf_position_with_api, record_delegation_confirmation_from_tree,
    record_van_position, record_vote_confirmation_from_tree, DelegationPhase, Network, RoundParams,
    VotePhase, VotingDb,
};

const ROUND_ID: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const WALLET_ID: &str = "wallet-reconcile";

fn round_params() -> RoundParams {
    RoundParams {
        vote_round_id: ROUND_ID.to_string(),
        snapshot_height: 100,
        ea_pk: vec![0xEA_u8; 32],
        nc_root: vec![0xAA_u8; 32],
        nullifier_imt_root: vec![0xBB_u8; 32],
    }
}

async fn test_db() -> (sqlx::SqlitePool, VotingDb) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().in_memory(true))
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let db = VotingDb::from_pool(pool.clone(), &mut conn).await.unwrap();
    drop(conn);
    db.set_wallet_id(WALLET_ID);
    db.create_round(Network::Testnet, &round_params(), None)
        .await
        .unwrap();
    (pool, db)
}

async fn insert_bundle(pool: &sqlx::SqlitePool, bundle_index: u32, gov_comm: Option<Vec<u8>>) {
    sqlx::query(
        "INSERT INTO voting_bundles (round_id, wallet_id, bundle_index, address_index, total_note_value, gov_comm)
         VALUES (?, ?, ?, 0, 100, ?)",
    )
    .bind(ROUND_ID)
    .bind(WALLET_ID)
    .bind(bundle_index as i64)
    .bind(gov_comm)
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_vote(
    pool: &sqlx::SqlitePool,
    bundle_index: u32,
    proposal_id: u32,
    with_recovery: bool,
) {
    sqlx::query(
        "INSERT INTO voting_votes (round_id, wallet_id, bundle_index, proposal_id, choice, commitment, created_at, commitment_bundle_json)
         VALUES (?, ?, ?, ?, 2, NULL, 1, ?)",
    )
    .bind(ROUND_ID)
    .bind(WALLET_ID)
    .bind(bundle_index as i64)
    .bind(proposal_id as i64)
    .bind(if with_recovery { Some(recovery_json()) } else { None })
    .execute(pool)
    .await
    .unwrap();
}

/// Mirrors the wire format produced by the fork's `serialize_recovery` for a
/// committed vote; the recovery JSON round-trips through `parse_recovery`.
fn recovery_json() -> String {
    serde_json::to_string(&serde_json::json!({
        "format": "zcash_voting_vote_recovery_v1",
        "vote_round_id": ROUND_ID,
        "bundle_index": 0,
        "proposal_id": 1,
        "vote_decision": 2,
        "anchor_height": 100,
        "vc_tree_position": 0,
        "single_share": false,
        "num_options": 3,
        "van_nullifier": vec![0x31_u8; 32],
        "vote_authority_note_new": vec![0x32_u8; 32],
        "vote_commitment": vec![0x33_u8; 32],
        "proof": vec![0x34_u8; 8],
        "shares_hash": vec![0x35_u8; 32],
        "r_vpk": vec![0x36_u8; 32],
        "alpha_v": vec![0x37_u8; 32],
        "vote_auth_sig": vec![0x38_u8; 64],
        "encrypted_shares": [],
        "share_blinds": [],
        "share_comms": [],
    }))
    .unwrap()
}

fn fp(x: u64) -> pasta_curves::Fp {
    pasta_curves::Fp::from(x)
}

#[tokio::test]
async fn tree_confirmation_records_vote_without_tx_hash() {
    let (pool, db) = test_db().await;
    insert_bundle(&pool, 0, None).await;
    insert_vote(&pool, 0, 1, true).await;

    let result = record_vote_confirmation_from_tree(&db, ROUND_ID, 0, 1, 42, Some(41))
        .await
        .unwrap();
    assert_eq!(result.vc_tree_position, 42);
    assert_eq!(result.van_leaf_position, Some(41));

    let mut conn = pool.acquire().await.unwrap();
    // Phase derivation reports Confirmed without a tx hash; the bundle's VAN
    // pointer advanced to the vote's VAN output position.
    assert_eq!(
        db.vote_phase(&mut conn, ROUND_ID, 0, 1).await.unwrap(),
        VotePhase::Confirmed
    );
    assert_eq!(
        db.delegation_phase(&mut conn, ROUND_ID, 0).await.unwrap(),
        DelegationPhase::Confirmed
    );
}

#[tokio::test]
async fn tree_confirmation_rejects_missing_recovery_bundle() {
    let (pool, db) = test_db().await;
    insert_bundle(&pool, 0, None).await;
    insert_vote(&pool, 0, 1, false).await;

    let err = record_vote_confirmation_from_tree(&db, ROUND_ID, 0, 1, 42, None)
        .await
        .unwrap_err();
    assert!(matches!(err, zcash_voting::VotingError::InvalidInput { .. }), "got {err:?}");
}

#[tokio::test]
async fn tree_confirmation_replay_is_idempotent_and_conflict_checked() {
    let (pool, db) = test_db().await;
    insert_bundle(&pool, 0, None).await;
    insert_vote(&pool, 0, 1, true).await;

    record_vote_confirmation_from_tree(&db, ROUND_ID, 0, 1, 42, Some(41))
        .await
        .unwrap();
    // Replay with the same evidence is accepted.
    record_vote_confirmation_from_tree(&db, ROUND_ID, 0, 1, 42, Some(41))
        .await
        .unwrap();
    // A different VC position conflicts with the recorded one.
    let err = record_vote_confirmation_from_tree(&db, ROUND_ID, 0, 1, 43, Some(42))
        .await
        .unwrap_err();
    assert!(matches!(err, zcash_voting::VotingError::InvalidInput { .. }), "got {err:?}");
}

#[tokio::test]
async fn delegation_tree_confirmation_stores_van_position_and_never_rewinds() {
    let (pool, db) = test_db().await;
    insert_bundle(&pool, 0, None).await;

    record_delegation_confirmation_from_tree(&db, ROUND_ID, 0, 7)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(
        db.delegation_phase(&mut conn, ROUND_ID, 0).await.unwrap(),
        DelegationPhase::Confirmed
    );

    // A later vote confirmation advanced the pointer past the delegation
    // position; recovery must not rewind it.
    drop(conn);
    record_van_position(&db, ROUND_ID, 0, 9).await.unwrap();
    record_delegation_confirmation_from_tree(&db, ROUND_ID, 0, 7)
        .await
        .unwrap();
    let position: Option<i64> = sqlx::query_scalar(
        "SELECT van_leaf_position FROM voting_bundles
         WHERE round_id = ? AND wallet_id = ? AND bundle_index = 0",
    )
    .bind(ROUND_ID)
    .bind(WALLET_ID)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(position, Some(9));
}

#[tokio::test]
async fn delegation_van_commitment_reads_persisted_gov_comm() {
    use ff::PrimeField as _;
    let (pool, db) = test_db().await;
    let gov_comm = fp(77);
    insert_bundle(&pool, 0, Some(gov_comm.to_repr().to_vec())).await;

    let recovered = delegation_van_commitment(&db, ROUND_ID, 0)
        .await
        .unwrap()
        .expect("bundle gov_comm must load");
    assert_eq!(recovered, gov_comm);
}

#[tokio::test]
async fn find_leaf_position_locates_leaf_in_server_pages() {
    let mut server = MemoryTreeServer::empty();
    // One delegation leaf, then a cast-vote pair (VAN output, then vote
    // commitment) — mirroring the chain's append order.
    server.append(fp(10)).unwrap();
    server.checkpoint(1).unwrap();
    server.append_two(fp(20), fp(21)).unwrap();
    server.checkpoint(2).unwrap();

    assert_eq!(
        find_leaf_position_with_api(&server, fp(10), 4).await.unwrap(),
        Some(0)
    );
    assert_eq!(
        find_leaf_position_with_api(&server, fp(21), 4).await.unwrap(),
        Some(2)
    );
    assert_eq!(
        find_leaf_position_with_api(&server, fp(99), 4).await.unwrap(),
        None
    );
}

#[tokio::test]
async fn find_leaf_position_returns_none_on_empty_tree() {
    let server = MemoryTreeServer::empty();
    assert_eq!(
        find_leaf_position_with_api(&server, fp(1), 4).await.unwrap(),
        None
    );
}

#[tokio::test]
async fn migration_v13_upgrade_adds_confirmed_without_hash_column() {
    let (pool, _db) = test_db().await;
    // Simulate a v13 wallet DB: drop the v14 column and rewind the version.
    sqlx::query("ALTER TABLE voting_votes DROP COLUMN confirmed_without_hash")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE voting_schema_version SET version = 13")
        .execute(&pool)
        .await
        .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let db = VotingDb::from_pool(pool.clone(), &mut conn).await.unwrap();
    drop(conn);
    db.set_wallet_id(WALLET_ID);

    // The upgrade must restore the column and phase derivation end to end.
    insert_bundle(&pool, 0, None).await;
    insert_vote(&pool, 0, 1, true).await;
    record_vote_confirmation_from_tree(&db, ROUND_ID, 0, 1, 42, Some(41))
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(
        db.vote_phase(&mut conn, ROUND_ID, 0, 1).await.unwrap(),
        VotePhase::Confirmed
    );
    drop(conn);
    let version: i64 = sqlx::query_scalar("SELECT MAX(version) FROM voting_schema_version")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(version, 14);
}
