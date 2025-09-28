use crate::{api::account::{Category, Folder}, coin::Network};
use anyhow::{Ok, Result};
use bincode::config::legacy;
use bip32::PrivateKey;
use jubjub::Fr;
use orchard::{
    keys::FullViewingKey,
    note::{RandomSeed, Rho},
    tree::MerkleHashOrchard,
    value::NoteValue,
    Note,
};
use ripemd::{Digest as _, Ripemd160};
use sapling_crypto::zip32::{DiversifiableFullViewingKey, ExtendedSpendingKey};
use sha2::Sha256;
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};
use zcash_keys::{address::UnifiedAddress, encoding::AddressCodec as _};
use zcash_primitives::legacy::TransparentAddress;
use zcash_protocol::consensus::{NetworkUpgrade, Parameters};
use zcash_transparent::keys::{
    AccountPrivKey, AccountPubKey, NonHardenedChildIndex, TransparentKeyScope,
};

use crate::{
    api::account::FrostParams,
    db::store_account_transparent_addr,
    sync::trim_sync_data,
    warp::{AuthPath, Witness, MERKLE_DEPTH},
};

pub fn derive_transparent_sk(tsk: &AccountPrivKey, scope: u32, dindex: u32) -> Result<Vec<u8>> {
    let scope = match scope {
        0 => TransparentKeyScope::EXTERNAL,
        1 => TransparentKeyScope::INTERNAL,
        _ => unreachable!(),
    };
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
) -> Result<(Vec<u8>, TransparentAddress)> {
    let sindex = TransparentKeyScope::custom(scope).unwrap();
    let tpk = tvk
        .derive_address_pubkey(sindex, NonHardenedChildIndex::from_index(dindex).unwrap())
        .unwrap()
        .serialize();
    let pkh: [u8; 20] = Ripemd160::digest(Sha256::digest(tpk)).into();
    let addr = TransparentAddress::PublicKeyHash(pkh);
    Ok((tpk.to_vec(), addr))
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

pub async fn get_sapling_note(
    connection: &mut SqliteConnection,
    id: u32,
    height: u32,
    fvk: &sapling_crypto::keys::FullViewingKey,
    edge: &AuthPath,
    empty_roots: &AuthPath,
) -> Result<(sapling_crypto::Note, sapling_crypto::MerklePath)> {
    let r = sqlx::query(
        "SELECT position, diversifier, value, rcm, witness FROM notes
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

        let diversifier = sapling_crypto::Diversifier(diversifier.try_into().unwrap());
        let recipient = fvk.vk.to_payment_address(diversifier).unwrap();

        let value = sapling_crypto::value::NoteValue::from_raw(value);

        let rseed =
            sapling_crypto::Rseed::BeforeZip212(Fr::from_bytes(&rcm.try_into().unwrap()).unwrap());

        let note = sapling_crypto::Note::from_parts(recipient, value, rseed);

        let (witness, _) = bincode::decode_from_slice::<Witness, _>(&witness, legacy()).unwrap();

        let auth_path = witness.build_auth_path(edge, empty_roots);
        let mut mp = vec![];
        for i in 0..MERKLE_DEPTH as usize {
            mp.push(sapling_crypto::Node::from_bytes(auth_path.0[i]).unwrap());
        }

        assert_eq!(position, witness.position);
        let merkle_path =
            sapling_crypto::MerklePath::from_parts(mp, (witness.position as u64).into()).unwrap();

        (note, merkle_path)
    })
    .fetch_one(connection)
    .await?;

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
    height: u32,
    ovk: &orchard::keys::FullViewingKey,
    eo: &AuthPath,
    ero: &AuthPath,
) -> Result<(orchard::Note, orchard::tree::MerklePath)> {
    let (scope, position, diversifier, value, rcm, rho, witness) = sqlx::query(
        "SELECT scope, position, diversifier, value, rcm, rho, witness FROM notes
        JOIN witnesses w ON notes.id_note = w.note
        WHERE id_note = ? AND w.height = ?",
    )
    .bind(id)
    .bind(height)
    .map(|row: SqliteRow| {
        let scope: Option<u8> = row.get(0);
        let position: u32 = row.get(1);
        let diversifier: Vec<u8> = row.get(2);
        let value: u64 = row.get::<i64, _>(3) as u64;
        let rcm: Vec<u8> = row.get(4);
        let rho: Vec<u8> = row.get(5);
        let witness: Vec<u8> = row.get(6);
        (scope, position, diversifier, value, rcm, rho, witness)
    })
    .fetch_one(connection)
    .await?;

    let scope = scope.unwrap_or(0);
    let scope = match scope {
        1 => orchard::keys::Scope::Internal,
        0 => orchard::keys::Scope::External,
        _ => unreachable!(),
    };
    let (witness, _) = bincode::decode_from_slice::<Witness, _>(&witness, legacy()).unwrap();
    let rho = Rho::from_bytes(&rho.try_into().unwrap()).unwrap();

    let diversifer = orchard::keys::Diversifier::from_bytes(diversifier.try_into().unwrap());
    let recipient = ovk.address(diversifer, scope);
    let value = NoteValue::from_raw(value);
    let rseed = RandomSeed::from_bytes(rcm.try_into().unwrap(), &rho).unwrap();
    let note = Note::from_parts(recipient, value, rho, rseed)
        .into_option()
        .unwrap();

    assert_eq!(witness.position, position);
    let auth_path = witness.build_auth_path(eo, ero);
    let auth_path = auth_path
        .0
        .iter()
        .map(|a| MerkleHashOrchard::from_bytes(a).unwrap())
        .collect::<Vec<_>>();
    let auth_path: [MerkleHashOrchard; MERKLE_DEPTH as usize] = auth_path.try_into().unwrap();
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
        "SELECT a.dindex, sa.xvk FROM sapling_accounts sa
        JOIN accounts a ON sa.account = a.id_account AND sa.account = ?",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let dindex: u32 = row.get(0);
        let xvk: Vec<u8> = row.get(1);
        let fvk = DiversifiableFullViewingKey::from_bytes(&xvk.try_into().unwrap()).unwrap();
        if scope == 1 {
            // we do not need to derive a diversified change address
            // since they are not exposed to the user
            let (_, pa) = fvk.change_address();
            pa
        } else {
            fvk.address(dindex.into()).unwrap()
        }
    })
    .fetch_optional(&mut *connection)
    .await?;

    let oaddress = sqlx::query(
        "SELECT a.dindex, oa.xvk FROM orchard_accounts oa
        JOIN accounts a ON oa.account = a.id_account AND oa.account = ?",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let dindex: u32 = row.get(0);
        let xvk: Vec<u8> = row.get(1);
        let fvk = FullViewingKey::read(&*xvk).unwrap();
        let scope = if scope == 1 {
            orchard::keys::Scope::Internal
        } else {
            orchard::keys::Scope::External
        };
        fvk.address_at(dindex, scope)
    })
    .fetch_optional(connection)
    .await?;

    let address = match (taddress, saddress, oaddress) {
        (Some(taddress), None, None) => taddress.encode(network),
        _ => {
            let ua = UnifiedAddress::from_receivers(oaddress, saddress, taddress).unwrap();
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
    let (mut dindex,): (u32,) = sqlx::query_as("SELECT dindex FROM accounts WHERE id_account = ?")
        .bind(account)
        .fetch_one(&mut *connection)
        .await?;
    let svk = get_sapling_vk(&mut *connection, account).await?;
    if let Some(svk) = svk.as_ref() {
        dindex += 1;
        let (di, _) = svk.find_address(dindex.into()).unwrap();
        dindex = di.try_into()?;
    } else {
        // without sapling, any dindex is ok, just increment
        dindex += 1;
    }

    sqlx::query("UPDATE accounts SET dindex = ? WHERE id_account = ?")
        .bind(dindex)
        .bind(account)
        .execute(&mut *connection)
        .await?;

    let (xsk, xvk) = get_transparent_keys(connection, account).await?;
    if let Some(xvk) = xvk.as_ref() {
        let sk = xsk
            .as_ref()
            .map(|tsk| derive_transparent_sk(tsk, 0, dindex).unwrap());
        let (tpk, taddress) = derive_transparent_address(xvk, 0, dindex)?;
        store_account_transparent_addr(
            &mut *connection,
            account,
            0,
            dindex,
            sk,
            &tpk,
            &taddress.encode(network),
        )
        .await?;
    }

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
        let (change_pk, change_address) = derive_transparent_address(tvk, 1, dindex)?;
        let change_address = change_address.encode(network);

        store_account_transparent_addr(
            &mut *connection,
            account,
            1,
            dindex,
            sk,
            &change_pk,
            &change_address,
        )
        .await?;

        return Ok(Some(change_address));
    }

    Ok(None)
}

