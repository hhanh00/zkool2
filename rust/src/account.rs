use std::collections::HashMap;
use std::str::FromStr as _;

use crate::{
    api::{
        account::{get_ledger, Addresses, TAddressTxCount},
        coin::Network,
    },
    bip38,
    db::{
        init_account_orchard, init_account_sapling, init_account_transparent,
        store_account_orchard_sk, store_account_orchard_vk, store_account_sapling_sk,
        store_account_sapling_vk, store_account_seed, store_account_transparent_addr,
        store_account_transparent_sk, store_account_transparent_vk, update_dindex,
    },
    key::{is_valid_phrase, is_valid_sapling_key, is_valid_transparent_key, is_valid_ufvk},
    keys::{sapling_dfvk_to_fvk, SaplingAddressDerivation, ScopeExt},
    ledger::{HwKind, LedgerHandle},
    pay::pool::PoolMask,
    tiu,
};
use crate::{
    api::{
        account::{
            Category, Folder, NewAccount, Seed, TxAccount, TxMemo, TxOutput, TxSpend,
        },
        key::generate_seed,
    },
    db::{
        get_account_dindex, get_account_hw, store_account_hw,
        store_account_metadata,
    },
    pay::pool::{ALL_POOLS, POOL_IRONWOOD, POOL_ORCHARD, POOL_SAPLING, POOL_TRANSPARENT},
};
use secp256k1::{PublicKey, SecretKey};
use zcash_keys::keys::{sapling::ExtendedSpendingKey, UnifiedFullViewingKey, UnifiedSpendingKey};
use zcash_transparent::address::TransparentAddress;

use anyhow::{anyhow, Context, Result};
use bincode::config::legacy;
use bip32::{ExtendedPrivateKey, ExtendedPublicKey, PrivateKey};
use jubjub::Fr;
use orchard::{
    keys::FullViewingKey,
    note::{AssetBase, NoteVersion, RandomSeed, Rho},
    tree::MerkleHashOrchard,
    value::NoteValue,
    Note,
};
use ripemd::{Digest as _, Ripemd160};
use sapling_crypto::{
    zip32::{sapling_derive_internal_fvk, DiversifiableFullViewingKey},
    PaymentAddress,
};
use sha2::Sha256;
use sqlx::{sqlite::SqliteRow, Connection, Row, SqliteConnection};
use zcash_keys::{
    address::UnifiedAddress, encoding::AddressCodec as _, keys::UnifiedAddressRequest,
};
use zcash_protocol::consensus::{NetworkConstants, Parameters};
use zcash_transparent::keys::{
    AccountPrivKey, AccountPubKey, NonHardenedChildIndex, TransparentKeyScope,
};
use zcash_trees::warp::FragmentAuthPath;
use zip32::{fingerprint::SeedFingerprint, AccountId};

use crate::{
    api::account::FrostParams,
    sync::chain_log,
    transparent_keys::{ensure_transparent_address, DerivedTransparentAddr, TransparentKeySource},
    warp::{AuthPath, Witness, MERKLE_DEPTH},
};

pub async fn new_account(
    network: &Network,
    connection: &mut SqliteConnection,
    na: &NewAccount,
) -> Result<u32> {
    let mut db_tx = connection.begin().await?;

    let birth = na.birth.unwrap_or_else(|| {
        network
            .activation_height(zcash_protocol::consensus::NetworkUpgrade::Sapling)
            .unwrap()
            .into()
    });

    let ledger_kind = HwKind::from_hw(na.hw);
    // Official Ledger accounts always use an internal change address, so the
    // transparent change output can carry the derivation path the Ledger app
    // requires to verify it as change.
    let use_internal = if ledger_kind == HwKind::Official {
        true
    } else {
        na.use_internal
    };

    let account = store_account_metadata(
        &mut db_tx,
        &na.name,
        &na.icon,
        &na.fingerprint,
        birth,
        use_internal,
        na.internal,
    )
    .await?;

    let mut key = na.key.clone();
    if key.is_empty() && !ledger_kind.is_ledger() {
        key = generate_seed()?;
    }

    let pools = resolve_pools(ledger_kind, na.pools)?;
    let source = classify_key(network, ledger_kind, &key)?;
    let passphrase = na.passphrase.clone().unwrap_or_default();

    let keys = match source {
        KeySource::ZondaxSeed(mnemonic) => {
            derive_zondax_seed_keys(network, &mnemonic, na.aindex, pools).await?
        }
        KeySource::ZondaxDevice => {
            store_account_hw(&mut db_tx, account, HwKind::Zondax as u8, na.aindex).await?;
            let ledger = get_ledger(&mut db_tx, account).await?;
            derive_zondax_device_keys(network, &ledger, na.aindex, pools).await?
        }
        KeySource::OfficialDevice => {
            store_account_hw(&mut db_tx, account, HwKind::Official as u8, na.aindex).await?;
            let ledger = get_ledger(&mut db_tx, account).await?;
            derive_official_device_keys(network, &ledger, na.aindex, pools).await?
        }
        source => derive_account_keys(network, source, &passphrase, na.aindex, pools).await?,
    };

    store_account_keys(network, &mut db_tx, account, birth, keys).await?;

    db_tx.commit().await?;
    Ok(account)
}

/// Resolve the pools a new account supports, applying the per-wallet-kind
/// default and the Ledger app restrictions.
fn resolve_pools(hw: HwKind, requested: Option<u8>) -> Result<u8> {
    let pools = requested.unwrap_or_else(|| default_pools(hw));
    if pools == 0 {
        anyhow::bail!("an account must support at least one pool");
    }
    match hw {
        HwKind::Official => {
            if pools & !(POOL_TRANSPARENT | POOL_IRONWOOD) != 0 {
                anyhow::bail!("Official Ledger accounts support transparent and ironwood pools only");
            }
        }
        HwKind::Zondax => {
            if pools & !(POOL_TRANSPARENT | POOL_SAPLING) != 0 {
                anyhow::bail!("Zondax Ledger accounts support transparent and sapling pools only");
            }
        }
        HwKind::Software => {}
    }
    Ok(pools)
}

fn default_pools(hw: HwKind) -> u8 {
    match hw {
        HwKind::Zondax => POOL_TRANSPARENT | POOL_SAPLING,
        HwKind::Official => POOL_TRANSPARENT | POOL_IRONWOOD,
        HwKind::Software => ALL_POOLS,
    }
}

/// What a new account's key material is, decoded from the user supplied key
/// string. Classification decides the derivation path; it does not touch the
/// database or any device.
enum KeySource {
    /// BIP-39 seed phrase; the account owns every pool it supports.
    Seed(String),
    /// Transparent extended private key (xprv).
    TransparentSk(AccountPrivKey),
    /// Transparent extended public key (xpub).
    TransparentVk(AccountPubKey),
    /// Single transparent key: a BIP-38 secret key, or a bare `zpk` public key.
    TransparentSingle {
        sk: Option<Vec<u8>>,
        pk: Vec<u8>,
        uncompressed: bool,
    },
    SaplingSk(Box<ExtendedSpendingKey>),
    SaplingVk(Box<sapling_crypto::zip32::ExtendedFullViewingKey>),
    Ufvk(Box<UnifiedFullViewingKey>),
    /// Zondax Ledger account recovered from its seed phrase (no device).
    ZondaxSeed(String),
    /// Zondax Ledger account whose keys are read from the device.
    ZondaxDevice,
    /// Official Ledger account whose viewing key is read from the device.
    OfficialDevice,
}

fn classify_key(network: &Network, hw: HwKind, key: &str) -> Result<KeySource> {
    match hw {
        HwKind::Zondax => {
            return Ok(if key.is_empty() {
                KeySource::ZondaxDevice
            } else {
                KeySource::ZondaxSeed(key.to_string())
            });
        }
        HwKind::Official => {
            if key.is_empty() {
                return Ok(KeySource::OfficialDevice);
            }
            if !is_valid_phrase(key) {
                anyhow::bail!(
                    "Official Ledger accounts accept a seed phrase or no key to import the viewing key from the device"
                );
            }
        }
        HwKind::Software => {}
    }

    if is_valid_phrase(key) {
        Ok(KeySource::Seed(key.to_string()))
    } else if is_valid_transparent_key(key) {
        if let Ok(xsk) = ExtendedPrivateKey::<SecretKey>::from_str(key) {
            Ok(KeySource::TransparentSk(AccountPrivKey::from_extended_privkey(xsk)))
        } else if let Ok(xvk) = ExtendedPublicKey::<PublicKey>::from_str(key) {
            // No AccountPubKey::from_extended_pubkey, we need to use the bytes
            let mut buf = xvk.attrs().chain_code.to_vec();
            buf.extend_from_slice(&xvk.to_bytes());
            let xvk = AccountPubKey::deserialize(&buf.try_into().unwrap()).unwrap();
            Ok(KeySource::TransparentVk(xvk))
        } else if let Ok(sk) = bip38::import_tsk(key) {
            let pk = sk.0.public_key();
            let tpk = if sk.1 {
                pk.serialize_uncompressed().to_vec()
            } else {
                pk.serialize().to_vec()
            };
            Ok(KeySource::TransparentSingle {
                sk: Some(sk.0.to_bytes().to_vec()),
                pk: tpk,
                uncompressed: sk.1,
            })
        } else if let Ok((_hrp, tpk)) = bech32::decode(key) {
            Ok(KeySource::TransparentSingle {
                sk: None,
                pk: tpk,
                uncompressed: false,
            })
        } else {
            Err(anyhow!("Unsupported transparent key"))
        }
    } else if is_valid_sapling_key(network, key) {
        if let Ok(xsk) = zcash_keys::encoding::decode_extended_spending_key(
            network.hrp_sapling_extended_spending_key(),
            key,
        ) {
            Ok(KeySource::SaplingSk(Box::new(xsk)))
        } else if let Ok(xvk) = zcash_keys::encoding::decode_extended_full_viewing_key(
            network.hrp_sapling_extended_full_viewing_key(),
            key,
        ) {
            Ok(KeySource::SaplingVk(Box::new(xvk)))
        } else {
            Err(anyhow!("Invalid Sapling Key"))
        }
    } else if is_valid_ufvk(network, key) {
        let uvk =
            UnifiedFullViewingKey::decode(network, key).map_err(|_| anyhow!("Invalid Key"))?;
        Ok(KeySource::Ufvk(Box::new(uvk)))
    } else {
        Err(anyhow!("Unsupported key"))
    }
}

