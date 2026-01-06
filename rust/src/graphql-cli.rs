use anyhow::Result;
use figment::Figment;
use figment::providers::{Format, Toml};
use juniper::{EmptySubscription, RootNode};
use rlz::api::coin::Coin;
use rlz::graphql::mutation::run_mempool;
use rlz::graphql::{mutation::Mutation, query::Query, Context};

type Schema = RootNode<Query, Mutation, EmptySubscription<Context>>;

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider().install_default().unwrap();
    let config = Figment::new().merge(Toml::file("zkool.toml"));
    let db_path: String = config.extract_inner("custom.db_path")?;
    let lwd_url: String = config.extract_inner("custom.lwd_url")?;
    let coin = Coin::new()
        .open_database(db_path, None)
        .await?
        .set_lwd(0, lwd_url)?;
    let context = Context::new(coin);
    tokio::spawn(run_mempool(context.clone()));

    Ok(())
}
