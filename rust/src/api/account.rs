use std::str::FromStr;

use anyhow::{anyhow, Result};
use bip32::{ExtendedPrivateKey, ExtendedPublicKey, Prefix, PrivateKey};
use bip39::Mnemonic;
use csv_async::AsyncWriter;
use flutter_rust_bridge::frb;
use orchard::keys::{FullViewingKey, Scope};
use ripemd::{Digest as _, Ripemd160};
use sapling_crypto::{zip32::DiversifiableFullViewingKey, PaymentAddress};
use secp256k1::{PublicKey, SecretKey};
use sha2::Sha256;
use sqlx::{sqlite::SqliteRow, Acquire, Row};
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
use zcash_protocol::consensus::NetworkConstants;
use zcash_transparent::keys::{AccountPrivKey, AccountPubKey};

use crate::{
    account::{derive_transparent_address, derive_transparent_sk, TxAccount, TxNote},
    api::{key::generate_seed, ledger::get_hw_fvk},
    bip38,
    coin::Network,
    db::{
        get_account_dindex, init_account_orchard, init_account_sapling, init_account_transparent,
        store_account_hw, store_account_metadata, store_account_orchard_sk,
        store_account_orchard_vk, store_account_sapling_sk, store_account_sapling_vk,
        store_account_seed, store_account_transparent_addr, store_account_transparent_sk,
        store_account_transparent_vk, update_dindex, LEDGER_CODE,
    },
    get_coin,
    io::{decrypt, encrypt},
    key::{is_valid_phrase, is_valid_sapling_key, is_valid_transparent_key, is_valid_ufvk},
    pay::pool::ALL_POOLS,
    setup, tiu,
};

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
use crate::ledger::fvk::{
    get_hw_sapling_address, get_hw_transparent_address, show_sapling_address,
    show_transparent_address,
};
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
use crate::no_ledger::{
    get_hw_sapling_address, get_hw_transparent_address, show_sapling_address,
    show_transparent_address,
};

#[frb]
pub async fn get_account_pools(account: u32) -> Result<u8> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    let dindex = get_account_dindex(&mut connection, account).await?;
    let tkeys = crate::db::select_account_transparent(&mut connection, account, dindex).await?;
    let skeys = crate::db::select_account_sapling(&c.network, &mut connection, account).await?;
    let okeys = crate::db::select_account_orchard(&mut connection, account).await?;

    let mut pools = 0;
    if tkeys.xvk.is_some() || tkeys.address.is_some() {
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

    let fingerprint = crate::db::get_account_fingerprint(&mut connection, account).await?;
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
    let accounts = crate::db::list_accounts(&mut connection, c.coin).await?;

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
    if let Some(icon) = update.icon.as_ref() {
        let icon = if icon.is_empty() { None } else { Some(icon) };
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
    match update.folder {
        0 => {
            sqlx::query("UPDATE accounts SET folder = NULL WHERE id_account = ?")
                .bind(update.id)
                .execute(&mut *connection)
                .await?;
        }
        folder => {
            sqlx::query("UPDATE accounts SET folder = ? WHERE id_account = ?")
                .bind(folder)
                .bind(update.id)
                .execute(&mut *connection)
                .await?;
        }
    }

    Ok(())
}

#[frb]
pub async fn delete_account(account: u32) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::db::delete_account(&mut connection, account).await?;

    Ok(())
}

