//! Ledger account creation tests.
//!
//! No device or emulator is needed: the Official app derives everything
//! locally from the seed (standard ZIP-32), and the Zondax restore path
//! replicates the device derivation in software (`recover::recover_ledger_seed`).

use rlz::api::account::{get_account_pools, list_accounts, new_account, NewAccount};
use rlz::api::coin::Coin;
use rlz::ledger::HwKind;

const SEED_PHRASE: &str = "equal clock rain latin plastic toss scrub modify clarify fold armor exchange gesture erase habit plug state forward demise demand limb risk only document";

async fn temp_coin(name: &str) -> Coin {
    let db_path = format!("/tmp/ledger_accounts_test_{}_{name}.db", std::process::id());
    let _ = std::fs::remove_file(&db_path);
    Coin::new(Some(3))
        .open_database(db_path, None)
        .await
        .expect("open database")
}

fn na(name: &str, key: &str, hw: u8, pools: Option<u8>) -> NewAccount {
    NewAccount {
        icon: None,
        name: name.to_string(),
        restore: !key.is_empty(),
        key: key.to_string(),
        passphrase: Some("".to_string()),
        fingerprint: None,
        aindex: 0,
        birth: None,
        folder: "".to_string(),
        pools,
        use_internal: false,
        internal: false,
        hw,
    }
}

async fn created_hw(coin: &Coin, name: &str) -> u8 {
    list_accounts(coin)
        .await
        .unwrap()
        .into_iter()
        .find(|a| a.name == name)
        .unwrap()
        .hw
}

#[tokio::test]
async fn official_ledger_with_seed_is_created_locally() {
    let coin = temp_coin("official_seed").await;
    let id = new_account(&na("official", SEED_PHRASE, HwKind::Official as u8, Some(9)), &coin)
        .await
        .expect("create Official Ledger account from seed");

    assert_eq!(created_hw(&coin, "official").await, HwKind::Official as u8);
    let pools = get_account_pools(id, &coin).await.unwrap();
    assert_ne!(pools & 8, 0, "ironwood must be present");
    assert_eq!(pools & 2, 0, "sapling must be absent");
}

#[tokio::test]
async fn official_ledger_without_seed_is_rejected() {
    let coin = temp_coin("official_noseed").await;
    let r = new_account(&na("official", "", HwKind::Official as u8, Some(9)), &coin).await;
    let err = r.err().expect("must be rejected without a seed");
    assert!(
        err.to_string().contains("seed"),
        "unexpected error: {err:#}"
    );
}

#[tokio::test]
async fn zondax_restore_with_seed_stays_software() {
    let coin = temp_coin("zondax_seed").await;
    let id = new_account(&na("zondax", SEED_PHRASE, HwKind::Zondax as u8, Some(3)), &coin)
        .await
        .expect("create Zondax Ledger account from seed");

    // Existing behavior: a Zondax account restored from seed keeps the
    // Zondax-derived sapling sk in software and is NOT flagged as hardware.
    // Only keyless (device-only) Zondax accounts are flagged hw=1.
    assert_eq!(created_hw(&coin, "zondax").await, HwKind::Software as u8);
    let pools = get_account_pools(id, &coin).await.unwrap();
    assert_ne!(pools & 2, 0, "sapling must be present");
    assert_eq!(pools & 4, 0, "orchard must be absent");
}

#[tokio::test]
async fn zondax_ledger_rejects_orchard_pools() {
    let coin = temp_coin("zondax_orchard").await;
    let r = new_account(&na("zondax", SEED_PHRASE, HwKind::Zondax as u8, Some(7)), &coin).await;
    let err = r.err().expect("orchard pools must be rejected for Zondax");
    assert!(
        err.to_string().contains("transparent and sapling"),
        "unexpected error: {err:#}"
    );
}

#[tokio::test]
async fn official_ledger_rejects_sapling_pools() {
    let coin = temp_coin("official_sapling").await;
    let r = new_account(&na("official", SEED_PHRASE, HwKind::Official as u8, Some(7)), &coin).await;
    let err = r.err().expect("sapling pools must be rejected for Official");
    assert!(
        err.to_string().contains("transparent and ironwood"),
        "unexpected error: {err:#}"
    );
}
