use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use figment::providers::{Format, Serialized, Toml};
use figment::Figment;
use juniper::RootNode;
use juniper_graphql_ws::ConnectionConfig;
use rlz::api::coin::Coin;
use rlz::graphql::mutation::run_mempool;
use rlz::graphql::{mutation::Mutation, query::Query, subs::Subscription, Context};
use serde::{Deserialize, Serialize};
use warp::Filter;

type Schema = RootNode<Query, Mutation, Subscription>;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .unwrap();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .compact()
        .finish();
    let c = Config::parse();
    let config_path = c.config_path.clone().unwrap_or("zkool.toml".to_string());
    let _ = tracing::subscriber::set_global_default(subscriber);
    let config: Config = Figment::new()
        .merge(Serialized::defaults(c))
        .merge(Toml::file(&config_path))
        .extract()?;
    let Config {
        db_path,
        lwd_url,
        port,
        ..
    } = config;
    let db_path = db_path.unwrap_or("zkool.db".to_string());
    let lwd_url = lwd_url.unwrap_or("https://zec.rocks".to_string());
    let port = port.unwrap_or(8000);

    tracing::info!("db_path {db_path} lwd_url {lwd_url} port {port}");
    let coin = Coin::new()
        .open_database(db_path, None)
        .await?
        .set_lwd(0, lwd_url)?;

    let context = Context::new(coin);
    tokio::spawn(run_mempool(context.clone()));

    let schema = Schema::new(Query {}, Mutation {}, Subscription {});

    let c = context.clone();
    let context_extractor = warp::any().map(move || context.clone());

    let schema = Arc::new(schema);

    let routes = (warp::post()
        .and(warp::path("graphql"))
        .and(juniper_warp::make_graphql_filter(
            schema.clone(),
            context_extractor,
        )))
    .or(
        warp::path("subscriptions").and(juniper_warp::subscriptions::make_ws_filter(
            schema,
            ConnectionConfig::new(c),
        )),
    )
    .or(warp::get()
        .and(warp::path("graphiql"))
        .and(juniper_warp::graphiql_filter(
            "/graphql",
            Some("/subscriptions"),
        )));

    tracing::info!("Listening on 127.0.0.1:{port}");
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;

    Ok(())
}
