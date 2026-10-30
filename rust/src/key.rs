use std::str::FromStr as _;

use crate::{api::coin::Network, db::get_account_dindex};
use anyhow::Result;
use bech32::{primitives::decode::CheckedHrpstring, Checksum, Hrp};
use bip32::{ChildNumber, ExtendedKeyAttrs, ExtendedPrivateKey, ExtendedPublicKey, Prefix};
use bip39::Mnemonic;
use secp256k1::{PublicKey, SecretKey};
use sqlx::SqliteConnection;
use zcash_address::unified::{Encoding as _, Fvk, Ufvk};
use zcash_keys::{
    encoding::{decode_extended_full_viewing_key, decode_extended_spending_key, AddressCodec as _},
    keys::{UnifiedFullViewingKey, UnifiedIncomingViewingKey},
};
use zcash_protocol::consensus::Parameters as _;
use zcash_transparent::{
    address::TransparentAddress,
    keys::{AccountPubKey, IncomingViewingKey as _},
};

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

/// What a pool selection of an account's viewing keys comes to: a unified
/// full viewing key, or, for the transparent pool alone, the transparent
/// account key, since a Revision 0 unified key needs a shielded item.
enum AccountViewingKey {
    Transparent(AccountPubKey),
    Unified(UnifiedFullViewingKey),
}

async fn account_viewing_key(
    connection: &mut SqliteConnection,
    network: &Network,
    account: u32,
    pools: u8,
) -> Result<AccountViewingKey> {
    let pools = key_pools(pools);
    let dindex = get_account_dindex(connection, account).await?;
    let tkeys = select_account_transparent(connection, account, dindex).await?;
    let skeys = select_account_sapling(network, connection, account).await?;
    let okeys = select_account_orchard(connection, account).await?;

    let transparent = tkeys.xvk.filter(|_| pools & 1 != 0);
    let items = [
        transparent
            .as_ref()
            .map(|vk| Fvk::P2pkh(vk.serialize().try_into().unwrap())),
        skeys
            .xvk
            .filter(|_| pools & 2 != 0)
            .map(|vk| Fvk::Sapling(vk.to_bytes())),
        okeys
            .xvk
            .filter(|_| pools & 4 != 0)
            .map(|vk| Fvk::Orchard(vk.to_bytes())),
    ];
    let items = items.into_iter().flatten().collect::<Vec<Fvk>>();

    if items.is_empty() {
        return Err(anyhow::anyhow!("Viewing key has no receivers"));
    }
    if let ([Fvk::P2pkh(_)], Some(vk)) = (items.as_slice(), transparent) {
        return Ok(AccountViewingKey::Transparent(vk));
    }

    let ufvk = Ufvk::try_from_items(
        zcash_address::unified::Revision::R0,
        items
            .into_iter()
            .map(zcash_address::unified::Uitem::Data)
            .collect(),
    )?;
    Ok(AccountViewingKey::Unified(UnifiedFullViewingKey::parse(&ufvk)?))
}

/// An xpub over `data`, the 32-byte chain code followed by the 33-byte
/// public key, at `depth` and `child_number`. The parent fingerprint is
/// a dummy: the key's own derivation path is not recorded.
fn xpub(data: &[u8], depth: u8, child_number: ChildNumber) -> Result<String> {
    xpub_impl(data, depth, child_number)
}

/// [`xpub`] for tests elsewhere in the crate.
#[cfg(test)]
pub(crate) fn xpub_for_test(data: &[u8], depth: u8, child_number: ChildNumber) -> String {
    xpub_impl(data, depth, child_number).unwrap()
}

fn xpub_impl(data: &[u8], depth: u8, child_number: ChildNumber) -> Result<String> {
    let chain_code = data[..32].try_into().unwrap();
    let public_key = PublicKey::from_slice(&data[32..])?;
    let xpub = ExtendedPublicKey::new(
        public_key,
        ExtendedKeyAttrs {
            depth,
            parent_fingerprint: [0xff, 0xff, 0xff, 0xff],
            child_number,
            chain_code,
        },
    );
    Ok(xpub.to_extended_key(Prefix::XPUB).to_string())
}

