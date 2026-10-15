use anyhow::Result;

use crate::api::coin::Coin;
#[cfg(feature = "flutter")]
use crate::Sink;

use crate::db::calculate_balance;
use crate::io::SyncHeight;
use crate::sync::BlockHeader;

#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

/// Synchronize `accounts` up to `current_height`, streaming progress and
/// finally the outcome. A failure is delivered as the stream's error; the
/// stream never ends without either a `Finished` event or an error.
///
/// `fast` skips the memo and transaction-detail pass; the caller fetches
/// details afterwards.
#[cfg(feature = "flutter")]
#[allow(clippy::too_many_arguments)]
#[cfg_attr(feature = "flutter", frb)]
pub async fn synchronize(
    progress: StreamSink<SyncEvent>,
    accounts: Vec<u32>,
    current_height: u32,
    actions_per_sync: u32,
    transparent_limit: u32,
    checkpoint_age: u32,
    fast: bool,
    c: &Coin,
) -> Result<()> {
    match crate::sync::synchronize_impl(
        progress.clone(),
        accounts,
        current_height,
        actions_per_sync,
        transparent_limit,
        checkpoint_age,
        fast,
        c,
    )
    .await
    {
        Ok(outcome) => progress.send(SyncEvent::Finished { outcome }).await,
        Err(e) => progress.send_error(e).await,
    }
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn balance(account: u32, c: &Coin) -> Result<PoolBalance> {
    let mut connection = c.get_connection().await?;

    calculate_balance(&mut *connection, account, None).await
}

/// Ask the wallet's running sync, if any, to stop.
#[cfg_attr(feature = "flutter", frb)]
pub async fn cancel_sync(c: &Coin) -> Result<()> {
    c.sync.cancel();
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn rewind_sync(height: u32, account: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::sync::rewind_sync(&c.network(), &mut *connection, account, height).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_db_height(account: u32, c: &Coin) -> Result<SyncHeight> {
    let mut connection = c.get_connection().await?;
    crate::sync::get_db_height(&mut *connection, account).await
}

/// Decrypt memos and store transaction details for `account`.
///
/// Deliberately does NOT take the wallet's sync session. It did briefly, to
/// stop it colliding with `do_dkg` — but holding the session here let it
/// starve the next sync, which was then skipped, and the DKG went on to build
/// a transaction from stale notes and double-spent. Detail fetching is best
/// effort and its `database is locked` failures are self-healing; a starved
/// sync is not.
#[cfg_attr(feature = "flutter", frb)]
pub async fn fetch_tx_details(account: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    crate::memo::fetch_tx_details(&c.network(), &mut *connection, &mut client, account).await?;
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn cache_block_time(account: u32, height: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    let block = client.block(&c.network(), height).await?;
    let bh = BlockHeader {
        height,
        hash: block.hash,
        time: block.time,
    };
    crate::sync::chain_log::store_header(&mut connection, account, &bh).await?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct SyncProgress {
    pub height: u32,
    pub time: u32,
}

/// How a sync run ended. A failure is not an outcome: it is the error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncOutcome {
    /// Every account is synced to `height`.
    Synced { height: u32 },
    /// Another sync was already running; nothing was done.
    Skipped,
    /// Stopped by the user; progress so far is committed.
    Cancelled,
}

/// What the sync stream carries: progress while it runs, then how it ended.
#[derive(Clone, Debug)]
pub enum SyncEvent {
    Progress { height: u32, time: u32 },
    Finished { outcome: SyncOutcome },
}

pub struct PoolBalance(pub Vec<u64>);
