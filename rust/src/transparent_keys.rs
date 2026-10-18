//! Where an account's transparent addresses come from.
//!
//! An account derives transparent addresses either from its extended public
//! key (and signs with its extended private key when it has one) or by asking
//! a Ledger device. Every place that needs an address at a `(scope, dindex)`
//! goes through [`TransparentKeySource::derive`], so the software-or-device
//! decision is made once, here, and the callers (the address sweep, change
//! address pairing, next-address generation, account creation) only speak in
//! scopes and indexes.
//!
//! Scope 0 is the external (receiving) chain, scope 1 the internal (change)
//! chain, as in BIP 44.

use anyhow::{anyhow, Result};
use sqlx::SqliteConnection;
use zcash_keys::encoding::AddressCodec;
use zcash_transparent::keys::{AccountPrivKey, AccountPubKey, ExternalIvk, IncomingViewingKey as _};

use crate::account::{
    derive_transparent_address, derive_transparent_address_from_ivk, derive_transparent_sk,
    get_transparent_keys,
};
use crate::api::coin::Network;
use crate::db::{get_account_aindex, get_account_hw, store_account_transparent_addr};
use crate::ledger::LedgerHandle;

/// A derived transparent address and the keys the wallet holds for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DerivedTransparentAddr {
    /// The secret key, when the account has the spending key.
    pub sk: Option<Vec<u8>>,
    pub pk: Vec<u8>,
    /// Encoded for the network.
    pub address: String,
}

pub enum TransparentKeySource {
    /// Extended keys held by the wallet.
    Software {
        xvk: AccountPubKey,
        xsk: Option<AccountPrivKey>,
    },
    /// The external chain's incoming key of an incoming-only account:
    /// receiving addresses only, no change chain, no signing.
    Incoming { ivk: ExternalIvk },
    /// A Ledger device deriving for ZIP-32 account `aindex`.
    Ledger { handle: LedgerHandle, aindex: u32 },
}

impl TransparentKeySource {
    pub fn software(xvk: AccountPubKey, xsk: Option<AccountPrivKey>) -> Self {
        TransparentKeySource::Software { xvk, xsk }
    }

    /// The account's key source, or `None` when the account has no
    /// transparent key and is not on a device.
    pub async fn try_load(conn: &mut SqliteConnection, account: u32) -> Result<Option<Self>> {
        let (xsk, xvk) = get_transparent_keys(conn, account).await?;
        if let Some(xvk) = xvk {
            return Ok(Some(TransparentKeySource::Software { xvk, xsk }));
        }
        let ivk: Option<(Option<Vec<u8>>,)> =
            sqlx::query_as("SELECT ivk FROM transparent_accounts WHERE account = ?")
                .bind(account)
                .fetch_optional(&mut *conn)
                .await?;
        if let Some((Some(ivk),)) = ivk {
            let ivk = ExternalIvk::deserialize(&ivk.try_into().unwrap())?;
            return Ok(Some(TransparentKeySource::Incoming { ivk }));
        }
        let hw = get_account_hw(conn, account).await?;
        if hw == 0 {
            return Ok(None);
        }
        let aindex = get_account_aindex(conn, account).await?;
        Ok(Some(TransparentKeySource::Ledger {
            handle: LedgerHandle::from_hw(hw),
            aindex,
        }))
    }

    /// The account's key source; an account without one is an error.
    pub async fn load(conn: &mut SqliteConnection, account: u32) -> Result<Self> {
        Self::try_load(conn, account)
            .await?
            .ok_or_else(|| anyhow!("account {account} has no transparent key"))
    }

    pub fn is_software(&self) -> bool {
        matches!(self, TransparentKeySource::Software { .. })
    }

    /// The address at `(scope, dindex)`. A device may be slow or refuse.
    pub async fn derive(
        &self,
        network: &Network,
        scope: u32,
        dindex: u32,
    ) -> Result<DerivedTransparentAddr> {
        match self {
            TransparentKeySource::Software { xvk, xsk } => {
                let sk = xsk
                    .as_ref()
                    .map(|xsk| derive_transparent_sk(xsk, scope, dindex))
                    .transpose()?;
                let (pk, address) = derive_transparent_address(xvk, scope, dindex, false)?;
                Ok(DerivedTransparentAddr {
                    sk,
                    pk,
                    address: address.encode(network),
                })
            }
            TransparentKeySource::Incoming { ivk } => {
                if scope != 0 {
                    return Err(anyhow!(
                        "an incoming-only account has no internal (change) addresses"
                    ));
                }
                let (pk, address) = derive_transparent_address_from_ivk(ivk, dindex)?;
                Ok(DerivedTransparentAddr {
                    sk: None,
                    pk,
                    address: address.encode(network),
                })
            }
            TransparentKeySource::Ledger { handle, aindex } => {
                let (pk, address) = handle
                    .get_transparent_pubkey(network, *aindex, scope, dindex)
                    .await?;
                Ok(DerivedTransparentAddr {
                    sk: None,
                    pk,
                    address: address.encode(network),
                })
            }
        }
    }
}

