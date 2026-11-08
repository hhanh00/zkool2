//! Every key of an account as a tree.
//!
//! The root is the key that created the account, inferred from what the
//! account stores since nothing records how it was imported: the seed when
//! there is one, else the FROST backup string, else the one pool key of a
//! single-pool account, else the unified viewing key. Below the root hang the
//! keys that derive from it: a seed's passphrase, index and pool keys, a
//! transparent key's addresses.
//!
//! Each node has a secret face and a viewing face so the Key Tree page can
//! flip between them without changing the tree. A face with no value reads
//! N/A on the page. The Orchard spending key is never encoded: it has no
//! standard encoding and nothing imports it.

use anyhow::Result;
use bech32::{Bech32, Hrp};
use std::str::FromStr as _;

use bip32::{ChildNumber, ExtendedPrivateKey, Prefix};
use secp256k1::SecretKey;
use sqlx::SqliteConnection;
use zcash_address::unified::{Encoding as _, Ivk, Revision, Uitem, Uivk};
use zcash_keys::encoding::{encode_extended_full_viewing_key, encode_extended_spending_key};
use zcash_protocol::consensus::Parameters as _;
use zcash_transparent::keys::IncomingViewingKey as _;

use crate::{
    api::{
        account::{KeyFace, KeyNode},
        coin::Network,
    },
    bip38::{export_tsk, TSecretKey},
    db::{
        get_account_dindex, get_account_hw, is_incoming_only, list_transparent_address_keys,
        select_account_orchard, select_account_sapling, select_account_transparent,
        OrchardKeys, SaplingKeys, TransparentAddressKey, TransparentKeys,
    },
    key::{get_account_ufvk, xpub},
    pay::pool::{POOL_IRONWOOD, POOL_ORCHARD, POOL_SAPLING, POOL_TRANSPARENT},
};

const ORCHARD_SECRET_NOTE: &str = "Not shown: the Orchard spending key has no standard encoding \
and no wallet imports it. Restore Orchard from the seed.";
const ORCHARD_FROST_NOTE: &str = "Not shown: the group's Orchard key is split across its members. \
Your share is in the backup string above.";
const CARRIED_NOTE: &str = "Carried inside the unified key above.";
const SAPLING_UNIFIED_NOTE: &str =
    "Encoded as a Sapling-only unified key: the extended key's derivation data is not stored.";

fn face(label: &str, value: Option<String>) -> KeyFace {
    KeyFace {
        label: label.to_string(),
        value,
        note: None,
    }
}

fn noted(label: &str, note: &str) -> KeyFace {
    KeyFace {
        label: label.to_string(),
        value: None,
        note: Some(note.to_string()),
    }
}

fn node(name: &str, secret: KeyFace, viewing: KeyFace) -> KeyNode {
    KeyNode {
        name: name.to_string(),
        summary: false,
        secret,
        viewing,
        detail: None,
        children: vec![],
    }
}

