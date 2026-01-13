use anyhow::{Error, Result};
use dataloader::cached::Loader;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use zcash_keys::address::UnifiedAddress;
use zcash_keys::encoding::AddressCodec;
use zcash_keys::keys::UnifiedFullViewingKey;

use crate::api::coin::{Coin, Network};
use crate::api::pay::{DustChangePolicy, PcztPackage};
use crate::db::{calculate_balance, get_sync_height};
use crate::graphql::data::{
    Account, Addresses, Balance, DKGStatus, Note, Transaction, UnconfirmedTx,
};
use crate::graphql::mutation::{Output, Payment, UnsignedTx};
use crate::graphql::mutation::MEMPOOL;
use crate::graphql::Context;
use crate::pay::TxPlan;
use crate::tiu;
use bigdecimal::num_bigint::BigInt;
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, NaiveDateTime};
use juniper::{graphql_object, FieldError, FieldResult};
use orchard::keys::Scope;
use sqlx::{query, sqlite::SqliteRow, Row};

pub struct Query {}

#[graphql_object]
#[graphql(context = Context)]
impl Query {
    fn api_version() -> &'static str {
        "1.0"
    }

    async fn accounts(context: &Context) -> FieldResult<Vec<Account>> {
        let mut conn = context.coin.get_connection().await?;
        let accounts = query(
            "SELECT id_account, name, seed, passphrase, aindex, dindex, birth
            FROM accounts ORDER BY id_account",
        )
        .map(row_to_account)
        .fetch_all(&mut *conn)
        .await?;
        Ok(accounts)
    }

    async fn accounts_by_name(name: String, context: &Context) -> FieldResult<Vec<Account>> {
        let mut conn = context.coin.get_connection().await?;
        let accounts = query(
            "SELECT id_account, name, seed, passphrase, aindex, dindex, birth FROM accounts
            WHERE name = ?1",
        )
        .bind(&name)
        .map(row_to_account)
        .fetch_all(&mut *conn)
        .await?;
        Ok(accounts)
    }

    pub async fn account_by_id(id_account: i32, context: &Context) -> FieldResult<Account> {
        let mut conn = context.coin.get_connection().await?;
        let account = query(
            "SELECT id_account, name, seed, passphrase, aindex, dindex, birth FROM accounts
            WHERE id_account = ?1",
        )
        .bind(id_account)
        .map(row_to_account)
        .fetch_optional(&mut *conn)
        .await?
        .ok_or(anyhow::anyhow!("Unknown account"))?;
        Ok(account)
    }

    pub async fn transactions_by_account(
        id_account: i32,
        height: Option<i32>,
        context: &Context,
    ) -> FieldResult<Vec<Transaction>> {
        let height = height.unwrap_or_default();
        let mut conn = context.coin.get_connection().await?;
        let transactions = query(
            "SELECT account, id_tx, txid, height, time, value, fee FROM transactions
            WHERE account = ?1 AND height >= ?2 ORDER BY height DESC",
        )
        .bind(id_account)
        .bind(height)
        .map(row_to_transaction)
        .fetch_all(&mut *conn)
        .await?;
        Ok(transactions)
    }

    pub async fn transaction_by_id(
        id_account: i32,
        txid: String,
        context: &Context,
    ) -> FieldResult<Transaction> {
        let mut txid = hex::decode(&txid)?;
        txid.reverse();
        let mut conn = context.coin.get_connection().await?;
        let transaction = query(
            "SELECT account, id_tx, txid, height, time, value, fee FROM transactions
            WHERE account = ?1 AND txid = ?2 ORDER BY height DESC",
        )
        .bind(id_account)
        .bind(&txid)
        .map(row_to_transaction)
        .fetch_optional(&mut *conn)
        .await?
        .ok_or(FieldError::new("Unknown txid", juniper::Value::Null))?;
        Ok(transaction)
    }

    pub async fn memos_by_transaction(
        id_transaction: i32,
        context: &Context,
    ) -> FieldResult<Vec<String>> {
        let mut conn = context.coin.get_connection().await?;
        let memos = query(
            "SELECT memo_text FROM memos
            WHERE tx = ?1 AND memo_text IS NOT NULL ORDER BY id_memo",
        )
        .bind(id_transaction)
        .map(|r: SqliteRow| r.get::<String, _>(0))
        .fetch_all(&mut *conn)
        .await?;
        Ok(memos)
    }

    pub async fn balance_by_account(
        id_account: i32,
        height: Option<i32>,
        context: &Context,
    ) -> FieldResult<Balance> {
        let height = height.map(|h| h as u32);
        let mut conn = context.coin.get_connection().await?;
        let current_height = get_sync_height(&mut conn, id_account as u32).await?;
        let height = height.or(current_height);
        let b = calculate_balance(&mut conn, id_account as u32, height).await?;
        let total = b.0[0] + b.0[1] + b.0[2];
        let balance = Balance {
            height: height.map(|h| h as i32),
            transparent: zats_to_zec(b.0[0] as i64),
            sapling: zats_to_zec(b.0[1] as i64),
            orchard: zats_to_zec(b.0[2] as i64),
            total: zats_to_zec(total as i64),
        };
        Ok(balance)
    }

    async fn address_by_account(
        id_account: i32,
        pools: Option<i32>,
        context: &Context,
    ) -> FieldResult<Addresses> {
        let mut conn = context.coin.get_connection().await?;
        let ua_pools = pools.unwrap_or(6) as u8;
        let addresses = crate::account::get_addresses(
            &context.coin.network(),
            &mut conn,
            id_account as u32,
            ua_pools,
        )
        .await?;
        let addresses = Addresses {
            ua: addresses.ua,
            transparent: addresses.taddr,
            sapling: addresses.saddr,
            orchard: addresses.oaddr,
        };
        Ok(addresses)
    }

    async fn unconfirmed_by_account(id_account: i32) -> FieldResult<Vec<UnconfirmedTx>> {
        let mempool = MEMPOOL.lock().await;
        if let Some(unconfirmed_txs) = mempool.unconfirmed.get(&(id_account as u32)) {
            let txs: Vec<_> = unconfirmed_txs
                .iter()
                .map(|(txid, value)| UnconfirmedTx {
                    txid: txid.clone(),
                    value: value.clone(),
                })
                .collect();
            return Ok(txs);
        }
        Ok(vec![])
    }

    async fn notes_by_account(id_account: i32, context: &Context) -> FieldResult<Vec<Note>> {
        let network = context.coin.network();
        let mut conn = context.coin.get_connection().await?;
        let ufvk = crate::key::get_account_ufvk(&network, &mut conn, id_account as u32, 7).await?;
        let ufvk = UnifiedFullViewingKey::decode(&network, &ufvk)?;

        let notes = crate::db::get_notes(&mut conn, id_account as u32).await?;
        let notes: Vec<_> = notes
            .into_iter()
            .map(|n| resolve_note(&network, &ufvk, n))
            .collect();
        Ok(notes)
    }

    async fn current_height(context: &Context) -> FieldResult<i32> {
        let height = crate::api::network::get_current_height(&context.coin).await?;
        Ok(height as i32)
    }

    async fn prepare_send(
        id_account: i32,
        payment: Payment,
        context: &Context,
    ) -> FieldResult<String> {
        let tx = prepare_tx(id_account, payment, &context.coin).await?;
        let txbin = bincode::encode_to_vec(&tx, bincode::config::standard())?;
        let txhex = hex::encode(&txbin);
        Ok(txhex)
    }

    async fn decode_pczt(pczt: String, context: &Context) -> FieldResult<UnsignedTx> {
        let pczt = hex::decode(&pczt)?;
        let (pczt, _) =
            bincode::decode_from_slice::<PcztPackage, _>(&pczt, bincode::config::standard())?;
        let tx_plan = TxPlan::from_package(&context.coin.network(), &pczt)?;
        let unsigned = tx_plan.to_unsigned_tx();
        Ok(unsigned)
    }

    async fn sign_tx(id_account: i32, pczt: String, context: &Context) -> FieldResult<String> {
        let mut connection = context.coin.get_connection().await?;
        let pczt = hex::decode(&pczt)?;
        let (pczt, _) =
            bincode::decode_from_slice::<PcztPackage, _>(&pczt, bincode::config::standard())?;
        let signed = crate::pay::plan::sign_transaction(&mut connection, id_account as u32, &pczt).await?;
        let tx_bin = crate::pay::plan::extract_transaction(&signed).await?;
        let tx = hex::encode(&tx_bin);
        Ok(tx)
    }

    async fn broadcast_tx(tx_bytes: String, context: &Context) -> FieldResult<String> {
        let tx = hex::decode(&tx_bytes)?;
        let mut client = context.coin.client().await?;
        let height = client.latest_height().await?;
        let result = client.post_transaction(height, &tx).await?;
        Ok(result)
    }
}

