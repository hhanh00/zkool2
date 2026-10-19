//! Everything a transaction builder needs to spend the notes a plan selected:
//! the notes themselves with their merkle paths, the viewing keys, the change
//! addresses, the anchors and the asset names. The solver knows a note by its
//! id, pool and amount; this module turns that into builder inputs.
//!
//! The loader only reads the wallet database. The tree state, which comes
//! from lightwalletd, is handed in as [`TreeEdges`] so the loader can be
//! tested against an in-memory wallet.

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use orchard::NoteVersion;
use ripemd::Ripemd160;
use sapling_crypto::zip32::DiversifiableFullViewingKey;
use secp256k1::PublicKey;
use sha2::{Digest as _, Sha256};
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};
use zcash_transparent::bundle::OutPoint;

use crate::{
    account::{
        get_account_full_address, get_orchard_note, get_orchard_sk, get_orchard_vk,
        get_sapling_note, get_sapling_sk, get_sapling_vk, transparent_change_address,
    },
    api::coin::Network,
    db::{get_account_dindex, get_account_hw, select_account_transparent},
    warp::{
        hasher::{empty_roots, OrchardHasher, SaplingHasher},
        legacy::CommitmentTreeFrontier,
        Edge,
    },
};

use super::plan::PlanOutputs;

/// A note the plan decided to spend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectedNote {
    pub id: u32,
    pub pool: u8,
}

/// What the loader needs to know about a plan: which notes it spends, how
/// orchard notes are to be decoded, and whether change goes to a transparent
/// address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpendSelection {
    /// In builder order: the transparent spends first, then sapling, orchard
    /// and ironwood.
    pub notes: Vec<SelectedNote>,
    pub orchard_note_version: NoteVersion,
    pub transparent_change: bool,
}

impl PlanOutputs {
    /// Reduce the plan to the facts the loader needs.
    pub fn spend_selection(&self) -> SpendSelection {
        let notes = self
            .input_pools
            .iter()
            .flatten()
            .filter(|n| n.is_used())
            .map(|n| SelectedNote {
                id: n.id,
                pool: n.pool,
            })
            .collect();
        SpendSelection {
            notes,
            orchard_note_version: self.orchard_note_version,
            transparent_change: self.change > 0 && self.change_pool == 0,
        }
    }
}

/// The commitment tree edges at the anchor height, and how the stored
/// witnesses relate to them.
pub struct TreeEdges {
    pub sapling: Edge,
    pub orchard: Edge,
    pub ironwood: Edge,
    /// Height the wallet stored its witnesses at (the account checkpoint).
    pub witness_height: u32,
    /// The anchor is older than the checkpoint, so shielded witnesses must be
    /// rewound to the edge position.
    pub rewind: bool,
}

impl TreeEdges {
    pub fn new(
        sapling: &CommitmentTreeFrontier,
        orchard: &CommitmentTreeFrontier,
        ironwood: &CommitmentTreeFrontier,
        witness_height: u32,
        rewind: bool,
    ) -> Self {
        Self {
            sapling: sapling.to_edge(&SaplingHasher::default()),
            orchard: orchard.to_edge(&OrchardHasher::default()),
            ironwood: ironwood.to_edge(&OrchardHasher::default()),
            witness_height,
            rewind,
        }
    }

    pub fn anchors(&self) -> Anchors {
        Anchors {
            sapling: self.sapling.root(&SaplingHasher::default()),
            orchard: self.orchard.root(&OrchardHasher::default()),
            ironwood: self.ironwood.root(&OrchardHasher::default()),
        }
    }
}

/// Tree roots at the anchor height, one per shielded pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Anchors {
    pub sapling: [u8; 32],
    pub orchard: [u8; 32],
    pub ironwood: [u8; 32],
}

/// A transparent coin to spend, with what the PCZT records about it so the
/// signer can derive its key.
#[derive(Clone, Debug)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub amount: u64,
    pub pubkey: PublicKey,
    pub scope: u32,
    pub dindex: u32,
    pub address: String,
    pub uncompressed: bool,
}

