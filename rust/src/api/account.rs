use std::str::FromStr;

use anyhow::{anyhow, Result};
use bip32::{secp256k1::sha2::{Digest as _, Sha256}, PublicKey as _};
use flutter_rust_bridge::frb;
use orchard::keys::FullViewingKey;
use zcash_keys::{
    encoding::AddressCodec,
    keys::{UnifiedAddressRequest, UnifiedFullViewingKey, UnifiedSpendingKey},
};
use zcash_primitives::{legacy::TransparentAddress, zip32::AccountId};
use zcash_protocol::consensus::{Network, NetworkConstants};
use zcash_transparent::keys::{NonHardenedChildIndex, TransparentKeyScope};

use crate::{
    bip38, coin::COINS, db::{create_schema, init_account_orchard, init_account_sapling, init_account_transparent, store_account_metadata, store_account_orchard_sk, store_account_orchard_vk, store_account_sapling_sk, store_account_sapling_vk, store_account_seed, store_account_transparent_addr, store_account_transparent_sk, store_account_transparent_vk, update_dindex}, get_coin, setup
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

#[frb(sync)]
pub fn put_account_metadata(
    coin: u8,
    name: &str,
    icon: Option<Vec<u8>>,
    birth: u32,
    height: u32,
) -> Result<u32> {
    setup!(coin, 0);

    let c = get_coin!();
    let connection = c.connect()?;
    let id = store_account_metadata(&connection, name, icon, birth, height)?;

    Ok(id)
}

#[frb(sync)]
pub fn put_account_seed(coin: u8, id: u32, phrase: &str, aindex: u32) -> Result<u32> {
    setup!(coin, id);

    let c = get_coin!();

    let seed_phrase = bip39::Mnemonic::from_str(phrase)?;
    let seed = seed_phrase.to_seed("");
    let usk =
        UnifiedSpendingKey::from_seed(&c.network, &seed, AccountId::try_from(aindex).unwrap())?;
    let uvk = usk.to_unified_full_viewing_key();
    println!("{}", uvk.encode(&c.network));
    let (ua, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
    let dindex: u32 = di.try_into()?;
    println!("dindex: {dindex}");

    store_account_seed(&phrase, aindex)?;
    init_account_transparent()?;
    let tsk = usk.transparent();
    store_account_transparent_sk(tsk)?;
    let tvk = &tsk.to_account_pubkey();
    store_account_transparent_vk(tvk)?;
    let tpk = tvk.derive_address_pubkey(
        TransparentKeyScope::EXTERNAL,
        NonHardenedChildIndex::from_index(dindex).unwrap()).unwrap();
    let taddr = ua.transparent().unwrap();
    store_account_transparent_addr(
        0,
        dindex,
        &tpk.serialize().to_vec(),
        &taddr.encode(&c.network),
    )?;
    init_account_sapling()?;
    let sxsk = usk.sapling();
    store_account_sapling_sk(sxsk)?;
    let sxvk = sxsk.to_diversifiable_full_viewing_key();
    store_account_sapling_vk(&sxvk)?;
    init_account_orchard()?;
    let oxsk = usk.orchard();
    store_account_orchard_sk(oxsk)?;
    let oxvk = FullViewingKey::from(oxsk);
    store_account_orchard_vk(&oxvk)?;

    update_dindex(dindex, true)?;

    Ok(id)
}

#[frb(sync)]
pub fn put_account_sapling_secret(coin: u8, id: u32, esk: &str) -> Result<u32> {
    setup!(coin, id);

    let c = get_coin!();
    let network = c.network;


    let xsk = zcash_keys::encoding::decode_extended_spending_key(
        network.hrp_sapling_extended_spending_key(),
        esk
    )?;
    init_account_sapling()?;
    store_account_sapling_sk(&xsk)?;
    let xvk = xsk.to_diversifiable_full_viewing_key();
    store_account_sapling_vk(&xvk)?;

    Ok(id)
}

#[frb(sync)]
pub fn put_account_sapling_viewing(coin: u8, id: u32, evk: &str) -> Result<u32> {
    setup!(coin, id);

    let c = get_coin!();
    let network = c.network;

    let xvk = zcash_keys::encoding::decode_extended_full_viewing_key(
        network.hrp_sapling_extended_full_viewing_key(),
        evk
    )?.to_diversifiable_full_viewing_key();
    init_account_sapling()?;
    store_account_sapling_vk(&xvk)?;

    Ok(id)
}

#[frb(sync)]
pub fn put_account_unified_viewing(coin: u8, id: u32, uvk: &str) -> Result<u32> {
    setup!(coin, id);

    let c = get_coin!();
    let network = c.network;

    let uvk = UnifiedFullViewingKey::decode(&network, uvk)
        .map_err(|_| anyhow!("Invalid Key"))?;
    let (ua, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
    let dindex: u32 = di.try_into()?;

    if let Some(tvk) = uvk.transparent() {
        init_account_transparent()?;
        store_account_transparent_vk(tvk)?;
        let tpk = tvk.derive_address_pubkey(
            TransparentKeyScope::EXTERNAL,
            NonHardenedChildIndex::from_index(dindex).unwrap()).unwrap();
        let address = ua.transparent().unwrap();
        store_account_transparent_addr(0, dindex, &tpk.serialize(), &address.encode(&network))?;
    }
    if let Some(svk) = uvk.sapling() {
        init_account_sapling()?;
        store_account_sapling_vk(svk)?;
    }
    if let Some(ovk) = uvk.orchard() {
        init_account_orchard()?;
        store_account_orchard_vk(ovk)?;
    }
    update_dindex(dindex, true)?;

    Ok(id)
}

pub fn put_account_transparent_secret(coin: u8, id: u32, sk: &str) -> Result<u32> {
    setup!(coin, id);

    let c = get_coin!();
    let network = c.network;

    let sk = bip38::import_tsk(sk)?;
    init_account_transparent()?;

    let pubkey = sk.public_key();
    let pubkey = pubkey.to_bytes();
    let ta = TransparentAddress::PublicKeyHash(
        *ripemd::Ripemd160::digest(Sha256::digest(&pubkey)).as_ref(),
    );
    let address = ta.encode(&network);
    store_account_transparent_addr(0, 0, &pubkey, &address)?;

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
