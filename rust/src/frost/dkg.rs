use std::collections::BTreeMap;

use super::{
    db::{get_addresses, get_coordinator_broadcast_account, get_mailbox_account},
    to_arb_memo, PK1Map, PK2Map,
};
use anyhow::{Context, Result};
use bincode::config;
use futures::StreamExt as _;
use orchard::keys::{FullViewingKey, Scope};
use rand_core::OsRng;
use reddsa::frost::redpallas::keys::{
    dkg::{self, round1, round2},
    EvenY,
};
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use tracing::info;
use zcash_keys::address::UnifiedAddress;
use zcash_protocol::{consensus::Network, memo::Memo};

use crate::{
    account::get_orchard_vk,
    api::{
        account::{delete_account, get_account_seed},
        frost::{DKGParams, DKGStatus},
        sync::SYNCING,
    },
    db::{init_account_orchard, store_account_metadata, store_account_orchard_vk},
    frb_generated::StreamSink,
    frost::FrostMessage,
    pay::{
        plan::{extract_transaction, plan_transaction, sign_transaction},
        pool::ALL_POOLS,
        Recipient,
    },
    Client,
};

pub async fn set_dkg_address(
    connection: &sqlx::Pool<sqlx::Sqlite>,
    account: u32,
    id: u16,
    address: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO dkg_packages(account, public, round, from_id, data)
        VALUES (?, TRUE, 0, ?, ?) ON CONFLICT DO NOTHING",
    )
    .bind(account)
    .bind(id)
    .bind(address)
    .execute(connection)
    .await?;
    Ok(())
}

/// Get the round 1 secret package from the database
///
async fn get_spkg1(connection: &SqlitePool, account: u32) -> Result<Option<round1::SecretPackage>> {
    let spkg = sqlx::query(
        "SELECT data FROM dkg_packages WHERE
            account = ? AND public = 0 AND round = 1",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let data: Vec<u8> = row.get(0);
        let spkg =
            round1::SecretPackage::deserialize(&data).expect("Failed to decode SecretPackage");
        spkg
    })
    .fetch_optional(connection)
    .await?;
    info!("Secret package: {:?}", spkg);
    Ok(spkg)
}

/// Get the round 2 secret package from the database
///
async fn get_spkg2(connection: &SqlitePool, account: u32) -> Result<Option<round2::SecretPackage>> {
    let spkg = sqlx::query(
        "SELECT data FROM dkg_packages WHERE
            account = ? AND public = 0 AND round = 2",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let data: Vec<u8> = row.get(0);
        let spkg =
            round2::SecretPackage::deserialize(&data).expect("Failed to decode SecretPackage");
        spkg
    })
    .fetch_optional(connection)
    .await?;
    info!("Secret package: {:?}", spkg);
    Ok(spkg)
}

/// Get the round 1 public packages from the database
/// and return them as a BTreeMap
async fn get_ppkg1(connection: &SqlitePool, account: u32, self_id: u16) -> Result<PK1Map> {
    let mut pkg1map: PK1Map = BTreeMap::new();

    let pkgs = sqlx::query(
        "SELECT from_id, data FROM dkg_packages WHERE
            account = ?1 AND public = 1 AND round = 1 AND from_id != ?2",
    )
    .bind(account)
    .bind(self_id)
    .map(|row: SqliteRow| {
        let id: u16 = row.get(0);
        let data: Vec<u8> = row.get(1);
        let pkg = round1::Package::deserialize(&data).expect("Failed to decode round1::Package");
        (id, pkg)
    })
    .fetch_all(connection)
    .await?;

    for (id, pkg) in pkgs {
        pkg1map.insert(id.try_into().unwrap(), pkg);
    }

    Ok(pkg1map)
}

/// Get the round 2 public packages from the database
/// Return them as a BTreeMap
///
async fn get_ppkg2(connection: &SqlitePool, account: u32) -> Result<PK2Map> {
    let mut pkg2map: PK2Map = BTreeMap::new();
    let pkgs = sqlx::query(
        "SELECT from_id, data FROM dkg_packages WHERE
            account = ?1 AND public = 1 AND round = 2",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let id: u16 = row.get(0);
        let data: Vec<u8> = row.get(1);
        let pkg = round2::Package::deserialize(&data).expect("Failed to decode round2::Package");
        (id, pkg)
    })
    .fetch_all(connection)
    .await?;

    for (id, pkg) in pkgs {
        pkg2map.insert(id.try_into().unwrap(), pkg);
    }
    Ok(pkg2map)
}

