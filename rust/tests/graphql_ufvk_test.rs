#![cfg(feature = "graphql")]

use juniper::{EmptySubscription, RootNode, Variables};
use rlz::{
    api::{
        account::{new_account, NewAccount},
        coin::Coin,
    },
    graphql::{jwt::Claims, mutation::Mutation, query::Query, Context},
};

async fn query(context: &Context, selection: &str) -> Result<String, String> {
    let schema = RootNode::new(Query {}, Mutation {}, EmptySubscription::<Context>::new());
    let (value, errors) = juniper::execute(
        &format!("{{ unifiedFullViewingKey({selection}) }}"),
        None,
        &schema,
        &Variables::new(),
        context,
    )
    .await
    .unwrap();
    if let Some(error) = errors.first() {
        return Err(error.error().message().to_owned());
    }
    Ok(
        serde_json::to_value(value).unwrap()["unifiedFullViewingKey"]
            .as_str()
            .unwrap()
            .to_owned(),
    )
}

#[tokio::test]
async fn viewing_key_pools_and_authorization() {
    let path = format!("/tmp/graphql_ufvk_{}.db", uuid::Uuid::new_v4());
    let coin = Coin::new(Some(3))
        .open_database(path.clone(), None)
        .await
        .unwrap();
    let mut account = NewAccount {
        icon: None, name: "software".into(), restore: true,
        key: "equal clock rain latin plastic toss scrub modify clarify fold armor exchange gesture erase habit plug state forward demise demand limb risk only document".into(),
        passphrase: None, fingerprint: None, aindex: 0, birth: None,
        folder: String::new(), pools: Some(7), use_internal: false, internal: false, hw: 0,
    };
    let id = new_account(&account, &coin).await.unwrap();
    let mut context = Context::new(coin.clone());
    context.auth = Some(Claims {
        exp: usize::MAX,
        sub: id,
        write: false,
    });
    for pools in 1..=15 {
        let key_pools = if pools & 8 != 0 {
            (pools & !8) | 4
        } else {
            pools
        };
        let expected = rlz::api::account::get_account_ufvk(id, key_pools, &coin)
            .await
            .unwrap();
        assert_eq!(
            query(&context, &format!("idAccount: {id}, pools: {pools}"))
                .await
                .unwrap(),
            expected
        );
    }
    assert_eq!(
        query(&context, &format!("idAccount: {id}")).await.unwrap(),
        rlz::api::account::get_account_ufvk(id, 7, &coin)
            .await
            .unwrap()
    );
    for pools in [-1, 0, 16, 257] {
        assert!(query(&context, &format!("idAccount: {id}, pools: {pools}"))
            .await
            .unwrap_err()
            .contains("Invalid pool mask"));
    }
    context.auth.as_mut().unwrap().sub = id + 1;
    assert_eq!(
        query(&context, &format!("idAccount: {id}, pools: 7"))
            .await
            .unwrap_err(),
        "Unauthorized"
    );
    context.auth.as_mut().unwrap().sub = 0;
    assert!(query(&context, &format!("idAccount: {id}, pools: 7"))
        .await
        .is_ok());
    // An account missing a pool cannot silently export a subset of the selection.
    account.name = "without sapling".into();
    account.pools = Some(5);
    let limited_id = new_account(&account, &coin).await.unwrap();
    assert!(
        query(&context, &format!("idAccount: {limited_id}, pools: 7"))
            .await
            .unwrap_err()
            .contains("unavailable")
    );
    drop(context);
    drop(coin);
    let _ = std::fs::remove_file(path);
}
