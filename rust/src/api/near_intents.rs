use crate::api::coin::Coin;
pub use crate::near_intents::{
    SavedSwap, SwapAsset, SwapDetails, SwapQuote, SwapQuoteResponse, SwapRequest, SwapStatus,
    SwapTransaction, SwapType,
};
use anyhow::Result;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_list_swaps(pending_only: bool, c: &Coin) -> Result<Vec<SavedSwap>> {
    crate::near_intents::list_swaps(pending_only, c).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_assets(c: &Coin) -> Result<Vec<SwapAsset>> {
    crate::near_intents::assets(c).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_quote(request: SwapRequest, c: &Coin) -> Result<SwapQuoteResponse> {
    crate::near_intents::quote(request, c).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_status(
    deposit_address: String,
    deposit_memo: Option<String>,
    c: &Coin,
) -> Result<SwapStatus> {
    crate::near_intents::status(&deposit_address, deposit_memo.as_deref(), c).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_refresh_swap_status(id_swap: i64, c: &Coin) -> Result<SwapStatus> {
    crate::near_intents::refresh_swap_status(id_swap, c).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn near_intents_submit_deposit(
    deposit_address: String,
    tx_hash: String,
    c: &Coin,
) -> Result<()> {
    crate::near_intents::submit_deposit(&deposit_address, &tx_hash, c).await
}
