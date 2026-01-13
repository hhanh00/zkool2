use anyhow::{Ok, Result};
use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;

use crate::{
    api::coin::Coin,
    frb_generated::StreamSink,
    frost::{db::get_mailbox_account, dkg::get_dkg_params},
};
use std::str::FromStr;

use super::pay::PcztPackage;

#[frb]
pub async fn set_dkg_params(
    name: &str,
    id: u8,
    n: u8,
    t: u8,
    funding_account: u32,
    c: &Coin,
) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    crate::frost::dkg::set_dkg_params(
        &c.network(),
        &mut connection,
        &mut client,
        name,
        id,
        n,
        t,
        funding_account,
    )
    .await
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
    let account = get_funding_account(&mut *connection).await?;
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
    let account = get_funding_account(&mut *connection).await?;
    let dkg_params = get_dkg_params(&mut *connection, account).await?;
    let addresses =
        crate::frost::db::get_addresses(&mut *connection, account, dkg_params.n).await?;
    Ok(addresses.iter().all(|a| !a.is_empty()))
}

#[frb]
pub async fn do_dkg(status: StreamSink<DKGStatus>, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let mut client = c.client().await?;
    let height = client.latest_height().await?;
    let account = get_funding_account(&mut connection).await?;

    let r = crate::frost::dkg::do_dkg(
        &c.network(),
        &mut connection,
        account,
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

pub async fn get_dkg_addresses(c: &Coin) -> Result<Vec<String>> {
    let mut connection = c.get_connection().await?;
    let account = get_funding_account(&mut connection).await?;
    let n = get_dkg_params(&mut connection, account).await?.n;
    let addresses = crate::frost::db::get_addresses(&mut connection, account, n).await?;
    Ok(addresses)
}

pub async fn set_dkg_address(id: u8, address: &str, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let account = get_funding_account(&mut connection).await?;
    let dkg_params = get_dkg_params(&mut connection, account).await?;
    let my_id = dkg_params.id;
    crate::frost::dkg::set_dkg_address(&mut connection, account, id, my_id, address).await
}

#[frb]
pub async fn cancel_dkg(c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let account = get_funding_account(&mut connection).await?;
    crate::frost::dkg::cancel_dkg(&mut connection, account).await
}

pub(crate) async fn get_funding_account(connection: &mut SqliteConnection) -> Result<u32> {
    let (account,): (String,) = sqlx::query_as("SELECT value FROM props WHERE key = 'dkg_funding'")
        .fetch_one(&mut *connection)
        .await?;
    let account = u32::from_str(&account).unwrap();
    Ok(account)
}

#[frb(dart_metadata = ("freezed"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DKGParams {
    pub id: u8,
    pub n: u8,
    pub t: u8,
    pub birth_height: u32,
}

#[derive(Clone, Debug)]
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
pub async fn init_sign(
    coordinator: u8,
    funding_account: u32,
    pczt: &PcztPackage,
    c: &Coin,
) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::frost::sign::init_sign(
        &mut *connection,
        c.account,
        funding_account,
        coordinator,
        pczt,
    )
    .await
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
    pub account: u32,
    pub coordinator: u8,
    pub funding_account: u32,
}

#[derive(Clone, Debug)]
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
