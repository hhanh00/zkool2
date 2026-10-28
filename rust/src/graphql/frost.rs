use bincode::config;
use juniper::{FieldError, FieldResult};

use crate::{
    frost::{driver::Cancel, keys::CURRENT_DKG_VERSION},
    graphql::{
        data::{FrostParamsResult, FrostRestoreResult},
        Context,
    },
};

fn invalid_integer(name: &str) -> FieldError {
    FieldError::new(format!("Invalid {name}"), juniper::Value::Null)
}

pub async fn dkg_start(
    name: String,
    threshold: i32,
    participants: i32,
    message_account: i32,
    id_participant: i32,
    dkg_version: Option<i32>,
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
    let dkg_version = match dkg_version {
        None => CURRENT_DKG_VERSION,
        Some(v) if (0..=i32::from(CURRENT_DKG_VERSION)).contains(&v) => v as u8,
        Some(_) => return Err(invalid_integer("dkg_version")),
    };

    let coin = &context.coin;
    crate::api::frost::set_dkg_params_versioned(
        &name,
        id_participant,
        participants,
        threshold,
        message_account,
        dkg_version,
        coin,
    )
    .await?;
    crate::api::frost::init_dkg(coin).await?;
    let addresses = crate::api::frost::get_dkg_addresses(coin).await?;
    let address = addresses[usize::from(id_participant) - 1].clone();
    Ok(address)
}

pub async fn dkg_cancel(context: &Context) -> FieldResult<bool> {
    context.driver.cancel(Cancel::Dkg).await?;
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

/// A frost account's parameters, DKG version, sibling and recoverability.
pub async fn frost_params(
    id_account: i32,
    context: &Context,
) -> FieldResult<Option<FrostParamsResult>> {
    let id_account = id_account
        .try_into()
        .map_err(|_| invalid_integer("id_account"))?;
    let params = crate::api::account::get_account_frost_params(id_account, &context.coin).await?;
    Ok(params.map(|p| FrostParamsResult {
        id: p.id.into(),
        n: p.n.into(),
        t: p.t.into(),
        dkg_version: p.dkg_version.into(),
        sibling: p.sibling.map(|s| s as i32),
        quantum_recoverable: p.quantum_recoverable,
    }))
}

/// Create (or return) the ZIP 2005 sibling of a legacy frost account.
pub async fn frost_upgrade(id_account: i32, context: &Context) -> FieldResult<i32> {
    let id_account = id_account
        .try_into()
        .map_err(|_| invalid_integer("id_account"))?;
    let sibling = crate::api::frost::upgrade_frost_account(id_account, &context.coin).await?;
    Ok(sibling as i32)
}

/// Plan the sweep of a legacy frost account into its sibling and start the
/// signing session for it, as `frostSign` would. Returns the PCZT package
/// (hex) the other members pass to `frostSign`.
pub async fn frost_sweep(
    id_account: i32,
    id_coordinator: i32,
    message_account: i32,
    context: &Context,
) -> FieldResult<String> {
    let account: u32 = id_account
        .try_into()
        .map_err(|_| invalid_integer("id_account"))?;
    let message_account = message_account
        .try_into()
        .map_err(|_| invalid_integer("message_account"))?;
    let id_coordinator = id_coordinator
        .try_into()
        .map_err(|_| invalid_integer("id_coordinator"))?;
    let coin = &context.coin;
    let pczt = crate::api::frost::plan_frost_sweep(account, coin).await?;
    let mut connection = coin.get_connection().await?;
    crate::frost::sign::init_sign(&mut connection, account, message_account, id_coordinator, &pczt)
        .await?;
    Ok(hex::encode(bincode::encode_to_vec(&pczt, config::standard())?))
}

/// The member's `zkfrost` backup string for a frost account.
pub async fn frost_export_backup(id_account: i32, context: &Context) -> FieldResult<String> {
    let id_account = id_account
        .try_into()
        .map_err(|_| invalid_integer("id_account"))?;
    Ok(crate::api::frost::export_frost_backup(id_account, &context.coin).await?)
}

/// Restore frost accounts from a backup string and the group's UFVK: the
/// generations asked for, or the one the key matches when none is.
pub async fn frost_restore(
    name: String,
    backup: String,
    ufvk: String,
    legacy: Option<bool>,
    zip2005: Option<bool>,
    birth: Option<i32>,
    context: &Context,
) -> FieldResult<Vec<FrostRestoreResult>> {
    let coin = &context.coin;
    let (legacy, zip2005) = match (legacy, zip2005) {
        (None, None) => {
            let matched = crate::api::frost::match_frost_backup(&backup, &ufvk, coin)?;
            (matched < 2, matched >= 2)
        }
        (l, z) => (l.unwrap_or(false), z.unwrap_or(false)),
    };
    let birth = birth
        .map(|b| u32::try_from(b).map_err(|_| invalid_integer("birth")))
        .transpose()?;
    let restored = crate::api::frost::restore_frost_backup(
        &name, &backup, &ufvk, legacy, zip2005, birth, coin,
    )
    .await?;
    Ok(restored
        .into_iter()
        .map(|r| FrostRestoreResult {
            id_account: r.account as i32,
            dkg_version: r.dkg_version as i32,
        })
        .collect())
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
