use std::str::FromStr as _;

use crate::{api::coin::Network, db::get_account_dindex};
use anyhow::Result;
use bech32::Hrp;
use bip32::{ChildNumber, ExtendedKeyAttrs, ExtendedPrivateKey, ExtendedPublicKey, Prefix};
use bip39::Mnemonic;
use secp256k1::{PublicKey, SecretKey};
use sqlx::SqliteConnection;
use zcash_address::unified::{Encoding as _, Fvk, Ufvk};
use zcash_keys::{
    encoding::{decode_extended_full_viewing_key, decode_extended_spending_key, AddressCodec as _},
    keys::UnifiedFullViewingKey,
};
use zcash_protocol::consensus::NetworkConstants as _;
use zcash_transparent::address::TransparentAddress;

use crate::{
    bip38,
    db::{select_account_orchard, select_account_sapling, select_account_transparent},
    pay::pool::{POOL_IRONWOOD, POOL_ORCHARD},
};

/// The key items a pool selection asks for. Ironwood and Orchard share one
/// viewing key and one unified receiver, so selecting Ironwood selects the
/// Orchard item.
fn key_pools(pools: u8) -> u8 {
    if pools & POOL_IRONWOOD != 0 {
        (pools & !POOL_IRONWOOD) | POOL_ORCHARD
    } else {
        pools
    }
}

pub async fn get_account_ufvk(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    pools: u8,
) -> Result<String> {
    let pools = key_pools(pools);
    let dindex = get_account_dindex(connection, account).await?;
    let tkeys = select_account_transparent(connection, account, dindex).await?;
    let skeys = select_account_sapling(network, connection, account).await?;
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
    let items = items.into_iter().flatten().collect::<Vec<Fvk>>();

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

    let ufvk = Ufvk::try_from_items(
        zcash_address::unified::Revision::R0,
        items
            .into_iter()
            .map(zcash_address::unified::Uitem::Data)
            .collect(),
    )?;
    let ufvk = UnifiedFullViewingKey::parse(&ufvk)?;

    Ok(ufvk.encode(network)?)
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

    if let Ok((hrp, pk)) = bech32::decode(key) {
        if hrp == Hrp::parse_unchecked("zpk") && PublicKey::from_slice(&pk).is_ok() {
            return true;
        }
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

pub fn is_valid_transparent_address(network: &Network, address: &str) -> bool {
    TransparentAddress::decode(network, address).is_ok()
}

#[cfg(test)]
mod tests {
    use sqlx::{Connection as _, SqliteConnection};

    use super::*;
    use crate::{
        account::new_account,
        api::account::NewAccount,
        db::create_schema,
        pay::pool::{ALL_POOLS, POOL_SAPLING, POOL_TRANSPARENT},
    };

    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const NETWORK: Network = Network::Main;

    /// A seed account with every pool, in a fresh in-memory wallet.
    async fn seed_account() -> (SqliteConnection, u32) {
        let mut db = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        create_schema(&mut db).await.unwrap();
        let na = NewAccount {
            icon: None,
            name: "seed".into(),
            restore: true,
            key: MNEMONIC.into(),
            passphrase: None,
            fingerprint: None,
            aindex: 0,
            birth: Some(1),
            folder: String::new(),
            pools: Some(ALL_POOLS),
            use_internal: true,
            internal: false,
            hw: 0,
        };
        let account = new_account(&NETWORK, &mut db, &na).await.unwrap();
        (db, account)
    }

    /// Ironwood shares Orchard's key: selecting it, alone or with others,
    /// exports the Orchard item, the way the GraphQL layer used to map it.
    #[tokio::test]
    async fn ironwood_selects_the_orchard_item() {
        let (mut db, account) = seed_account().await;
        let mut ufvk = async |pools| {
            get_account_ufvk(&NETWORK, &mut db, account, pools)
                .await
                .unwrap()
        };
        let orchard = ufvk(POOL_ORCHARD).await;
        assert_eq!(ufvk(POOL_IRONWOOD).await, orchard);
        assert_eq!(ufvk(POOL_ORCHARD | POOL_IRONWOOD).await, orchard);
        assert_eq!(
            ufvk(POOL_SAPLING | POOL_IRONWOOD).await,
            ufvk(POOL_SAPLING | POOL_ORCHARD).await
        );
        assert!(ufvk(POOL_TRANSPARENT).await.starts_with("xpub"));
    }
}
