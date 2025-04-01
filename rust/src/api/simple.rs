use std::{cell::RefCell, str::FromStr};

use anyhow::Result;
use flutter_rust_bridge::frb;
use orchard::keys::FullViewingKey;
use zcash_keys::{
    encoding::AddressCodec,
    keys::{UnifiedAddressRequest, UnifiedSpendingKey},
};
use zcash_primitives::zip32::AccountId;
use zcash_protocol::consensus::Network;
use zcash_transparent::keys::{NonHardenedChildIndex, TransparentKeyScope};

use crate::{
    coin::{Coin, COINS},
    db::create_schema,
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

thread_local! {
    pub static COIN: RefCell<Option<Coin>> = RefCell::new(None);
}

macro_rules! setup {
    ($coin: expr, $account: expr) => {
        let coins = COINS.lock().unwrap();
        let mut c = coins[$coin as usize].clone();
        c.account = $account;
        COIN.with(|cc| {
            *cc.borrow_mut() = Some(c);
        });
    };
}

#[macro_export]
macro_rules! coin {
    () => {
        crate::api::simple::COIN.with(|cc| cc.borrow().clone().unwrap())
    };
}

#[frb(sync)]
pub fn store_account_metadata(
    coin: u8,
    name: &str,
    icon: Option<Vec<u8>>,
    birth: u32,
    height: u32,
) -> Result<u32> {
    setup!(coin, 0);

    let c = coin!();
    let connection = c.connect()?;
    let id = crate::db::store_account_metadata(&connection, name, icon, birth, height)?;

    Ok(id)
}

// #[frb(sync)]
pub fn store_account_seed(coin: u8, id: u32, phrase: &str, aindex: u32) -> Result<u32> {
    setup!(coin, id);

    let c = coin!();

    let seed_phrase = bip39::Mnemonic::from_str(phrase)?;
    let seed = seed_phrase.to_seed("");
    let usk =
        UnifiedSpendingKey::from_seed(&c.network, &seed, AccountId::try_from(aindex).unwrap())?;
    let uvk = usk.to_unified_full_viewing_key();
    let (ua, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
    let dindex: u32 = di.try_into()?;
    println!("dindex: {dindex}");

    crate::db::store_account_seed(&phrase, aindex)?;
    crate::db::init_account_transparent()?;
    let tsk = usk.transparent();
    crate::db::store_account_transparent_sk(tsk)?;
    let tvk = &tsk.to_account_pubkey();
    crate::db::store_account_transparent_vk(tvk)?;
    let tpk = tvk.derive_address_pubkey(
        TransparentKeyScope::EXTERNAL,
        NonHardenedChildIndex::from_index(dindex).unwrap()).unwrap();
    let taddr = ua.transparent().unwrap();
    crate::db::store_account_transparent_addr(
        0,
        dindex,
        &tpk.serialize().to_vec(),
        &taddr.encode(&c.network),
    )?;
    crate::db::init_account_sapling()?;
    let sxsk = usk.sapling();
    crate::db::store_account_sapling_sk(sxsk)?;
    let sxvk = sxsk.to_diversifiable_full_viewing_key();
    crate::db::store_account_sapling_vk(&sxvk)?;
    crate::db::init_account_orchard()?;
    let oxsk = usk.orchard();
    crate::db::store_account_orchard_sk(oxsk)?;
    let oxvk = FullViewingKey::from(oxsk);
    crate::db::store_account_orchard_vk(&oxvk)?;

    crate::db::update_dindex(dindex, true)?;

    Ok(id)
}

/*
A:
- id
- name
- seed*
- aindex
- dindex
- def_dindex
- icon
- birth
- height
- position
- hidden
- saved
T*:
- id
- xsk*
- xvk*
TA:
- id
- index
- sk*
- address
S:
- id
- xsk*
- xvk
O:
- id
- xsk*
- xvk
*/

#[frb(sync)]
pub fn set_db_filepath(coin: u8, db_filepath: String) -> Result<()> {
    let mut coins = COINS.lock().unwrap();
    let c = &mut coins[coin as usize];
    c.set_db_filepath(db_filepath)?;

    let connection = c.pool.get()?;
    create_schema(&connection)?;

    Ok(())
}

#[frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
    let _ = env_logger::builder().try_init();
}
