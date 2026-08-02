// rust/src/rpc.rs
//
// A monero-wallet-rpc-compatible HTTP/JSON API for zkool, mirroring the
// endpoint shapes exposed by zcash-walletd. Runs as an ADDITIONAL warp
// server, on its own port, alongside zkool's GraphQL server. Calls the
// same underlying Rust functions directly (no GraphQL proxying).

use std::{collections::HashMap, sync::LazyLock};
use std::convert::Infallible;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use tokio_util::sync::CancellationToken;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};
use tokio::sync::{Mutex};
use bigdecimal::BigDecimal;
use zcash_keys::address::UnifiedAddress;
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_keys::encoding::AddressCodec;
use zcash_trees::network::Network;
// use zcash_protocol::consensus::Network;

use crate::account::{generate_next_dindex, get_addresses};
use crate::api::account::{new_account as create_new_account, NewAccount};
use crate::api::coin::Coin;
use crate::api::mempool::MempoolMsg;
use crate::api::network::get_current_height;
use crate::db::{get_sync_height, list_accounts};
use crate::graphql::query::zats_to_zec;
use crate::mempool::run_mempool_impl;
use crate::keys::{SaplingDiversifiedAddress, ScopeExt};

// use bytes::Bytes;
// use bytes::Bytes;
use warp::hyper::body::Bytes;
use warp::http::Method;
use warp::path::FullPath;
// use warp::{Filter, Rejection};

#[derive(Debug)]
struct JsonError(serde_json::Error);
impl warp::reject::Reject for JsonError {}



// ---------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------

#[derive(Clone)]
pub struct RpcState {
    pub coin: Coin,
    pub notify_tx_url: Option<String>,
    pub notify_block_url: Option<String>,
    pub address_creation_lock: Arc<Mutex<()>>,
    pub http: reqwest::Client,
}

// ---------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------

#[derive(Debug)]
struct RpcError(anyhow::Error);

impl warp::reject::Reject for RpcError {}

fn reject(e: anyhow::Error) -> Rejection {
    warp::reject::custom(RpcError(e))
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = if let Some(RpcError(e)) = err.find() {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    } else if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not found".to_string())
    } else {
        (StatusCode::BAD_REQUEST, "Bad request".to_string())
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&ErrorBody { error: message }),
        code,
    ))
}