pub async fn have_all_addresses(connection: &SqlitePool, account: u32, n: u8) -> Result<bool> {
    let addresses = get_addresses(connection, account, n).await?;
    let have_all_addresses = addresses.iter().all(|a| !a.is_empty());
    Ok(have_all_addresses)
}

pub async fn get_dkg_params(connection: &SqlitePool, account: u32) -> Result<DKGParams> {
    let dkg_params = sqlx::query("SELECT id, n, t, birth_height FROM dkg_params WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let id: u16 = row.get(0);
            let n: u8 = row.get(1);
            let t: u8 = row.get(2);
            let birth_height: u32 = row.get(3);
            DKGParams {
                id,
                n,
                t,
                birth_height,
            }
        })
        .fetch_one(connection)
        .await
        .context("Fetch id, n, t, ...")?;

    Ok(dkg_params)
}

pub async fn do_dkg(
    network: &Network,
    connection: &SqlitePool,
    account: u32,
    client: &mut Client,
    height: u32,
    status: StreamSink<DKGStatus>,
) -> Result<()> {
    info!("dkg: {account}");

    let guard = SYNCING.try_lock();
    if guard.is_err() {
        return Ok(());
    }

    let dkg_params = get_dkg_params(connection, account).await?;
    let DKGParams {
        id: self_id,
        n,
        t,
        birth_height,
    } = dkg_params;
    let addresses = get_addresses(connection, account, n).await?;

    // Create a mailbox account if it doesn't exist
    let (mailbox_account, _mailbox_address) =
        get_mailbox_account(network, connection, account, self_id, birth_height).await?;

    let (broadcast_account, broadcast_address) =
        get_coordinator_broadcast_account(network, connection, account, birth_height).await?;

    let Some(spkg1) = get_spkg1(connection, account).await? else {
        let (spkg1, ppkg1) =
            dkg::part1::<_>(self_id.try_into().unwrap(), n as u16, t as u16, OsRng)?;
        // in round 1, every other participant receives the same public package from us
        status
            .add(DKGStatus::PublishRound1Pkg)
            .map_err(anyhow::Error::msg)?;
        publish_round1(
            network,
            connection,
            account,
            self_id,
            client,
            height,
            &broadcast_address,
            &spkg1,
            &ppkg1,
        )
        .await?;
        status
            .add(DKGStatus::WaitRound1Pkg)
            .map_err(anyhow::Error::msg)?;
        return Ok(());
    };

    process_memos(connection, account, broadcast_account, 1, b"DK11").await?;
    let ppkg1s = get_ppkg1(connection, account, self_id).await?;
    info!("Round 1 packages: {}", ppkg1s.len());
    if ppkg1s.len() != n as usize - 1 {
        status
            .add(DKGStatus::WaitRound1Pkg)
            .map_err(anyhow::Error::msg)?;
        return Ok(());
    }

    info!("Round 1 Complete");

    let Some(spkg2) = get_spkg2(connection, account).await? else {
        let (spkg2, ppkg2s) = dkg::part2(spkg1, &ppkg1s)?;
        status
            .add(DKGStatus::PublishRound2Pkg)
            .map_err(anyhow::Error::msg)?;
        publish_round2(
            network, connection, account, self_id, client, height, &addresses, &spkg2, &ppkg2s,
        )
        .await?;
        status
            .add(DKGStatus::WaitRound2Pkg)
            .map_err(anyhow::Error::msg)?;
        return Ok(());
    };
    process_memos(connection, account, mailbox_account, 2, b"DK21").await?;
    let ppkg2s = get_ppkg2(connection, account).await?;
    info!("Round 2 packages: {}", ppkg2s.len());
    if ppkg2s.len() != n as usize - 1 {
        status
            .add(DKGStatus::WaitRound2Pkg)
            .map_err(anyhow::Error::msg)?;
        return Ok(());
    }

    info!("Round 2 Complete");

    let (sk, pk) = dkg::part3(&spkg2, &ppkg1s, &ppkg2s)?;

    // Save the sk and pk to the database
    sqlx::query(
        "INSERT INTO dkg_packages(account, public, round, from_id, data)
        VALUES (?, FALSE, 3, ?, ?)",
    )
    .bind(account)
    .bind(self_id)
    .bind(sk.serialize()?)
    .execute(connection)
    .await?;
    sqlx::query(
        "INSERT INTO dkg_packages(account, public, round, from_id, data)
        VALUES (?, TRUE, 3, ?, ?)",
    )
    .bind(account)
    .bind(self_id)
    .bind(pk.serialize()?)
    .execute(connection)
    .await?;

    // Build the shared key out of the public key and parts of the broadcast account
    let fvk = get_orchard_vk(connection, broadcast_account)
        .await?
        .expect("broadcast account vk not found");

    // Replace the first 32 bytes of the FVK with the public key
    // This is the spend authorization key
    let mut fvkb = fvk.to_bytes();
    let pk = pk.into_even_y(None);
    let vk = pk.verifying_key();

    let pkb = vk.serialize().expect("pk serialize");
    fvkb[0..32].copy_from_slice(&pkb);
    let fvk = FullViewingKey::from_bytes(&fvkb).expect("Failed to create shared FVK");
    let address = fvk.address_at(0u64, Scope::External);
    let ua = UnifiedAddress::from_receivers(Some(address), None, None).unwrap();
    let sua = ua.encode(network);
    info!("Shared address: {sua}");

    let (name,) = sqlx::query_as::<_, (String,)>("SELECT value FROM props WHERE key = 'dkg_name'")
        .fetch_one(connection)
        .await?;
    let frost_account =
        store_account_metadata(connection, &name, &None, &None, height, false, false).await?;
    init_account_orchard(connection, frost_account, height).await?;
    store_account_orchard_vk(connection, frost_account, &fvk).await?;

    dkg_finalize(
        connection,
        account,
        frost_account,
        mailbox_account,
        broadcast_account,
    )
    .await?;

    status
        .add(DKGStatus::SharedAddress(sua))
        .map_err(anyhow::Error::msg)?;

    cancel_dkg(connection, Some(account)).await?;
    Ok(())
}

