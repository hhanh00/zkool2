use juniper::GraphQLObject;

#[derive(GraphQLObject)]
#[graphql(description = "A Zcash wallet account")]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub seed: Option<String>,
    pub passphrase: Option<String>,
    pub aindex: i32,
    pub dindex: i32,
    pub birth: i32,
}