impl TxPlan {
    pub fn to_unsigned_tx(self) -> UnsignedTx {
        let recipients: Vec<_> = self
            .outputs
            .into_iter()
            .map(|o| Output {
                address: o.address,
                amount: zats_to_zec(o.amount as i64),
            })
            .collect();
        let fee = zats_to_zec(self.fee as i64);
        UnsignedTx {
            recipients,
            fee,
        }
    }
}

pub async fn prepare_tx(
    id_account: i32,
    payment: Payment,
    coin: &Coin,
) -> FieldResult<crate::api::pay::PcztPackage> {
    let mut recipients = vec![];
    for r in payment.recipients {
        recipients.push(crate::pay::Recipient {
            address: r.address,
            amount: zec_to_zats(r.amount)? as u64,
            pools: None,
            user_memo: r.memo,
            memo_bytes: None,
            price: None,
        });
    }
    let network = coin.network();
    let mut connection = coin.get_connection().await?;
    let mut client = coin.client().await?;

    let pczt = crate::pay::plan::plan_transaction(
        &network,
        &mut connection,
        &mut client,
        id_account as u32,
        payment.src_pools.unwrap_or(7) as u8,
        &recipients,
        false,
        payment.recipient_pays_fee.unwrap_or_default(),
        DustChangePolicy::Discard,
        None,
    )
    .await?;
    Ok(pczt)
}

