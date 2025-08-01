use std::str::FromStr;

use anyhow::{anyhow, Result};
use bip32::{ExtendedPrivateKey, ExtendedPublicKey, Prefix, PrivateKey};
use bip39::Mnemonic;
use flutter_rust_bridge::frb;
use orchard::keys::{FullViewingKey, Scope};
use rand_core::{OsRng, RngCore as _};
use ripemd::{Digest as _, Ripemd160};
use sapling_crypto::PaymentAddress;
use secp256k1::{PublicKey, SecretKey};
use sha2::Sha256;
use sqlx::{sqlite::SqliteRow, Row};
use tracing::info;
use zcash_address::unified::{Container, Encoding};
use zcash_keys::{
    address::UnifiedAddress,
    encoding::AddressCodec,
    keys::{UnifiedAddressRequest, UnifiedFullViewingKey, UnifiedSpendingKey},
};
use zcash_primitives::{
    consensus::Parameters as ZkParams,
    legacy::TransparentAddress,
    zip32::{fingerprint::SeedFingerprint, AccountId},
};
use zcash_protocol::consensus::{Network, NetworkConstants};
use zcash_transparent::keys::{AccountPrivKey, AccountPubKey};

use crate::{
    account::{derive_transparent_address, derive_transparent_sk, TxAccount, TxNote},
    bip38,
    db::{
        init_account_orchard, init_account_sapling, init_account_transparent,
        store_account_metadata, store_account_orchard_sk, store_account_orchard_vk,
        store_account_sapling_sk, store_account_sapling_vk, store_account_seed,
        store_account_transparent_addr, store_account_transparent_sk, store_account_transparent_vk,
        update_dindex,
    },
    get_coin,
    io::{decrypt, encrypt},
    key::{is_valid_phrase, is_valid_sapling_key, is_valid_transparent_key, is_valid_ufvk},
    pay::pool::ALL_POOLS,
    setup,
};

#[frb(sync)]
pub fn new_seed(phrase: &str) -> Result<String> {
    let seed_phrase = bip39::Mnemonic::from_str(phrase)?;
    let seed = seed_phrase.to_seed("");

    let usk = UnifiedSpendingKey::from_seed(&Network::MainNetwork, &seed, AccountId::ZERO)?;
    let uvk = usk.to_unified_full_viewing_key();
    let (ua, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
    info!("initial di: {}", u64::try_from(di).unwrap());
    if let Some(pa) = ua.sapling() {
        return Ok(pa.encode(&Network::MainNetwork));
    }

    let address = ua.encode(&Network::MainNetwork);

    Ok(address)
}

#[frb]
pub async fn get_account_pools(account: u32) -> Result<u8> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    let tkeys = crate::db::select_account_transparent(&mut *connection, account).await?;
    let skeys = crate::db::select_account_sapling(&mut *connection, account).await?;
    let okeys = crate::db::select_account_orchard(&mut *connection, account).await?;

    let mut pools = 0;
    if tkeys.xvk.is_some() {
        pools |= 1;
    }
    if skeys.xvk.is_some() {
        pools |= 2;
    }
    if okeys.xvk.is_some() {
        pools |= 4;
    }
    Ok(pools)
}

#[frb]
pub async fn get_account_ufvk(account: u32, pools: u8) -> Result<String> {
    let c = get_coin!();
    let network = c.network;
    let mut connection = c.get_connection().await?;

    let ufvk = crate::key::get_account_ufvk(&network, &mut connection, account, pools).await?;
    Ok(ufvk)
}

pub async fn get_account_seed(account: u32) -> Result<Option<Seed>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    let seed = sqlx::query("SELECT seed, passphrase, aindex FROM accounts WHERE id_account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let mnemonic: Option<String> = row.get(0);
            let phrase: Option<String> = row.get(1);
            let aindex: u32 = row.get(2);
            let phrase = phrase.unwrap_or_default();
            mnemonic.map(|mnemonic| Seed {
                mnemonic,
                phrase,
                aindex,
            })
        })
        .fetch_one(&mut *connection)
        .await?;
    Ok(seed)
}

#[frb(dart_metadata = ("freezed"))]
pub struct Seed {
    pub mnemonic: String,
    pub phrase: String,
    pub aindex: u32,
}

#[frb]
pub async fn get_account_fingerprint(account: u32) -> Result<Option<String>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    let fingerprint = crate::db::get_account_fingerprint(&mut *connection, account).await?;
    let fingerprint = fingerprint.as_ref().map(|fp| hex::encode(&fp[..4]));
    Ok(fingerprint)
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
    let mut connection = c.get_connection().await?;
    let accounts = crate::db::list_accounts(&mut *connection, c.coin).await?;

    Ok(accounts)
}

