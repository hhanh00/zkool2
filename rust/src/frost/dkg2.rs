use anyhow::Result;
use bincode::config;
use orchard::keys::{FullViewingKey, Scope};
use sqlx::{query, sqlite::SqliteRow, Acquire, Row, SqliteConnection};
use zcash_keys::address::UnifiedAddress;
use zcash_protocol::memo::Memo;

use crate::{
    account::{get_orchard_vk, new_account},
    api::{account::NewAccount, coin::Network, key::generate_seed},
    db::put_prop,
    frost::{
        dkg2::{
            mail::FrostParams,
            pkg::{DKGRound1Public, DKGRound2Public},
            protocol::BinaryRW,
        },
        FrostMessage,
    },
    Client,
};

pub mod mail;
pub mod pkg;
pub mod protocol;

pub async fn process_messages(
    connection: &mut SqliteConnection,
    account: u32,
    params: &FrostParams,
) -> Result<()> {
    tracing::info!("process_messages 1");
    let height = query("SELECT height FROM dkg_params WHERE account = ?1")
        .bind(params.funding)
        .map(|r: SqliteRow| r.get::<Option<u32>, _>(0))
        .fetch_one(&mut *connection)
        .await?
        .unwrap_or(params.birth);
    let memos = query(
        "SELECT memo_bytes FROM memos
        WHERE account = ?1 AND height > ?2",
    )
    .bind(account)
    .bind(height)
    .map(|r: SqliteRow| r.get::<Vec<u8>, _>(0))
    .fetch_all(&mut *connection)
    .await?;
    for memo_bytes in memos {
        let handler = async {
            let memo = Memo::from_bytes(&memo_bytes);
            let Ok(Memo::Arbitrary(pkg_bytes)) = memo else {
                anyhow::bail!("Not a byte memo");
            };
            if pkg_bytes.len() < 4 {
                anyhow::bail!("Too short");
            }
            let (prefix, payload) = pkg_bytes.split_at(4);
            let prefix = String::from_utf8_lossy(prefix).to_string();
            let (message, _) =
                bincode::decode_from_slice::<FrostMessage, _>(payload, config::legacy())?;
            tracing::info!("{prefix} {}", message.from_id);

            let round = match prefix.as_str() {
                "DK12" => {
                    tracing::info!("DKG Round 1 Version 2");
                    DKGRound1Public::try_from_bytes(&message.data)?;
                    Some(1)
                }
                "DK22" => {
                    tracing::info!("DKG Round 2 Version 2");
                    DKGRound2Public::try_from_bytes(&message.data)?;
                    Some(2)
                }
                _ => None,
            };
            tracing::info!("Payload checked");
            if let Some(round) = round {
                protocol::store_dkg_package_item(
                    &mut *connection,
                    params.funding,
                    true,
                    round,
                    message.from_id,
                    &message.data,
                )
                .await?;
            }
            Ok::<_, anyhow::Error>(())
        };
        let _ = handler.await;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn dkg_workflow(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    params: &FrostParams,
) -> Result<()> {
    tracing::info!("WF 0");
    process_messages(&mut *connection, params.mailbox, params).await?;
    tracing::info!("WF 1");
    let Some(broadcast) = params.broadcast else {
        anyhow::bail!("At least one peer address is missing");
    };
    process_messages(&mut *connection, broadcast, params).await?;

    tracing::info!("WF 2");
    let mut db_tx = connection.begin().await?;
    tracing::info!("WF 3");
    let s1p = protocol::get_or_generate::<pkg::DKGRound1Secret>(
        network,
        params.birth,
        &mut db_tx,
        client,
        "DK12",
        1,
        params,
        move || pkg::build_part1(params.id, params.n, params.t),
    )
    .await?;

    let p1p = protocol::get_and_collect(&mut db_tx, 1, params, |data| {
        DKGRound1Public::try_from_bytes(&data)
            .expect("Data was validated before insertion in dkg_packages")
    })
    .await?;
    tracing::info!("R1 public count: {}", p1p.len());
    if p1p.len() < params.n as usize - 1 {
        anyhow::bail!(
            "Need {} more public round 1 packages",
            params.n as usize - p1p.len()
        );
    }
    tracing::info!("WF 4");

    let s2p = protocol::get_or_generate::<pkg::DKGRound2Secret>(
        network,
        params.birth,
        &mut db_tx,
        client,
        "DK22",
        2,
        params,
        || pkg::build_part2(s1p, p1p.clone()),
    )
    .await?;
    let p2p = protocol::get_and_collect(&mut db_tx, 2, params, |data| {
        DKGRound2Public::try_from_bytes(&data)
            .expect("Data was validated before insertion in dkg_packages")
    })
    .await?;
    tracing::info!("R2 public count: {}", p2p.len());
    if p2p.len() < params.n as usize - 1 {
        anyhow::bail!(
            "Need {} more public round 2 packages",
            params.n as usize - p2p.len()
        );
    }
    tracing::info!("WF 5");

    let sk = protocol::get_or_generate::<pkg::DKGRound3Secret>(
        network,
        params.birth,
        &mut db_tx,
        client,
        "",
        3,
        params,
        || pkg::build_part3(s2p, p1p, p2p),
    )
    .await?;


    // Build the shared key out of the public key and parts of the broadcast account
    let fvk = get_orchard_vk(&mut db_tx, broadcast)
        .await?
        .expect("broadcast account vk not found");

    // Replace the first 32 bytes of the FVK with the public key
    // This is the spend authorization key
    let mut fvkb = fvk.to_bytes();
    let vk = sk.0.verifying_key();

    let pkb = vk.serialize().expect("pk serialize");
    fvkb[0..32].copy_from_slice(&pkb);
    let fvk = FullViewingKey::from_bytes(&fvkb).expect("Failed to create shared FVK");
    let address = fvk.address_at(0u64, Scope::External);
    let ua = UnifiedAddress::from_receivers(Some(address), None, None).unwrap();
    let sua = ua.encode(network);
    tracing::info!("Shared address: {sua}");

    // Get all mailbox addresses
    // Generate broadcast address
    // Get Round 1 private key
    // Get Round 1 public commitments
    // Get Round 2 private key
    // Get Round 2 public commitments
    // Get private key share
    // Get pub key share
    db_tx.commit().await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn set_dkg_params(
    network: &Network,
    connection: &mut SqliteConnection,
    client: &mut Client,
    name: &str,
    id: u8,
    n: u8,
    t: u8,
    funding_account: u32,
) -> Result<String> {
    let mut db_tx = connection.begin().await?;
    put_prop(&mut db_tx, "dkg_account", &funding_account.to_string()).await?;

    let mailbox_seed = generate_seed()?;
    let birth = client.latest_height().await? - 1000;
    let mailbox_account = new_account(
        network,
        &mut db_tx,
        &NewAccount {
            name: "frost-mailbox".to_string(),
            key: mailbox_seed.clone(),
            birth: Some(birth),
            internal: true,
            ..NewAccount::default()
        },
    )
    .await?;
    let fvk = get_orchard_vk(&mut db_tx, mailbox_account)
        .await?
        .expect("Orchard Keys should be available");
    let oaddress = fvk.address_at(0u64, Scope::External);
    let address = UnifiedAddress::from_receivers(Some(oaddress), None, None)
        .expect("Orchard only UA should be derivable");
    let mailbox_address = address.encode(network);

    query(
        "INSERT INTO dkg_params(account, name, id, t, n, seed, birth_height)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(funding_account)
    .bind(name)
    .bind(id)
    .bind(t)
    .bind(n)
    .bind(&mailbox_seed)
    .bind(birth)
    .execute(&mut *db_tx)
    .await?;
    set_dkg_address(&mut db_tx, funding_account, id, &mailbox_address).await?;
    db_tx.commit().await?;
    Ok(mailbox_address)
}

pub(crate) async fn set_dkg_address(
    connection: &mut SqliteConnection,
    account: u32,
    id: u8,
    address: &str,
) -> Result<()> {
    protocol::store_dkg_package_item(connection, account, true, 0, id, address.as_bytes()).await?;
    Ok(())
}
