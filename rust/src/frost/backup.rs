//! The member backup string, and the restore from it.
//!
//! One bech32m string, human-readable part `zkfrost`, 68 bytes of payload:
//! format version (1, value 1), participant id (1), threshold t (1), group
//! size n (1), the FROST signing share (32), and `sk` (32), the group's
//! shared Orchard spending key. No network and no mailbox seed: the string
//! works on any chain the group lives on, and a restored member gets a
//! fresh mailbox at its first signing session.
//!
//! Restore takes the string and the group's unified full viewing key. `ak`
//! comes from the key; `nk` and `rivk` are recomputed from `sk` and `ak`
//! under both derivations and compared with the key's own, which both
//! decides the account's DKG version and catches a string from another
//! group. The key package is rebuilt from the share, the id, t and `ak`;
//! no peer addresses, round-0 keys or public key package are needed,
//! since the signing transport no longer relies on them.
//!
//! The string is as sensitive as the share it carries. For a legacy group
//! it is a device-loss backup only: it does not make the group's notes
//! quantum recoverable.

use anyhow::{anyhow, bail, Context, Result};
use bech32::{Bech32m, Hrp};
use orchard::keys::{FullViewingKey, SpendingKey};
use reddsa_legacy::frost::redpallas::{
    frost::{
        keys::{KeyPackage, SigningShare, VerifyingShare},
        VerifyingKey,
    },
    Identifier,
};
use sqlx::{Connection as _, SqliteConnection};
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_protocol::consensus::{NetworkUpgrade, Parameters as _};

use crate::{
    api::coin::Network,
    db::{init_account_orchard, store_account_metadata, store_account_orchard_vk},
    frost::{
        keys::{group_fvk, is_recoverable, DKG_VERSION_0, DKG_VERSION_2},
        protocol::group_sk,
        FrostBytes, P,
    },
};

pub const HRP: &str = "zkfrost";
pub const FORMAT_VERSION: u8 = 1;
const PAYLOAD_LEN: usize = 68;

/// What the string carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrostBackup {
    pub id: u8,
    pub t: u8,
    pub n: u8,
    pub share: [u8; 32],
    pub sk: [u8; 32],
}

impl FrostBackup {
    pub fn encode(&self) -> Result<String> {
        let mut payload = Vec::with_capacity(PAYLOAD_LEN);
        payload.push(FORMAT_VERSION);
        payload.push(self.id);
        payload.push(self.t);
        payload.push(self.n);
        payload.extend_from_slice(&self.share);
        payload.extend_from_slice(&self.sk);
        Ok(bech32::encode::<Bech32m>(Hrp::parse(HRP)?, &payload)?)
    }

    /// Rejects a wrong prefix, a bad checksum, non-canonical padding, a
    /// wrong length and an unknown format version.
    pub fn decode(s: &str) -> Result<Self> {
        let (hrp, payload) = crate::key::decode_bech32::<Bech32m>(s.trim())
            .map_err(|e| anyhow!("not a valid zkfrost backup string: {e}"))?;
        if hrp.to_lowercase() != HRP {
            bail!("not a zkfrost backup string");
        }
        if payload.len() != PAYLOAD_LEN {
            bail!(
                "invalid zkfrost backup string: expected {PAYLOAD_LEN} bytes, got {}",
                payload.len()
            );
        }
        if payload[0] != FORMAT_VERSION {
            bail!(
                "unsupported zkfrost backup format version {}; this app supports version {FORMAT_VERSION}",
                payload[0]
            );
        }
        let (id, t, n) = (payload[1], payload[2], payload[3]);
        if n == 0 || id == 0 || id > n || t == 0 || t > n {
            bail!("invalid zkfrost backup string: participant {id}, threshold {t} of {n}");
        }
        Ok(Self {
            id,
            t,
            n,
            share: payload[4..36].try_into().unwrap(),
            sk: payload[36..68].try_into().unwrap(),
        })
    }
}