fn resolve_note(
    network: &Network,
    ufvk: &UnifiedFullViewingKey,
    n: crate::api::account::TxNote,
) -> Note {
    let address = match n.pool {
        1 => {
            let div = n.diversifier.as_ref().unwrap().clone();
            let d = sapling_crypto::keys::Diversifier(tiu!(div));
            let sfvk = ufvk.sapling().unwrap();
            let address = if n.scope == 0 {
                sfvk.diversified_address(d)
            } else {
                sfvk.diversified_change_address(d)
            }
            .unwrap();
            Some(address.encode(&network))
        }
        2 => {
            let div = n.diversifier.as_ref().unwrap().clone();
            let d = orchard::keys::Diversifier::from_bytes(tiu!(div));
            let ofvk = ufvk.orchard().unwrap();
            let scope = if n.scope == 0 {
                Scope::External
            } else {
                Scope::Internal
            };
            let address = ofvk.address(d, scope);
            let ua = UnifiedAddress::from_receivers(Some(address), None, None).unwrap();
            Some(ua.encode(&network))
        }
        _ => None,
    };

    Note {
        height: n.height as i32,
        pool: n.pool as i32,
        tx: n.tx as i32,
        scope: n.scope as i32,
        diversifier: n.diversifier.map(|d| hex::encode(&d)).unwrap_or_default(),
        address: address.unwrap_or_default(),
        value: zats_to_zec(n.value as i64),
    }
}