/// Create the RPC-only bookkeeping table. Called once at server startup.
/// Lives entirely outside zkool's core schema/migrations.
pub async fn ensure_schema(coin: &Coin) -> anyhow::Result<()> {
    let mut conn = coin.get_connection().await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS rpc_subaddresses (
            account INTEGER NOT NULL,
            address_index INTEGER NOT NULL,   -- sequential, monero-rpc facing
            diversifier_index INTEGER NOT NULL,
            label TEXT,
            ua TEXT,
            transparent TEXT,
            sapling TEXT,
            orchard TEXT,
            created_at INTEGER NOT NULL,
            PRIMARY KEY (account, address_index)
        )",
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_rpc_subaddresses_ua
        ON rpc_subaddresses(ua)",
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Insert one row per pool address for a given account/diversifier index.
/// `addresses` pools follow zkool's convention: 0=transparent, 1=sapling,
/// 2=orchard, 3=ironwood.
async fn record_subaddress(
    coin: &Coin,
    account: u32,
    address_index: u32,
    diversifier_index: u32,
    label: &Option<String>,
    addresses: &crate::api::account::Addresses,
) -> anyhow::Result<()> {
    let mut conn = coin.get_connection().await?;
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO rpc_subaddresses
         (account, address_index, diversifier_index, label, ua, transparent, sapling, orchard, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )
    .bind(account as i64)
    .bind(address_index as i64)
    .bind(diversifier_index as i64)
    .bind(label)
    .bind(&addresses.ua)
    .bind(&addresses.taddr)
    .bind(&addresses.saddr)
    .bind(&addresses.oaddr)
    .bind(now)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

#[derive(Deserialize)]
struct CreateAccountRequest {
    label: Option<String>,
    key: String,
    #[serde(default)]
    aindex: i32,
    passphrase: Option<String>,
    birth: Option<i32>,
    pools: Option<i32>,
    #[serde(default)]
    use_internal: bool,
}

#[derive(Serialize)]
struct CreateAccountResponse {
    account_index: u32,
}

async fn create_account(
    req: CreateAccountRequest,
    state: RpcState,
) -> Result<impl Reply, Rejection> {
    let inner = async {
        let height = get_current_height(&state.coin).await?;
        let na = NewAccount {
            name: req.label.clone().unwrap_or_default(),
            restore: false,
            key: req.key,
            passphrase: req.passphrase,
            fingerprint: None,
            icon: None,
            aindex: req.aindex as u32,
            birth: req.birth.map(|v| v as u32).or(Some(height)),
            pools: req.pools.map(|v| v as u8),
            use_internal: req.use_internal,
            folder: String::new(),
            internal: false,
            ledger: false,
        };
        let id_account = create_new_account(&na, &state.coin).await?;

        Ok::<_, anyhow::Error>(CreateAccountResponse {
            account_index: id_account,
        })
    };
    let resp = inner.await.map_err(reject)?;
    Ok(warp::reply::json(&resp))
}

// ---------------------------------------------------------------------
// create_address
// ---------------------------------------------------------------------

#[derive(Deserialize)]
struct CreateAddressRequest {
    account_index: u32,
    label: Option<String>,
}

#[derive(Serialize)]
struct CreateAddressResponse {
    address: String,
    address_index: u32,
}

async fn create_address(
    req: CreateAddressRequest,
    state: RpcState,
) -> Result<impl Reply, Rejection> {
    let inner = async {
        let network = state.coin.network();

        // Serialize address creation per-account to avoid racing on
        // address_index / diversifier_index.
        let _guard = state.address_creation_lock.lock().await;

        let mut conn = state.coin.get_connection().await?;

        let dindex = generate_next_dindex(&network, &mut conn, req.account_index).await?;
        // get_addresses reads accounts.dindex, which generate_next_dindex
        // just updated (and committed) above, so this derives the address
        // at the same dindex we just reserved. Both calls are serialized
        // by address_creation_lock, so no other caller can bump dindex
        // between the two.
        let addresses = get_addresses(&network, &mut conn,req.account_index, 6).await?;
        debug_assert_eq!(
            addresses.diversifier_index, dindex as u32,
            "get_addresses returned a different dindex than generate_next_dindex reserved"
        );

        let address_index = next_address_index(&state.coin, req.account_index).await?;

        record_subaddress(
            &state.coin,
            req.account_index,
            address_index,
            dindex,
            &req.label,
            &addresses,
        )
        .await?;

        Ok::<_, anyhow::Error>(CreateAddressResponse {
            address: addresses.ua.unwrap_or_default(),
            address_index,
        })
    };
    let resp = inner.await.map_err(reject)?;
    Ok(warp::reply::json(&resp))
}



async fn next_address_index(coin: &Coin, account: u32) -> anyhow::Result<u32> {
    let mut conn = coin.get_connection().await?;
    let (max,): (Option<i64>,) = sqlx::query_as(
        "SELECT MAX(address_index) FROM rpc_subaddresses WHERE account = ?1",
    )
    .bind(account as i64)
    .fetch_one(&mut *conn)
    .await?;
    Ok(max.map(|v| v as u32 + 1).unwrap_or(0))
}

// ---------------------------------------------------------------------
// get_accounts
// ---------------------------------------------------------------------

#[derive(Deserialize)]
struct GetAccountsRequest {
    #[allow(dead_code)]
    tag: Option<String>,
}

#[derive(Serialize)]
struct SubaddressAccount {
    account_index: u32,
    balance: u64,
    unlocked_balance: u64,
    base_address: String,
    label: String,
}

#[derive(Serialize)]
struct GetAccountsResponse {
    subaddress_accounts: Vec<SubaddressAccount>,
    total_balance: u64,
    total_unlocked_balance: u64,
}

async fn get_accounts(
    _req: GetAccountsRequest,
    state: RpcState,
) -> Result<impl Reply, Rejection> {
    let inner = async {
        let network = state.coin.network();
        let mut conn = state.coin.get_connection().await?;
        let accounts = list_accounts(&mut conn, state.coin.coin).await?;

        let mut subaddress_accounts = Vec::with_capacity(accounts.len());
        let mut total_balance = 0u64;
        let mut total_unlocked = 0u64;

        for a in accounts {
            let balance = a.balance;
            let unlocked = balance;

            // Prefer the RPC bookkeeping table, but fall back to deriving
            // the base (index-0) UA straight from the account's UFVK if the
            // table is missing the row (e.g. it was wiped, or this account
            // predates the RPC server). This keeps `base_address` from
            // silently going blank.
            let base_address: Option<String> = sqlx::query_scalar(
                "SELECT ua FROM rpc_subaddresses
                 WHERE account = ? AND address_index = 0",
            )
            .bind(a.id as u32)
            .fetch_optional(&mut *conn)
            .await?
            .flatten();

            let base_address = match base_address {
                Some(addr) if !addr.is_empty() => addr,
                _ => derive_base_address(&network, &mut conn, a.id as u32)
                    .await
                    .unwrap_or_default(),
            };

            total_balance += balance;
            total_unlocked += unlocked;

            subaddress_accounts.push(SubaddressAccount {
                account_index: a.id as u32,
                balance,
                unlocked_balance: unlocked,
                base_address,
                label: a.name,
            });
        }

        Ok::<_, anyhow::Error>(GetAccountsResponse {
            subaddress_accounts,
            total_balance,
            total_unlocked_balance: total_unlocked,
        })
    };
    let resp = inner.await.map_err(reject)?;
    Ok(warp::reply::json(&resp))
}

/// Derive the account's base (diversifier index 0) unified address directly
/// from its UFVK, without touching `rpc_subaddresses`. Used as a fallback
/// when the bookkeeping table doesn't have a row yet.
async fn derive_base_address(
    network: &Network,
    conn: &mut SqliteConnection,
    account: u32,
) -> anyhow::Result<String> {
    let addresses = get_addresses(network, conn, account, 6).await?;
    Ok(addresses.ua.unwrap_or_default())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transfer {
    pub txid: String,
    pub payment_id: Option<String>,
    pub height: u32,          // 0 if unconfirmed
    pub confirmations: u32,   // 0 if unconfirmed
    pub timestamp: i64,
    pub amount: u64,
    pub fee: u64,
    pub note: String,
    pub r#type: String,       // "pending" | "in"
    pub subaddr_index: SubaddrIndex,
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubaddrIndex {
    pub major: u32,
    pub minor: u32,
}

// ---------------------------------------------------------------------
// Unified confirmed-transfers implementation
//
// Both get_transfers and get_transfer_by_txid funnel through this single
// function, so address resolution, subaddress mapping, fee handling,
// confirmation counting, and transfer construction all stay consistent.
// ---------------------------------------------------------------------

/// Confirmed transfers for one account.
///
/// * `subaddr_indices` — restrict to these address_index values; empty
///   means "all".
/// * `txid_filter` — if `Some`, restrict to that single txid (raw bytes,
///   internal byte order, i.e. NOT display/reversed order).
/// * `confirmations` — minimum confirmations required to include a tx.
///   Pass `1` to include everything already in a block.
pub async fn get_confirmed_transfers_impl(
    conn: &mut SqliteConnection,
    network: &Network,
    latest_height: u32,
    account: u32,
    subaddr_indices: &[u32],
    txid_filter: Option<&[u8]>,
    confirmations: u32,
) -> anyhow::Result<Vec<Transfer>> {
    let confirmations = confirmations.max(1);
    let max_height = latest_height.saturating_sub(confirmations - 1);

    let ufvk_str = crate::key::get_account_ufvk(network, conn, account, 7).await?;
    let ufvk =
        UnifiedFullViewingKey::decode(network, &ufvk_str).map_err(anyhow::Error::msg)?;

    let notes = if let Some(txid) = txid_filter {
        crate::db::get_notes_txid(conn, account, txid).await?
    } else {
        crate::db::get_notes(conn, account).await?
    };
    if notes.is_empty() {
        return Ok(vec![]);
    }

    let tx_by_id: HashMap<u32, (Vec<u8>, u32, i64, u64)> = if let Some(txid) = txid_filter {
        // Point lookup: at most one transaction row for this txid/account.
        let row: Option<(u32, Vec<u8>, u32, i64, u64)> = sqlx::query_as(
            "SELECT id_tx, txid, height, time, fee \
             FROM transactions WHERE account = ? AND txid = ?",
        )
        .bind(account)
        .bind(txid)
        .fetch_optional(&mut *conn)
        .await?;
        row.into_iter()
            .map(|(id_tx, txid, height, time, fee)| (id_tx, (txid, height, time, fee)))
            .collect()
    } else {
        let tx_rows: Vec<(u32, Vec<u8>, u32, i64, u64)> = sqlx::query_as(
            "SELECT id_tx, txid, height, time, fee FROM transactions WHERE account = ?",
        )
        .bind(account)
        .fetch_all(&mut *conn)
        .await?;
        tx_rows
            .into_iter()
            .map(|(id_tx, txid, height, time, fee)| (id_tx, (txid, height, time, fee)))
            .collect()
    };

    // diversifier_index -> address_index, from rpc_subaddresses.
    let addr_map = address_index_map(conn, account).await?;

    let mut grouped: HashMap<(u32, u32), (u64, String)> = HashMap::new(); // (id_tx, address_index) -> (value, address)

    for n in notes {
        let Some(&(_, height, _, _)) = tx_by_id.get(&n.tx) else {
            continue;
        };
        if height > max_height {
            continue;
        }

        let id_tx = n.tx;
        let value = n.value;

        let resolved = resolve_note(network, &ufvk, n)
            .map_err(|e| anyhow::anyhow!("Failed to resolve note: {e}"))?;

        let div_index_i64 = resolved
            .diversifier_index
            .as_ref()
            .and_then(|d| d.to_string().parse::<i64>().ok());

        let address_index = div_index_i64
            .and_then(|d| addr_map.get(&d).copied())
            .unwrap_or(0);

        if !subaddr_indices.is_empty() && !subaddr_indices.contains(&address_index) {
            continue;
        }

        let entry = grouped
            .entry((id_tx, address_index))
            .or_insert((0, resolved.address.clone()));
        entry.0 += value;
    }

    let mut transfers = Vec::with_capacity(grouped.len());
    for ((id_tx, address_index), (amount, address)) in grouped {
        let Some((txid, height, time, fee)) = tx_by_id.get(&id_tx) else {
            continue;
        };
        let mut txid_display = txid.clone();
        txid_display.reverse();

        transfers.push(Transfer {
            txid: hex::encode(&txid_display),
            payment_id: None,
            height: *height,
            confirmations: latest_height.saturating_sub(*height) + 1,
            timestamp: *time,
            amount,
            fee: *fee,
            note: String::new(),
            r#type: "in".to_string(),
            subaddr_index: SubaddrIndex {
                major: account,
                minor: address_index,
            },
            address,
        });
    }

    transfers.sort_by(|a, b| b.height.cmp(&a.height));
    Ok(transfers)
}

/// Convenience wrapper: all confirmed transfers for an account.
pub async fn get_confirmed_transfers(
    conn: &mut SqliteConnection,
    network: &Network,
    latest_height: u32,
    account: u32,
    subaddr_indices: &[u32],
    confirmations: u32,
) -> anyhow::Result<Vec<Transfer>> {
    get_confirmed_transfers_impl(
        conn,
        network,
        latest_height,
        account,
        subaddr_indices,
        None,
        confirmations,
    )
    .await
}

/// Convenience wrapper: confirmed transfers for a single txid (any
/// confirmation depth >= 1).
pub async fn get_confirmed_transfers_by_txid(
    conn: &mut SqliteConnection,
    network: &Network,
    latest_height: u32,
    txid_hex: &str,
    account: u32,
) -> anyhow::Result<Vec<Transfer>> {
    let mut txid = hex::decode(txid_hex)?;
    txid.reverse();

    get_confirmed_transfers_impl(conn, network, latest_height, account, &[], Some(&txid), 1)
        .await
}

#[derive(Deserialize)]
struct GetTransferByTxidRequest {
    txid: String,
    account_index: u32,
}

#[derive(Serialize, Debug)]
struct GetTransferByTxidResponse {
    transfer: Transfer,
    transfers: Vec<Transfer>,
}

async fn get_transfer_by_txid(
    req: GetTransferByTxidRequest,
    state: RpcState,
) -> Result<impl Reply, Rejection> {
    let inner = async {
        let network = state.coin.network();
        let mut conn = state.coin.get_connection().await?;

        // 1. Check mempool first.
        let unconfirmed = get_unconfirmed_for_account(req.account_index).await;
        if let Some((txid, value, notes)) =
            unconfirmed.into_iter().find(|(txid, _, _)| txid == &req.txid)
        {
            let transfer = unconfirmed_to_transfer(
                &mut conn,
                req.account_index,
                &txid,
                &value,
                &notes,
            )
            .await?;
            return Ok::<_, anyhow::Error>(GetTransferByTxidResponse {
                transfer: transfer.clone(),
                transfers: vec![transfer],
            });
        }

        // 2. Fall back to confirmed transfers from the db.
        let latest_height = state.coin.client().await?.latest_height().await?;
        let transfers = get_confirmed_transfers_by_txid(
            &mut conn,
            &network,
            latest_height,
            &req.txid,
            req.account_index,
        )
        .await?;

        if transfers.is_empty() {
            anyhow::bail!("Unknown txid {}", req.txid);
        }

        Ok(GetTransferByTxidResponse {
            transfer: transfers[0].clone(),
            transfers,
        })
    };
    let resp = inner.await.map_err(reject)?;
    Ok(warp::reply::json(&resp))
}

/// Convert a zats/zec `BigDecimal` amount to zats (u64) without going
/// through floating point. Assumes at most 8 decimal places (ZEC).
fn zec_bigdecimal_to_zats(value: &BigDecimal) -> u64 {
    // Scale by 10^8 and truncate to an integer, entirely in decimal
    // arithmetic. This avoids precision loss from f64 round-tripping.
    let scaled = value * BigDecimal::from(100_000_000u64);
    let (digits, _) = scaled.into_bigint_and_scale();
    // `digits` is already scaled to zats (scale should now be <= 0 for
    // exact ZEC amounts); use the integer value directly.
    digits.to_string().parse::<u64>().unwrap_or(0)
}

async fn unconfirmed_to_transfer(
    conn: &mut SqliteConnection,
    account_index: u32,
    txid: &str,
    value: &BigDecimal,
    notes: &[UnconfirmedNote],
) -> anyhow::Result<Transfer> {
    let amount = zec_bigdecimal_to_zats(value);

    // Resolve the subaddress index from the first note's diversifier_index,
    // using the same rpc_subaddresses mapping as confirmed transfers.
    let addr_map = address_index_map(conn, account_index).await?;
    let address_index = notes
        .first()
        .and_then(|n| n.diversifier_index.as_ref())
        .and_then(|d| d.to_string().parse::<i64>().ok())
        .and_then(|d| addr_map.get(&d).copied())
        .unwrap_or(0);

    Ok(Transfer {
        txid: txid.to_string(),
        payment_id: None,
        height: 0,
        confirmations: 0,
        timestamp: chrono::Utc::now().timestamp(),
        amount,
        fee: 0,
        note: String::new(),
        r#type: "pending".to_string(),
        subaddr_index: SubaddrIndex {
            major: account_index,
            minor: address_index,
        },
        address: notes
            .first()
            .and_then(|n| n.address.clone())
            .unwrap_or_default(),
    })
}

// ---------------------------------------------------------------------
// get_transfers
// ---------------------------------------------------------------------

pub async fn get_unconfirmed_for_account(
    account: u32,
) -> Vec<(String, BigDecimal, Vec<UnconfirmedNote>)> {
    let mempool = MEMPOOL.lock().await;
    mempool
        .unconfirmed
        .get(&account)
        .map(|txs| {
            txs.iter()
                .map(|(txid, (value, notes))| (txid.clone(), value.clone(), notes.clone()))
                .collect()
        })
        .unwrap_or_default()
}

/// Remove a txid from the in-memory mempool cache for an account, once it
/// has confirmed. Without this, `get_transfer_by_txid` / `get_transfers`
/// would show the transaction as both pending and confirmed forever.
pub async fn clear_confirmed_from_mempool(account: u32, txid: &str) {
    let mut mempool = MEMPOOL.lock().await;
    if let Some(txs) = mempool.unconfirmed.get_mut(&account) {
        txs.remove(txid);
    }
}

/// Build a diversifier_index -> address_index map for one account,
/// from the RPC-only bookkeeping table.
async fn address_index_map(
    conn: &mut SqliteConnection,
    account: u32,
) -> anyhow::Result<HashMap<i64, u32>> {
    let rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT diversifier_index, address_index FROM rpc_subaddresses WHERE account = ?",
    )
    .bind(account)
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows.into_iter().map(|(d, a)| (d, a as u32)).collect())
}

#[derive(Clone, Debug)]
pub struct Note {
    pub id: i32,
    pub height: i32,
    pub pool: i32,
    pub tx: i32,
    pub value: BigDecimal,
    pub scope: i32,
    pub diversifier: String,
    pub diversifier_index: Option<BigDecimal>,
    pub address: String,
    pub memo: Option<String>,
    pub id_asset: Option<i32>,
    pub asset_base: Option<String>,
}

fn resolve_note(
    network: &Network,
    ufvk: &UnifiedFullViewingKey,
    n: crate::api::account::TxNote,
) -> Result<Note, String> {
    let (address, diversifier_index) = match n.pool {
        1 => {
            let div = n
                .diversifier
                .as_ref()
                .ok_or_else(|| "Sapling note missing diversifier".to_string())?
                .clone();
            let d = sapling_crypto::keys::Diversifier(
                div.clone()
                    .try_into()
                    .map_err(|_| format!("Sapling diversifier wrong length: {} bytes", div.len()))?,
            );
            let sfvk = ufvk
                .sapling()
                .ok_or_else(|| "UFVK missing sapling key".to_string())?;
            let address = sfvk
                .diversified_address_for_scope(n.scope, d)
                .ok_or_else(|| "Sapling diversified address derivation failed".to_string())?;
            let diversifier_index: Option<u64> = sfvk
                .decrypt_diversifier(&address)
                .and_then(|d| d.0.try_into().ok());
            (Some(address.encode(&network)), diversifier_index)
        }
        2 | 3 => {
            let div = n
                .diversifier
                .as_ref()
                .ok_or_else(|| "Orchard/Ironwood note missing diversifier".to_string())?
                .clone();
            let d = orchard::keys::Diversifier::from_bytes(
                div.clone()
                    .try_into()
                    .map_err(|_| format!("Orchard diversifier wrong length: {} bytes", div.len()))?,
            );
            let ofvk = ufvk
                .orchard()
                .ok_or_else(|| "UFVK missing orchard key".to_string())?;
            let scope = n.scope.orchard_scope();
            let ivk = ofvk.to_ivk(scope);
            let address = ofvk.address(d, scope);
            let diversifier_index: Option<u64> =
                ivk.diversifier_index(&address).and_then(|d| d.try_into().ok());
            let ua = UnifiedAddress::from_receivers(Some(address), None, None)
                .ok_or_else(|| "UnifiedAddress::from_receivers returned None".to_string())?;
            (Some(ua.encode(&network)), diversifier_index)
        }
        _ => (None, None),
    };

    let asset_base = if (n.pool == 2 || n.pool == 3) && n.id_asset.is_some() {
        Some("".to_string()) // Placeholder: could look up actual asset_base from assets
    } else {
        None
    };

    Ok(Note {
        id: n.id as i32,
        height: n.height as i32,
        pool: n.pool as i32,
        tx: n.tx as i32,
        scope: n.scope as i32,
        diversifier: n.diversifier.map(|d| hex::encode(&d)).unwrap_or_default(),
        diversifier_index: diversifier_index.map(BigDecimal::from),
        address: address.unwrap_or_default(),
        value: zats_to_zec(n.value as i64),
        memo: n.memo,
        id_asset: n.id_asset.map(|v| v as i32),
        asset_base,
    })
}

#[derive(Deserialize)]
struct GetTransfersRequest {
    account_index: u32,
    r#in: bool,
    subaddr_indices: Vec<u32>,
}

#[derive(Serialize)]
struct GetTransfersResponse {
    r#in: Vec<Transfer>,
}

async fn get_transfers(
    req: GetTransfersRequest,
    state: RpcState,
) -> Result<impl Reply, Rejection> {
    let inner = async {
        anyhow::ensure!(req.r#in, "only incoming transfers are supported");

        let mut conn = state.coin.get_connection().await?;
        let latest_height = state.coin.client().await?.latest_height().await?;
        let network = state.coin.network();

        // Confirmed, from db.
        let mut transfers = get_confirmed_transfers(
            &mut conn,
            &network,
            latest_height,
            req.account_index,
            &req.subaddr_indices,
            1,
        )
        .await?;

        // Unconfirmed, from mempool, prepended (most recent first is
        // typical Monero convention). Filtered by subaddr_indices to match
        // confirmed-transfer behavior.
        let unconfirmed = get_unconfirmed_for_account(req.account_index).await;
        let mut pending: Vec<Transfer> = Vec::with_capacity(unconfirmed.len());
        for (txid, value, notes) in &unconfirmed {
            let transfer =
                unconfirmed_to_transfer(&mut conn, req.account_index, txid, value, notes)
                    .await?;
            if !req.subaddr_indices.is_empty()
                && !req.subaddr_indices.contains(&transfer.subaddr_index.minor)
            {
                continue;
            }
            pending.push(transfer);
        }

        pending.append(&mut transfers);

        Ok::<_, anyhow::Error>(GetTransfersResponse { r#in: pending })
    };
    let resp = inner.await.map_err(reject)?;
    Ok(warp::reply::json(&resp))
}


#[derive(Deserialize)]
struct GetFeeEstimateRequest {}

#[derive(Serialize)]
struct GetFeeEstimateResponse {
    fee: u64,
}

// Roughly estimate at 2 transparent in/out + 2 shielded in/out
// We cannot implement ZIP-321 here because we don't have
// the transaction
const LOGICAL_ACTION_FEE: u64 = 5000u64;

async fn get_fee_estimate(
    _req: GetFeeEstimateRequest,
    _state: RpcState,
) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&GetFeeEstimateResponse {
        fee: 4 * LOGICAL_ACTION_FEE,
    }))
}

#[derive(Deserialize)]
struct GetHeightRequest {}

#[derive(Serialize)]
struct GetHeightResponse {
    height: u32,
}

async fn get_height(
    _req: GetHeightRequest,
    state: RpcState,
) -> Result<impl Reply, Rejection> {
    let height = crate::api::network::get_current_height(&state.coin)
        .await
        .map_err(|_| warp::reject::reject())?;

    Ok(warp::reply::json(&GetHeightResponse { height }))
}

// ---------------------------------------------------------------------
// sync_info
// ---------------------------------------------------------------------

#[derive(Deserialize)]
struct SyncInfoRequest {}

#[derive(Serialize)]
struct SyncInfoResponse {
    target_height: u32,
    height: u32,
}

async fn sync_info(_req: SyncInfoRequest, state: RpcState) -> Result<impl Reply, Rejection> {
    let inner = async {
        let mut client = state.coin.client().await?;
        let target_height = client.latest_height().await?;

        let mut conn = state.coin.get_connection().await?;
        let accounts = list_accounts(&mut conn, state.coin.coin).await?;
        let mut min_height = target_height;
        for a in &accounts {
            if let Some(h) = get_sync_height(&mut conn, a.id as u32).await? {
                min_height = min_height.min(h);
            }
        }
        Ok::<_, anyhow::Error>(SyncInfoResponse {
            target_height,
            height: min_height,
        })
    };
    let resp = inner.await.map_err(reject)?;
    Ok(warp::reply::json(&resp))
}

// ---------------------------------------------------------------------
// request_scan
// ---------------------------------------------------------------------

#[derive(Deserialize)]
struct RequestSyncRequest {
    account_index: u32,
    fast: Option<bool>,
    transparent_limit: Option<u32>,
}

#[derive(Serialize)]
struct RequestSyncResponse {
    height: u32,
}

async fn request_scan(
    req: RequestSyncRequest,
    state: RpcState,
) -> Result<impl Reply, Rejection> {
    let inner = async {
        let fast = req.fast.unwrap_or_default();
        let transparent_limit = req
            .transparent_limit
            .unwrap_or(crate::sync::DEFAULT_TRANSPARENT_LIMIT);

        let current_height = crate::api::network::get_current_height(&state.coin).await?;

        let height = crate::sync::synchronize_impl(
            (),
            vec![req.account_index],
            current_height,
            100_000,
            transparent_limit,
            10_000,
            fast,
            &state.coin,
        )
        .await?;

        Ok::<_, anyhow::Error>(RequestSyncResponse {
            height: height as u32,
        })
    };
    let resp = inner.await.map_err(reject)?;
    Ok(warp::reply::json(&resp))
}

// ---------------------------------------------------------------------
// reorg (stub)
// ---------------------------------------------------------------------

async fn reorg(_state: RpcState) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&serde_json::json!({})))
}

