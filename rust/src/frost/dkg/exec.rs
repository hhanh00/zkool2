//! Task execution: the side effects, and nothing else.
//!
//! Every decision has already been made by the time control reaches here — a
//! task arrives naming exactly what to do, and these functions carry it out.
//! The crypto (`dkg::part1/2/3`, the address derivations) is called, never
//! reimplemented, here.
//!
//! The one invariant worth calling out is intent before publish: producing a
//! round commits the secret **and** the outgoing bytes in one transaction
//! with no network I/O, and only then does the broadcast go out. A crash
//! after the send but before the marker is cleared retries the identical
//! bytes — every peer's `dkg_peers` primary key dedups them — so the
//! produce-then-publish window that could once wedge a session now costs at
//! most one wasted fee.

use anyhow::{bail, Context, Result};
use orchard::keys::Scope;
use reddsa_legacy::frost::redpallas::{
    frost::keys::PublicKeyPackage,
    keys::{dkg, EvenY},
};
use sqlx::{Connection, SqliteConnection};
use tracing::info;
use zcash_keys::address::UnifiedAddress;

use crate::{
    account::{get_account_seed, get_orchard_vk},
    api::{coin::Network, frost::DKGParams},
    db::{
        delete_account, init_account_orchard, select_account_orchard, store_account_metadata,
        store_account_orchard_vk,
    },
    frost::{
        keys::{group_fvk, is_recoverable},
        Dispatch, FrostBytes, FrostMessage, RouteCtx, P,
    },
    Client,
};

use super::state::{DkgState, PendingPublish};
use super::{
    delete_frost_state, get_addresses, get_coordinator_broadcast_account, get_mailbox_account,
    DkgInit, DkgRound0, DkgRound1, DkgRound2, Round,
};

/// EnsureAccounts: create the private mailbox and the shared broadcast
/// account if missing. Both helpers are create-if-missing and idempotent.
pub async fn ensure_accounts(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    params: &DKGParams,
) -> Result<()> {
    // The broadcast account's seed is a hash over every participant address;
    // creating it with addresses missing would derive a seed other
    // participants never find. The planner gates this behind WaitAddresses,
    // but the guard costs one query and makes the effect safe on its own.
    let addresses = get_addresses(connection, account, params.n).await?;
    anyhow::ensure!(
        addresses.iter().all(|a| !a.is_empty()),
        "participant addresses incomplete; cannot create the broadcast account"
    );
    get_mailbox_account(network, connection, account, params.id, params.birth_height).await?;
    get_coordinator_broadcast_account(network, connection, account, params.birth_height).await?;
    Ok(())
}

/// PublishRound: stage our outgoing package if the round's secret is not
/// stored yet (secret + staged bytes commit together, before any network
/// I/O), then send whatever is staged and clear the marker.
pub async fn publish_round(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    account: u32,
    round: u8,
    state: &DkgState,
) -> Result<()> {
    if !state.rounds[round as usize].secret_present {
        stage(connection, round, account, state).await?;
    }
    publish_staged(
        network,
        connection,
        client,
        account,
        state.params.id,
    )
    .await
}

/// Produce the round's package and commit it — secret plus outgoing bytes —
/// in one transaction. No network I/O here.
async fn stage(
    connection: &mut SqliteConnection,
    round: u8,
    account: u32,
    state: &DkgState,
) -> Result<()> {
    let broadcast_address = state
        .broadcast_address
        .clone()
        .context("broadcast account missing")?;
    let route_ctx = RouteCtx {
        broadcast_address: broadcast_address.clone(),
        coordinator_address: broadcast_address, // unused in DKG
        peer_addresses: state.addresses.clone(),
    };

    match round {
        0 => {
            let input = DkgInit {
                self_id: state.params.id,
                n: state.params.n,
                t: state.params.t,
                dkg_version: state.dkg_version,
            };
            let (secret, outgoing) =
                DkgRound0::produce(&input).context("round 0 produce failed")?;
            let recipients = outgoing.into_recipients(&route_ctx)?;
            let mut tx = connection.begin().await?;
            <DkgRound0 as Round>::store_secret(&mut *tx, account, &secret).await?;
            store_pending(&mut *tx, account, round, recipients).await?;
            tx.commit().await?;
        }
        1 => {
            let input = state
                .state0
                .as_ref()
                .context("round 1 input not reconstructed")?;
            let (secret, outgoing) = DkgRound1::produce(input).context("round 1 produce failed")?;
            let recipients = outgoing.into_recipients(&route_ctx)?;
            let mut tx = connection.begin().await?;
            <DkgRound1 as Round>::store_secret(&mut *tx, account, &secret).await?;
            store_pending(&mut *tx, account, round, recipients).await?;
            tx.commit().await?;
        }
        2 => {
            let input = state
                .state1
                .as_ref()
                .context("round 2 input not reconstructed")?;
            let (secret, outgoing) = DkgRound2::produce(input).context("round 2 produce failed")?;
            let recipients = outgoing.into_recipients(&route_ctx)?;
            let mut tx = connection.begin().await?;
            <DkgRound2 as Round>::store_secret(&mut *tx, account, &secret).await?;
            store_pending(&mut *tx, account, round, recipients).await?;
            tx.commit().await?;
        }
        _ => bail!("invalid round {round}"),
    }
    Ok(())
}

