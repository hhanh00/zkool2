mod exec;
mod plan;
mod state;
mod step;
mod task;

pub use step::{sign_step, SignStepOutcome};

pub(super) const COMMITMENT_PREFIX: &[u8] = b"CMT6";
pub(super) const SIGPACKAGE_PREFIX: &[u8] = b"SPK5";
pub(super) const SIGSHARE_PREFIX: &[u8] = b"SSH3";

// ── Public API (unchanged surface for flutter / graphql callers) ──────────────

use anyhow::Result;
use bincode::config;
use sqlx::SqliteConnection;
use tracing::info;

use crate::{
    api::{coin::Network, frost::FrostSignParams, pay::PcztPackage},
    frost::dkg::delete_frost_state,
    Client, Sink,
};

#[cfg(feature = "flutter")]
use crate::frb_generated::StreamSink;

pub async fn init_sign(
    connection: &mut SqliteConnection,
    account: u32,
    funding_account: u32,
    coordinator: u8,
    pczt: &PcztPackage,
) -> Result<()> {
    info!(
        "init_sign: account={}, funding_account={}, coordinator={}",
        account, funding_account, coordinator
    );
    let pczt_bytes = bincode::encode_to_vec(pczt, config::legacy()).unwrap();
    sqlx::query("INSERT INTO props(key, value) VALUES ('frost_pczt', ?) ON CONFLICT DO NOTHING")
        .bind(&pczt_bytes)
        .execute(&mut *connection)
        .await?;
    let params = FrostSignParams {
        account,
        coordinator,
        funding_account,
    };
    let params_json = serde_json::to_string(&params).unwrap();
    sqlx::query(
        "INSERT INTO props(key, value) VALUES ('frost_sign_params', ?) ON CONFLICT DO NOTHING",
    )
    .bind(&params_json)
    .execute(&mut *connection)
    .await?;
    sqlx::query("INSERT INTO props(key, value) VALUES ('dkg_account', ?) ON CONFLICT DO NOTHING")
        .bind(funding_account)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

pub async fn reset_sign(connection: &mut SqliteConnection) -> Result<()> {
    delete_frost_state(&mut *connection).await
}

pub async fn is_signing_in_progress(connection: &mut SqliteConnection) -> Result<bool> {
    let exists =
        sqlx::query_as::<_, (bool,)>("SELECT TRUE FROM props WHERE key = 'frost_pczt'")
            .fetch_optional(&mut *connection)
            .await?;
    Ok(exists.is_some())
}

pub async fn in_sign(connection: &mut SqliteConnection) -> Result<bool> {
    let exists = sqlx::query("SELECT 1 FROM props WHERE key LIKE 'frost_%'")
        .fetch_optional(connection)
        .await?;
    Ok(exists.is_some())
}

// ── do_sign / do_sign_impl: backward-compat wrappers ─────────────────────────
//
// These drive sign_step in a loop, sleeping between iterations, which matches
// the old sequential orchestrator behaviour. The Sink receives progress updates
// on each step.

#[cfg(feature = "flutter")]
pub async fn do_sign(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    height: u32,
    status: StreamSink<crate::api::frost::SigningStatus>,
) -> Result<()> {
    do_sign_impl(network, connection, client, height, status).await
}

pub async fn do_sign_impl(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    height: u32,
    _status: impl Sink<crate::api::frost::SigningStatus>,
) -> Result<()> {
    info!("sign: starting at height {}", height);

    let Some(outcome) = sign_step(network, connection, client, height).await? else {
        info!("sign: no signing session in progress");
        return Ok(());
    };

    if let Some(txid) = outcome.txid {
        info!("sign: transaction broadcast: {txid}");
    }
    Ok(())
}
