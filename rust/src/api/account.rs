use std::str::FromStr;

use anyhow::{anyhow, Result};
use bip32::{ExtendedPrivateKey, ExtendedPublicKey, PrivateKey};
use flutter_rust_bridge::frb;
use orchard::keys::FullViewingKey;
use rand_core::{OsRng, RngCore as _};
use ripemd::{Digest as _, Ripemd160};
use sapling_crypto::PaymentAddress;
use secp256k1::{PublicKey, SecretKey};
use sha2::Sha256;
use zcash_address::unified::{Container, Encoding};
use zcash_keys::{
    address::UnifiedAddress,
    encoding::AddressCodec,
    keys::{UnifiedAddressRequest, UnifiedFullViewingKey, UnifiedSpendingKey},
};
use zcash_primitives::{consensus::Parameters as ZkParams, legacy::TransparentAddress, zip32::{fingerprint::SeedFingerprint, AccountId}};
use zcash_protocol::consensus::{Network, NetworkConstants};
use zcash_transparent::keys::{AccountPrivKey, AccountPubKey};

use crate::{
    account::{derive_transparent_address, derive_transparent_sk},
    bip38,
    db::{
        init_account_orchard, init_account_sapling, init_account_transparent,
        store_account_metadata, store_account_orchard_sk, store_account_orchard_vk,
        store_account_sapling_sk, store_account_sapling_vk, store_account_seed,
        store_account_transparent_addr, store_account_transparent_sk, store_account_transparent_vk,
        update_dindex,
    },
    get_coin,
    key::{is_valid_phrase, is_valid_sapling_key, is_valid_transparent_key, is_valid_ufvk},
    setup,
};

