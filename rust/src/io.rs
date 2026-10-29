use std::collections::HashMap;

use age::{scrypt::Identity, Decryptor, Encryptor};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use zcash_keys::encoding::AddressCodec as _;
use serde_with::{hex::Hex, serde_as};
use sqlx::{
    encode::IsNull,
    error::BoxDynError,
    sqlite::{SqliteArgumentsBuffer, SqliteRow, SqliteValueRef},
    Connection, Decode, Encode, Row, Sqlite, SqliteConnection, Type,
};
use std::io::prelude::*;
use tracing::info;
use zcash_keys::keys::sapling::{DiversifiableFullViewingKey, ExtendedSpendingKey};
use zstd::DEFAULT_COMPRESSION_LEVEL;

use crate::api::coin::Network;
use crate::db::DB_VERSION;
use crate::frost::keys::{DKG_VERSION_0, DKG_VERSION_1};
use crate::sync::chain_log::{
    self, AssetRow, MemoRow, NoteKey, NoteRow, OutputRow, SpendRow, TransactionRow,
};
use crate::sync::BlockHeader;

pub async fn export_account(connection: &mut SqliteConnection, account: u32) -> Result<Vec<u8>> {
    info!("Exporting account {}", account);
    let mut io_account = sqlx::query(
        "SELECT id_account, a.name, seed, passphrase, seed_fingerprint, aindex, dindex, def_dindex, icon, birth, position, use_internal, hidden, saved, enabled, internal,
        hw,
        f.name
        FROM accounts a
        LEFT JOIN folders f ON a.folder = f.id_folder
        WHERE id_account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let id_account: u32 = row.get(0);
            let name: String = row.get(1);
            let seed: Option<String> = row.get(2);
            let passphrase: String = row.get(3);
            let seed_fingerprint: Option<Vec<u8>> = row.get(4);
            let aindex: u32 = row.get(5);
            let dindex: u32 = row.get(6);
            let def_dindex: u32 = row.get(7);
            let icon: Option<Vec<u8>> = row.get(8);
            let birth: u32 = row.get(9);
            let position: u32 = row.get(10);
            let use_internal: bool = row.get(11);
            let hidden: bool = row.get(12);
            let saved: bool = row.get(13);
            let enabled: bool = row.get(14);
            let internal: bool = row.get(15);
            let hw: u8 = row.get(16);
            let folder_name: Option<String> = row.get(17);

            IOAccount {
                version: DB_VERSION,
                id_account,
                name,
                seed,
                passphrase,
                seed_fingerprint: seed_fingerprint.map(HexBytes),
                aindex,
                dindex,
                def_dindex,
                icon: icon.map(HexBytes),
                birth,
                position,
                use_internal,
                hidden,
                saved,
                enabled,
                internal,
                hw,
                folder: folder_name.unwrap_or_default(),
                ..Default::default()
            }
            // Process the data as needed
        })
        .fetch_one(&mut *connection)
        .await?;

    info!("Exporting transparent account");
    if let Some(t_account) =
        sqlx::query("SELECT xsk, xvk, ivk FROM transparent_accounts WHERE account = ?")
            .bind(account)
            .map(|row: SqliteRow| {
                let xsk: Option<Vec<u8>> = row.get(0);
                let xvk: Option<Vec<u8>> = row.get(1);
                let ivk: Option<Vec<u8>> = row.get(2);

                IOKeys {
                    xsk: xsk.map(HexBytes),
                    xvk: xvk.map(HexBytes),
                    ivk: ivk.map(HexBytes),
                }
            })
            .fetch_optional(&mut *connection)
            .await?
    {
        io_account.tkeys = Some(t_account);
    }

    info!("Exporting sapling account");
    if let Some(s_account) = sqlx::query("SELECT xsk, xvk, ivk FROM sapling_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let xsk: Option<Vec<u8>> = row.get(0);
            let xvk: Option<Vec<u8>> = row.get(1);
            let ivk: Option<Vec<u8>> = row.get(2);

            IOKeys {
                xsk: xsk.map(HexBytes),
                xvk: xvk.map(HexBytes),
                ivk: ivk.map(HexBytes),
            }
        })
        .fetch_optional(&mut *connection)
        .await?
    {
        io_account.skeys = Some(s_account);
    }

    info!("Exporting orchard account");
    if let Some(o_account) = sqlx::query("SELECT xsk, xvk, ivk FROM orchard_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let xsk: Option<Vec<u8>> = row.get(0);
            let xvk: Option<Vec<u8>> = row.get(1);
            let ivk: Option<Vec<u8>> = row.get(2);

            IOKeys {
                xsk: xsk.map(HexBytes),
                xvk: xvk.map(HexBytes),
                ivk: ivk.map(HexBytes),
            }
        })
        .fetch_optional(&mut *connection)
        .await?
    {
        io_account.okeys = Some(o_account);
    }

    info!("Exporting transparent addresses");
    let t_addresses = sqlx::query("SELECT id_taddress, scope, dindex, sk, pk, address, uncompressed FROM transparent_address_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let id_taddress: u32 = row.get(0);
            let scope: u32 = row.get(1);
            let dindex: u32 = row.get(2);
            let sk: Option<Vec<u8>> = row.get(3);
            let pk: Vec<u8> = row.get(4);
            let address: String = row.get(5);
            let uncompressed: bool = row.get(6);

            TAddress {
                id_taddress, scope, dindex, sk: sk.map(HexBytes),
                pk: pk.into(), address, uncompressed,
            }

        })
        .fetch_all(&mut *connection)
        .await?;
    io_account.taddrs = t_addresses;

    info!("Exporting synch heights");
    // Get the sync heights
    let sync_heights = sqlx::query("SELECT pool, height FROM sync_heights WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let pool: u8 = row.get(0);
            let height: u32 = row.get(1);
            SyncHeight {
                pool,
                height,
                time: 0,
            }
        })
        .fetch_all(&mut *connection)
        .await?;
    io_account.sync_heights = sync_heights;

    info!("Exporting checkpoints");
    // Get checkpoint heights
    let checkpoints =
        sqlx::query("SELECT DISTINCT height FROM witnesses WHERE account = ? ORDER BY height")
            .bind(account)
            .map(|row: SqliteRow| {
                let height: u32 = row.get(0);
                height
            })
            .fetch_all(&mut *connection)
            .await?;

    let mut blocks = vec![];
    for height in checkpoints.iter() {
        // Get headers for given height
        let mut block =
            sqlx::query("SELECT hash, time FROM headers WHERE height = ? AND account = ?")
                .bind(height)
                .bind(account)
                .map(|row: SqliteRow| {
                    let hash: Vec<u8> = row.get(0);
                    let time: u32 = row.get(1);

                    IOBlock {
                        height: *height,
                        hash: hash.into(),
                        time,
                        ..Default::default()
                    }
                })
                .fetch_one(&mut *connection)
                .await?;

        // Get witness for given height
        let witness =
            sqlx::query("SELECT note, witness FROM witnesses WHERE account = ? AND height = ?")
                .bind(account)
                .bind(height)
                .map(|row: SqliteRow| {
                    let note: u32 = row.get(0);
                    let witness: Vec<u8> = row.get(1);
                    IOWitness {
                        height: *height,
                        note,
                        witness: witness.into(),
                    }
                })
                .fetch_all(&mut *connection)
                .await?;
        block.witness = witness;

        blocks.push(block);
    }
    io_account.blocks = blocks;

    info!("Exporting assets");
    let assets = sqlx::query(
        "SELECT id_asset, asset_desc_hash, ik, asset_base, finalized, first_seen_height FROM assets",
    )
    .map(|row: SqliteRow| {
        let id_asset: u32 = row.get(0);
        let asset_desc_hash: Vec<u8> = row.get(1);
        let ik: Vec<u8> = row.get(2);
        let asset_base: Vec<u8> = row.get(3);
        let finalized: bool = row.get(4);
        let first_seen_height: u32 = row.get(5);
        IOAsset {
            id_asset,
            asset_desc_hash: asset_desc_hash.into(),
            ik: ik.into(),
            asset_base: asset_base.into(),
            finalized,
            first_seen_height,
        }
    })
    .fetch_all(&mut *connection)
    .await?;
    io_account.assets = assets;

    info!("Exporting categories");
    let categories = sqlx::query("SELECT id_category, name, income FROM categories")
        .map(|r: SqliteRow| {
            let id_category: u32 = r.get(0);
            let name: String = r.get(1);
            let income: bool = r.get(2);
            IOCategory {
                id_category,
                name,
                income,
            }
        })
        .fetch_all(&mut *connection)
        .await?;

    info!("Exporting transactions");
    // Get the transactions for the given account
    let mut transactions = sqlx::query(
        "SELECT id_tx, txid, height, time, details, tpe, value, fee, price, category FROM transactions WHERE account = ?",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let id_tx: u32 = row.get(0);
        let txid: Vec<u8> = row.get(1);
        let height: u32 = row.get(2);
        let time: u32 = row.get(3);
        let details: bool = row.get(4);
        let tpe: u8 = row.get(5);
        let value: i64 = row.get(6);
        let fee: u64 = row.get(7);
        let price: Option<f64> = row.get(8);
        let category: Option<u32> = row.get(9);

        IOTransaction {
            id_tx,
            txid: txid.into(),
            height,
            time,
            details,
            tpe,
            value,
            fee,
            price,
            category,
            ..Default::default()
        }
    })
    .fetch_all(&mut *connection)
    .await?;

    for tx in transactions.iter_mut() {
        // Get the notes and memos for the given transaction
        let notes = sqlx::query(
            "SELECT id_note, n.height, n.account, n.pool, n.scope, nullifier, value, cmx,
        taddress, position, diversifier, rcm, rho, locked, vout, id_asset, memo_text, memo_bytes
        FROM notes n LEFT JOIN memos m ON n.id_note = m.note WHERE n.tx = ?",
        )
        .bind(tx.id_tx)
        .map(|row: SqliteRow| {
            let id_note: u32 = row.get(0);
            let height: u32 = row.get(1);
            let account: u32 = row.get(2);
            let pool: u8 = row.get(3);
            let scope: Option<u8> = row.get(4);
            let nullifier: Vec<u8> = row.get(5);
            let value: u64 = row.get::<i64, _>(6) as u64;
            let cmx: Option<Vec<u8>> = row.get(7);
            let taddress: Option<u32> = row.get(8);
            let position: Option<u32> = row.get(9);
            let diversifier: Option<Vec<u8>> = row.get(10);
            let rcm: Option<Vec<u8>> = row.get(11);
            let rho: Option<Vec<u8>> = row.get(12);
            let locked: bool = row.get(13);
            let vout: Option<u32> = row.get(14);
            let id_asset: Option<u32> = row.get(15);
            let memo_text: Option<String> = row.get(16);
            let memo_bytes: Option<Vec<u8>> = row.get(17);

            IONote {
                id_note,
                height,
                account,
                pool,
                scope,
                nullifier: nullifier.into(),
                value,
                cmx: cmx.map(HexBytes),
                taddress,
                position,
                diversifier: diversifier.map(HexBytes),
                rcm: rcm.map(HexBytes),
                rho: rho.map(HexBytes),
                locked,
                vout,
                id_asset,
                memo_text,
                memo_bytes: memo_bytes.map(HexBytes),
            }
        })
        .fetch_all(&mut *connection)
        .await?;
        tx.notes = notes;

        // Get the outputs for the given transaction
        let outputs = sqlx::query(
            "SELECT id_output, o.account, o.height, o.pool, o.value, o.address,
            o.vout, memo_text, memo_bytes
            FROM outputs o
            LEFT JOIN memos m ON o.id_output = m.output WHERE o.tx = ?",
        )
        .bind(tx.id_tx)
        .map(|row: SqliteRow| {
            let id_output: u32 = row.get(0);
            let account: u32 = row.get(1);
            let height: u32 = row.get(2);
            let pool: u8 = row.get(3);
            let value: u64 = row.get::<i64, _>(4) as u64;
            let address: String = row.get(5);
            let vout: u32 = row.get(6);
            let memo_text: Option<String> = row.get(7);
            let memo_bytes: Option<Vec<u8>> = row.get(8);

            IOOutput {
                id_output,
                height,
                account,
                pool,
                value,
                address,
                vout,
                memo_text,
                memo_bytes: memo_bytes.map(HexBytes),
            }
        })
        .fetch_all(&mut *connection)
        .await?;
        tx.outputs = outputs;

        // Get the spends for the given transaction
        let spends =
            sqlx::query("SELECT id_note, height, account, pool, value FROM spends WHERE tx = ?")
                .bind(tx.id_tx)
                .map(|row: SqliteRow| {
                    let id_note: u32 = row.get(0);
                    let height: u32 = row.get(1);
                    let account: u32 = row.get(2);
                    let pool: u8 = row.get(3);
                    let value: u64 = row.get::<i64, _>(4) as u64;

                    IOSpend {
                        id_note,
                        height,
                        account,
                        pool,
                        value,
                    }
                })
                .fetch_all(&mut *connection)
                .await?;
        tx.spends = spends;
    }
    io_account.transactions = transactions;

    // Export user memos
    let user_memos = sqlx::query("SELECT id_tx, user_memo FROM user_memos WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let id_tx: u32 = row.get(0);
            let user_memo: String = row.get(1);
            IOUserMemo { id_tx, user_memo }
        })
        .fetch_all(&mut *connection)
        .await?;
    io_account.user_memos = user_memos;

    let dkg_params = sqlx::query(
        "SELECT id, n, t, seed, birth_height, name, dkg_version, sk
        FROM dkg_params WHERE account = ?",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let id: u8 = row.get(0);
        let n: u8 = row.get(1);
        let t: u8 = row.get(2);
        let seed: String = row.get(3);
        let birth: u32 = row.get(4);
        let name: String = row.get(5);
        let dkg_version: u8 = row.get(6);
        let sk: Option<Vec<u8>> = row.get(7);

        DKGParams {
            id,
            n,
            t,
            seed,
            birth,
            name,
            dkg_version: Some(dkg_version),
            sk: sk.map(HexBytes),
        }
    })
    .fetch_optional(&mut *connection)
    .await?;
    io_account.dkg_params = dkg_params;

    // The DKG tables as they exist since V9. `dkg_packages` is no longer
    // written by any DKG, so it is not exported; import still maps it for
    // files produced before this change.
    let dkg_state = sqlx::query(
        "SELECT signing_keypair, spkg1, spkg2, key_pkg, pending_publish
        FROM dkg_state WHERE account = ?",
    )
    .bind(account)
    .map(|row: SqliteRow| DKGState {
        signing_keypair: row.get::<Option<Vec<u8>>, _>(0).map(HexBytes),
        spkg1: row.get::<Option<Vec<u8>>, _>(1).map(HexBytes),
        spkg2: row.get::<Option<Vec<u8>>, _>(2).map(HexBytes),
        key_pkg: row.get::<Option<Vec<u8>>, _>(3).map(HexBytes),
        pending_publish: row.get::<Option<Vec<u8>>, _>(4).map(HexBytes),
    })
    .fetch_optional(&mut *connection)
    .await?;
    io_account.dkg_state = dkg_state;

    let dkg_peers = sqlx::query("SELECT round, from_id, data FROM dkg_peers WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| DKGPeer {
            round: row.get(0),
            from_id: row.get(1),
            data: HexBytes(row.get(2)),
        })
        .fetch_all(&mut *connection)
        .await?;
    io_account.dkg_peers = dkg_peers;

    let dkg_addresses = sqlx::query("SELECT from_id, address FROM dkg_addresses WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| DKGAddress {
            from_id: row.get(0),
            address: row.get(1),
        })
        .fetch_all(&mut *connection)
        .await?;
    io_account.dkg_addresses = dkg_addresses;
    io_account.categories = categories;

    let io_account = serde_json::to_string(&io_account)?;
    let mut encoder = zstd::Encoder::new(Vec::new(), DEFAULT_COMPRESSION_LEVEL)?;
    encoder.write_all(io_account.as_bytes())?;
    let data = encoder.finish()?;

    info!("Exported account size: {}", data.len());
    Ok(data)
}

