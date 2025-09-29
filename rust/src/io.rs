use std::{borrow::Cow, collections::HashMap};

use age::{scrypt::Identity, Decryptor, Encryptor};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as};
use sqlx::{
    encode::IsNull,
    error::BoxDynError,
    sqlite::{SqliteArgumentValue, SqliteRow, SqliteValueRef},
    Connection, Decode, Encode, Row, Sqlite, SqliteConnection, Type,
};
use std::io::prelude::*;
use tracing::info;
use zstd::DEFAULT_COMPRESSION_LEVEL;

use crate::db::DB_VERSION;

pub async fn export_account(connection: &mut SqliteConnection, account: u32) -> Result<Vec<u8>> {
    info!("Exporting account {}", account);
    let mut io_account = sqlx::query(
        "SELECT id_account, a.name, seed, passphrase, seed_fingerprint, aindex, dindex, def_dindex, icon, birth, position, use_internal, hidden, saved, enabled, internal,
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
            let folder_name: Option<String> = row.get(16);

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
                folder: folder_name.unwrap_or_default(),
                ..Default::default()
            }
            // Process the data as needed
        })
        .fetch_one(&mut *connection)
        .await?;

    info!("Exporting transparent account");
    if let Some(t_account) =
        sqlx::query("SELECT xsk, xvk FROM transparent_accounts WHERE account = ?")
            .bind(account)
            .map(|row: SqliteRow| {
                let xsk: Option<Vec<u8>> = row.get(0);
                let xvk: Vec<u8> = row.get(1);

                IOKeys {
                    xsk: xsk.map(HexBytes),
                    xvk: xvk.into(),
                }
            })
            .fetch_optional(&mut *connection)
            .await?
    {
        io_account.tkeys = Some(t_account);
    }

    info!("Exporting sapling account");
    if let Some(s_account) = sqlx::query("SELECT xsk, xvk FROM sapling_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let xsk: Option<Vec<u8>> = row.get(0);
            let xvk: Vec<u8> = row.get(1);

            IOKeys {
                xsk: xsk.map(HexBytes),
                xvk: xvk.into(),
            }
        })
        .fetch_optional(&mut *connection)
        .await?
    {
        io_account.skeys = Some(s_account);
    }

    info!("Exporting orchard account");
    if let Some(o_account) = sqlx::query("SELECT xsk, xvk FROM orchard_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let xsk: Option<Vec<u8>> = row.get(0);
            let xvk: Vec<u8> = row.get(1);

            IOKeys {
                xsk: xsk.map(HexBytes),
                xvk: xvk.into(),
            }
        })
        .fetch_optional(&mut *connection)
        .await?
    {
        io_account.okeys = Some(o_account);
    }

    info!("Exporting transparent addresses");
    let t_addresses = sqlx::query("SELECT id_taddress, scope, dindex, sk, pk, address FROM transparent_address_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let id_taddress: u32 = row.get(0);
            let scope: u32 = row.get(1);
            let dindex: u32 = row.get(2);
            let sk: Option<Vec<u8>> = row.get(3);
            let pk: Vec<u8> = row.get(4);
            let address: String = row.get(5);

            TAddress {
                id_taddress, scope, dindex, sk: sk.map(HexBytes),
                pk: pk.into(), address
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
        let mut block = sqlx::query("SELECT hash, time FROM headers WHERE height = ?")
            .bind(height)
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
        taddress, position, diversifier, rcm, rho, locked, vout, memo_text, memo_bytes
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
            let memo_text: Option<String> = row.get(15);
            let memo_bytes: Option<Vec<u8>> = row.get(16);

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

    let dkg_params =
        sqlx::query("SELECT id, n, t, seed, birth_height FROM dkg_params WHERE account = ?")
            .bind(account)
            .map(|row: SqliteRow| {
                let id: u8 = row.get(0);
                let n: u8 = row.get(1);
                let t: u8 = row.get(2);
                let seed: String = row.get(3);
                let birth: u32 = row.get(4);

                DKGParams {
                    id,
                    n,
                    t,
                    seed,
                    birth,
                }
            })
            .fetch_optional(&mut *connection)
            .await?;
    io_account.dkg_params = dkg_params;

    let dkg_packages = sqlx::query(
        "SELECT account, public, round, from_id, data FROM dkg_packages WHERE account = ?",
    )
    .bind(account)
    .map(|row: SqliteRow| {
        let account: u32 = row.get(0);
        let public: bool = row.get(1);
        let round: u32 = row.get(2);
        let from_id: u16 = row.get(3);
        let data: Vec<u8> = row.get(4);

        DKGPackage {
            account,
            public,
            round,
            from_id,
            data: data.into(),
        }
    })
    .fetch_all(&mut *connection)
    .await?;
    io_account.dkg_packages = dkg_packages;
    io_account.categories = categories;

    let io_account = serde_json::to_string(&io_account)?;
    let mut encoder = zstd::Encoder::new(Vec::new(), DEFAULT_COMPRESSION_LEVEL)?;
    encoder.write_all(io_account.as_bytes())?;
    let data = encoder.finish()?;

    info!("Exported account size: {}", data.len());
    Ok(data)
}

pub async fn import_account(connection: &mut SqliteConnection, data: &[u8]) -> Result<()> {
    let mut decoder = zstd::Decoder::new(data)?;
    let mut data = String::new();
    decoder.read_to_string(&mut data)?;
    let io_account = serde_json::from_str::<IOAccount>(&data)?;
    if io_account.version != DB_VERSION {
        anyhow::bail!("This version only supports database version {DB_VERSION}");
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
        (name, seed, passphrase, seed_fingerprint, aindex, dindex, def_dindex, icon, birth, folder, position, use_internal, hidden, saved, enabled, internal)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?, ?, ?, ?)")
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
        .execute(&mut *tx)
        .await?;
    let new_id_account = r.last_insert_rowid() as u32;
    // account must be replaced by new_id_account

    info!("Importing transparent key");
    if let Some(tkeys) = io_account.tkeys.as_ref() {
        sqlx::query(
            "INSERT INTO transparent_accounts
            (account, xsk, xvk) VALUES (?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(&tkeys.xsk)
        .bind(&tkeys.xvk)
        .execute(&mut *tx)
        .await?;
    }
    info!("Importing sapling key");
    if let Some(skeys) = io_account.skeys.as_ref() {
        sqlx::query(
            "INSERT INTO sapling_accounts
            (account, xsk, xvk) VALUES (?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(&skeys.xsk)
        .bind(&skeys.xvk)
        .execute(&mut *tx)
        .await?;
    }
    info!("Importing orchard key");
    if let Some(okeys) = io_account.okeys.as_ref() {
        sqlx::query(
            "INSERT INTO orchard_accounts
            (account, xsk, xvk) VALUES (?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(&okeys.xsk)
        .bind(&okeys.xvk)
        .execute(&mut *tx)
        .await?;
    }
    info!("Importing transparent addresses");
    let mut new_taddresses = HashMap::<u32, u32>::new();
    for taddr in io_account.taddrs.iter() {
        let r = sqlx::query(
            "INSERT INTO transparent_address_accounts
            (account, scope, dindex, sk, pk, address) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(taddr.scope)
        .bind(taddr.dindex)
        .bind(&taddr.sk)
        .bind(&taddr.pk)
        .bind(&taddr.address)
        .execute(&mut *tx)
        .await?;
        let new_id_taddress = r.last_insert_rowid() as u32;
        new_taddresses.insert(taddr.id_taddress, new_id_taddress);
    }

    info!("Importing sync heights");
    for sync_height in io_account.sync_heights.iter() {
        sqlx::query(
            "INSERT INTO sync_heights
            (account, pool, height) VALUES (?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(sync_height.pool)
        .bind(sync_height.height)
        .execute(&mut *tx)
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
        let r = sqlx::query(
            "INSERT INTO transactions
            (account, txid, height, time, details, tpe, value, fee, price, category) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(&transaction.txid)
        .bind(transaction.height)
        .bind(transaction.time)
        .bind(transaction.details)
        .bind(transaction.tpe)
        .bind(transaction.value)
        .bind(transaction.fee as i64)
        .bind(transaction.price)
        .bind(category)
        .execute(&mut *tx)
        .await?;
        let new_id_tx = r.last_insert_rowid() as u32;
        new_txs.insert(transaction.id_tx, new_id_tx);

        for note in transaction.notes.iter() {
            let new_taddress = note
                .taddress
                .and_then(|id_taddress| new_taddresses.get(&id_taddress));
            let r = sqlx::query("INSERT INTO notes
                (tx, height, account, pool, scope, nullifier, value, cmx, taddress, position, diversifier,
                rcm, rho, locked)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(new_id_tx)
                .bind(note.height)
                .bind(new_id_account)
                .bind(note.pool)
                .bind(note.scope)
                .bind(&note.nullifier)
                .bind(note.value as i64)
                .bind(&note.cmx)
                .bind(new_taddress)
                .bind(note.position)
                .bind(&note.diversifier)
                .bind(&note.rcm)
                .bind(&note.rho)
                .bind(note.locked)
                .execute(&mut *tx)
                .await?;
            let new_id_note = r.last_insert_rowid() as u32;
            new_notes.insert(note.id_note, new_id_note);

            if let Some(vout) = note.vout {
                sqlx::query(
                    "INSERT INTO memos
                    (account, height, tx, pool, vout, note, memo_text, memo_bytes)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(new_id_account)
                .bind(note.height)
                .bind(new_id_tx)
                .bind(note.pool)
                .bind(vout)
                .bind(new_id_note)
                .bind(&note.memo_text)
                .bind(&note.memo_bytes)
                .execute(&mut *tx)
                .await?;
            }
        }

        for output in transaction.outputs.iter() {
            let r = sqlx::query(
                "INSERT INTO outputs
                (tx, height, account, pool, value, address, vout) VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(new_id_tx)
            .bind(output.height)
            .bind(new_id_account)
            .bind(output.pool)
            .bind(output.value as i64)
            .bind(&output.address)
            .bind(output.vout)
            .execute(&mut *tx)
            .await?;
            let new_id_output = r.last_insert_rowid() as u32;

            if let Some(memo_bytes) = &output.memo_bytes {
                sqlx::query(
                    "INSERT INTO memos
                    (account, height, tx, pool, vout, output, memo_text, memo_bytes)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(new_id_account)
                .bind(output.height)
                .bind(new_id_tx)
                .bind(output.pool)
                .bind(output.vout)
                .bind(new_id_output) // Assuming output is linked to the note
                .bind(&output.memo_text)
                .bind(memo_bytes)
                .execute(&mut *tx)
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
                .expect("new_id_note not found");
            sqlx::query(
                "INSERT INTO spends
                (id_note, tx, height, account, pool, value) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(new_id_note)
            .bind(new_id_tx)
            .bind(spend.height)
            .bind(new_id_account)
            .bind(spend.pool)
            .bind(spend.value as i64)
            .execute(&mut *tx)
            .await?;
        }
    }

    info!("Importing checkpoints");
    for block in io_account.blocks.iter() {
        sqlx::query(
            "INSERT INTO headers
            (height, hash, time) VALUES (?, ?, ?) ON CONFLICT DO NOTHING",
        )
        .bind(block.height)
        .bind(&block.hash)
        .bind(block.time)
        .execute(&mut *tx)
        .await?;

        for witness in block.witness.iter() {
            let new_id_note = new_notes.get(&witness.note).unwrap();
            sqlx::query(
                "INSERT INTO witnesses
                (account, height, note, witness) VALUES (?, ?, ?, ?)",
            )
            .bind(new_id_account)
            .bind(witness.height)
            .bind(new_id_note)
            .bind(&witness.witness)
            .execute(&mut *tx)
            .await?;
        }
    }

    if let Some(dkg_params) = io_account.dkg_params {
        sqlx::query(
            "INSERT INTO dkg_params
            (account, id, n, t, seed, birth_height) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(dkg_params.id)
        .bind(dkg_params.n)
        .bind(dkg_params.t)
        .bind(&dkg_params.seed)
        .bind(dkg_params.birth)
        .execute(&mut *tx)
        .await?;
    }

    for dkg_package in io_account.dkg_packages.iter() {
        sqlx::query(
            "INSERT INTO dkg_packages
            (account, public, round, from_id, data) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(new_id_account)
        .bind(dkg_package.public)
        .bind(dkg_package.round)
        .bind(dkg_package.from_id)
        .bind(&dkg_package.data)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

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
    pub tkeys: Option<IOKeys>,
    pub skeys: Option<IOKeys>,
    pub okeys: Option<IOKeys>,
    pub taddrs: Vec<TAddress>,
    pub sync_heights: Vec<SyncHeight>,
    pub blocks: Vec<IOBlock>,
    pub transactions: Vec<IOTransaction>,
    pub dkg_params: Option<DKGParams>,
    pub dkg_packages: Vec<DKGPackage>,
    pub categories: Vec<IOCategory>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct IOKeys {
    pub xsk: Option<HexBytes>,
    pub xvk: HexBytes,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct TAddress {
    pub id_taddress: u32,
    pub scope: u32,
    pub dindex: u32,
    pub sk: Option<HexBytes>,
    pub pk: HexBytes,
    pub address: String,
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
    fn encode(self, args: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Blob(Cow::Owned(self.0)));

        Ok(IsNull::No)
    }

    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Blob(Cow::Owned(self.0.clone())));

        Ok(IsNull::No)
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