#[frb]
pub async fn update_account(update: &AccountUpdate) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    if let Some(ref name) = update.name {
        sqlx::query("UPDATE accounts SET name = ? WHERE id_account = ?")
            .bind(name)
            .bind(update.id)
            .execute(&mut *connection)
            .await?;
    }
    if let Some(ref icon) = update.icon {
        sqlx::query("UPDATE accounts SET icon = ? WHERE id_account = ?")
            .bind(icon)
            .bind(update.id)
            .execute(&mut *connection)
            .await?;
    }
    if let Some(ref birth) = update.birth {
        sqlx::query("UPDATE accounts SET birth = ? WHERE id_account = ?")
            .bind(birth)
            .bind(update.id)
            .execute(&mut *connection)
            .await?;
    }
    if let Some(ref enabled) = update.enabled {
        sqlx::query("UPDATE accounts SET enabled = ? WHERE id_account = ?")
            .bind(enabled)
            .bind(update.id)
            .execute(&mut *connection)
            .await?;
    }
    if let Some(ref hidden) = update.hidden {
        sqlx::query("UPDATE accounts SET hidden = ? WHERE id_account = ?")
            .bind(hidden)
            .bind(update.id)
            .execute(&mut *connection)
            .await?;
    }

    Ok(())
}

#[frb]
pub async fn delete_account(account: u32) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::db::delete_account(&mut *connection, account).await?;

    Ok(())
}

#[frb]
pub async fn reorder_account(old_position: u32, new_position: u32) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::db::reorder_account(&mut *connection, old_position, new_position).await
}

#[frb]
pub fn set_account(account: u32) -> Result<()> {
    setup!(account);
    Ok(())
}