// ---------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------

fn with_state(state: RpcState) -> impl Filter<Extract = (RpcState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

pub fn routes(
    state: RpcState,
) -> impl Filter<Extract = (impl Reply,), Error = Infallible> + Clone {
    fn json_body<T: serde::de::DeserializeOwned + Send>(
    ) -> impl Filter<Extract = (T,), Error = Rejection> + Clone {
        warp::path::full()
            .and(warp::method())
            .and(warp::body::content_length_limit(1024 * 1024))
            .and(warp::body::bytes())
            .and_then(
                |path: FullPath, method: Method, bytes: Bytes| async move {
                    let body_str = String::from_utf8_lossy(&bytes);

                    tracing::info!(
                        method = %method,
                        path = %path.as_str(),
                        body = %body_str,
                        "incoming request"
                    );

                    serde_json::from_slice::<T>(&bytes)
                        .map_err(|e| warp::reject::custom(JsonError(e)))
                },
            )
    }

    let create_account_r = warp::path("create_account")
        .and(warp::post())
        .and(json_body())
        .and(with_state(state.clone()))
        .and_then(create_account);

    let create_address_r = warp::path("create_address")
        .and(warp::post())
        .and(json_body())
        .and(with_state(state.clone()))
        .and_then(create_address);

    let get_accounts_r = warp::path("get_accounts")
        .and(warp::post())
        .and(json_body())
        .and(with_state(state.clone()))
        .and_then(get_accounts);

    let get_transaction_r = warp::path("get_transfer_by_txid")
        .and(warp::post())
        .and(json_body())
        .and(with_state(state.clone()))
        .and_then(get_transfer_by_txid);

    let get_transfers_r = warp::path("get_transfers")
        .and(warp::post())
        .and(json_body())
        .and(with_state(state.clone()))
        .and_then(get_transfers);

    let get_fee_estimate_r = warp::path("get_fee_estimate")
        .and(warp::post())
        .and(json_body())
        .and(with_state(state.clone()))
        .and_then(get_fee_estimate);

    let get_height_r = warp::path("get_height")
        .and(warp::post())
        .and(json_body())
        .and(with_state(state.clone()))
        .and_then(get_height);

    let sync_info_r = warp::path("sync_info")
        .and(warp::post())
        .and(json_body())
        .and(with_state(state.clone()))
        .and_then(sync_info);

    let request_scan_r = warp::path("request_scan")
        .and(warp::post())
        .and(json_body())
        .and(with_state(state.clone()))
        .and_then(request_scan);

    let reorg_r = warp::path("reorg")
        .and(warp::post())
        .and(with_state(state))
        .and_then(reorg);

    create_account_r
        .or(create_address_r)
        .or(get_accounts_r)
        .or(get_transaction_r)
        .or(get_transfers_r)
        .or(get_fee_estimate_r)
        .or(get_height_r)
        .or(sync_info_r)
        .or(request_scan_r)
        .or(reorg_r)
        .recover(handle_rejection)
}

#[derive(Clone, Default)]
pub struct UnconfirmedNote {
    pub pool: i32,
    pub scope: i32,
    pub value: BigDecimal,
    pub diversifier: String,
    pub diversifier_index: Option<BigDecimal>,
    pub address: Option<String>,
    pub memo: Option<String>,
}

pub async fn run_rpc_mempool(state: RpcState) {
    let coin = state.coin.clone();
    let network = coin.network();

    loop {
        let mut conn = match coin.get_connection().await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("get_connection error: {e:#}");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };
        let mut client = match coin.client().await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("client error: {e:#}");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        let (tx, mut rx) = tokio::sync::mpsc::channel::<MempoolMsg>(10);
        let notify_state = state.clone();

        // Consumer task: fire HTTP notifications for each message.
        let consumer = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    MempoolMsg::TxId(tx) => {
                        let txid = tx.txid;
                        tracing::info!("notify_mempool {txid}");
                        let all_notes = tx.notes;

                        for item in tx.amounts {
                            let (account, value) = (item.account, item.value);
                            tracing::info!("notify_mempool:value {value}");
                            let notes: Vec<UnconfirmedNote> = all_notes
                                .iter()
                                .filter(|n| n.account == account)
                                .map(|n| UnconfirmedNote {
                                    pool: n.pool as i32,
                                    scope: n.scope as i32,
                                    value: zats_to_zec(n.value),
                                    diversifier: n
                                        .diversifier
                                        .as_deref()
                                        .map(hex::encode)
                                        .unwrap_or_default(),
                                    diversifier_index: n.diversifier_index.map(BigDecimal::from),
                                    address: n.address.clone(),
                                    memo: n.memo.clone(),
                                })
                                .collect();
                            {
                                tracing::info!("notify_mempool start");
                                let mut mempool = MEMPOOL.lock().await;
                                let e = mempool.unconfirmed.entry(account);
                                let e = e.or_insert_with(HashMap::new);
                                e.insert(txid.clone(), (zats_to_zec(value), notes.clone()));
                                tracing::info!("notify_mempool end {txid}");
                            }
                            {
                                let account = account as i32;
                                tracing::info!("notify_txid={txid}, account={account}");
                                if let Some(url) = &notify_state.notify_tx_url {
                                    notify_tx(&notify_state.http, url, &txid, account).await;
                                } else {
                                    tracing::warn!(
                                        "notify_tx_url not configured, skipping notify for {txid}"
                                    );
                                }
                            }
                        }
                    }
                    MempoolMsg::BlockHeight(height) => {
                        tracing::info!("notify_block {height}");
                        if let Some(url) = &notify_state.notify_block_url {
                            notify_block(&notify_state.http, url, height).await;
                        }

                        // A new block landed: reconcile the mempool cache
                        // against confirmed transactions so we don't show
                        // the same tx as both pending and confirmed. We
                        // check every account currently tracked in the
                        // mempool cache.
                        if let Err(e) =
                            reconcile_mempool_with_chain(&notify_state.coin, &network).await
                        {
                            tracing::warn!("mempool reconciliation failed: {e:#}");
                        }
                    }
                }
            }
        });

        let cancel_token = CancellationToken::new();
        let runner = run_mempool_impl(tx, &network, &mut conn, &mut client, cancel_token);

        if let Err(error) = runner.await {
            tracing::error!("mempool watcher error: {error:#}");
        }

        // Ensure the consumer task is cleaned up before restarting.
        consumer.abort();
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

