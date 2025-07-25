use anyhow::Result;
use flutter_rust_bridge::frb;
use zcash_keys::{encoding::AddressCodec as _, keys::UnifiedFullViewingKey};
use zcash_primitives::legacy::TransparentAddress;

use crate::{
    get_coin,
    key::{is_valid_sapling_key, is_valid_transparent_key},
};

#[frb(sync)]
pub fn is_valid_phrase(phrase: &str) -> bool {
    crate::key::is_valid_phrase(phrase)
}

#[frb(sync)]
pub fn is_valid_fvk(fvk: &str) -> bool {
    let c = get_coin!();
    crate::key::is_valid_ufvk(&c.network, fvk)
}

#[frb(sync)]
pub fn is_valid_key(key: &str) -> bool {
    let c = get_coin!();
    let network = &c.network;

    if crate::key::is_valid_phrase(key) {
        return true;
    }

    if is_valid_transparent_key(key) {
        return true;
    }

    if is_valid_sapling_key(network, key) {
        return true;
    }

    if crate::key::is_valid_ufvk(network, key) {
        return true;
    }

    false
}

#[frb(sync)]
pub fn is_valid_address(address: &str) -> bool {
    let r = zcash_address::ZcashAddress::try_from_encoded(address);
    r.is_ok()
}

#[frb(sync)]
pub fn is_valid_transparent_address(address: &str) -> bool {
    let c = get_coin!();
    TransparentAddress::decode(&c.network, address).is_ok()
}

#[frb(sync)]
pub fn is_tex_address(address: &str) -> bool {
    let c = get_coin!();
    let Some(address) = zcash_keys::address::Address::decode(&c.network, address) else {
        return false;
    };
    let is_tex = match address {
        zcash_keys::address::Address::Tex(_) => true,
        _ => false,
    };
    is_tex
}

#[frb(sync)]
pub fn get_key_pools(key: &str) -> Result<u8> {
    let c = get_coin!();
    let network = &c.network;

    if crate::key::is_valid_phrase(key) {
        return Ok(7);
    }

    if is_valid_transparent_key(key) {
        return Ok(1);
    }

    if is_valid_sapling_key(network, key) {
        return Ok(2);
    }

    if crate::key::is_valid_ufvk(network, key) {
        let mut pools = 0;
        let ufvk = UnifiedFullViewingKey::decode(network, key).map_err(|_| anyhow::anyhow!("Invalid UFVK"))?;
        if ufvk.transparent().is_some() { pools |= 1; }
        if ufvk.sapling().is_some() { pools |= 2; }
        if ufvk.orchard().is_some() { pools |= 4; }
        return Ok(pools);
    }

    Ok(0)
}