impl Utxo {
    /// The public key as it was hashed into the address.
    pub fn pubkey_bytes(&self) -> Vec<u8> {
        if self.uncompressed {
            self.pubkey.serialize_uncompressed().to_vec()
        } else {
            self.pubkey.serialize().to_vec()
        }
    }

    pub fn pubkey_hash(&self) -> [u8; 20] {
        Ripemd160::digest(Sha256::digest(self.pubkey_bytes())).into()
    }
}

pub struct SaplingSpend {
    pub note: sapling_crypto::Note,
    pub scope: u32,
    pub merkle_path: sapling_crypto::MerklePath,
}

pub struct OrchardSpend {
    pub note: orchard::Note,
    pub merkle_path: orchard::tree::MerklePath,
}

/// Builder inputs for one plan. No secret key rides along: `can_sign` records
/// whether the wallet holds the spending key of every spent note.
pub struct SpendMaterial {
    pub transparent: Vec<Utxo>,
    pub sapling: Vec<SaplingSpend>,
    pub orchard: Vec<OrchardSpend>,
    pub ironwood: Vec<OrchardSpend>,
    pub sapling_vk: Option<DiversifiableFullViewingKey>,
    pub orchard_vk: Option<orchard::keys::FullViewingKey>,
    pub can_sign: bool,
    /// The account's full address at its change scope; shielded change and
    /// ZSA change outputs go here.
    pub change_address: String,
    /// The stored transparent change address, when the plan sends change to
    /// pool 0 and the account has a transparent key.
    pub transparent_change_address: Option<String>,
    pub dindex: u32,
    pub hw: u8,
    pub anchors: Anchors,
    /// Names of the assets the wallet knows, attached to orchard outputs.
    pub asset_names: HashMap<[u8; 32], String>,
}

impl SpendMaterial {
    pub async fn load(
        network: &Network,
        connection: &mut SqliteConnection,
        account: u32,
        selection: &SpendSelection,
        edges: &TreeEdges,
    ) -> Result<Self> {
        let dindex = get_account_dindex(connection, account).await?;
        let hw = get_account_hw(&mut *connection, account).await?;
        let (use_internal,): (bool,) =
            sqlx::query_as("SELECT use_internal FROM accounts WHERE id_account = ?")
                .bind(account)
                .fetch_one(&mut *connection)
                .await?;

        let change_scope = if use_internal { 1 } else { 0 };
        let change_address =
            get_account_full_address(network, connection, account, change_scope, hw).await?;
        let tkeys = select_account_transparent(connection, account, dindex).await?;
        let transparent_change_address = if selection.transparent_change && tkeys.xvk.is_some() {
            Some(transparent_change_address(connection, account, use_internal, dindex).await?)
        } else {
            None
        };

        let sapling_vk = get_sapling_vk(connection, account).await?;
        let orchard_vk = get_orchard_vk(connection, account).await?;
        let has_sapling_sk = get_sapling_sk(&mut *connection, account).await?.is_some();
        let has_orchard_sk = get_orchard_sk(&mut *connection, account).await?.is_some();

        let sapling_hasher = SaplingHasher::default();
        let orchard_hasher = OrchardHasher::default();
        let es = edges.sapling.to_auth_path(&sapling_hasher);
        let eo = edges.orchard.to_auth_path(&orchard_hasher);
        let ei = edges.ironwood.to_auth_path(&orchard_hasher);
        let ers = empty_roots(&sapling_hasher);
        let ero = empty_roots(&orchard_hasher);

        let mut material = SpendMaterial {
            transparent: vec![],
            sapling: vec![],
            orchard: vec![],
            ironwood: vec![],
            sapling_vk,
            orchard_vk,
            can_sign: true,
            change_address,
            transparent_change_address,
            dindex,
            hw,
            anchors: edges.anchors(),
            asset_names: HashMap::new(),
        };

        for &SelectedNote { id, pool } in &selection.notes {
            match pool {
                0 => {
                    let (utxo, has_sk) = load_utxo(connection, id).await?;
                    material.can_sign &= has_sk;
                    material.transparent.push(utxo);
                }
                1 => {
                    let dfvk = material
                        .sapling_vk
                        .as_ref()
                        .ok_or_else(|| anyhow!("No sapling key to spend note {id}"))?;
                    let (note, scope, merkle_path) =
                        get_sapling_note(connection, id, edges.witness_height, dfvk, &es, &ers)
                            .await?;
                    material.can_sign &= has_sapling_sk;
                    material.sapling.push(SaplingSpend {
                        note,
                        scope,
                        merkle_path,
                    });
                }
                2 | 3 => {
                    let fvk = material
                        .orchard_vk
                        .as_ref()
                        .ok_or_else(|| anyhow!("No orchard key to spend note {id}"))?;
                    let (edge, version) = if pool == 2 {
                        (&eo, selection.orchard_note_version)
                    } else {
                        (&ei, NoteVersion::V3)
                    };
                    let (note, merkle_path) = get_orchard_note(
                        connection,
                        id,
                        edges.witness_height,
                        fvk,
                        edge,
                        &ero,
                        version,
                        edges.rewind.then_some(edge.1),
                    )
                    .await?;
                    material.can_sign &= has_orchard_sk;
                    let spend = OrchardSpend { note, merkle_path };
                    if pool == 2 {
                        material.orchard.push(spend);
                    } else {
                        material.ironwood.push(spend);
                    }
                }
                _ => anyhow::bail!("Unknown pool {pool} for note {id}"),
            }
        }

        material.asset_names = load_asset_names(connection).await?;
        Ok(material)
    }
}

