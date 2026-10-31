//! Legacy groups and ZIP 2005: the recoverability flag, the upgrade that
//! creates the recoverable sibling account, and the sweep into it.
//!
//! A dkg-0 or dkg-1 group's notes are not quantum recoverable: its `rivk`
//! satisfies neither branch of the Recovery Statement. The group cannot
//! change its address in place, so the upgrade creates a second frost
//! account, version 2, from the same key package and `sk`: same `ak`, same
//! `nk`, ZIP 2005 `rivk`, hence a new address every member derives alike.
//! The sweep then moves the legacy balance to that address in the
//! Ironwood pool as an ordinary signing session. The flag says when either
//! step is still due.
//!
//! The pairing is not stored: the sibling is the dkg-2 account with the
//! same `sk`, which also pairs two accounts restored from one backup
//! string.

use anyhow::{bail, Context, Result};
use sqlx::{Connection as _, SqliteConnection};

use crate::{
    account::get_orchard_vk,
    api::{
        coin::Network,
        pay::PcztPackage,
    },
    db::{
        init_account_orchard, max_spendable, store_account_metadata, store_account_orchard_vk,
    },
    frost::{
        dkg::exec::shared_address,
        keys::{group_fvk, is_recoverable, DKG_VERSION_2},
        protocol::group_sk,
    },
    pay::{
        plan::{plan_transaction, PaymentRequest, PlanMode},
        pool::{PoolMask, ALL_SHIELDED_POOLS},
        Recipient,
    },
    Client,
};

/// Unspent, unlocked note value of an account in the Orchard and Ironwood
/// pools, dust left out: what a sweep could move.
async fn spendable_by_pool(connection: &mut SqliteConnection, account: u32) -> Result<(u64, u64)> {
    let rows: Vec<(u8, i64)> = sqlx::query_as(
        "SELECT n.pool, COALESCE(SUM(n.value), 0) FROM notes n
        LEFT JOIN spends s ON n.id_note = s.id_note
        WHERE s.id_note IS NULL AND n.account = ?1 AND n.id_asset IS NULL AND NOT n.locked
        AND n.value >= ?2
        GROUP BY n.pool",
    )
    .bind(account)
    .bind(crate::pay::fee::COST_PER_ACTION as i64)
    .fetch_all(&mut *connection)
    .await?;
    let mut orchard = 0u64;
    let mut ironwood = 0u64;
    for (pool, value) in rows {
        match pool {
            2 => orchard += value as u64,
            3 => ironwood += value as u64,
            _ => {}
        }
    }
    Ok((orchard, ironwood))
}

/// Whether the account's funds are quantum recoverable as they stand:
/// false for a legacy frost account holding any funds, and for any frost
/// account holding Orchard-pool notes. Computed from the notes, never
/// stored, so it clears on its own once the funds have moved.
pub async fn quantum_recoverable(
    connection: &mut SqliteConnection,
    account: u32,
    dkg_version: u8,
) -> Result<bool> {
    let (orchard, ironwood) = spendable_by_pool(connection, account).await?;
    if orchard > 0 {
        return Ok(false);
    }
    Ok(is_recoverable(dkg_version) || ironwood == 0)
}

/// The dkg-2 account built on the same `sk` as `account`, if any: the
/// sibling an upgrade created, or the other half of a restored pair.
pub async fn sibling_of(connection: &mut SqliteConnection, account: u32) -> Result<Option<u32>> {
    let sk: Option<(Option<Vec<u8>>,)> =
        sqlx::query_as("SELECT sk FROM dkg_params WHERE account = ?")
            .bind(account)
            .fetch_optional(&mut *connection)
            .await?;
    let Some((Some(sk),)) = sk else {
        return Ok(None);
    };
    let r: Option<(u32,)> = sqlx::query_as(
        "SELECT account FROM dkg_params
        WHERE sk = ?1 AND dkg_version >= ?2 AND account != ?3 ORDER BY account LIMIT 1",
    )
    .bind(sk)
    .bind(DKG_VERSION_2)
    .bind(account)
    .fetch_optional(&mut *connection)
    .await?;
    Ok(r.map(|(a,)| a))
}