/// After a new block arrives, drop any mempool-cached txids that now have a
/// confirmed row in `transactions`, for every account currently tracked in
/// the mempool cache. Without this, pending transfers never disappear once
/// confirmed.
async fn reconcile_mempool_with_chain(coin: &Coin, _network: &Network) -> anyhow::Result<()> {
    let accounts: Vec<u32> = {
        let mempool = MEMPOOL.lock().await;
        mempool.unconfirmed.keys().copied().collect()
    };

    for account in accounts {
        let pending_txids: Vec<String> = {
            let mempool = MEMPOOL.lock().await;
            mempool
                .unconfirmed
                .get(&account)
                .map(|txs| txs.keys().cloned().collect())
                .unwrap_or_default()
        };
        if pending_txids.is_empty() {
            continue;
        }

        let mut conn = coin.get_connection().await?;
        for txid_hex in pending_txids {
            let mut txid = match hex::decode(&txid_hex) {
                Ok(t) => t,
                Err(_) => continue,
            };
            txid.reverse();

            let confirmed: Option<(i64,)> = sqlx::query_as(
                "SELECT height FROM transactions WHERE account = ?1 AND txid = ?2",
            )
            .bind(account)
            .bind(&txid)
            .fetch_optional(&mut *conn)
            .await?;

            if confirmed.is_some() {
                clear_confirmed_from_mempool(account, &txid_hex).await;
            }
        }
    }

    Ok(())
}

