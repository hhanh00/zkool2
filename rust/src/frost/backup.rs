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
use bech32::{primitives::decode::CheckedHrpstring, Bech32m, Hrp};
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
        keys::{group_fvk, DKG_VERSION_0, DKG_VERSION_2},
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

    /// Rejects a wrong prefix, a bad checksum, a wrong length and an
    /// unknown format version.
    pub fn decode(s: &str) -> Result<Self> {
        let checked = CheckedHrpstring::new::<Bech32m>(s.trim())
            .map_err(|e| anyhow!("not a valid zkfrost backup string: {e}"))?;
        if checked.hrp().to_lowercase() != HRP {
            bail!("not a zkfrost backup string");
        }
        let payload: Vec<u8> = checked.byte_iter().collect();
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

/// Which derivation a viewing key follows for a given `sk`.
fn match_derivation(fvk: &FullViewingKey, sk: &SpendingKey) -> Result<u8> {
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

/// Restore a frost account from a backup string and the group's unified
/// full viewing key. Returns the new account and its DKG version (0 for a
/// legacy group, 2 for a ZIP 2005 one).
pub async fn restore_backup(
    network: &Network,
    connection: &mut SqliteConnection,
    name: &str,
    backup: &str,
    ufvk: &str,
) -> Result<(u32, u8)> {
    let backup = FrostBackup::decode(backup)?;
    let ufvk = UnifiedFullViewingKey::decode(network, ufvk.trim())
        .map_err(|e| anyhow!("invalid unified full viewing key: {e}"))?;
    let fvk = ufvk
        .orchard()
        .context("the viewing key has no Orchard key; a frost group's key is Orchard-only")?;
    let sk = Option::from(SpendingKey::from_bytes(backup.sk))
        .context("the backup string carries an invalid sk")?;
    let dkg_version = match_derivation(fvk, &sk)?;

    // The key package, from what the string and the key hold.
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

    // Birth at Orchard activation: the notes are wherever the group has
    // received since it was formed, and a rescan from there finds them.
    let birth: u32 = network
        .activation_height(NetworkUpgrade::Nu5)
        .context("Orchard is not active on this network")?
        .into();

    let mut tx = connection.begin().await?;
    let account = store_account_metadata(&mut *tx, name, &None, &None, birth, false, false).await?;
    init_account_orchard(network, &mut *tx, account, birth).await?;
    store_account_orchard_vk(&mut *tx, account, fvk).await?;
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
    tx.commit().await?;
    Ok((account, dkg_version))
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
            let (account, restored_version) =
                restore_backup(&network, &mut db, "group", &b, &ufvk_of(&fvk))
                    .await
                    .unwrap();
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
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("does not belong"), "{err}");
    }
}