/// The stored transparent note with its address row, and whether the wallet
/// holds the address's secret key.
async fn load_utxo(connection: &mut SqliteConnection, id: u32) -> Result<(Utxo, bool)> {
    let row = sqlx::query(
        "SELECT nullifier, value, t.pk, t.sk, t.scope, t.dindex, t.address, t.uncompressed
        FROM notes
        JOIN transparent_address_accounts t ON notes.taddress = t.id_taddress
        WHERE id_note = ?",
    )
    .bind(id)
    .fetch_one(&mut *connection)
    .await?;

    let nf: Vec<u8> = row.get(0);
    let amount: i64 = row.get(1);
    let pk: Vec<u8> = row.get(2);
    let sk: Option<Vec<u8>> = row.get(3);
    let scope: u32 = row.get(4);
    let dindex: u32 = row.get(5);
    let address: String = row.get(6);
    let uncompressed: bool = row.get(7);

    anyhow::ensure!(
        nf.len() == 36,
        "Transparent note {id} has a malformed outpoint ({} bytes)",
        nf.len()
    );
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&nf[0..32]);
    let n = u32::from_le_bytes(nf[32..36].try_into().unwrap());
    let pubkey = PublicKey::from_slice(&pk)
        .map_err(|e| anyhow!("Transparent note {id} has an invalid public key: {e}"))?;

    Ok((
        Utxo {
            outpoint: OutPoint::new(hash, n),
            amount: amount as u64,
            pubkey,
            scope,
            dindex,
            address,
            uncompressed,
        },
        sk.is_some(),
    ))
}

