use crate::api::coin::Coin;
pub use crate::near_intents::{
    SavedSwap, SwapAsset, SwapDetails, SwapQuote, SwapQuoteResponse, SwapRequest, SwapStatus,
    SwapTransaction, SwapType,
};
use anyhow::{ensure, Context, Result};
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

async fn saved_swap(id_swap: i64, c: &Coin) -> Result<SavedSwap> {
    let mut connection = c.get_connection().await?;
    crate::near_intents::read_swaps(&mut connection, c.account, false)
        .await?
        .into_iter()
        .find(|s| s.id_swap == id_swap)
        .context("Swap not found for the current account")
}

/// Prepare the exact saved deposit for the normal transaction UI.
#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_prepare_swap(id_swap: i64, c: &Coin) -> Result<crate::api::pay::PcztPackage> {
    use zcash_protocol::consensus::{NetworkType, Parameters};
    ensure!(c.account != 0 && c.network().network_type() == NetworkType::Main, "Select a Zcash mainnet account");
    let swap = saved_swap(id_swap, c).await?;
    ensure!(swap.deposit_tx_hash.is_none(), "Swap deposit already sent");
    let recipient = funding_recipient(&swap)?;
    ensure_swap_deadline(&swap, c).await?;
    crate::api::pay::prepare(&[recipient], crate::api::pay::PaymentOptions {
        src_pools: 15, recipient_pays_fee: false, smart_transparent: false, category: None,
    }, c).await
}

async fn ensure_swap_deadline(swap: &SavedSwap, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let valid: Option<bool> = sqlx::query_scalar("SELECT julianday(?) > julianday('now')")
        .bind(&swap.deadline)
        .fetch_one(&mut *connection)
        .await?;
    ensure!(
        valid == Some(true),
        "Swap deadline has passed; create a new quote"
    );
    Ok(())
}

fn funding_recipient(swap: &SavedSwap) -> Result<crate::pay::Recipient> {
    ensure!(
        swap.origin_asset == crate::near_intents::ZEC_ASSET,
        "Not a ZEC deposit"
    );
    ensure!(
        swap.completed_at.is_none()
            && matches!(swap.status.as_deref(), None | Some("PENDING_DEPOSIT")),
        "Swap is already processing or complete"
    );
    ensure!(
        swap.deposit_memo.is_none(),
        "ZEC deposit unexpectedly requires a memo"
    );
    crate::openalias::try_validate_zcash_address(
        &swap.deposit_address,
        zcash_protocol::consensus::NetworkType::Main,
    )?;
    let amount: u64 = swap.amount_in.parse().context("Invalid deposit amount")?;
    ensure!(
        (1..=2_100_000_000_000_000).contains(&amount),
        "Invalid ZEC deposit amount"
    );
    Ok(crate::pay::Recipient {
        address: swap.deposit_address.clone(),
        amount,
        ..Default::default()
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Connection;

    #[tokio::test]
    async fn funding_uses_exact_quoted_deposit_and_rejects_ineligible_swaps() -> Result<()> {
        let mut db = sqlx::SqliteConnection::connect("sqlite::memory:").await?;
        sqlx::raw_sql("CREATE TABLE accounts(id_account INTEGER PRIMARY KEY); INSERT INTO accounts VALUES (1);")
            .execute(&mut db).await?;
        sqlx::raw_sql(include_str!("../db/swaps.sql"))
            .execute(&mut db)
            .await?;
        let request = SwapRequest {
            dry: false,
            swap_type: SwapType::ExactOutput,
            origin_asset: crate::near_intents::ZEC_ASSET.into(),
            destination_asset: "usdt".into(),
            amount: "5000000".into(),
            slippage_tolerance: 100,
            recipient: "external".into(),
            refund_to: "wallet".into(),
            deadline: "2099-01-01T00:00:00Z".into(),
        };
        let quote: SwapQuote = serde_json::from_str(
            r#"{"amountIn":"437166","amountOut":"5000000","depositAddress":"t1VmmGiyjVNeCjxDZzg7vZmd99WyzVby9yC"}"#,
        )?;
        let mut swap = crate::near_intents::save_swap(
            &mut db,
            1,
            &request,
            &SwapQuoteResponse {
                quote,
                raw_response: "{}".into(),
            },
        )
        .await?;
        let recipient = funding_recipient(&swap)?;
        assert_eq!(recipient.amount, 437166); // deposit, not requested USDT amount
        assert_eq!(recipient.address, swap.deposit_address);
        assert!(recipient.asset_base.is_empty()); // native ZEC
        assert!(recipient.memo_bytes.is_none());
        for amount in ["0", "1.5", "-1", "18446744073709551616", "2100000000000001"] {
            swap.amount_in = amount.into();
            assert!(funding_recipient(&swap).is_err());
        }
        swap.amount_in = "437166".into();
        swap.status = Some("PROCESSING".into());
        assert!(funding_recipient(&swap).is_err());
        swap.status = None;
        swap.origin_asset = "usdt".into();
        assert!(funding_recipient(&swap).is_err());
        Ok(())
    }
}