#[frb]
pub async fn reorder_account(old_position: u32, new_position: u32) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::db::reorder_account(&mut connection, old_position, new_position).await
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
    let mut db_tx = connection.begin().await?;
    let network = c.network;

    let birth = na.birth.unwrap_or_else(|| {
        network
            .activation_height(zcash_protocol::consensus::NetworkUpgrade::Sapling)
            .unwrap()
            .into()
    });

    let account = store_account_metadata(
        &mut db_tx,
        &na.name,
        &na.icon,
        &na.fingerprint,
        birth,
        na.use_internal,
        na.internal,
    )
    .await?;

    let mut key = na.key.clone();
    if key.is_empty() && !na.ledger {
        key = generate_seed()?;
    }

    let pools = na.pools.unwrap_or(ALL_POOLS);

    if na.ledger {
        store_account_hw(&mut db_tx, account, LEDGER_CODE, na.aindex).await?;
        // we must do sapling derivation first to know a valid dindex
        // because in sapling some indices are invalid
        let dindex = 0;
        if pools & 2 != 0 {
            init_account_sapling(&network, &mut db_tx, account, birth).await?;
            let fvk = get_hw_fvk(&network, LEDGER_CODE, na.aindex).await?;
            let mut dfvk = fvk.to_bytes().to_vec();
            dfvk.extend_from_slice(&[0u8; 32]); // add a dummy dk because we cannot get the one from the Ledger
            let xvk = DiversifiableFullViewingKey::from_bytes(&tiu!(dfvk)).unwrap();
            // We should get the default address dindex by using the get_div_list
            // api but it is currently not working
            // instead, we "assume" the dindex = 0 is the default sapling address
            // let (dindex, address) = get_hw_next_diversifier_address(&network, na.aindex, 0).await?;
            let address = get_hw_sapling_address(&network, na.aindex).await?;
            store_account_sapling_vk(&mut db_tx, account, &xvk, &address).await?;
        }
        if pools & 1 != 0 {
            init_account_transparent(&mut db_tx, account, birth).await?;
            let (pk, taddr) = get_hw_transparent_address(&network, na.aindex, 0, dindex).await?;
            store_account_transparent_addr(
                &mut db_tx,
                account,
                0,
                dindex,
                None,
                &pk,
                &taddr.encode(&c.network),
            )
            .await?;
        }
        update_dindex(&mut db_tx, account, dindex, true).await?;
    } else if is_valid_phrase(&key) {
        let seed_phrase = bip39::Mnemonic::from_str(&key)?;
        let passphrase = na.passphrase.clone().unwrap_or_default();
        let seed = seed_phrase.to_seed(&passphrase);
        let seed_fingerprint = SeedFingerprint::from_seed(&seed).unwrap().to_bytes();
        store_account_seed(
            &mut db_tx,
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
            init_account_transparent(&mut db_tx, account, birth).await?;
            let tsk = usk.transparent();
            store_account_transparent_sk(&mut db_tx, account, tsk).await?;
            let tvk = &tsk.to_account_pubkey();
            store_account_transparent_vk(&mut db_tx, account, tvk).await?;
            let sk = derive_transparent_sk(tsk, 0, dindex)?;
            let (pk, taddr) = derive_transparent_address(tvk, 0, dindex)?;
            store_account_transparent_addr(
                &mut db_tx,
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
            init_account_sapling(&network, &mut db_tx, account, birth).await?;
            let sxsk = usk.sapling();
            store_account_sapling_sk(&mut db_tx, account, sxsk).await?;
            let sxvk = sxsk.to_diversifiable_full_viewing_key();
            let address = derive_sapling_address(&network, &sxvk, dindex);
            store_account_sapling_vk(&mut db_tx, account, &sxvk, &address).await?;
        }

        if pools & 4 != 0 {
            init_account_orchard(&network, &mut db_tx, account, birth).await?;
            let oxsk = usk.orchard();
            store_account_orchard_sk(&mut db_tx, account, oxsk).await?;
            let oxvk = FullViewingKey::from(oxsk);
            store_account_orchard_vk(&mut db_tx, account, &oxvk).await?;
        }

        update_dindex(&mut db_tx, account, dindex, true).await?;
    } else if is_valid_transparent_key(&key) {
        init_account_transparent(&mut db_tx, account, birth).await?;
        if let Ok(xsk) = ExtendedPrivateKey::<SecretKey>::from_str(&key) {
            let xsk = AccountPrivKey::from_extended_privkey(xsk);
            store_account_transparent_sk(&mut db_tx, account, &xsk).await?;
            let xvk = xsk.to_account_pubkey();
            store_account_transparent_vk(&mut db_tx, account, &xvk).await?;
            let sk = derive_transparent_sk(&xsk, 0, 0)?;
            let (pk, address) = derive_transparent_address(&xvk, 0, 0)?;
            store_account_transparent_addr(
                &mut db_tx,
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
            store_account_transparent_vk(&mut db_tx, account, &xvk).await?;
            let (pk, address) = derive_transparent_address(&xvk, 0, 0)?;
            store_account_transparent_addr(
                &mut db_tx,
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
            let pk = sk.public_key(&secp);
            let tpk = pk.serialize().to_vec();
            let pkh: [u8; 20] = Ripemd160::digest(Sha256::digest(&tpk)).into();
            let addr = TransparentAddress::PublicKeyHash(pkh);
            store_account_transparent_addr(
                &mut db_tx,
                account,
                0,
                0,
                Some(sk.to_bytes().to_vec()),
                &tpk,
                &addr.encode(&network),
            )
            .await?;
        } else if let Ok((_, tpk)) = bech32::decode(&key) {
            let pkh: [u8; 20] = Ripemd160::digest(Sha256::digest(&tpk)).into();
            let addr = TransparentAddress::PublicKeyHash(pkh);
            store_account_transparent_addr(
                &mut db_tx,
                account,
                0,
                0,
                None,
                &tpk,
                &addr.encode(&network),
            )
            .await?;
        }
    } else if is_valid_sapling_key(&network, &key) {
        init_account_sapling(&network, &mut db_tx, account, birth).await?;
        let di = if let Ok(xsk) = zcash_keys::encoding::decode_extended_spending_key(
            network.hrp_sapling_extended_spending_key(),
            &key,
        ) {
            store_account_sapling_sk(&mut db_tx, account, &xsk).await?;
            let xvk = xsk.to_diversifiable_full_viewing_key();
            let (di, address) = xvk.default_address();
            let address = address.encode(&network);
            store_account_sapling_vk(&mut db_tx, account, &xvk, &address).await?;
            di
        } else if let Ok(xvk) = zcash_keys::encoding::decode_extended_full_viewing_key(
            network.hrp_sapling_extended_full_viewing_key(),
            &key,
        ) {
            let (di, address) = xvk.default_address();
            let address = address.encode(&network);
            store_account_sapling_vk(
                &mut db_tx,
                account,
                &xvk.to_diversifiable_full_viewing_key(),
                &address,
            )
            .await?;
            di
        } else {
            return Err(anyhow!("Invalid Sapling Key"));
        };
        let dindex: u32 = di.try_into()?;
        update_dindex(&mut db_tx, account, dindex, true).await?;
    } else if is_valid_ufvk(&network, &key) {
        let uvk =
            UnifiedFullViewingKey::decode(&network, &key).map_err(|_| anyhow!("Invalid Key"))?;
        let (ua, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
        let dindex: u32 = di.try_into()?;

        match uvk.transparent() {
            Some(tvk) if pools & 1 != 0 => {
                init_account_transparent(&mut db_tx, account, birth).await?;
                store_account_transparent_vk(&mut db_tx, account, tvk).await?;
                let (pk, address) = derive_transparent_address(tvk, 0, dindex)?;
                store_account_transparent_addr(
                    &mut db_tx,
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
                init_account_sapling(&network, &mut db_tx, account, birth).await?;
                let address = ua.sapling().unwrap();
                let address = address.encode(&network);
                store_account_sapling_vk(&mut db_tx, account, sxvk, &address).await?;
            }
            _ => {}
        }
        match uvk.orchard() {
            Some(ovk) if pools & 4 != 0 => {
                init_account_orchard(&network, &mut db_tx, account, birth).await?;
                store_account_orchard_vk(&mut db_tx, account, ovk).await?;
            }
            _ => {}
        }
        update_dindex(&mut db_tx, account, dindex, true).await?;
    }
    db_tx.commit().await?;
    Ok(account)
}

fn derive_sapling_address(
    network: &Network,
    sxvk: &DiversifiableFullViewingKey,
    dindex: u32,
) -> String {
    let address = sxvk.address(dindex.into()).unwrap();
    address.encode(network)
}

#[frb]
pub async fn has_transparent_pub_key() -> Result<bool> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    let r = crate::account::has_transparent_pub_key(&mut connection, c.account).await?;
    Ok(r)
}

#[frb]
pub async fn generate_next_dindex() -> Result<u32> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::generate_next_dindex(&c.network, &mut connection, c.account).await
}

#[frb]
pub async fn generate_next_change_address() -> Result<Option<String>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::generate_next_change_address(&c.network, &mut connection, c.account).await
}

#[frb]
pub async fn reset_sync(id: u32) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::reset_sync(&c.network, &mut connection, id).await
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
    pub folder: Folder,
    pub position: u8,
    pub hidden: bool,
    pub saved: bool,
    pub enabled: bool,
    pub internal: bool,
    pub hw: u8,
    pub height: u32,
    pub time: u32,
    pub balance: u64,
}

