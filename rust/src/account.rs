use std::str::FromStr as _;

use crate::{
    api::{coin::Network, ledger::{
        get_hw_fvk, get_hw_next_diversifier_address, get_hw_sapling_address,
        get_hw_transparent_address,
    }}, bip38, db::{
        init_account_orchard, init_account_sapling, init_account_transparent,
        store_account_orchard_sk, store_account_orchard_vk, store_account_sapling_sk,
        store_account_sapling_vk, store_account_seed, store_account_transparent_addr,
        store_account_transparent_sk, store_account_transparent_vk, update_dindex,
    }, key::{is_valid_phrase, is_valid_sapling_key, is_valid_transparent_key, is_valid_ufvk}, pay::plan::sapling_dfvk_to_fvk, tiu
};
use crate::{
    api::{
        account::{
            Category, Folder, NewAccount, Seed, TxAccount, TxMemo, TxNote, TxOutput, TxSpend,
        },
        key::generate_seed,
    },
    db::{
        get_account_hw, select_account_transparent, store_account_hw, store_account_metadata,
        LEDGER_CODE,
    },
    pay::pool::ALL_POOLS,
};
use secp256k1::{PublicKey, SecretKey};
use zcash_keys::keys::{sapling::ExtendedSpendingKey, UnifiedFullViewingKey, UnifiedSpendingKey};
use zcash_transparent::address::TransparentAddress;

use anyhow::{anyhow, Result};
use bincode::config::legacy;
use bip32::{ExtendedPrivateKey, ExtendedPublicKey, PrivateKey};
use jubjub::Fr;
use orchard::{
    keys::FullViewingKey,
    note::{RandomSeed, Rho},
    tree::MerkleHashOrchard,
    value::NoteValue,
    Note,
};
use ripemd::{Digest as _, Ripemd160};
use sapling_crypto::{zip32::DiversifiableFullViewingKey, PaymentAddress};
use sha2::Sha256;
use sqlx::{sqlite::SqliteRow, Connection, Row, SqliteConnection};
use zcash_keys::{
    address::UnifiedAddress, encoding::AddressCodec as _, keys::UnifiedAddressRequest,
};
use zcash_protocol::consensus::{NetworkConstants, NetworkUpgrade, Parameters};
use zcash_transparent::keys::{
    AccountPrivKey, AccountPubKey, NonHardenedChildIndex, TransparentKeyScope,
};
use zip32::{fingerprint::SeedFingerprint, AccountId};

