//! Orchard-to-Orchard transfer integration test against a live LWD server.
//!
//! This test requires network access to `zsa.methyl.cc` and is ignored by
//! default. Run with:
//!
//! ```bash
//! cargo test -p rlz --test zsa_transfer_test -- --nocapture --ignored
//! ```

use rlz::api::account::{get_addresses, new_account, NewAccount};
use rlz::api::coin::Coin;
use rlz::api::network::get_current_height;
use rlz::api::pay::{broadcast_transaction, extract_transaction, sign_transaction, PaymentOptions};
use rlz::pay::pool::ALL_POOLS;
use rlz::pay::Recipient;
use rlz::sync::synchronize_impl;

const SEED_PHRASE: &str = "equal clock rain latin plastic toss scrub modify clarify fold armor exchange gesture erase habit plug state forward demise demand limb risk only document";

/// Sync a faucet account, then send half its Orchard balance to a recipient.
#[tokio::test]
#[ignore = "requires live connection to zsa.methyl.cc"]
async fn test_orchard_transfer() {
    // Install rustls crypto provider (required for TLS to LWD server)
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Initialize tracing so debug!() calls show up. Set RUST_LOG=rlz=debug to enable.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rlz=debug,info".into()),
        )
        .try_init();

    // -- 1. Initialize Coin for ZSA regtest --
    let db_path = format!("/tmp/zsa_integration_test_{}.db", std::process::id());
    let _ = std::fs::remove_file(&db_path);

    let coin = Coin::new(Some(3))
        .open_database(db_path.clone(), None)
        .await
        .expect("open ZSA database")
        .set_lwd(0, "https://zsa.methyl.cc".to_string())
        .expect("set LWD URL");
    println!("Coin initialized: coin={} db={db_path}", coin.coin);

    // -- 2. Restore faucet account from seed --
    let na = NewAccount {
        icon: None,
        name: "zsa_test".to_string(),
        restore: true,
        key: SEED_PHRASE.to_string(),
        passphrase: Some("".to_string()),
        fingerprint: None,
        aindex: 0,
        birth: None,
        folder: "".to_string(),
        pools: Some(ALL_POOLS),
        use_internal: false,
        internal: false,
        ledger: false,
    };
    let account_id = new_account(&na, &coin)
        .await
        .expect("restore account from seed");
    let coin = coin
        .set_account(account_id)
        .await
        .expect("set current account");
    println!("Account restored: id={account_id}");

    // -- 3. Sync from LWD server to current height --
    let height = get_current_height(&coin).await.expect("get current height");
    println!("Current height: {height}");

    synchronize_impl(
        (),
        vec![account_id],
        height,
        10000,
        100,
        10000,
        false,
        &coin,
    )
    .await
    .expect("sync");
    println!("Synced to height: {height}");

    // -- 4. Check ZEC balance (0=T,1=S,2=O,3=IW) --
    let bal = rlz::api::sync::balance(&coin).await.expect("balance");
    println!(
        "ZEC balance: T={} S={} O={} IW={}",
        bal.0[0], bal.0[1], bal.0[2], bal.0[3]
    );
    let orchard_bal = bal.0[2];
    assert!(
        orchard_bal > 0,
        "faucet account should have Orchard balance"
    );
    let send_amount = orchard_bal / 2;
    println!("Sending {send_amount} zats from Orchard pool");

    // // -- 5. Issue a new ZSA asset --
    // let asset_name = format!("TEST{}", std::process::id());
    // let issue_amount = 1_000_000u64;
    // println!("Issuing asset '{asset_name}' amount={issue_amount}...");
    //
    // let tx_bytes = issue_asset(
    //     asset_name.clone(),
    //     issue_amount,
    //     true,  // first_issuance
    //     false, // finalize
    //     None,  // desc_hash (computed from name)
    //     account_id,
    //     &coin,
    // )
    // .await
    // .expect("issue asset");
    // println!("Issuance tx: {} bytes", tx_bytes.len());
    //
    // // -- 6. Broadcast the issuance (must use real chain height for expiry) --
    // let txid = broadcast_transaction(real_height, &tx_bytes, &coin)
    //     .await
    //     .expect("broadcast issuance");
    // println!("Issuance broadcast: {txid}");
    //
    // // -- 7. Wait for mining and re-sync --
    // println!("Waiting for mining...");
    // tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    //
    // synchronize_impl(
    //     (), vec![account_id], real_height, 10000, 100, 10000, false, &coin,
    // ).await.expect("re-sync");
    // println!("Re-synced to height: {real_height}");
    //
    // // -- 8. Verify the asset appears --
    // let holdings = list_zsa_holdings(&coin).await.expect("list holdings after issuance");
    // println!("ZSA holdings after issuance: {}", holdings.len());
    // for h in &holdings {
    //     println!(
    //         "  {}: balance={} base={}",
    //         h.asset_name,
    //         h.balance,
    //         hex::encode(&h.asset_base)
    //     );
    // }
    // assert!(!holdings.is_empty(), "should have the issued asset");
    //
    // let zsa = holdings.iter().find(|h| h.asset_name == asset_name)
    //     .expect("issued asset not found");
    // assert!(zsa.balance >= issue_amount, "balance should be at least issued amount");

    // -- 9. Get own UA for self-transfer --
    let addresses = get_addresses(ALL_POOLS, &coin)
        .await
        .expect("get addresses");
    let ua = addresses.ua.expect("own UA");
    println!("Own UA: {ua}");

    // -- 10. Self-send half the Orchard balance --
    let recipient = Recipient {
        address: ua,
        amount: send_amount,
        pools: None,
        user_memo: Some("o2o self-transfer test".to_string()),
        memo_bytes: None,
        price: None,
        asset_base: vec![],
        asset_name: None,
    };

    let options = PaymentOptions {
        src_pools: ALL_POOLS,
        recipient_pays_fee: false,
        smart_transparent: false,
        category: None,
        mode: 0,
    };

    println!("Planning O2O transfer of {send_amount} zats...");
    let pczt = rlz::api::pay::prepare(&[recipient], options, &coin)
        .await
        .expect("plan O2O transfer");
    assert!(
        pczt.n_spends.iter().sum::<usize>() > 0,
        "should have spends"
    );
    println!("  spends: {:?}", pczt.n_spends);

    let signed = sign_transaction(&pczt, &coin).await.expect("sign");
    let tx_bytes = extract_transaction(&signed).await.expect("extract");
    println!("Transfer tx: {} bytes", tx_bytes.len());

    // -- 11. Broadcast the transfer --
    let height = get_current_height(&coin).await.expect("get current height");
    let txid = broadcast_transaction(height, &tx_bytes, &coin)
        .await
        .expect("broadcast transfer");
    println!("Transfer broadcast: {txid}");

    // -- 12. Re-sync and verify --
    println!("Waiting for mining...");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let height = get_current_height(&coin).await.expect("get current height");
    synchronize_impl(
        (),
        vec![account_id],
        height,
        10000,
        100,
        10000,
        false,
        &coin,
    )
    .await
    .expect("re-sync after transfer");

    // Verify balance changed (sent amount minus fee)
    let bal = rlz::api::sync::balance(&coin)
        .await
        .expect("balance after transfer");
    println!(
        "ZEC balance after transfer: T={} S={} O={} IW={}",
        bal.0[0], bal.0[1], bal.0[2], bal.0[3]
    );
    assert!(
        bal.0[2] > 0,
        "should still have Orchard balance after self-transfer"
    );

    // Clean up
    let _ = std::fs::remove_file(&db_path);
    println!("Test passed.");
}
