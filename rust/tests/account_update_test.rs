//! Account settings update regression tests.

use rlz::api::account::{list_accounts, new_account, update_account, AccountUpdate, NewAccount};
use rlz::api::coin::Coin;

const SEED_PHRASE: &str = "equal clock rain latin plastic toss scrub modify clarify fold armor exchange gesture erase habit plug state forward demise demand limb risk only document";

#[tokio::test]
async fn changing_internal_change_persists_and_materializes_addresses() {
    let db_path = format!("/tmp/account_update_test_{}.db", std::process::id());
    let _ = std::fs::remove_file(&db_path);
    let coin = Coin::new(Some(3))
        .open_database(db_path.clone(), None)
        .await
        .unwrap();
    let account = NewAccount {
        icon: None,
        name: "software".to_string(),
        restore: true,
        key: SEED_PHRASE.to_string(),
        passphrase: Some(String::new()),
        fingerprint: None,
        aindex: 0,
        birth: None,
        folder: String::new(),
        pools: Some(3),
        use_internal: false,
        internal: false,
        hw: 0,
    };
    let id = new_account(&account, &coin)
        .await
        .unwrap();
    for use_internal in [true, false] {
        update_account(
            &AccountUpdate {
                coin: coin.coin,
                id,
                name: None,
                icon: None,
                birth: None,
                folder: 0,
                hidden: None,
                enabled: None,
                use_internal: Some(use_internal),
            },
            &coin,
        )
        .await
        .unwrap();
        let account = list_accounts(&coin)
            .await
            .unwrap()
            .into_iter()
            .find(|a| a.id == id)
            .unwrap();
        assert_eq!(account.use_internal, use_internal);
        let mut connection = <sqlx::SqliteConnection as sqlx::Connection>::connect(
            &format!("sqlite://{db_path}"),
        )
        .await
        .unwrap();
        let address = rlz::account::transparent_change_address(
            &mut connection,
            id,
            use_internal,
            account.dindex,
        )
        .await;
        assert!(
            address.is_ok(),
            "change address must be available immediately: {address:?}"
        );
    }
}

