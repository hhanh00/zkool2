use anyhow::Result;
use bincode::{config::legacy, Decode, Encode};
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use tracing::info;

pub async fn export_account(connection: &SqlitePool, account: u32) -> Result<Vec<u8>> {
    let mut io_account = sqlx::query(
        "SELECT id_account, name, seed, seed_fingerprint, aindex, dindex, def_dindex, icon, birth, position, hidden, saved, enabled
        FROM accounts WHERE id_account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let id_account: u32 = row.get(0);
            let name: String = row.get(1);
            let seed: Option<Vec<u8>> = row.get(2);
            let seed_fingerprint: Option<Vec<u8>> = row.get(3);
            let aindex: u32 = row.get(4);
            let dindex: u32 = row.get(5);
            let def_dindex: u32 = row.get(6);
            let icon: Option<Vec<u8>> = row.get(7);
            let birth: u32 = row.get(8);
            let position: u32 = row.get(9);
            let hidden: bool = row.get(10);
            let saved: bool = row.get(11);
            let enabled: bool = row.get(12);

            IOAccount {
                id_account,
                name,
                seed,
                seed_fingerprint,
                aindex,
                dindex,
                def_dindex,
                icon,
                birth,
                position,
                hidden,
                saved,
                enabled,
                ..Default::default()
            }
            // Process the data as needed
        })
        .fetch_one(connection)
        .await?;

    if let Some(t_account) = sqlx::query("SELECT xsk, xvk FROM transparent_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let xsk: Option<Vec<u8>> = row.get(0);
            let xvk: Vec<u8> = row.get(1);

            IOKeys { xsk, xvk }
        })
        .fetch_optional(connection)
        .await? {
        io_account.tkeys = Some(t_account);
    }

    if let Some(s_account) = sqlx::query("SELECT xsk, xvk FROM sapling_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let xsk: Option<Vec<u8>> = row.get(0);
            let xvk: Vec<u8> = row.get(1);

            IOKeys { xsk, xvk }
        })
        .fetch_optional(connection)
        .await? {
        io_account.skeys = Some(s_account);
    }

    if let Some(o_account) = sqlx::query("SELECT xsk, xvk FROM orchard_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let xsk: Option<Vec<u8>> = row.get(0);
            let xvk: Vec<u8> = row.get(1);

            IOKeys { xsk, xvk }
        })
        .fetch_optional(connection)
        .await? {
        io_account.okeys = Some(o_account);
    }

    let t_addresses = sqlx::query("SELECT id_taddress, scope, dindex, sk, address FROM transparent_address_accounts WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let id_taddress: u32 = row.get(0);
            let scope: u32 = row.get(1);
            let dindex: u32 = row.get(2);
            let sk: Option<Vec<u8>> = row.get(3);
            let address: String = row.get(4);

            TAddress {id_taddress, scope, dindex, sk, address}

        })
        .fetch_all(connection)
        .await?;
    io_account.taddrs = t_addresses;

    // Get the sync heights
    let sync_heights = sqlx::query("SELECT pool, height FROM sync_heights WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let pool: u8 = row.get(0);
            let height: u32 = row.get(1);
            SyncHeight { pool, height }
        })
        .fetch_all(connection)
        .await?;
    io_account.sync_heights = sync_heights;

    // Get checkpoint heights
    let checkpoints = sqlx::query("SELECT DISTINCT height FROM witnesses WHERE account = ? ORDER BY height")
        .bind(account)
        .map(|row: SqliteRow| {
            let height: u32 = row.get(0);
            height
        })
        .fetch_all(connection)
        .await?;

    let mut blocks = vec![];
    for height in checkpoints.iter() {
        // Get headers for given height
        let mut block = sqlx::query("SELECT hash, time FROM headers WHERE height = ?")
            .bind(height)
            .map(|row: SqliteRow| {
                let hash: Vec<u8> = row.get(0);
                let time: u32 = row.get(1);

                IOBlock {height: *height, hash, time, ..Default::default()}
            })
            .fetch_one(connection)
            .await?;

        // Get witness for given height
        let witness = sqlx::query("SELECT note, witness FROM witnesses WHERE account = ? AND height = ?")
            .bind(account)
            .bind(height)
            .map(|row: SqliteRow| {
                let note: u32 = row.get(0);
                let witness: Vec<u8> = row.get(1);
                IOWitness { note, witness }
            })
            .fetch_all(connection)
            .await?;
        block.witness = witness;

        blocks.push(block);
    }

    // Get the transactions for the given account
    let mut transactions = sqlx::query("SELECT id_tx, txid, height, time, details FROM transactions WHERE account = ?")
        .bind(account)
        .map(|row: SqliteRow| {
            let id_tx: u32 = row.get(0);
            let txid: Vec<u8> = row.get(1);
            let height: u32 = row.get(2);
            let time: u32 = row.get(3);
            let details: bool = row.get(4);

            IOTransaction { id_tx, txid, height, time, details, ..Default::default() }
        })
        .fetch_all(connection)
        .await?;

    for tx in transactions.iter_mut() {
        // Get the notes and memos for the given transaction
        let notes = sqlx::query("SELECT id_note, n.height, n.account, n.pool, nullifier, value, cmx,
        taddress, position, diversifier, rcm, rho, locked, vout, memo_text, memo_bytes
        FROM notes n LEFT JOIN memos m ON n.id_note = m.note WHERE n.tx = ?")
            .bind(tx.id_tx)
            .map(|row: SqliteRow| {
                let id_note: u32 = row.get(0);
                let height: u32 = row.get(1);
                let account: u32 = row.get(2);
                let pool: u8 = row.get(3);
                let nullifier: Vec<u8> = row.get(4);
                let value: u64 = row.get::<i64, _>(5) as u64;
                let cmx: Option<Vec<u8>> = row.get(6);
                let taddress: Option<u32> = row.get(7);
                let position: Option<u32> = row.get(8);
                let diversifier: Option<Vec<u8>> = row.get(9);
                let rcm: Option<Vec<u8>> = row.get(10);
                let rho: Option<Vec<u8>> = row.get(11);
                let locked: bool = row.get(12);
                let vout: Option<u32> = row.get(13);
                let memo_text: Option<String> = row.get(14);
                let memo_bytes: Option<Vec<u8>> = row.get(15);

                IONote { id_note, height, account, pool, nullifier, value, cmx, taddress, position, diversifier, rcm, rho, locked,
                    vout, memo_text, memo_bytes }
            })
            .fetch_all(connection)
            .await?;
        tx.notes = notes;

        // Get the spends for the given transaction
        let spends = sqlx::query("SELECT id_note, height, account, pool, value FROM spends WHERE tx = ?")
            .bind(tx.id_tx)
            .map(|row: SqliteRow| {
                let id_note: u32 = row.get(0);
                let height: u32 = row.get(1);
                let account: u32 = row.get(2);
                let pool: u8 = row.get(3);
                let value: u64 = row.get::<i64, _>(4) as u64;

                IOSpend { id_note, height, account, pool, value }
            })
            .fetch_all(connection)
            .await?;
        tx.spends = spends;
    }

    let io_account = bincode::encode_to_vec(&io_account, legacy())?;
    info!("Exported account size: {}", io_account.len());
    Ok(io_account)
}