async fn dkg_finalize(
    connection: &SqlitePool,
    account: u32,
    frost_account: u32,
    mailbox_account: u32,
    broadcast_account: u32,
) -> Result<()> {
    // Reassign dkg_params and dkg_packages to the frost account
    sqlx::query("UPDATE dkg_params SET account = ?1 WHERE account = ?2")
        .bind(frost_account)
        .bind(account)
        .execute(connection)
        .await?;
    sqlx::query("UPDATE dkg_packages SET account = ?1 WHERE account = ?2")
        .bind(frost_account)
        .bind(account)
        .execute(connection)
        .await?;
    // Delete the dkg_* keys from the props table
    sqlx::query("DELETE FROM props WHERE key LIKE 'dkg_%'")
        .execute(connection)
        .await?;
    let seed = get_account_seed(mailbox_account)
        .await?
        .expect("mailbox seed not found")
        .mnemonic;
    sqlx::query("UPDATE dkg_params SET seed = ?1 WHERE account = ?2")
        .bind(seed)
        .bind(frost_account)
        .execute(connection)
        .await?;
    delete_account(mailbox_account).await?;
    delete_account(broadcast_account).await?;
    Ok(())
}

async fn publish_round1(
    network: &Network,
    connection: &SqlitePool,
    account: u32,
    self_id: u16,
    client: &mut Client,
    height: u32,
    broadcast_address: &str,
    spkg1: &round1::SecretPackage,
    ppkg1: &round1::Package,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO dkg_packages(account, public, round, from_id, data)
        VALUES (?, FALSE, 1, ?, ?) ON CONFLICT DO NOTHING",
    )
    .bind(account)
    .bind(self_id)
    .bind(spkg1.serialize()?)
    .execute(connection)
    .await?;

    let message = FrostMessage {
        from_id: self_id,
        data: ppkg1.serialize()?,
    };
    let data = message.encode_with_prefix(b"DK11")?;

    let txid = publish(
        network,
        connection,
        account,
        client,
        height,
        &[(&broadcast_address, data)],
    )
    .await?;
    info!("Broadcasted transaction: {txid}");

    Ok(())
}

