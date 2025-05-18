use anyhow::Result;
use bip39::Mnemonic;
use orchard::keys::{FullViewingKey, Scope};
use sqlx::SqlitePool;
use tracing::info;
use zcash_keys::address::UnifiedAddress;
use zcash_protocol::consensus::Network;
use futures::TryStreamExt;

use crate::{account::get_orchard_vk, api::account::{new_account, NewAccount}};

/// Get (and create if needed) the private mailbox address for
/// communication between all participants in the group
///
pub async fn get_mailbox_account(
    network: &Network,
    connection: &SqlitePool,
    account: u32,
    self_id: u16,
    birth_height: u32,
) -> Result<(u32, String)> {
    let (account, mailbox_address) = loop {
        let address = sqlx::query_as::<_, (String,)>(
            "SELECT data FROM dkg_packages WHERE account = ? AND round = 0
            AND public = 1 AND from_id = ?",
        )
        .bind(account)
        .bind(self_id)
        .fetch_optional(connection)
        .await?;
        let mailbox_account = sqlx::query_as::<_, (u32,)>(
            "SELECT data FROM dkg_packages WHERE account = ? AND round = 0
            AND public = 0",
        )
        .bind(account)
        .bind(self_id)
        .fetch_optional(connection)
        .await?;

        match (address, mailbox_account) {
            (Some((mailbox_address, )), Some((mailbox_account, ))) => {
                break (mailbox_account, mailbox_address);
            }
            (None, None) => {
                // The account does not exist, create it
                let na = NewAccount {
                    name: "frost-mailbox".to_string(),
                    icon: None,
                    restore: true,
                    key: String::new(),
                    passphrase: None,
                    fingerprint: None,
                    aindex: 0,
                    birth: Some(birth_height),
                    use_internal: false,
                    internal: true,
                };
                let mailbox_account = new_account(&na).await?;
                let fvk = get_orchard_vk(connection, mailbox_account).await?.expect("Mailbox account should have orchard");
                let address = fvk.address_at(0u64, Scope::External);
                let ua = UnifiedAddress::from_receivers(Some(address), None, None).unwrap();
                let ua = ua.encode(network);
                sqlx::query(
                    "INSERT INTO dkg_packages (account, round, public, from_id, data)
                    VALUES (?1, 0, 1, ?2, ?3)",
                )
                .bind(account)
                .bind(self_id)
                .bind(ua)
                .execute(connection)
                .await?;
                sqlx::query(
                    "INSERT INTO dkg_packages (account, round, public, from_id, data)
                    VALUES (?1, 0, 0, ?2, ?3)",
                )
                .bind(account)
                .bind(self_id)
                .bind(mailbox_account)
                .execute(connection)
                .await?;
            }
            _ => unreachable!(),
        }
    };

    Ok((account, mailbox_address))
}

/// Get (and create if needed) the shared broadcast address for
/// communication between all participants in the group
/// It is derived from the hash of the participant private mailbox addresses
///
pub async fn get_coordinator_broadcast_account(
    network: &Network,
    connection: &SqlitePool,
    account: u32,
    height: u32,
) -> Result<(u32, String)> {
    let addresses = sqlx::query_as::<_, (String,)>(
        "SELECT data FROM dkg_packages WHERE account = ?1 AND round = 0
        AND public = 1 ORDER BY from_id",
    )
    .bind(account)
    .fetch_all(connection)
    .await?;
    let addresses = addresses.into_iter().map(|row| row.0).collect::<Vec<_>>();

    let mut state = blake2b_simd::Params::new()
        .hash_length(32)
        .personal(b"Zcash__FROST_DKG")
        .to_state();
    for a in addresses.iter() {
        state.update(a.as_bytes());
    }
    let hash = state.finalize();
    let m = Mnemonic::from_entropy(hash.as_ref()).expect("Failed to create mnemonic from hash");
    let seed = m.to_string();

    let (account, broadcast_address) = loop {
        // Check if the account already exists
        let r = sqlx::query_as::<_, (u32, Vec<u8>)>(
            "SELECT a.id_account, o.xvk FROM accounts a
            JOIN orchard_accounts o ON a.id_account = o.account
            WHERE seed = ?1",
        )
        .bind(&seed)
        .fetch_optional(connection)
        .await?;

        match r {
            None => {
                // The account does not exist, create it
                let na = NewAccount {
                    name: "frost-broadcast".to_string(),
                    icon: None,
                    restore: true,
                    key: seed.to_string(),
                    passphrase: None,
                    fingerprint: None,
                    aindex: 0,
                    birth: Some(height),
                    use_internal: false,
                    internal: true,
                };
                new_account(&na).await?;
                // Loop again to retrieve the account
            }
            Some((account, xvk)) => {
                let fvk = FullViewingKey::from_bytes(&xvk.try_into().unwrap())
                    .expect("Failed to create shared FVK");
                let address = fvk.address_at(0u64, Scope::External);
                let ua = UnifiedAddress::from_receivers(Some(address), None, None).unwrap();
                let broadcast_address = ua.encode(network);
                info!("Broadcast address: {broadcast_address}");
                break (account, broadcast_address);
            }
        }
    };

    Ok((account, broadcast_address))
}

pub async fn get_addresses(connection: &SqlitePool, account: u32, n: u8) -> Result<Vec<String>> {
    let mut addresses = vec![String::new(); n as usize];
    let mut rs = sqlx::query_as::<_, (u16, String,)>(
        "SELECT from_id, data FROM dkg_packages WHERE account = ?1 AND round = 0
        AND public = 1",
    )
    .bind(account)
    .fetch(connection);
    while let Some((from_id, address)) = rs.try_next().await? {
        addresses[(from_id - 1) as usize] = address;
    }

    Ok(addresses)
}