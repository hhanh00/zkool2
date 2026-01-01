use anyhow::Result;
use juniper::{EmptySubscription, RootNode};
use rlz::api::coin::Coin;
use rlz::graphql::{query::Query, mutation::Mutation, Context};
use rocket::{Config, State, routes, response::content::RawHtml};

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
    let config = Config::figment();
    let db_path: String = config.extract_inner("custom.db_path")?;
    let coin = Coin::new();
    let coin = coin.open_database(db_path, None).await?;
    let context = Context::new(coin);

    rocket::build()
        .manage(context)
        .manage(Schema::new(
            Query {},
            Mutation {},
            EmptySubscription::new(),
        ))
        .mount(
            "/",
            routes![graphiql, get_graphql, post_graphql],
        )
        .launch()
        .await
        .expect("server to launch");
    Ok(())
}
