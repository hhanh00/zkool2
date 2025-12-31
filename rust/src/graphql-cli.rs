use anyhow::Result;
use juniper::{EmptyMutation, EmptySubscription, RootNode};
use rlz::graphql::{query::Query, Context};
use rocket::{Config, State, routes, response::content::RawHtml};

type Schema = RootNode<Query, EmptyMutation<Context>, EmptySubscription<Context>>;

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;

#[rocket::get("/graphiql")]
fn graphiql() -> RawHtml<String> {
    juniper_rocket::graphiql_source("/graphql", None)
}

#[rocket::get("/playground")]
fn playground() -> RawHtml<String> {
    juniper_rocket::playground_source("/graphql", None)
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
    let config = Config::figment();
    let db_path: String = config.extract_inner("custom.db_path")?;
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;
    let context = Context::new(pool);

    rocket::build()
        .manage(context)
        .manage(Schema::new(
            Query {},
            EmptyMutation::new(),
            EmptySubscription::new(),
        ))
        .mount(
            "/",
            routes![graphiql, playground, get_graphql, post_graphql],
        )
        .launch()
        .await
        .expect("server to launch");
    Ok(())
}
