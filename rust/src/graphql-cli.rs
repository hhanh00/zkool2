use std::sync::Arc;

use anyhow::Result;
use figment::providers::{Format, Toml};
use figment::Figment;
use juniper::RootNode;
use juniper_graphql_ws::ConnectionConfig;
use rlz::api::coin::Coin;
use rlz::graphql::mutation::run_mempool;
use rlz::graphql::{mutation::Mutation, query::Query, subs::Subscription, Context};
use warp::Filter;
use warp::http::Response;

type Schema = RootNode<Query, Mutation, Subscription>;

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
    let port: u16 = config.extract_inner("port").unwrap_or(8000);
    tracing::info!("db_path {db_path} lwd_url {lwd_url} polling_interval {polling_interval}");
    let coin = Coin::new()
        .open_database(db_path, None)
        .await?
        .set_lwd(0, lwd_url)?;

    let context = Context::new(coin);
    tokio::spawn(run_mempool(context.clone()));

    let schema = Schema::new(Query {}, Mutation {}, Subscription {});

    let c = context.clone();
    let context_extractor = warp::any()
    .map(move || context.clone());

    let homepage = warp::path::end().map(|| {
        Response::builder()
            .header("content-type", "text/html")
            .body(
                "<html><h1>juniper_warp/subscription example</h1>\
                       <div>visit <a href=\"/graphiql\">GraphiQL</a></div>\
                       <div>visit <a href=\"/playground\">GraphQL Playground</a></div>\
                 </html>",
            )
    });
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
        )))
    .or(homepage);

    tracing::info!("Listening on 127.0.0.1:{port}");
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;

    Ok(())
}