#[frb]
pub async fn new_account(na: &NewAccount) -> Result<u32> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    let network = c.network;
    let min_height: u32 = (network
        .activation_height(zcash_protocol::consensus::NetworkUpgrade::Sapling)
        .unwrap()
        + 1)
    .into();

    let birth = na.birth.unwrap_or(min_height).max(min_height);

    let account = store_account_metadata(
        &mut *connection,
        &na.name,
        &na.icon,
        &na.fingerprint,
        birth,
        na.use_internal,
        na.internal,
    )
    .await?;

    let mut key = na.key.clone();
    if key.is_empty() {
        let mut entropy = [0u8; 32];
        OsRng.fill_bytes(&mut entropy);
        let m = bip39::Mnemonic::from_entropy(&entropy)?;
        key = m.to_string();
    }

    let pools = na.pools.unwrap_or(ALL_POOLS);

    if is_valid_phrase(&key) {
        let seed_phrase = bip39::Mnemonic::from_str(&key)?;
        let passphrase = na.passphrase.clone().unwrap_or(String::new());
        let seed = seed_phrase.to_seed(&passphrase);
        let seed_fingerprint = SeedFingerprint::from_seed(&seed).unwrap().to_bytes();
        store_account_seed(
            &mut *connection,
            account,
            &key,
            &passphrase,
            &seed_fingerprint,
            na.aindex,
        )
        .await?;
        let usk = UnifiedSpendingKey::from_seed(
            &c.network,
            &seed,
            AccountId::try_from(na.aindex).unwrap(),
        )?;
        let uvk = usk.to_unified_full_viewing_key();
        let (_, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
        let dindex: u32 = di.try_into()?;

        if pools & 1 != 0 {
            init_account_transparent(&mut *connection, account, birth).await?;
            let tsk = usk.transparent();
            store_account_transparent_sk(&mut *connection, account, tsk).await?;
            let tvk = &tsk.to_account_pubkey();
            store_account_transparent_vk(&mut *connection, account, tvk).await?;
            let sk = derive_transparent_sk(&tsk, 0, dindex)?;
            let (pk, taddr) = derive_transparent_address(tvk, 0, dindex)?;
            store_account_transparent_addr(
                &mut *connection,
                account,
                0,
                dindex,
                Some(sk),
                &pk,
                &taddr.encode(&c.network),
            )
            .await?;
        }

        if pools & 2 != 0 {
            init_account_sapling(&mut *connection, account, birth).await?;
            let sxsk = usk.sapling();
            store_account_sapling_sk(&mut *connection, account, sxsk).await?;
            let sxvk = sxsk.to_diversifiable_full_viewing_key();
            store_account_sapling_vk(&mut *connection, account, &sxvk).await?;
        }

        if pools & 4 != 0 {
            init_account_orchard(&mut *connection, account, birth).await?;
            let oxsk = usk.orchard();
            store_account_orchard_sk(&mut *connection, account, oxsk).await?;
            let oxvk = FullViewingKey::from(oxsk);
            store_account_orchard_vk(&mut *connection, account, &oxvk).await?;
        }

        update_dindex(&mut *connection, account, dindex, true).await?;
    }
    if is_valid_transparent_key(&key) {
        init_account_transparent(&mut *connection, account, birth).await?;
        if let Ok(xsk) = ExtendedPrivateKey::<SecretKey>::from_str(&key) {
            let xsk = AccountPrivKey::from_extended_privkey(xsk);
            store_account_transparent_sk(&mut *connection, account, &xsk).await?;
            let xvk = xsk.to_account_pubkey();
            store_account_transparent_vk(&mut *connection, account, &xvk).await?;
            let sk = derive_transparent_sk(&xsk, 0, 0)?;
            let (pk, address) = derive_transparent_address(&xvk, 0, 0)?;
            store_account_transparent_addr(
                &mut *connection,
                account,
                0,
                0,
                Some(sk),
                &pk,
                &address.encode(&network),
            )
            .await?;
        } else if let Ok(xvk) = ExtendedPublicKey::<PublicKey>::from_str(&key) {
            // No AccountPubKey::from_extended_pubkey, we need to use the bytes
            let mut buf = xvk.attrs().chain_code.to_vec();
            buf.extend_from_slice(&xvk.to_bytes());
            let xvk = AccountPubKey::deserialize(&buf.try_into().unwrap()).unwrap();
            store_account_transparent_vk(&mut *connection, account, &xvk).await?;
            let (pk, address) = derive_transparent_address(&xvk, 0, 0)?;
            store_account_transparent_addr(
                &mut *connection,
                account,
                0,
                0,
                None,
                &pk,
                &address.encode(&network),
            )
            .await?;
        } else if let Ok(sk) = bip38::import_tsk(&key) {
            let secp = secp256k1::Secp256k1::new();
            let tpk = sk.public_key(&secp).serialize();
            let pkh: [u8; 20] = Ripemd160::digest(&Sha256::digest(&tpk)).into();
            let addr = TransparentAddress::PublicKeyHash(pkh.clone());
            store_account_transparent_addr(
                &mut *connection,
                account,
                0,
                0,
                Some(sk.to_bytes().to_vec()),
                &tpk,
                &addr.encode(&network),
            )
            .await?;
        }
    }
    if is_valid_sapling_key(&network, &key) {
        init_account_sapling(&mut *connection, account, birth).await?;
        let di = if let Ok(xsk) = zcash_keys::encoding::decode_extended_spending_key(
            network.hrp_sapling_extended_spending_key(),
            &key,
        ) {
            store_account_sapling_sk(&mut *connection, account, &xsk).await?;
            let xvk = xsk.to_diversifiable_full_viewing_key();
            store_account_sapling_vk(&mut *connection, account, &xvk).await?;
            let (di, _) = xvk.default_address();
            di
        } else if let Ok(xvk) = zcash_keys::encoding::decode_extended_full_viewing_key(
            network.hrp_sapling_extended_full_viewing_key(),
            &key,
        ) {
            store_account_sapling_vk(
                &mut *connection,
                account,
                &xvk.to_diversifiable_full_viewing_key(),
            )
            .await?;
            let (di, _) = xvk.default_address();
            di
        } else {
            return Err(anyhow!("Invalid Sapling Key"));
        };
        let dindex: u32 = di.try_into()?;
        update_dindex(&mut *connection, account, dindex, true).await?;
    }
    if is_valid_ufvk(&network, &key) {
        let uvk =
            UnifiedFullViewingKey::decode(&network, &key).map_err(|_| anyhow!("Invalid Key"))?;
        let (_, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
        let dindex: u32 = di.try_into()?;

        match uvk.transparent() {
            Some(tvk) if pools & 1 != 0 => {
                init_account_transparent(&mut *connection, account, birth).await?;
                store_account_transparent_vk(&mut *connection, account, tvk).await?;
                let (pk, address) = derive_transparent_address(tvk, 0, dindex)?;
                store_account_transparent_addr(
                    &mut *connection,
                    account,
                    0,
                    dindex,
                    None,
                    &pk,
                    &address.encode(&network),
                )
                .await?;
            }
            _ => {}
        }
        match uvk.sapling() {
            Some(sxvk) if pools & 2 != 0 => {
                init_account_sapling(&mut *connection, account, birth).await?;
                store_account_sapling_vk(&mut *connection, account, &sxvk).await?;
            }
            _ => {}
        }
        match uvk.orchard() {
            Some(ovk) if pools & 4 != 0 => {
                init_account_orchard(&mut *connection, account, birth).await?;
                store_account_orchard_vk(&mut *connection, account, &ovk).await?;
            }
            _ => {}
        }
        update_dindex(&mut *connection, account, dindex, true).await?;
    }

    Ok(account)
}

#[frb]
pub async fn generate_next_dindex() -> Result<u32> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::generate_next_dindex(&c.network, &mut *connection, c.account).await
}

