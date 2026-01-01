use crate::db::{calculate_balance, get_sync_height};
use crate::graphql::data::{Account, Addresses, Balance, Transaction};
use crate::graphql::Context;
use bigdecimal::num_bigint::BigInt;
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::DateTime;
use juniper::{graphql_object, FieldResult};
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

    pub async fn transactions_by_account(
        id_account: i32,
        context: &Context,
    ) -> FieldResult<Vec<Transaction>> {
        let mut conn = context.coin.get_connection().await?;
        let transactions = query(
            "SELECT id_tx, txid, height, time, value, fee FROM transactions
        WHERE account = ?1 ORDER BY height DESC",
        )
        .bind(id_account)
        .map(row_to_transaction)
        .fetch_all(&mut *conn)
        .await?;
        Ok(transactions)
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

    pub async fn balance_by_account(id_account: i32, context: &Context) -> FieldResult<Balance> {
        let mut conn = context.coin.get_connection().await?;
        let height = get_sync_height(&mut conn, id_account as u32)
            .await?
            .unwrap_or_default();
        let b = calculate_balance(&mut conn, id_account as u32).await?;
        let balance = Balance {
            height: height as i32,
            transparent: zats_to_zec(b.0[0] as i64),
            sapling: zats_to_zec(b.0[1] as i64),
            orchard: zats_to_zec(b.0[2] as i64),
        };
        Ok(balance)
    }

    async fn address_by_account(
        id_account: i32,
        pools: Option<i32>,
        context: &Context,
    ) -> FieldResult<Addresses> {
        let mut conn = context.coin.get_connection().await?;
        let ua_pools = pools.unwrap_or(7) as u8;
        let addresses =
            crate::account::get_addresses(&context.coin.network(), &mut conn, id_account as u32, ua_pools)
                .await?;
        let addresses = Addresses {
            ua: addresses.ua,
            transparent: addresses.taddr,
            sapling: addresses.saddr,
            orchard: addresses.oaddr,
        };
        Ok(addresses)
    }

    async fn current_height(context: &Context) -> FieldResult<i32> {
        let height = crate::api::network::get_current_height(&context.coin).await?;
        Ok(height as i32)
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
    let id: i32 = r.get(0);
    let mut txid: Vec<u8> = r.get(1);
    let height: i32 = r.get(2);
    let time: i32 = r.get(3);
    let value: i64 = r.get(4);
    let fee: i64 = r.get(5);
    txid.reverse();
    let txid = hex::encode(&txid);
    let time = DateTime::from_timestamp_millis(time as i64 * 1000)
        .unwrap()
        .naive_local();
    let value = zats_to_zec(value);
    let fee = zats_to_zec(fee);
    Transaction {
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