/// The backup string of a frost account, whatever its DKG version. A legacy
/// account's `sk` is rederived from its stored addresses.
pub async fn export_backup(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<String> {
    let (id, n, t): (u8, u8, u8) =
        sqlx::query_as("SELECT id, n, t FROM dkg_params WHERE account = ?")
            .bind(account)
            .fetch_optional(&mut *connection)
            .await?
            .context("not a frost account")?;
    let (key_pkg,): (Vec<u8>,) = sqlx::query_as(
        "SELECT key_pkg FROM dkg_state WHERE account = ? AND key_pkg IS NOT NULL",
    )
    .bind(account)
    .fetch_optional(&mut *connection)
    .await?
    .context("the frost account has no key package: the DKG did not complete")?;
    let key_pkg = KeyPackage::<P>::from_bytes(&key_pkg)?;
    let share: [u8; 32] = key_pkg
        .signing_share()
        .serialize()
        .try_into()
        .map_err(|_| anyhow!("signing share is not 32 bytes"))?;
    let sk = group_sk(network, connection, account).await?;
    FrostBackup {
        id,
        t,
        n,
        share,
        sk: *sk.to_bytes(),
    }
    .encode()
}

/// Which derivation a viewing key follows for a given `sk`: the generation
/// a restore defaults to. An error when neither matches, i.e. the string
/// is not this group's.
pub fn match_derivation(fvk: &FullViewingKey, sk: &SpendingKey) -> Result<u8> {
    let ak: [u8; 32] = fvk.to_bytes()[..32].try_into().unwrap();
    if group_fvk(DKG_VERSION_0, sk, &ak)? == *fvk {
        Ok(DKG_VERSION_0)
    } else if group_fvk(DKG_VERSION_2, sk, &ak)? == *fvk {
        Ok(DKG_VERSION_2)
    } else {
        bail!(
            "the backup string does not belong to this viewing key's group: \
             neither the legacy nor the ZIP 2005 derivation of nk and rivk from \
             its sk matches the key"
        )
    }
}

/// Which generation a backup string and a group viewing key match: 0 for a
/// legacy key, 2 for a ZIP 2005 one. An error when the string is not the
/// group's.
pub fn match_backup(network: &Network, backup: &str, ufvk: &str) -> Result<u8> {
    let backup = FrostBackup::decode(backup)?;
    let fvk = parse_group_fvk(network, ufvk)?;
    let sk = Option::from(SpendingKey::from_bytes(backup.sk))
        .context("the backup string carries an invalid sk")?;
    match_derivation(&fvk, &sk)
}

fn parse_group_fvk(network: &Network, ufvk: &str) -> Result<FullViewingKey> {
    let ufvk = UnifiedFullViewingKey::decode(network, ufvk.trim())
        .map_err(|e| anyhow!("invalid unified full viewing key: {e}"))?;
    ufvk.orchard()
        .cloned()
        .context("the viewing key has no Orchard key; a frost group's key is Orchard-only")
}

/// Restore frost accounts from a backup string and the group's unified
/// full viewing key: one account per requested generation (below 2 is the
/// legacy derivation, 2 is ZIP 2005), since `sk` and the key's `ak` derive
/// both. The key must match one of the derivations, which proves the
/// string belongs to the group; the generation it matches is what a caller
/// with no preference should ask for (see [`match_backup`]). Born at
/// `birth`, or at Orchard activation so a rescan finds every note. Returns
/// (account, version) per account, legacy first.
pub async fn restore_backup(
    network: &Network,
    connection: &mut SqliteConnection,
    name: &str,
    backup: &str,
    ufvk: &str,
    versions: &[u8],
    birth: Option<u32>,
) -> Result<Vec<(u32, u8)>> {
    let backup = FrostBackup::decode(backup)?;
    let fvk = parse_group_fvk(network, ufvk)?;
    let sk = Option::from(SpendingKey::from_bytes(backup.sk))
        .context("the backup string carries an invalid sk")?;
    match_derivation(&fvk, &sk)?;
    let mut wanted: Vec<u8> = versions
        .iter()
        .map(|v| if is_recoverable(*v) { DKG_VERSION_2 } else { DKG_VERSION_0 })
        .collect();
    wanted.sort_unstable();
    wanted.dedup();
    if wanted.is_empty() {
        bail!("no generation selected to restore");
    }

    let ak: [u8; 32] = fvk.to_bytes()[..32].try_into().unwrap();
    let identifier = Identifier::try_from(backup.id as u16)?;
    let signing_share = SigningShare::<P>::deserialize(&backup.share)
        .map_err(|e| anyhow!("the backup string carries an invalid share: {e}"))?;
    let verifying_share = VerifyingShare::from(signing_share);
    let verifying_key = VerifyingKey::deserialize(&ak)?;
    let key_pkg = KeyPackage::new(
        identifier,
        signing_share,
        verifying_share,
        verifying_key,
        backup.t as u16,
    );

    // Birth at Orchard activation unless told otherwise: the notes are
    // wherever the group has received since it was formed, and a rescan
    // from there finds them.
    let activation: u32 = network
        .activation_height(NetworkUpgrade::Nu5)
        .context("Orchard is not active on this network")?
        .into();
    let birth = birth.unwrap_or(activation).max(activation);

    let mut tx = connection.begin().await?;
    let mut restored = Vec::with_capacity(wanted.len());
    for dkg_version in wanted {
        let fvk = group_fvk(dkg_version, &sk, &ak)?;
        // The names an upgrade would have given the pair.
        let account_name = if is_recoverable(dkg_version) && restored.len() == 1 {
            format!("{name} (ZIP 2005)")
        } else {
            name.to_string()
        };
        let account =
            store_account_metadata(&mut *tx, &account_name, &None, &None, birth, false, false)
                .await?;
        init_account_orchard(network, &mut *tx, account, birth).await?;
        store_account_orchard_vk(&mut *tx, account, &fvk).await?;
        // `sk` is kept for both versions: a restored member has no stored
        // addresses to rederive it from, and the signing transport needs it.
        sqlx::query(
            "INSERT INTO dkg_params(account, id, n, t, seed, birth_height, name, dkg_version, sk)
            VALUES (?, ?, ?, ?, '', ?, ?, ?, ?)",
        )
        .bind(account)
        .bind(backup.id)
        .bind(backup.n)
        .bind(backup.t)
        .bind(birth)
        .bind(name)
        .bind(dkg_version)
        .bind(sk.to_bytes().to_vec())
        .execute(&mut *tx)
        .await?;
        sqlx::query("INSERT INTO dkg_state(account, key_pkg) VALUES (?, ?)")
            .bind(account)
            .bind(key_pkg.to_bytes()?)
            .execute(&mut *tx)
            .await?;
        restored.push((account, dkg_version));
    }
    tx.commit().await?;
    Ok(restored)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_schema;
    use zcash_address::unified::{Encoding as _, Fvk, Ufvk};

    // Vector 0 of frost::keys: an sk and an even-y ak.
    const SK: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    const AK: &str = "262473baab7c8fbf24c39e0b2d2dd4736f7b74d7f3b6d32d1e8c62421368ad0e";
    const OTHER_SK: &str = "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f";

    fn bytes32(hex: &str) -> [u8; 32] {
        hex::decode(hex).unwrap().try_into().unwrap()
    }

    fn backup(sk: &str) -> FrostBackup {
        let mut share = [0u8; 32];
        share[0] = 7; // a small canonical scalar
        FrostBackup {
            id: 2,
            t: 2,
            n: 3,
            share,
            sk: bytes32(sk),
        }
    }

    fn ufvk_of(fvk: &FullViewingKey) -> String {
        let ufvk = Ufvk::try_from_items(
            zcash_address::unified::Revision::R0,
            vec![zcash_address::unified::Uitem::Data(Fvk::Orchard(fvk.to_bytes()))],
        )
        .unwrap();
        UnifiedFullViewingKey::parse(&ufvk)
            .unwrap()
            .encode(&Network::Main)
            .unwrap()
    }

    #[test]
    fn zkfrost_round_trips_and_rejects_bad_strings() {
        let b = backup(SK);
        let s = b.encode().unwrap();
        assert!(s.starts_with("zkfrost1"));
        assert_eq!(FrostBackup::decode(&s).unwrap(), b);
        assert_eq!(FrostBackup::decode(&format!("  {}\n", s.to_uppercase())).unwrap(), b);

        // Non-canonical padding (ZIP 173): the one leftover bit set, and
        // one extra zero character, which leaves six. Both spell the fields
        // of member 1 of group A in the draft ZIP's test vectors (#1466).
        const PADDING_BIT_SET: &str = "zkfrost1qyqsyql3xwkurktqyq2hmycdz7pwxvyjd4kxvtt82n4fq0487auamzzyxx683wrs9swj26072tja0kada33z8ngsl494qndt43ll7garwd3argkjsqp";
        const EXTRA_CHARACTER: &str = "zkfrost1qyqsyql3xwkurktqyq2hmycdz7pwxvyjd4kxvtt82n4fq0487auamzzyxx683wrs9swj26072tja0kada33z8ngsl494qndt43ll7garwd3azqf2e54r";
        for s in [PADDING_BIT_SET, EXTRA_CHARACTER] {
            let err = FrostBackup::decode(s).unwrap_err().to_string();
            assert!(err.contains("not a valid zkfrost backup string"), "{err}");
        }

        // Wrong checksum: one data character changed.
        let mut chars: Vec<char> = s.chars().collect();
        let i = 12;
        chars[i] = if chars[i] == 'q' { 'p' } else { 'q' };
        let corrupted: String = chars.into_iter().collect();
        assert!(FrostBackup::decode(&corrupted).is_err());

        // Wrong prefix, valid checksum.
        let mut payload = vec![FORMAT_VERSION, 2, 2, 3];
        payload.extend_from_slice(&b.share);
        payload.extend_from_slice(&b.sk);
        let other_hrp = bech32::encode::<Bech32m>(Hrp::parse("zkfros").unwrap(), &payload).unwrap();
        assert!(FrostBackup::decode(&other_hrp).is_err());

        // Wrong length, valid checksum.
        let short = bech32::encode::<Bech32m>(Hrp::parse(HRP).unwrap(), &payload[..67]).unwrap();
        assert!(FrostBackup::decode(&short).is_err());

        // Unknown format version.
        payload[0] = 2;
        let v2 = bech32::encode::<Bech32m>(Hrp::parse(HRP).unwrap(), &payload).unwrap();
        assert!(FrostBackup::decode(&v2)
            .unwrap_err()
            .to_string()
            .contains("format version 2"));
    }

    async fn restored_rows(
        db: &mut SqliteConnection,
        account: u32,
    ) -> (Vec<u8>, (u8, u8, u8, u8, Option<Vec<u8>>), Vec<u8>) {
        let (xvk,): (Vec<u8>,) =
            sqlx::query_as("SELECT xvk FROM orchard_accounts WHERE account = ?")
                .bind(account)
                .fetch_one(&mut *db)
                .await
                .unwrap();
        let params = sqlx::query_as(
            "SELECT id, n, t, dkg_version, sk FROM dkg_params WHERE account = ?",
        )
        .bind(account)
        .fetch_one(&mut *db)
        .await
        .unwrap();
        let (key_pkg,): (Vec<u8>,) =
            sqlx::query_as("SELECT key_pkg FROM dkg_state WHERE account = ?")
                .bind(account)
                .fetch_one(&mut *db)
                .await
                .unwrap();
        (xvk, params, key_pkg)
    }

    /// A legacy key restores as a dkg-0 account and a ZIP 2005 key as a
    /// dkg-2 one, each with the group's viewing key, the parameters, sk,
    /// and a key package for the member's share under the group's ak. A
    /// string from another group is refused.
    #[tokio::test]
    async fn restore_picks_the_derivation_and_rebuilds_the_key_package() {
        let network = Network::Main;
        let sk = SpendingKey::from_bytes(bytes32(SK)).unwrap();
        let ak = bytes32(AK);
        let b = backup(SK).encode().unwrap();

        for (version, fvk) in [
            (DKG_VERSION_0, group_fvk(DKG_VERSION_0, &sk, &ak).unwrap()),
            (DKG_VERSION_2, group_fvk(DKG_VERSION_2, &sk, &ak).unwrap()),
        ] {
            let mut db = SqliteConnection::connect("sqlite::memory:").await.unwrap();
            create_schema(&mut db).await.unwrap();
            assert_eq!(match_backup(&network, &b, &ufvk_of(&fvk)).unwrap(), version);
            let restored = restore_backup(&network, &mut db, "group", &b, &ufvk_of(&fvk), &[version], None)
                .await
                .unwrap();
            let [(account, restored_version)] = restored[..] else {
                panic!("one account expected");
            };
            assert_eq!(restored_version, version);

            let (xvk, params, key_pkg) = restored_rows(&mut db, account).await;
            assert_eq!(xvk, fvk.to_bytes().to_vec());
            assert_eq!(params, (2, 3, 2, version, Some(bytes32(SK).to_vec())));
            let key_pkg = KeyPackage::<P>::from_bytes(&key_pkg).unwrap();
            assert_eq!(*key_pkg.identifier(), Identifier::try_from(2u16).unwrap());
            assert_eq!(*key_pkg.min_signers(), 2);
            assert_eq!(key_pkg.verifying_key().serialize().unwrap(), ak.to_vec());
            assert_eq!(
                export_backup(&network, &mut db, account).await.unwrap(),
                b,
                "the restored account exports the same string"
            );
        }

        // A string whose sk does not derive this key's nk and rivk.
        let mut db = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        create_schema(&mut db).await.unwrap();
        let fvk = group_fvk(DKG_VERSION_2, &sk, &ak).unwrap();
        let err = restore_backup(
            &network,
            &mut db,
            "group",
            &backup(OTHER_SK).encode().unwrap(),
            &ufvk_of(&fvk),
            &[DKG_VERSION_2],
            None,
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("does not belong"), "{err}");
    }

    /// Both generations from one string and one key, either key: the
    /// legacy account first under the given name, the ZIP 2005 one named
    /// as an upgrade would name it, both at the requested birth height.
    #[tokio::test]
    async fn restore_can_create_both_generations_from_either_key() {
        let network = Network::Main;
        let sk = SpendingKey::from_bytes(bytes32(SK)).unwrap();
        let ak = bytes32(AK);
        let b = backup(SK).encode().unwrap();
        let legacy_fvk = group_fvk(DKG_VERSION_0, &sk, &ak).unwrap();
        let zip_fvk = group_fvk(DKG_VERSION_2, &sk, &ak).unwrap();

        for key in [&legacy_fvk, &zip_fvk] {
            let mut db = SqliteConnection::connect("sqlite::memory:").await.unwrap();
            create_schema(&mut db).await.unwrap();
            let restored = restore_backup(
                &network,
                &mut db,
                "group",
                &b,
                &ufvk_of(key),
                &[DKG_VERSION_2, DKG_VERSION_0, 1],
                Some(2_500_000),
            )
            .await
            .unwrap();
            assert_eq!(restored.len(), 2);
            assert_eq!(restored[0].1, DKG_VERSION_0);
            assert_eq!(restored[1].1, DKG_VERSION_2);
            let (xvk, params, _) = restored_rows(&mut db, restored[0].0).await;
            assert_eq!(xvk, legacy_fvk.to_bytes().to_vec());
            assert_eq!(params.3, DKG_VERSION_0);
            let (xvk, params, _) = restored_rows(&mut db, restored[1].0).await;
            assert_eq!(xvk, zip_fvk.to_bytes().to_vec());
            assert_eq!(params.3, DKG_VERSION_2);
            let names: Vec<(String, u32)> = sqlx::query_as(
                "SELECT name, birth FROM accounts ORDER BY id_account",
            )
            .fetch_all(&mut db)
            .await
            .unwrap();
            assert_eq!(
                names,
                vec![
                    ("group".to_string(), 2_500_000),
                    ("group (ZIP 2005)".to_string(), 2_500_000)
                ]
            );
            // The pair is a sibling pair, as after an upgrade.
            assert_eq!(
                crate::frost::upgrade::sibling_of(&mut db, restored[0].0).await.unwrap(),
                Some(restored[1].0)
            );
        }
    }
}