/// POST the txid as %s in the notify_tx_url (Monero convention).
async fn notify_tx(http: &reqwest::Client, url_template: &str, txid: &str, account_index: i32) {
    tracing::info!("notify_tx {txid}");
    let url = url_template
        .replace("%s", txid)
        .replace("%w", &account_index.to_string());
    if let Err(e) = http.get(&url).send().await {
        tracing::warn!("notify_tx failed for {txid}: {e:#}");
    }
}

/// GET/POST the height as %s in the notify_block_url (Monero convention).
async fn notify_block(http: &reqwest::Client, url_template: &str, height: u32) {
    let url = url_template.replace("%s", &height.to_string());
    if let Err(e) = http.get(&url).send().await {
        tracing::warn!("notify_block failed for height {height}: {e:#}");
    }
}

/// Entry point: spawns the mempool watcher, and runs the RPC server on its
/// own port, forever.
pub async fn run_rpc_server(
    coin: Coin,
    port: u16,
    notify_tx_url: Option<String>,
    notify_block_url: Option<String>,
) {
    let state = RpcState {
        coin,
        notify_tx_url,
        notify_block_url,
        address_creation_lock: Arc::new(Mutex::new(())),
        http: reqwest::Client::new(),
    };

    tokio::spawn(run_rpc_mempool(state.clone()));

    tracing::info!("zcash-walletd-compatible RPC listening on 0.0.0.0:{port}");
    warp::serve(routes(state)).run(([0, 0, 0, 0], port)).await;
}

#[derive(Default)]
pub struct Mempool {
    pub unconfirmed: HashMap<u32, HashMap<String, (BigDecimal, Vec<UnconfirmedNote>)>>,
}

pub static MEMPOOL: LazyLock<Mutex<Mempool>> = LazyLock::new(|| Mutex::new(Mempool::default()));
