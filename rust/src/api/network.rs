use anyhow::{anyhow, Result};
use flutter_rust_bridge::frb;
use serde::Deserialize;
use serde_json::Value;

use crate::{coin::ServerType, get_coin};

#[frb]
pub async fn init_datadir(directory: &str) -> Result<()> {
    crate::coin::init_datadir(directory).await
}

#[frb(sync)]
pub fn set_lwd(server_type: ServerType, lwd: &str) {
    let mut coin = crate::coin::COIN.lock().unwrap();
    coin.set_url(server_type, lwd);
}

#[frb(sync)]
pub fn set_use_tor(use_tor: bool) {
    let mut coin = crate::coin::COIN.lock().unwrap();
    coin.set_use_tor(use_tor);
}

pub async fn get_current_height() -> Result<u32> {
    let c = crate::get_coin!();
    let mut client = c.client().await?;
    let height = client.latest_height().await?;
    Ok(height as u32)
}

pub async fn get_coingecko_price() -> Result<f64> {
    let rep =
        reqwest::get("https://api.coingecko.com/api/v3/simple/price?ids=zcash&vs_currencies=usd")
            .await?
            .error_for_status()?
            .json::<ZcashUSD>()
            .await?;
    Ok(rep.zcash.usd)
}

#[frb]
pub async fn get_network_name() -> String {
    let c = get_coin!();
    c.get_name().to_string()
}

#[derive(Deserialize)]
struct Usd {
    usd: f64,
}

#[derive(Deserialize)]
struct ZcashUSD {
    zcash: Usd,
}

pub async fn get_historical_prices(days: u32) -> Result<Vec<PriceQuote>> {
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
        let time = pt[0].as_u64().ok_or(anyhow!("time not int"))? as u32;
        let price = pt[1].as_f64().ok_or(anyhow!("price not double"))?;
        let pq = PriceQuote { time, price };
        pqs.push(pq);
    }

    Ok(pqs)
}

pub struct PriceQuote {
    pub time: u32,
    pub price: f64,
}

pub struct TxUSD {
    pub id: u32,
    pub time: u32,
    pub value: i64,
}

#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf, str::FromStr};

    use anyhow::Result;
    use sqlx::{sqlite::SqliteRow, Row};

    use crate::{
        api::{db::open_database, network::TxUSD},
        get_coin,
    };

    #[tokio::test]
    pub async fn fill_transaction_usd() -> Result<()> {
        let home = env::var("HOME").unwrap();
        let db_path = PathBuf::from_str(&home)?
            .join("Library/Containers/cc.methyl.zkool/Data/Documents/testdb.db");
        open_database(&db_path.to_string_lossy(), None).await?;
        let c = get_coin!();
        let mut connection = c.get_connection().await?;

        let account = 1;

        sqlx::query(
            "SELECT id_tx, time, value FROM transactions 
            WHERE account = ?1 AND fiat IS NULL ORDER BY time",
        )
        .bind(account)
        .map(|r: SqliteRow| {
            let id: u32 = r.get(0);
            let time: u32 = r.get(1);
            let value: i64 = r.get(2);
            TxUSD { id, time, value }
        })
        .fetch_all(&mut *connection)
        .await?;

        Ok(())
    }

    #[tokio::test]
    pub async fn test_get_historical_prices() -> Result<()> {
        let pqs = super::get_historical_prices(1).await?;
        for pq in pqs {
            println!("{} {}", pq.time, pq.price);
        }
        Ok(())
    }
}
