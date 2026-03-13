use anyhow::{Context, Result};
use bip39::Mnemonic;
use futures::TryStreamExt;
use reddsa::frost::redpallas::Identifier;
use sqlx::{query, query_as, sqlite::SqliteRow, Row, SqliteConnection};
use zcash_keys::address::UnifiedAddress;

use crate::{
    account::{get_orchard_vk, new_account},
    api::{account::NewAccount, coin::Network},
    db::get_prop,
    frost::dkg2::protocol::FrostMap, tiu,
};

pub fn derive_broadcast_seed(mailbox_addresses: &FrostMap<String>) -> Result<Option<String>> {
    if mailbox_addresses.iter().all(|(_, a)| !a.is_empty()) {
        let mut state = blake2b_simd::Params::new()
            .hash_length(32)
            .personal(b"Zcash__FROST_DKG")
            .to_state();
        for a in mailbox_addresses.values() {
            state.update(a.as_bytes());
        }
        let hash = state.finalize();
        let m = Mnemonic::from_entropy(hash.as_ref()).expect("Failed to create mnemonic from hash");
        let seed = m.to_string();
        return Ok(Some(seed));
    }
    Ok(None)
}

#[derive(Debug)]
pub struct FrostParams {
    pub id: u8,
    pub t: u8,
    pub n: u8,
    pub mailbox_addresses: FrostMap<String>,
    pub broadcast_address: Option<String>,
    pub funding: u32,
    pub mailbox: u32,
    pub broadcast: Option<u32>,
    pub name: String,
    pub birth: u32,
}

pub async fn get_frost_params(
    network: &Network,
    connection: &mut SqliteConnection,
) -> Result<Option<FrostParams>> {
    if let Some(funding) = get_prop(&mut *connection, "dkg_account").await? {
        let funding = funding.parse::<u32>()?;
        let (id, n, t, name, birth): (u8, u8, u8, String, u32) = query_as(
            "SELECT id, n, t, name, birth_height
            FROM dkg_params WHERE account = ?1",
        )
        .bind(funding)
        .fetch_one(&mut *connection)
        .await
        .context("get_frost_params::dkg_params")?;
        let mut mailbox_addresses = FrostMap::new();
        {
            let mut address_rows = query(
                "SELECT from_id, data FROM dkg_packages
            WHERE account = ?1 AND public = 1 AND round = 0
            ORDER BY from_id",
            )
            .bind(funding)
            .map(|r: SqliteRow| {
                let id: u16 = r.get(0);
                let address: Vec<u8> = r.get(1);
                (id, String::from_utf8_lossy(&address).to_string())
            })
            .fetch(&mut *connection);
            while let Some((id, address)) = address_rows.try_next().await? {
                let id: Identifier = tiu!(id);
                mailbox_addresses.insert(id, address);
            }
        }
        let broadcast_address = match derive_broadcast_seed(&mailbox_addresses)? {
            Some(seed) => {
                let broadcast_account = match get_account_by_seed(&mut *connection, &seed).await? {
                    Some(broadcast_account) => broadcast_account,
                    None => {
                        new_account(
                            network,
                            &mut *connection,
                            &NewAccount {
                                name: "frost-broadcast".to_string(),
                                key: seed.to_string(),
                                birth: Some(birth),
                                internal: true,
                                ..Default::default()
                            },
                        )
                        .await?
                    }
                };
                let Some(vk) = get_orchard_vk(&mut *connection, broadcast_account).await? else {
                    anyhow::bail!("Broadcast account should have an orchard key");
                };
                let broadcast_address = vk.address_at(0u64, orchard::keys::Scope::External);
                let broadcast_address =
                    UnifiedAddress::from_receivers(Some(broadcast_address), None, None)
                        .unwrap()
                        .encode(network);
                Some(broadcast_address)
            }
            None => None,
        };

        let mailbox = get_internal_account_by_name(&mut *connection, "frost-mailbox")
            .await?
            .context("Mailbox account missing")?;
        let broadcast = get_internal_account_by_name(&mut *connection, "frost-broadcast").await?;

        return Ok(Some(FrostParams {
            id,
            t,
            n,
            mailbox_addresses,
            broadcast_address,
            funding,
            mailbox,
            broadcast,
            name,
            birth,
        }));
    }
    Ok(None)
}

async fn get_internal_account_by_name(
    connection: &mut SqliteConnection,
    name: &str,
) -> Result<Option<u32>> {
    let account = query("SELECT id_account FROM accounts WHERE name = ?1 AND internal = 1")
        .bind(name)
        .map(|r: SqliteRow| r.get::<u32, _>(0))
        .fetch_optional(connection)
        .await?;
    Ok(account)
}

async fn get_account_by_seed(connection: &mut SqliteConnection, seed: &str) -> Result<Option<u32>> {
    let account = query("SELECT id_account FROM accounts WHERE seed = ?1")
        .bind(seed)
        .map(|r: SqliteRow| r.get::<u32, _>(0))
        .fetch_optional(connection)
        .await?;
    Ok(account)
}
