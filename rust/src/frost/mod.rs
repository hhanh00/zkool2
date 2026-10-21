pub mod backup;
pub mod dkg;
pub mod keys;
pub mod driver;
pub mod protocol;
pub mod sign;
pub mod upgrade;

pub use protocol::{
    to_arb_memo, Broadcast, Dispatch, FrostBytes, FrostMessage, FrostSigMessage, Indexed, NoSend,
    PK1Map, PK2Map, PerPeer, Round, RouteCtx, ToCoordinator, P,
};

/// Whether `account` is a FROST group account: one holding a key package,
/// so its spends go through a signing session rather than the wallet's keys.
/// The `dkg_params` row alone does not say so: it sits on the funding
/// account until the DKG finalizes and rekeys it onto the new account.
pub async fn is_group_account(
    connection: &mut sqlx::SqliteConnection,
    account: u32,
) -> anyhow::Result<bool> {
    let row: Option<(u8,)> =
        sqlx::query_as("SELECT 1 FROM dkg_state WHERE account = ? AND key_pkg IS NOT NULL")
            .bind(account)
            .fetch_optional(&mut *connection)
            .await?;
    Ok(row.is_some())
}