/// The account's full viewing key for the selected pools: a unified full
/// viewing key, or for the transparent pool alone the account-level xpub
/// (depth 3, hardened child 0).
pub async fn get_account_ufvk(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    pools: u8,
) -> Result<String> {
    match account_viewing_key(connection, network, account, pools).await? {
        AccountViewingKey::Transparent(vk) => {
            xpub(&vk.serialize(), 3, ChildNumber::new(0, true).unwrap())
        }
        AccountViewingKey::Unified(ufvk) => Ok(ufvk.encode(network)?),
    }
}

/// The account's incoming viewing key for the selected pools: it finds
/// payments to the account's external addresses and nothing else, neither
/// spends nor internal change. A unified incoming viewing key, or for the
/// transparent pool alone the xpub of the external chain (depth 4,
/// non-hardened child 0), which the importer tells apart from an account
/// key by those two fields.
pub async fn get_account_uivk(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    pools: u8,
) -> Result<String> {
    match account_viewing_key(connection, network, account, pools).await? {
        AccountViewingKey::Transparent(vk) => {
            let external = vk.derive_external_ivk()?;
            xpub(&external.serialize(), 4, ChildNumber::new(0, false).unwrap())
        }
        AccountViewingKey::Unified(ufvk) => {
            Ok(ufvk.to_unified_incoming_viewing_key().encode(network)?)
        }
    }
}

pub fn is_valid_phrase(phrase: &str) -> bool {
    let mnemonic = Mnemonic::parse(phrase);
    mnemonic.is_ok()
}

/// Decodes a bech32 string the strict way: checksum `Ck` only, and the
/// bits left over when the data part is regrouped into bytes at most 4 and
/// all zero, as ZIP 173 (and BIP 173) require. `bech32::decode` and
/// `CheckedHrpstring::byte_iter` drop those bits unchecked, which lets one
/// payload have a second valid spelling (#1466). The padding check is the
/// crate's segwit one; the rule is the same, and nothing here is segwit.
pub fn decode_bech32<Ck: Checksum>(s: &str) -> Result<(Hrp, Vec<u8>)> {
    let checked = CheckedHrpstring::new::<Ck>(s)?;
    checked.validate_segwit_padding()?;
    Ok((checked.hrp(), checked.byte_iter().collect()))
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

    if let Ok((hrp, pk)) = decode_bech32::<bech32::Bech32>(key) {
        if hrp == Hrp::parse_unchecked("zpk") && PublicKey::from_slice(&pk).is_ok() {
            return true;
        }
    }

    false
}

pub fn is_valid_sapling_key(network: &Network, key: &str) -> bool {
    let net = network.network_type();
    if decode_extended_spending_key(
        zcash_keys::constants::hrp_sapling_extended_spending_key(net),
        key,
    )
    .is_ok()
    {
        return true;
    }

    if decode_extended_full_viewing_key(
        zcash_keys::constants::hrp_sapling_extended_full_viewing_key(net),
        key,
    )
    .is_ok()
    {
        return true;
    }

    false
}

pub fn is_valid_ufvk(network: &Network, key: &str) -> bool {
    UnifiedFullViewingKey::decode(network, key).is_ok()
}

pub fn is_valid_uivk(network: &Network, key: &str) -> bool {
    UnifiedIncomingViewingKey::decode(network, key).is_ok()
}

pub fn is_valid_transparent_address(network: &Network, address: &str) -> bool {
    TransparentAddress::decode(network, address).is_ok()
}

#[cfg(test)]
mod bech32_tests {
    use bech32::{Bech32, ByteIterExt as _, Fe32, Fe32IterExt as _};

    use super::*;

    /// A `zpk` string with its one leftover bit set is the same key in a
    /// second spelling. ZIP 173 says the leftover bits are zero; the lenient
    /// decoder takes the string, the strict one does not.
    #[test]
    fn zpk_with_a_non_zero_padding_bit_is_rejected() {
        let sk = SecretKey::from_secret_bytes([7u8; 32]).unwrap();
        let pk = sk.public_key().serialize();
        let hrp = Hrp::parse("zpk").unwrap();
        let canonical = bech32::encode::<Bech32>(hrp, &pk).unwrap();
        assert!(is_valid_transparent_key(&canonical));

        // 33 bytes are 264 bits: 53 characters, one bit left over. Set it
        // and checksum the string again.
        let mut fes: Vec<Fe32> = pk.iter().copied().bytes_to_fes().collect();
        let last = fes.pop().unwrap();
        fes.push(Fe32::try_from(u8::from(last) | 1).unwrap());
        let padded: String = fes.into_iter().with_checksum::<Bech32>(&hrp).chars().collect();
        assert_ne!(padded, canonical);
        assert_eq!(bech32::decode(&padded).unwrap().1, pk, "the lenient decoder takes it");
        assert!(!is_valid_transparent_key(&padded));
    }
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