async fn publish_round2(
    network: &Network,
    connection: &SqlitePool,
    account: u32,
    self_id: u16,
    client: &mut Client,
    height: u32,
    addresses: &[String],
    spkg2: &round2::SecretPackage,
    ppkg2s: &PK2Map,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO dkg_packages(account, public, round, from_id, data)
        VALUES (?, FALSE, 2, ?, ?) ON CONFLICT DO NOTHING",
    )
    .bind(account)
    .bind(self_id)
    .bind(spkg2.serialize()?)
    .execute(connection)
    .await?;

    let mut recipients = vec![];
    for (idx, address) in addresses.iter().enumerate() {
        let id = (idx + 1) as u16;
        if let Some(ppkg2) = ppkg2s.get(&id.try_into().unwrap()) {
            let message = FrostMessage {
                from_id: self_id,
                data: ppkg2.serialize()?,
            };
            let data = message.encode_with_prefix(b"DK21")?;
            recipients.push((address.as_str(), data));
        }
    }

    let txid = publish(network, connection, account, client, height, &recipients).await?;
    info!("Broadcasted transaction: {txid}");

    Ok(())
}

pub async fn publish(
    network: &Network,
    connection: &SqlitePool,
    account: u32,
    client: &mut Client,
    height: u32,
    recipients: &[(&str, Vec<u8>)],
) -> Result<String, anyhow::Error> {
    let recipients = recipients
        .iter()
        .map(|(address, data)| Recipient {
            address: address.to_string(),
            amount: 0,
            pools: None,
            user_memo: None,
            memo_bytes: Some(to_arb_memo(data)),
        })
        .collect::<Vec<_>>();
    let pczt = plan_transaction(
        network,
        connection,
        client,
        account,
        ALL_POOLS,
        &recipients,
        false,
    )
    .await?;
    let pczt = sign_transaction(connection, account, &pczt).await?;
    let txb = extract_transaction(&pczt).await?;
    let txid = crate::pay::send(client, height as u32, &txb).await?;
    Ok(txid)
}

async fn process_memos(
    connection: &SqlitePool,
    account: u32,
    mailbox_account: u32,
    round: u8,
    prefix: &[u8],
) -> Result<()> {
    info!("process_memos: {account} {mailbox_account}");
    let mut pkgs = sqlx::query("SELECT memo_bytes FROM memos WHERE account = ?")
        .bind(mailbox_account)
        .map(|row: SqliteRow| {
            let memo_bytes: Vec<u8> = row.get(0);
            let memo = Memo::from_bytes(&memo_bytes);
            if let Ok(memo) = memo {
                match memo {
                    Memo::Arbitrary(pkg_bytes) => {
                        if pkg_bytes.len() < 4 || pkg_bytes[0..4] != *prefix {
                            return None;
                        }
                        if let Ok((pkg, _)) = bincode::decode_from_slice::<FrostMessage, _>(
                            &pkg_bytes[4..],
                            config::legacy(),
                        )
                        .context("Failed to decode DKGRound1Package")
                        {
                            return Some(pkg);
                        }
                    }
                    _ => (),
                }
            }
            None
        })
        .fetch(connection);
    while let Some(pkg) = pkgs.next().await {
        if let Some(pkg) = pkg? {
            sqlx::query(
                r#"INSERT INTO dkg_packages(account, public, round, from_id, data)
                VALUES (?, TRUE, ?, ?, ?) ON CONFLICT DO NOTHING"#,
            )
            .bind(account)
            .bind(round)
            .bind(pkg.from_id)
            .bind(&pkg.data)
            .execute(connection)
            .await?;
        }
    }

    Ok(())
}

pub async fn cancel_dkg(connection: &SqlitePool, account: Option<u32>) -> Result<()> {
    if let Some(account) = account {
        sqlx::query("DELETE FROM dkg_packages WHERE account = ?")
            .bind(account)
            .execute(connection)
            .await?;
        sqlx::query("DELETE FROM dkg_params WHERE account = ?")
            .bind(account)
            .execute(connection)
            .await?;
        sqlx::query("DELETE FROM dkg_packages WHERE account = ?")
            .bind(account)
            .execute(connection)
            .await?;
    }
    sqlx::query("DELETE FROM props WHERE key LIKE 'dkg_%'")
        .execute(connection)
        .await?;
    delete_frost_accounts(connection).await
}

pub async fn delete_frost_accounts(connection: &SqlitePool) -> Result<()> {
    let frost_accounts = sqlx::query_as::<_, (u32,)>(
        "SELECT id_account FROM accounts WHERE name LIKE 'frost-%' AND internal = 1",
    )
    .fetch_all(connection)
    .await?;
    for (frost_account,) in frost_accounts {
        delete_account(frost_account).await?;
    }

    Ok(())
}