/// Create the ZIP 2005 sibling of a legacy frost account, or return the
/// one that exists. The legacy account is left as it is. Every member
/// obtains the same address because the derivation is deterministic.
///
/// The sibling lookup and the inserts run in one immediate transaction,
/// so two upgrades of one account cannot both find no sibling: the second
/// waits for the first to commit, then returns what it created (#1517).
pub async fn upgrade_account(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<u32> {
    let (id, n, t, seed, group_name, dkg_version): (u8, u8, u8, String, String, u8) =
        sqlx::query_as("SELECT id, n, t, seed, name, dkg_version FROM dkg_params WHERE account = ?")
            .bind(account)
            .fetch_optional(&mut *connection)
            .await?
            .context("not a frost account")?;
    if is_recoverable(dkg_version) {
        bail!("this frost account already uses the ZIP 2005 derivation");
    }
    let mut tx = crate::db::begin_immediate(connection).await?;
    // Caches sk in dkg_params, which is what sibling_of pairs on.
    let sk = group_sk(network, &mut tx, account).await?;
    if let Some(sibling) = sibling_of(&mut tx, account).await? {
        tx.commit().await?;
        return Ok(sibling);
    }

    let legacy_fvk = get_orchard_vk(&mut *tx, account)
        .await?
        .context("frost account has no Orchard key")?;
    let ak: [u8; 32] = legacy_fvk.to_bytes()[..32].try_into().unwrap();
    let fvk = group_fvk(DKG_VERSION_2, &sk, &ak)?;
    let (account_name,): (String,) =
        sqlx::query_as("SELECT name FROM accounts WHERE id_account = ?")
            .bind(account)
            .fetch_one(&mut *tx)
            .await?;
    let name = format!("{account_name} (ZIP 2005)");

    let sibling =
        store_account_metadata(&mut *tx, &name, &None, &None, height, false, false).await?;
    init_account_orchard(network, &mut *tx, sibling, height).await?;
    store_account_orchard_vk(&mut *tx, sibling, &fvk).await?;
    sqlx::query(
        "INSERT INTO dkg_params(account, id, n, t, seed, birth_height, name, dkg_version, sk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(sibling)
    .bind(id)
    .bind(n)
    .bind(t)
    .bind(&seed)
    .bind(height)
    .bind(&group_name)
    .bind(DKG_VERSION_2)
    .bind(sk.to_bytes().to_vec())
    .execute(&mut *tx)
    .await?;
    // The same shares, message key, peer keys and public key package: the
    // group is the same, only its viewing key differs.
    sqlx::query(
        "INSERT INTO dkg_state(account, signing_keypair, key_pkg)
        SELECT ?1, signing_keypair, key_pkg FROM dkg_state WHERE account = ?2",
    )
    .bind(sibling)
    .bind(account)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO dkg_peers(account, round, from_id, data)
        SELECT ?1, round, from_id, data FROM dkg_peers WHERE account = ?2",
    )
    .bind(sibling)
    .bind(account)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO dkg_addresses(account, from_id, address)
        SELECT ?1, from_id, address FROM dkg_addresses WHERE account = ?2",
    )
    .bind(sibling)
    .bind(account)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(sibling)
}

