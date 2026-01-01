use anyhow::Result;
use juniper::{EmptySubscription, RootNode};
use rlz::api::coin::Coin;
use rlz::graphql::{mutation::Mutation, query::Query, Context};
use rocket::{response::content::RawHtml, routes, Config, State};

type Schema = RootNode<Query, Mutation, EmptySubscription<Context>>;

#[rocket::get("/graphiql")]
fn graphiql() -> RawHtml<String> {
    juniper_rocket::graphiql_source("/graphql", None)
}

#[rocket::get("/graphql?<request..>")]
async fn get_graphql(
    db: &State<Context>,
    request: juniper_rocket::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(schema, db).await
}

#[rocket::post("/graphql", data = "<request>")]
async fn post_graphql(
    db: &State<Context>,
    request: juniper_rocket::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(schema, db).await
}

#[rocket::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider().install_default().unwrap();
    let config = Config::figment();
    let db_path: String = config.extract_inner("custom.db_path")?;
    let lwd_url: String = config.extract_inner("custom.lwd_url")?;
    let coin = Coin::new()
        .open_database(db_path, None)
        .await?
        .set_lwd(0, lwd_url)?;
    let context = Context::new(coin);

    rocket::build()
        .manage(context)
        .manage(Schema::new(Query {}, Mutation {}, EmptySubscription::new()))
        .mount("/", routes![graphiql, get_graphql, post_graphql])
        .launch()
        .await
        .expect("server to launch");
    Ok(())
}