#[derive(Clone, Encode, Decode, Default, Debug)]
pub struct IOAccount {
    pub id_account: u32,
    pub name: String,
    pub seed: Option<Vec<u8>>,
    pub seed_fingerprint: Option<Vec<u8>>,
    pub aindex: u32,
    pub dindex: u32,
    pub def_dindex: u32,
    pub icon: Option<Vec<u8>>,
    pub birth: u32,
    pub position: u32,
    pub hidden: bool,
    pub saved: bool,
    pub enabled: bool,
    pub tkeys: Option<IOKeys>,
    pub skeys: Option<IOKeys>,
    pub okeys: Option<IOKeys>,
    pub taddrs: Vec<TAddress>,
    pub sync_heights: Vec<SyncHeight>,
}

#[derive(Clone, Encode, Decode, Default, Debug)]
pub struct IOKeys {
    pub xsk: Option<Vec<u8>>,
    pub xvk: Vec<u8>,
}

#[derive(Clone, Encode, Decode, Default, Debug)]
pub struct TAddress {
    pub id_taddress: u32,
    pub scope: u32,
    pub dindex: u32,
    pub sk: Option<Vec<u8>>,
    pub address: String,
}

#[derive(Clone, Encode, Decode, Default, Debug)]
pub struct SyncHeight {
    pub pool: u8,
    pub height: u32,
}

#[derive(Clone, Encode, Decode, Default, Debug)]
pub struct IOBlock {
    pub height: u32,
    pub hash: Vec<u8>,
    pub time: u32,
    pub witness: Vec<IOWitness>,
}

#[derive(Clone, Encode, Decode, Default, Debug)]
pub struct IOWitness {
    pub note: u32,
    pub witness: Vec<u8>,
}

#[derive(Clone, Encode, Decode, Default, Debug)]
pub struct IOTransaction {
    pub id_tx: u32,
    pub txid: Vec<u8>,
    pub height: u32,
    pub time: u32,
    pub details: bool,
    pub notes: Vec<IONote>,
    pub spends: Vec<IOSpend>,
}

#[derive(Clone, Encode, Decode, Default, Debug)]
pub struct IONote {
    pub id_note: u32,
    pub height: u32,
    pub account: u32,
    pub pool: u8,
    pub nullifier: Vec<u8>,
    pub value: u64,
    pub cmx: Option<Vec<u8>>,
    pub taddress: Option<u32>,
    pub position: Option<u32>,
    pub diversifier: Option<Vec<u8>>,
    pub rcm: Option<Vec<u8>>,
    pub rho: Option<Vec<u8>>,
    pub locked: bool,
    pub vout: Option<u32>,
    pub memo_text: Option<String>,
    pub memo_bytes: Option<Vec<u8>>,
}

#[derive(Clone, Encode, Decode, Default, Debug)]
pub struct IOSpend {
    pub id_note: u32,
    pub height: u32,
    pub account: u32,
    pub pool: u8,
    pub value: u64,
}