/// Plan the sweep of a legacy account's whole spendable balance to its
/// sibling's address in the Ironwood pool, the fee taken from the amount.
/// The result is signed as an ordinary FROST session.
pub async fn plan_sweep(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    account: u32,
) -> Result<PcztPackage> {
    let sibling = sibling_of(connection, account)
        .await?
        .context("this frost account has no ZIP 2005 sibling yet; upgrade it first")?;
    let amount = max_spendable(connection, account, ALL_SHIELDED_POOLS).await?;
    if amount == 0 {
        bail!("nothing to sweep: the account has no spendable balance");
    }
    let address = shared_address(network, connection, sibling).await?;
    let request = PaymentRequest {
        recipients: vec![Recipient {
            address,
            amount,
            pools: Some(PoolMask::from_pool(3).0), // Ironwood
            ..Recipient::default()
        }],
        src_pools: ALL_SHIELDED_POOLS,
        recipient_pays_fee: true,
        confirmations: 0,
        category: None,
    };
    plan_transaction(network, connection, client, account, &request, &PlanMode::Standard).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::create_schema,
        frost::{
            backup::{restore_backup, FrostBackup},
            keys::DKG_VERSION_0,
        },
    };
    use orchard::keys::{FullViewingKey, SpendingKey};
    use zcash_address::unified::{Encoding as _, Fvk, Ufvk};
    use zcash_keys::keys::UnifiedFullViewingKey;

    const SK: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    const AK: &str = "262473baab7c8fbf24c39e0b2d2dd4736f7b74d7f3b6d32d1e8c62421368ad0e";

    fn bytes32(hex: &str) -> [u8; 32] {
        hex::decode(hex).unwrap().try_into().unwrap()
    }

    fn ufvk_of(fvk: &FullViewingKey) -> String {
        let ufvk = Ufvk::try_from_items(
            zcash_address::unified::Revision::R0,
            vec![zcash_address::unified::Uitem::Data(Fvk::Orchard(fvk.to_bytes()))],
        )
        .unwrap();
        UnifiedFullViewingKey::parse(&ufvk)
            .unwrap()
            .encode(&Network::Main)
            .unwrap()
    }

    /// A legacy frost account in a fresh database, through the restore path.
    async fn legacy_account(db: &mut SqliteConnection) -> (u32, SpendingKey, [u8; 32]) {
        create_schema(db).await.unwrap();
        let sk = SpendingKey::from_bytes(bytes32(SK)).unwrap();
        let ak = bytes32(AK);
        let mut share = [0u8; 32];
        share[0] = 7;
        let backup = FrostBackup {
            id: 1,
            t: 2,
            n: 3,
            share,
            sk: bytes32(SK),
        }
        .encode()
        .unwrap();
        let fvk = group_fvk(DKG_VERSION_0, &sk, &ak).unwrap();
        let restored = restore_backup(
            &Network::Main,
            db,
            "legacy",
            &backup,
            &ufvk_of(&fvk),
            &[DKG_VERSION_0],
            None,
        )
        .await
        .unwrap();
        let [(account, version)] = restored[..] else {
            panic!("one account expected");
        };
        assert_eq!(version, DKG_VERSION_0);
        (account, sk, ak)
    }

    async fn add_note(db: &mut SqliteConnection, account: u32, pool: u8, value: u64, locked: bool) {
        sqlx::query(
            "INSERT INTO notes(height, account, pool, nullifier, tx, value, locked)
            VALUES (100, ?, ?, randomblob(32), 1, ?, ?)",
        )
        .bind(account)
        .bind(pool)
        .bind(value as i64)
        .bind(locked)
        .execute(db)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn upgrade_creates_the_zip2005_sibling_once() {
        let mut db = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        let (legacy, sk, ak) = legacy_account(&mut db).await;
        assert_eq!(sibling_of(&mut db, legacy).await.unwrap(), None);

        let sibling = upgrade_account(&Network::Main, &mut db, legacy, 500).await.unwrap();
        assert_ne!(sibling, legacy);
        assert_eq!(sibling_of(&mut db, legacy).await.unwrap(), Some(sibling));
        assert_eq!(sibling_of(&mut db, sibling).await.unwrap(), None);
        // Idempotent: the same sibling again, no third account.
        assert_eq!(
            upgrade_account(&Network::Main, &mut db, legacy, 600).await.unwrap(),
            sibling
        );
        let (accounts,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM dkg_params")
            .fetch_one(&mut db)
            .await
            .unwrap();
        assert_eq!(accounts, 2);

        // The sibling's key: same ak and nk, ZIP 2005 rivk.
        let fvk = get_orchard_vk(&mut db, sibling).await.unwrap().unwrap();
        assert_eq!(fvk, group_fvk(DKG_VERSION_2, &sk, &ak).unwrap());
        let legacy_fvk = get_orchard_vk(&mut db, legacy).await.unwrap().unwrap();
        assert_eq!(fvk.to_bytes()[..64], legacy_fvk.to_bytes()[..64]);
        assert_ne!(fvk.to_bytes()[64..], legacy_fvk.to_bytes()[64..]);

        // Same parameters and shares, version 2, same sk.
        let (id, n, t, version, sk_bytes): (u8, u8, u8, u8, Vec<u8>) = sqlx::query_as(
            "SELECT id, n, t, dkg_version, sk FROM dkg_params WHERE account = ?",
        )
        .bind(sibling)
        .fetch_one(&mut db)
        .await
        .unwrap();
        assert_eq!((id, n, t, version), (1, 3, 2, DKG_VERSION_2));
        assert_eq!(sk_bytes, bytes32(SK).to_vec());
        let key_pkgs: Vec<(Vec<u8>,)> =
            sqlx::query_as("SELECT key_pkg FROM dkg_state WHERE account IN (?, ?)")
                .bind(legacy)
                .bind(sibling)
                .fetch_all(&mut db)
                .await
                .unwrap();
        assert_eq!(key_pkgs.len(), 2);
        assert_eq!(key_pkgs[0], key_pkgs[1]);

        // A dkg-2 account cannot be upgraded.
        assert!(upgrade_account(&Network::Main, &mut db, sibling, 600)
            .await
            .is_err());
    }

    /// Legacy funds are flagged until swept; Orchard-pool notes are flagged
    /// on any account; Ironwood notes on a dkg-2 account are fine. Spent
    /// and locked notes do not count.
    #[tokio::test]
    async fn recoverability_follows_the_notes() {
        let mut db = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        let (legacy, _, _) = legacy_account(&mut db).await;
        let sibling = upgrade_account(&Network::Main, &mut db, legacy, 500).await.unwrap();

        assert!(quantum_recoverable(&mut db, legacy, DKG_VERSION_0).await.unwrap());
        // Dust is nothing a sweep could move, so it does not count.
        add_note(&mut db, legacy, 3, 4_999, false).await;
        assert!(quantum_recoverable(&mut db, legacy, DKG_VERSION_0).await.unwrap());
        add_note(&mut db, legacy, 3, 10_000, false).await;
        assert!(!quantum_recoverable(&mut db, legacy, DKG_VERSION_0).await.unwrap());

        assert!(quantum_recoverable(&mut db, sibling, DKG_VERSION_2).await.unwrap());
        add_note(&mut db, sibling, 3, 10_000, false).await;
        assert!(quantum_recoverable(&mut db, sibling, DKG_VERSION_2).await.unwrap());
        add_note(&mut db, sibling, 2, 6_000, false).await;
        assert!(!quantum_recoverable(&mut db, sibling, DKG_VERSION_2).await.unwrap());

        // The Orchard note spent: recoverable again. A locked note does not
        // count either.
        sqlx::query(
            "INSERT INTO spends(id_note, height, account, pool, tx, value)
            SELECT id_note, 101, account, pool, 2, value FROM notes WHERE account = ? AND pool = 2",
        )
        .bind(sibling)
        .execute(&mut db)
        .await
        .unwrap();
        assert!(quantum_recoverable(&mut db, sibling, DKG_VERSION_2).await.unwrap());
        add_note(&mut db, sibling, 2, 6_000, true).await;
        assert!(quantum_recoverable(&mut db, sibling, DKG_VERSION_2).await.unwrap());
    }

    #[tokio::test]
    async fn the_sweep_amount_is_what_selection_can_fund() {
        let mut db = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        let (legacy, _, _) = legacy_account(&mut db).await;
        add_note(&mut db, legacy, 3, 25_000, false).await;
        add_note(&mut db, legacy, 3, 4_999, false).await; // dust: the planner drops it
        add_note(&mut db, legacy, 0, 7_000, false).await; // transparent: not in a sweep
        add_note(&mut db, legacy, 2, 9_000, true).await; // locked
        assert_eq!(
            max_spendable(&mut db, legacy, ALL_SHIELDED_POOLS).await.unwrap(),
            25_000
        );
        assert_eq!(
            max_spendable(&mut db, legacy, crate::pay::pool::ALL_POOLS).await.unwrap(),
            32_000,
            "the send page's Max counts transparent too, never dust"
        );
    }
}