async fn get_transparent_keys(
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

pub async fn reset_sync(network: &Network, connection: &mut SqliteConnection, account: u32) -> Result<()> {
    let birth_height = sqlx::query("SELECT birth FROM accounts WHERE id_account = ?")
        .bind(account)
        .map(|row: SqliteRow| row.get::<u32, _>(0))
        .fetch_one(&mut *connection)
        .await?;
    trim_sync_data(&mut *connection, account, 0).await?;
    init_sync_heights(network, &mut *connection, account, birth_height).await?;
    Ok(())
}

pub async fn init_sync_heights(network: &Network, connection: &mut SqliteConnection, account: u32, birth_height: u32) -> Result<()> {
    for pool in 0..3 {
        let activation_height: u32 = match pool {
            0 => 0,
            1 => network.activation_height(NetworkUpgrade::Sapling).unwrap().into(),
            2 => network.activation_height(NetworkUpgrade::Nu5).unwrap().into(),
            _ => unreachable!()
        };
        sqlx::query("UPDATE sync_heights SET height = ?3 WHERE account = ?1 AND pool = ?2")
        .bind(account)
        .bind(pool)
        .bind(birth_height.max(activation_height))
        .execute(&mut *connection)
        .await?;
    }
    Ok(())
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
        }
    })
    .fetch_one(&mut *connection)
    .await?;

    let notes = sqlx::query(
        "SELECT id_note, pool, height, value, locked FROM notes
        WHERE account = ? AND tx = ?",
    )
    .bind(account)
    .bind(tx.id)
    .map(|row: SqliteRow| {
        let id_note: u32 = row.get(0);
        let pool: u8 = row.get(1);
        let height: u32 = row.get(2);
        let value: u64 = row.get(3);
        let locked: bool = row.get(4);
        TxNote {
            id: id_note,
            pool,
            height,
            value,
            locked,
        }
    })
    .fetch_all(&mut *connection)
    .await?;

    let outputs = sqlx::query(
        "SELECT id_output, pool, height, value, address FROM outputs
        WHERE account = ? AND tx = ?",
    )
    .bind(account)
    .bind(tx.id)
    .map(|row: SqliteRow| {
        let id_output: u32 = row.get(0);
        let pool: u8 = row.get(1);
        let height: u32 = row.get(2);
        let value: u64 = row.get(3);
        let address: String = row.get(4);
        TxOutput {
            id: id_output,
            pool,
            height,
            value,
            address,
        }
    })
    .fetch_all(&mut *connection)
    .await?;

    let spends = sqlx::query(
        "SELECT id_note, pool, height, value FROM spends
        WHERE account = ? AND tx = ?",
    )
    .bind(account)
    .bind(tx.id)
    .map(|row: SqliteRow| {
        let id: u32 = row.get(0);
        let pool: u8 = row.get(1);
        let height: u32 = row.get(2);
        let value: i64 = row.get(3);
        TxSpend {
            id,
            pool,
            height,
            value: -value as u64,
        }
    })
    .fetch_all(&mut *connection)
    .await?;

    let memos = sqlx::query(
        "SELECT note, output, pool, memo_text FROM memos
        WHERE account = ? AND tx = ?",
    )
    .bind(account)
    .bind(tx.id)
    .map(|row: SqliteRow| {
        let note: Option<u32> = row.get(0);
        let output: Option<u32> = row.get(1);
        let pool: u8 = row.get(2);
        let memo: Option<String> = row.get(3);
        TxMemo {
            note,
            output,
            pool,
            memo,
        }
    })
    .fetch_all(&mut *connection)
    .await?;

    tx.notes = notes;
    tx.spends = spends;
    tx.memos = memos;
    tx.outputs = outputs;

    Ok(tx)
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

