use std::str::FromStr as _;

use anyhow::Result;
use bip32::{ChildNumber, ExtendedKeyAttrs, ExtendedPrivateKey, ExtendedPublicKey, Prefix};
use bip39::Mnemonic;
use secp256k1::{PublicKey, SecretKey};
use sqlx::SqliteConnection;
use zcash_address::unified::{Encoding as _, Fvk, Ufvk};
use zcash_keys::{
    encoding::{decode_extended_full_viewing_key, decode_extended_spending_key},
    keys::UnifiedFullViewingKey,
};
use zcash_protocol::consensus::{Network, NetworkConstants as _};

use crate::{
    bip38,
    db::{select_account_orchard, select_account_sapling, select_account_transparent},
};

pub async fn get_account_ufvk(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    pools: u8,
) -> Result<String> {
    let tkeys = select_account_transparent(connection, account).await?;
    let skeys = select_account_sapling(connection, account).await?;
    let okeys = select_account_orchard(connection, account).await?;

    let items = vec![
        tkeys.xvk.clone().and_then(|vk| {
            if pools & 1 != 0 {
                Some(Fvk::P2pkh(vk.serialize().try_into().unwrap()))
            } else {
                None
            }
        }),
        skeys.xvk.and_then(|vk| {
            if pools & 2 != 0 {
                Some(Fvk::Sapling(vk.to_bytes()))
            } else {
                None
            }
        }),
        okeys.xvk.and_then(|vk| {
            if pools & 4 != 0 {
                Some(Fvk::Orchard(vk.to_bytes()))
            } else {
                None
            }
        }),
    ];
    let items = items.into_iter().filter_map(|x| x).collect::<Vec<Fvk>>();

    if items.is_empty() {
        return Err(anyhow::anyhow!("Viewing key has no receivers"));
    }
    if items.len() == 1 {
        if let Some(Fvk::P2pkh(data)) = items.first() {
            // special case for transparent keys since UFVK do not support them
            let chain_code = data[..32].try_into().unwrap(); // first 32 bytes is the chain code
            let public_key = PublicKey::from_slice(&data[32..])?; // next 33 bytes is the public key
            let xpub = ExtendedPublicKey::new(
                public_key,
                ExtendedKeyAttrs {
                    depth: 3,
                    // dummy values for parent fingerprint and child number
                    parent_fingerprint: [0xff, 0xff, 0xff, 0xff],
                    child_number: ChildNumber::new(0, true).unwrap(),
                    chain_code,
                },
            );
            let xpub = xpub.to_extended_key(Prefix::XPUB);
            return Ok(xpub.to_string());
        }
    }

    let ufvk = Ufvk::try_from_items(items)?;
    let ufvk = UnifiedFullViewingKey::parse(&ufvk)?;

    Ok(ufvk.encode(network))
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

    if decode_extended_full_viewing_key(network.hrp_sapling_extended_full_viewing_key(), key)
        .is_ok()
    {
        return true;
    }

    false
}

pub fn is_valid_ufvk(network: &Network, key: &str) -> bool {
    UnifiedFullViewingKey::decode(network, key).is_ok()
}
