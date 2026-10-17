#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;
use anyhow::{Ok, Result};
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};
use sqlx::{query, sqlite::SqliteRow, Row, SqliteConnection};

use crate::{
    api::coin::Coin,
    frost::dkg::{get_dkg_params, get_mailbox_account},
    frost::keys::CURRENT_DKG_VERSION,
    sync::{synchronize_impl, DEFAULT_ACTIONS_PER_SYNC},
};
use crate::api::sync::SyncOutcome;
use crate::frost::driver::Driver;
use std::str::FromStr;
use std::time::Duration;

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
    set_dkg_params_versioned(name, id, n, t, funding_account, CURRENT_DKG_VERSION, c).await
}

/// Start a DKG under an explicit generation. The app always runs the
/// current one; the GraphQL server exposes the choice so that tests can
/// form a legacy group and exercise its upgrade.
pub async fn set_dkg_params_versioned(
    name: &str,
    id: u8,
    n: u8,
    t: u8,
    funding_account: u32,
    dkg_version: u8,
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
        dkg_version,
    )
    .await
}

/// Create the ZIP 2005 sibling of a legacy frost account (or return the
/// existing one). The legacy account is unchanged; the sibling has the
/// same shares and a new, quantum-recoverable address.
#[cfg_attr(feature = "flutter", frb)]
pub async fn upgrade_frost_account(account: u32, c: &Coin) -> Result<u32> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    let height = client.latest_height().await?;
    crate::frost::upgrade::upgrade_account(&c.network(), &mut connection, account, height).await
}

/// Plan the sweep of a legacy frost account's spendable balance to its
/// sibling in the Ironwood pool. The package is signed through the usual
/// FROST signing flow (`init_sign`).
#[cfg_attr(feature = "flutter", frb)]
pub async fn plan_frost_sweep(account: u32, c: &Coin) -> Result<PcztPackage> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    crate::frost::upgrade::plan_sweep(&c.network(), &mut connection, &mut client, account).await
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

