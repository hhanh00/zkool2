use anyhow::Result;
use flutter_rust_bridge::frb;
use rustls::client;
use serde::{Deserialize, Serialize};
use tonic::Request;

use crate::{api::network, frost, get_coin, lwd::ChainSpec};
use std::str::FromStr;

#[frb]
pub async fn set_dkg_params(name: &str, id: u8, n: u8, t: u8, funding_account: u32) -> Result<()> {
    let c = get_coin!();
    let connection = c.get_pool();
    let mut client = c.client().await?;
    let height = client
        .get_latest_block(Request::new(ChainSpec {}))
        .await?
        .into_inner()
        .height as u32;
    let birth_height = height - 10000;

    sqlx::query("INSERT INTO dkg_params(account, id, n, t, seed, birth_height) VALUES (?, ?, ?, ?, '', ?) ON CONFLICT DO NOTHING")
        .bind(funding_account)
        .bind(id)
        .bind(n)
        .bind(t)
        .bind(birth_height)
        .execute(connection)
        .await?;
    sqlx::query("INSERT INTO props(key, value) VALUES ('dkg_name', ?1) ON CONFLICT DO NOTHING")
        .bind(name)
        .execute(connection)
        .await?;
    sqlx::query("INSERT INTO props(key, value) VALUES ('dkg_funding', ?1) ON CONFLICT DO NOTHING")
        .bind(funding_account)
        .execute(connection)
        .await?;

    Ok(())
}

#[frb]
pub async fn dkg() -> Result<DKGStatus> {
    let c = get_coin!();
    let connection = c.get_pool();
    let mut client = c.client().await?;
    let height = client
        .get_latest_block(Request::new(ChainSpec {}))
        .await?
        .into_inner()
        .height as u32;

    let Some((account,)) =
        sqlx::query_as::<_, (String,)>("SELECT value FROM props WHERE key = 'dkg_funding'")
            .fetch_optional(connection)
            .await?
    else {
        return Ok(DKGStatus::WaitParams);
    };
    crate::frost::dkg::do_dkg(
        &c.network,
        connection,
        u32::from_str(&account).unwrap(),
        &mut client,
        height,
    )
    .await
}

pub async fn set_dkg_address(id: u16, address: &str) -> Result<()> {
    let c = get_coin!();
    let connection = c.get_pool();
    let (account,) =
        sqlx::query_as::<_, (String,)>("SELECT value FROM props WHERE key = 'dkg_funding'")
            .fetch_one(connection)
            .await?;

    crate::frost::dkg::set_dkg_address(
        connection,
        u32::from_str(&account).unwrap(),
        id,
        address,
    ).await
}

#[frb(dart_metadata = ("freezed"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DKGParams {
    pub name: String,
    pub id: u8,
    pub n: u8,
    pub t: u8,
    pub funding_account: u32,
}

pub enum DKGStatus {
    WaitParams,
    WaitAddresses(Vec<String>),
    WaitRound1Pkg,
    WaitRound2Pkg,
    Finalize,
    SharedAddress(String),
}