#[frb(sync)]
pub fn new_seed(phrase: &str) -> Result<String> {
    let seed_phrase = bip39::Mnemonic::from_str(phrase)?;
    let seed = seed_phrase.to_seed("");

    let usk = UnifiedSpendingKey::from_seed(&Network::MainNetwork, &seed, AccountId::ZERO)?;
    let uvk = usk.to_unified_full_viewing_key();
    let (ua, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
    println!("di: {}", hex::encode(&di.as_bytes()));
    if let Some(pa) = ua.sapling() {
        return Ok(pa.encode(&Network::MainNetwork));
    }

    let address = ua.encode(&Network::MainNetwork);

    Ok(address)
}

#[frb]
pub async fn get_account_ufvk() -> Result<String> {
    let c = get_coin!();
    let network = c.network;

    let ufvk = crate::key::get_account_ufvk().await?;
    Ok(ufvk.encode(&network))
}

#[frb(sync)]
pub fn ua_from_ufvk(ufvk: &str, di: Option<u32>) -> Result<String> {
    let c = get_coin!();
    let network = c.network;

    let ufvk = UnifiedFullViewingKey::decode(&network, ufvk).map_err(|_| anyhow!("Invalid Key"))?;
    let ua = match di {
        Some(di) => ufvk.address(di.into(), UnifiedAddressRequest::AllAvailableKeys)?,
        None => {
            ufvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?
                .0
        }
    };

    Ok(ua.encode(&network))
}

#[frb(sync)]
pub fn receivers_from_ua(ua: &str) -> Result<Receivers> {
    let c = get_coin!();
    let network = c.network;

    let (net, ua) = zcash_address::unified::Address::decode(ua)?;
    if net != network.network_type() {
        anyhow::bail!("Invalid Network");
    }

    let mut receivers = Receivers::default();
    for item in ua.items() {
        match item {
            zcash_address::unified::Receiver::P2pkh(pkh) => {
                let taddr = TransparentAddress::PublicKeyHash(pkh);
                receivers.taddr = Some(taddr.encode(&network));
            }
            zcash_address::unified::Receiver::P2sh(sh) => {
                let taddr = TransparentAddress::ScriptHash(sh);
                receivers.taddr = Some(taddr.encode(&network));
            }
            zcash_address::unified::Receiver::Sapling(s) => {
                let saddr = PaymentAddress::from_bytes(&s).unwrap();
                receivers.saddr = Some(saddr.encode(&network));
            }
            zcash_address::unified::Receiver::Orchard(o) => {
                let oaddr = orchard::Address::from_raw_address_bytes(&o)
                    .into_option()
                    .unwrap();
                let oaddr = UnifiedAddress::from_receivers(Some(oaddr), None, None).unwrap();
                receivers.oaddr = Some(oaddr.encode(&network));
            }
            _ => {}
        }
    }

    Ok(receivers)
}

#[derive(Default)]
pub struct Receivers {
    pub taddr: Option<String>,
    pub saddr: Option<String>,
    pub oaddr: Option<String>,
}

#[frb]
pub async fn list_accounts() -> Result<Vec<Account>> {
    let c = get_coin!();
    let accounts = crate::db::list_accounts(c.get_pool(), c.coin).await?;

    Ok(accounts)
}

#[frb]
pub async fn update_account(update: &AccountUpdate) -> Result<()> {
    let c = get_coin!();
    let pool = c.get_pool();

    if let Some(ref name) = update.name {
        sqlx::query("UPDATE accounts SET name = ? WHERE id_account = ?")
            .bind(name)
            .bind(update.id)
            .execute(pool)
            .await?;
    }
    if let Some(ref icon) = update.icon {
        sqlx::query("UPDATE accounts SET icon = ? WHERE id_account = ?")
            .bind(icon)
            .bind(update.id)
            .execute(pool)
            .await?;
    }
    if let Some(ref birth) = update.birth {
        sqlx::query("UPDATE accounts SET birth = ? WHERE id_account = ?")
            .bind(birth)
            .bind(update.id)
            .execute(pool)
            .await?;
        // TODO: Clear all notes and transactions
        sqlx::query("UPDATE sync_heights SET height = ? WHERE account = ?")
            .bind(birth)
            .bind(update.id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn drop_schema() -> Result<()> {
    let c = get_coin!();
    let pool = c.get_pool();

    sqlx::query("DROP TABLE IF EXISTS transparent_accounts")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS transparent_address_accounts")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS sapling_accounts")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS orchard_accounts")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_account(account: &Account) -> Result<()> {
    let c = get_coin!();
    let pool = c.get_pool();

    crate::db::delete_account(pool, account.id).await?;

    Ok(())
}

#[frb]
pub async fn reorder_account(old_position: u32, new_position: u32) -> Result<()> {
    let c = get_coin!();
    let pool = c.get_pool();

    crate::db::reorder_account(pool, old_position, new_position).await
}

#[frb]
pub fn set_account(id: u32) -> Result<()> {
    setup!(id);
    Ok(())
}

#[frb]
pub async fn new_account(na: &NewAccount) -> Result<()> {
    let c = get_coin!();
    let pool = c.get_pool();
    let network = c.network;
    let min_height: u32 = network
        .activation_height(zcash_protocol::consensus::NetworkUpgrade::Sapling)
        .unwrap()
        .into();

    let birth = na.birth.unwrap_or(min_height);

    let account = store_account_metadata(pool, &na.name, &na.icon, birth).await?;
    setup!(account);

    let mut key = na.key.clone();
    if key.is_empty() {
        let mut entropy = [0u8; 32];
        OsRng.try_fill_bytes(&mut entropy)?;
        let m = bip39::Mnemonic::from_entropy(&entropy)?;
        key = m.to_string();
    }

    if is_valid_phrase(&key) {
        let seed_phrase = bip39::Mnemonic::from_str(&key)?;
        let seed = seed_phrase.to_seed("");
        let seed_fingerprint = SeedFingerprint::from_seed(&seed).unwrap().to_bytes();
        store_account_seed(pool, account, &key, &seed_fingerprint, na.aindex).await?;
        let usk = UnifiedSpendingKey::from_seed(
            &c.network,
            &seed,
            AccountId::try_from(na.aindex).unwrap(),
        )?;
        let uvk = usk.to_unified_full_viewing_key();
        let (_, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
        let dindex: u32 = di.try_into()?;

        init_account_transparent(pool, account).await?;
        let tsk = usk.transparent();
        store_account_transparent_sk(pool, account, tsk).await?;
        let tvk = &tsk.to_account_pubkey();
        store_account_transparent_vk(pool, account, tvk).await?;
        let sk = derive_transparent_sk(&tsk, dindex)?;
        let taddr = derive_transparent_address(tvk, 0, dindex)?;
        store_account_transparent_addr(pool, account, 0, dindex, Some(&sk), &taddr.encode(&c.network))
            .await?;

        init_account_sapling(pool, account).await?;
        let sxsk = usk.sapling();
        store_account_sapling_sk(pool, account, sxsk).await?;
        let sxvk = sxsk.to_diversifiable_full_viewing_key();
        store_account_sapling_vk(pool, account, &sxvk).await?;

        init_account_orchard(pool, account).await?;
        let oxsk = usk.orchard();
        store_account_orchard_sk(pool, account, oxsk).await?;
        let oxvk = FullViewingKey::from(oxsk);
        store_account_orchard_vk(pool, account, &oxvk).await?;

        update_dindex(pool, account, dindex, true).await?;
    }
    if is_valid_transparent_key(&key) {
        init_account_transparent(pool, account).await?;
        if let Ok(xsk) = ExtendedPrivateKey::<SecretKey>::from_str(&key) {
            let xsk = AccountPrivKey::from_extended_privkey(xsk);
            store_account_transparent_sk(pool, account, &xsk).await?;
            let xvk = xsk.to_account_pubkey();
            store_account_transparent_vk(pool, account, &xvk).await?;
            let sk = derive_transparent_sk(&xsk, 0)?;
            let address = derive_transparent_address(&xvk, 0, 0)?;
            store_account_transparent_addr(pool, account, 0, 0, Some(&sk), &address.encode(&network))
                .await?;
        }
        if let Ok(xvk) = ExtendedPublicKey::<PublicKey>::from_str(&key) {
            // No AccountPubKey::from_extended_pubkey, we need to use the bytes
            let mut buf = xvk.attrs().chain_code.to_vec();
            buf.extend_from_slice(&xvk.to_bytes());
            let xvk = AccountPubKey::deserialize(&buf.try_into().unwrap()).unwrap();
            store_account_transparent_vk(pool, account, &xvk).await?;
            let address = derive_transparent_address(&xvk, 0, 0)?;
            store_account_transparent_addr(pool, account, 0, 0, None, &address.encode(&network))
                .await?;
        }
        if let Ok(sk) = bip38::import_tsk(&key) {
            let secp = secp256k1::Secp256k1::new();
            let tpk = sk.public_key(&secp).serialize();
            let pkh: [u8; 20] = Ripemd160::digest(&Sha256::digest(&tpk)).into();
            let addr = TransparentAddress::PublicKeyHash(pkh.clone());
            store_account_transparent_addr(pool, account, 0, 0, Some(&sk.to_bytes()), &addr.encode(&network))
                .await?;
        }
    }
    if is_valid_sapling_key(&network, &key) {
        init_account_sapling(pool, account).await?;
        if let Ok(xsk) = zcash_keys::encoding::decode_extended_spending_key(
            network.hrp_sapling_extended_spending_key(),
            &key,
        ) {
            store_account_sapling_sk(pool, account, &xsk).await?;
            let xvk = xsk.to_diversifiable_full_viewing_key();
            store_account_sapling_vk(pool, account, &xvk).await?;
        }
        if let Ok(xvk) = zcash_keys::encoding::decode_extended_full_viewing_key(
            network.hrp_sapling_extended_full_viewing_key(),
            &key,
        ) {
            store_account_sapling_vk(pool, account, &xvk.to_diversifiable_full_viewing_key())
                .await?;
        }
    }
    if is_valid_ufvk(&network, &key) {
        let uvk =
            UnifiedFullViewingKey::decode(&network, &key).map_err(|_| anyhow!("Invalid Key"))?;
        let (_, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
        let dindex: u32 = di.try_into()?;

        if let Some(tvk) = uvk.transparent() {
            init_account_transparent(pool, account).await?;
            store_account_transparent_vk(pool, account, tvk).await?;
            let address = derive_transparent_address(tvk, 0, dindex)?;
            store_account_transparent_addr(
                pool,
                account,
                0,
                dindex,
                None,
                &address.encode(&network),
            )
            .await?;
        }
        if let Some(svk) = uvk.sapling() {
            init_account_sapling(pool, account).await?;
            store_account_sapling_vk(pool, account, svk).await?;
        }
        if let Some(ovk) = uvk.orchard() {
            init_account_orchard(pool, account).await?;
            store_account_orchard_vk(pool, account, ovk).await?;
        }
        update_dindex(pool, account, dindex, true).await?;
    }
    Ok(())
}

#[frb(dart_metadata = ("freezed"))]
pub struct Account {
    pub coin: u8,
    pub id: u32,
    pub name: String,
    pub seed: Option<String>,
    pub aindex: u32,
    pub icon: Option<Vec<u8>>,
    pub birth: u32,
    pub position: u8,
    pub hidden: bool,
    pub saved: bool,
    pub enabled: bool,
}

#[frb(dart_metadata = ("freezed"))]
pub struct AccountUpdate {
    pub coin: u8,
    pub id: u32,
    pub name: Option<String>,
    pub icon: Option<Vec<u8>>,
    pub birth: Option<u32>,
}

#[frb(dart_metadata = ("freezed"))]
pub struct NewAccount {
    pub icon: Option<Vec<u8>>,
    pub name: String,
    pub restore: bool,
    pub key: String,
    pub aindex: u32,
    pub birth: Option<u32>,
}

#[frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
    let _ = env_logger::builder().try_init();
    rustls::crypto::ring::default_provider().install_default().unwrap();
}

pub async fn get_all_accounts() -> Result<Vec<Account>> {
    let c = get_coin!();
    let accounts = crate::db::list_accounts(c.get_pool(), c.coin).await?;
    Ok(accounts)
}

pub async fn remove_account(account_id: u32) -> Result<()> {
    let c = get_coin!();
    crate::db::delete_account(c.get_pool(), account_id).await?;
    Ok(())
}

pub async fn move_account(old_position: u32, new_position: u32) -> Result<()> {
    let c = get_coin!();
    crate::db::reorder_account(c.get_pool(), old_position, new_position).await?;
    Ok(())
}
