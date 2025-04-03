use flutter_rust_bridge::frb;

use crate::{get_coin, key::{is_valid_phrase, is_valid_sapling_key, is_valid_transparent_key, is_valid_ufvk}, setup};

#[frb(sync)]
pub fn is_valid_key(coin: u8, key: &str) -> bool {
    setup!(coin, 0);

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
