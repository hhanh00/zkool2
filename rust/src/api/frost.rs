#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;
use anyhow::{Ok, Result};
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};
use sqlx::{query, sqlite::SqliteRow, Row, SqliteConnection};

use crate::{
    api::coin::Coin,
    frost::dkg::{
        get_dkg_params, get_mailbox_account, task::DkgTask,
    },
    sync::{synchronize_impl, DEFAULT_ACTIONS_PER_SYNC},
    Sink,
};
use std::str::FromStr;

use super::pay::PcztPackage;

#[cfg_attr(feature = "flutter", frb)]
pub async fn set_dkg_params(
    name: &str,
    id: u8,
    n: u8,
    t: u8,
    funding_account: u32,
    c: &Coin,
) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    crate::frost::dkg::set_dkg_params(
        &c.network(),
        &mut connection,
        &mut client,
        name,
        id,
        n,
        t,
        funding_account,
    )
    .await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn has_dkg_params(c: &Coin) -> Result<bool> {
    let mut connection = c.get_connection().await?;
    let exists =
        sqlx::query_as::<_, (String,)>("SELECT value FROM props WHERE key = 'dkg_account'")
            .fetch_optional(&mut *connection)
            .await?;
    Ok(exists.is_some())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn init_dkg(c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let account = get_funding_account(&mut *connection).await?;
    let dkg_params = get_dkg_params(&mut *connection, account).await?;
    get_mailbox_account(
        &c.network(),
        &mut *connection,
        account,
        dkg_params.id,
        dkg_params.birth_height,
    )
    .await?;

    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn has_dkg_addresses(c: &Coin) -> Result<bool> {
    let mut connection = c.get_connection().await?;
    let account = get_funding_account(&mut *connection).await?;
    let dkg_params = get_dkg_params(&mut *connection, account).await?;
    let addresses =
        crate::frost::dkg::get_addresses(&mut *connection, account, dkg_params.n).await?;
    Ok(addresses.iter().all(|a| !a.is_empty()))
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
/// Advance the DKG by one pass. Does **not** synchronize.
///
/// Like the note migration, the DKG depends on the wallet's autosync to advance
/// the funding and internal frost accounts; the step runs against whatever is
/// already synced. Syncing was moved out because it must not race the rounds and
/// autosync already drives the chain forward — the caller retries this on each
/// new block (`lib/pages/dkg.dart`), and the headless server steps from
/// `graphql::frost::new_block`, which syncs first.
///
/// Each pass executes every effect the planner names until it hits a wait —
/// one precondition-checked effect at a time. Statuses arrive after the pass,
/// one per executed task plus the wait it stopped on. A publish that cannot be
/// funded yet (previous round's change not mined) ends the pass with a
/// [`DKGStatus::WaitingForFunds`] warning rather than an error.
pub async fn do_dkg(status: StreamSink<DKGStatus>, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    // A completed or cancelled DKG must stay silent: the retry keeps firing
    // after SharedAddress until the page is closed.
    if !crate::frost::dkg::in_dkg(&mut connection).await? {
        return Ok(());
    }
    let mut client = c.client().await?;
    let height = client.latest_height().await?;
    let account = get_funding_account(&mut connection).await?;

    let outcome =
        crate::frost::dkg::step::dkg_step(&c.network(), &mut connection, &mut client, height, account)
            .await?;
    for task in &outcome.executed {
        if let Some(s) = dkg_status_for(task) {
            status.send(s).await;
        }
    }
    if outcome.waiting_for_funds {
        status.send(DKGStatus::WaitingForFunds).await;
    }
    if let Some(shared_address) = outcome.shared_address {
        status.send(DKGStatus::SharedAddress(shared_address)).await;
    }
    Ok(())
}

/// Map a task to the UI status it corresponds to. Publishes and waits map to
/// their round's variants; the three finalize stages all share `Finalize`.
fn dkg_status_for(task: &DkgTask) -> Option<DKGStatus> {
    match task {
        DkgTask::PublishRound { round: 0 } => Some(DKGStatus::PublishRound0Pkg),
        DkgTask::PublishRound { round: 1 } => Some(DKGStatus::PublishRound1Pkg),
        DkgTask::PublishRound { round: 2 } => Some(DKGStatus::PublishRound2Pkg),
        DkgTask::WaitRound { round: 0 } => Some(DKGStatus::WaitRound0Pkg),
        DkgTask::WaitRound { round: 1 } => Some(DKGStatus::WaitRound1Pkg),
        DkgTask::WaitRound { round: 2 } => Some(DKGStatus::WaitRound2Pkg),
        DkgTask::FinalizeKey | DkgTask::CreateFrostAccount | DkgTask::CompleteFinalize => {
            Some(DKGStatus::Finalize)
        }
        _ => None,
    }
}

/// Sync the funding account plus the internal frost-* accounts — the mailbox
/// and broadcast addresses the protocol messages arrive on. The funding
/// account pays for the memos. Returns the post-sync height.
///
/// `pub(crate)`, not `pub`: it takes a `&mut SqliteConnection` that cannot cross
/// the flutter_rust_bridge boundary, and only the headless GraphQL server
/// (`graphql::frost`) drives it — the app relies on its autosync instead. Hence
/// it is unused (dead) in a flutter-only build.
#[cfg_attr(not(feature = "graphql"), allow(dead_code))]
pub(crate) async fn sync_frost_accounts(
    c: &Coin,
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<u32> {
    let mut accounts =
        query("SELECT id_account FROM accounts WHERE name LIKE 'frost-%' AND internal = 1")
            .map(|r: SqliteRow| r.get::<u32, _>(0))
            .fetch_all(&mut *connection)
            .await?;
    accounts.push(account);
    synchronize_impl(
        (),
        accounts,
        height,
        DEFAULT_ACTIONS_PER_SYNC,
        1,
        100,
        false,
        c,
    )
    .await
}

pub async fn get_dkg_addresses(c: &Coin) -> Result<Vec<String>> {
    let mut connection = c.get_connection().await?;
    let account = get_funding_account(&mut connection).await?;
    let n = get_dkg_params(&mut connection, account).await?.n;
    let addresses = crate::frost::dkg::get_addresses(&mut connection, account, n).await?;
    Ok(addresses)
}

pub async fn set_dkg_address(id: u8, address: &str, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let account = get_funding_account(&mut connection).await?;
    let dkg_params = get_dkg_params(&mut connection, account).await?;
    let my_id = dkg_params.id;
    crate::frost::dkg::set_dkg_address(&mut connection, account, id, my_id, address).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn cancel_dkg(c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let account = get_funding_account(&mut connection).await?;
    crate::frost::dkg::cancel_dkg(&mut connection, account).await
}

pub(crate) async fn get_funding_account(connection: &mut SqliteConnection) -> Result<u32> {
    let (account,): (String,) = sqlx::query_as("SELECT value FROM props WHERE key = 'dkg_account'")
        .fetch_one(&mut *connection)
        .await?;
    let account = u32::from_str(&account).unwrap();
    Ok(account)
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DKGParams {
    pub id: u8,
    pub n: u8,
    pub t: u8,
    pub birth_height: u32,
}

#[derive(Clone, Debug)]
pub enum DKGStatus {
    WaitParams,
    WaitAddresses(Vec<String>),
    /// Round 0 exchanges participant signing keys before the FROST rounds
    /// proper; it needs its own status so the UI does not report it as round 1.
    PublishRound0Pkg,
    WaitRound0Pkg,
    PublishRound1Pkg,
    WaitRound1Pkg,
    PublishRound2Pkg,
    WaitRound2Pkg,
    /// A round's package could not be funded yet: the note spent for the
    /// previous round is locked and its change is not mined. Transient — the
    /// next block retries. Shown as an info/warning, not an error.
    WaitingForFunds,
    Finalize,
    SharedAddress(String),
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn reset_sign(c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::frost::sign::reset_sign(&mut *connection).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn init_sign(
    coordinator: u8,
    funding_account: u32,
    pczt: &PcztPackage,
    c: &Coin,
) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::frost::sign::init_sign(
        &mut *connection,
        c.account,
        funding_account,
        coordinator,
        pczt,
    )
    .await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn is_signing_in_progress(c: &Coin) -> Result<bool> {
    let mut connection = c.get_connection().await?;
    crate::frost::sign::is_signing_in_progress(&mut *connection).await
}

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
/// Advance the signing rounds by one step. Does **not** synchronize.
///
/// Like [`do_dkg`], signing depends on the wallet's autosync to advance the
/// funding and internal frost accounts; the step runs against what is already
/// synced and the caller retries on each new block. A signing message that
/// cannot be funded yet ends the step with a [`SigningStatus::WaitingForFunds`]
/// warning rather than an error.
pub async fn do_sign(status: StreamSink<SigningStatus>, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    let height = client.latest_height().await?;

    let r = crate::frost::sign::do_sign(
        &c.network(),
        &mut *connection,
        &mut client,
        height,
        status.clone(),
    )
    .await;
    if let Err(e) = r {
        let _ = status.add_error(e);
    }
    Ok(())
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrostSignParams {
    pub account: u32,
    pub coordinator: u8,
    pub funding_account: u32,
}

#[derive(Clone, Debug)]
pub enum SigningStatus {
    SendingCommitment,
    WaitingForCommitments,
    SendingSigningPackage,
    WaitingForSigningPackage,
    SendingSignatureShare,
    SigningCompleted,
    WaitingForSignatureShares,
    /// A signing message could not be funded yet (previous broadcast's change
    /// not mined). Transient — the next block retries. Info/warning, not error.
    WaitingForFunds,
    PreparingTransaction,
    SendingTransaction,
    TransactionSent(String),
}
