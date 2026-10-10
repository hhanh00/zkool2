use crate::api::coin::Coin;
pub use crate::near_intents::{
    SavedSwap, SwapAsset, SwapDetails, SwapQuote, SwapQuoteResponse, SwapRequest, SwapStatus,
    SwapTransaction, SwapType,
};
use anyhow::{ensure, Result};
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

fn transport(c: &Coin) -> crate::near_intents::Transport {
    crate::near_intents::Transport {
        mode: c.transport,
        proxy: c.proxy.clone(),
    }
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_list_swaps(pending_only: bool, c: &Coin) -> Result<Vec<SavedSwap>> {
    let mut connection = c.get_connection().await?;
    crate::near_intents::read_swaps(&mut connection, c.account, pending_only).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_assets(c: &Coin) -> Result<Vec<SwapAsset>> {
    crate::near_intents::assets(&transport(c)).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_quote(request: SwapRequest, c: &Coin) -> Result<SwapQuoteResponse> {
    use zcash_protocol::consensus::{NetworkType, Parameters};
    ensure!(
        c.network().network_type() == NetworkType::Main,
        "1Click requires Zcash mainnet"
    );
    crate::near_intents::quote(request, &transport(c)).await
}

/// Create a deposit quote and persist it before any wallet payment is prepared.
#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_create_swap(mut request: SwapRequest, c: &Coin) -> Result<SavedSwap> {
    ensure!(c.account != 0, "Select an account");
    request.dry = false;
    let response = near_intents_quote(request.clone(), c).await?;
    let mut connection = c.get_connection().await?;
    crate::near_intents::save_swap(&mut connection, c.account, &request, &response).await
}

/// Persist an already broadcast deposit before notifying the provider. Retrying
/// notification never sends another wallet transaction.
#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_record_deposit(id_swap: i64, tx_hash: String, c: &Coin) -> Result<()> {
    ensure!(
        tx_hash.len() == 64 && tx_hash.bytes().all(|b| b.is_ascii_hexdigit()),
        "Invalid Zcash transaction hash"
    );
    let address = {
        let mut connection = c.get_connection().await?;
        let result = sqlx::query(
            "UPDATE swaps SET deposit_tx_hash = ?, updated_at = unixepoch()
            WHERE id_swap = ? AND account = ? AND origin_asset = 'nep141:zec.omft.near'
            AND (deposit_tx_hash IS NULL OR deposit_tx_hash = ?)",
        )
        .bind(&tx_hash)
        .bind(id_swap)
        .bind(c.account)
        .bind(&tx_hash)
        .execute(&mut *connection)
        .await?;
        ensure!(
            result.rows_affected() == 1,
            "Swap not found, not a ZEC deposit, or already funded by another transaction"
        );
        crate::near_intents::swap_status_snapshot(&mut connection, id_swap, c.account)
            .await?
            .0
    };
    near_intents_submit_deposit(address, tx_hash, c).await?;
    let mut connection = c.get_connection().await?;
    sqlx::query("UPDATE swaps SET deposit_submitted_at = COALESCE(deposit_submitted_at, unixepoch()), updated_at = unixepoch() WHERE id_swap = ? AND account = ?")
        .bind(id_swap).bind(c.account).execute(&mut *connection).await?;
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_status(
    deposit_address: String,
    deposit_memo: Option<String>,
    c: &Coin,
) -> Result<SwapStatus> {
    crate::near_intents::status(&deposit_address, deposit_memo.as_deref(), &transport(c)).await
}

/// Refresh a saved swap owned by the current account. Provider failures leave
/// the saved snapshot untouched, and concurrent updates are checked on write.
#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_refresh_swap_status(id_swap: i64, c: &Coin) -> Result<SwapStatus> {
    let (address, memo, previous) = {
        let mut connection = c.get_connection().await?;
        crate::near_intents::swap_status_snapshot(&mut connection, id_swap, c.account).await?
    };
    // Release the database connection before contacting the provider.
    let current = crate::near_intents::status(&address, memo.as_deref(), &transport(c)).await?;
    let mut connection = c.get_connection().await?;
    crate::near_intents::persist_swap_status(
        &mut connection,
        id_swap,
        c.account,
        &address,
        memo.as_deref(),
        previous.as_deref(),
        &current,
    )
    .await?;
    Ok(current)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_submit_deposit(
    deposit_address: String,
    tx_hash: String,
    c: &Coin,
) -> Result<()> {
    crate::near_intents::submit_deposit(&deposit_address, &tx_hash, &transport(c)).await
}