/// Import an exported account. For a caller holding the sync session: the
/// sync driver, so a sync cannot pick the account up half-written.
pub async fn import_account(
    network: &Network,
    connection: &mut crate::sync::session::Held<'_>,
    data: &[u8],
) -> Result<()> {
    let mut decoder = zstd::Decoder::new(data)?;
    let mut data = String::new();
    decoder.read_to_string(&mut data)?;
    let io_account = serde_json::from_str::<IOAccount>(&data)?;
    if io_account.version > DB_VERSION {
        anyhow::bail!("This backup was created by a newer app version. Please update.");
    }

    info!("Importing account {}", io_account.name);
    let mut tx = connection.begin().await?;
    // Move all accounts down by one position
    sqlx::query("UPDATE accounts SET position = position + 1")
        .execute(&mut *tx)
        .await?;

    let mut category_map: HashMap<u32, u32> = HashMap::new();
    for category in io_account.categories.iter() {
        let new_id_category = sqlx::query(
            "INSERT INTO categories(name, income)
            VALUES (?,?) ON CONFLICT DO UPDATE SET name = excluded.name
            RETURNING (id_category)",
        )
        .bind(&category.name)
        .bind(category.income)
        .map(|r: SqliteRow| r.get::<u32, _>(0))
        .fetch_one(&mut *tx)
        .await?;
        category_map.insert(category.id_category, new_id_category);
    }

    let mut new_assets: HashMap<u32, u32> = HashMap::new();
    for asset in io_account.assets.iter() {
        chain_log::record_asset(
            &mut tx,
            &AssetRow {
                desc_hash: &asset.asset_desc_hash.0,
                ik: &asset.ik.0,
                base: &asset.asset_base.0,
                name: None,
                finalized: asset.finalized,
                first_seen_height: asset.first_seen_height,
            },
        )
        .await?;
        // Look up the id_asset (either newly created or existing)
        let new_id_asset = chain_log::asset_id(&mut tx, &asset.asset_desc_hash.0, &asset.ik.0)
            .await?
            .ok_or_else(|| anyhow!("asset {} was not stored", asset.id_asset))?;
        new_assets.insert(asset.id_asset, new_id_asset);
    }

    let mut folder: Option<u32> = None;
    let folder_name = &io_account.folder;
    if !folder_name.is_empty() {
        sqlx::query("INSERT OR IGNORE INTO folders(name) VALUES (?1)")
            .bind(folder_name)
            .execute(&mut *tx)
            .await?;
        folder = sqlx::query("SELECT id_folder FROM folders WHERE name = ?1")
            .bind(folder_name)
            .map(|row: SqliteRow| {
                let id_folder: u32 = row.get(0);
                id_folder
            })
            .fetch_optional(&mut *tx)
            .await?;
    }

    // Insert the account into the database
    let r = sqlx::query("INSERT INTO accounts
        (name, seed, passphrase, seed_fingerprint, aindex, dindex, def_dindex, icon, birth, folder, position, use_internal, hidden, saved, enabled, internal, hw)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?, ?, ?, ?, ?)")
        .bind(&io_account.name)
        .bind(&io_account.seed)
        .bind(&io_account.passphrase)
        .bind(&io_account.seed_fingerprint)
        .bind(io_account.aindex)
        .bind(io_account.dindex)
        .bind(io_account.def_dindex)
        .bind(&io_account.icon)
        .bind(io_account.birth)
        .bind(folder)
        .bind(io_account.use_internal)
        .bind(io_account.hidden)
        .bind(io_account.saved)
        .bind(io_account.enabled)
        .bind(io_account.internal)
        .bind(io_account.hw)
        .execute(&mut *tx)
        .await?;
    let new_id_account = r.last_insert_rowid() as u32;
    // account must be replaced by new_id_account

    info!("Importing transparent key");
    if let Some(tkeys) = io_account.tkeys.as_ref() {
        sqlx::query(
            "INSERT INTO transparent_accounts
            (account, xsk, xvk, ivk) VALUES (?, ?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(&tkeys.xsk)
        .bind(&tkeys.xvk)
        .bind(&tkeys.ivk)
        .execute(&mut *tx)
        .await?;
    }
    info!("Importing sapling key");
    if let Some(skeys) = io_account.skeys.as_ref() {
        sqlx::query(
            "INSERT INTO sapling_accounts
            (account, xsk, xvk, ivk) VALUES (?, ?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(&skeys.xsk)
        .bind(&skeys.xvk)
        .bind(&skeys.ivk)
        .execute(&mut *tx)
        .await?;
        // The backup file carries only the sapling keys; the address column
        // would stay at its empty default, which select_account_sapling
        // cannot decode (it panicked on it). Derive the address from the
        // imported keys, mirroring new_account's derivation: from the full
        // key, or from the incoming key of an incoming-only account.
        let address = match (skeys.xsk.as_ref(), skeys.xvk.as_ref(), skeys.ivk.as_ref()) {
            (Some(xsk), _, _) => {
                let sxvk = ExtendedSpendingKey::from_bytes(&xsk.0)
                    .map_err(|_| anyhow!("invalid sapling xsk in account backup"))?
                    .to_diversifiable_full_viewing_key();
                crate::account::derive_sapling_address(network, &sxvk, io_account.dindex)
            }
            (None, Some(xvk), _) => {
                let bytes: [u8; 128] = xvk
                    .0
                    .as_slice()
                    .try_into()
                    .map_err(|_| anyhow!("invalid sapling xvk length in account backup"))?;
                let sxvk = DiversifiableFullViewingKey::from_bytes(&bytes)
                    .ok_or_else(|| anyhow!("invalid sapling xvk in account backup"))?;
                crate::account::derive_sapling_address(network, &sxvk, io_account.dindex)
            }
            (None, None, Some(ivk)) => {
                let bytes: [u8; 64] = ivk
                    .0
                    .as_slice()
                    .try_into()
                    .map_err(|_| anyhow!("invalid sapling ivk length in account backup"))?;
                let ivk = sapling_crypto::zip32::IncomingViewingKey::from_bytes(&bytes)
                    .into_option()
                    .ok_or_else(|| anyhow!("invalid sapling ivk in account backup"))?;
                ivk.address_at(io_account.dindex)
                    .ok_or_else(|| anyhow!("no sapling address at the backup's diversifier index"))?
                    .encode(network)
            }
            (None, None, None) => {
                anyhow::bail!("the sapling keys in the account backup carry no viewing key")
            }
        };
        sqlx::query("UPDATE sapling_accounts SET address = ?2 WHERE account = ?1")
            .bind(new_id_account)
            .bind(&address)
            .execute(&mut *tx)
            .await?;
    }
    info!("Importing orchard key");
    if let Some(okeys) = io_account.okeys.as_ref() {
        sqlx::query(
            "INSERT INTO orchard_accounts
            (account, xsk, xvk, ivk) VALUES (?, ?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(&okeys.xsk)
        .bind(&okeys.xvk)
        .bind(&okeys.ivk)
        .execute(&mut *tx)
        .await?;
    }
    info!("Importing transparent addresses");
    let mut new_taddresses = HashMap::<u32, u32>::new();
    for taddr in io_account.taddrs.iter() {
        let r = sqlx::query(
            "INSERT INTO transparent_address_accounts
            (account, scope, dindex, sk, pk, address, uncompressed) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(taddr.scope)
        .bind(taddr.dindex)
        .bind(&taddr.sk)
        .bind(&taddr.pk)
        .bind(&taddr.address)
        .bind(taddr.uncompressed)
        .execute(&mut *tx)
        .await?;
        let new_id_taddress = r.last_insert_rowid() as u32;
        new_taddresses.insert(taddr.id_taddress, new_id_taddress);
    }

    info!("Importing sync heights");
    for sync_height in io_account.sync_heights.iter() {
        chain_log::set_checkpoint(&mut tx, new_id_account, sync_height.pool, sync_height.height)
            .await?;
    }

    info!("Importing transactions");
    let mut new_txs = HashMap::<u32, u32>::new();
    let mut new_notes = HashMap::<u32, u32>::new();
    for transaction in io_account.transactions.iter() {
        let category = transaction
            .category
            .and_then(|c| category_map.get(&c))
            .cloned();
        let new_id_tx = chain_log::record_transaction(
            &mut tx,
            new_id_account,
            &TransactionRow {
                txid: &transaction.txid.0,
                height: transaction.height,
                time: transaction.time,
                details: transaction.details,
                tpe: Some(transaction.tpe),
                value: transaction.value,
                fee: transaction.fee as i64,
                price: transaction.price,
                category,
            },
        )
        .await?
        .ok_or_else(|| anyhow!("duplicate transaction {} in backup", transaction.id_tx))?;
        new_txs.insert(transaction.id_tx, new_id_tx);

        for note in transaction.notes.iter() {
            let new_taddress = note
                .taddress
                .and_then(|id_taddress| new_taddresses.get(&id_taddress));
            let new_id_asset = note.id_asset.and_then(|id_asset| new_assets.get(&id_asset));
            let new_id_note = chain_log::record_note(
                &mut tx,
                new_id_account,
                &NoteRow {
                    id_tx: new_id_tx,
                    height: note.height,
                    pool: note.pool,
                    scope: note.scope,
                    nullifier: &note.nullifier.0,
                    value: note.value as i64,
                    cmx: note.cmx.as_ref().map(|b| b.0.as_slice()),
                    taddress: new_taddress.copied(),
                    position: note.position,
                    diversifier: note.diversifier.as_ref().map(|b| b.0.as_slice()),
                    rcm: note.rcm.as_ref().map(|b| b.0.as_slice()),
                    rho: note.rho.as_ref().map(|b| b.0.as_slice()),
                    locked: note.locked,
                    id_asset: new_id_asset.copied(),
                    diversifier_index: None,
                },
            )
            .await?
            .ok_or_else(|| anyhow!("duplicate note {} in backup", note.id_note))?;
            new_notes.insert(note.id_note, new_id_note);

            if let (Some(vout), Some(memo_bytes)) = (note.vout, &note.memo_bytes) {
                chain_log::record_memo(
                    &mut tx,
                    new_id_account,
                    &MemoRow {
                        id_tx: new_id_tx,
                        height: note.height,
                        pool: note.pool,
                        vout,
                        note: Some(new_id_note),
                        output: None,
                        text: note.memo_text.as_deref(),
                        bytes: &memo_bytes.0,
                    },
                )
                .await?;
            }
        }

        for output in transaction.outputs.iter() {
            let new_id_output = chain_log::record_output(
                &mut tx,
                new_id_account,
                &OutputRow {
                    id_tx: new_id_tx,
                    height: output.height,
                    pool: output.pool,
                    vout: output.vout,
                    value: output.value as i64,
                    address: &output.address,
                },
            )
            .await?;

            if let Some(memo_bytes) = &output.memo_bytes {
                chain_log::record_memo(
                    &mut tx,
                    new_id_account,
                    &MemoRow {
                        id_tx: new_id_tx,
                        height: output.height,
                        pool: output.pool,
                        vout: output.vout,
                        note: None,
                        output: Some(new_id_output),
                        text: output.memo_text.as_deref(),
                        bytes: &memo_bytes.0,
                    },
                )
                .await?;
            }
        }
    }

    info!("Importing spends");
    for transaction in io_account.transactions.iter() {
        let new_id_tx = new_txs
            .get(&transaction.id_tx)
            .expect("new_id_tx not found");
        for spend in transaction.spends.iter() {
            let new_id_note = new_notes
                .get(&spend.id_note)
                .ok_or_else(|| anyhow!("spend of unknown note {} in backup", spend.id_note))?;
            chain_log::record_spend(
                &mut tx,
                new_id_account,
                &SpendRow {
                    note: NoteKey::Id(*new_id_note),
                    id_tx: *new_id_tx,
                    height: spend.height,
                    pool: spend.pool,
                    value: spend.value as i64,
                },
            )
            .await?;
        }
    }

    info!("Importing user memos");
    for user_memo in io_account.user_memos.iter() {
        if let Some(new_id_tx) = new_txs.get(&user_memo.id_tx) {
            sqlx::query(
                "INSERT INTO user_memos (account, id_tx, height, user_memo)
                SELECT account, id_tx, height, ? FROM transactions
                WHERE account = ? AND id_tx = ?",
            )
            .bind(&user_memo.user_memo)
            .bind(new_id_account)
            .bind(new_id_tx)
            .execute(&mut *tx)
            .await?;
        }
    }

    info!("Importing checkpoints");
    for block in io_account.blocks.iter() {
        chain_log::store_header(
            &mut tx,
            new_id_account,
            &BlockHeader {
                height: block.height,
                hash: block.hash.0.clone(),
                time: block.time,
            },
        )
        .await?;

        for witness in block.witness.iter() {
            let new_id_note = new_notes
                .get(&witness.note)
                .ok_or_else(|| anyhow!("witness of unknown note {} in backup", witness.note))?;
            chain_log::record_witness(
                &mut tx,
                new_id_account,
                NoteKey::Id(*new_id_note),
                witness.height,
                &witness.witness.0,
            )
            .await?;
        }
    }

    if let Some(dkg_params) = &io_account.dkg_params {
        // A file from before the version column: the same rule as the
        // column backfill, a completed DKG with no round-0 package is dkg-0.
        let dkg_version = dkg_params.dkg_version.unwrap_or_else(|| {
            let completed = io_account
                .dkg_state
                .as_ref()
                .is_some_and(|s| s.key_pkg.is_some())
                || io_account
                    .dkg_packages
                    .iter()
                    .any(|p| !p.public && p.round == 3);
            let has_round0 = io_account.dkg_peers.iter().any(|p| p.round == 0);
            if completed && !has_round0 {
                DKG_VERSION_0
            } else {
                DKG_VERSION_1
            }
        });
        sqlx::query(
            "INSERT INTO dkg_params
            (account, id, n, t, seed, birth_height, name, dkg_version, sk)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(dkg_params.id)
        .bind(dkg_params.n)
        .bind(dkg_params.t)
        .bind(&dkg_params.seed)
        .bind(dkg_params.birth)
        .bind(&dkg_params.name)
        .bind(dkg_version)
        .bind(dkg_params.sk.as_ref().map(|b| &b.0))
        .execute(&mut *tx)
        .await?;
    }

    if let Some(dkg_state) = io_account.dkg_state {
        sqlx::query(
            "INSERT INTO dkg_state
            (account, signing_keypair, spkg1, spkg2, key_pkg, pending_publish)
            VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(dkg_state.signing_keypair.as_ref().map(|b| &b.0))
        .bind(dkg_state.spkg1.as_ref().map(|b| &b.0))
        .bind(dkg_state.spkg2.as_ref().map(|b| &b.0))
        .bind(dkg_state.key_pkg.as_ref().map(|b| &b.0))
        .bind(dkg_state.pending_publish.as_ref().map(|b| &b.0))
        .execute(&mut *tx)
        .await?;
    }

    for peer in io_account.dkg_peers.iter() {
        sqlx::query(
            "INSERT OR IGNORE INTO dkg_peers (account, round, from_id, data)
            VALUES (?, ?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(peer.round)
        .bind(peer.from_id)
        .bind(&peer.data.0)
        .execute(&mut *tx)
        .await?;
    }

    for address in io_account.dkg_addresses.iter() {
        sqlx::query(
            "INSERT OR IGNORE INTO dkg_addresses (account, from_id, address)
            VALUES (?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(address.from_id)
        .bind(&address.address)
        .execute(&mut *tx)
        .await?;
    }

    // Files written before the V9 tables carry `dkg_packages`. Map them the
    // way the V9 migration does: round 0 public rows are addresses, other
    // public rows are peer packages, private rows 1/2/3 are the state
    // columns. OR IGNORE keeps the sections above authoritative when a file
    // has both.
    import_legacy_dkg_packages(&mut tx, new_id_account, &io_account.dkg_packages).await?;
    tx.commit().await?;

    Ok(())
}

async fn import_legacy_dkg_packages(
    tx: &mut SqliteConnection,
    account: u32,
    packages: &[DKGPackage],
) -> Result<()> {
    if packages.is_empty() {
        return Ok(());
    }
    let has_secrets = packages
        .iter()
        .any(|p| !p.public && (1..=3).contains(&p.round));
    if has_secrets {
        sqlx::query("INSERT OR IGNORE INTO dkg_state (account) VALUES (?)")
            .bind(account)
            .execute(&mut *tx)
            .await?;
    }
    for p in packages {
        match (p.public, p.round) {
            (true, 0) => {
                let address = String::from_utf8(p.data.0.clone())
                    .context("legacy dkg address is not UTF-8")?;
                sqlx::query(
                    "INSERT OR IGNORE INTO dkg_addresses (account, from_id, address)
                    VALUES (?, ?, ?)",
                )
                .bind(account)
                .bind(p.from_id)
                .bind(address)
                .execute(&mut *tx)
                .await?;
            }
            (true, 1..=3) => {
                sqlx::query(
                    "INSERT OR IGNORE INTO dkg_peers (account, round, from_id, data)
                    VALUES (?, ?, ?, ?)",
                )
                .bind(account)
                .bind(p.round)
                .bind(p.from_id)
                .bind(&p.data.0)
                .execute(&mut *tx)
                .await?;
            }
            (false, round @ 1..=3) => {
                let column = match round {
                    1 => "spkg1",
                    2 => "spkg2",
                    _ => "key_pkg",
                };
                // Audited: `column` is one of the three literals above.
                sqlx::query(sqlx::AssertSqlSafe(format!(
                    "UPDATE dkg_state SET {column} = ?1 WHERE account = ?2 AND {column} IS NULL"
                )))
                .bind(&p.data.0)
                .bind(account)
                .execute(&mut *tx)
                .await?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct IOAccount {
    pub version: u16,
    pub id_account: u32,
    pub name: String,
    pub seed: Option<String>,
    pub passphrase: String,
    pub seed_fingerprint: Option<HexBytes>,
    pub aindex: u32,
    pub dindex: u32,
    pub def_dindex: u32,
    pub icon: Option<HexBytes>,
    pub birth: u32,
    pub folder: String,
    pub position: u32,
    pub use_internal: bool,
    pub hidden: bool,
    pub saved: bool,
    pub enabled: bool,
    pub internal: bool,
    pub hw: u8,
    pub tkeys: Option<IOKeys>,
    pub skeys: Option<IOKeys>,
    pub okeys: Option<IOKeys>,
    pub taddrs: Vec<TAddress>,
    pub sync_heights: Vec<SyncHeight>,
    pub blocks: Vec<IOBlock>,
    pub transactions: Vec<IOTransaction>,
    pub dkg_params: Option<DKGParams>,
    /// Pre-V9 layout; read on import only.
    #[serde(default)]
    pub dkg_packages: Vec<DKGPackage>,
    #[serde(default)]
    pub dkg_state: Option<DKGState>,
    #[serde(default)]
    pub dkg_peers: Vec<DKGPeer>,
    #[serde(default)]
    pub dkg_addresses: Vec<DKGAddress>,
    pub categories: Vec<IOCategory>,
    pub assets: Vec<IOAsset>,
    pub user_memos: Vec<IOUserMemo>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct IOAsset {
    pub id_asset: u32,
    pub asset_desc_hash: HexBytes,
    pub ik: HexBytes,
    pub asset_base: HexBytes,
    pub finalized: bool,
    pub first_seen_height: u32,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
/// One pool's keys. A full account has `xvk`; an incoming-only account,
/// created from an incoming viewing key, has `ivk` alone and no `xvk`.
pub struct IOKeys {
    pub xsk: Option<HexBytes>,
    pub xvk: Option<HexBytes>,
    #[serde(default)]
    pub ivk: Option<HexBytes>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct TAddress {
    pub id_taddress: u32,
    pub scope: u32,
    pub dindex: u32,
    pub sk: Option<HexBytes>,
    pub pk: HexBytes,
    pub address: String,
    /// Absent from backups written before uncompressed keys were supported,
    /// which could hold none.
    #[serde(default)]
    pub uncompressed: bool,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct SyncHeight {
    pub pool: u8,
    pub height: u32,
    pub time: u32,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct IOBlock {
    pub height: u32,
    pub hash: HexBytes,
    pub time: u32,
    pub witness: Vec<IOWitness>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct IOWitness {
    pub height: u32,
    pub note: u32,
    pub witness: HexBytes,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct IOCategory {
    pub id_category: u32,
    pub name: String,
    pub income: bool,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct IOTransaction {
    pub id_tx: u32,
    pub txid: HexBytes,
    pub height: u32,
    pub time: u32,
    pub details: bool,
    pub tpe: u8,
    pub value: i64,
    pub fee: u64,
    pub price: Option<f64>,
    pub category: Option<u32>,
    pub notes: Vec<IONote>,
    pub spends: Vec<IOSpend>,
    pub outputs: Vec<IOOutput>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct IONote {
    pub id_note: u32,
    pub height: u32,
    pub account: u32,
    pub pool: u8,
    pub scope: Option<u8>,
    pub nullifier: HexBytes,
    pub value: u64,
    pub cmx: Option<HexBytes>,
    pub taddress: Option<u32>,
    pub position: Option<u32>,
    pub diversifier: Option<HexBytes>,
    pub rcm: Option<HexBytes>,
    pub rho: Option<HexBytes>,
    pub locked: bool,
    pub vout: Option<u32>,
    pub id_asset: Option<u32>,
    pub memo_text: Option<String>,
    pub memo_bytes: Option<HexBytes>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct IOSpend {
    pub id_note: u32,
    pub height: u32,
    pub account: u32,
    pub pool: u8,
    pub value: u64,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct IOUserMemo {
    pub id_tx: u32,
    pub user_memo: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct IOOutput {
    pub id_output: u32,
    pub height: u32,
    pub account: u32,
    pub pool: u8,
    pub value: u64,
    pub address: String,
    pub vout: u32,
    pub memo_text: Option<String>,
    pub memo_bytes: Option<HexBytes>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct DKGParams {
    pub id: u8,
    pub n: u8,
    pub t: u8,
    pub seed: String,
    pub birth: u32,
    #[serde(default)]
    pub name: String,
    /// Absent in files written before the version column; import infers it.
    #[serde(default)]
    pub dkg_version: Option<u8>,
    /// The broadcast account's spending key, dkg-2 accounts only.
    #[serde(default)]
    pub sk: Option<HexBytes>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct DKGState {
    pub signing_keypair: Option<HexBytes>,
    pub spkg1: Option<HexBytes>,
    pub spkg2: Option<HexBytes>,
    pub key_pkg: Option<HexBytes>,
    pub pending_publish: Option<HexBytes>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct DKGPeer {
    pub round: u8,
    pub from_id: u8,
    pub data: HexBytes,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct DKGAddress {
    pub from_id: u8,
    pub address: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct DKGPackage {
    pub account: u32,
    pub public: bool,
    pub round: u32,
    pub from_id: u16,
    pub data: HexBytes,
}

pub fn encrypt(passphrase: &str, data: &[u8]) -> Result<Vec<u8>> {
    let encryptor = Encryptor::with_user_passphrase(passphrase.into());
    let mut ciphertext = vec![];
    let mut writer = encryptor.wrap_output(&mut ciphertext)?;
    writer.write_all(data)?;
    writer.finish()?;
    Ok(ciphertext)
}

pub fn decrypt(passphrase: &str, data: &[u8]) -> Result<Vec<u8>> {
    let decryptor = Decryptor::new_buffered(data)?;
    let identity = Identity::new(passphrase.into());
    let identity: Vec<&dyn age::Identity> = vec![&identity];
    let mut reader = decryptor.decrypt(identity.into_iter())?;
    let mut output = vec![];
    reader.read_to_end(&mut output)?;
    Ok(output)
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct HexBytes(#[serde_as(as = "Hex")] pub Vec<u8>);

impl From<Vec<u8>> for HexBytes {
    fn from(value: Vec<u8>) -> Self {
        HexBytes(value)
    }
}

impl From<HexBytes> for Vec<u8> {
    fn from(value: HexBytes) -> Self {
        value.0
    }
}

impl<'q> Encode<'q, Sqlite> for HexBytes {
    fn encode(self, args: &mut SqliteArgumentsBuffer) -> Result<IsNull, BoxDynError> {
        <Vec<u8> as Encode<'q, Sqlite>>::encode(self.0, args)
    }

    fn encode_by_ref(&self, args: &mut SqliteArgumentsBuffer) -> Result<IsNull, BoxDynError> {
        <Vec<u8> as Encode<'q, Sqlite>>::encode_by_ref(&self.0, args)
    }
}

impl<'r> Decode<'r, Sqlite> for HexBytes {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let inner = <Vec<u8> as Decode<'r, Sqlite>>::decode(value)?;
        Ok(HexBytes(inner))
    }
}

impl Type<Sqlite> for HexBytes {
    fn type_info() -> <sqlx::Sqlite as sqlx::Database>::TypeInfo {
        <Vec<u8> as Type<Sqlite>>::type_info()
    }
}

#[cfg(test)]
mod incoming_only_export_tests {
    use super::*;
    use crate::{account::new_account, api::account::NewAccount, db::create_schema};
    use sqlx::Connection;
    use zcash_keys::keys::UnifiedSpendingKey;
    use zip32::AccountId;

    const NETWORK: Network = Network::Main;

    type KeyRow = (Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>);

    async fn key_rows(db: &mut SqliteConnection, account: u32) -> Result<(KeyRow, KeyRow, KeyRow, String)> {
        let t = sqlx::query_as("SELECT xsk, xvk, ivk FROM transparent_accounts WHERE account = ?")
            .bind(account)
            .fetch_one(&mut *db)
            .await?;
        let s = sqlx::query_as("SELECT xsk, xvk, ivk FROM sapling_accounts WHERE account = ?")
            .bind(account)
            .fetch_one(&mut *db)
            .await?;
        let o = sqlx::query_as("SELECT xsk, xvk, ivk FROM orchard_accounts WHERE account = ?")
            .bind(account)
            .fetch_one(&mut *db)
            .await?;
        let (address,): (String,) =
            sqlx::query_as("SELECT address FROM sapling_accounts WHERE account = ?")
                .bind(account)
                .fetch_one(&mut *db)
                .await?;
        Ok((t, s, o, address))
    }

    /// An incoming-only account survives the encrypted export: the imported
    /// account holds the same incoming keys, no full key, and the same
    /// sapling address, derived from the incoming key.
    /// `(uncompressed, length(pk))` of the account's transparent address rows.
    async fn taddr_rows(db: &mut SqliteConnection, account: u32) -> Result<Vec<(bool, i64)>> {
        Ok(sqlx::query_as(
            "SELECT uncompressed, length(pk) FROM transparent_address_accounts
            WHERE account = ? ORDER BY id_taddress",
        )
        .bind(account)
        .fetch_all(&mut *db)
        .await?)
    }

    #[tokio::test]
    async fn an_uncompressed_wif_account_round_trips_with_its_flag() -> Result<()> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        // A WIF without the compression byte: 0x80 || key, base58check.
        let wif = bs58::encode([7u8; 32]).with_check_version(0x80).into_string();
        let na = NewAccount {
            icon: None,
            name: "legacy".into(),
            restore: true,
            key: wif,
            passphrase: None,
            fingerprint: None,
            aindex: 0,
            birth: Some(1),
            folder: String::new(),
            pools: None,
            use_internal: false,
            internal: false,
            hw: 0,
        };
        let account = new_account(&NETWORK, &mut db, &na).await?;
        assert_eq!(taddr_rows(&mut db, account).await?, vec![(true, 65)]);

        let data = export_account(&mut db, account).await?;
        let session = crate::sync::session::SyncSession::new();
        let run = session.try_begin().unwrap();
        import_account(&NETWORK, &mut run.held(&mut db), &data).await?;
        let (imported,): (u32,) = sqlx::query_as("SELECT MAX(id_account) FROM accounts")
            .fetch_one(&mut db)
            .await?;
        assert_ne!(imported, account);
        assert_eq!(
            taddr_rows(&mut db, imported).await?,
            vec![(true, 65)],
            "the restored address keeps the flag its key was hashed with"
        );
        Ok(())
    }

    #[tokio::test]
    async fn opening_a_wallet_repairs_flags_lost_by_an_earlier_import() -> Result<()> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        for (pk_len, id) in [(65usize, 1), (33, 2)] {
            sqlx::query(
                "INSERT INTO transparent_address_accounts(account, scope, dindex, pk, address, uncompressed)
                VALUES (?, 0, 0, ?, 't1', FALSE)",
            )
            .bind(id)
            .bind(vec![4u8; pk_len])
            .execute(&mut db)
            .await?;
        }
        create_schema(&mut db).await?;
        assert_eq!(taddr_rows(&mut db, 1).await?, vec![(true, 65)], "repaired");
        assert_eq!(taddr_rows(&mut db, 2).await?, vec![(false, 33)], "untouched");
        Ok(())
    }

    #[tokio::test]
    async fn an_incoming_only_account_round_trips() -> Result<()> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        let seed = bip39::Mnemonic::parse(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )?
        .to_seed("");
        let uivk = UnifiedSpendingKey::from_seed(&NETWORK, &seed, AccountId::ZERO)?
            .to_unified_full_viewing_key()
            .to_unified_incoming_viewing_key()
            .encode(&NETWORK)?;
        let na = NewAccount {
            icon: None,
            name: "watch".into(),
            restore: true,
            key: uivk,
            passphrase: None,
            fingerprint: None,
            aindex: 0,
            birth: Some(1),
            folder: String::new(),
            pools: Some(7),
            use_internal: false,
            internal: false,
            hw: 0,
        };
        let account = new_account(&NETWORK, &mut db, &na).await?;
        let before = key_rows(&mut db, account).await?;
        assert!(before.1 .1.is_none() && before.1 .2.is_some(), "incoming-only sapling row");

        let data = export_account(&mut db, account).await?;
        let session = crate::sync::session::SyncSession::new();
        let run = session.try_begin().unwrap();
        import_account(&NETWORK, &mut run.held(&mut db), &data).await?;
        let (imported,): (u32,) = sqlx::query_as("SELECT MAX(id_account) FROM accounts")
            .fetch_one(&mut db)
            .await?;
        assert_ne!(imported, account);
        assert_eq!(key_rows(&mut db, imported).await?, before);
        assert!(crate::db::is_incoming_only(&mut db, imported).await?);
        Ok(())
    }
}

#[cfg(test)]
mod dkg_export_tests {
    use super::*;
    use crate::db::create_schema;
    use sqlx::Connection;

    async fn fresh_db() -> Result<SqliteConnection> {
        let mut db = SqliteConnection::connect("sqlite::memory:").await?;
        create_schema(&mut db).await?;
        Ok(db)
    }

    async fn bare_account(db: &mut SqliteConnection, name: &str) -> Result<u32> {
        let id: u32 = sqlx::query(
            "INSERT INTO accounts(name, aindex, dindex, def_dindex, birth, position,
            use_internal, hidden, saved) VALUES (?, 0, 0, 0, 0, 0, 0, 0, 0)
            RETURNING id_account",
        )
        .bind(name)
        .map(|r: SqliteRow| r.get(0))
        .fetch_one(&mut *db)
        .await?;
        Ok(id)
    }

    async fn dkg_rows(db: &mut SqliteConnection, account: u32) -> Result<DkgSnapshot> {
        let params: DkgParamsRow = sqlx::query_as(
            "SELECT id, n, t, seed, birth_height, name, dkg_version, sk
            FROM dkg_params WHERE account = ?",
        )
        .bind(account)
        .fetch_optional(&mut *db)
        .await?;
        let state: Option<(
            Option<Vec<u8>>,
            Option<Vec<u8>>,
            Option<Vec<u8>>,
            Option<Vec<u8>>,
            Option<Vec<u8>>,
        )> = sqlx::query_as(
            "SELECT signing_keypair, spkg1, spkg2, key_pkg, pending_publish
            FROM dkg_state WHERE account = ?",
        )
        .bind(account)
        .fetch_optional(&mut *db)
        .await?;
        let peers: Vec<(u8, u8, Vec<u8>)> = sqlx::query_as(
            "SELECT round, from_id, data FROM dkg_peers WHERE account = ? ORDER BY round, from_id",
        )
        .bind(account)
        .fetch_all(&mut *db)
        .await?;
        let addresses: Vec<(u8, String)> = sqlx::query_as(
            "SELECT from_id, address FROM dkg_addresses WHERE account = ? ORDER BY from_id",
        )
        .bind(account)
        .fetch_all(&mut *db)
        .await?;
        Ok(DkgSnapshot {
            params,
            state,
            peers,
            addresses,
        })
    }

    /// (id, n, t, seed, birth_height, name, dkg_version, sk)
    type DkgParamsRow = Option<(u8, u8, u8, String, u32, String, u8, Option<Vec<u8>>)>;

    #[derive(Debug, PartialEq)]
    struct DkgSnapshot {
        params: DkgParamsRow,
        state: Option<(
            Option<Vec<u8>>,
            Option<Vec<u8>>,
            Option<Vec<u8>>,
            Option<Vec<u8>>,
            Option<Vec<u8>>,
        )>,
        peers: Vec<(u8, u8, Vec<u8>)>,
        addresses: Vec<(u8, String)>,
    }

    async fn newest_account(db: &mut SqliteConnection) -> Result<u32> {
        let (id,): (u32,) = sqlx::query_as("SELECT MAX(id_account) FROM accounts")
            .fetch_one(&mut *db)
            .await?;
        Ok(id)
    }

    /// A FROST account completed on the V9 tables exports and imports every
    /// DKG row: parameters, state (secrets, key package), peer packages and
    /// addresses.
    #[tokio::test]
    async fn v9_dkg_tables_round_trip() -> Result<()> {
        let mut src = fresh_db().await?;
        let account = bare_account(&mut src, "frost").await?;
        sqlx::query(
            "INSERT INTO dkg_params(account, id, n, t, seed, birth_height, name, dkg_version, sk)
            VALUES (?, 2, 3, 2, 'mailbox words', 100, 'group', 2, X'AA')",
        )
        .bind(account)
        .execute(&mut src)
        .await?;
        sqlx::query(
            "INSERT INTO dkg_state(account, signing_keypair, spkg1, spkg2, key_pkg, pending_publish)
            VALUES (?, X'01', X'02', NULL, X'04', NULL)",
        )
        .bind(account)
        .execute(&mut src)
        .await?;
        for (round, from_id, data) in [
            (0u8, 1u8, vec![0x10u8]),
            (1, 3, vec![0x11]),
            (3, 2, vec![0x33]),
        ] {
            sqlx::query("INSERT INTO dkg_peers(account, round, from_id, data) VALUES (?, ?, ?, ?)")
                .bind(account)
                .bind(round)
                .bind(from_id)
                .bind(data)
                .execute(&mut src)
                .await?;
        }
        for (from_id, address) in [
            (1u8, "uregtest1aaa"),
            (2, "uregtest1bbb"),
            (3, "uregtest1ccc"),
        ] {
            sqlx::query("INSERT INTO dkg_addresses(account, from_id, address) VALUES (?, ?, ?)")
                .bind(account)
                .bind(from_id)
                .bind(address)
                .execute(&mut src)
                .await?;
        }
        let before = dkg_rows(&mut src, account).await?;
        assert!(before.params.is_some() && before.state.is_some());

        let data = export_account(&mut src, account).await?;
        let mut dst = fresh_db().await?;
        let session = crate::sync::session::SyncSession::new();
        let run = session.try_begin().unwrap();
        import_account(&Network::Main, &mut run.held(&mut dst), &data).await?;
        let imported = newest_account(&mut dst).await?;
        let after = dkg_rows(&mut dst, imported).await?;
        assert_eq!(after, before);
        Ok(())
    }

    /// An export file from before the V9 tables carries `dkg_packages`;
    /// import maps it onto the V9 tables the way the migration does.
    #[tokio::test]
    async fn legacy_dkg_packages_import_into_v9_tables() -> Result<()> {
        let package = |public: bool, round: u32, from_id: u16, data: &[u8]| DKGPackage {
            account: 7,
            public,
            round,
            from_id,
            data: HexBytes(data.to_vec()),
        };
        let io_account = IOAccount {
            version: DB_VERSION,
            name: "frost-legacy".to_string(),
            dkg_params: Some(DKGParams {
                id: 1,
                n: 2,
                t: 2,
                seed: "mailbox words".to_string(),
                birth: 100,
                name: String::new(),
                dkg_version: None,
                sk: None,
            }),
            dkg_packages: vec![
                package(true, 0, 1, b"uregtest1aaa"),
                package(true, 0, 2, b"uregtest1bbb"),
                package(true, 1, 2, &[0x11]),
                package(true, 3, 2, &[0x33]),
                package(false, 1, 1, &[0x02]),
                package(false, 2, 1, &[0x03]),
                package(false, 3, 1, &[0x04]),
            ],
            ..Default::default()
        };
        let json = serde_json::to_string(&io_account)?;
        let mut encoder = zstd::Encoder::new(Vec::new(), DEFAULT_COMPRESSION_LEVEL)?;
        encoder.write_all(json.as_bytes())?;
        let data = encoder.finish()?;

        let mut dst = fresh_db().await?;
        let session = crate::sync::session::SyncSession::new();
        let run = session.try_begin().unwrap();
        import_account(&Network::Main, &mut run.held(&mut dst), &data).await?;
        let imported = newest_account(&mut dst).await?;
        let rows = dkg_rows(&mut dst, imported).await?;
        // No version in the file, a key package and no round-0 package:
        // a DKG from before message signing, so dkg-0.
        assert_eq!(
            rows.params,
            Some((1, 2, 2, "mailbox words".to_string(), 100, String::new(), 0, None))
        );
        assert_eq!(
            rows.state,
            Some((
                None,
                Some(vec![0x02]),
                Some(vec![0x03]),
                Some(vec![0x04]),
                None
            ))
        );
        assert_eq!(rows.peers, vec![(1, 2, vec![0x11]), (3, 2, vec![0x33])]);
        assert_eq!(
            rows.addresses,
            vec![
                (1, "uregtest1aaa".to_string()),
                (2, "uregtest1bbb".to_string())
            ]
        );
        Ok(())
    }

    /// A V9 file written before the version column, for a group that did
    /// exchange round-0 keys: dkg-1, the column's default.
    #[tokio::test]
    async fn unversioned_export_with_round0_keys_imports_as_dkg1() -> Result<()> {
        let io_account = IOAccount {
            version: DB_VERSION,
            name: "frost-v9".to_string(),
            dkg_params: Some(DKGParams {
                id: 1,
                n: 2,
                t: 2,
                seed: String::new(),
                birth: 100,
                name: String::new(),
                dkg_version: None,
                sk: None,
            }),
            dkg_state: Some(DKGState {
                key_pkg: Some(HexBytes(vec![0x04])),
                ..Default::default()
            }),
            dkg_peers: vec![DKGPeer {
                round: 0,
                from_id: 2,
                data: HexBytes(vec![0u8; 32]),
            }],
            ..Default::default()
        };
        let json = serde_json::to_string(&io_account)?;
        let mut encoder = zstd::Encoder::new(Vec::new(), DEFAULT_COMPRESSION_LEVEL)?;
        encoder.write_all(json.as_bytes())?;
        let data = encoder.finish()?;

        let mut dst = fresh_db().await?;
        let session = crate::sync::session::SyncSession::new();
        let run = session.try_begin().unwrap();
        import_account(&Network::Main, &mut run.held(&mut dst), &data).await?;
        let imported = newest_account(&mut dst).await?;
        let rows = dkg_rows(&mut dst, imported).await?;
        assert_eq!(rows.params.map(|p| (p.6, p.7)), Some((1, None)));
        Ok(())
    }
}
