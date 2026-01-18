use anyhow::Result;
use rand_core::{OsRng, RngCore as _};
use zcash_keys::keys::UnifiedFullViewingKey;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

use crate::{
    api::coin::Coin, key::{is_valid_sapling_key, is_valid_transparent_key}
};

#[cfg_attr(feature = "flutter", frb(sync))]
pub fn generate_seed() -> Result<String> {
    let mut entropy = [0u8; 32];
    OsRng.fill_bytes(&mut entropy);
    let m = bip39::Mnemonic::from_entropy(&entropy)?;
    Ok(m.to_string())
}

#[cfg_attr(feature = "flutter", frb(sync))]
pub fn is_valid_phrase(phrase: &str) -> bool {
    crate::key::is_valid_phrase(phrase)
}

#[cfg_attr(feature = "flutter", frb(sync))]
pub fn is_valid_fvk(fvk: &str, c: &Coin) -> bool {
    crate::key::is_valid_ufvk(&c.network(), fvk)
}

#[cfg_attr(feature = "flutter", frb(sync))]
pub fn is_valid_key(key: &str, c: &Coin) -> bool {
    let network = &c.network();

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

    if crate::key::is_valid_transparent_address(network, key) {
        return true;
    }

    false
}

#[cfg_attr(feature = "flutter", frb(sync))]
pub fn is_valid_address(address: &str) -> bool {
    let r = zcash_address::ZcashAddress::try_from_encoded(address);
    r.is_ok()
}

#[cfg_attr(feature = "flutter", frb(sync))]
pub fn is_valid_transparent_address(address: &str, c: &Coin) -> bool {
    crate::key::is_valid_transparent_address(&c.network(), address)
}

#[cfg_attr(feature = "flutter", frb(sync))]
pub fn is_tex_address(address: &str, c: &Coin) -> bool {
    let Some(address) = zcash_keys::address::Address::decode(&c.network(), address) else {
        return false;
    };
    let is_tex = match address {
        zcash_keys::address::Address::Tex(_) => true,
        _ => false,
    };
    is_tex
}

#[cfg_attr(feature = "flutter", frb(sync))]
pub fn get_key_pools(key: &str, c: &Coin) -> Result<u8> {
    let network = &c.network();

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
