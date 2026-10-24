//! Market data for the native price chart: Binance USDⓈ-M futures klines.
//!
//! The chart shows the ZECUSDC perpetual, the symbol the TradingView embed
//! used to show. Binance serves klines without an API key, up to 1500 per
//! request, with volume. The base URL is the `klines_url` prop so tests can
//! point it at a local mock and users whose region Binance blocks can point it
//! at a mirror; nothing in the app sets it.

use anyhow::{anyhow, Result};
use serde_json::Value;
use sqlx::SqliteConnection;

use crate::api::network::Candle;
use crate::net::http;

/// The klines API base used when the wallet has no `klines_url` prop.
pub const DEFAULT_KLINES_URL: &str = "https://fapi.binance.com";

/// The only symbol the chart shows.
pub const KLINES_SYMBOL: &str = "ZECUSDC";

/// Binance's per-request cap on the number of klines.
pub const MAX_KLINES: u32 = 1500;

/// The intervals Binance accepts; anything else is rejected before the request.
const INTERVALS: [&str; 15] = [
    "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h", "1d", "3d", "1w", "1M",
];

/// Base URL of the klines API for this wallet: the `klines_url` prop, without
/// a trailing slash, or [`DEFAULT_KLINES_URL`].
pub async fn klines_url(connection: &mut SqliteConnection) -> Result<String> {
    let url = crate::db::get_prop(connection, "klines_url").await?;
    Ok(url
        .map(|u| u.trim_end_matches('/').to_string())
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| DEFAULT_KLINES_URL.to_string()))
}

/// The request URL for the most recent `limit` klines of `interval`.
///
/// `interval` must be one of Binance's; `limit` must be 1..=[`MAX_KLINES`].
pub fn klines_request_url(base: &str, interval: &str, limit: u32) -> Result<String> {
    if !INTERVALS.contains(&interval) {
        return Err(anyhow!("Unsupported kline interval: {interval}"));
    }
    if limit == 0 || limit > MAX_KLINES {
        return Err(anyhow!("Kline limit must be 1..={MAX_KLINES}, got {limit}"));
    }
    Ok(format!(
        "{base}/fapi/v1/klines?symbol={KLINES_SYMBOL}&interval={interval}&limit={limit}"
    ))
}

/// Parses Binance's kline response: an array of arrays whose first six
/// entries are open time in milliseconds, then open, high, low, close and
/// volume as decimal strings. The remaining entries are ignored.
pub fn parse_klines(rep: &Value) -> Result<Vec<Candle>> {
    let rows = rep.as_array().ok_or(anyhow!("klines not array"))?;
    let mut candles = Vec::with_capacity(rows.len());
    for row in rows {
        let fields = row.as_array().ok_or(anyhow!("kline not array"))?;
        if fields.len() < 6 {
            return Err(anyhow!("kline has {} fields, expected at least 6", fields.len()));
        }
        let time = fields[0].as_u64().ok_or(anyhow!("kline open time not int"))?;
        let number = |i: usize, name: &str| -> Result<f64> {
            let s = fields[i]
                .as_str()
                .ok_or(anyhow!("kline {name} not string"))?;
            s.parse::<f64>()
                .map_err(|e| anyhow!("kline {name} {s:?}: {e}"))
        };
        candles.push(Candle {
            time: (time / 1000) as u32,
            open: number(1, "open")?,
            high: number(2, "high")?,
            low: number(3, "low")?,
            close: number(4, "close")?,
            volume: number(5, "volume")?,
        });
    }
    Ok(candles)
}

/// Fetches the most recent `limit` klines of `interval`, oldest first.
pub async fn get_klines(
    base: &str,
    interval: &str,
    limit: u32,
    client: &reqwest::Client,
) -> Result<Vec<Candle>> {
    let url = klines_request_url(base, interval, limit)?;
    let rep: Value = http::http_get(client, &[&url], http::RetryPolicy::default())
        .await?
        .json()
        .await?;
    parse_klines(&rep)
}

#[cfg(test)]
mod tests {
    use super::{get_klines, klines_request_url, parse_klines, MAX_KLINES};
    use serde_json::json;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn two_klines() -> serde_json::Value {
        json!([
            [
                1761177600000u64, "1313.83", "1336.78", "1185.50", "1217.00", "91308.209",
                1761263999999u64, "113000000.0", 250000, "45000.0", "56000000.0", "0"
            ],
            [
                1761264000000u64, "1217.27", "1257.15", "1156.52", "1180.82", "92673.484",
                1761350399999u64, "110000000.0", 240000, "46000.0", "55000000.0", "0"
            ]
        ])
    }

    #[test]
    fn parses_binance_rows() {
        let candles = parse_klines(&two_klines()).unwrap();
        assert_eq!(candles.len(), 2);
        let c = &candles[0];
        assert_eq!(c.time, 1761177600);
        assert_eq!(c.open, 1313.83);
        assert_eq!(c.high, 1336.78);
        assert_eq!(c.low, 1185.50);
        assert_eq!(c.close, 1217.00);
        assert_eq!(c.volume, 91308.209);
        assert_eq!(candles[1].time, 1761264000);
    }

    #[test]
    fn rejects_malformed_rows() {
        assert!(parse_klines(&json!({"code": -1121})).is_err());
        assert!(parse_klines(&json!([[1761177600000u64, "1.0"]])).is_err());
        assert!(parse_klines(&json!([[1761177600000u64, "x", "1", "1", "1", "1"]])).is_err());
        assert!(parse_klines(&json!([["t", "1", "1", "1", "1", "1"]])).is_err());
        assert_eq!(parse_klines(&json!([])).unwrap().len(), 0);
    }

    #[test]
    fn builds_and_validates_request_url() {
        assert_eq!(
            klines_request_url("https://fapi.binance.com", "1d", 1500).unwrap(),
            "https://fapi.binance.com/fapi/v1/klines?symbol=ZECUSDC&interval=1d&limit=1500"
        );
        assert!(klines_request_url("http://x", "2d", 10).is_err());
        assert!(klines_request_url("http://x", "1d; rm", 10).is_err());
        assert!(klines_request_url("http://x", "1d", 0).is_err());
        assert!(klines_request_url("http://x", "1d", MAX_KLINES + 1).is_err());
    }

    #[tokio::test]
    async fn fetches_from_a_local_server() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let body = two_klines().to_string();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                request.push(socket.read_u8().await.unwrap());
            }
            assert!(request.starts_with(
                b"GET /fapi/v1/klines?symbol=ZECUSDC&interval=4h&limit=2 HTTP/1.1\r\n"
            ));
            socket
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
        });
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let candles = tokio::time::timeout(
            Duration::from_secs(5),
            get_klines(&base, "4h", 2, &client),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(candles.len(), 2);
        assert_eq!(candles[1].close, 1180.82);
        server.await.unwrap();
    }
}
