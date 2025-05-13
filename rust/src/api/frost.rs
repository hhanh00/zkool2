use anyhow::Result;
use bincode::config;
use flutter_rust_bridge::frb;
use orchard::keys::Scope;
use rand_core::OsRng;
use reddsa::frost::redpallas::{
    frost::{self, keys::dkg::round1},
    PallasBlake2b512,
};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, Row};
use tracing::info;
use zcash_keys::address::UnifiedAddress;

use crate::{
    account::get_orchard_vk, get_coin, pay::{
        plan::{extract_transaction, plan_transaction, sign_transaction},
        pool::ALL_POOLS, Recipient,
    }
};

type RedPallas = reddsa::frost::redpallas::PallasBlake2b512;

use super::{
    account::{new_account, NewAccount},
    network::get_current_height,
};

#[frb]
pub async fn new_frost(
    name: &str,
    id: u8,
    n: u8,
    t: u8,
    funding_account: u32,
) -> Result<FrostPackage> {
    let c = get_coin!();
    let connection = c.get_pool();

    let height = get_current_height().await?;
    // generate an internal account for receiving messages from the
    // other participants
    let na = NewAccount {
        name: format!("{}-frost", name),
        icon: None,
        restore: false,
        key: String::new(),
        passphrase: None,
        fingerprint: None,
        aindex: 0,
        birth: Some(height),
        use_internal: false,
        internal: true,
    };
    let mailbox_account = new_account(&na).await?;
    let ovk = get_orchard_vk(&connection, mailbox_account)
        .await?
        .expect("Mailbox account should have orchard");
    let oaddr = ovk.address_at(0u64, Scope::External);
    let ua = UnifiedAddress::from_receivers(Some(oaddr), None, None).unwrap();

    let mut addresses = vec![String::new(); n as usize];
    addresses[(id - 1) as usize] = ua.encode(&c.network);

    let frost = FrostPackage {
        name: name.to_string(),
        id,
        n,
        t,
        funding_account,
        mailbox_account,
        addresses,
    };
    submit_dkg(&frost).await?;

    Ok(frost)
}

#[frb]
pub async fn load_frost() -> Result<Option<FrostPackage>> {
    let c = get_coin!();
    let connection = c.get_pool();
    let frost = sqlx::query("SELECT value FROM props WHERE key = $1")
        .bind("frost")
        .map(|row: SqliteRow| {
            let value: String = row.get(0);
            let frost: FrostPackage = serde_json::from_str(value.as_str()).unwrap();
            frost
        })
        .fetch_optional(connection)
        .await?;

    Ok(frost)
}

#[frb]
pub async fn submit_dkg(package: &FrostPackage) -> Result<()> {
    let c = get_coin!();
    let connection = c.get_pool();
    sqlx::query(
        "INSERT INTO props(key, value) VALUES ($1, $2) ON CONFLICT(key) DO UPDATE SET value = $2",
    )
    .bind("frost")
    .bind(serde_json::to_string(&package)?)
    .execute(connection)
    .await?;

    Ok(())
}

#[frb(dart_metadata = ("freezed"))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrostPackage {
    pub name: String,
    pub id: u8,
    pub n: u8,
    pub t: u8,
    pub funding_account: u32,
    pub mailbox_account: u32,
    pub addresses: Vec<String>,
}

impl FrostPackage {
    pub fn user_input_completed(&self) -> bool {
        self.addresses.iter().all(|a| !a.is_empty())
    }

    pub fn to_state(self) -> Option<DKGState> {
        if self.user_input_completed() {
            Some(DKGState::new(self))
        } else {
            None
        }
    }
}

#[frb(opaque)]
pub struct DKGState {
    pub package: FrostPackage,
    pub broadcast_account: u32,
}