#[derive(Default, Debug)]
pub struct TxAccount {
    pub id: u32,
    pub account: u32,
    pub txid: Vec<u8>,
    pub height: u32,
    pub time: u32,
    pub price: Option<f64>,
    pub category: Option<u32>,
    pub notes: Vec<TxNote>,
    pub spends: Vec<TxSpend>,
    pub outputs: Vec<TxOutput>,
    pub memos: Vec<TxMemo>,
}

#[derive(Default, Debug)]
pub struct TxNote {
    pub id: u32,
    pub pool: u8,
    pub height: u32,
    pub value: u64,
    pub locked: bool,
}

#[derive(Default, Debug)]
pub struct TxSpend {
    pub id: u32,
    pub pool: u8,
    pub height: u32,
    pub value: u64,
}

#[derive(Default, Debug)]
pub struct TxOutput {
    pub id: u32,
    pub pool: u8,
    pub height: u32,
    pub value: u64,
    pub address: String,
}

#[derive(Default, Debug)]
pub struct TxMemo {
    pub note: Option<u32>,
    pub output: Option<u32>,
    pub pool: u8,
    pub memo: Option<String>,
}

pub async fn list_folders(connection: &mut SqliteConnection) -> Result<Vec<Folder>> {
    let folders = sqlx::query("SELECT id_folder, name FROM folders ORDER BY name")
        .map(|r: SqliteRow| {
            Folder {
                id: r.get(0),
                name: r.get(1),
            }
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
    .await?.is_some() {
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
    let folders = sqlx::query("SELECT id_category, name, income FROM categories ORDER BY income, name")
        .map(|r: SqliteRow| {
            Category {
                id: r.get(0),
                name: r.get(1),
                is_income: r.get(2),
            }
        })
        .fetch_all(connection)
        .await?;
    Ok(folders)
}

pub async fn create_new_category(connection: &mut SqliteConnection, category: &Category) -> Result<u32> {
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
    .await?.is_some() {
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
        sqlx::query("UPDATE transactions SET category = NULL where category = ?1")
        .bind(id)
        .execute(&mut *connection)
        .await?;

        sqlx::query("DELETE FROM categories WHERE id_category = ?1")
        .bind(id)
        .execute(&mut *connection)
        .await?;
    }
    Ok(())
}

pub async fn has_transparent_pub_key(connection: &mut SqliteConnection, account: u32) -> Result<bool> {
    let r = sqlx::query("SELECT xvk FROM transparent_accounts WHERE account = ?1")
    .bind(account)
    .map(|r: SqliteRow| r.get::<Option<Vec<u8>>, _>(0))
    .fetch_optional(connection)
    .await?.flatten();
    Ok(r.is_some())
}