async fn store_pending(
    tx: &mut sqlx::SqliteConnection,
    account: u32,
    round: u8,
    recipients: Vec<(String, Vec<u8>)>,
) -> Result<()> {
    let pending = PendingPublish { round, recipients };
    sqlx::query("UPDATE dkg_state SET pending_publish = ?1 WHERE account = ?2")
        .bind(pending.encode()?)
        .bind(account)
        .execute(&mut *tx)
        .await?;
    Ok(())
}

/// Plan, sign and stage the transaction for the staged bytes, if any, then
/// send it. The signed transaction, its marker and the note locks commit
/// together with the clearing of the staged bytes, before the broadcast
/// (#1565): a crash in between is repaired by the resend in the publish
/// wait, and a message that could not be funded keeps its bytes staged for
/// the next pass.
async fn publish_staged(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    account: u32,
    self_id: u8,
) -> Result<()> {
    let pending = load_pending(connection, account).await?;
    let Some(pending) = pending else {
        return Ok(());
    };
    let prefix = match pending.round {
        0 => <DkgRound0 as Round>::PREFIX,
        1 => <DkgRound1 as Round>::PREFIX,
        2 => <DkgRound2 as Round>::PREFIX,
        _ => bail!("invalid round {}", pending.round),
    };
    let refs: Vec<(&str, Vec<u8>)> = pending
        .recipients
        .iter()
        .map(|(addr, data)| {
            let msg = FrostMessage {
                from_id: self_id,
                data: data.clone(),
            };
            Ok((addr.as_str(), msg.encode_with_prefix(&prefix)?))
        })
        .collect::<Result<_>>()?;
    let mut tx = connection.begin().await?;
    let staged = crate::frost::protocol::stage_publish(
        network,
        &mut tx,
        account,
        client,
        &refs,
        crate::frost::protocol::DKG_LAST_TXID_PROP,
    )
    .await
    .context("publish DKG package")?;
    sqlx::query("UPDATE dkg_state SET pending_publish = NULL WHERE account = ?")
        .bind(account)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    crate::frost::protocol::send_staged(client, &staged).await?;
    Ok(())
}

/// The wait on our own DKG message, as in signing (#1565): resend it, or
/// give it up past its expiry so the round is published afresh.
pub async fn own_publish(
    connection: &mut SqliteConnection,
    client: &mut Client,
    funding_account: u32,
    height: u32,
) -> Result<bool> {
    if crate::frost::protocol::publish_expired(
        connection,
        crate::frost::protocol::DKG_LAST_TXID_PROP,
        funding_account,
        height,
    )
    .await?
    {
        return Ok(true);
    }
    crate::frost::protocol::resend_publish(
        connection,
        client,
        crate::frost::protocol::DKG_LAST_TXID_PROP,
    )
    .await?;
    Ok(false)
}

async fn load_pending(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<PendingPublish>> {
    sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT pending_publish FROM dkg_state WHERE account = ? AND pending_publish IS NOT NULL",
    )
    .bind(account)
    .fetch_optional(&mut *connection)
    .await?
    .map(|(b,)| PendingPublish::decode(&b))
    .transpose()
}

/// FinalizeKey: derive our key package and the group public key package from
/// the completed rounds. Idempotent via the stored key package.
pub async fn finalize_key(
    connection: &mut SqliteConnection,
    account: u32,
    state: &DkgState,
) -> Result<()> {
    if state.key_pkg_present {
        return Ok(());
    }
    let state2 = state
        .state2
        .as_ref()
        .context("round 2 not complete; cannot finalize")?;
    info!(
        "DKG: calling dkg::part3 (self_id={}, n={}, t={})",
        state.params.id, state.params.n, state.params.t
    );
    let (kp, pp) = dkg::part3(&state2.spkg2, &state2.state1.ppkg1s, &state2.ppkg2s)?;
    info!("DKG: dkg::part3 completed successfully");
    // One transaction, as the other stages: a key package without its
    // round-3 row sent the planner to CreateFrostAccount, which failed on
    // the missing row on every pass (#1503).
    let mut tx = connection.begin().await?;
    sqlx::query("UPDATE dkg_state SET key_pkg = ?1 WHERE account = ?2")
        .bind(kp.to_bytes()?)
        .bind(account)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO dkg_peers(account, round, from_id, data) VALUES(?1, 3, ?2, ?3)
        ON CONFLICT DO NOTHING",
    )
    .bind(account)
    .bind(state.params.id)
    .bind(pp.to_bytes()?)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// CreateFrostAccount: build the group Orchard viewing key from the broadcast