#[frb]
pub async fn generate_next_change_address() -> Result<Option<String>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::generate_next_change_address(&c.network, &mut *connection, c.account).await
}

#[frb]
pub async fn reset_sync(id: u32) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::reset_sync(&mut connection, id).await
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
    pub internal: bool,
    pub height: u32,
    pub balance: u64,
}

#[frb(dart_metadata = ("freezed"))]
pub struct AccountUpdate {
    pub coin: u8,
    pub id: u32,
    pub name: Option<String>,
    pub icon: Option<Vec<u8>>,
    pub birth: Option<u32>,
    pub hidden: Option<bool>,
    pub enabled: Option<bool>,
}

#[frb(dart_metadata = ("freezed"))]
pub struct NewAccount {
    pub icon: Option<Vec<u8>>,
    pub name: String,
    pub restore: bool,
    pub key: String,
    pub passphrase: Option<String>,
    pub fingerprint: Option<Vec<u8>>,
    pub aindex: u32,
    pub birth: Option<u32>,
    pub pools: Option<u8>,
    pub use_internal: bool,
    pub internal: bool,
}

#[frb(dart_metadata = ("freezed"))]
pub struct Tx {
    pub id: u32,
    pub txid: Vec<u8>,
    pub height: u32,
    pub time: u32,
    pub value: i64,
    pub tpe: Option<u8>,
}

pub struct TAddressTxCount {
    pub address: String,
    pub scope: u8,
    pub dindex: u32,
    pub amount: u64,
    pub tx_count: u32,
    pub time: u32,
}

pub async fn remove_account(account_id: u32) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    crate::db::delete_account(&mut connection, account_id).await?;
    Ok(())
}

pub async fn move_account(old_position: u32, new_position: u32) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    crate::db::reorder_account(&mut connection, old_position, new_position).await?;
    Ok(())
}

pub async fn list_tx_history() -> Result<Vec<Tx>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    let txs = crate::db::fetch_txs(&mut connection, c.account).await?;
    Ok(txs)
}

#[frb(dart_metadata = ("freezed"))]
pub struct Memo {
    pub id: u32,
    pub id_tx: u32,
    pub id_note: Option<u32>,
    pub pool: u8,
    pub height: u32,
    pub vout: u32,
    pub time: u32,
    pub memo_bytes: Vec<u8>,
    pub memo: Option<String>,
}

#[frb]
pub async fn list_memos() -> Result<Vec<Memo>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    let memos = crate::db::fetch_memos(&mut *connection, c.account).await?;
    Ok(memos)
}

#[frb]
pub async fn get_addresses(ua_pools: u8) -> Result<Addresses> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    let tkeys = crate::db::select_account_transparent(&mut *connection, c.account).await?;
    let skeys = crate::db::select_account_sapling(&mut *connection, c.account).await?;
    let okeys = crate::db::select_account_orchard(&mut *connection, c.account).await?;

    let dindex = crate::db::get_account_dindex(&mut *connection, c.account).await?;

    let taddr = tkeys
        .xvk
        .as_ref()
        .map(|xvk| derive_transparent_address(xvk, 0, dindex).unwrap().1);

    let dindex = dindex as u64;
    let saddr = skeys
        .xvk
        .as_ref()
        .map(|xvk| xvk.address(dindex.into()).unwrap());
    let oaddr = okeys
        .xvk
        .as_ref()
        .map(|xvk| xvk.address_at(dindex, Scope::External));

    let ua_orchard = UnifiedAddress::from_receivers(oaddr, None, None);

    let ua = UnifiedAddress::from_receivers(
        if ua_pools & 4 != 0 { oaddr } else { None },
        if ua_pools & 2 != 0 { saddr } else { None },
        if ua_pools & 1 != 0 { taddr } else { None },
    );

    // final fallback if we have a transparent address from a BIP 38 secret key
    let taddr = taddr.map(|x| x.encode(&c.network)).or(tkeys.address);

    let addresses = Addresses {
        taddr,
        saddr: saddr.map(|x| x.encode(&c.network)),
        oaddr: ua_orchard.map(|x| x.encode(&c.network)),
        ua: ua.map(|x| x.encode(&c.network)),
    };

    Ok(addresses)
}

