use anyhow::Result;
use figment::providers::{Format, Toml};
use figment::Figment;
use juniper::{EmptySubscription, RootNode};
use rlz::api::coin::Coin;
use rlz::graphql::mutation::run_mempool;
use rlz::graphql::{mutation::Mutation, query::Query, Context};

type Schema = RootNode<Query, Mutation, EmptySubscription<Context>>;

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .unwrap();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .compact()
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
    let config = Figment::new().merge(Toml::file("zkool.toml"));
    let db_path: String = config.extract_inner("db_path")?;
    let lwd_url: String = config.extract_inner("lwd_url")?;
    let polling_interval: u32 = config.extract_inner("polling_interval")?;
    tracing::info!("db_path {db_path} lwd_url {lwd_url} polling_interval {polling_interval}");
    let coin = Coin::new()
        .open_database(db_path, None)
        .await?
        .set_lwd(0, lwd_url)?;

    let context = Context::new(coin);

    tokio::spawn(run_mempool(context.clone()));

    Ok(())
}
