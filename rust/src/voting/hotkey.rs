//! The wallet's voting hotkey.
//!
//! The hotkey signs vote commitments and receives the delegation. zkool
//! generates one per wallet and persists its stored secret in the wallet
//! props table; the voting crate reconstructs it from those bytes. Every
//! round's delegation must target the same hotkey, so creation happens once
//! and loading fails closed when it is missing.

use anyhow::{ensure, Result};
use sqlx::SqliteConnection;
use zcash_voting::prelude::{generate_random_voting_hotkey, VotingHotkey};
use zcash_voting::Network as VotingNetwork;

/// Prop key holding the hex-encoded voting hotkey stored secret.
pub const VOTING_HOTKEY_PROP: &str = "voting_hotkey_secret";

/// Generates a fresh app-owned voting hotkey and persists its stored secret
/// (hex) in the wallet props table.
pub async fn create(
    connection: &mut SqliteConnection,
    network: VotingNetwork,
) -> Result<VotingHotkey> {
    ensure!(
        crate::db::get_prop(connection, VOTING_HOTKEY_PROP)
            .await?
            .is_none(),
        "voting hotkey already exists for this wallet"
    );
    let hotkey = generate_random_voting_hotkey(network)?;
    crate::db::put_prop(
        connection,
        VOTING_HOTKEY_PROP,
        &hex::encode(hotkey.stored_secret()),
    )
    .await?;
    Ok(hotkey)
}

/// Loads the persisted voting hotkey from the wallet props table.
pub async fn load(
    connection: &mut SqliteConnection,
    network: VotingNetwork,
) -> Result<VotingHotkey> {
    let secret = crate::db::get_prop(connection, VOTING_HOTKEY_PROP)
        .await?
        .ok_or_else(|| anyhow::anyhow!("no voting hotkey; create one first"))?;
    let secret = hex::decode(secret)?;
    Ok(VotingHotkey::from_stored_secret(&secret, network)?)
}

/// Loads the voting hotkey, creating it on first use.
///
/// Submission paths call this so a wallet that has never voted gets its
/// hotkey at delegation time instead of failing.
pub async fn load_or_create(
    connection: &mut SqliteConnection,
    network: VotingNetwork,
) -> Result<VotingHotkey> {
    if let Some(secret) = crate::db::get_prop(connection, VOTING_HOTKEY_PROP).await? {
        let secret = hex::decode(secret)?;
        return Ok(VotingHotkey::from_stored_secret(&secret, network)?);
    }
    create(connection, network).await
}