pub struct Addresses {
    pub taddr: Option<String>,
    pub saddr: Option<String>,
    pub oaddr: Option<String>,
    pub ua: Option<String>,
}

#[frb]
pub async fn get_tx_details(id_tx: u32) -> Result<TxAccount> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    let tx = crate::account::get_tx_details(&mut *connection, c.account, id_tx).await?;
    Ok(tx)
}

#[frb]
pub async fn list_notes() -> Result<Vec<TxNote>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    let notes = crate::db::get_notes(&mut *connection, c.account).await?;
    Ok(notes)
}

#[frb]
pub async fn lock_note(id: u32, locked: bool) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    crate::db::lock_note(&mut *connection, c.account, id, locked).await?;
    Ok(())
}

#[frb]
pub async fn fetch_transparent_address_tx_count() -> Result<Vec<TAddressTxCount>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    crate::db::fetch_transparent_address_tx_count(&mut *connection, c.account).await
}

#[frb]
pub async fn transparent_sweep(end_height: u32, gap_limit: u32) -> Result<u32> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    crate::sync::transparent_sweep(
        &c.network,
        &mut *connection,
        &mut c.client().await?,
        c.account,
        end_height,
        gap_limit,
    )
    .await
}

#[frb]
pub async fn export_account(id: u32, passphrase: &str) -> Result<Vec<u8>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    let data = crate::io::export_account(&mut *connection, id).await?;
    let encrypted = encrypt(passphrase, &data)?;
    Ok(encrypted)
}

#[frb]
pub async fn import_account(passphrase: &str, data: &[u8]) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    let decrypted = decrypt(passphrase, data)?;
    crate::io::import_account(&mut *connection, &decrypted).await?;
    Ok(())
}

#[frb]
pub async fn print_keys(id: u32) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    let (seed, aindex) = sqlx::query(
        "SELECT name, seed, seed_fingerprint, aindex, dindex,
        def_dindex, birth FROM accounts WHERE id_account = ?",
    )
    .bind(id)
    .map(|row: SqliteRow| {
        let name: String = row.get(0);
        let seed: Option<String> = row.get(1);
        let seed_fingerprint: Vec<u8> = row.get(2);
        let aindex: u32 = row.get(3);
        let dindex: u32 = row.get(4);
        let def_dindex: u32 = row.get(5);
        let birth: u32 = row.get(6);

        info!(
            "Account {}: {} - {:?} - {} - {} - {} - {} - {}",
            id,
            name,
            seed,
            hex::encode(seed_fingerprint),
            aindex,
            dindex,
            def_dindex,
            birth
        );
        (seed, aindex)
    })
    .fetch_one(&mut *connection)
    .await?;

    sqlx::query("SELECT xsk, xvk FROM transparent_accounts WHERE account = ?")
        .bind(id)
        .map(|row: SqliteRow| {
            let xsk: Option<Vec<u8>> = row.get(0);
            let xvk: Vec<u8> = row.get(1);
            let xsk = xsk.as_ref().map(|xsk| {
                let mut bytes = Prefix::XPRV.to_bytes().to_vec();
                bytes.extend_from_slice(xsk);
                bs58::encode(bytes).with_check().into_string()
            });

            let xvk = hex::encode(&xvk);

            info!("Transparent Account {}: {:?} - {}", id, &xsk, &xvk,);
        })
        .fetch_all(&mut *connection)
        .await?;

    let seed = seed.unwrap();
    let memo = Mnemonic::from_str(&seed).unwrap();
    let seed = memo.to_seed("");

    let usk =
        UnifiedSpendingKey::from_seed(&c.network, &seed, AccountId::try_from(aindex).unwrap())?;
    let uvk = usk.to_unified_full_viewing_key();
    if uvk.sapling().is_some() {
        println!("Has Sapling");
    }
    if uvk.orchard().is_some() {
        println!("Has Orchard");
    }
    let uvk = uvk.encode(&c.network);
    println!("Unified Full Viewing Key: {}", uvk);

    Ok(())
}

#[frb]
pub async fn get_account_frost_params() -> Result<Option<FrostParams>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::get_account_frost_params(&mut *connection, c.account).await
}

#[frb(dart_metadata = ("freezed"))]
pub struct FrostParams {
    pub id: u8,
    pub n: u8,
    pub t: u8,
}
