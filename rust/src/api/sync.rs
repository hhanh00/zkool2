use anyhow::Result;

use crate::api::coin::Coin;
#[cfg(feature = "flutter")]
use crate::Sink;

use crate::db::calculate_balance;
use crate::io::SyncHeight;
#[cfg(feature = "flutter")]
use crate::sync::driver::SyncRequest;
use crate::sync::driver::{Apply, Rewind, DETAILS_BATCH};
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
    // The driver reports through a channel; forward it to the stream.
    let (events, mut receiver) = tokio::sync::mpsc::channel(16);
    let stream = progress.clone();
    tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            stream.send(event).await;
        }
    });
    let request = SyncRequest {
        accounts,
        current_height,
        actions_per_sync,
        transparent_limit,
        checkpoint_age,
        skip_details: fast,
    };
    let result = match c.driver() {
        Ok(driver) => driver.sync(c, request, events).await,
        Err(e) => Err(e),
    };
    match result {
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

/// Rewind `account` to the last checkpoint at or before `height`, through
/// the sync driver: a sync in flight is cancelled first.
#[cfg_attr(feature = "flutter", frb)]
pub async fn rewind_sync(height: u32, account: u32, c: &Coin) -> Result<()> {
    c.driver()?.rewind(c, account, Rewind::To(height)).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_db_height(account: u32, c: &Coin) -> Result<SyncHeight> {
    let mut connection = c.get_connection().await?;
    crate::sync::get_db_height(&mut *connection, account).await
}

/// Decrypt memos and store transaction details for `account`.
///
/// The fetching and decryption run outside the wallet's sync session: they
/// take a network round trip per transaction, and holding the session for
/// that once starved a sync, which was skipped while a DKG went on to build
/// a transaction from stale notes and double-spent. Only the writes go
/// through the sync driver, a batch at a time, so they never land on rows a
/// rewind has deleted meanwhile.
#[cfg_attr(feature = "flutter", frb)]
pub async fn fetch_tx_details(account: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    let network = c.network();
    let txids = crate::memo::pending_details(&mut connection, account).await?;
    for batch in txids.chunks(DETAILS_BATCH) {
        let mut txs = Vec::with_capacity(batch.len());
        for txid in batch {
            txs.push(
                crate::memo::decrypt_tx(&network, &mut connection, &mut client, account, txid)
                    .await?,
            );
        }
        c.driver()?
            .apply(c, Apply::TxDetails { account, txs })
            .await?;
    }
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
