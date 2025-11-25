use std::str::FromStr;

use anyhow::{anyhow, Result};
use bip32::Prefix;
use bip39::Mnemonic;
use csv_async::AsyncWriter;
use flutter_rust_bridge::frb;
use orchard::keys::Scope;
use sapling_crypto::PaymentAddress;
use sqlx::{sqlite::SqliteRow, Row};
use tracing::info;
use zcash_address::unified::{Container, Encoding};
use zcash_keys::{
    address::UnifiedAddress,
    encoding::AddressCodec,
    keys::{UnifiedAddressRequest, UnifiedFullViewingKey, UnifiedSpendingKey},
};
use zcash_primitives::{
    consensus::Parameters as ZkParams, legacy::TransparentAddress, zip32::AccountId,
};

use crate::{
    account::derive_transparent_address,
    api::coin::Coin,
    db::get_account_dindex,
    io::{decrypt, encrypt},
};

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
use crate::ledger::fvk::{
    show_sapling_address,
    show_transparent_address,
};
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
use crate::no_ledger::{
    show_sapling_address,
    show_transparent_address,
};

#[frb]
pub async fn get_account_pools(account: u32, c: &Coin) -> Result<u8> {
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
pub async fn get_account_ufvk(account: u32, pools: u8, c: &Coin) -> Result<String> {
    let network = c.network;
    let mut connection = c.get_connection().await?;

    let ufvk = crate::key::get_account_ufvk(&network, &mut connection, account, pools).await?;
    Ok(ufvk)
}

pub async fn get_account_seed(account: u32, c: &Coin) -> Result<Option<Seed>> {
    let mut connection = c.get_connection().await?;
    crate::account::get_account_seed(&mut connection, account).await
}

#[frb(dart_metadata = ("freezed"))]
pub struct Seed {
    pub mnemonic: String,
    pub phrase: String,
    pub aindex: u32,
}

#[frb]
pub async fn get_account_fingerprint(account: u32, c: &Coin) -> Result<Option<String>> {
    let mut connection = c.get_connection().await?;

    let fingerprint = crate::db::get_account_fingerprint(&mut connection, account).await?;
    let fingerprint = fingerprint.as_ref().map(|fp| hex::encode(&fp[..4]));
    Ok(fingerprint)
}

#[frb(sync)]
pub fn ua_from_ufvk(ufvk: &str, di: Option<u32>, c: &Coin) -> Result<String> {
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
pub fn receivers_from_ua(ua: &str, c: &Coin) -> Result<Receivers> {
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
pub async fn list_accounts(c: &Coin) -> Result<Vec<Account>> {
    let mut connection = c.get_connection().await?;
    let accounts = crate::db::list_accounts(&mut connection, c.coin).await?;

    Ok(accounts)
}

#[frb]
pub async fn update_account(update: &AccountUpdate, c: &Coin) -> Result<()> {
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
pub async fn delete_account(account: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;

    crate::db::delete_account(&mut connection, account).await?;

    Ok(())
}

#[frb]
pub async fn reorder_account(old_position: u32, new_position: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;

    crate::db::reorder_account(&mut connection, old_position, new_position).await
}

#[frb]
pub async fn new_account(na: &NewAccount, c: &Coin) -> Result<u32> {
    let mut connection = c.get_connection().await?;
    crate::account::new_account(&c.network, &mut connection, na).await
}

#[frb]
pub async fn has_transparent_pub_key(c: &Coin) -> Result<bool> {
    let mut connection = c.get_connection().await?;
    let r = crate::account::has_transparent_pub_key(&mut connection, c.account).await?;
    Ok(r)
}

#[frb]
pub async fn generate_next_dindex(c: &Coin) -> Result<u32> {
    let mut connection = c.get_connection().await?;

    crate::account::generate_next_dindex(&c.network, &mut connection, c.account).await
}

#[frb]
pub async fn generate_next_change_address(c: &Coin) -> Result<Option<String>> {
    let mut connection = c.get_connection().await?;

    crate::account::generate_next_change_address(&c.network, &mut connection, c.account).await
}

#[frb]
pub async fn reset_sync(id: u32, c: &Coin) -> Result<()> {
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

pub async fn remove_account(account_id: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::delete_account(&mut connection, account_id).await?;
    Ok(())
}

pub async fn list_tx_history(c: &Coin) -> Result<Vec<Tx>> {
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
pub async fn list_memos(c: &Coin) -> Result<Vec<Memo>> {
    let mut connection = c.get_connection().await?;
    let memos = crate::db::fetch_memos(&mut connection, c.account).await?;
    Ok(memos)
}

#[frb]
pub async fn get_addresses(ua_pools: u8, c: &Coin) -> Result<Addresses> {
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
pub async fn get_tx_details(id_tx: u32, c: &Coin) -> Result<TxAccount> {
    let mut connection = c.get_connection().await?;
    let tx = crate::account::get_tx_details(&mut connection, c.account, id_tx).await?;
    Ok(tx)
}

#[frb]
pub async fn list_notes(c: &Coin) -> Result<Vec<TxNote>> {
    let mut connection = c.get_connection().await?;
    let notes = crate::db::get_notes(&mut connection, c.account).await?;
    Ok(notes)
}

#[frb]
pub async fn lock_note(id: u32, locked: bool, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::lock_note(&mut connection, c.account, id, locked).await?;
    Ok(())
}

#[frb]
pub async fn fetch_transparent_address_tx_count(c: &Coin) -> Result<Vec<TAddressTxCount>> {
    let mut connection = c.get_connection().await?;
    crate::db::fetch_transparent_address_tx_count(&mut connection, c.account).await
}

#[frb]
pub async fn export_account(id: u32, passphrase: &str, c: &Coin) -> Result<Vec<u8>> {
    let mut connection = c.get_connection().await?;

    let data = crate::io::export_account(&mut connection, id).await?;
    let encrypted = encrypt(passphrase, &data)?;
    Ok(encrypted)
}

#[frb]
pub async fn import_account(passphrase: &str, data: &[u8], c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;

    let decrypted = decrypt(passphrase, data)?;
    crate::io::import_account(&mut connection, &decrypted).await?;
    Ok(())
}

#[frb]
pub async fn print_keys(id: u32, c: &Coin) -> Result<()> {
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
pub async fn get_account_frost_params(c: &Coin) -> Result<Option<FrostParams>> {
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
pub async fn list_folders(c: &Coin) -> Result<Vec<Folder>> {
    let mut connection = c.get_connection().await?;

    crate::account::list_folders(&mut connection).await
}

#[frb]
pub async fn create_new_folder(name: &str, c: &Coin) -> Result<Folder> {
    let mut connection = c.get_connection().await?;

    crate::account::create_new_folder(&mut connection, name).await
}

#[frb]
pub async fn rename_folder(id: u32, name: &str, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;

    crate::account::rename_folder(&mut connection, id, name).await
}

#[frb]
pub async fn delete_folders(ids: &[u32], c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;

    crate::account::delete_folders(&mut connection, ids).await
}

#[frb(dart_metadata = ("freezed"))]
pub struct Folder {
    pub id: u32,
    pub name: String,
}

#[frb]
pub async fn list_categories(c: &Coin) -> Result<Vec<Category>> {
    let mut connection = c.get_connection().await?;

    crate::account::list_categories(&mut connection).await
}

#[frb]
pub async fn create_new_category(category: &Category, c: &Coin) -> Result<u32> {
    let mut connection = c.get_connection().await?;

    crate::account::create_new_category(&mut connection, category).await
}

#[frb]
pub async fn rename_category(category: &Category, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;

    crate::account::rename_category(&mut connection, category).await
}

#[frb]
pub async fn delete_categories(ids: &[u32], c: &Coin) -> Result<()> {
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
pub async fn get_exported_data(r#type: u8, c: &Coin) -> Result<String> {
    let buffer = vec![];
    let mut writer = AsyncWriter::from_writer(buffer);

    let mut connection = c.get_connection().await?;
    crate::db::export_data(&mut connection, c.account, r#type, &mut writer).await?;
    let buffer = writer.into_inner().await?;
    let res = String::from_utf8(buffer).unwrap();
    Ok(res)
}

#[frb]
pub async fn lock_recent_notes(height: u32, threshold: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::lock_recent_notes(&mut connection, c.account, height, threshold).await
}

#[frb]
pub async fn unlock_all_notes(c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::unlock_all_notes(&mut connection, c.account).await
}

#[frb]
pub async fn max_spendable(c: &Coin) -> Result<u64> {
    let mut connection = c.get_connection().await?;
    crate::db::max_spendable(&mut connection, c.account).await
}

#[frb]
pub async fn show_ledger_sapling_address(c: &Coin) -> Result<String> {
    let mut connection = c.get_connection().await?;
    let r = show_sapling_address(&c.network, &mut connection, c.account).await?;
    Ok(r)
}

#[frb]
pub async fn show_ledger_transparent_address(c: &Coin) -> Result<String> {
    let mut connection = c.get_connection().await?;
    let r = show_transparent_address(&c.network, &mut connection, c.account).await?;
    Ok(r)
}

#[derive(Default, Debug)]
pub struct TxAccount {
    pub id: u32,
    pub account: u32,
    pub txid: Vec<u8>,
    pub height: u32,
    pub time: u32,
    pub price: Option<f64>,
    pub category: Option<u32>,
    pub notes: Vec<TxNote>,
    pub spends: Vec<TxSpend>,
    pub outputs: Vec<TxOutput>,
    pub memos: Vec<TxMemo>,
}

#[derive(Default, Debug)]
pub struct TxNote {
    pub id: u32,
    pub pool: u8,
    pub height: u32,
    pub value: u64,
    pub locked: bool,
}

#[derive(Default, Debug)]
pub struct TxSpend {
    pub id: u32,
    pub pool: u8,
    pub height: u32,
    pub value: u64,
}

#[derive(Default, Debug)]
pub struct TxOutput {
    pub id: u32,
    pub pool: u8,
    pub height: u32,
    pub value: u64,
    pub address: String,
}

#[derive(Default, Debug)]
pub struct TxMemo {
    pub note: Option<u32>,
    pub output: Option<u32>,
    pub pool: u8,
    pub memo: Option<String>,
}
