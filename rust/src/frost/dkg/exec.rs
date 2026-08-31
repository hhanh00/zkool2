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
use orchard::keys::{FullViewingKey, Scope};
use reddsa::frost::redpallas::{
    frost::keys::PublicKeyPackage,
    keys::{dkg, EvenY},
};
use sqlx::{Connection, SqliteConnection};
use tracing::info;
use zcash_keys::address::UnifiedAddress;

use crate::{
    account::{get_account_seed, get_orchard_vk},
    api::{coin::Network, frost::DKGParams},
    db::{delete_account, init_account_orchard, store_account_metadata, store_account_orchard_vk},
    frost::{Dispatch, FrostBytes, FrostMessage, P, RouteCtx},
    Client,
};

use super::{
    delete_frost_state, get_addresses, get_coordinator_broadcast_account, get_mailbox_account,
    publish, DkgInit, DkgRound0, DkgRound1, DkgRound2, Round,
};
use super::state::{DkgState, PendingPublish};

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
    height: u32,
    round: u8,
    state: &DkgState,
) -> Result<()> {
    if !state.rounds[round as usize].secret_present {
        stage(connection, round, account, state).await?;
    }
    publish_staged(network, connection, client, account, height, state.params.id).await
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
            let (secret, outgoing) =
                DkgRound1::produce(input).context("round 1 produce failed")?;
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
            let (secret, outgoing) =
                DkgRound2::produce(input).context("round 2 produce failed")?;
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

/// Send the staged bytes, if any, and clear the marker only afterwards. A
/// crash in between retries the identical bytes; peers dedup them.
async fn publish_staged(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    account: u32,
    height: u32,
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
    publish(network, connection, account, client, height, &refs)
        .await
        .context("publish DKG package")?;
    sqlx::query("UPDATE dkg_state SET pending_publish = NULL WHERE account = ?")
        .bind(account)
        .execute(&mut *connection)
        .await?;
    Ok(())
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
    sqlx::query("UPDATE dkg_state SET key_pkg = ?1 WHERE account = ?2")
        .bind(kp.to_bytes()?)
        .bind(account)
        .execute(&mut *connection)
        .await?;
    sqlx::query(
        "INSERT INTO dkg_peers(account, round, from_id, data) VALUES(?1, 3, ?2, ?3)
        ON CONFLICT DO NOTHING",
    )
    .bind(account)
    .bind(state.params.id)
    .bind(pp.to_bytes()?)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

/// CreateFrostAccount: build the shared Orchard address by replacing the
/// spend-auth key in the broadcast account's FVK with the FROST group public
/// key, then create the frost account holding it. The creation and the
/// marker recording it commit together, so a crash cannot orphan the account
/// and re-running cannot duplicate it. Returns the shared address.
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
    let pkb = vk.serialize().expect("pk serialize");

    let fvk = get_orchard_vk(&mut *connection, broadcast_account)
        .await?
        .context("broadcast account vk not found")?;
    let mut fvkb = fvk.to_bytes();
    fvkb[0..32].copy_from_slice(&pkb);
    let shared_fvk = FullViewingKey::from_bytes(&fvkb).expect("Failed to create shared FVK");

    let (name,) = sqlx::query_as::<_, (String,)>(
        "SELECT name FROM dkg_params WHERE account = ?",
    )
    .bind(account)
    .fetch_one(&mut *connection)
    .await?;

    let mut tx = connection.begin().await?;
    let frost_account =
        store_account_metadata(&mut *tx, &name, &None, &None, height, false, false).await?;
    init_account_orchard(network, &mut *tx, frost_account, height).await?;
    store_account_orchard_vk(&mut *tx, frost_account, &shared_fvk).await?;
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
    let ua = UnifiedAddress::from_receivers(Some(address), None, None).unwrap();
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
        sqlx::query(&format!("UPDATE {table} SET account = ?1 WHERE account = ?2"))
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