use crate::{
    api::account::FrostParams,
    sync::trim_sync_data,
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

    let account = store_account_metadata(
        &mut db_tx,
        &na.name,
        &na.icon,
        &na.fingerprint,
        birth,
        na.use_internal,
        na.internal,
    )
    .await?;

    let mut key = na.key.clone();
    if key.is_empty() && !na.ledger {
        key = generate_seed()?;
    }

    let pools = na.pools.unwrap_or(ALL_POOLS);

    if na.ledger {
        let has_seed = !key.is_empty();
        if !has_seed {
            store_account_hw(&mut db_tx, account, LEDGER_CODE, na.aindex).await?;
        }
        // we must do sapling derivation first to know a valid dindex
        // because in sapling some indices are invalid
        let mut dindex = 0;
        if pools & 2 != 0 {
            init_account_sapling(network, &mut db_tx, account, birth).await?;
            if has_seed {
                let sxsk = crate::recover::recover_ledger_seed(&key, na.aindex).await?;
                store_account_sapling_sk(&mut db_tx, account, &sxsk).await?;
                let sxvk = sxsk.to_diversifiable_full_viewing_key();
                let (di, _) = sxvk.default_address();
                let di: u128 = di.into();
                dindex = di as u32;
                let address = derive_sapling_address(network, &sxvk, dindex);
                store_account_sapling_vk(&mut db_tx, account, &sxvk, &address).await?;
            } else {
                let fvk = get_hw_fvk(network, LEDGER_CODE, na.aindex).await?;
                let mut dfvk = fvk.to_bytes().to_vec();
                dfvk.extend_from_slice(&[0u8; 32]); // add a dummy dk because we cannot get the one from the Ledger
                let xvk = DiversifiableFullViewingKey::from_bytes(&tiu!(dfvk)).unwrap();
                // We should get the default address dindex by using the get_div_list
                // api but it is currently not working
                // instead, we "assume" the dindex = 0 is the default sapling address
                // let (dindex, address) = get_hw_next_diversifier_address(&network, na.aindex, 0).await?;
                let address = get_hw_sapling_address(network, na.aindex).await?;
                store_account_sapling_vk(&mut db_tx, account, &xvk, &address).await?;
            }
        }
        if pools & 1 != 0 && !has_seed {
            init_account_transparent(&mut db_tx, account, birth).await?;
            let (pk, taddr) = get_hw_transparent_address(network, na.aindex, 0, dindex).await?;
            store_account_transparent_addr(
                &mut db_tx,
                account,
                0,
                dindex,
                None,
                &pk,
                &taddr.encode(network),
            )
            .await?;
        }
        update_dindex(&mut db_tx, account, dindex, true).await?;
    } else if is_valid_phrase(&key) {
        let seed_phrase = bip39::Mnemonic::from_str(&key)?;
        let passphrase = na.passphrase.clone().unwrap_or_default();
        let seed = seed_phrase.to_seed(&passphrase);
        let seed_fingerprint = SeedFingerprint::from_seed(&seed).unwrap().to_bytes();
        store_account_seed(
            &mut db_tx,
            account,
            &key,
            &passphrase,
            &seed_fingerprint,
            na.aindex,
        )
        .await?;
        let usk = UnifiedSpendingKey::from_seed(
            &network,
            &seed,
            AccountId::try_from(na.aindex).unwrap(),
        )?;
        let uvk = usk.to_unified_full_viewing_key();
        let (_, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
        let dindex: u32 = di.try_into()?;

        if pools & 1 != 0 {
            init_account_transparent(&mut db_tx, account, birth).await?;
            let tsk = usk.transparent();
            store_account_transparent_sk(&mut db_tx, account, tsk).await?;
            let tvk = &tsk.to_account_pubkey();
            store_account_transparent_vk(&mut db_tx, account, tvk).await?;
            let sk = derive_transparent_sk(tsk, 0, dindex)?;
            let (pk, taddr) = derive_transparent_address(tvk, 0, dindex)?;
            store_account_transparent_addr(
                &mut db_tx,
                account,
                0,
                dindex,
                Some(sk),
                &pk,
                &taddr.encode(&network),
            )
            .await?;
        }

        if pools & 2 != 0 {
            init_account_sapling(network, &mut db_tx, account, birth).await?;
            let sxsk = usk.sapling();
            store_account_sapling_sk(&mut db_tx, account, sxsk).await?;
            let sxvk = sxsk.to_diversifiable_full_viewing_key();
            let address = derive_sapling_address(network, &sxvk, dindex);
            store_account_sapling_vk(&mut db_tx, account, &sxvk, &address).await?;
        }

        if pools & 4 != 0 {
            init_account_orchard(network, &mut db_tx, account, birth).await?;
            let oxsk = usk.orchard();
            store_account_orchard_sk(&mut db_tx, account, oxsk).await?;
            let oxvk = FullViewingKey::from(oxsk);
            store_account_orchard_vk(&mut db_tx, account, &oxvk).await?;
        }

        update_dindex(&mut db_tx, account, dindex, true).await?;
    } else if is_valid_transparent_key(&key) {
        init_account_transparent(&mut db_tx, account, birth).await?;
        if let Ok(xsk) = ExtendedPrivateKey::<SecretKey>::from_str(&key) {
            let xsk = AccountPrivKey::from_extended_privkey(xsk);
            store_account_transparent_sk(&mut db_tx, account, &xsk).await?;
            let xvk = xsk.to_account_pubkey();
            store_account_transparent_vk(&mut db_tx, account, &xvk).await?;
            let sk = derive_transparent_sk(&xsk, 0, 0)?;
            let (pk, address) = derive_transparent_address(&xvk, 0, 0)?;
            store_account_transparent_addr(
                &mut db_tx,
                account,
                0,
                0,
                Some(sk),
                &pk,
                &address.encode(&network),
            )
            .await?;
        } else if let Ok(xvk) = ExtendedPublicKey::<PublicKey>::from_str(&key) {
            // No AccountPubKey::from_extended_pubkey, we need to use the bytes
            let mut buf = xvk.attrs().chain_code.to_vec();
            buf.extend_from_slice(&xvk.to_bytes());
            let xvk = AccountPubKey::deserialize(&buf.try_into().unwrap()).unwrap();
            store_account_transparent_vk(&mut db_tx, account, &xvk).await?;
            let (pk, address) = derive_transparent_address(&xvk, 0, 0)?;
            store_account_transparent_addr(
                &mut db_tx,
                account,
                0,
                0,
                None,
                &pk,
                &address.encode(&network),
            )
            .await?;
        } else if let Ok(sk) = bip38::import_tsk(&key) {
            let secp = secp256k1::Secp256k1::new();
            let pk = sk.public_key(&secp);
            let tpk = pk.serialize().to_vec();
            let pkh: [u8; 20] = Ripemd160::digest(Sha256::digest(&tpk)).into();
            let addr = TransparentAddress::PublicKeyHash(pkh);
            store_account_transparent_addr(
                &mut db_tx,
                account,
                0,
                0,
                Some(sk.to_bytes().to_vec()),
                &tpk,
                &addr.encode(&network),
            )
            .await?;
        } else if let Ok((_, tpk)) = bech32::decode(&key) {
            let pkh: [u8; 20] = Ripemd160::digest(Sha256::digest(&tpk)).into();
            let addr = TransparentAddress::PublicKeyHash(pkh);
            store_account_transparent_addr(
                &mut db_tx,
                account,
                0,
                0,
                None,
                &tpk,
                &addr.encode(&network),
            )
            .await?;
        }
    } else if is_valid_sapling_key(network, &key) {
        init_account_sapling(network, &mut db_tx, account, birth).await?;
        let di = if let Ok(xsk) = zcash_keys::encoding::decode_extended_spending_key(
            network.hrp_sapling_extended_spending_key(),
            &key,
        ) {
            store_account_sapling_sk(&mut db_tx, account, &xsk).await?;
            let xvk = xsk.to_diversifiable_full_viewing_key();
            let (di, address) = xvk.default_address();
            let address = address.encode(&network);
            store_account_sapling_vk(&mut db_tx, account, &xvk, &address).await?;
            di
        } else if let Ok(xvk) = zcash_keys::encoding::decode_extended_full_viewing_key(
            network.hrp_sapling_extended_full_viewing_key(),
            &key,
        ) {
            let (di, address) = xvk.default_address();
            let address = address.encode(&network);
            store_account_sapling_vk(
                &mut db_tx,
                account,
                &xvk.to_diversifiable_full_viewing_key(),
                &address,
            )
            .await?;
            di
        } else {
            return Err(anyhow!("Invalid Sapling Key"));
        };
        let dindex: u32 = di.try_into()?;
        update_dindex(&mut db_tx, account, dindex, true).await?;
    } else if is_valid_ufvk(network, &key) {
        let uvk =
            UnifiedFullViewingKey::decode(&network, &key).map_err(|_| anyhow!("Invalid Key"))?;
        let (ua, di) = uvk.default_address(UnifiedAddressRequest::AllAvailableKeys)?;
        let dindex: u32 = di.try_into()?;

        match uvk.transparent() {
            Some(tvk) if pools & 1 != 0 => {
                init_account_transparent(&mut db_tx, account, birth).await?;
                store_account_transparent_vk(&mut db_tx, account, tvk).await?;
                let (pk, address) = derive_transparent_address(tvk, 0, dindex)?;
                store_account_transparent_addr(
                    &mut db_tx,
                    account,
                    0,
                    dindex,
                    None,
                    &pk,
                    &address.encode(&network),
                )
                .await?;
            }
            _ => {}
        }
        match uvk.sapling() {
            Some(sxvk) if pools & 2 != 0 => {
                init_account_sapling(network, &mut db_tx, account, birth).await?;
                let address = ua.sapling().unwrap();
                let address = address.encode(&network);
                store_account_sapling_vk(&mut db_tx, account, sxvk, &address).await?;
            }
            _ => {}
        }
        match uvk.orchard() {
            Some(ovk) if pools & 4 != 0 => {
                init_account_orchard(network, &mut db_tx, account, birth).await?;
                store_account_orchard_vk(&mut db_tx, account, ovk).await?;
            }
            _ => {}
        }
        update_dindex(&mut db_tx, account, dindex, true).await?;
    }
    db_tx.commit().await?;
    Ok(account)
}

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
    edge: &AuthPath,
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

        let auth_path = witness.build_auth_path(edge, empty_roots);
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
        "SELECT sa.xvk, sa.address FROM sapling_accounts sa
        JOIN accounts a ON sa.account = a.id_account AND sa.account = ?",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let xvk: Vec<u8> = row.get(0);
        let address: String = row.get(1);
        let fvk = DiversifiableFullViewingKey::from_bytes(&xvk.try_into().unwrap()).unwrap();
        if scope == 1 && hw == 0 {
            // we do not need to derive a diversified change address
            // since they are not exposed to the user
            let (_, pa) = fvk.change_address();
            pa
        } else {
            PaymentAddress::decode(network, &address).unwrap()
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
    let mut db_tx = connection.begin().await?;
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
            let (di, address) = get_hw_next_diversifier_address(network, aindex, dindex).await?;
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

    let tkeys = select_account_transparent(&mut db_tx, account, dindex).await?;
    let (sk, pk, address) = match tkeys.xvk {
        Some(xvk) => {
            let sk = tkeys
                .xsk
                .as_ref()
                .map(|tsk| derive_transparent_sk(tsk, 0, dindex).unwrap());
            let (pk, address) = derive_transparent_address(&xvk, 0, dindex)?;
            (sk, pk, Some(address))
        }
        None if hw != 0 => {
            let (pk, address) = get_hw_transparent_address(network, aindex, 0, dindex).await?;
            (None, pk, Some(address))
        }
        _ => (None, vec![], None),
    };

    if let Some(address) = address {
        store_account_transparent_addr(
            &mut db_tx,
            account,
            0,
            dindex,
            sk,
            &pk,
            &address.encode(network),
        )
        .await?;
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

pub async fn reset_sync(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<()> {
    let birth_height = sqlx::query("SELECT birth FROM accounts WHERE id_account = ?")
        .bind(account)
        .map(|row: SqliteRow| row.get::<u32, _>(0))
        .fetch_one(&mut *connection)
        .await?;
    trim_sync_data(&mut *connection, account, 0).await?;
    init_sync_heights(network, &mut *connection, account, birth_height).await?;
    Ok(())
}

pub async fn init_sync_heights(
    network: &Network,
    connection: &mut SqliteConnection,
    account: u32,
    birth_height: u32,
) -> Result<()> {
    for pool in 0..3 {
        let activation_height: u32 = match pool {
            0 => 0,
            1 => network
                .activation_height(NetworkUpgrade::Sapling)
                .unwrap()
                .into(),
            2 => network
                .activation_height(NetworkUpgrade::Nu5)
                .unwrap()
                .into(),
            _ => unreachable!(),
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
        _ => unreachable!(),
    };
    Ok(has_pool)
}

fn derive_sapling_address(
    network: &Network,
    sxvk: &DiversifiableFullViewingKey,
    dindex: u32,
) -> String {
    let address = sxvk.address(dindex.into()).unwrap();
    address.encode(network)
}
