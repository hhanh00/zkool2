use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use figment::providers::{Format, Serialized, Toml};
use figment::Figment;
use jsonwebtoken::{DecodingKey, Validation};
use juniper::RootNode;
use juniper_graphql_ws::ConnectionConfig;
use rlz::api::coin::Coin;
use rlz::graphql::jwt::{AuthError, Claims};
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
    #[clap(short, long, value_parser)]
    pub jwt_secret_file: Option<String>,
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
        .merge(Toml::file(&config_path))
        .merge(Serialized::defaults(c))
        .extract()?;
    let Config {
        db_path,
        lwd_url,
        port,
        jwt_secret_file,
        ..
    } = config;
    let db_path = db_path.unwrap_or("zkool.db".to_string());
    let lwd_url = lwd_url.unwrap_or("https://zec.rocks".to_string());
    let port = port.unwrap_or(8000);

    let secret = jwt_secret_file
        .map(|path| {
            let s = std::fs::read_to_string(&path)?.trim().to_string();
            Ok::<_, anyhow::Error>(s)
        })
        .transpose()?;

    tracing::info!("db_path {db_path} lwd_url {lwd_url} port {port}");
    let coin = Coin::new()
        .open_database(db_path, None)
        .await?
        .set_lwd(0, lwd_url)?;

    let context = Context::new(coin);
    tokio::spawn(run_mempool(context.clone()));

    let schema = Schema::new(Query {}, Mutation {}, Subscription {});

    let c = context.clone();
    let c2 = c.clone();
    let context_extractor = match secret {
        Some(secret) => {
            let filter = warp::header::optional::<String>("authorization").and_then(
                move |auth_header: Option<String>| {
                    let secret = secret.clone();
                    let c = c2.clone();
                    async move {
                        let token = match auth_header {
                            Some(h) if h.starts_with("Bearer ") => {
                                h.trim_start_matches("Bearer ").trim().to_string()
                            }
                            _ => return Err(warp::reject::custom(AuthError)), // missing or malformed
                        };

                        let token_data = jsonwebtoken::decode::<Claims>(
                            token,
                            &DecodingKey::from_secret(secret.as_bytes()),
                            &Validation::default(),
                        )
                        .map_err(|_| warp::reject::custom(AuthError))?;
                        let auth_context = Context {
                            auth: Some(token_data.claims),
                            ..c
                        };

                        Ok::<_, warp::reject::Rejection>(auth_context)
                    }
                },
            );
            filter.boxed()
        }
        None => {
            let filter = warp::any().map(move || c2.clone());
            filter.boxed()
        }
    };

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

    tracing::info!("Listening on 0.0.0.0:{port}");
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;

    Ok(())
}