/// Sync the funding account plus the internal frost-* accounts — the mailbox
/// and broadcast addresses the protocol messages arrive on. The funding
/// account pays for the memos. Returns the post-sync height.
///
/// `pub(crate)`, not `pub`: it takes a `&mut SqliteConnection` that cannot cross
/// the flutter_rust_bridge boundary; the protocol driver (`frost::driver`)
/// is its caller on both the app and the server.
///
/// Returns `None` when the sync did not run because the wallet's sync
/// session was busy or was cancelled: a step planned from stale notes can
/// double-spend, so the caller must not step on it and tries again later.
pub(crate) async fn sync_frost_accounts(
    c: &Coin,
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<Option<u32>> {
    let mut accounts =
        query("SELECT id_account FROM accounts WHERE name LIKE 'frost-%' AND internal = 1")
            .map(|r: SqliteRow| r.get::<u32, _>(0))
            .fetch_all(&mut *connection)
            .await?;
    accounts.push(account);
    let outcome = synchronize_impl(
        (),
        accounts,
        height,
        DEFAULT_ACTIONS_PER_SYNC,
        1,
        100,
        false,
        c,
    )
    .await?;
    Ok(match outcome {
        SyncOutcome::Synced { height } => Some(height),
        SyncOutcome::Skipped | SyncOutcome::Cancelled => None,
    })
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
    /// Terminal: participant `peer` runs DKG version `theirs` while this
    /// wallet runs `ours`. The group would not derive one address, so the
    /// DKG stops; every device must run the same build and start again.
    VersionMismatch { peer: u8, ours: u8, theirs: u8 },
}

/// The member's backup string for a frost account: one `zkfrost` bech32m
/// string holding the participant id, t, n, the signing share and the
/// group's sk. As sensitive as a seed phrase. For a legacy (dkg-0, dkg-1)
/// account it is a device-loss backup only.
#[cfg_attr(feature = "flutter", frb)]
pub async fn export_frost_backup(account: u32, c: &Coin) -> Result<String> {
    let mut connection = c.get_connection().await?;
    crate::frost::backup::export_backup(&c.network(), &mut connection, account).await
}

/// A frost account restored from a backup string.
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrostRestore {
    pub account: u32,
    /// 0 for a legacy group, 2 for a ZIP 2005 one.
    pub dkg_version: u8,
}

/// The generation a backup string and a group viewing key match: 0 for a
/// legacy key, 2 for a ZIP 2005 one. Fails when the string is not the
/// group's. What a restore form should tick by default.
#[cfg_attr(feature = "flutter", frb)]
pub fn match_frost_backup(backup: &str, ufvk: &str, c: &Coin) -> Result<u8> {
    crate::frost::backup::match_backup(&c.network(), backup, ufvk)
}

/// Restore frost accounts from a backup string and the group's unified
/// full viewing key: the legacy account, the ZIP 2005 one, or both, since
/// the string's sk derives both from the key's ak. No peer addresses are
/// needed; a fresh mailbox is created at the first signing session. Born
/// at `birth`, or at Orchard activation when not given.
#[cfg_attr(feature = "flutter", frb)]
pub async fn restore_frost_backup(
    name: &str,
    backup: &str,
    ufvk: &str,
    legacy: bool,
    zip2005: bool,
    birth: Option<u32>,
    c: &Coin,
) -> Result<Vec<FrostRestore>> {
    let mut connection = c.get_connection().await?;
    let mut versions = vec![];
    if legacy {
        versions.push(crate::frost::keys::DKG_VERSION_0);
    }
    if zip2005 {
        versions.push(crate::frost::keys::DKG_VERSION_2);
    }
    let restored = crate::frost::backup::restore_backup(
        &c.network(),
        &mut connection,
        name,
        backup,
        ufvk,
        &versions,
        birth,
    )
    .await?;
    Ok(restored
        .into_iter()
        .map(|(account, dkg_version)| FrostRestore {
            account,
            dkg_version,
        })
        .collect())
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
    account: u32,
    c: &Coin,
) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::frost::sign::init_sign(
        &mut *connection,
        account,
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


#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrostSignParams {
    pub account: u32,
    pub coordinator: u8,
    pub funding_account: u32,
}

#[derive(Clone, Debug)]
pub enum SigningStatus {
    /// Coordinator: broadcasting our mailbox address for this session.
    AnnouncingCoordinator,
    /// Non-coordinator: the coordinator has not announced its address yet.
    WaitingForCoordinator,
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

/// What the protocol driver reports: a DKG status or a signing status,
/// depending on which protocol the pass advanced.
#[cfg_attr(feature = "flutter", frb)]
#[derive(Clone, Debug)]
pub enum ProtocolStatus {
    Dkg(DKGStatus),
    Sign(SigningStatus),
}

/// The app's handle on the protocol driver: one task that syncs the funding
/// and frost accounts, then advances the DKG or the signing session, on
/// every new block and never twice at once. A protocol page runs it while
/// open and cancels it when closed; the driver resumes where the session
/// left off when started again.
#[cfg_attr(feature = "flutter", frb(opaque))]
pub struct ProtocolDriver {
    driver: Option<Driver>,
}

impl ProtocolDriver {
    #[cfg_attr(feature = "flutter", frb(sync))]
    #[allow(clippy::new_without_default)] // Default would surface in Dart
    pub fn new() -> Self {
        Self { driver: None }
    }

    /// Start driving `c`'s protocol session, streaming statuses to `status`.
    /// Polls the chain tip every 10 s, like the app's block watcher.
    #[cfg(feature = "flutter")]
    pub async fn run(&mut self, status: StreamSink<ProtocolStatus>, c: &Coin) -> Result<()> {
        self.cancel();
        self.driver = Some(Driver::start(c.clone(), Duration::from_secs(10), status));
        Ok(())
    }

    /// Stop after the pass in flight, if any; no further pass runs.
    #[cfg_attr(feature = "flutter", frb(sync))]
    pub fn cancel(&mut self) {
        self.driver.take();
    }
}