/// account's spending key and the FROST group public key, under the
/// account's DKG version (`frost::keys::group_fvk`), then create the frost
/// account holding it. A dkg-2 account also records `sk`, which its `rivk`
/// derives from. The creation and the marker recording it commit together,
/// so a crash cannot orphan the account and re-running cannot duplicate it.
/// Returns the shared address.
pub async fn create_frost_account(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    broadcast_account: u32,
    height: u32,
) -> Result<String> {
    let (pp_data,) = sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT data FROM dkg_peers WHERE account = ? AND round = 3 LIMIT 1",
    )
    .bind(account)
    .fetch_one(&mut *connection)
    .await?;
    let pub_key_pkg = PublicKeyPackage::<P>::from_bytes(&pp_data)?;
    let pub_key_pkg = pub_key_pkg.into_even_y(None);
    let vk = pub_key_pkg.verifying_key();
    let ak: [u8; 32] = vk
        .serialize()
        .expect("pk serialize")
        .try_into()
        .map_err(|_| anyhow::anyhow!("group verifying key is not 32 bytes"))?;

    let sk = select_account_orchard(&mut *connection, broadcast_account)
        .await?
        .xsk
        .context("broadcast account spending key not found")?;

    let (name, dkg_version) = sqlx::query_as::<_, (String, u8)>(
        "SELECT name, dkg_version FROM dkg_params WHERE account = ?",
    )
    .bind(account)
    .fetch_one(&mut *connection)
    .await?;
    let shared_fvk = group_fvk(dkg_version, &sk, &ak)?;

    let mut tx = connection.begin().await?;
    let frost_account =
        store_account_metadata(&mut *tx, &name, &None, &None, height, false, false).await?;
    init_account_orchard(network, &mut *tx, frost_account, height).await?;
    store_account_orchard_vk(&mut *tx, frost_account, &shared_fvk).await?;
    if is_recoverable(dkg_version) {
        sqlx::query("UPDATE dkg_params SET sk = ?1 WHERE account = ?2")
            .bind(sk.to_bytes().to_vec())
            .bind(account)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("INSERT OR REPLACE INTO props(key, value) VALUES ('dkg_frost_account', ?1)")
        .bind(frost_account.to_string())
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    info!("Frost account {frost_account} created with the shared key");

    shared_address(network, connection, frost_account).await
}

/// The shared address, derived from the frost account's stored group FVK.
pub async fn shared_address(
    network: &Network,
    connection: &mut SqliteConnection,
    frost_account: u32,
) -> Result<String> {
    let fvk = get_orchard_vk(&mut *connection, frost_account)
        .await?
        .context("frost account vk not found")?;
    let address = fvk.address_at(0u64, Scope::External);
    let ua = UnifiedAddress::from_receivers(Some(address), None, None, None, None).unwrap();
    Ok(ua.encode(network))
}

/// CompleteFinalize: rekey the protocol rows onto the frost account, record
/// the mailbox seed, tear down the helper accounts, and only then clear the
/// props that stop the drivers — a crash mid-teardown must leave the drivers
/// still willing to step, so the remaining operations re-run idempotently.
pub async fn complete_finalize(
    connection: &mut SqliteConnection,
    funding_account: u32,
    frost_account: u32,
    mailbox: Option<u32>,
    broadcast: Option<u32>,
) -> Result<()> {
    // 1. Rekey the protocol rows (no-ops when already done).
    for table in ["dkg_params", "dkg_state", "dkg_peers", "dkg_addresses"] {
        // Audited: `table` is one of the four string literals above.
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "UPDATE {table} SET account = ?1 WHERE account = ?2"
        )))
        .bind(frost_account)
        .bind(funding_account)
        .execute(&mut *connection)
        .await?;
    }
    // 2. The mailbox seed under the frost account, while the mailbox exists.
    if let Some(mailbox) = mailbox {
        let seed = get_account_seed(&mut *connection, mailbox)
            .await?
            .context("mailbox seed not found")?
            .mnemonic;
        sqlx::query("UPDATE dkg_params SET seed = ?1 WHERE account = ?2")
            .bind(seed)
            .bind(frost_account)
            .execute(&mut *connection)
            .await?;
    }
    // 3. Tear down the helper accounts.
    if let Some(mailbox) = mailbox {
        delete_account(&mut *connection, mailbox).await?;
    }
    if let Some(broadcast) = broadcast {
        delete_account(&mut *connection, broadcast).await?;
    }
    // 4. Cleanup LAST: the props this deletes are what stop the drivers.
    delete_frost_state(&mut *connection).await
}