impl DKGState {
    pub fn new(package: FrostPackage) -> Self {
        Self {
            package,
            broadcast_account: 0,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let c = get_coin!();
        let connection = c.get_pool();

        let m = self.seed();
        info!("Frost mnemonic: {}", m);

        let broadcast_account = self
            .create_broadcast_account(m.to_string().as_str())
            .await?;
        let fvk = get_orchard_vk(connection, broadcast_account)
            .await?
            .unwrap();
        let address = fvk.address_at(0u64, Scope::External);
        let ua = UnifiedAddress::from_receivers(Some(address), None, None).unwrap();
        let broadcast_address = ua.encode(&c.network);

        let id = self.package.id as u16;

        let sk1: round1::SecretPackage<PallasBlake2b512> = {
            let r: Option<(Vec<u8>,)> =
                sqlx::query_as("SELECT value FROM props WHERE key = 'frost-sk1'")
                    .fetch_optional(connection)
                    .await?;
            if let Some((sk1,)) = r {
                info!("Frost secret key 1 already exists, skipping to phase 2");
                let (sk1, _): (round1::SecretPackage<PallasBlake2b512>, usize) =
                    bincode::serde::decode_from_slice(&sk1, config::standard())?;
                sk1
            } else {
                let (sk1, pk1) = frost::keys::dkg::part1::<RedPallas, _>(
                    id.try_into().unwrap(),
                    self.package.n as u16,
                    self.package.t as u16,
                    OsRng,
                )?;
                let pk1 = DKGPackage {
                    from_id: id,
                    payload: pk1.serialize()?,
                };

                let sk1b = bincode::serde::encode_to_vec(&sk1, config::standard())?;
                let pk1 = bincode::serde::encode_to_vec(&pk1, config::standard())?;
                let mut client = c.client().await?;
                let height = get_current_height().await?;
                let pczt = plan_transaction(
                    &c.network,
                    connection,
                    &mut client,
                    self.package.funding_account,
                    ALL_POOLS,
                    std::slice::from_ref(&Recipient {
                        address: broadcast_address,
                        amount: 0,
                        pools: None,
                        user_memo: None,
                        memo_bytes: Some(to_arb_memo(&pk1)),
                    }),
                    false,
                )
                .await?;
                let pczt =
                    sign_transaction(connection, self.package.funding_account, &pczt).await?;
                let txb = extract_transaction(&pczt).await?;
                let txid = crate::pay::send(&mut client, height, &txb).await?;
                info!("Broadcasted transaction: {txid}");

                sqlx::query(
                r#"INSERT INTO props(key, value) VALUES ('frost-sk1', $1) ON CONFLICT(key) DO NOTHING"#,
                    )
                    .bind(&sk1b)
                    .execute(connection)
                    .await?;

                sqlx::query(
                    r#"INSERT INTO props(key, value) VALUES ($1, $2) ON CONFLICT(key) DO NOTHING"#,
                    )
                    .bind(format!("frost-pk1-{id}"))
                    .bind(&pk1)
                    .execute(connection)
                    .await?;

                sk1
            }
        };

        info!("Frost secret key 1: {:?}", sk1);

        Ok(())
    }
}

fn to_arb_memo(pk1: &[u8]) -> Vec<u8> {
    let mut memo_bytes = vec![0xFF];
    memo_bytes.extend_from_slice(&pk1);
    memo_bytes
}

#[derive(Serialize, Deserialize)]
pub struct DKGPackage {
    pub from_id: u16,
    pub payload: Vec<u8>,
}

/*
State machine.
1. Combine all the addresses, hash them and build a seed
    - Generate a broadcast account from the seed
    - Generate pub keys and send to broadcast address
    - Goto state 2
2. Wait for all the pub keys from other participants
    - Go to state 3
3. Combine secret key, public packages and generate
    new secret key and new individual public packages
    - Send new public packages to each participant (except ourself)
    - Goto state 4
4. Wait for all the new public packages from other participants
    - Combine secret key, public packages from stage 2 & 3
    - Generate new secret key and common public package
*/
