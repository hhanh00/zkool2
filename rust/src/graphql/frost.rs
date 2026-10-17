use bincode::config;
use juniper::{FieldError, FieldResult};

use crate::graphql::{data::FrostRestoreResult, Context};

fn invalid_integer(name: &str) -> FieldError {
    FieldError::new(format!("Invalid {name}"), juniper::Value::Null)
}

pub async fn dkg_start(
    name: String,
    threshold: i32,
    participants: i32,
    message_account: i32,
    id_participant: i32,
    context: &Context,
) -> FieldResult<String> {
    if id_participant <= 0 || id_participant > participants {
        return Err(invalid_integer("id_participant"));
    }
    let id_participant = id_participant
        .try_into()
        .map_err(|_| invalid_integer("id_participant"))?;
    let participants = participants
        .try_into()
        .map_err(|_| invalid_integer("participants"))?;
    let threshold = threshold
        .try_into()
        .map_err(|_| invalid_integer("threshold"))?;
    let message_account = message_account
        .try_into()
        .map_err(|_| invalid_integer("message_account"))?;

    let coin = &context.coin;
    crate::api::frost::set_dkg_params(
        &name,
        id_participant,
        participants,
        threshold,
        message_account,
        coin,
    )
    .await?;
    crate::api::frost::init_dkg(coin).await?;
    let addresses = crate::api::frost::get_dkg_addresses(coin).await?;
    let address = addresses[usize::from(id_participant) - 1].clone();
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
    let id_participant = id_participant
        .try_into()
        .map_err(|_| invalid_integer("id_participant"))?;
    crate::api::frost::set_dkg_address(id_participant, &address, &context.coin).await?;
    Ok(true)
}

/// Run one protocol pass now, through the driver, and report its result.
/// Passes also run on every new block without being asked.
pub async fn do_dkg(context: &Context) -> FieldResult<bool> {
    context.driver.step().await?;
    Ok(true)
}

/// The member's `zkfrost` backup string for a frost account.
pub async fn frost_export_backup(id_account: i32, context: &Context) -> FieldResult<String> {
    let id_account = id_account
        .try_into()
        .map_err(|_| invalid_integer("id_account"))?;
    Ok(crate::api::frost::export_frost_backup(id_account, &context.coin).await?)
}

/// Restore a frost account from a backup string and the group's UFVK.
pub async fn frost_restore(
    name: String,
    backup: String,
    ufvk: String,
    context: &Context,
) -> FieldResult<FrostRestoreResult> {
    let restored =
        crate::api::frost::restore_frost_backup(&name, &backup, &ufvk, &context.coin).await?;
    Ok(FrostRestoreResult {
        id_account: restored.account as i32,
        dkg_version: restored.dkg_version as i32,
    })
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

    // Check if signing is already in progress
    let in_progress = crate::frost::sign::in_sign(&mut *connection).await?;
    if in_progress {
        tracing::info!("frost_sign: signing already in progress, running a pass");
        context.driver.step().await?;
    } else {
        // Not in progress, initialize signing
        tracing::info!("frost_sign: initializing signing");
        let id_account = id_account
            .try_into()
            .map_err(|_| invalid_integer("id_account"))?;
        let message_account = message_account
            .try_into()
            .map_err(|_| invalid_integer("message_account"))?;
        let id_coordinator = id_coordinator
            .try_into()
            .map_err(|_| invalid_integer("id_coordinator"))?;
        let pczt = hex::decode(&pczt)?;
        let (pczt, _) = bincode::decode_from_slice(&pczt, config::standard())?;
        crate::frost::sign::init_sign(
            &mut connection,
            id_account,
            message_account,
            id_coordinator,
            &pczt,
        )
        .await?;
    }
    Ok(true)
}
