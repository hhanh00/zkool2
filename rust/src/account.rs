use anyhow::Result;
use bincode::config::legacy;
use bip32::PrivateKey;
use jubjub::Fr;
use orchard::{
    keys::Scope,
    note::{RandomSeed, Rho},
    tree::MerkleHashOrchard,
    value::NoteValue,
    Note,
};
use ripemd::{Digest as _, Ripemd160};
use sha2::Sha256;
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use zcash_primitives::legacy::TransparentAddress;
use zcash_transparent::keys::{
    AccountPrivKey, AccountPubKey, NonHardenedChildIndex, TransparentKeyScope,
};

use crate::warp::{AuthPath, Witness, MERKLE_DEPTH};

pub fn derive_transparent_sk(tvk: &AccountPrivKey, dindex: u32) -> Result<[u8; 32]> {
    let tsk = tvk
        .derive_external_secret_key(NonHardenedChildIndex::from_index(dindex).unwrap())
        .unwrap()
        .to_bytes();
    Ok(tsk)
}

pub fn derive_transparent_address(
    tvk: &AccountPubKey,
    scope: u32,
    dindex: u32,
) -> Result<TransparentAddress> {
    let sindex = TransparentKeyScope::custom(scope).unwrap();
    let tpk = tvk
        .derive_address_pubkey(sindex, NonHardenedChildIndex::from_index(dindex).unwrap())
        .unwrap()
        .serialize();
    let pkh: [u8; 20] = Ripemd160::digest(&Sha256::digest(&tpk)).into();
    let addr = TransparentAddress::PublicKeyHash(pkh);
    Ok(addr)
}

pub async fn get_sapling_vk(
    connection: &SqlitePool,
    account: u32,
) -> Result<sapling_crypto::keys::FullViewingKey> {
    let (fvk,): (Vec<u8>,) = sqlx::query_as("SELECT xvk FROM sapling_accounts WHERE account = ?")
        .bind(account)
        .fetch_one(connection)
        .await?;
    let fvk = sapling_crypto::keys::FullViewingKey::read(&*fvk).unwrap();

    Ok(fvk)
}

pub async fn get_sapling_note(
    connection: &SqlitePool,
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

pub async fn get_orchard_vk(
    connection: &sqlx::Pool<sqlx::Sqlite>,
    account: u32,
) -> Result<orchard::keys::FullViewingKey> {
    let (fvk,): (Vec<u8>,) = sqlx::query_as("SELECT xvk FROM orchard_accounts WHERE account = ?")
        .bind(account)
        .fetch_one(connection)
        .await?;
    let fvk = orchard::keys::FullViewingKey::read(&*fvk).unwrap();

    Ok(fvk)
}

pub async fn get_orchard_note(
    connection: &SqlitePool,
    id: u32,
    height: u32,
    ovk: &orchard::keys::FullViewingKey,
    eo: &AuthPath,
    ero: &AuthPath,
) -> Result<(orchard::Note, orchard::tree::MerklePath)> {
    let (position, diversifier, value, rcm, rho, witness) = sqlx::query(
        "SELECT position, diversifier, value, rcm, rho, witness FROM notes 
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
        let rho: Vec<u8> = row.get(4);
        let witness: Vec<u8> = row.get(5);
        (position, diversifier, value, rcm, rho, witness)
    })
    .fetch_one(connection)
    .await?;

    // let diversifier = vec![];
    // let value: u64 = 0;
    // let rho = vec![];
    // let rcm = vec![];
    // let witness = Witness::default();
    // let position = 0;

    let (witness, _) = bincode::decode_from_slice::<Witness, _>(&witness, legacy()).unwrap();
    let rho = Rho::from_bytes(&rho.try_into().unwrap()).unwrap();

    let diversifer = orchard::keys::Diversifier::from_bytes(diversifier.try_into().unwrap());
    let recipient = ovk.address(diversifer, Scope::External);
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
    let merkle_path = orchard::tree::MerklePath::from_parts(witness.position as u32, auth_path);

    Ok((note, merkle_path))
}
