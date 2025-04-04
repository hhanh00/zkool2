use std::str::FromStr as _;

use anyhow::Result;
use bip32::{ExtendedPrivateKey, ExtendedPublicKey};
use bip39::Mnemonic;
use secp256k1::{PublicKey, SecretKey};
use zcash_address::unified::{Encoding as _, Fvk, Ufvk};
use zcash_keys::{encoding::{decode_extended_full_viewing_key, decode_extended_spending_key}, keys::UnifiedFullViewingKey};
use zcash_protocol::consensus::{Network, NetworkConstants as _};

use crate::{bip38, db::{select_account_orchard, select_account_sapling, select_account_transparent}, get_coin};

pub async fn get_account_ufvk() -> Result<UnifiedFullViewingKey> {
    let c = get_coin!();

    let tkeys = select_account_transparent(c.get_pool(), c.account).await?;
    let skeys = select_account_sapling(c.get_pool(), c.account).await?;
    let okeys = select_account_orchard(c.get_pool(), c.account).await?;

    let items = vec![
        tkeys.xvk.map(|vk| Fvk::P2pkh(vk.serialize().try_into().unwrap())),
        skeys.xvk.map(|vk| Fvk::Sapling(vk.to_bytes())),
        okeys.xvk.map(|vk| Fvk::Orchard(vk.to_bytes())),
        ];
    let items = items.into_iter().filter_map(|x| x).collect::<Vec<Fvk>>();

    let ufvk = Ufvk::try_from_items(items)?;
    let ufvk = UnifiedFullViewingKey::parse(&ufvk)?;

    Ok(ufvk)
}

pub fn is_valid_phrase(phrase: &str) -> bool {
    let mnemonic = Mnemonic::parse(phrase);
    mnemonic.is_ok()
}

pub fn is_valid_transparent_key(key: &str) -> bool {
    if bip38::import_tsk(key).is_ok() {
        return true;
    }

    if ExtendedPrivateKey::<SecretKey>::from_str(key).is_ok() {
        return true;
    }
    
    if ExtendedPublicKey::<PublicKey>::from_str(key).is_ok() {
        return true;
    }

    false
}

pub fn is_valid_sapling_key(network: &Network, key: &str) -> bool {
    if decode_extended_spending_key(network.hrp_sapling_extended_spending_key(), key).is_ok() {
        return true;
    }
    
    if decode_extended_full_viewing_key(network.hrp_sapling_extended_full_viewing_key(), key).is_ok() {
        return true;
    }

    false
}

pub fn is_valid_ufvk(network: &Network, key: &str) -> bool {
    UnifiedFullViewingKey::decode(network, key).is_ok()
}
