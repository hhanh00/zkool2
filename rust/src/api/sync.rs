use anyhow::Result;
use std::sync::LazyLock;
use tokio::sync::{broadcast, Mutex};

use crate::api::coin::Coin;

use crate::db::calculate_balance;
use crate::io::SyncHeight;
use crate::sync::BlockHeader;

#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

#[cfg(feature = "flutter")]
#[cfg_attr(feature = "flutter", frb)]
pub async fn synchronize(
    progress: StreamSink<SyncProgress>,
    accounts: Vec<u32>,
    current_height: u32,
    actions_per_sync: u32,
    transparent_limit: u32,
    checkpoint_age: u32,
    c: &Coin,
) -> Result<u32> {
    crate::sync::synchronize_impl(
        progress,
        accounts,
        current_height,
        actions_per_sync,
        transparent_limit,
        checkpoint_age,
        c,
    )
    .await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn balance(c: &Coin) -> Result<PoolBalance> {
    let mut connection = c.get_connection().await?;
    let account = c.account;

    calculate_balance(&mut *connection, account, None).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn cancel_sync() -> Result<()> {
    let tx = CANCEL_SYNC.lock().await;
    if let Some(tx) = tx.as_ref() {
        tx.send(())?;
    }
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn rewind_sync(height: u32, account: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::sync::rewind_sync(&c.network(), &mut *connection, account, height).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_db_height(c: &Coin) -> Result<SyncHeight> {
    let mut connection = c.get_connection().await?;
    crate::sync::get_db_height(&mut *connection, c.account).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn fetch_tx_details(c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    crate::memo::fetch_tx_details(&c.network(), &mut *connection, &mut client, c.account).await?;
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn cache_block_time(height: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    let block = client.block(&c.network(), height).await?;
    let bh = BlockHeader {
        height,
        hash: block.hash,
        time: block.time,
    };
    crate::db::store_block_header(&mut connection, &bh).await?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct SyncProgress {
    pub height: u32,
    pub time: u32,
}

pub struct PoolBalance(pub Vec<u64>);

pub static SYNCING: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
pub static CANCEL_SYNC: LazyLock<Mutex<Option<broadcast::Sender<()>>>> =
    LazyLock::new(|| Mutex::new(None));
