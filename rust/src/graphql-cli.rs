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

/// Validate JWT token and return claims, or AuthError if invalid
fn validate_jwt(token: &str, decoding_key: &DecodingKey) -> Result<Claims, AuthError> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::ES256);
    validation.validate_exp = true;
    jsonwebtoken::decode::<Claims>(token, decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|_| AuthError)
}

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
    #[clap(short, long, value_parser, default_missing_value = "true", num_args = 0..=1, require_equals = false)]
    pub no_mempool: Option<bool>,
    // Note: Once set in a config file, jwt_public_key_file
    // cannot be unset by a later config source because
    // None means skip
    #[clap(short, long, value_parser)]
    pub jwt_public_key_file: Option<String>,
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
        jwt_public_key_file,
        no_mempool,
        ..
    } = config;
    let db_path = db_path.unwrap_or("zkool.db".to_string());
    let lwd_url = lwd_url.unwrap_or("https://zec.rocks".to_string());
    let port = port.unwrap_or(8000);
    let no_mempool = no_mempool.unwrap_or_default();

    let decoding_key = jwt_public_key_file
        .map(|path| {
            let pem = std::fs::read_to_string(&path)?;
            Ok::<_, anyhow::Error>(DecodingKey::from_ec_pem(pem.as_bytes())?)
        })
        .transpose()?;
    if decoding_key.is_none() {
        tracing::warn!("Server is running WITHOUT authentication. Everyone has full access.");
    }
    let decoding_key = Arc::new(decoding_key);

    // Note: To generate a pk/sk pair
    // sk: openssl ecparam -name prime256v1 -genkey -noout -out private.pem
    // pk: openssl ec -in private.pem -pubout -out public.pem
    // convert key format: openssl pkcs8 -topk8 -nocrypt -in private.pem -out private_p8.pem
    // issue jwt: jwt encode --secret @private_p8.pem --alg ES256 --exp=<epoch secs> --sub=<account id> '{"write": true}'

    tracing::info!("db_path {db_path} lwd_url {lwd_url} port {port}");
    let coin = Coin::new()
        .open_database(db_path, None)
        .await?
        .set_lwd(0, lwd_url)?;

    let context = Context::new(coin);
    if !no_mempool {
        tokio::spawn(run_mempool(context.clone()));
    }

    let schema = Schema::new(Query {}, Mutation {}, Subscription {});

    let ctx = context.clone();
    let dk = Arc::clone(&decoding_key); // For HTTP
    let context_extractor = warp::header::optional::<String>("authorization").and_then(
        move |auth_header: Option<String>| {
            let decoding_key = Arc::clone(&dk);
            let base_ctx = ctx.clone();
            async move {
                let token = auth_header
                    .and_then(|h| h.strip_prefix("Bearer ").map(str::trim).map(String::from));
                let ctx = match (&*decoding_key, token) {
                    (Some(key), Some(t)) => Context {
                        auth: Some(validate_jwt(&t, key).map_err(warp::reject::custom)?),
                        ..base_ctx
                    },
                    (Some(_), None) => return Err(warp::reject::custom(AuthError)),
                    (None, _) => base_ctx,
                };
                Ok::<_, warp::reject::Rejection>(ctx)
            }
        },
    );

    let schema = Arc::new(schema);

    let routes = (warp::post()
        .and(warp::path("graphql"))
        .and(juniper_warp::make_graphql_filter(
            schema.clone(),
            context_extractor.clone(),
        )))
    .or(
        warp::path("subscriptions").and(juniper_warp::subscriptions::make_ws_filter(
            schema,
            move |variables: juniper::Variables| {
                let base_ctx = context.clone();
                let decoding_key = Arc::clone(&decoding_key);
                async move {
                    let auth_token = variables.get("authToken").and_then(|v| v.convert::<String>().ok());

                    let ctx = match (&*decoding_key, auth_token) {
                        (Some(key), Some(token)) => Context {
                            auth: Some(validate_jwt(&token, key)?),
                            ..base_ctx
                        },
                        (Some(_), None) => return Err(AuthError),
                        (None, _) => base_ctx,
                    };

                    Ok::<_, AuthError>(ConnectionConfig::new(ctx))
                }
            },
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