/// Everything `store_account_keys` writes for a new account: the per-pool key
/// material the account has, the transparent addresses to register, and the
/// default diversifier index.
#[derive(Default)]
struct AccountKeys {
    seed: Option<SeedKeys>,
    transparent: Option<TransparentKeys>,
    sapling: Option<SaplingKeys>,
    orchard: Option<OrchardKeys>,
    dindex: u32,
}

struct SeedKeys {
    mnemonic: String,
    passphrase: String,
    fingerprint: [u8; 32],
    aindex: u32,
}

#[derive(Default)]
struct TransparentKeys {
    sk: Option<AccountPrivKey>,
    vk: Option<AccountPubKey>,
    /// External (scope 0) addresses to register.
    addresses: Vec<TransparentAddr>,
}

struct TransparentAddr {
    dindex: u32,
    sk: Option<Vec<u8>>,
    pk: Vec<u8>,
    address: String,
    uncompressed: bool,
}

struct SaplingKeys {
    sk: Option<ExtendedSpendingKey>,
    vk: DiversifiableFullViewingKey,
    address: String,
}

struct OrchardKeys {
    sk: Option<orchard::keys::SpendingKey>,
    vk: FullViewingKey,
}

async fn transparent_addr(
    network: &Network,
    source: &TransparentKeySource,
    dindex: u32,
) -> Result<TransparentAddr> {
    let derived = source.derive(network, 0, dindex).await?;
    Ok(TransparentAddr {
        dindex,
        sk: derived.sk,
        pk: derived.pk,
        address: derived.address,
        uncompressed: false,
    })
}

/// The external transparent addresses a new account registers: index 0 and
/// the account's default diversifier index (once, when they coincide).
async fn external_transparent_addrs(
    network: &Network,
    tvk: &AccountPubKey,
    tsk: Option<&AccountPrivKey>,
    dindex: u32,
) -> Result<Vec<TransparentAddr>> {
    let source = TransparentKeySource::software(tvk.clone(), tsk.cloned());
    let mut addresses = vec![transparent_addr(network, &source, 0).await?];
    if dindex != 0 {
        addresses.push(transparent_addr(network, &source, dindex).await?);
    }
    Ok(addresses)
}

