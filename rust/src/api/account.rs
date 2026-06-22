use std::collections::HashMap;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use bip39::Mnemonic;
use csv_async::AsyncWriter;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;
use sapling_crypto::{zip32::sapling_derive_internal_fvk, PaymentAddress};
use crate::keys::{SaplingAddressDerivation, ScopeExt};
use crate::pay::pool::PoolMask;
use sqlx::{Row, SqliteConnection, sqlite::SqliteRow};
use zcash_address::unified::{Container, Encoding};
use zcash_keys::{
    address::UnifiedAddress,
    encoding::AddressCodec,
    keys::{UnifiedAddressRequest, UnifiedFullViewingKey, UnifiedSpendingKey},
};
use zcash_protocol::consensus::Parameters as ZkParams;
use zcash_transparent::address::TransparentAddress;
use zip32::AccountId;

use crate::{api::pay::PcztPackage, frb_generated::StreamSink};
use crate::{
    api::{coin::Coin, pay::SigningEvent},
    db::{get_account_dindex, get_account_hw},
    io::{decrypt, encrypt},
    ledger::HWAPI,
};

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_account_pools(account: u32, c: &Coin) -> Result<u8> {
    let mut connection = c.get_connection().await?;

    let dindex = get_account_dindex(&mut connection, account).await?;
    let tkeys = crate::db::select_account_transparent(&mut connection, account, dindex).await?;
    let skeys = crate::db::select_account_sapling(&c.network(), &mut connection, account).await?;
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

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_account_ufvk(account: u32, pools: u8, c: &Coin) -> Result<String> {
    let network = c.network();
    let mut connection = c.get_connection().await?;

    let ufvk = crate::key::get_account_ufvk(&network, &mut connection, account, pools).await?;
    Ok(ufvk)
}

pub async fn get_account_seed(account: u32, c: &Coin) -> Result<Option<Seed>> {
    let mut connection = c.get_connection().await?;
    crate::account::get_account_seed(&mut connection, account).await
}

#[cfg_attr(feature = "flutter", frb)]
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
pub struct Seed {
    pub mnemonic: String,
    pub phrase: String,
    pub aindex: u32,
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_account_fingerprint(account: u32, c: &Coin) -> Result<Option<String>> {
    let mut connection = c.get_connection().await?;

    let fingerprint = crate::db::get_account_fingerprint(&mut connection, account).await?;
    let fingerprint = fingerprint.as_ref().map(|fp| hex::encode(&fp[..4]));
    Ok(fingerprint)
}

#[cfg_attr(feature = "flutter", frb(sync))]
pub fn ua_from_ufvk(ufvk: &str, di: Option<u32>, c: &Coin) -> Result<String> {
    let network = c.network();

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

#[cfg_attr(feature = "flutter", frb(sync))]
pub fn receivers_from_ua(ua: &str, c: &Coin) -> Result<Receivers> {
    let network = c.network();

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

#[cfg_attr(feature = "flutter", frb)]
pub async fn list_accounts(c: &Coin) -> Result<Vec<Account>> {
    let mut connection = c.get_connection().await?;
    let accounts = crate::db::list_accounts(&mut connection, c.coin).await?;

    Ok(accounts)
}

#[cfg_attr(feature = "flutter", frb)]
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

#[cfg_attr(feature = "flutter", frb)]
pub async fn delete_account(account: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;

    crate::db::delete_account(&mut connection, account).await?;

    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn reorder_account(old_position: u32, new_position: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;

    crate::db::reorder_account(&mut connection, old_position, new_position).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn new_account(na: &NewAccount, c: &Coin) -> Result<u32> {
    let mut connection = c.get_connection().await?;
    crate::account::new_account(&c.network(), &mut connection, na).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn has_transparent_pub_key(c: &Coin) -> Result<bool> {
    let mut connection = c.get_connection().await?;
    let r = crate::account::has_transparent_pub_key(&mut connection, c.account).await?;
    Ok(r)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn generate_next_dindex(c: &Coin) -> Result<u32> {
    let mut connection = c.get_connection().await?;

    crate::account::generate_next_dindex(&c.network(), &mut connection, c.account).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn generate_next_change_address(c: &Coin) -> Result<Option<String>> {
    let mut connection = c.get_connection().await?;

    crate::account::generate_next_change_address(&c.network(), &mut connection, c.account).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn reset_sync(id: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;

    crate::account::reset_sync(&c.network(), &mut connection, id).await
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
pub struct Account {
    pub coin: u8,
    pub id: u32,
    pub name: String,
    pub seed: Option<String>,
    pub passphrase: Option<String>,
    pub aindex: u32,
    pub dindex: u32,
    pub icon: Option<Vec<u8>>,
    pub use_internal: bool,
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

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
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

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
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

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
pub struct Tx {
    pub id: u32,
    pub txid: Vec<u8>,
    pub height: u32,
    pub time: u32,
    pub value: i64,
    pub tpe: Option<u8>,
    pub category: Option<String>,
    pub zsa_value: i64,
    pub asset_id: Option<i32>,
    pub asset_display: String,
    pub price: Option<f64>,
    pub memo: Option<String>,
    pub is_user_memo: bool,
    pub contact_name: Option<String>,
}

pub struct TAddressTxCount {
    pub pool: u8,
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
    tracing::debug!("list_tx_history: start for account {}", c.account);
    let mut connection = c.get_connection().await?;
    let txs = crate::db::fetch_txs(&mut connection, c.account).await?;
    tracing::debug!("list_tx_history: got {} transactions", txs.len());
    Ok(txs)
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
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
    pub is_user_memo: bool,
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn list_memos(c: &Coin) -> Result<Vec<Memo>> {
    let mut connection = c.get_connection().await?;
    let memos = crate::db::get_memos(&mut connection, c.account).await?;
    Ok(memos)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_addresses(ua_pools: u8, c: &Coin) -> Result<Addresses> {
    let mut connection = c.get_connection().await?;
    crate::account::get_addresses(&c.network(), &mut connection, c.account, ua_pools).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_account_addresses(account: u32, ua_pools: u8, c: &Coin) -> Result<Addresses> {
    let mut connection = c.get_connection().await?;
    crate::account::get_addresses(&c.network(), &mut connection, account, ua_pools).await
}

pub struct Addresses {
    pub taddr: Option<String>,
    pub saddr: Option<String>,
    pub oaddr: Option<String>,
    pub ua: Option<String>,
    pub diversifier_index: u32,
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_tx_details(id_tx: u32, c: &Coin) -> Result<TxAccount> {
    let mut connection = c.get_connection().await?;
    let tx = crate::account::get_tx_details(&mut connection, c.account, id_tx).await?;
    Ok(tx)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn list_notes(c: &Coin) -> Result<Vec<TxNote>> {
    let mut connection = c.get_connection().await?;
    let notes = crate::db::get_notes(&mut connection, c.account).await?;
    Ok(notes)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn lock_note(id: u32, locked: bool, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::lock_note(&mut connection, c.account, id, locked).await?;
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn fetch_transparent_address_tx_count(c: &Coin) -> Result<Vec<TAddressTxCount>> {
    let mut connection = c.get_connection().await?;
    crate::db::fetch_transparent_address_tx_count(&mut connection, c.account).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn fetch_address_tx_count(c: &Coin, aggregate: bool, pool_filter: u8) -> Result<Vec<TAddressTxCount>> {
    let mut connection = c.get_connection().await?;
    let network = c.network();

    let dindex = get_account_dindex(&mut connection, c.account).await?;

    let t_pool = crate::db::select_account_transparent(&mut connection, c.account, dindex).await;
    let s_pool = crate::db::select_account_sapling(&network, &mut connection, c.account).await;
    let o_pool = crate::db::select_account_orchard(&mut connection, c.account).await;

    let enabled_pools = (if t_pool.as_ref().map(|t| t.xvk.is_some() || t.address.is_some()).unwrap_or(false) { 1u8 } else { 0u8 })
        | (if s_pool.as_ref().map(|s| s.xvk.is_some()).unwrap_or(false) { 2u8 } else { 0u8 })
        | (if o_pool.as_ref().map(|o| o.xvk.is_some()).unwrap_or(false) { 4u8 } else { 0u8 });

    // Only derive addresses for pools that are both selected and enabled
    let selected = PoolMask(pool_filter).intersect(&PoolMask(enabled_pools));

    let tkeys = t_pool?;
    let skeys = s_pool?;
    let okeys = o_pool?;

    // Fetch per-slot stats for transparent and shielded pools
    let transparent_stats = crate::db::fetch_transparent_slot_stats(&mut connection, c.account).await?;
    let shielded_stats = crate::db::fetch_shielded_slot_stats(&mut connection, c.account).await?;

    // Per-pool stats lookups (kept separate, not merged)
    let t_stats: HashMap<(u8, u32), (u64, u32, u32)> = transparent_stats
        .iter()
        .map(|s| ((s.scope, s.dindex), (s.amount, s.tx_count, s.time)))
        .collect();
    let s_stats: HashMap<(u8, u8, u32), (u64, u32, u32)> = shielded_stats
        .iter()
        .map(|s| ((s.pool, s.scope, s.dindex), (s.amount, s.tx_count, s.time)))
        .collect();

    // Pre-compute internal Sapling IVK
    let internal_sap_ivk: Option<sapling_crypto::zip32::IncomingViewingKey> = skeys.xvk.as_ref().and_then(|dfvk| {
        let dfvk_bytes = dfvk.to_bytes();
        let dk_bytes: [u8; 32] = dfvk_bytes[96..128].try_into().ok()?;
        let dk = sapling_crypto::zip32::DiversifierKey::from_bytes(dk_bytes);
        let (int_fvk, int_dk) = sapling_derive_internal_fvk(dfvk.fvk(), &dk);
        let mut ivk_bytes = [0u8; 64];
        ivk_bytes[..32].copy_from_slice(int_dk.as_bytes());
        ivk_bytes[32..].copy_from_slice(&int_fvk.vk.ivk().to_repr());
        sapling_crypto::zip32::IncomingViewingKey::from_bytes(&ivk_bytes).into_option()
    });

    let mut results = Vec::new();

    for scope in [0u8, 1u8] {
        for d in 0..=dindex {
            // Determine if this (scope, d) slot is valid.
            // Sapling diversifier validity gates the entire slot — if the
            // account has sapling keys and the address at this diversifier
            // index is invalid, only the transparent fallback at d=0 applies.
            let mut valid_dindex = true;
            if skeys.xvk.is_some() {
                let dfvk = skeys.xvk.as_ref().unwrap();
                if !dfvk.has_sapling_address(scope, d as u64, internal_sap_ivk.as_ref()) {
                    valid_dindex = false;
                }
            }

            if valid_dindex {
                // Per-pool stats
                let t_st = t_stats.get(&(scope, d)).copied().unwrap_or((0, 0, 0));
                let s_st = s_stats.get(&(1, scope, d)).copied().unwrap_or((0, 0, 0));
                let o_st = s_stats.get(&(2, scope, d)).copied().unwrap_or((0, 0, 0));

                // Derive addresses for selected pools
                let mut t_addr_raw: Option<TransparentAddress> = None;
                let t_str = if selected.has_pool(0) {
                    if let Some(tvk) = &tkeys.xvk {
                        let (_, taddr) = crate::account::derive_transparent_address(tvk, scope as u32, d, false)?;
                        t_addr_raw = Some(taddr);
                        Some(taddr.encode(&network))
                    } else { None }
                } else { None };

                let mut s_addr_raw: Option<sapling_crypto::PaymentAddress> = None;
                let s_str = if selected.has_pool(1) {
                    if let Some(dfvk) = &skeys.xvk {
                        s_addr_raw = dfvk.sapling_address_at(scope, d as u64, internal_sap_ivk.as_ref());
                        s_addr_raw.as_ref().map(|pa| pa.encode(&network))
                    } else { None }
                } else { None };

                let mut o_addr_raw: Option<orchard::Address> = None;
                let o_str = if selected.has_pool(2) {
                    if let Some(fvk) = &okeys.xvk {
                        let s = scope.orchard_scope();
                        let addr = fvk.address_at(d as u64, s);
                        o_addr_raw = Some(addr);
                        Some(UnifiedAddress::from_receivers(Some(addr), None, None).unwrap().encode(&network))
                    } else { None }
                } else { None };

                if aggregate {
                    if let Some(ua) = UnifiedAddress::from_receivers(o_addr_raw, s_addr_raw, t_addr_raw) {
                        results.push(TAddressTxCount {
                            pool: 0, address: ua.encode(&network), scope, dindex: d,
                            amount: t_st.0.wrapping_add(s_st.0).wrapping_add(o_st.0),
                            tx_count: t_st.1.wrapping_add(s_st.1).wrapping_add(o_st.1),
                            time: t_st.2.max(s_st.2).max(o_st.2),
                        });
                    }
                } else {
                    if let Some(addr) = t_str {
                        results.push(TAddressTxCount { pool: 0, address: addr, scope, dindex: d, amount: t_st.0, tx_count: t_st.1, time: t_st.2 });
                    }
                    if let Some(addr) = s_str {
                        results.push(TAddressTxCount { pool: 1, address: addr, scope, dindex: d, amount: s_st.0, tx_count: s_st.1, time: s_st.2 });
                    }
                    if let Some(addr) = o_str {
                        results.push(TAddressTxCount { pool: 2, address: addr, scope, dindex: d, amount: o_st.0, tx_count: o_st.1, time: o_st.2 });
                    }
                }
            } else if d == 0 && selected.has_pool(0) {
                // Fallback: sapling address invalid at this slot, but d=0
                // always produces a valid transparent address.
                let t_st = t_stats.get(&(scope, 0)).copied().unwrap_or((0, 0, 0));

                let t_str = if let Some(tvk) = &tkeys.xvk {
                    let (_, taddr) = crate::account::derive_transparent_address(
                        tvk, scope as u32, 0, false,
                    )?;
                    Some(taddr.encode(&network))
                } else { None };

                // Aggregate of 1 taddr is the taddr itself.
                if let Some(addr) = t_str {
                    results.push(TAddressTxCount {
                        pool: 0, address: addr, scope, dindex: 0,
                        amount: t_st.0, tx_count: t_st.1, time: t_st.2,
                    });
                }
            }
        }
    }

    results.sort_by(|a, b| a.scope.cmp(&b.scope).then(a.dindex.cmp(&b.dindex)).then(a.pool.cmp(&b.pool)));
    Ok(results)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn export_account(id: u32, passphrase: &str, c: &Coin) -> Result<Vec<u8>> {
    let mut connection = c.get_connection().await?;

    let data = crate::io::export_account(&mut connection, id).await?;
    let encrypted = encrypt(passphrase, &data)?;
    Ok(encrypted)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn import_account(passphrase: &str, data: &[u8], c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;

    let decrypted = decrypt(passphrase, data)?;
    crate::io::import_account(&mut connection, &decrypted).await?;
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn print_keys(id: u32, c: &Coin) -> Result<()> {
    let network = c.network();
    let mut connection = c.get_connection().await?;

    let (seed, aindex) = sqlx::query(
        "SELECT name, seed, seed_fingerprint, aindex, dindex,
        def_dindex, birth FROM accounts WHERE id_account = ?",
    )
    .bind(id)
    .map(|row: SqliteRow| {
        let _name: String = row.get(0);
        let seed: Option<String> = row.get(1);
        let _seed_fingerprint: Vec<u8> = row.get(2);
        let aindex: u32 = row.get(3);
        let _dindex: u32 = row.get(4);
        let _def_dindex: u32 = row.get(5);
        let _birth: u32 = row.get(6);

        (seed, aindex)
    })
    .fetch_one(&mut *connection)
    .await?;


    let seed = seed.unwrap();
    let memo = Mnemonic::from_str(&seed).unwrap();
    let seed = memo.to_seed("");

    let usk = UnifiedSpendingKey::from_seed(&network, &seed, AccountId::try_from(aindex).unwrap())?;
    let uvk = usk.to_unified_full_viewing_key();
    if uvk.sapling().is_some() {
        println!("Has Sapling");
    }
    if uvk.orchard().is_some() {
        println!("Has Orchard");
    }
    let uvk = uvk.encode(&network);
    println!("Unified Full Viewing Key: {}", uvk);

    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_account_frost_params(c: &Coin) -> Result<Option<FrostParams>> {
    let mut connection = c.get_connection().await?;

    crate::account::get_account_frost_params(&mut connection, c.account).await
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
pub struct FrostParams {
    pub id: u8,
    pub n: u8,
    pub t: u8,
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn list_folders(c: &Coin) -> Result<Vec<Folder>> {
    let mut connection = c.get_connection().await?;

    crate::account::list_folders(&mut connection).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn create_new_folder(name: &str, c: &Coin) -> Result<Folder> {
    let mut connection = c.get_connection().await?;

    crate::account::create_new_folder(&mut connection, name).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn rename_folder(id: u32, name: &str, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;

    crate::account::rename_folder(&mut connection, id, name).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn delete_folders(ids: &[u32], c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;

    crate::account::delete_folders(&mut connection, ids).await
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
pub struct Folder {
    pub id: u32,
    pub name: String,
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn list_categories(c: &Coin) -> Result<Vec<Category>> {
    let mut connection = c.get_connection().await?;

    crate::account::list_categories(&mut connection).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn create_new_category(category: &Category, c: &Coin) -> Result<u32> {
    let mut connection = c.get_connection().await?;

    crate::account::create_new_category(&mut connection, category).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn rename_category(category: &Category, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;

    crate::account::rename_category(&mut connection, category).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn delete_categories(ids: &[u32], c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;

    crate::account::delete_categories(&mut connection, ids).await
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
pub struct Category {
    pub id: u32,
    pub name: String,
    pub is_income: bool,
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_exported_data(r#type: u8, c: &Coin) -> Result<String> {
    let buffer = vec![];
    let mut writer = AsyncWriter::from_writer(buffer);

    let mut connection = c.get_connection().await?;
    crate::db::export_data(&mut connection, c.account, r#type, &mut writer).await?;
    let buffer = writer.into_inner().await?;
    let res = String::from_utf8(buffer).unwrap();
    Ok(res)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn lock_recent_notes(height: u32, threshold: u32, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::lock_recent_notes(&mut connection, c.account, height, threshold).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn unlock_all_notes(c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    crate::db::unlock_all_notes(&mut connection, c.account).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn max_spendable(c: &Coin) -> Result<u64> {
    let mut connection = c.get_connection().await?;
    crate::db::max_spendable(&mut connection, c.account).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn show_ledger_sapling_address(c: &Coin) -> Result<String> {
    let mut connection = c.get_connection().await?;
    let ledger = get_ledger(&mut connection, c.account).await?;
    let r = ledger.show_sapling_address(&c.network(), &mut connection, c.account).await?;
    Ok(r)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn show_ledger_transparent_address(c: &Coin) -> Result<String> {
    let mut connection = c.get_connection().await?;
    let ledger = get_ledger(&mut connection, c.account).await?;
    let r = ledger.show_transparent_address(&c.network(), &mut connection, c.account).await?;
    Ok(r)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn sign_ledger_transaction(
    sink: StreamSink<SigningEvent>,
    package: PcztPackage,
    c: &Coin,
) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let ledger = get_ledger(&mut connection, c.account).await?;
    ledger.sign_ledger_transaction(sink, package, c).await?;
    Ok(())
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
    pub user_memo: Option<String>,
}

#[derive(Default, Debug)]
pub struct TxNote {
    pub id: u32,
    pub pool: u8,
    pub height: u32,
    pub tx: u32,
    pub scope: u8,
    pub diversifier: Option<Vec<u8>>,
    pub diversifier_index: Option<i64>,
    pub value: u64,
    pub locked: bool,
    pub memo: Option<String>,
    pub id_asset: Option<u32>,
    pub asset_display: String,
}

#[derive(Default, Debug)]
pub struct TxSpend {
    pub id: u32,
    pub pool: u8,
    pub height: u32,
    pub value: u64,
    pub id_asset: Option<u32>,
    pub asset_display: String,
}

#[derive(Default, Debug)]
pub struct TxOutput {
    pub id: u32,
    pub pool: u8,
    pub height: u32,
    pub value: u64,
    pub address: String,
    pub contact_name: Option<String>,
}

#[derive(Default, Debug)]
pub struct TxMemo {
    pub note: Option<u32>,
    pub output: Option<u32>,
    pub pool: u8,
    pub memo: Option<String>,
    pub memo_bytes: Vec<u8>,
}

pub(crate) async fn get_ledger(connection: &mut SqliteConnection, account: u32) -> Result<Box<dyn HWAPI + Send + Sync>> {
    let hw = get_account_hw(connection, account).await?;
    let r: Box<dyn HWAPI + Send + Sync> = if hw == 1 {
        #[cfg(feature = "ledger")]
        let d = Box::new(crate::ledger::nano::NanoLedger {});

        #[cfg(not(feature = "ledger"))]
        let d = Box::new(());

        d
    } else {
        Box::new(())
    };
    Ok(r)
}

#[frb]
pub fn dummy_export(_a: SigningEvent) {}
