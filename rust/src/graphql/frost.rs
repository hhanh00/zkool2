use bincode::config;
use juniper::{FieldError, FieldResult};
use sqlx::{query, sqlite::SqliteRow, Row};

use crate::{
    api::{coin::Coin, frost::get_funding_account},
    frost::dkg::{in_dkg, in_frost},
    graphql::Context,
    sync::{synchronize_impl, DEFAULT_ACTIONS_PER_SYNC},
};

pub async fn dkg_start(
    name: String,
    threshold: i32,
    participants: i32,
    message_account: i32,
    id_participant: i32,
    context: &Context,
) -> FieldResult<String> {
    let coin = &context.coin;
    crate::api::frost::set_dkg_params(
        &name,
        id_participant as u8,
        participants as u8,
        threshold as u8,
        message_account as u32,
        coin,
    )
    .await?;
    crate::api::frost::init_dkg(coin).await?;
    let addresses = crate::api::frost::get_dkg_addresses(coin).await?;
    if id_participant <= 0 || id_participant > participants {
        return Err(FieldError::new(
            "Invalid id_participant",
            juniper::Value::Null,
        ));
    }
    let address = addresses[id_participant as usize - 1].clone();
    Ok(address)
}

pub async fn dkg_cancel(context: &Context) -> FieldResult<bool> {
    crate::api::frost::cancel_dkg(&context.coin).await?;
    Ok(true)
}

pub async fn dkg_set_address(
    id_participant: i32,
    address: String,
    context: &Context,
) -> FieldResult<bool> {
    crate::api::frost::set_dkg_address(id_participant as u8, &address, &context.coin).await?;
    Ok(true)
}

pub async fn new_block(coin: Coin) -> anyhow::Result<()> {
    let mut connection = coin.get_connection().await?;
    let mut client = coin.client().await?;
    let height = client.latest_height().await?;
    tracing::info!("new_block {height}");

    let in_dkg = in_dkg(&mut connection).await?;
    let in_frost = in_frost(&mut connection).await?;
    tracing::info!("in_dkg {in_dkg} in_frost {in_frost}");
    if !in_dkg && !in_frost {
        return Ok(());
    }

    let account = get_funding_account(&mut connection).await?;
    tracing::info!("funding: {account}");
    let mut frost_accounts =
        query("SELECT id_account FROM accounts WHERE name LIKE 'frost-%' AND internal = 1")
            .map(|r: SqliteRow| r.get::<u32, _>(0))
            .fetch_all(&mut *connection)
            .await?;
    frost_accounts.push(account);
    let height = synchronize_impl(
        (),
        frost_accounts,
        height,
        DEFAULT_ACTIONS_PER_SYNC,
        1,
        100,
        &coin,
    )
    .await?;

    if in_dkg {
        crate::frost::dkg::do_dkg_impl(
            &coin.network(),
            &mut connection,
            account,
            &mut client,
            height,
            (),
        )
        .await?;
    }

    if in_frost {
        crate::frost::sign::do_sign_impl(
            &coin.network(),
            &mut connection,
            &mut client,
            height,
            (),
        )
        .await?;
    }
    Ok(())
}

pub async fn do_dkg(context: &Context) -> FieldResult<bool> {
    let coin = &context.coin;
    let mut connection = coin.get_connection().await?;
    let mut client = coin.client().await?;
    let height = client.latest_height().await?;
    let account = get_funding_account(&mut connection).await?;
    crate::frost::dkg::do_dkg_impl(
        &coin.network(),
        &mut connection,
        account,
        &mut client,
        height,
        (),
    )
    .await?;
    Ok(true)
}

pub async fn frost_sign(
    id_coordinator: i32,
    id_account: i32,
    message_account: i32,
    pczt: String,
    context: &Context,
) -> FieldResult<bool> {
    let coin = &context.coin;
    let mut connection = coin.get_connection().await?;
    let pczt = hex::decode(&pczt)?;
    let (pczt, _) = bincode::decode_from_slice(&pczt, config::standard())?;
    crate::frost::sign::init_sign(
        &mut connection,
        id_account as u32,
        message_account as u32,
        id_coordinator as u8,
        &pczt,
    )
    .await?;
    Ok(true)
}