/// The Key Tree of `account`, or `None` for a Ledger account.
pub async fn get_account_key_tree(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Option<KeyNode>> {
    if get_account_hw(connection, account).await? != 0 {
        return Ok(None);
    }
    let dindex = get_account_dindex(connection, account).await?;
    let tkeys = select_account_transparent(connection, account, dindex).await?;
    let skeys = select_account_sapling(network, connection, account).await?;
    let okeys = select_account_orchard(connection, account).await?;
    let addresses = list_transparent_address_keys(connection, account).await?;
    let seed = crate::account::get_account_seed(connection, account).await?;
    let frost = is_frost(connection, account).await?;
    let incoming_only = is_incoming_only(connection, account).await?;

    let has_t = tkeys.xvk.is_some() || tkeys.ivk.is_some() || !addresses.is_empty();
    let has_s = skeys.xvk.is_some() || skeys.ivk.is_some();
    let has_o = okeys.xvk.is_some() || okeys.ivk.is_some();
    let mut pools = 0u8;
    if has_t {
        pools |= POOL_TRANSPARENT;
    }
    if has_s {
        pools |= POOL_SAPLING;
    }
    if has_o {
        pools |= POOL_ORCHARD | POOL_IRONWOOD;
    }

    let mut pool_nodes = vec![];
    if has_t {
        pool_nodes.push(transparent_node(network, &tkeys, &addresses)?);
    }
    if has_s {
        pool_nodes.push(sapling_node(network, connection, account, &skeys).await?);
    }
    if has_o {
        pool_nodes.push(orchard_node(&okeys, frost));
    }

    let mut root = if let Some(seed) = seed {
        let viewing = unified_face(network, connection, account, pools, false).await?;
        let mut root = node("Seed", face("Seed phrase", Some(seed.mnemonic)), viewing);
        let mut children = vec![];
        if !seed.phrase.is_empty() {
            let mut n = node(
                "Passphrase",
                face("Passphrase", Some(seed.phrase)),
                face("Passphrase", None),
            );
            n.summary = true;
            children.push(n);
        }
        let mut n = node(
            "Account index",
            face("Account index", Some(seed.aindex.to_string())),
            face("Account index", None),
        );
        n.summary = true;
        children.push(n);
        children.extend(pool_nodes);
        root.children = children;
        root
    } else if frost {
        let backup = crate::frost::backup::export_backup(network, connection, account).await?;
        let viewing = unified_face(network, connection, account, pools, false).await?;
        let mut root = node("FROST group", face("Backup string", Some(backup)), viewing);
        root.children = pool_nodes;
        root
    } else if pool_nodes.len() == 1 {
        // The one pool key is the key that created the account.
        pool_nodes.pop().unwrap()
    } else {
        let viewing = unified_face(network, connection, account, pools, incoming_only).await?;
        let mut root = node("Unified key", face("Spending key", None), viewing);
        root.children = pool_nodes;
        root
    };
    root.summary = true;
    Ok(Some(root))
}

async fn is_frost(connection: &mut SqliteConnection, account: u32) -> Result<bool> {
    let (frost,): (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM dkg_params WHERE account = ?)")
            .bind(account)
            .fetch_one(&mut *connection)
            .await?;
    Ok(frost)
}

/// The viewing face of a root that spans several pools: the unified full
/// viewing key, or the unified incoming viewing key of an incoming-only
/// account. A seed account with only the transparent pool has an xpub.
async fn unified_face(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    pools: u8,
    incoming_only: bool,
) -> Result<KeyFace> {
    if incoming_only {
        // Assembled from the stored items: the unified key type only builds
        // from a full key or a string, and an incoming-only account has
        // neither.
        let dindex = get_account_dindex(connection, account).await?;
        let tkeys = select_account_transparent(connection, account, dindex).await?;
        let skeys = select_account_sapling(network, connection, account).await?;
        let okeys = select_account_orchard(connection, account).await?;
        let items = [
            tkeys
                .ivk
                .map(|ivk| Ivk::P2pkh(ivk.serialize().try_into().unwrap())),
            skeys.ivk.map(|ivk| Ivk::Sapling(ivk.to_bytes())),
            okeys.ivk.map(|ivk| Ivk::Orchard(ivk.to_bytes())),
        ];
        let uivk = Uivk::try_from_items(
            Revision::R0,
            items.into_iter().flatten().map(Uitem::Data).collect(),
        )?;
        return Ok(face(
            "Unified incoming viewing key",
            Some(uivk.encode(&network.network_type())),
        ));
    }
    let shielded = pools & (POOL_SAPLING | POOL_ORCHARD) != 0;
    let label = if shielded {
        "Unified full viewing key"
    } else {
        "Transparent xpub"
    };
    let key = get_account_ufvk(network, connection, account, pools).await?;
    Ok(face(label, Some(key)))
}

/// The xprv of a transparent account key: its stored bytes are the BIP 32
/// payload without the prefix, so the string comes back exactly as it was
/// imported or derived.
fn xprv(bytes: &[u8]) -> String {
    let mut payload = Prefix::XPRV.to_bytes().to_vec();
    payload.extend_from_slice(bytes);
    bs58::encode(payload).with_check().into_string()
}

fn zpk(pk: &[u8]) -> Result<String> {
    Ok(bech32::encode::<Bech32>(Hrp::parse("zpk")?, pk)?)
}

fn transparent_node(
    network: &Network,
    tkeys: &TransparentKeys,
    addresses: &[TransparentAddressKey],
) -> Result<KeyNode> {
    let _ = network;
    let mut n = match (&tkeys.xsk, &tkeys.xvk, &tkeys.ivk) {
        (Some(xsk), _, _) => {
            // The xpub comes from the xprv so both carry the same derivation
            // data; a stored viewing key alone has a dummy fingerprint.
            let xprv = xprv(&xsk.to_bytes());
            let xpub = ExtendedPrivateKey::<SecretKey>::from_str(&xprv)?
                .public_key()
                .to_extended_key(Prefix::XPUB)
                .to_string();
            node(
                "Transparent",
                face("Transparent xprv", Some(xprv)),
                face("Transparent xpub", Some(xpub)),
            )
        }
        (None, Some(xvk), _) => node(
            "Transparent",
            face("Transparent xprv", None),
            face(
                "Transparent xpub",
                Some(xpub(&xvk.serialize(), 3, ChildNumber::new(0, true)?)?),
            ),
        ),
        (None, None, Some(ivk)) => node(
            "Transparent",
            face("Transparent xprv", None),
            face(
                "Transparent xpub, external chain",
                Some(xpub(&ivk.serialize(), 4, ChildNumber::new(0, false)?)?),
            ),
        ),
        _ => {
            // A single imported key: WIF, public key or address. The one
            // address row is the key itself.
            let Some(a) = addresses.first() else {
                anyhow::bail!("transparent account without keys or addresses");
            };
            let mut n = address_node(a)?;
            n.name = "Transparent".to_string();
            return Ok(n);
        }
    };
    n.children = addresses.iter().map(address_node).collect::<Result<_>>()?;
    Ok(n)
}

fn address_node(a: &TransparentAddressKey) -> Result<KeyNode> {
    let wif = a
        .sk
        .as_ref()
        .map(|sk| {
            let sk = SecretKey::from_secret_bytes(sk.as_slice().try_into()?)?;
            Ok::<_, anyhow::Error>(export_tsk(&TSecretKey(sk, a.uncompressed)))
        })
        .transpose()?;
    let name = match a.scope {
        0 => format!("External #{}", a.dindex),
        _ => format!("Change #{}", a.dindex),
    };
    let mut n = node(
        &name,
        face("Transparent WIF", wif),
        face("Transparent public key", Some(zpk(&a.pk)?)),
    );
    n.detail = Some(a.address.clone());
    Ok(n)
}

// The extended viewing key is what a `zxviews` string encodes; the
// diversifiable key the crate steers to has no string form.
#[allow(deprecated)]
async fn sapling_node(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    skeys: &SaplingKeys,
) -> Result<KeyNode> {
    let net = network.network_type();
    let n = match (&skeys.xsk, &skeys.xvk) {
        (Some(xsk), _) => node(
            "Sapling",
            face(
                "Sapling spending key",
                Some(encode_extended_spending_key(
                    zcash_keys::constants::hrp_sapling_extended_spending_key(net),
                    xsk,
                )),
            ),
            face(
                "Sapling viewing key",
                Some(encode_extended_full_viewing_key(
                    zcash_keys::constants::hrp_sapling_extended_full_viewing_key(net),
                    &xsk.to_extended_full_viewing_key(),
                )),
            ),
        ),
        (None, Some(_)) => {
            // Only the diversifiable key is stored: the extended key's depth,
            // tag and chain code are gone with the import, so the viewing face
            // is the Sapling-only unified key, which imports back the same.
            let ufvk = get_account_ufvk(network, connection, account, POOL_SAPLING).await?;
            let mut viewing = face("Sapling viewing key", Some(ufvk));
            viewing.note = Some(SAPLING_UNIFIED_NOTE.to_string());
            node("Sapling", face("Sapling spending key", None), viewing)
        }
        (None, None) => node(
            "Sapling",
            face("Sapling spending key", None),
            noted("Sapling viewing key", CARRIED_NOTE),
        ),
    };
    Ok(n)
}

fn orchard_node(okeys: &OrchardKeys, frost: bool) -> KeyNode {
    let _ = okeys;
    let secret_note = if frost {
        ORCHARD_FROST_NOTE
    } else {
        ORCHARD_SECRET_NOTE
    };
    node(
        "Orchard",
        noted("Orchard spending key", secret_note),
        noted("Orchard viewing key", CARRIED_NOTE),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        account::new_account,
        api::account::NewAccount,
        pay::pool::ALL_POOLS,
    };
    use bip32::ExtendedPublicKey;
    use secp256k1::PublicKey;
    use sqlx::Connection as _;
    use zcash_keys::keys::UnifiedSpendingKey;
    use zcash_transparent::keys::NonHardenedChildIndex;
    use zip32::AccountId;

    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const NETWORK: Network = Network::Main;

    async fn open_db() -> SqliteConnection {
        let mut db = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        crate::db::create_schema(&mut db).await.unwrap();
        db
    }

    async fn account(db: &mut SqliteConnection, key: &str) -> u32 {
        let na = NewAccount {
            icon: None,
            name: "a".into(),
            restore: true,
            key: key.into(),
            passphrase: None,
            fingerprint: None,
            aindex: 0,
            birth: Some(1),
            folder: String::new(),
            pools: Some(ALL_POOLS),
            use_internal: None,
            internal: false,
            hw: 0,
        };
        new_account(&NETWORK, db, &na).await.unwrap()
    }

    async fn tree(db: &mut SqliteConnection, account: u32) -> KeyNode {
        get_account_key_tree(&NETWORK, db, account)
            .await
            .unwrap()
            .expect("a software account has a key tree")
    }

    fn usk() -> UnifiedSpendingKey {
        let seed = bip39::Mnemonic::from_str(MNEMONIC).unwrap().to_seed("");
        UnifiedSpendingKey::from_seed(&NETWORK, &seed, AccountId::ZERO).unwrap()
    }

    fn child<'a>(n: &'a KeyNode, name: &str) -> &'a KeyNode {
        n.children
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("no child {name} under {}", n.name))
    }

    fn names(n: &KeyNode) -> Vec<&str> {
        n.children.iter().map(|c| c.name.as_str()).collect()
    }

    fn sapling_sk_string() -> String {
        encode_extended_spending_key(
            zcash_keys::constants::hrp_sapling_extended_spending_key(NETWORK.network_type()),
            usk().sapling(),
        )
    }

    #[allow(deprecated)]
    fn sapling_vk_string() -> String {
        encode_extended_full_viewing_key(
            zcash_keys::constants::hrp_sapling_extended_full_viewing_key(NETWORK.network_type()),
            &usk().sapling().to_extended_full_viewing_key(),
        )
    }

    fn xprv_string() -> String {
        let seed = bip39::Mnemonic::from_str(MNEMONIC).unwrap().to_seed("");
        ExtendedPrivateKey::<SecretKey>::derive_from_path(&seed, &"m/44'/133'/0'".parse().unwrap())
            .unwrap()
            .to_extended_key(Prefix::XPRV)
            .to_string()
    }

    fn external_sk(index: u32) -> SecretKey {
        usk()
            .transparent()
            .derive_external_secret_key(NonHardenedChildIndex::from_index(index).unwrap())
            .unwrap()
    }

    /// A seed account's root is the phrase; its pool keys and addresses
    /// derive from it, and the viewing faces are the matching viewing keys.
    #[tokio::test]
    async fn a_seed_account_shows_the_phrase_and_what_derives_from_it() {
        let mut db = open_db().await;
        let id = account(&mut db, MNEMONIC).await;
        let root = tree(&mut db, id).await;

        assert!(root.summary);
        assert_eq!(root.secret.value.as_deref(), Some(MNEMONIC));
        assert_eq!(
            root.viewing.value.as_deref(),
            Some(usk().to_unified_full_viewing_key().encode(&NETWORK).unwrap().as_str())
        );
        // No passphrase row: the account has none.
        assert_eq!(names(&root), ["Account index", "Transparent", "Sapling", "Orchard"]);

        let index = child(&root, "Account index");
        assert!(index.summary);
        assert_eq!(index.secret.value.as_deref(), Some("0"));
        assert_eq!(index.viewing.value, None, "an index has no viewing face");

        let t = child(&root, "Transparent");
        assert!(!t.summary);
        assert_eq!(t.secret.value.as_deref(), Some(xprv_string().as_str()));
        let xpub = ExtendedPublicKey::<PublicKey>::from_str(t.viewing.value.as_deref().unwrap()).unwrap();
        assert_eq!(xpub.attrs().depth, 3);
        assert_eq!(
            xpub.public_key().serialize(),
            usk().transparent().to_account_pubkey().serialize()[32..]
        );

        // Every stored address, external then change, with its WIF and key.
        assert_eq!(names(t), ["External #0", "Change #0"]);
        let a0 = child(t, "External #0");
        let wif = crate::bip38::import_tsk(a0.secret.value.as_deref().unwrap()).unwrap();
        assert_eq!(wif.0, external_sk(0));
        assert!(!wif.1, "a derived key is compressed");
        let zpk = a0.viewing.value.as_deref().unwrap();
        assert!(zpk.starts_with("zpk1"));
        let (hrp, pk) = crate::key::decode_bech32::<Bech32>(zpk).unwrap();
        assert_eq!(hrp, Hrp::parse("zpk").unwrap());
        assert_eq!(pk, external_sk(0).public_key().serialize());
        assert!(a0.detail.as_deref().unwrap().starts_with("t1"), "{:?}", a0.detail);

        let s = child(&root, "Sapling");
        assert_eq!(s.secret.value.as_deref(), Some(sapling_sk_string().as_str()));
        assert_eq!(s.viewing.value.as_deref(), Some(sapling_vk_string().as_str()));
        assert!(s.children.is_empty());

        // Orchard is never encoded: both faces are N/A with a note.
        let o = child(&root, "Orchard");
        assert_eq!(o.secret.value, None);
        assert_eq!(o.secret.note.as_deref(), Some(ORCHARD_SECRET_NOTE));
        assert_eq!(o.viewing.value, None);
        assert_eq!(o.viewing.note.as_deref(), Some(CARRIED_NOTE));
    }

    /// An xprv account's root is the xprv itself, with its addresses below.
    #[tokio::test]
    async fn an_xprv_account_shows_the_xprv_it_was_made_from() {
        let mut db = open_db().await;
        let id = account(&mut db, &xprv_string()).await;
        let root = tree(&mut db, id).await;

        assert_eq!(root.name, "Transparent");
        assert!(root.summary);
        assert_eq!(root.secret.value.as_deref(), Some(xprv_string().as_str()));
        assert_eq!(
            root.viewing.value.as_deref(),
            Some(
                ExtendedPrivateKey::<SecretKey>::from_str(&xprv_string())
                    .unwrap()
                    .public_key()
                    .to_extended_key(Prefix::XPUB)
                    .to_string()
                    .as_str()
            ),
            "the xpub of the same key, derivation data included"
        );
        assert_eq!(names(&root), ["External #0", "Change #0"]);
        let wif = crate::bip38::import_tsk(child(&root, "External #0").secret.value.as_deref().unwrap()).unwrap();
        assert_eq!(wif.0, external_sk(0));
    }

    /// A WIF account is a single address: the root is the WIF as imported,
    /// compression preserved, with its public key as the viewing face.
    #[tokio::test]
    async fn a_wif_account_shows_the_wif_it_was_made_from() {
        let sk = external_sk(0);
        for uncompressed in [false, true] {
            let wif = export_tsk(&TSecretKey(sk, uncompressed));
            let mut db = open_db().await;
            let id = account(&mut db, &wif).await;
            let root = tree(&mut db, id).await;

            assert_eq!(root.name, "Transparent");
            assert_eq!(root.secret.value.as_deref(), Some(wif.as_str()), "uncompressed={uncompressed}");
            let (_, pk) = crate::key::decode_bech32::<Bech32>(root.viewing.value.as_deref().unwrap()).unwrap();
            let expected = if uncompressed {
                sk.public_key().serialize_uncompressed().to_vec()
            } else {
                sk.public_key().serialize().to_vec()
            };
            assert_eq!(pk, expected);
            assert!(root.detail.is_some());
            assert!(root.children.is_empty());
        }
    }

    /// A Sapling spending key account shows the key and its exact extended
    /// viewing key; a Sapling viewing key account has only the viewing face.
    #[tokio::test]
    async fn sapling_key_accounts_show_their_key() {
        let mut db = open_db().await;
        let id = account(&mut db, &sapling_sk_string()).await;
        let root = tree(&mut db, id).await;
        assert_eq!(root.name, "Sapling");
        assert_eq!(root.secret.value.as_deref(), Some(sapling_sk_string().as_str()));
        assert_eq!(root.viewing.value.as_deref(), Some(sapling_vk_string().as_str()));
        assert!(root.children.is_empty());

        let mut db = open_db().await;
        let id = account(&mut db, &sapling_vk_string()).await;
        let root = tree(&mut db, id).await;
        assert_eq!(root.secret.value, None);
        let ufvk = root.viewing.value.as_deref().unwrap();
        assert!(ufvk.starts_with("uview1"), "{ufvk}");
        assert_eq!(root.viewing.note.as_deref(), Some(SAPLING_UNIFIED_NOTE));
        let decoded = zcash_keys::keys::UnifiedFullViewingKey::decode(&NETWORK, ufvk).unwrap();
        assert_eq!(
            decoded.sapling().unwrap().to_bytes(),
            usk().sapling().to_diversifiable_full_viewing_key().to_bytes()
        );
        assert!(decoded.p2pkh().is_none() && decoded.orchard().is_none());
    }

    /// Viewing-only unified accounts have no secret face anywhere; the root
    /// is the unified key and the pool nodes carry their bare viewing keys.
    #[tokio::test]
    async fn unified_viewing_accounts_show_the_unified_key() {
        let ufvk = usk().to_unified_full_viewing_key();
        let mut db = open_db().await;
        let id = account(&mut db, &ufvk.encode(&NETWORK).unwrap()).await;
        let root = tree(&mut db, id).await;
        assert_eq!(root.name, "Unified key");
        assert_eq!(root.secret.value, None);
        assert_eq!(root.viewing.value.as_deref(), Some(ufvk.encode(&NETWORK).unwrap().as_str()));
        assert_eq!(names(&root), ["Transparent", "Sapling", "Orchard"]);
        let t = child(&root, "Transparent");
        assert_eq!(t.secret.value, None);
        assert!(t.viewing.value.as_deref().unwrap().starts_with("xpub"));
        assert_eq!(child(t, "External #0").secret.value, None);
        assert!(child(t, "External #0").viewing.value.as_deref().unwrap().starts_with("zpk1"));
        assert!(child(&root, "Sapling").viewing.value.as_deref().unwrap().starts_with("uview1"));
        assert_eq!(child(&root, "Orchard").viewing.note.as_deref(), Some(CARRIED_NOTE));

        let uivk = ufvk.to_unified_incoming_viewing_key();
        let mut db = open_db().await;
        let id = account(&mut db, &uivk.encode(&NETWORK).unwrap()).await;
        let root = tree(&mut db, id).await;
        assert_eq!(root.viewing.label, "Unified incoming viewing key");
        assert_eq!(root.viewing.value.as_deref(), Some(uivk.encode(&NETWORK).unwrap().as_str()));
        let t = child(&root, "Transparent");
        assert_eq!(t.viewing.label, "Transparent xpub, external chain");
        assert_eq!(child(&root, "Sapling").viewing.note.as_deref(), Some(CARRIED_NOTE));
    }
}
