use anyhow::Result;
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};

pub async fn get_tx_without_usd_range(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<(Option<u32>, Option<u32>)> {
    let (min, max) = sqlx::query(
        "SELECT MIN(time), MAX(time) FROM transactions
        WHERE account = ?1 AND fiat IS NULL",
    )
    .bind(account)
    .map(|r: SqliteRow| (r.get::<Option<u32>, _>(0), r.get::<Option<u32>, _>(1)))
    .fetch_one(connection)
    .await?;
    Ok((min, max))
}