/// Derive the keys of a software account from its classified key source.
/// No database, no device: the only `await` is the software key source.
async fn derive_account_keys(
    network: &Network,
    source: KeySource,
    passphrase: &str,
    aindex: u32,
    pools: u8,
) -> Result<AccountKeys> {
    let mut keys = AccountKeys::default();
    match source {
        KeySource::Seed(mnemonic) => {
            let seed_phrase = bip39::Mnemonic::from_str(&mnemonic)?;
            let seed = seed_phrase.to_seed(passphrase);
            let fingerprint = SeedFingerprint::from_seed(&seed).unwrap().to_bytes();
            keys.seed = Some(SeedKeys {
                mnemonic,
                passphrase: passphrase.to_string(),
                fingerprint,
                aindex,
            });
            let usk =
                UnifiedSpendingKey::from_seed(network, &seed, AccountId::try_from(aindex).unwrap())?;
            let uvk = usk.to_unified_full_viewing_key();
            let (_, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
            let dindex: u32 = di.try_into()?;
            keys.dindex = dindex;

            if pools & POOL_TRANSPARENT != 0 {
                let tsk = usk.transparent();
                let tvk = tsk.to_account_pubkey();
                let addresses =
                    external_transparent_addrs(network, &tvk, Some(tsk), dindex).await?;
                keys.transparent = Some(TransparentKeys {
                    sk: Some(tsk.clone()),
                    vk: Some(tvk),
                    addresses,
                });
            }
            if pools & POOL_SAPLING != 0 {
                let sxsk = usk.sapling();
                let sxvk = sxsk.to_diversifiable_full_viewing_key();
                let address = derive_sapling_address(network, &sxvk, dindex);
                keys.sapling = Some(SaplingKeys {
                    sk: Some(sxsk.clone()),
                    vk: sxvk,
                    address,
                });
            }
            if pools & (POOL_ORCHARD | POOL_IRONWOOD) != 0 {
                let oxsk = usk.orchard();
                keys.orchard = Some(OrchardKeys {
                    sk: Some(oxsk.clone()),
                    vk: FullViewingKey::from(oxsk),
                });
            }
        }
        KeySource::TransparentSk(xsk) => {
            let xvk = xsk.to_account_pubkey();
            let addresses = external_transparent_addrs(network, &xvk, Some(&xsk), 0).await?;
            keys.transparent = Some(TransparentKeys {
                sk: Some(xsk),
                vk: Some(xvk),
                addresses,
            });
        }
        KeySource::TransparentVk(xvk) => {
            let addresses = external_transparent_addrs(network, &xvk, None, 0).await?;
            keys.transparent = Some(TransparentKeys {
                sk: None,
                vk: Some(xvk),
                addresses,
            });
        }
        KeySource::TransparentSingle {
            sk,
            pk,
            uncompressed,
        } => {
            let pkh: [u8; 20] = Ripemd160::digest(Sha256::digest(&pk)).into();
            let address = TransparentAddress::PublicKeyHash(pkh).encode(network);
            keys.transparent = Some(TransparentKeys {
                sk: None,
                vk: None,
                addresses: vec![TransparentAddr {
                    dindex: 0,
                    sk,
                    pk,
                    address,
                    uncompressed,
                }],
            });
        }
        KeySource::SaplingSk(xsk) => {
            let xvk = xsk.to_diversifiable_full_viewing_key();
            let (di, address) = xvk.default_address();
            keys.dindex = di.try_into()?;
            keys.sapling = Some(SaplingKeys {
                sk: Some(*xsk),
                vk: xvk,
                address: address.encode(network),
            });
        }
        KeySource::SaplingVk(xvk) => {
            let (di, address) = xvk.default_address();
            keys.dindex = di.try_into()?;
            keys.sapling = Some(SaplingKeys {
                sk: None,
                vk: xvk.to_diversifiable_full_viewing_key(),
                address: address.encode(network),
            });
        }
        KeySource::Ufvk(uvk) => {
            let (ua, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
            let dindex: u32 = di.try_into()?;
            keys.dindex = dindex;
            if let Some(tvk) = uvk.p2pkh().filter(|_| pools & POOL_TRANSPARENT != 0) {
                let addresses = external_transparent_addrs(network, tvk, None, dindex).await?;
                keys.transparent = Some(TransparentKeys {
                    sk: None,
                    vk: Some(tvk.clone()),
                    addresses,
                });
            }
            if let Some(sxvk) = uvk.sapling().filter(|_| pools & POOL_SAPLING != 0) {
                let address = ua.sapling().unwrap().encode(network);
                keys.sapling = Some(SaplingKeys {
                    sk: None,
                    vk: sxvk.clone(),
                    address,
                });
            }
            if let Some(ovk) = uvk.orchard().filter(|_| pools & POOL_ORCHARD != 0) {
                keys.orchard = Some(OrchardKeys {
                    sk: None,
                    vk: ovk.clone(),
                });
            }
        }
        KeySource::ZondaxSeed(_) | KeySource::ZondaxDevice | KeySource::OfficialDevice => {
            anyhow::bail!("Ledger accounts are derived with the device helpers");
        }
    }
    Ok(keys)
}

/// Zondax Ledger account recovered from its seed phrase: only the sapling
/// spending key is recoverable, there is no device and no transparent key.
async fn derive_zondax_seed_keys(
    network: &Network,
    mnemonic: &str,
    aindex: u32,
    pools: u8,
) -> Result<AccountKeys> {
    let mut keys = AccountKeys::default();
    if pools & POOL_SAPLING != 0 {
        let sxsk = crate::recover::recover_ledger_seed(mnemonic, aindex).await?;
        let sxvk = sxsk.to_diversifiable_full_viewing_key();
        let (di, _) = sxvk.default_address();
        let di: u128 = di.into();
        keys.dindex = di as u32;
        let address = derive_sapling_address(network, &sxvk, keys.dindex);
        keys.sapling = Some(SaplingKeys {
            sk: Some(sxsk),
            vk: sxvk,
            address,
        });
    }
    Ok(keys)
}

/// Zondax Ledger account: the sapling viewing key and the transparent address
/// are read from the device.
async fn derive_zondax_device_keys(
    network: &Network,
    ledger: &LedgerHandle,
    aindex: u32,
    pools: u8,
) -> Result<AccountKeys> {
    let mut keys = AccountKeys::default();
    // we must do sapling derivation first to know a valid dindex
    // because in sapling some indices are invalid
    if pools & POOL_SAPLING != 0 {
        let fvk = ledger.import_sapling_fvk(network, aindex).await?;
        let mut dfvk = fvk.to_bytes().to_vec();
        dfvk.extend_from_slice(&[0u8; 32]); // add a dummy dk because we cannot get the one from the Ledger
        let xvk = DiversifiableFullViewingKey::from_bytes(&tiu!(dfvk)).unwrap();
        // We should get the default address dindex by using the get_div_list
        // api but it is currently not working
        // instead, we "assume" the dindex = 0 is the default sapling address
        let address = ledger.get_sapling_address(network, aindex).await?;
        keys.sapling = Some(SaplingKeys {
            sk: None,
            vk: xvk,
            address,
        });
    }
    if pools & POOL_TRANSPARENT != 0 {
        // Zondax accounts only ever register the transparent address at index
        // 0, regardless of the sapling default diversifier index.
        let source = TransparentKeySource::Ledger {
            handle: LedgerHandle::from_hw(HwKind::Zondax as u8),
            aindex,
        };
        keys.transparent = Some(TransparentKeys {
            addresses: vec![transparent_addr(network, &source, 0).await?],
            ..Default::default()
        });
    }
    Ok(keys)
}

/// Official Ledger account: the unified viewing key is read from the device.
async fn derive_official_device_keys(
    network: &Network,
    ledger: &LedgerHandle,
    aindex: u32,
    pools: u8,
) -> Result<AccountKeys> {
    let ufvk = ledger.get_ufvk(network, aindex).await?;
    let uvk = UnifiedFullViewingKey::decode(network, &ufvk)
        .map_err(|_| anyhow!("Invalid viewing key from the device"))?;

    // no sapling key on the device, so there is no derived default
    // diversifier index; use 0
    let mut keys = AccountKeys::default();
    if pools & POOL_TRANSPARENT != 0 {
        let tvk = uvk
            .p2pkh()
            .ok_or_else(|| anyhow!("device viewing key has no transparent key"))?;
        let addresses = external_transparent_addrs(network, tvk, None, keys.dindex).await?;
        keys.transparent = Some(TransparentKeys {
            sk: None,
            vk: Some(tvk.clone()),
            addresses,
        });
    }
    if pools & (POOL_ORCHARD | POOL_IRONWOOD) != 0 {
        let ovk = uvk
            .orchard()
            .ok_or_else(|| anyhow!("device viewing key has no orchard key"))?;
        keys.orchard = Some(OrchardKeys {
            sk: None,
            vk: ovk.clone(),
        });
    }
    Ok(keys)
}

/// Write a new account's derived keys. The account metadata row must exist.
async fn store_account_keys(
    network: &Network,
    db_tx: &mut SqliteConnection,
    account: u32,
    birth: u32,
    keys: AccountKeys,
) -> Result<()> {
    if let Some(seed) = &keys.seed {
        store_account_seed(
            db_tx,
            account,
            &seed.mnemonic,
            &seed.passphrase,
            &seed.fingerprint,
            seed.aindex,
        )
        .await?;
    }

    if let Some(t) = &keys.transparent {
        init_account_transparent(db_tx, account, birth).await?;
        if let Some(sk) = &t.sk {
            store_account_transparent_sk(db_tx, account, sk).await?;
        }
        if let Some(vk) = &t.vk {
            store_account_transparent_vk(db_tx, account, vk).await?;
        }
        for a in &t.addresses {
            store_account_transparent_addr(
                db_tx,
                account,
                0,
                a.dindex,
                a.sk.clone(),
                &a.pk,
                &a.address,
                a.uncompressed,
            )
            .await?;
        }
    }

    if let Some(s) = &keys.sapling {
        init_account_sapling(network, db_tx, account, birth).await?;
        if let Some(sk) = &s.sk {
            store_account_sapling_sk(db_tx, account, sk).await?;
        }
        store_account_sapling_vk(db_tx, account, &s.vk, &s.address).await?;
    }

    if let Some(o) = &keys.orchard {
        init_account_orchard(network, db_tx, account, birth).await?;
        if let Some(sk) = &o.sk {
            store_account_orchard_sk(db_tx, account, sk).await?;
        }
        store_account_orchard_vk(db_tx, account, &o.vk).await?;
    }

    update_dindex(db_tx, account, keys.dindex, true).await?;

    if let Some(t) = &keys.transparent {
        for a in &t.addresses {
            ensure_internal_change_address(network, db_tx, account, a.dindex).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod new_account_tests {
    use super::*;
    use crate::pay::pool::{ALL_POOLS, POOL_IRONWOOD, POOL_ORCHARD, POOL_SAPLING, POOL_TRANSPARENT};
    use bip32::Prefix;

    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const NETWORK: Network = Network::Main;

    async fn seed_keys(pools: u8) -> AccountKeys {
        derive_account_keys(&NETWORK, KeySource::Seed(MNEMONIC.to_string()), "", 0, pools)
            .await
            .unwrap()
    }

    fn usk() -> UnifiedSpendingKey {
        let seed = bip39::Mnemonic::from_str(MNEMONIC).unwrap().to_seed("");
        UnifiedSpendingKey::from_seed(&NETWORK, &seed, AccountId::ZERO).unwrap()
    }

    #[test]
    fn account_pool_defaults_match_wallet_kind() {
        assert_eq!(resolve_pools(HwKind::Software, None).unwrap(), ALL_POOLS);
        assert_eq!(
            resolve_pools(HwKind::Zondax, None).unwrap(),
            POOL_TRANSPARENT | POOL_SAPLING
        );
        assert_eq!(
            resolve_pools(HwKind::Official, None).unwrap(),
            POOL_TRANSPARENT | POOL_IRONWOOD
        );
    }

    #[test]
    fn resolve_pools_rejects_unsupported_requests() {
        assert!(resolve_pools(HwKind::Software, Some(0)).is_err());
        assert!(resolve_pools(HwKind::Zondax, Some(POOL_ORCHARD)).is_err());
        assert!(resolve_pools(HwKind::Official, Some(POOL_SAPLING)).is_err());
        assert_eq!(
            resolve_pools(HwKind::Software, Some(POOL_SAPLING)).unwrap(),
            POOL_SAPLING
        );
        assert_eq!(
            resolve_pools(HwKind::Official, Some(POOL_TRANSPARENT)).unwrap(),
            POOL_TRANSPARENT
        );
    }

    #[test]
    fn classify_ledger_keys() {
        assert!(matches!(
            classify_key(&NETWORK, HwKind::Zondax, "").unwrap(),
            KeySource::ZondaxDevice
        ));
        assert!(matches!(
            classify_key(&NETWORK, HwKind::Zondax, MNEMONIC).unwrap(),
            KeySource::ZondaxSeed(_)
        ));
        assert!(matches!(
            classify_key(&NETWORK, HwKind::Official, "").unwrap(),
            KeySource::OfficialDevice
        ));
        // An Official account restored from its phrase is a software seed account.
        assert!(matches!(
            classify_key(&NETWORK, HwKind::Official, MNEMONIC).unwrap(),
            KeySource::Seed(_)
        ));
        assert!(classify_key(&NETWORK, HwKind::Official, "not a phrase").is_err());
    }

    #[test]
    fn classify_software_keys() {
        assert!(matches!(
            classify_key(&NETWORK, HwKind::Software, MNEMONIC).unwrap(),
            KeySource::Seed(_)
        ));

        let seed = bip39::Mnemonic::from_str(MNEMONIC).unwrap().to_seed("");
        let xprv = ExtendedPrivateKey::<SecretKey>::new(&seed).unwrap();
        let xprv_str = xprv.to_extended_key(Prefix::XPRV).to_string();
        let xpub_str = xprv.public_key().to_extended_key(Prefix::XPUB).to_string();
        assert!(matches!(
            classify_key(&NETWORK, HwKind::Software, &xprv_str).unwrap(),
            KeySource::TransparentSk(_)
        ));
        assert!(matches!(
            classify_key(&NETWORK, HwKind::Software, &xpub_str).unwrap(),
            KeySource::TransparentVk(_)
        ));

        let usk = usk();
        let sxsk = usk.sapling();
        let sxsk_str = zcash_keys::encoding::encode_extended_spending_key(
            NETWORK.hrp_sapling_extended_spending_key(),
            sxsk,
        );
        let sxvk_str = zcash_keys::encoding::encode_extended_full_viewing_key(
            NETWORK.hrp_sapling_extended_full_viewing_key(),
            &sxsk.to_extended_full_viewing_key(),
        );
        assert!(matches!(
            classify_key(&NETWORK, HwKind::Software, &sxsk_str).unwrap(),
            KeySource::SaplingSk(_)
        ));
        assert!(matches!(
            classify_key(&NETWORK, HwKind::Software, &sxvk_str).unwrap(),
            KeySource::SaplingVk(_)
        ));

        let ufvk = usk.to_unified_full_viewing_key().encode(&NETWORK).unwrap();
        assert!(matches!(
            classify_key(&NETWORK, HwKind::Software, &ufvk).unwrap(),
            KeySource::Ufvk(_)
        ));

        assert!(classify_key(&NETWORK, HwKind::Software, "garbage").is_err());
    }

    #[tokio::test]
    async fn seed_derivation_covers_requested_pools() {
        let keys = seed_keys(ALL_POOLS).await;
        let seed = keys.seed.as_ref().unwrap();
        assert_eq!(seed.mnemonic, MNEMONIC);
        assert_eq!(seed.aindex, 0);

        let t = keys.transparent.as_ref().unwrap();
        assert!(t.sk.is_some() && t.vk.is_some());
        assert_eq!(t.addresses[0].dindex, 0);
        assert_eq!(t.addresses.last().unwrap().dindex, keys.dindex);
        assert!(t.addresses.iter().all(|a| a.sk.is_some()));
        let distinct: std::collections::HashSet<u32> =
            t.addresses.iter().map(|a| a.dindex).collect();
        assert_eq!(distinct.len(), t.addresses.len());

        let s = keys.sapling.as_ref().unwrap();
        assert!(s.sk.is_some());
        assert_eq!(s.address, derive_sapling_address(&NETWORK, &s.vk, keys.dindex));
        assert!(keys.orchard.as_ref().unwrap().sk.is_some());

        let keys = seed_keys(POOL_SAPLING).await;
        assert!(keys.transparent.is_none());
        assert!(keys.sapling.is_some());
        assert!(keys.orchard.is_none());

        let keys = seed_keys(POOL_IRONWOOD).await;
        assert!(keys.orchard.is_some());
    }

    #[tokio::test]
    async fn ufvk_derivation_matches_seed_derivation() {
        let from_seed = seed_keys(ALL_POOLS).await;
        let ufvk = usk().to_unified_full_viewing_key().encode(&NETWORK).unwrap();
        let source = classify_key(&NETWORK, HwKind::Software, &ufvk).unwrap();
        let from_ufvk = derive_account_keys(&NETWORK, source, "", 0, ALL_POOLS)
            .await
            .unwrap();

        assert!(from_ufvk.seed.is_none());
        assert_eq!(from_ufvk.dindex, from_seed.dindex);

        let (st, ut) = (
            from_seed.transparent.unwrap(),
            from_ufvk.transparent.unwrap(),
        );
        assert!(ut.sk.is_none() && ut.vk.is_some());
        let addrs = |t: &TransparentKeys| -> Vec<(u32, String)> {
            t.addresses.iter().map(|a| (a.dindex, a.address.clone())).collect()
        };
        assert_eq!(addrs(&st), addrs(&ut));
        assert!(ut.addresses.iter().all(|a| a.sk.is_none()));

        let (ss, us) = (from_seed.sapling.unwrap(), from_ufvk.sapling.unwrap());
        assert!(us.sk.is_none());
        assert_eq!(ss.address, us.address);
        assert_eq!(ss.vk.to_bytes(), us.vk.to_bytes());

        let (so, uo) = (from_seed.orchard.unwrap(), from_ufvk.orchard.unwrap());
        assert!(uo.sk.is_none());
        assert_eq!(so.vk.to_bytes(), uo.vk.to_bytes());
    }

    #[tokio::test]
    async fn ledger_sources_are_not_software_derivable() {
        assert!(
            derive_account_keys(&NETWORK, KeySource::ZondaxDevice, "", 0, ALL_POOLS)
                .await
                .is_err()
        );
    }
}

pub fn derive_transparent_sk(tsk: &AccountPrivKey, scope: u32, dindex: u32) -> Result<Vec<u8>> {
    let scope = scope.transparent_scope();
    let tsk = tsk
        .derive_secret_key(scope, NonHardenedChildIndex::from_index(dindex).unwrap())
        .unwrap()
        .to_bytes();
    Ok(tsk.to_vec())
}

pub fn derive_transparent_address(
    tvk: &AccountPubKey,
    scope: u32,
    dindex: u32,
    uncompressed: bool,
) -> Result<(Vec<u8>, TransparentAddress)> {
    let sindex = TransparentKeyScope::custom(scope).unwrap();
    let pk = tvk
        .derive_address_pubkey(sindex, NonHardenedChildIndex::from_index(dindex).unwrap())
        .unwrap();
    let tpk = if uncompressed {
        pk.serialize_uncompressed().to_vec()
    } else {
        pk.serialize().to_vec()
    };
    let pkh: [u8; 20] = Ripemd160::digest(Sha256::digest(&tpk)).into();
    let addr = TransparentAddress::PublicKeyHash(pkh);
    Ok((tpk, addr))
}

pub async fn get_account_seed(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<Seed>> {
    let seed = sqlx::query("SELECT seed, passphrase, aindex FROM accounts WHERE id_account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let mnemonic: Option<String> = row.get(0);
            let phrase: Option<String> = row.get(1);
            let aindex: u32 = row.get(2);
            let phrase = phrase.unwrap_or_default();
            mnemonic.map(|mnemonic| Seed {
                mnemonic,
                phrase,
                aindex,
            })
        })
        .fetch_one(&mut *connection)
        .await?;
    Ok(seed)
}

pub async fn get_sapling_sk(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<ExtendedSpendingKey>> {
    let sk = sqlx::query("SELECT xsk FROM sapling_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let sk: Option<Vec<u8>> = row.get(0);
            sk.map(|sk| ExtendedSpendingKey::read(&*sk).unwrap())
        })
        .fetch_optional(connection)
        .await?;

    Ok(sk.flatten())
}

pub async fn get_sapling_vk(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<DiversifiableFullViewingKey>> {
    let vk = sqlx::query("SELECT xvk FROM sapling_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let vk: Vec<u8> = row.get(0);
            DiversifiableFullViewingKey::from_bytes(&vk.try_into().unwrap()).unwrap()
        })
        .fetch_optional(&mut *connection)
        .await?;

    Ok(vk)
}

pub async fn get_sapling_address(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<PaymentAddress>> {
    let address = sqlx::query("SELECT address FROM sapling_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let address: String = row.get(0);
            PaymentAddress::decode(network, &address).unwrap()
        })
        .fetch_optional(&mut *connection)
        .await?;

    Ok(address)
}

pub async fn get_sapling_note(
    connection: &mut SqliteConnection,
    id: u32,
    height: u32,
    dfvk: &DiversifiableFullViewingKey,
    edge: &FragmentAuthPath,
    empty_roots: &AuthPath,
) -> Result<(sapling_crypto::Note, u32, sapling_crypto::MerklePath)> {
    let r = sqlx::query(
        "SELECT position, diversifier, value, rcm, witness, scope FROM notes
        JOIN witnesses w ON notes.id_note = w.note
        WHERE id_note = ? AND w.height = ?",
    )
    .bind(id)
    .bind(height)
    .map(|row: SqliteRow| {
        let position: u32 = row.get(0);
        let diversifier: Vec<u8> = row.get(1);
        let value: u64 = row.get::<i64, _>(2) as u64;
        let rcm: Vec<u8> = row.get(3);
        let witness: Vec<u8> = row.get(4);
        let scope: u32 = row.get(5);

        let diversifier = sapling_crypto::Diversifier(diversifier.try_into().unwrap());
        let fvk = sapling_dfvk_to_fvk(scope, dfvk);
        let recipient = fvk.vk.to_payment_address(diversifier).unwrap();

        let value = sapling_crypto::value::NoteValue::from_raw(value);

        let rseed =
            sapling_crypto::Rseed::BeforeZip212(Fr::from_bytes(&rcm.try_into().unwrap()).unwrap());

        let note = sapling_crypto::Note::from_parts(recipient, value, rseed);

        let (witness, _) = bincode::decode_from_slice::<Witness, _>(&witness, legacy()).unwrap();

        let auth_path = witness.build_auth_path(edge, empty_roots).unwrap();
        let mut mp = vec![];
        for i in 0..MERKLE_DEPTH as usize {
            mp.push(sapling_crypto::Node::from_bytes(auth_path.0[i]).unwrap());
        }

        assert_eq!(position, witness.position);
        let merkle_path =
            sapling_crypto::MerklePath::from_parts(mp, (witness.position as u64).into()).unwrap();

        (note, scope, merkle_path)
    })
    .fetch_one(connection)
    .await
    .context("retrieve sinput")?;

    Ok(r)
}

pub async fn get_orchard_sk(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<orchard::keys::SpendingKey>> {
    let sk = sqlx::query("SELECT xsk FROM orchard_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let sk: Option<Vec<u8>> = row.get(0);
            sk.map(|sk| orchard::keys::SpendingKey::from_bytes(sk.try_into().unwrap()).unwrap())
        })
        .fetch_optional(&mut *connection)
        .await?;

    Ok(sk.flatten())
}

pub async fn get_orchard_vk(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<orchard::keys::FullViewingKey>> {
    let vk = sqlx::query("SELECT xvk FROM orchard_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let fvk: Vec<u8> = row.get(0);
            orchard::keys::FullViewingKey::read(&*fvk).unwrap()
        })
        .fetch_optional(&mut *connection)
        .await?;

    Ok(vk)
}

pub async fn get_orchard_note(
    connection: &mut SqliteConnection,
    id: u32,
    witness_height: u32,
    ovk: &orchard::keys::FullViewingKey,
    eo: &FragmentAuthPath,
    ero: &AuthPath,
    note_version: NoteVersion,
    rewind_to_position: Option<u32>,
) -> Result<(orchard::Note, orchard::tree::MerklePath)> {
    let (scope, position, diversifier, value, rcm, rho, witness, asset_base) = sqlx::query(
        "SELECT scope, position, diversifier, value, rcm, rho, witness,
                COALESCE(a.asset_base, X'0000000000000000000000000000000000000000000000000000000000000000') as asset_base
         FROM notes
         JOIN witnesses w ON notes.id_note = w.note
         LEFT JOIN assets a ON notes.id_asset = a.id_asset
         WHERE id_note = ? AND w.height = ?",
    )
    .bind(id)
    .bind(witness_height)
    .map(|row: SqliteRow| {
        let scope: Option<u8> = row.get(0);
        let position: u32 = row.get(1);
        let diversifier: Vec<u8> = row.get(2);
        let value: u64 = row.get::<i64, _>(3) as u64;
        let rcm: Vec<u8> = row.get(4);
        let rho: Vec<u8> = row.get(5);
        let witness: Vec<u8> = row.get(6);
        let asset_base: Vec<u8> = row.get(7);
        (scope, position, diversifier, value, rcm, rho, witness, asset_base)
    })
    .fetch_one(connection)
    .await
    .context("retrieve oinput")?;

    let scope = scope.unwrap_or(0);
    anyhow::ensure!(scope <= 1, "Invalid Orchard note scope {scope}");
    let scope = scope.orchard_scope();
    let (witness, _) = bincode::decode_from_slice::<Witness, _>(&witness, legacy())
        .context("decode Orchard witness")?;
    let witness = match rewind_to_position {
        Some(position) => {
            anyhow::ensure!(
                witness.position <= position,
                "Note position {} is after anchor edge position {position}",
                witness.position,
            );
            witness.rewind(position)
        }
        None => witness,
    };
    let rho = Rho::from_bytes(
        &rho.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid rho length"))?,
    )
    .into_option()
    .context("Invalid Orchard rho")?;

    let diversifer = orchard::keys::Diversifier::from_bytes(
        diversifier
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid diversifier length"))?,
    );
    let recipient = ovk.address(diversifer, scope);
    let value = NoteValue::from_raw(value);
    let rseed = RandomSeed::from_bytes(
        rcm.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid rseed length"))?,
        &rho,
    )
    .into_option()
    .context("Invalid Orchard rseed")?;
    let zec_asset_base = [0u8; 32];
    let asset_base = if asset_base == zec_asset_base {
        AssetBase::zatoshi()
    } else {
        AssetBase::from_bytes(
            &asset_base
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid asset base length"))?,
        )
        .into_option()
        .context("Invalid Orchard asset base")?
    };
    let note = Note::from_parts(recipient, value, asset_base, rho, rseed, note_version)
        .into_option()
        .context("Invalid Orchard note")?;

    anyhow::ensure!(
        witness.position == position,
        "Orchard witness position does not match note"
    );
    let auth_path = witness
        .build_auth_path(eo, ero)
        .context("build Orchard authentication path")?;
    let auth_path = auth_path
        .0
        .iter()
        .map(|a| {
            MerkleHashOrchard::from_bytes(a)
                .into_option()
                .context("Invalid Orchard witness sibling")
        })
        .collect::<Result<Vec<_>>>()?;
    let auth_path: [MerkleHashOrchard; MERKLE_DEPTH as usize] = auth_path
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid Orchard authentication path length"))?;
    let merkle_path = orchard::tree::MerklePath::from_parts(witness.position, auth_path);

    Ok((note, merkle_path))
}

pub async fn get_birth_height(connection: &mut SqliteConnection, account: u32) -> Result<u32> {
    let (birth,): (u32,) = sqlx::query_as("SELECT birth FROM accounts WHERE id_account = ?")
        .bind(account)
        .fetch_one(connection)
        .await?;

    Ok(birth)
}

pub async fn get_account_full_address(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    scope: u8,
    hw: u8,
) -> Result<String> {
    let taddress = sqlx::query(
        "SELECT ta.address FROM transparent_address_accounts ta
        JOIN accounts a ON ta.account = a.id_account AND ta.dindex = a.dindex
        AND ta.scope = 0
        WHERE ta.account = ?",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let taddress: String = row.get(0);
        TransparentAddress::decode(network, &taddress).unwrap()
    })
    .fetch_optional(&mut *connection)
    .await?;

    let saddress = sqlx::query(
        "SELECT sa.xvk, sa.address, a.dindex FROM sapling_accounts sa
        JOIN accounts a ON sa.account = a.id_account AND sa.account = ?",
    )
    .bind(account)
    .map(|row: SqliteRow| -> Result<PaymentAddress, anyhow::Error> {
        let xvk: Vec<u8> = row.get(0);
        let address: String = row.get(1);
        let _dindex: u32 = row.get(2);
        let fvk = DiversifiableFullViewingKey::from_bytes(
            &xvk.try_into()
                .map_err(|_| anyhow!("invalid sapling xvk length for account {account}"))?,
        )
        .ok_or_else(|| anyhow!("invalid sapling xvk for account {account}"))?;
        if scope == 1 && hw == 0 {
            // we do not need to derive a diversified change address
            // since they are not exposed to the user
            let (_, pa) = fvk.change_address();
            Ok(pa)
        } else {
            PaymentAddress::decode(network, &address)
                .map_err(|e| anyhow!("invalid stored sapling address for account {account}: {e}"))
        }
    })
    .fetch_optional(&mut *connection)
    .await?
    .transpose()?;

    let oaddress = sqlx::query(
        "SELECT a.dindex, oa.xvk FROM orchard_accounts oa
        JOIN accounts a ON oa.account = a.id_account AND oa.account = ?",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let dindex: u32 = row.get(0);
        let xvk: Vec<u8> = row.get(1);
        let fvk = FullViewingKey::read(&*xvk).unwrap();
        let scope = scope.orchard_scope();
        fvk.address_at(dindex, scope)
    })
    .fetch_optional(connection)
    .await?;

    let address = match (taddress, saddress, oaddress) {
        (Some(taddress), None, None) => taddress.encode(network),
        _ => {
            let ua =
                UnifiedAddress::from_receivers(oaddress, saddress, taddress, None, None).unwrap();
            ua.encode(network)
        }
    };

    Ok(address)
}

pub async fn generate_next_dindex(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<u32> {
    let mut db_tx = connection.begin().await?;
    let ledger = get_ledger(&mut db_tx, account).await?;
    let (aindex, mut dindex): (u32, u32) =
        sqlx::query_as("SELECT aindex, dindex FROM accounts WHERE id_account = ?")
            .bind(account)
            .fetch_one(&mut *db_tx)
            .await?;
    let hw = get_account_hw(&mut db_tx, account).await?;
    // Next Sapling address. Some dindex must be skipped because they do not
    // correspond to a valid sapling address
    let svk = get_sapling_vk(&mut db_tx, account).await?;
    if let Some(svk) = svk.as_ref() {
        dindex += 1;
        let address = if hw != 0 {
            let (di, address) = ledger
                .next_diversifier_address(network, aindex, dindex)
                .await?;
            dindex = di;
            address
        } else {
            let (di, address) = svk.find_address(dindex.into()).unwrap();
            dindex = di.try_into()?;
            address.encode(network)
        };
        sqlx::query("UPDATE sapling_accounts SET address = ?2 WHERE account = ?1")
            .bind(account)
            .bind(&address)
            .execute(&mut *db_tx)
            .await?;
    } else {
        // without sapling, any dindex is ok, just increment
        dindex += 1;
    }

    sqlx::query("UPDATE accounts SET dindex = ? WHERE id_account = ?")
        .bind(dindex)
        .bind(account)
        .execute(&mut *db_tx)
        .await?;

    let address = match TransparentKeySource::try_load(&mut db_tx, account).await? {
        Some(source) => Some(source.derive(network, 0, dindex).await?),
        None => None,
    };

    if let Some(DerivedTransparentAddr { sk, pk, address }) = address {
        store_account_transparent_addr(
            &mut db_tx,
            account,
            0,
            dindex,
            sk,
            &pk,
            &address,
            false,
        )
        .await?;
        ensure_internal_change_address(network, &mut db_tx, account, dindex).await?;
    }
    db_tx.commit().await?;

    Ok(dindex)
}

pub async fn generate_next_change_address(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<String>> {
    let dindex = sqlx::query(
        "SELECT MAX(dindex) FROM transparent_address_accounts WHERE account = ? AND scope = 1",
    )
    .bind(account)
    .map(|row: SqliteRow| row.get::<Option<u32>, _>(0))
    .fetch_one(&mut *connection)
    .await?;

    let (xsk, xvk) = get_transparent_keys(connection, account).await?;

    if let Some(tvk) = xvk.as_ref() {
        let dindex = match dindex {
            Some(dindex) => dindex + 1, // increment
            None => 0,                  // first change address
        };

        let sk = xsk
            .as_ref()
            .map(|tsk| derive_transparent_sk(tsk, 1, dindex).unwrap());
        let (change_pk, change_address) = derive_transparent_address(tvk, 1, dindex, false)?;
        let change_address = change_address.encode(network);

        store_account_transparent_addr(
            &mut *connection,
            account,
            1,
            dindex,
            sk,
            &change_pk,
            &change_address,
            false,
        )
        .await?;

        return Ok(Some(change_address));
    }

    Ok(None)
}

pub(crate) async fn get_transparent_keys(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<(Option<AccountPrivKey>, Option<AccountPubKey>)> {
    let tkeys = sqlx::query("SELECT xsk, xvk FROM transparent_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let xsk: Option<Vec<u8>> = row.get(0);
            let xvk: Option<Vec<u8>> = row.get(1);
            let xsk = xsk.map(|xsk| AccountPrivKey::from_bytes(&xsk).unwrap());
            let xvk = xvk.map(|xvk| AccountPubKey::deserialize(&xvk.try_into().unwrap()).unwrap());
            (xsk, xvk)
        })
        .fetch_optional(&mut *connection)
        .await?;
    let (xsk, xvk) = match tkeys {
        Some((xsk, xvk)) => (xsk, xvk),
        None => (None, None),
    };
    Ok((xsk, xvk))
}

/// The stored transparent address that receives a transaction's transparent
/// change: the account's change-scope address at the current dindex. Pairs are
/// materialized together when addresses are derived, so a missing row is a
/// pairing-invariant violation and is surfaced as an error rather than derived.
pub async fn transparent_change_address(
    connection: &mut SqliteConnection,
    account: u32,
    use_internal: bool,
    dindex: u32,
) -> Result<String> {
    let scope: u32 = if use_internal { 1 } else { 0 };
    let kind = if use_internal { "change" } else { "external" };
    let address: Option<String> = sqlx::query(
        "SELECT address FROM transparent_address_accounts
        WHERE account = ?1 AND scope = ?2 AND dindex = ?3",
    )
    .bind(account)
    .bind(scope)
    .bind(dindex)
    .map(|row: SqliteRow| row.get(0))
    .fetch_optional(&mut *connection)
    .await?;
    address.ok_or_else(|| {
        anyhow!(
            "no stored {kind} transparent address at dindex {dindex} for account {account}; \
             generate a new address set or resync to materialize the change-address pair"
        )
    })
}

/// Derive and store the transparent address at (scope, dindex) when missing.
/// Ensure the internal (scope 1) change counterpart of external index `dindex`
/// exists for accounts that use internal change. Returns whether a new row was
/// inserted.
pub async fn ensure_internal_change_address(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    dindex: u32,
) -> Result<bool> {
    let (use_internal,): (bool,) =
        sqlx::query_as("SELECT use_internal FROM accounts WHERE id_account = ?")
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;
    if !use_internal {
        return Ok(false);
    }
    let source = TransparentKeySource::load(connection, account).await?;
    ensure_transparent_address(connection, network, &source, account, 1, dindex).await
}

/// Materialize the internal (change) counterpart of every stored external
/// transparent address for accounts that use internal change. Idempotent; run
/// at database open to reconcile databases that predate change-address pairing.
pub async fn backfill_transparent_change_addresses(
    network: &Network,
    connection: &mut SqliteConnection,
) -> Result<usize> {
    let mut added = 0;
    let accounts: Vec<u32> = sqlx::query(
        "SELECT a.id_account FROM accounts a
        WHERE a.use_internal = 1 AND EXISTS (
            SELECT 1 FROM transparent_address_accounts ta
            WHERE ta.account = a.id_account AND ta.scope = 0)",
    )
    .map(|row: SqliteRow| row.get(0))
    .fetch_all(&mut *connection)
    .await?;
    for account in accounts {
        added += backfill_account_transparent_change_addresses(network, connection, account).await?;
    }
    Ok(added)
}

/// Materialize missing internal change addresses for one account.
pub async fn backfill_account_transparent_change_addresses(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<usize> {
    // Only software keys: a device may not be connected when this runs.
    let source = match TransparentKeySource::try_load(connection, account).await? {
        Some(source) if source.is_software() => source,
        _ => return Ok(0),
    };
    let (use_internal,): (bool,) =
        sqlx::query_as("SELECT use_internal FROM accounts WHERE id_account = ?")
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;
    if !use_internal {
        return Ok(0);
    }
    let indexes: Vec<u32> = sqlx::query(
        "SELECT dindex FROM transparent_address_accounts WHERE account = ? AND scope = 0",
    )
    .bind(account)
    .map(|row: SqliteRow| row.get(0))
    .fetch_all(&mut *connection)
    .await?;
    let mut added = 0;
    for dindex in indexes {
        if ensure_transparent_address(connection, network, &source, account, 1, dindex).await? {
            added += 1;
        }
    }
    Ok(added)
}

pub async fn get_addresses(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    ua_pools: u8,
) -> Result<Addresses> {
    let dindex = crate::db::get_account_dindex(connection, account).await?;
    let diversifier_index = dindex;

    let tkeys = crate::db::select_account_transparent(connection, account, dindex).await?;
    let skeys = crate::db::select_account_sapling(network, connection, account).await?;
    let okeys = crate::db::select_account_orchard(connection, account).await?;

    let taddr = tkeys
        .xvk
        .as_ref()
        .map(|xvk| derive_transparent_address(xvk, 0, dindex, false).unwrap().1);

    let dindex = dindex as u64;
    let saddr = skeys.address;
    let oaddr = okeys
        .xvk
        .as_ref()
        .map(|xvk| xvk.address_at(dindex, orchard::keys::Scope::External));

    let ua_orchard = UnifiedAddress::from_receivers(oaddr, None, None, None, None);

    let ua = UnifiedAddress::from_receivers(
        if ua_pools & POOL_ORCHARD != 0 {
            oaddr
        } else {
            None
        },
        if ua_pools & POOL_SAPLING != 0 {
            saddr
        } else {
            None
        },
        if ua_pools & POOL_TRANSPARENT != 0 {
            taddr
        } else {
            None
        },
        None,
        None,
    );

    // final fallback if we have a transparent address from a BIP 38 secret key
    let taddr = taddr.map(|x| x.encode(&network)).or(tkeys.address);

    let addresses = Addresses {
        taddr,
        saddr: saddr.map(|x| x.encode(&network)),
        oaddr: ua_orchard.map(|x| x.encode(&network)),
        ua: ua.map(|x| x.encode(&network)),
        diversifier_index,
    };

    Ok(addresses)
}

pub async fn reset_sync(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<()> {
    let birth_height = get_birth_height(&mut *connection, account).await?;
    let mut db_tx = connection.begin().await?;
    chain_log::trim_to(&mut db_tx, account, 0).await?;
    chain_log::init_checkpoints(&mut db_tx, network, account, birth_height).await?;
    db_tx.commit().await?;
    Ok(())
}

pub(crate) fn asset_display(
    id_asset: Option<i32>,
    asset_name: Option<String>,
    asset_desc_hash: Option<Vec<u8>>,
) -> String {
    match id_asset {
        Some(_) => asset_name.filter(|n| !n.is_empty()).unwrap_or_else(|| {
            asset_desc_hash
                .map(|h| hex::encode(&h[..8.min(h.len())]))
                .unwrap_or_else(|| "ZSA".to_string())
        }),
        None => "ZEC".to_string(),
    }
}

pub async fn get_tx_details(
    connection: &mut SqliteConnection,
    account: u32,
    id_tx: u32,
) -> Result<TxAccount> {
    let mut tx = sqlx::query(
        "SELECT txid, height, time, price, category FROM transactions t
        WHERE account = ? AND id_tx = ?",
    )
    .bind(account)
    .bind(id_tx)
    .map(|row: SqliteRow| {
        let txid: Vec<u8> = row.get(0);
        let height: u32 = row.get(1);
        let time: u32 = row.get(2);
        let price: Option<f64> = row.get(3);
        let category: Option<u32> = row.get(4);
        TxAccount {
            id: id_tx,
            account,
            txid,
            height,
            time,
            price,
            category,
            notes: vec![],
            spends: vec![],
            outputs: vec![],
            memos: vec![],
            user_memo: None,
        }
    })
    .fetch_one(&mut *connection)
    .await?;

    tx.notes = crate::db::get_tx_notes(connection, account, tx.id).await?;
    tx.outputs = tx_outputs(connection, account, tx.id).await?;
    tx.spends = tx_spends(connection, account, tx.id).await?;
    tx.memos = tx_memos(connection, account, tx.id).await?;

    let user_memo: Option<String> =
        sqlx::query("SELECT user_memo FROM user_memos WHERE account = ?1 AND id_tx = ?2")
            .bind(account)
            .bind(id_tx)
            .map(|row: SqliteRow| row.get(0))
            .fetch_optional(&mut *connection)
            .await?
            .flatten();
    tx.user_memo = user_memo;

    Ok(tx)
}

/// A transaction's outputs for an account, with the first matching contact
/// name and the output's memo when one was decoded.
pub async fn tx_outputs(
    connection: &mut SqliteConnection,
    account: u32,
    id_tx: u32,
) -> Result<Vec<TxOutput>> {
    let outputs = sqlx::query(
        "SELECT o.id_output, o.pool, o.vout, o.height, o.value, o.address,
                c.name as contact_name, m.memo_text
         FROM outputs o
         LEFT JOIN contact_addresses ca ON o.address = ca.receiver AND o.pool = ca.pool
         LEFT JOIN contacts c ON ca.contact_id = c.id_contact
         LEFT JOIN memos m ON o.id_output = m.output
         WHERE o.account = ?1 AND o.tx = ?2
         ORDER BY o.id_output",
    )
    .bind(account)
    .bind(id_tx)
    .map(|row: SqliteRow| TxOutput {
        id: row.get(0),
        pool: row.get(1),
        vout: row.get(2),
        height: row.get(3),
        value: row.get(4),
        address: row.get(5),
        contact_name: row.get(6),
        memo: row.get(7),
    })
    .fetch_all(&mut *connection)
    .await?;
    Ok(outputs)
}

/// What a transaction spent for an account, as positive amounts per note.
pub async fn tx_spends(
    connection: &mut SqliteConnection,
    account: u32,
    id_tx: u32,
) -> Result<Vec<TxSpend>> {
    let spends = sqlx::query(
        "SELECT s.id_note, s.pool, s.height, s.value,
        n.id_asset, a.asset_name, a.asset_desc_hash
        FROM spends s
        JOIN notes n ON s.id_note = n.id_note
        LEFT JOIN assets a ON n.id_asset = a.id_asset
        WHERE s.account = ?1 AND s.tx = ?2",
    )
    .bind(account)
    .bind(id_tx)
    .map(|row: SqliteRow| {
        let value: i64 = row.get(3);
        let id_asset: Option<i32> = row.get(4);
        TxSpend {
            id: row.get(0),
            pool: row.get(1),
            height: row.get(2),
            value: -value as u64,
            id_asset: id_asset.map(|v| v as u32),
            asset_display: asset_display(id_asset, row.get(5), row.get(6)),
        }
    })
    .fetch_all(&mut *connection)
    .await?;
    Ok(spends)
}

/// A transaction's memos for an account, in memo id order.
pub async fn tx_memos(
    connection: &mut SqliteConnection,
    account: u32,
    id_tx: u32,
) -> Result<Vec<TxMemo>> {
    let memos = sqlx::query(
        "SELECT note, output, pool, memo_text, memo_bytes FROM memos
        WHERE account = ?1 AND tx = ?2
        ORDER BY id_memo",
    )
    .bind(account)
    .bind(id_tx)
    .map(|row: SqliteRow| TxMemo {
        note: row.get(0),
        output: row.get(1),
        pool: row.get(2),
        memo: row.get(3),
        memo_bytes: row.get(4),
    })
    .fetch_all(&mut *connection)
    .await?;
    Ok(memos)
}

/// Per-address transaction statistics for an account, one row per
/// (scope, diversifier index, pool) of the selected pools, or one aggregated
/// unified-address row per slot when `aggregate` is set.
pub async fn fetch_address_tx_count(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    aggregate: bool,
    pool_filter: u8,
) -> Result<Vec<TAddressTxCount>> {
    let dindex = get_account_dindex(connection, account).await?;

    let t_pool = crate::db::select_account_transparent(connection, account, dindex).await;
    let s_pool = crate::db::select_account_sapling(network, connection, account).await;
    let o_pool = crate::db::select_account_orchard(connection, account).await;

    let enabled_pools = (if t_pool
        .as_ref()
        .map(|t| t.xvk.is_some() || t.address.is_some())
        .unwrap_or(false)
    {
        1u8
    } else {
        0u8
    }) | (if s_pool.as_ref().map(|s| s.xvk.is_some()).unwrap_or(false) {
        2u8
    } else {
        0u8
    }) | (if o_pool.as_ref().map(|o| o.xvk.is_some()).unwrap_or(false) {
        4u8
    } else {
        0u8
    });

    // Only derive addresses for pools that are both selected and enabled
    let selected = PoolMask(pool_filter).intersect(&PoolMask(enabled_pools));

    let tkeys = t_pool?;
    let skeys = s_pool?;
    let okeys = o_pool?;

    // Fetch per-slot stats for transparent and shielded pools
    let transparent_stats =
        crate::db::fetch_transparent_slot_stats(connection, account).await?;
    let shielded_stats = crate::db::fetch_shielded_slot_stats(connection, account).await?;

    // Per-pool stats lookups (kept separate, not merged)
    let t_stats: HashMap<(u8, u32), (u64, u32, u32)> = transparent_stats
        .iter()
        .map(|s| ((s.scope, s.dindex), (s.amount, s.tx_count, s.time)))
        .collect();
    let s_stats: HashMap<(u8, u8, u32), (u64, u32, u32)> = shielded_stats
        .iter()
        .map(|s| ((s.pool, s.scope, s.dindex), (s.amount, s.tx_count, s.time)))
        .collect();

    // Pre-compute internal Sapling IVK
    let internal_sap_ivk: Option<sapling_crypto::zip32::IncomingViewingKey> =
        skeys.xvk.as_ref().and_then(|dfvk| {
            let dfvk_bytes = dfvk.to_bytes();
            let dk_bytes: [u8; 32] = dfvk_bytes[96..128].try_into().ok()?;
            let dk = sapling_crypto::zip32::DiversifierKey::from_bytes(dk_bytes);
            let (int_fvk, int_dk) = sapling_derive_internal_fvk(dfvk.fvk(), &dk)?;
            let mut ivk_bytes = [0u8; 64];
            ivk_bytes[..32].copy_from_slice(int_dk.as_bytes());
            ivk_bytes[32..].copy_from_slice(&int_fvk.vk.ivk().to_repr());
            sapling_crypto::zip32::IncomingViewingKey::from_bytes(&ivk_bytes).into_option()
        });

    let mut results = Vec::new();

    for scope in [0u8, 1u8] {
        for d in 0..=dindex {
            // Determine if this (scope, d) slot is valid.
            // Sapling diversifier validity gates the entire slot — if the
            // account has sapling keys and the address at this diversifier
            // index is invalid, only the transparent fallback at d=0 applies.
            let mut valid_dindex = true;
            if skeys.xvk.is_some() {
                let dfvk = skeys.xvk.as_ref().unwrap();
                if !dfvk.has_sapling_address(scope, d as u64, internal_sap_ivk.as_ref()) {
                    valid_dindex = false;
                }
            }

            if valid_dindex {
                // Per-pool stats
                let t_st = t_stats.get(&(scope, d)).copied().unwrap_or((0, 0, 0));
                let s_st = s_stats.get(&(1, scope, d)).copied().unwrap_or((0, 0, 0));
                let o_st = s_stats.get(&(2, scope, d)).copied().unwrap_or((0, 0, 0));

                // Derive addresses for selected pools
                let mut t_addr_raw: Option<TransparentAddress> = None;
                let t_str = if selected.has_pool(0) {
                    if let Some(tvk) = &tkeys.xvk {
                        let (_, taddr) = crate::account::derive_transparent_address(
                            tvk,
                            scope as u32,
                            d,
                            false,
                        )?;
                        t_addr_raw = Some(taddr);
                        Some(taddr.encode(network))
                    } else {
                        None
                    }
                } else {
                    None
                };

                let mut s_addr_raw: Option<sapling_crypto::PaymentAddress> = None;
                let s_str = if selected.has_pool(1) {
                    if let Some(dfvk) = &skeys.xvk {
                        s_addr_raw =
                            dfvk.sapling_address_at(scope, d as u64, internal_sap_ivk.as_ref());
                        s_addr_raw.as_ref().map(|pa| pa.encode(network))
                    } else {
                        None
                    }
                } else {
                    None
                };

                let mut o_addr_raw: Option<orchard::Address> = None;
                let o_str = if selected.has_pool(2) {
                    if let Some(fvk) = &okeys.xvk {
                        let s = scope.orchard_scope();
                        let addr = fvk.address_at(d as u64, s);
                        o_addr_raw = Some(addr);
                        Some(
                            UnifiedAddress::from_receivers(Some(addr), None, None, None, None)
                                .unwrap()
                                .encode(network),
                        )
                    } else {
                        None
                    }
                } else {
                    None
                };

                if aggregate {
                    if let Some(ua) = UnifiedAddress::from_receivers(
                        o_addr_raw, s_addr_raw, t_addr_raw, None, None,
                    ) {
                        results.push(TAddressTxCount {
                            pool: 0,
                            address: ua.encode(network),
                            scope,
                            dindex: d,
                            amount: t_st.0.wrapping_add(s_st.0).wrapping_add(o_st.0),
                            tx_count: t_st.1.wrapping_add(s_st.1).wrapping_add(o_st.1),
                            time: t_st.2.max(s_st.2).max(o_st.2),
                        });
                    }
                } else {
                    if let Some(addr) = t_str {
                        results.push(TAddressTxCount {
                            pool: 0,
                            address: addr,
                            scope,
                            dindex: d,
                            amount: t_st.0,
                            tx_count: t_st.1,
                            time: t_st.2,
                        });
                    }
                    if let Some(addr) = s_str {
                        results.push(TAddressTxCount {
                            pool: 1,
                            address: addr,
                            scope,
                            dindex: d,
                            amount: s_st.0,
                            tx_count: s_st.1,
                            time: s_st.2,
                        });
                    }
                    if let Some(addr) = o_str {
                        results.push(TAddressTxCount {
                            pool: 2,
                            address: addr,
                            scope,
                            dindex: d,
                            amount: o_st.0,
                            tx_count: o_st.1,
                            time: o_st.2,
                        });
                    }
                }
            } else if d == 0 && selected.has_pool(0) {
                // Fallback: sapling address invalid at this slot, but d=0
                // always produces a valid transparent address.
                let t_st = t_stats.get(&(scope, 0)).copied().unwrap_or((0, 0, 0));

                let t_str = if let Some(tvk) = &tkeys.xvk {
                    let (_, taddr) =
                        crate::account::derive_transparent_address(tvk, scope as u32, 0, false)?;
                    Some(taddr.encode(network))
                } else {
                    None
                };

                // Aggregate of 1 taddr is the taddr itself.
                if let Some(addr) = t_str {
                    results.push(TAddressTxCount {
                        pool: 0,
                        address: addr,
                        scope,
                        dindex: 0,
                        amount: t_st.0,
                        tx_count: t_st.1,
                        time: t_st.2,
                    });
                }
            }
        }
    }

    results.sort_by(|a, b| {
        a.scope
            .cmp(&b.scope)
            .then(a.dindex.cmp(&b.dindex))
            .then(a.pool.cmp(&b.pool))
    });
    Ok(results)
}

pub async fn get_account_frost_params(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<FrostParams>> {
    let frost = sqlx::query("SELECT id, n, t FROM dkg_params WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let id: u8 = row.get(0);
            let n: u8 = row.get(1);
            let t: u8 = row.get(2);
            FrostParams { id, n, t }
        })
        .fetch_optional(connection)
        .await?;

    Ok(frost)
}

pub async fn list_folders(connection: &mut SqliteConnection) -> Result<Vec<Folder>> {
    let folders = sqlx::query("SELECT id_folder, name FROM folders ORDER BY name")
        .map(|r: SqliteRow| Folder {
            id: r.get(0),
            name: r.get(1),
        })
        .fetch_all(connection)
        .await?;
    Ok(folders)
}

pub async fn create_new_folder(connection: &mut SqliteConnection, name: &str) -> Result<Folder> {
    sqlx::query("INSERT OR IGNORE INTO folders(name) VALUES (?1)")
        .bind(name)
        .execute(&mut *connection)
        .await?;
    let id = sqlx::query("SELECT id_folder FROM folders WHERE name = ?1")
        .bind(name)
        .map(|r: SqliteRow| r.get::<u32, _>(0))
        .fetch_one(&mut *connection)
        .await?;

    Ok(Folder {
        id,
        name: name.to_string(),
    })
}

pub async fn rename_folder(connection: &mut SqliteConnection, id: u32, name: &str) -> Result<()> {
    if sqlx::query("SELECT 1 FROM folders WHERE name = ?1")
        .bind(name)
        .fetch_optional(&mut *connection)
        .await?
        .is_some()
    {
        anyhow::bail!("Folder name already exists");
    }

    sqlx::query("UPDATE folders SET name = ?2 WHERE id_folder = ?1")
        .bind(id)
        .bind(name)
        .execute(connection)
        .await?;
    Ok(())
}

pub async fn delete_folders(connection: &mut SqliteConnection, ids: &[u32]) -> Result<()> {
    for id in ids {
        sqlx::query("UPDATE accounts SET folder = NULL where folder = ?1")
            .bind(id)
            .execute(&mut *connection)
            .await?;

        sqlx::query("DELETE FROM folders WHERE id_folder = ?1")
            .bind(id)
            .execute(&mut *connection)
            .await?;
    }
    Ok(())
}

pub async fn list_categories(connection: &mut SqliteConnection) -> Result<Vec<Category>> {
    let folders =
        sqlx::query("SELECT id_category, name, income FROM categories ORDER BY income, name")
            .map(|r: SqliteRow| Category {
                id: r.get(0),
                name: r.get(1),
                is_income: r.get(2),
            })
            .fetch_all(connection)
            .await?;
    Ok(folders)
}

pub async fn create_new_category(
    connection: &mut SqliteConnection,
    category: &Category,
) -> Result<u32> {
    sqlx::query("INSERT OR IGNORE INTO categories(name, income) VALUES (?1, ?2)")
        .bind(&category.name)
        .bind(category.is_income)
        .execute(&mut *connection)
        .await?;
    let id = sqlx::query("SELECT id_category FROM categories WHERE name = ?1")
        .bind(&category.name)
        .map(|r: SqliteRow| r.get::<u32, _>(0))
        .fetch_one(&mut *connection)
        .await?;

    Ok(id)
}

pub async fn rename_category(connection: &mut SqliteConnection, category: &Category) -> Result<()> {
    if sqlx::query("SELECT 1 FROM categories WHERE name = ?1")
        .bind(&category.name)
        .fetch_optional(&mut *connection)
        .await?
        .is_some()
    {
        anyhow::bail!("Category name already exists");
    }

    sqlx::query("UPDATE categories SET name = ?2, income = ?3 WHERE id_category = ?1")
        .bind(category.id)
        .bind(&category.name)
        .bind(category.is_income)
        .execute(connection)
        .await?;
    Ok(())
}

pub async fn delete_categories(connection: &mut SqliteConnection, ids: &[u32]) -> Result<()> {
    for id in ids {
        chain_log::clear_category(&mut *connection, *id).await?;

        sqlx::query("DELETE FROM categories WHERE id_category = ?1")
            .bind(id)
            .execute(&mut *connection)
            .await?;
    }
    Ok(())
}

pub async fn has_transparent_pub_key(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<bool> {
    let r = sqlx::query("SELECT xvk FROM transparent_accounts WHERE account = ?1")
        .bind(account)
        .map(|r: SqliteRow| r.get::<Option<Vec<u8>>, _>(0))
        .fetch_optional(connection)
        .await?
        .flatten();
    Ok(r.is_some())
}

pub async fn has_pool(connection: &mut SqliteConnection, account: u32, pool: u8) -> Result<bool> {
    let has_pool = match pool {
        0 => sqlx::query("SELECT 1 FROM transparent_accounts WHERE account = ?1")
            .bind(account)
            .fetch_optional(connection)
            .await?
            .is_some(),
        1 => sqlx::query("SELECT 1 FROM sapling_accounts WHERE account = ?1")
            .bind(account)
            .fetch_optional(connection)
            .await?
            .is_some(),
        2 => sqlx::query("SELECT 1 FROM orchard_accounts WHERE account = ?1")
            .bind(account)
            .fetch_optional(connection)
            .await?
            .is_some(),
        3 => sqlx::query("SELECT 1 FROM orchard_accounts WHERE account = ?1")
            .bind(account)
            .fetch_optional(connection)
            .await?
            .is_some(),
        _ => unreachable!(),
    };
    Ok(has_pool)
}

pub(crate) fn derive_sapling_address(
    network: &Network,
    sxvk: &DiversifiableFullViewingKey,
    dindex: u32,
) -> String {
    let address = sxvk.address(dindex.into()).unwrap();
    address.encode(network)
}