async fn load_asset_names(connection: &mut SqliteConnection) -> Result<HashMap<[u8; 32], String>> {
    let names = sqlx::query(
        "SELECT asset_base, asset_name FROM assets
         WHERE asset_name IS NOT NULL AND asset_name != ''",
    )
    .map(|row: SqliteRow| {
        let asset_base: Vec<u8> = row.get(0);
        let asset_name: String = row.get(1);
        (asset_base, asset_name)
    })
    .fetch_all(connection)
    .await?
    .into_iter()
    .filter_map(|(asset_base, asset_name)| {
        asset_base
            .try_into()
            .ok()
            .map(|asset_base| (asset_base, asset_name))
    })
    .collect();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_schema;
    use crate::pay::plan::PlanOutputs;
    use crate::pay::pool::NUM_POOLS;
    use crate::pay::InputNote;
    use crate::sync::chain_log::{self, NoteRow, TransactionRow};
    use secp256k1::SecretKey;
    use sqlx::Connection;
    use zcash_transparent::keys::AccountPrivKey;

    const T_ADDR: &str = "t1VmmGiyjVNeCjxDZzg7vZmd99WyzVby9yC";

    fn pubkey() -> PublicKey {
        let sk = SecretKey::from_secret_bytes([1u8; 32]).unwrap();
        PublicKey::from_secret_key(&sk)
    }

    fn outpoint() -> OutPoint {
        OutPoint::new([7u8; 32], 1)
    }

    fn edges() -> TreeEdges {
        TreeEdges {
            sapling: Edge::default(),
            orchard: Edge::default(),
            ironwood: Edge::default(),
            witness_height: 100,
            rewind: false,
        }
    }

    fn selection(notes: Vec<SelectedNote>, transparent_change: bool) -> SpendSelection {
        SpendSelection {
            notes,
            orchard_note_version: NoteVersion::V2,
            transparent_change,
        }
    }

    /// A transparent-only account at dindex 3 holding one 5000 zat coin.
    /// `with_sk` stores the coin's secret key; `with_xvk` stores the
    /// account-level transparent viewing key.
    async fn transparent_wallet(with_sk: bool, with_xvk: bool) -> Result<(SqliteConnection, u32)> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        sqlx::query(
            "INSERT INTO accounts(id_account, name, aindex, dindex, def_dindex, birth, position,
             use_internal, hidden, saved) VALUES (1, 'a', 0, 3, 0, 0, 0, FALSE, FALSE, FALSE)",
        )
        .execute(&mut db)
        .await?;
        if with_xvk {
            let xvk =
                AccountPrivKey::from_seed(&Network::Main, &[2u8; 32], zip32::AccountId::ZERO)?
                    .to_account_pubkey()
                    .serialize();
            sqlx::query("INSERT INTO transparent_accounts(account, xsk, xvk) VALUES (1, NULL, ?)")
                .bind(xvk)
                .execute(&mut db)
                .await?;
        }
        sqlx::query(
            "INSERT INTO transparent_address_accounts(id_taddress, account, scope, dindex, sk, pk, address)
             VALUES (1, 1, 0, 3, ?, ?, ?)",
        )
        .bind(with_sk.then(|| vec![1u8; 32]))
        .bind(pubkey().serialize().to_vec())
        .bind(T_ADDR)
        .execute(&mut db)
        .await?;
        let id_tx = chain_log::record_transaction(&mut db, 1, &TransactionRow::chain(&[9], 100, 0))
            .await?
            .unwrap();
        let mut nf = vec![];
        outpoint().write(&mut nf)?;
        chain_log::record_note(
            &mut db,
            1,
            &NoteRow {
                id_tx,
                height: 100,
                pool: 0,
                nullifier: &nf,
                value: 5000,
                taddress: Some(1),
                ..Default::default()
            },
        )
        .await?;
        let id_note: u32 = sqlx::query_scalar("SELECT id_note FROM notes")
            .fetch_one(&mut db)
            .await?;
        Ok((db, id_note))
    }

    #[tokio::test]
    async fn a_transparent_spend_carries_its_coin_and_derivation() -> Result<()> {
        let (mut db, id) = transparent_wallet(true, true).await?;
        let sel = selection(vec![SelectedNote { id, pool: 0 }], true);
        let m = SpendMaterial::load(&Network::Main, &mut db, 1, &sel, &edges()).await?;

        assert_eq!(m.transparent.len(), 1);
        let utxo = &m.transparent[0];
        assert_eq!(utxo.outpoint, outpoint());
        assert_eq!(utxo.amount, 5000);
        assert_eq!(utxo.pubkey, pubkey());
        assert_eq!((utxo.scope, utxo.dindex), (0, 3));
        assert_eq!(utxo.address, T_ADDR);
        assert!(!utxo.uncompressed);
        assert!(m.sapling.is_empty() && m.orchard.is_empty() && m.ironwood.is_empty());
        assert!(m.can_sign);
        assert_eq!(m.change_address, T_ADDR);
        assert_eq!(m.transparent_change_address.as_deref(), Some(T_ADDR));
        assert_eq!((m.dindex, m.hw), (3, 0));
        assert!(m.sapling_vk.is_none() && m.orchard_vk.is_none());
        assert!(m.asset_names.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn a_coin_without_its_secret_key_cannot_be_signed() -> Result<()> {
        let (mut db, id) = transparent_wallet(false, true).await?;
        let sel = selection(vec![SelectedNote { id, pool: 0 }], false);
        let m = SpendMaterial::load(&Network::Main, &mut db, 1, &sel, &edges()).await?;
        assert!(!m.can_sign);
        Ok(())
    }

    #[tokio::test]
    async fn transparent_change_address_is_read_only_when_wanted_and_keyed() -> Result<()> {
        let (mut db, id) = transparent_wallet(true, true).await?;
        let sel = selection(vec![SelectedNote { id, pool: 0 }], false);
        let m = SpendMaterial::load(&Network::Main, &mut db, 1, &sel, &edges()).await?;
        assert_eq!(
            m.transparent_change_address, None,
            "no transparent change planned"
        );

        let (mut db, id) = transparent_wallet(true, false).await?;
        let sel = selection(vec![SelectedNote { id, pool: 0 }], true);
        let m = SpendMaterial::load(&Network::Main, &mut db, 1, &sel, &edges()).await?;
        assert_eq!(
            m.transparent_change_address, None,
            "no transparent account key"
        );
        Ok(())
    }

    #[tokio::test]
    async fn a_shielded_spend_needs_the_pool_key() -> Result<()> {
        let (mut db, id) = transparent_wallet(true, true).await?;
        for pool in [1u8, 2, 3] {
            let sel = selection(vec![SelectedNote { id, pool }], false);
            let err = SpendMaterial::load(&Network::Main, &mut db, 1, &sel, &edges())
                .await
                .err()
                .expect("a spend without a viewing key must be rejected");
            assert!(
                err.to_string().contains("key to spend"),
                "pool {pool}: {err}"
            );
        }
        Ok(())
    }

    #[test]
    fn spend_selection_keeps_only_spent_notes_in_builder_order() {
        let note = |id, pool, remaining| InputNote {
            id,
            height: 100,
            amount: 1000,
            remaining,
            pool,
            id_asset: None,
            asset_base: vec![],
            taddress: None,
            uncompressed: false,
        };
        let mut input_pools = vec![vec![]; NUM_POOLS];
        input_pools[2].push(note(5, 2, 0));
        input_pools[0].push(note(1, 0, 1000)); // untouched
        input_pools[0].push(note(2, 0, 400));
        input_pools[3].push(note(9, 3, 0));
        let plan = PlanOutputs {
            input_pools,
            recipient_states: vec![],
            change: 10,
            change_pool: 0,
            fee: 0,
            has_pool: [true, false, true, true],
            ironwood_active: true,
            orchard_note_version: NoteVersion::V3,
            price: None,
        };
        let sel = plan.spend_selection();
        assert_eq!(
            sel.notes,
            vec![
                SelectedNote { id: 2, pool: 0 },
                SelectedNote { id: 5, pool: 2 },
                SelectedNote { id: 9, pool: 3 },
            ]
        );
        assert_eq!(sel.orchard_note_version, NoteVersion::V3);
        assert!(sel.transparent_change);

        let plan = PlanOutputs {
            change_pool: 2,
            ..plan
        };
        assert!(!plan.spend_selection().transparent_change);
    }
}