#[frb(dart_metadata = ("freezed"))]
pub struct AccountUpdate {
    pub coin: u8,
    pub id: u32,
    pub name: Option<String>,
    pub icon: Option<Vec<u8>>,
    pub birth: Option<u32>,
    pub folder: u32,
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
    pub folder: String,
    pub pools: Option<u8>,
    pub use_internal: bool,
    pub internal: bool,
    pub ledger: bool,
}

#[frb(dart_metadata = ("freezed"))]
pub struct Tx {
    pub id: u32,
    pub txid: Vec<u8>,
    pub height: u32,
    pub time: u32,
    pub value: i64,
    pub tpe: Option<u8>,
    pub category: Option<String>,
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
    let memos = crate::db::fetch_memos(&mut connection, c.account).await?;
    Ok(memos)
}

#[frb]
pub async fn get_addresses(ua_pools: u8) -> Result<Addresses> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    let dindex = crate::db::get_account_dindex(&mut connection, c.account).await?;

    let tkeys = crate::db::select_account_transparent(&mut connection, c.account, dindex).await?;
    let skeys = crate::db::select_account_sapling(&c.network, &mut connection, c.account).await?;
    let okeys = crate::db::select_account_orchard(&mut connection, c.account).await?;

    let taddr = tkeys
        .xvk
        .as_ref()
        .map(|xvk| derive_transparent_address(xvk, 0, dindex).unwrap().1);

    let dindex = dindex as u64;
    let saddr = skeys.address;
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
    let tx = crate::account::get_tx_details(&mut connection, c.account, id_tx).await?;
    Ok(tx)
}

