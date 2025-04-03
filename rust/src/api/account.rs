use std::str::FromStr;

use anyhow::{anyhow, Result};
use bip32::{ExtendedPrivateKey, ExtendedPublicKey, PublicKey as _};
use flutter_rust_bridge::frb;
use orchard::keys::FullViewingKey;
use ripemd::{Digest as _, Ripemd160};
use rusqlite::params;
use sapling_crypto::PaymentAddress;
use secp256k1::{PublicKey, SecretKey};
use sha2::Sha256;
use zcash_address::unified::{Container, Encoding};
use zcash_keys::{
    address::UnifiedAddress,
    encoding::AddressCodec,
    keys::{UnifiedAddressRequest, UnifiedFullViewingKey, UnifiedSpendingKey},
};
use zcash_primitives::{legacy::TransparentAddress, zip32::AccountId};
use zcash_protocol::consensus::{Network, NetworkConstants, Parameters};
use zcash_transparent::keys::{
    AccountPrivKey, AccountPubKey, NonHardenedChildIndex, TransparentKeyScope,
};

use crate::{
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

#[frb(sync)]
pub fn put_account_metadata(
    name: &str,
    icon: &Option<Vec<u8>>,
    birth: u32,
    height: u32,
) -> Result<u32> {
    let c = get_coin!();
    let connection = c.connect()?;
    let id = store_account_metadata(&connection, name, icon, birth, height)?;

    Ok(id)
}

#[frb(sync)]
pub fn put_account_seed(id: u32, phrase: &str, aindex: u32) -> Result<u32> {
    setup!(id);

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
    let tpk = tvk
        .derive_address_pubkey(
            TransparentKeyScope::EXTERNAL,
            NonHardenedChildIndex::from_index(dindex).unwrap(),
        )
        .unwrap();
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
pub fn put_account_sapling_secret(id: u32, esk: &str) -> Result<u32> {
    setup!(id);

    let c = get_coin!();
    let network = c.network;

    let xsk = zcash_keys::encoding::decode_extended_spending_key(
        network.hrp_sapling_extended_spending_key(),
        esk,
    )?;
    init_account_sapling()?;
    store_account_sapling_sk(&xsk)?;
    let xvk = xsk.to_diversifiable_full_viewing_key();
    store_account_sapling_vk(&xvk)?;

    Ok(id)
}

#[frb(sync)]
pub fn put_account_sapling_viewing(id: u32, evk: &str) -> Result<u32> {
    setup!(id);

    let c = get_coin!();
    let network = c.network;

    let xvk = zcash_keys::encoding::decode_extended_full_viewing_key(
        network.hrp_sapling_extended_full_viewing_key(),
        evk,
    )?
    .to_diversifiable_full_viewing_key();
    init_account_sapling()?;
    store_account_sapling_vk(&xvk)?;

    Ok(id)
}

#[frb(sync)]
pub fn put_account_unified_viewing(id: u32, uvk: &str) -> Result<u32> {
    setup!(id);

    let c = get_coin!();
    let network = c.network;

    let uvk = UnifiedFullViewingKey::decode(&network, uvk).map_err(|_| anyhow!("Invalid Key"))?;
    let (ua, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
    let dindex: u32 = di.try_into()?;

    if let Some(tvk) = uvk.transparent() {
        init_account_transparent()?;
        store_account_transparent_vk(tvk)?;
        let tpk = tvk
            .derive_address_pubkey(
                TransparentKeyScope::EXTERNAL,
                NonHardenedChildIndex::from_index(dindex).unwrap(),
            )
            .unwrap();
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

pub fn put_account_transparent_secret(id: u32, sk: &str) -> Result<u32> {
    setup!(id);
    let c = get_coin!();
    let network = c.network;

    let sk = bip38::import_tsk(sk)?;
    init_account_transparent()?;

    let secp = secp256k1::Secp256k1::new();

    let pubkey = sk.public_key(&secp);
    let pubkey = pubkey.to_bytes();
    let ta = TransparentAddress::PublicKeyHash(
        *ripemd::Ripemd160::digest(Sha256::digest(&pubkey)).as_ref(),
    );
    let address = ta.encode(&network);
    store_account_transparent_addr(0, 0, &pubkey, &address)?;

    Ok(id)
}

#[frb(sync)]
pub fn get_account_ufvk(id: u32) -> Result<String> {
    setup!(id);
    let c = get_coin!();
    let network = c.network;

    let ufvk = crate::key::get_account_ufvk()?;
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

pub fn list_accounts() -> Result<Vec<Account>> {
    let accounts = crate::db::list_accounts()?;

    Ok(accounts)
}

#[frb(sync)]
pub fn update_account(update: &AccountUpdate) -> Result<()> {
    let id = update.id;
    setup!(id);

    let c = get_coin!();
    let connection = c.connect()?;

    if let Some(ref name) = update.name {
        connection.execute(
            "UPDATE accounts SET name = ? WHERE id_account = ?",
            params![name, id],
        )?;
    }
    if let Some(ref icon) = update.icon {
        connection.execute(
            "UPDATE accounts SET icon = ? WHERE id_account = ?",
            params![icon, id],
        )?;
    }
    if let Some(ref birth) = update.birth {
        connection.execute(
            "UPDATE accounts SET birth = ? WHERE id_account = ?",
            params![birth, id],
        )?;
    }

    Ok(())
}

#[frb(sync)]
pub fn delete_account(account: &Account) -> Result<()> {
    setup!(account.id);

    crate::db::delete_account()?;

    Ok(())
}

pub fn reorder_account(old_position: u32, new_position: u32) -> Result<()> {
    crate::db::reorder_account(old_position, new_position)
}

// #[frb(sync)]
pub fn new_account(na: &NewAccount) -> Result<()> {
    let c = get_coin!();
    let connection = c.connect()?;
    let network = c.network;
    let min_height: u32 = network
        .activation_height(zcash_protocol::consensus::NetworkUpgrade::Sapling)
        .unwrap()
        .into();

    let birth = na.birth.unwrap_or(min_height);

    store_account_metadata(&connection, &na.name, &na.icon, birth, birth)?;
    if is_valid_phrase(&na.key) {
        store_account_seed(&na.key, na.aindex)?;

        let seed_phrase = bip39::Mnemonic::from_str(&na.key)?;
        let seed = seed_phrase.to_seed("");
            let usk =
            UnifiedSpendingKey::from_seed(&c.network, &seed, AccountId::try_from(na.aindex).unwrap())?;
        let uvk = usk.to_unified_full_viewing_key();
        let (_, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
        let dindex: u32 = di.try_into()?;

        init_account_transparent()?;
        let tsk = usk.transparent();
        store_account_transparent_sk(tsk)?;
        let tvk = &tsk.to_account_pubkey();
        store_account_transparent_vk(tvk)?;
        let (tpk, taddr) = derive_transparent_address(tvk, 0, dindex)?;
        store_account_transparent_addr(
            0,
            dindex,
            &tpk,
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
    }
    if is_valid_transparent_key(&na.key) {
        init_account_transparent()?;
        if let Ok(xsk) = ExtendedPrivateKey::<SecretKey>::from_str(&na.key) {
            let xsk = AccountPrivKey::from_extended_privkey(xsk);
            store_account_transparent_sk(&xsk)?;
            let xvk = xsk.to_account_pubkey();
            store_account_transparent_vk(&xvk)?;
            let (pkh, address) = derive_transparent_address(&xvk, 0, 0)?;
            store_account_transparent_addr(0, 0, &pkh, &address.encode(&network))?;
        }
        if let Ok(xvk) = ExtendedPublicKey::<PublicKey>::from_str(&na.key) {
            // No AccountPubKey::from_extended_pubkey, we need to use the bytes
            let mut buf = xvk.attrs().chain_code.to_vec();
            buf.extend_from_slice(&xvk.to_bytes());
            let xvk = AccountPubKey::deserialize(&buf.try_into().unwrap()).unwrap();
            store_account_transparent_vk(&xvk)?;
            let (pkh, address) = derive_transparent_address(&xvk, 0, 0)?;
            store_account_transparent_addr(0, 0, &pkh, &address.encode(&network))?;
        }
        if let Ok(sk) = bip38::import_tsk(&na.key) {
            let secp = secp256k1::Secp256k1::new();
            let tpk = sk.public_key(&secp).serialize();
            let pkh: [u8; 20] = Ripemd160::digest(&Sha256::digest(&tpk)).into();
            let addr = TransparentAddress::PublicKeyHash(pkh.clone());
            store_account_transparent_addr(0, 0, &tpk, &addr.encode(&network))?;
        }
    }
    if is_valid_sapling_key(&network, &na.key) {
        init_account_sapling()?;
        if let Ok(xsk) = zcash_keys::encoding::decode_extended_spending_key(
            network.hrp_sapling_extended_spending_key(),
            &na.key,
        ) {
            store_account_sapling_sk(&xsk)?;
            let xvk = xsk.to_diversifiable_full_viewing_key();
            store_account_sapling_vk(&xvk)?;
        }
        if let Ok(xvk) = zcash_keys::encoding::decode_extended_full_viewing_key(
            network.hrp_sapling_extended_full_viewing_key(),
            &na.key,
        ) {
            store_account_sapling_vk(&xvk.to_diversifiable_full_viewing_key())?;
        }
    }
    if is_valid_ufvk(&network, &na.key) {
        let uvk =
            UnifiedFullViewingKey::decode(&network, &na.key).map_err(|_| anyhow!("Invalid Key"))?;
        let (_, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
        let dindex: u32 = di.try_into()?;

        if let Some(tvk) = uvk.transparent() {
            init_account_transparent()?;
            store_account_transparent_vk(tvk)?;
            let (tpk, address) = derive_transparent_address(tvk, 0, dindex)?;
            store_account_transparent_addr(0, dindex, &tpk, &address.encode(&network))?;
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
    pub height: u32,
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

pub fn derive_transparent_address(
    tvk: &AccountPubKey,
    scope: u32,
    dindex: u32,
) -> Result<([u8; 33], TransparentAddress)> {
    let sindex = TransparentKeyScope::custom(scope).unwrap();
    let tpk = tvk
        .derive_address_pubkey(sindex, NonHardenedChildIndex::from_index(dindex).unwrap())
        .unwrap()
        .serialize();
    let pkh: [u8; 20] = Ripemd160::digest(&Sha256::digest(&tpk)).into();
    let addr = TransparentAddress::PublicKeyHash(pkh);
    Ok((tpk, addr))
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

#[frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
    let _ = env_logger::builder().try_init();
}
