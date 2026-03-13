use std::collections::BTreeMap;

use crate::{
    api::coin::Network,
    frost::{dkg::publish, dkg2::mail::FrostParams, FrostMessage},
    tiu, Client,
};
use anyhow::{Context, Result};
use either::Either;
use reddsa::frost::redpallas::Identifier;
use sqlx::{query, sqlite::SqliteRow, Row, SqliteConnection};
use std::fmt::Debug;

pub trait BinaryRW: Sized {
    fn try_from_bytes(bytes: &[u8]) -> Result<Self>;
    fn to_bytes(&self) -> Vec<u8>;
}

pub trait SecretData: Debug + BinaryRW + Sized + Send + Sync {
    type Public: PublicData;
}

pub trait PublicData: Debug + BinaryRW + Sized + Send + Sync + Unpin {}

pub type FrostMap<T> = BTreeMap<Identifier, T>;

#[allow(clippy::too_many_arguments)]
pub async fn get_or_generate<T: SecretData>(
    network: &Network,
    height: u32,
    connection: &mut SqliteConnection,
    client: &mut Client,
    prefix: &str,
    round: u8,
    params: &FrostParams,
    builder: impl FnOnce() -> Result<(T, Either<T::Public, FrostMap<T::Public>>)>,
) -> Result<T> {
    tracing::info!("get_or_generate 0");
    let s = match get_opt_dkg_package_item(connection, params.funding, false, round, params.id)
        .await?
    {
        Some(data) => T::try_from_bytes(&data)?,
        None => {
            tracing::info!("get_or_generate 1");
            let (s, p) = builder()?;
            store_dkg_package_item(
                &mut *connection,
                params.funding,
                false,
                round,
                params.id,
                &s.to_bytes(),
            )
            .await?;

            tracing::info!("get_or_generate 2");
            tracing::info!("{params:?}");
            if !prefix.is_empty() {
                match p {
                    Either::Left(p) => {
                        let m = FrostMessage {
                            from_id: params.id,
                            data: p.to_bytes(),
                        };
                        let m = m.encode_with_prefix(prefix.as_bytes())?;
                        let broadcast_address = params
                            .broadcast_address
                            .as_ref()
                            .expect("Must have broadcast address");
                        publish(
                            network,
                            connection,
                            params.funding,
                            client,
                            height,
                            &[(broadcast_address, m)],
                        )
                        .await?;
                    }
                    Either::Right(p) => {
                        let mut recipients = vec![];
                        for (id, p) in p {
                            let m = FrostMessage {
                                from_id: params.id,
                                data: p.to_bytes(),
                            };
                            let m = m.encode_with_prefix(prefix.as_bytes())?;
                            recipients.push((&*params.mailbox_addresses[&id], m));
                        }
                        publish(
                            network,
                            connection,
                            params.funding,
                            client,
                            height,
                            &recipients,
                        )
                        .await?;
                    }
                }
            }
            s
        }
    };
    Ok(s)
}

pub async fn try_get<T: PublicData>(
    connection: &mut SqliteConnection,
    account: u32,
    id: u16,
    round: i8,
) -> Result<Option<T>> {
    let data = query(
        "SELECT data FROM dkg_packages
        WHERE account = ?1 AND public = 0 AND from_id = ?2
        AND round = ?3",
    )
    .bind(account)
    .bind(id)
    .bind(round)
    .map(|r: SqliteRow| {
        let data: Vec<u8> = r.get(0);
        T::try_from_bytes(&data)
    })
    .fetch_optional(connection)
    .await?;
    data.transpose()
}

pub async fn get_and_collect<P: Send + Unpin>(
    connection: &mut SqliteConnection,
    round: u8,
    params: &FrostParams,
    convert: impl (Fn(Vec<u8>) -> P) + Send + Sync,
) -> Result<BTreeMap<Identifier, P>> {
    let items = query(
        "SELECT from_id, data FROM dkg_packages
        WHERE account = ?1 AND public = 1 AND round = ?2
        ORDER BY from_id",
    )
    .bind(params.funding)
    .bind(round)
    .map(|r: SqliteRow| {
        let id: u8 = r.get(0);
        let data: Vec<u8> = r.get(1);
        let public_data = convert(data);
        (id, public_data)
    })
    .fetch_all(&mut *connection)
    .await?;
    let mut map: BTreeMap<Identifier, P> = BTreeMap::new();
    for (id, data) in items {
        if id != params.id {
            // skip our own package
            map.insert(tiu!(id as u16), data);
        }
    }
    Ok(map)
}

pub async fn get_dkg_package_item(
    connection: &mut SqliteConnection,
    account: u32,
    public: bool,
    round: u8,
    id: u8,
) -> Result<Vec<u8>> {
    let data = get_opt_dkg_package_item(connection, account, public, round, id)
        .await?
        .context("get_dkg_package_item")?;
    Ok(data)
}

pub async fn get_opt_dkg_package_item(
    connection: &mut SqliteConnection,
    account: u32,
    public: bool,
    round: u8,
    id: u8,
) -> Result<Option<Vec<u8>>> {
    let data = query(
        "SELECT data FROM dkg_packages
        WHERE account = ?1 AND public = ?2
        AND round = ?3 AND from_id = ?4",
    )
    .bind(account)
    .bind(public)
    .bind(round)
    .bind(id)
    .map(|r: SqliteRow| r.get::<Vec<u8>, _>(0))
    .fetch_optional(connection)
    .await
    .context("get_dkg_package_item")?;
    Ok(data)
}

pub async fn store_dkg_package_item(
    connection: &mut SqliteConnection,
    account: u32,
    public: bool,
    round: u8,
    id: u8,
    data: &[u8],
) -> Result<()> {
    query(
        "INSERT INTO dkg_packages
    (account, public, round, from_id, data)
    VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT DO NOTHING",
    )
    .bind(account)
    .bind(public)
    .bind(round)
    .bind(id)
    .bind(data)
    .execute(connection)
    .await?;
    Ok(())
}
