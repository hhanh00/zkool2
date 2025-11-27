use anyhow::{Ok, Result};
use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;

use crate::{
    api::coin::Coin, frb_generated::StreamSink, frost::{db::get_mailbox_account, dkg::get_dkg_params}
};
use std::str::FromStr;

use super::pay::PcztPackage;

#[frb]
pub async fn set_dkg_params(name: &str, id: u8, n: u8, t: u8, funding_account: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    let height = client.latest_height().await?;
    let birth_height = height - 10000;

    sqlx::query(
        "INSERT INTO dkg_params(account, id, n, t, seed, birth_height) VALUES (?, ?, ?, ?, '', ?)",
    )
    .bind(funding_account)
    .bind(id)
    .bind(n)
    .bind(t)
    .bind(birth_height)
    .execute(&mut *connection)
    .await?;
    sqlx::query("INSERT INTO props(key, value) VALUES ('dkg_name', ?1)")
        .bind(name)
        .execute(&mut *connection)
        .await?;
    sqlx::query("INSERT INTO props(key, value) VALUES ('dkg_funding', ?1)")
        .bind(funding_account)
        .execute(&mut *connection)
        .await?;

    Ok(())
}

#[frb]
pub async fn has_dkg_params(c: &Coin) -> Result<bool> {
    let mut connection = c.get_connection().await?;
    let exists =
        sqlx::query_as::<_, (String,)>("SELECT value FROM props WHERE key = 'dkg_funding'")
            .fetch_optional(&mut *connection)
            .await?;
    Ok(exists.is_some())
}

#[frb]
pub async fn init_dkg(c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let account = get_funding_account(&mut *connection)
        .await?
        .expect("Funding account not set");
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

#[frb]
pub async fn has_dkg_addresses(c: &Coin) -> Result<bool> {
    let mut connection = c.get_connection().await?;
    let account = get_funding_account(&mut *connection)
        .await?
        .expect("Funding account not set");
    let dkg_params = get_dkg_params(&mut *connection, account).await?;
    let addresses = crate::frost::db::get_addresses(&mut *connection, account, dkg_params.n).await?;
    Ok(addresses.iter().all(|a| !a.is_empty()))
}

#[frb]
pub async fn do_dkg(status: StreamSink<DKGStatus>, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    let height = client.latest_height().await?;
    let account = get_funding_account(&mut connection)
        .await?
        .expect("Funding account not set");

    let r = crate::frost::dkg::do_dkg(&c.network(), &mut connection, account, &mut client, height, status.clone()).await;
    if let Err(e) = r {
        let _ = status.add_error(e);
    }
    Ok(())
}

pub async fn get_dkg_addresses(c: &Coin) -> Result<Vec<String>> {
    let mut connection = c.get_connection().await?;
    let account = get_funding_account(&mut connection)
        .await?
        .expect("Funding account not set");
    let n = get_dkg_params(&mut connection, account).await?.n;
    let addresses = crate::frost::db::get_addresses(&mut connection, account, n).await?;
    Ok(addresses)
}

pub async fn set_dkg_address(id: u16, address: &str, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let account = get_funding_account(&mut connection)
        .await?
        .expect("Funding account not set");

    crate::frost::dkg::set_dkg_address(&mut connection, account, id, address).await
}

#[frb]
pub async fn cancel_dkg(c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let account = get_funding_account(&mut connection).await?;
    crate::frost::dkg::cancel_dkg(&mut connection, account).await
}

async fn get_funding_account(connection: &mut SqliteConnection) -> Result<Option<u32>> {
    let rs = sqlx::query_as::<_, (String,)>("SELECT value FROM props WHERE key = 'dkg_funding'")
        .fetch_optional(&mut *connection)
        .await?;
    let account = rs.map(|(account,)| u32::from_str(&account).unwrap());
    Ok(account)
}

#[frb(dart_metadata = ("freezed"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DKGParams {
    pub id: u16,
    pub n: u8,
    pub t: u8,
    pub birth_height: u32,
}

#[derive(Clone)]
pub enum DKGStatus {
    WaitParams,
    WaitAddresses(Vec<String>),
    PublishRound1Pkg,
    WaitRound1Pkg,
    PublishRound2Pkg,
    WaitRound2Pkg,
    Finalize,
    SharedAddress(String),
}

#[frb]
pub async fn reset_sign(c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::frost::sign::reset_sign(&mut *connection).await
}

#[frb]
pub async fn init_sign(coordinator: u16, funding_account: u32, pczt: &PcztPackage, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::frost::sign::init_sign(&mut *connection, c.account, funding_account, coordinator, pczt).await
}

#[frb]
pub async fn is_signing_in_progress(c: &Coin) -> Result<bool> {
    let mut connection = c.get_connection().await?;
    crate::frost::sign::is_signing_in_progress(&mut *connection).await
}

#[frb]
pub async fn do_sign(status: StreamSink<SigningStatus>, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    let height = client.latest_height().await?;
    let r = crate::frost::sign::do_sign(
        &c.network(),
        &mut *connection,
        c.account,
        &mut client,
        height,
        status.clone(),
    )
    .await;
    if let Err(e) = r {
        let _ = status.add_error(e);
    }
    Ok(())
}

#[frb(dart_metadata = ("freezed"))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrostSignParams {
    pub coordinator: u16,
    pub funding_account: u32,
}

#[derive(Clone)]
pub enum SigningStatus {
    SendingCommitment,
    WaitingForCommitments,
    SendingSigningPackage,
    WaitingForSigningPackage,
    SendingSignatureShare,
    SigningCompleted,
    WaitingForSignatureShares,
    PreparingTransaction,
    SendingTransaction,
    TransactionSent(String),
}