#[cfg(test)]
mod finalize_tests {
    use super::*;
    use crate::db::create_schema;
    use crate::frost::dkg::{DkgInit, DkgState0, DkgState1, DkgState2};
    use ed25519_dalek::SigningKey;
    use rand_core_legacy::OsRng;
    use reddsa_legacy::frost::redpallas::Identifier;
    use std::collections::BTreeMap;

    /// Participant 1 of a 2-of-2 DKG after round 2, as observe rebuilds it.
    fn after_round_2() -> DkgState {
        let id1: Identifier = 1u16.try_into().unwrap();
        let id2: Identifier = 2u16.try_into().unwrap();
        let (s1, p1) = dkg::part1(id1, 2, 2, OsRng).unwrap();
        let (s2, p2) = dkg::part1(id2, 2, 2, OsRng).unwrap();
        let ppkg1s = BTreeMap::from([(id2, p2)]);
        let (spkg2, _) = dkg::part2(s1.clone(), &ppkg1s).unwrap();
        let (_, from_2) = dkg::part2(s2, &BTreeMap::from([(id1, p1)])).unwrap();
        let ppkg2s = BTreeMap::from([(id2, from_2[&id1].clone())]);
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let init = DkgInit { self_id: 1, n: 2, t: 2, dkg_version: 2 };
        let state0 = DkgState0 {
            init,
            verifying_key: signing_key.verifying_key(),
            signing_key,
            peer_verifying_keys: BTreeMap::new(),
        };
        let state1 = DkgState1 { state0, spkg1: s1, ppkg1s };
        let round = super::super::state::RoundState { secret_present: true, others: 1, pending: None };
        DkgState {
            funding_account: 1,
            params: DKGParams { id: 1, n: 2, t: 2, birth_height: 1 },
            addresses: vec!["a".into(), "b".into()],
            mailbox: None,
            broadcast: None,
            broadcast_address: None,
            rounds: [round; 3],
            key_pkg_present: false,
            frost_account: None,
            rekeyed: false,
            dkg_version: 2,
            version_mismatch: None,
            publish_unmined: false,
            state0: None,
            state1: None,
            state2: Some(DkgState2 { state1, spkg2, ppkg2s }),
        }
    }

    async fn wallet() -> SqliteConnection {
        let mut db = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        create_schema(&mut db).await.unwrap();
        sqlx::query("INSERT INTO dkg_state(account) VALUES (1)")
            .execute(&mut db)
            .await
            .unwrap();
        db
    }

    /// (key package stored, round-3 rows).
    async fn written(db: &mut SqliteConnection) -> (bool, i64) {
        let (kp,): (bool,) =
            sqlx::query_as("SELECT key_pkg IS NOT NULL FROM dkg_state WHERE account = 1")
                .fetch_one(&mut *db)
                .await
                .unwrap();
        let (n,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM dkg_peers WHERE account = 1 AND round = 3")
                .fetch_one(&mut *db)
                .await
                .unwrap();
        (kp, n)
    }

    /// The key package and its round-3 row commit together: a refused
    /// row leaves no key package behind to send the planner past
    /// FinalizeKey (#1503).
    #[tokio::test]
    async fn the_key_package_and_its_round_3_row_commit_together() {
        let mut db = wallet().await;
        let state = after_round_2();
        sqlx::query(
            "CREATE TRIGGER refuse_round_3 BEFORE INSERT ON dkg_peers WHEN NEW.round = 3
            BEGIN SELECT RAISE(ABORT, 'refused'); END",
        )
        .execute(&mut db)
        .await
        .unwrap();
        assert!(finalize_key(&mut db, 1, &state).await.is_err());
        assert_eq!(written(&mut db).await, (false, 0), "neither write survived");

        sqlx::query("DROP TRIGGER refuse_round_3").execute(&mut db).await.unwrap();
        finalize_key(&mut db, 1, &state).await.unwrap();
        assert_eq!(written(&mut db).await, (true, 1));

        let mut done = state.clone();
        done.key_pkg_present = true;
        finalize_key(&mut db, 1, &done).await.unwrap();
        assert_eq!(written(&mut db).await, (true, 1), "a second call is a no-op");
    }
}