/// Make sure the account has its address at `(scope, dindex)` stored,
/// deriving it when missing. Returns whether a row was added.
pub async fn ensure_transparent_address(
    conn: &mut SqliteConnection,
    network: &Network,
    source: &TransparentKeySource,
    account: u32,
    scope: u32,
    dindex: u32,
) -> Result<bool> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM transparent_address_accounts
        WHERE account = ?1 AND scope = ?2 AND dindex = ?3)",
    )
    .bind(account)
    .bind(scope)
    .bind(dindex)
    .fetch_one(&mut *conn)
    .await?;
    if exists {
        return Ok(false);
    }
    let addr = source.derive(network, scope, dindex).await?;
    store_account_transparent_addr(
        conn,
        account,
        scope,
        dindex,
        addr.sk,
        &addr.pk,
        &addr.address,
        false,
    )
    .await
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::db::create_schema;
    use crate::ledger::mock::StubLedger;
    use sqlx::Connection;
    use zcash_keys::keys::UnifiedSpendingKey;
    use zip32::AccountId;

    pub(crate) fn software_keys() -> (AccountPubKey, AccountPrivKey) {
        let seed = bip39::Mnemonic::parse(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .unwrap()
        .to_seed("");
        let usk = UnifiedSpendingKey::from_seed(&Network::Main, &seed, AccountId::ZERO).unwrap();
        let xsk = usk.transparent().clone();
        (xsk.to_account_pubkey(), xsk)
    }

    #[tokio::test]
    async fn software_source_matches_the_primitives() -> Result<()> {
        let (xvk, xsk) = software_keys();
        let with_sk = TransparentKeySource::software(xvk.clone(), Some(xsk.clone()));
        let view_only = TransparentKeySource::software(xvk.clone(), None);
        for (scope, dindex) in [(0, 0), (0, 7), (1, 3)] {
            let (pk, address) = derive_transparent_address(&xvk, scope, dindex, false)?;
            let expected = DerivedTransparentAddr {
                sk: Some(derive_transparent_sk(&xsk, scope, dindex)?),
                pk: pk.clone(),
                address: address.encode(&Network::Main),
            };
            assert_eq!(
                with_sk.derive(&Network::Main, scope, dindex).await?,
                expected
            );
            assert_eq!(
                view_only.derive(&Network::Main, scope, dindex).await?,
                DerivedTransparentAddr {
                    sk: None,
                    ..expected
                }
            );
        }
        Ok(())
    }

    /// The incoming source derives the external chain exactly as the
    /// account key does, refuses the change chain, and loads for an account
    /// that stores an incoming key alone.
    #[tokio::test]
    async fn incoming_source_derives_the_external_chain_only() -> Result<()> {
        let (xvk, _) = software_keys();
        let ivk = xvk.derive_external_ivk().unwrap();
        let full = TransparentKeySource::software(xvk, None);
        let incoming = TransparentKeySource::Incoming { ivk: ivk.clone() };
        for dindex in [0, 7] {
            assert_eq!(
                incoming.derive(&Network::Main, 0, dindex).await?,
                full.derive(&Network::Main, 0, dindex).await?
            );
        }
        assert!(incoming.derive(&Network::Main, 1, 0).await.is_err());
        assert!(!incoming.is_software());

        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        sqlx::query("INSERT INTO accounts(id_account, name, aindex, dindex, def_dindex, birth, position, use_internal, hidden, saved, hw)
            VALUES (1, 'a', 0, 0, 0, 1, 1, FALSE, FALSE, FALSE, 0)")
            .execute(&mut db)
            .await?;
        crate::db::init_account_transparent(&mut db, 1, 1).await?;
        crate::db::store_account_transparent_ivk(&mut db, 1, &ivk).await?;
        assert!(matches!(
            TransparentKeySource::load(&mut db, 1).await?,
            TransparentKeySource::Incoming { .. }
        ));
        Ok(())
    }

    #[tokio::test]
    async fn ledger_source_reports_the_device_error() {
        let source = TransparentKeySource::Ledger {
            handle: LedgerHandle::Stub(StubLedger::no_support()),
            aindex: 0,
        };
        let err = source.derive(&Network::Main, 0, 0).await.unwrap_err();
        assert!(err.to_string().contains("no Ledger support"), "{err}");
    }

    #[tokio::test]
    async fn ensure_stores_a_missing_address_once() -> Result<()> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        let (xvk, xsk) = software_keys();
        let source = TransparentKeySource::software(xvk, Some(xsk));
        assert!(ensure_transparent_address(&mut db, &Network::Main, &source, 1, 1, 4).await?);
        assert!(!ensure_transparent_address(&mut db, &Network::Main, &source, 1, 1, 4).await?);
        let rows: Vec<(u32, u32, String)> = sqlx::query_as(
            "SELECT scope, dindex, address FROM transparent_address_accounts WHERE account = 1",
        )
        .fetch_all(&mut db)
        .await?;
        let expected = source.derive(&Network::Main, 1, 4).await?.address;
        assert_eq!(rows, vec![(1, 4, expected)]);
        Ok(())
    }

    #[tokio::test]
    async fn load_reflects_the_account_keys() -> Result<()> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        sqlx::query("INSERT INTO accounts(id_account, name, aindex, dindex, def_dindex, birth, position, use_internal, hidden, saved, hw)
            VALUES (1, 'a', 0, 0, 0, 1, 1, FALSE, FALSE, FALSE, 0), (2, 'b', 3, 0, 0, 1, 2, FALSE, FALSE, FALSE, 1)")
            .execute(&mut db)
            .await?;
        assert!(TransparentKeySource::try_load(&mut db, 1).await?.is_none());
        assert!(TransparentKeySource::load(&mut db, 1).await.is_err());
        match TransparentKeySource::try_load(&mut db, 2).await? {
            Some(TransparentKeySource::Ledger { aindex, .. }) => assert_eq!(aindex, 3),
            _ => panic!("hardware account should load a Ledger source"),
        }
        let (xvk, _) = software_keys();
        crate::db::init_account_transparent(&mut db, 1, 1).await?;
        crate::db::store_account_transparent_vk(&mut db, 1, &xvk).await?;
        assert!(TransparentKeySource::load(&mut db, 1).await?.is_software());
        Ok(())
    }
}
