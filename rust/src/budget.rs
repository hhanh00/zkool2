use anyhow::{anyhow, Result};
use serde_json::Value;
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};

async fn get_historical_prices_all() -> Result<Vec<PriceQuote>> {
    // 1, 90 and 365 are the max day ranges per interval
    let mut pqs = get_historical_prices(1).await?;
    pqs.extend(get_historical_prices(90).await?);
    pqs.extend(get_historical_prices(365).await?);
    pqs.sort_by_key(|pq| pq.time);
    Ok(pqs)
}

async fn get_historical_prices(days: u32) -> Result<Vec<PriceQuote>> {
    let historical_price_url = format!(
        "https://api.coingecko.com/api/v3/coins/zcash/market_chart?vs_currency=usd&days={days}"
    );
    let rep: Value = reqwest::get(&historical_price_url).await?.json().await?;
    let prices = rep
        .pointer("/prices")
        .ok_or(anyhow!("No /prices"))?
        .as_array()
        .ok_or(anyhow!("prices not array"))?;
    let mut pqs = vec![];
    for p in prices {
        let pt = p.as_array().ok_or(anyhow!("price/time not array"))?;
        let time = pt[0].as_u64().ok_or(anyhow!("time not int"))? as u64;
        let price = pt[1].as_f64().ok_or(anyhow!("price not double"))?;
        let pq = PriceQuote {
            time: (time / 1000) as u32,
            price,
        };
        pqs.push(pq);
    }

    Ok(pqs)
}

async fn fetch_missing_tx_prices(
    connection: &mut SqliteConnection,
    account: u32,
) -> Result<Vec<TxUSD>> {
    let txs = sqlx::query(
        "SELECT id_tx, time FROM transactions
            WHERE account = ?1 AND price IS NULL ORDER BY time",
    )
    .bind(account)
    .map(|r: SqliteRow| {
        let id: u32 = r.get(0);
        let time: u32 = r.get(1);
        TxUSD {
            id,
            time,
            price: 0.0,
        }
    })
    .fetch_all(&mut *connection)
    .await?;
    Ok(txs)
}

async fn fill_historical_prices(txs: &mut [TxUSD], pqs: &[PriceQuote]) -> Result<()> {
    assert!(!pqs.is_empty());
    let mut i = 0;
    for tx in txs.iter_mut() {
        loop {
            let time = if i == pqs.len() {
                u32::MAX
            } else {
                pqs[i].time
            };
            if time > tx.time {
                break;
            }
            i += 1;
        }
        let pq = if i == 0 { &pqs[0] } else { &pqs[i - 1] };
        tx.price = pq.price;
    }
    Ok(())
}

pub async fn store_tx_prices(connection: &mut SqliteConnection, txs: &[TxUSD]) -> Result<()> {
    for tx in txs {
        sqlx::query("UPDATE transactions SET price = ?2 WHERE id_tx = ?1")
            .bind(tx.id)
            .bind(tx.price)
            .execute(&mut *connection)
            .await?;
    }
    Ok(())
}

pub async fn fill_missing_tx_prices(connection: &mut SqliteConnection, account: u32) -> Result<()> {
    let mut txs = fetch_missing_tx_prices(&mut *connection, account).await?;
    let pqs = get_historical_prices_all().await?;
    fill_historical_prices(&mut txs, &pqs).await?;
    store_tx_prices(&mut *connection, &txs).await?;
    Ok(())
}

pub async fn merge_pending_txs(
    connection: &mut SqliteConnection,
    account: u32,
    height: u32,
) -> Result<()> {
    sqlx::query(
        "WITH upd AS (SELECT t.id_tx, p.price, p.category
        FROM pending_txs p JOIN transactions t ON p.txid = t.txid
        WHERE t.account = ?1)
        UPDATE transactions SET price = upd.price, category = upd.category
        FROM upd WHERE upd.id_tx = transactions.id_tx",
    )
    .bind(account)
    .execute(&mut *connection)
    .await?;
    // delete pending txs that could not be merged for more than 100 blocks
    // they were probably never mined
    sqlx::query("DELETE FROM pending_txs WHERE account = ?1 AND height < ?2")
        .bind(account)
        .bind(height - 100)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

pub async fn fetch_category_amounts(connection: &mut SqliteConnection,
    account: u32, from: u32, to: u32, income: bool) -> Result<Vec<(String, f64)>> {
    let category_amts = sqlx::query(
        "SELECT c.name, SUM(value) * price / 1e8 AS amount
        FROM transactions t
        JOIN categories c ON c.id_category = t.category
        WHERE account = ?1 AND c.income = ?2 AND t.time >= ?3
        AND t.time < ?4
        GROUP BY id_category")
    .bind(account)
    .bind(income)
    .bind(from)
    .bind(to)
    .map(|r: SqliteRow| {
        let category: String = r.get(0);
        let amount: f64 = r.get(1);
        (category, amount)
    })
    .fetch_all(&mut *connection)
    .await?;
    Ok(category_amts)
}

pub struct PriceQuote {
    pub time: u32,
    pub price: f64,
}

#[derive(Debug)]
pub struct TxUSD {
    pub id: u32,
    pub time: u32,
    pub price: f64,
}
