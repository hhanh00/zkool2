use anyhow::Result;
use bip39::Mnemonic;
use futures::TryStreamExt;
use orchard::keys::{FullViewingKey, Scope};
use sqlx::SqliteConnection;
use tracing::info;
use zcash_keys::address::UnifiedAddress;
use crate::coin::Network;

use crate::{
    account::get_orchard_vk,
    api::account::{get_account_seed, new_account, NewAccount},
};

/// Get (and create if needed) the private mailbox address for
/// communication between all participants in the group
///
pub async fn get_mailbox_account(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    self_id: u16,
    birth_height: u32,
) -> Result<(u32, String)> {
    let mut retry = 0;
    let (account, mailbox_address) = loop {
        if retry > 1 {
            anyhow::bail!("Failed to create mailbox account");
        }

        // seed or empty string if not set
        let seed = sqlx::query_as::<_, (String,)>("SELECT seed FROM dkg_params WHERE account = ?1")
            .bind(account)
            .fetch_optional(&mut *connection)
            .await?
            .map(|row| row.0)
            .unwrap_or_default();

        let address = sqlx::query_as::<_, (Vec<u8>,)>(
            "SELECT data FROM dkg_packages WHERE account = ? AND round = 0
            AND public = 1 AND from_id = ?",
        )
        .bind(account)
        .bind(self_id)
        .fetch_optional(&mut *connection)
        .await?
        .map(|a| String::from_utf8(a.0).expect("Failed to convert utf8"));
        let mailbox_account = if !seed.is_empty() {
            sqlx::query_as::<_, (u32,)>("SELECT id_account FROM accounts WHERE seed = ?1")
                .bind(&seed)
                .fetch_optional(&mut *connection)
                .await?
        } else {
            None
        };

        match (address, mailbox_account) {
            (Some(mailbox_address), Some((mailbox_account,))) => {
                break (mailbox_account, mailbox_address);
            }
            (_, None) => {
                info!("Creating mailbox account");
                // The account does not exist, create it with a random seed
                let na = NewAccount {
                    name: "frost-mailbox".to_string(),
                    icon: None,
                    restore: true,
                    key: seed.clone(),
                    passphrase: None,
                    fingerprint: None,
                    aindex: 0,
                    birth: Some(birth_height),
                    folder: "".to_string(),
                    pools: None,
                    use_internal: false,
                    internal: true,
                    ledger: false,
                };
                let mailbox_account = new_account(&na).await?;
                let fvk = get_orchard_vk(&mut *connection, mailbox_account)
                    .await?
                    .expect("Mailbox account should have orchard");
                let address = fvk.address_at(0u64, Scope::External);
                let ua = UnifiedAddress::from_receivers(Some(address), None, None).unwrap();
                let ua = ua.encode(network);
                sqlx::query(
                    "INSERT INTO dkg_packages (account, round, public, from_id, data)
                    VALUES (?1, 0, 1, ?2, ?3) ON CONFLICT DO NOTHING",
                )
                .bind(account)
                .bind(self_id)
                .bind(ua)
                .execute(&mut *connection)
                .await?;
                let seed = get_account_seed(mailbox_account)
                    .await?
                    .expect("Seed should be set");
                sqlx::query("UPDATE dkg_params SET seed = ?1 WHERE account = ?2")
                    .bind(&seed.mnemonic)
                    .bind(account)
                    .execute(&mut *connection)
                    .await?;
            }
            _ => unreachable!(),
        }
        retry += 1;
    };

    Ok((account, mailbox_address))
}

/// Get (and create if needed) the shared broadcast address for
/// communication between all participants in the group
/// It is derived from the hash of the participant private mailbox addresses
///
pub async fn get_coordinator_broadcast_account(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<(u32, String)> {
    let addresses = sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT data FROM dkg_packages WHERE account = ?1 AND round = 0
        AND public = 1 ORDER BY from_id",
    )
    .bind(account)
    .fetch_all(&mut *connection)
    .await?;
    let addresses = addresses
        .into_iter()
        .map(|row| String::from_utf8(row.0).unwrap())
        .collect::<Vec<_>>();

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
        .fetch_optional(&mut *connection)
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
                    folder: "".to_string(),
                    pools: None,
                    use_internal: false,
                    internal: true,
                    ledger: false,
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

pub async fn get_addresses(connection: &mut SqliteConnection, account: u32, n: u8) -> Result<Vec<String>> {
    let mut addresses = vec![String::new(); n as usize];
    let mut rs = sqlx::query_as::<_, (u16, Vec<u8>)>(
        "SELECT from_id, data FROM dkg_packages WHERE account = ?1 AND round = 0
        AND public = 1",
    )
    .bind(account)
    .fetch(connection);
    while let Some((from_id, address)) = rs.try_next().await? {
        addresses[(from_id - 1) as usize] = String::from_utf8(address).unwrap();
    }

    Ok(addresses)
}