#[frb]
pub async fn list_notes() -> Result<Vec<TxNote>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    let notes = crate::db::get_notes(&mut connection, c.account).await?;
    Ok(notes)
}

#[frb]
pub async fn lock_note(id: u32, locked: bool) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    crate::db::lock_note(&mut connection, c.account, id, locked).await?;
    Ok(())
}

#[frb]
pub async fn fetch_transparent_address_tx_count() -> Result<Vec<TAddressTxCount>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    crate::db::fetch_transparent_address_tx_count(&mut connection, c.account).await
}

#[frb]
pub async fn export_account(id: u32, passphrase: &str) -> Result<Vec<u8>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    let data = crate::io::export_account(&mut connection, id).await?;
    let encrypted = encrypt(passphrase, &data)?;
    Ok(encrypted)
}

#[frb]
pub async fn import_account(passphrase: &str, data: &[u8]) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    let decrypted = decrypt(passphrase, data)?;
    crate::io::import_account(&mut connection, &decrypted).await?;
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

    crate::account::get_account_frost_params(&mut connection, c.account).await
}

#[frb(dart_metadata = ("freezed"))]
pub struct FrostParams {
    pub id: u8,
    pub n: u8,
    pub t: u8,
}

#[frb]
pub async fn list_folders() -> Result<Vec<Folder>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::list_folders(&mut connection).await
}

#[frb]
pub async fn create_new_folder(name: &str) -> Result<Folder> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::create_new_folder(&mut connection, name).await
}

#[frb]
pub async fn rename_folder(id: u32, name: &str) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::rename_folder(&mut connection, id, name).await
}

#[frb]
pub async fn delete_folders(ids: &[u32]) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::delete_folders(&mut connection, ids).await
}

#[frb(dart_metadata = ("freezed"))]
pub struct Folder {
    pub id: u32,
    pub name: String,
}

#[frb]
pub async fn list_categories() -> Result<Vec<Category>> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::list_categories(&mut connection).await
}

#[frb]
pub async fn create_new_category(category: &Category) -> Result<u32> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::create_new_category(&mut connection, category).await
}

#[frb]
pub async fn rename_category(category: &Category) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::rename_category(&mut connection, category).await
}

#[frb]
pub async fn delete_categories(ids: &[u32]) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;

    crate::account::delete_categories(&mut connection, ids).await
}

#[frb(dart_metadata = ("freezed"))]
pub struct Category {
    pub id: u32,
    pub name: String,
    pub is_income: bool,
}

#[frb]
pub async fn get_exported_data(r#type: u8) -> Result<String> {
    let c = get_coin!();
    let buffer = vec![];
    let mut writer = AsyncWriter::from_writer(buffer);

    let mut connection = c.get_connection().await?;
    crate::db::export_data(&mut connection, c.account, r#type, &mut writer).await?;
    let buffer = writer.into_inner().await?;
    let res = String::from_utf8(buffer).unwrap();
    Ok(res)
}

#[frb]
pub async fn lock_recent_notes(height: u32, threshold: u32) -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    crate::db::lock_recent_notes(&mut connection, c.account, height, threshold).await
}

#[frb]
pub async fn unlock_all_notes() -> Result<()> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    crate::db::unlock_all_notes(&mut connection, c.account).await
}

#[frb]
pub async fn max_spendable() -> Result<u64> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    crate::db::max_spendable(&mut connection, c.account).await
}

#[frb]
pub async fn show_ledger_sapling_address() -> Result<String> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    let r = show_sapling_address(&c.network, &mut connection, c.account).await?;
    Ok(r)
}

#[frb]
pub async fn show_ledger_transparent_address() -> Result<String> {
    let c = get_coin!();
    let mut connection = c.get_connection().await?;
    let r = show_transparent_address(&c.network, &mut connection, c.account).await?;
    Ok(r)
}