    /// Every incoming key the account exports is the one its full key
    /// derives, item by item, carries exactly the selected pools, and is a
    /// Revision 0 `uivk`. Transparent items are compared as bytes: the
    /// library's own subsumption check compares them with their BIP 32
    /// depth and parent fingerprint, which decoding resets, so it rejects
    /// a decoded key that is byte-for-byte the derived one.
    #[tokio::test]
    async fn incoming_keys_follow_the_full_key_and_the_selection() {
        let (mut db, account) = seed_account().await;
        let full = get_account_ufvk(&NETWORK, &mut db, account, ALL_POOLS)
            .await
            .unwrap();
        let derived = UnifiedFullViewingKey::decode(&NETWORK, &full)
            .unwrap()
            .to_unified_incoming_viewing_key();
        let cases = [
            (ALL_POOLS, true, true, true),
            (POOL_SAPLING | POOL_ORCHARD, false, true, true),
            (POOL_TRANSPARENT | POOL_ORCHARD, true, false, true),
            (POOL_IRONWOOD, false, false, true),
            (POOL_SAPLING, false, true, false),
        ];
        for (pools, t, s, o) in cases {
            let encoded = get_account_uivk(&NETWORK, &mut db, account, pools)
                .await
                .unwrap();
            assert!(encoded.starts_with("uivk1"), "{pools}: {encoded}");
            let uivk = UnifiedIncomingViewingKey::decode(&NETWORK, &encoded).unwrap();
            if uivk.has_orchard() {
                assert_eq!(uivk.orchard(), derived.orchard(), "{pools}");
            }
            if uivk.has_sapling() {
                assert_eq!(uivk.sapling(), derived.sapling(), "{pools}");
            }
            if uivk.has_transparent() {
                let bytes = |k: &UnifiedIncomingViewingKey| k.p2pkh().map(|t| t.serialize());
                assert_eq!(bytes(&uivk), bytes(&derived), "{pools}");
            }
            assert_eq!(
                (uivk.has_transparent(), uivk.has_sapling(), uivk.has_orchard()),
                (t, s, o),
                "{pools}"
            );
        }
    }

    /// For the transparent pool alone the incoming key is the external
    /// chain's xpub, one level below the full key's account xpub, and the
    /// two are told apart by depth and child number.
    #[tokio::test]
    async fn transparent_alone_exports_the_external_chain() {
        let (mut db, account) = seed_account().await;
        let full = get_account_ufvk(&NETWORK, &mut db, account, POOL_TRANSPARENT)
            .await
            .unwrap();
        let incoming = get_account_uivk(&NETWORK, &mut db, account, POOL_TRANSPARENT)
            .await
            .unwrap();
        let full = ExtendedPublicKey::<PublicKey>::from_str(&full).unwrap();
        let incoming = ExtendedPublicKey::<PublicKey>::from_str(&incoming).unwrap();

        assert_eq!(full.attrs().depth, 3);
        assert_eq!(full.attrs().child_number, ChildNumber::new(0, true).unwrap());
        assert_eq!(incoming.attrs().depth, 4);
        assert_eq!(incoming.attrs().child_number, ChildNumber::new(0, false).unwrap());

        let mut account_key = full.attrs().chain_code.to_vec();
        account_key.extend_from_slice(&full.to_bytes());
        let account_key = AccountPubKey::deserialize(&account_key.try_into().unwrap()).unwrap();
        let mut external = incoming.attrs().chain_code.to_vec();
        external.extend_from_slice(&incoming.to_bytes());
        assert_eq!(external, account_key.derive_external_ivk().unwrap().serialize());
    }
}

