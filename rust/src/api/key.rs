use flutter_rust_bridge::frb;
use zcash_keys::encoding::AddressCodec as _;
use zcash_primitives::legacy::TransparentAddress;

use crate::{get_coin, key::{is_valid_phrase, is_valid_sapling_key, is_valid_transparent_key, is_valid_ufvk}};

#[frb(sync)]
pub fn is_valid_key(key: &str) -> bool {
    let c = get_coin!();
    let network = &c.network;

    if is_valid_phrase(key) {
        return true;
    }

    if is_valid_transparent_key(key) {
        return true;
    }

    if is_valid_sapling_key(network, key) {
        return true;
    }

    if is_valid_ufvk(network, key) {
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
