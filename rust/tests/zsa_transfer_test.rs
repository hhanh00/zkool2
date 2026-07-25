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
const RECIPIENT_SEED: &str = "recall chat clerk swallow clap grant asset acoustic media brave front edit rail front silly cousin wolf cliff leopard dizzy element number risk episode";

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
    let sender_id = new_account(&na, &coin)
        .await
        .expect("restore sender account from seed");
    let sender = coin
        .clone()
        .set_account(sender_id)
        .await
        .expect("set sender account");
    println!("Sender account restored: id={sender_id}");

    // -- 3. Sync sender from LWD server to current height --
    let height = get_current_height(&sender).await.expect("get current height");
    println!("Current height: {height}");

    synchronize_impl(
        (), vec![sender_id], height, 10000, 100, 10000, false, &sender,
    ).await.expect("sync sender");
    println!("Sender synced to height: {height}");

    // -- 4. Check ZEC balance (0=T,1=S,2=O,3=IW) --
    let bal = rlz::api::sync::balance(&sender).await.expect("sender balance");
    println!("ZEC balance: T={} S={} O={} IW={}", bal.0[0], bal.0[1], bal.0[2], bal.0[3]);
    let orchard_bal = bal.0[2];
    assert!(orchard_bal > 0, "sender should have Orchard balance");
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

    // -- 9. Restore recipient account and get its address --
    let na2 = NewAccount {
        icon: None,
        name: "zsa_recipient".to_string(),
        restore: true,
        key: RECIPIENT_SEED.to_string(),
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
    let recipient_id = new_account(&na2, &coin).await.expect("restore recipient");
    println!("Recipient account restored: id={recipient_id}");

    let recipient = coin
        .clone()
        .set_account(recipient_id)
        .await
        .expect("set recipient account");

    // Sync recipient account (just needs the UA, no notes needed)
    let height = get_current_height(&recipient).await.expect("get current height");
    synchronize_impl(
        (), vec![recipient_id], height, 10000, 100, 10000, false, &recipient,
    ).await.expect("sync recipient");
    println!("Recipient synced");

    let recipient_addresses = get_addresses(ALL_POOLS, &recipient).await.expect("get recipient addresses");
    let recipient_ua = recipient_addresses.ua.expect("recipient UA");
    println!("Recipient UA: {recipient_ua}");

    // -- 10. Send half the Orchard balance to the recipient --
    let pay_recipient = Recipient {
        address: recipient_ua,
        amount: send_amount,
        pools: None,
        user_memo: Some("o2o transfer test".to_string()),
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
    let pczt = rlz::api::pay::prepare(&[pay_recipient], options, &sender)
        .await
        .expect("plan O2O transfer");
    assert!(pczt.n_spends.iter().sum::<usize>() > 0, "should have spends");
    println!("  spends: {:?}", pczt.n_spends);

    let signed = sign_transaction(&pczt, &sender).await.expect("sign");
    std::fs::write("/tmp/zsa_postsigned.pczt", &signed.pczt).expect("save postsigned pczt");
    let tx_bytes = extract_transaction(&signed).await.expect("extract");
    std::fs::write("/tmp/zsa_tx.bin", &tx_bytes).expect("save tx bytes");
    println!("Transfer tx: {} bytes (saved /tmp/zsa_tx.bin)", tx_bytes.len());

    // -- 11. Broadcast the transfer --
    let height = get_current_height(&sender).await.expect("get current height");
    let txid = broadcast_transaction(height, &tx_bytes, &sender)
        .await
        .expect("broadcast transfer");
    println!("Transfer broadcast: {txid}");

    // -- 12. Wait for at least 1 block to be mined --
    println!("Waiting for mining...");
    let start_height = get_current_height(&sender).await.expect("get current height");
    let mut attempts = 0;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let current = get_current_height(&sender).await.expect("get current height");
        attempts += 1;
        if current > start_height {
            println!("New block mined: {start_height} -> {current} (after {attempts} attempts)");
            break;
        }
        if attempts % 5 == 0 {
            println!("Still waiting for block after {attempts} attempts (height={current})");
        }
    }

    let height = get_current_height(&sender).await.expect("get current height");
    synchronize_impl(
        (), vec![sender_id, recipient_id], height, 10000, 100, 10000, false, &coin,
    ).await.expect("re-sync after transfer");

    // Verify sender balance decreased
    let bal = rlz::api::sync::balance(&sender).await.expect("sender balance");
    println!("Sender ZEC balance after transfer: T={} S={} O={} IW={}", bal.0[0], bal.0[1], bal.0[2], bal.0[3]);
    assert!(bal.0[2] < orchard_bal, "sender Orchard balance should have decreased");

    // Switch to recipient and verify receipt
    let recv_bal = rlz::api::sync::balance(&recipient).await.expect("recipient balance");
    println!("Recipient ZEC balance: T={} S={} O={} IW={}", recv_bal.0[0], recv_bal.0[1], recv_bal.0[2], recv_bal.0[3]);
    assert!(recv_bal.0[2] >= send_amount, "recipient should have received the ZEC");

    // Clean up
    let _ = std::fs::remove_file(&db_path);
    println!("Test passed.");
}