#[graphql_object]
impl Account {
    pub fn id(&self) -> i32 {
        self.id
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn seed(&self) -> Option<String> {
        self.seed.clone()
    }
    pub fn passphrase(&self) -> Option<String> {
        self.passphrase.clone()
    }
    pub fn aindex(&self) -> i32 {
        self.aindex
    }
    pub fn dindex(&self) -> i32 {
        self.dindex
    }
    pub fn birth(&self) -> i32 {
        self.birth
    }

    pub async fn transactions(&self, context: &Context) -> FieldResult<Vec<Transaction>> {
        let txs = Query::transactions_by_account(self.id, None, context).await?;
        Ok(txs)
    }

    pub async fn dkg_status() -> FieldResult<DKGStatus> {
        crate::graphql::frost::dkg_status().await
    }
}

#[graphql_object]
impl Transaction {
    pub fn id(&self) -> i32 {
        self.id
    }
    pub fn txid(&self) -> String {
        self.txid.clone()
    }
    pub fn account(&self) -> i32 {
        self.account
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn time(&self) -> NaiveDateTime {
        self.time
    }
    pub fn value(&self) -> BigDecimal {
        self.value.clone()
    }
    pub fn fee(&self) -> BigDecimal {
        self.fee.clone()
    }

    pub async fn notes(&self, context: &Context) -> FieldResult<Vec<Note>> {
        let mut conn = context.coin.get_connection().await?;
        let network = context.coin.network();
        let ufvk =
            crate::key::get_account_ufvk(&network, &mut conn, self.account as u32, 7).await?;
        let ufvk = UnifiedFullViewingKey::decode(&network, &ufvk)?;
        let mut txid = hex::decode(&self.txid)?;
        txid.reverse();
        let notes = crate::db::get_notes_txid(&mut conn, self.account as u32, &txid).await?;
        let notes: Vec<_> = notes
            .into_iter()
            .map(|n| resolve_note(&network, &ufvk, n))
            .collect();
        Ok(notes)
    }

    pub async fn memos(&self, context: &Context) -> FieldResult<Vec<String>> {
        let mut conn = context.coin.get_connection().await?;
        let mut txid = hex::decode(&self.txid)?;
        txid.reverse();
        let memos = crate::db::get_memos_txid(&mut conn, self.account as u32, &txid).await?;
        let memos: Vec<_> = memos.into_iter().filter_map(|m| m.memo).collect();
        Ok(memos)
    }
}

#[graphql_object]
impl Note {
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn pool(&self) -> i32 {
        self.pool
    }
    pub fn value(&self) -> BigDecimal {
        self.value.clone()
    }
    pub fn address(&self) -> String {
        self.address.clone()
    }
    pub fn scope(&self) -> i32 {
        self.scope
    }
    pub fn diversifier(&self) -> String {
        self.diversifier.clone()
    }
    pub async fn tx(&self, context: &Context) -> FieldResult<Transaction> {
        context
            .tx_loader
            .try_load(self.tx)
            .await
            .map_err(|_| anyhow::anyhow!("No tx exists for ID {}", self.tx))?
            .map_err(|err| FieldError::new(err.to_string(), juniper::Value::Null))
    }
}

fn row_to_account(r: SqliteRow) -> Account {
    let id: i32 = r.get(0);
    let name: String = r.get(1);
    let seed: Option<String> = r.get(2);
    let passphrase: Option<String> = r.get(3);
    let aindex: i32 = r.get(4);
    let dindex: i32 = r.get(5);
    let birth: i32 = r.get(6);
    Account {
        id,
        name,
        seed,
        passphrase,
        aindex,
        dindex,
        birth,
    }
}

fn row_to_transaction(r: SqliteRow) -> Transaction {
    let account: i32 = r.get(0);
    let id: i32 = r.get(1);
    let mut txid: Vec<u8> = r.get(2);
    let height: i32 = r.get(3);
    let time: i32 = r.get(4);
    let value: i64 = r.get(5);
    let fee: i64 = r.get(6);
    txid.reverse();
    let txid = hex::encode(&txid);
    let time = DateTime::from_timestamp_millis(time as i64 * 1000)
        .unwrap()
        .naive_local();
    let value = zats_to_zec(value);
    let fee = zats_to_zec(fee);
    Transaction {
        account,
        id,
        txid,
        height,
        time,
        value,
        fee,
    }
}

pub fn zats_to_zec(zats: i64) -> bigdecimal::BigDecimal {
    let digits = BigInt::from_i64(zats).unwrap();
    BigDecimal::from_bigint(digits, 8)
}

pub fn zec_to_zats(zec: bigdecimal::BigDecimal) -> FieldResult<i64> {
    let (digit, _) = zec.with_scale(8).into_bigint_and_scale();
    let digits = digit.to_u64_digits().1;
    if digits.len() > 1 {
        return Err("Invalid amount".into());
    }
    Ok(digits[0] as i64)
}

pub struct TxDataLoader {}

pub struct TxBatcher {
    pub pool: SqlitePool,
}

impl dataloader::BatchFn<i32, Result<Transaction, Arc<Error>>> for TxBatcher {
    async fn load(&mut self, keys: &[i32]) -> HashMap<i32, Result<Transaction, Arc<Error>>> {
        let f = async move {
            let mut conn = self.pool.acquire().await?;
            query("CREATE TEMP TABLE tmp_ids (id INTEGER PRIMARY KEY)")
                .execute(&mut *conn)
                .await?;
            for id in keys {
                query("INSERT INTO tmp_ids(id) VALUES (?1)")
                    .bind(*id)
                    .execute(&mut *conn)
                    .await?;
            }
            let txs = query(
                "SELECT account, id_tx, txid, height, time, value, fee FROM transactions t
            JOIN tmp_ids i ON t.id_tx = i.id",
            )
            .map(row_to_transaction)
            .fetch_all(&mut *conn)
            .await?;
            let txs: HashMap<i32, Result<Transaction, Arc<Error>>> =
                txs.into_iter().map(|tx| (tx.id, Ok(tx))).collect();
            tracing::info!("{txs:?}");
            Ok::<_, anyhow::Error>(txs)
        };

        let txs = f.await.unwrap();
        txs
    }
}

pub type TxLoader = Loader<i32, Result<Transaction, Arc<anyhow::Error>>, TxBatcher>;

pub fn new_tx_loader(pool: SqlitePool) -> TxLoader {
    TxLoader::new(TxBatcher { pool }).with_yield_count(100)
}
