// rust/src/rpc-cli.rs
//
// Standalone zcash-walletd-compatible RPC server for zkool.
// Talks to the same SQLite db and lightwalletd/zebra backend as
// zkool_graphql, but exposes a monero-wallet-rpc-shaped JSON API instead
// of GraphQL. Can run alongside zkool_graphql or entirely on its own.

use anyhow::Result;
use clap::Parser;
use figment::providers::{Format, Serialized, Toml};
use figment::Figment;
use rlz::api::coin::Coin;
use rlz::rpc::ensure_schema;
use serde::{Deserialize, Serialize};
use rlz::rpc::run_rpc_server;

#[serde_with::skip_serializing_none]
#[derive(Parser, Serialize, Deserialize, Debug)]
pub struct Config {
    #[clap(short, long, value_parser)]
    pub config_path: Option<String>,
    #[clap(short, long, value_parser)]
    pub db_path: Option<String>,
    #[clap(short, long, value_parser)]
    pub lwd_url: Option<String>,
    #[clap(short, long, value_parser)]
    pub port: Option<u16>,
    #[clap(short = 'Z', long, value_parser, default_missing_value = "true", num_args = 0..=1, require_equals = false)]
    pub zebra: Option<bool>,
    #[clap(short = 'C', long, value_parser)]
    pub coin: Option<u8>,
    #[clap(long, value_parser)]
    pub notify_tx_url: Option<String>,
    #[clap(long, value_parser)]
    pub notify_block_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .unwrap();
    let subscriber = tracing_subscriber::fmt().with_ansi(false).compact().finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let c = Config::parse();
    let config_path = c.config_path.clone().unwrap_or("zkool.toml".to_string());
    let config: Config = Figment::new()
        .merge(Toml::file(&config_path))
        .merge(Serialized::defaults(c))
        .extract()?;

    let Config {
        db_path,
        lwd_url,
        port,
        zebra,
        coin,
        notify_tx_url,
        notify_block_url,
        ..
    } = config;

    let db_path = db_path.unwrap_or("zkool.db".to_string());
    let lwd_url = lwd_url.unwrap_or("https://zec.rocks".to_string());
    let port = port.unwrap_or(8000);
    let zebra = zebra.unwrap_or_default();
    let server_type: u8 = if zebra { 1 } else { 0 };

    let sapling_status = rlz::api::sapling::check_sapling_params();
    if !sapling_status.downloaded {
        tracing::info!("Sapling parameters not found, downloading …");
        rlz::api::sapling::download_sapling_params().await?;
    }

    tracing::info!("db_path {db_path} lwd_url {lwd_url} port {port} zebra {zebra}");
    let coin = Coin::new(coin)
        .open_database(db_path, None)
        .await?
        .set_lwd(server_type, lwd_url)?;

    if let Err(e) = ensure_schema(&coin).await {
        tracing::error!("Failed to create rpc_subaddresses table: {e:#}");
    }
    run_rpc_server(coin, port, notify_tx_url, notify_block_url).await;
    Ok(())
}
