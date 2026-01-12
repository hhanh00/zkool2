use anyhow::anyhow;
use juniper::{FieldError, FieldResult};

use crate::{
    api::frost::get_funding_account,
    graphql::{data::DKGStatus, Context},
};

pub async fn dkg_status() -> FieldResult<DKGStatus> {
    todo!()
}

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

pub async fn do_dkg(context: &Context) -> FieldResult<bool> {
    let coin = &context.coin;
    let mut connection = coin.get_connection().await?;
    let mut client = coin.client().await?;
    let height = client.latest_height().await?;
    let account = get_funding_account(&mut connection)
        .await?
        .ok_or(anyhow!("No messaging account"))?;
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
