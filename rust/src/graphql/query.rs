use crate::graphql::data::Account;
use crate::graphql::Context;
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
        let mut conn = context.db.acquire().await?;
        let accounts = query(
            "SELECT id_account, name, seed, passphrase, aindex, dindex, birth
            FROM accounts",
        )
        .map(row_to_account)
        .fetch_all(&mut *conn)
        .await?;
        Ok(accounts)
    }

    async fn accounts_by_name(name: String, context: &Context) -> FieldResult<Vec<Account>> {
        let mut conn = context.db.acquire().await?;
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
