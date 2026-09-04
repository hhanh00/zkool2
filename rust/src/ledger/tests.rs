use crate::ledger::transport::{APDUCommand, Device, LEDGER_ZEMU};

use super::*;

/// Speculos REST API port for the device screen/buttons (see run-emulator.sh).
fn ui_port() -> u16 {
    std::env::var("ZEMU_UI_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(5000)
}

fn ui_post(path: &str, body: &str) {
    use std::io::{Read as _, Write as _};

    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let Ok(mut s) = std::net::TcpStream::connect(("127.0.0.1", ui_port())) else {
        return;
    };
    if s.write_all(req.as_bytes()).is_ok() {
        let _ = s.read_to_end(&mut vec![]);
    }
}

fn press_right() {
    ui_post(
        "/button/right",
        r#"{"action": "press-and-release"}"#,
    );
}

fn press_both() {
    ui_post("/button/both", r#"{"action": "press-and-release"}"#);
}

fn screen_text() -> String {
    use std::io::{Read as _, Write as _};

    let req = format!(
        "GET /events?currentscreenonly=true HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    let Ok(mut s) = std::net::TcpStream::connect(("127.0.0.1", ui_port())) else {
        return String::new();
    };
    if s.write_all(req.as_bytes()).is_err() {
        return String::new();
    }
    let mut buf = vec![];
    if s.read_to_end(&mut buf).is_err() {
        return String::new();
    }
    let body = String::from_utf8_lossy(&buf);
    let Ok(json) = serde_json::from_str::<serde_json::Value>(body.split("\r\n\r\n").nth(1).unwrap_or("")) else {
        return String::new();
    };
    json["events"]
        .as_array()
        .map(|events| {
            events
                .iter()
                .filter_map(|e| e["text"].as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default()
}

/// Drive the NBGL review: advance pages with the right button and confirm
/// the export with a both-button press on the Confirm choice. Loops until
/// the deadline so stale reviews from earlier failed runs are approved too.
fn spawn_approval_driver() {
    std::thread::spawn(|| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        while std::time::Instant::now() < deadline {
            let text = screen_text();
            if text.contains("Confirm") {
                eprintln!("driver: both (screen: {text:?})");
                press_both();
            } else {
                eprintln!("driver: right (screen: {text:?})");
                press_right();
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
}

/// Get the account UFVK from the Official Ledger app (GET_VK, CLA 0xE0) and
/// compare it against the same key derived locally from the emulator seed.
/// The expected value is therefore computed, not provided.
///
/// Run with the emulator up (seeded with EMULATOR_SEED):
/// `cargo test --features zemu -- --ignored --nocapture ledger_get_ufvk`
#[tokio::test]
#[ignore]
pub async fn ledger_get_ufvk() -> LedgerResult<()> {
    use std::str::FromStr as _;

    use zcash_address::unified::{Encoding as _, Fvk, Ufvk};
    use zcash_keys::keys::{UnifiedFullViewingKey, UnifiedSpendingKey};
    use zip32::AccountId;

    const EMULATOR_SEED: &str = "display accident enable raw glimpse engine know fog bubble price bunker minimum entry tuna joy motor rate tennis evolve october verb jelly indoor dance";

    let ledger = LEDGER_ZEMU.lock().await.clone().unwrap();
    let network = Network::Main;

    // expected UFVK (orchard + transparent receivers) from the emulator seed
    let mnemonic = bip39::Mnemonic::from_str(EMULATOR_SEED)
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("bad seed: {e}")))?;
    let seed = mnemonic.to_seed("");
    let usk = UnifiedSpendingKey::from_seed(&network, &seed, AccountId::try_from(0).unwrap())
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("usk derivation failed: {e}")))?;
    let uvk = usk.to_unified_full_viewing_key();
    let items = vec![
        Fvk::P2pkh(uvk.transparent().unwrap().serialize().try_into().unwrap()),
        Fvk::Orchard(uvk.orchard().unwrap().to_bytes()),
    ];
    let expected = UnifiedFullViewingKey::parse(
        &Ufvk::try_from_items(items).map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?,
    )
    .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?
    .encode(&network);

    spawn_approval_driver();

    let ufvk = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        official::get_ufvk(&ledger, &network, 0),
    )
    .await
    .map_err(|_| LedgerError::Protocol("timeout waiting for the device answer".into()))??;

    println!("device   : {ufvk}");
    println!("expected : {expected}");
    assert_eq!(ufvk, expected, "device UFVK does not match the seed-derived key");
    Ok(())
}

/// Create an Official Ledger account (hw=2, no seed phrase) against the
/// device and check that the stored keys match the device UFVK.
/// Needs the emulator up, seeded with EMULATOR_SEED (see ledger_get_ufvk):
/// `ZEMU_UI_PORT=5001 cargo test --features zemu -- --ignored --nocapture ledger_account_import`
#[tokio::test]
#[ignore]
pub async fn ledger_account_import() -> LedgerResult<()> {
    use std::str::FromStr as _;

    use sqlx::Connection as _;
    use zcash_address::unified::Encoding as _;
    use zcash_keys::keys::{UnifiedFullViewingKey, UnifiedSpendingKey};
    use zip32::AccountId;

    const EMULATOR_SEED: &str = "display accident enable raw glimpse engine know fog bubble price bunker minimum entry tuna joy motor rate tennis evolve october verb jelly indoor dance";

    let network = Network::Main;
    let mut db = std::env::temp_dir();
    db.push(format!("zkool_ol_import_{}.db", std::process::id()));
    let mut connection = sqlx::sqlite::SqliteConnection::connect_with(
        &sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&db)
            .create_if_missing(true),
    )
    .await
    .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?;
    crate::db::create_schema(&mut connection)
        .await
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?;

    let na = crate::api::account::NewAccount {
        icon: None,
        name: "ol".into(),
        restore: false,
        key: String::new(),
        passphrase: None,
        fingerprint: None,
        aindex: 0,
        birth: None,
        folder: String::new(),
        pools: Some(crate::pay::pool::POOL_TRANSPARENT | crate::pay::pool::POOL_IRONWOOD),
        use_internal: false,
        internal: false,
        hw: HwKind::Official as u8,
    };

    spawn_approval_driver();
    let account = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        crate::account::new_account(&network, &mut connection, &na),
    )
    .await
    .map_err(|_| LedgerError::Protocol("timeout waiting for the device answer".into()))?
    .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?;

    // stored keys must compose the same UFVK as the device exported
    let dindex = crate::db::get_account_dindex(&mut connection, account)
        .await
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?;
    assert_eq!(dindex, 0);
    let hw = crate::db::get_account_hw(&mut connection, account)
        .await
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?;
    assert_eq!(hw, HwKind::Official as u8);
    let ufvk = crate::key::get_account_ufvk(&network, &mut connection, account, 5)
        .await
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?;

    let mnemonic = bip39::Mnemonic::from_str(EMULATOR_SEED)
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("bad seed: {e}")))?;
    let seed = mnemonic.to_seed("");
    let usk = UnifiedSpendingKey::from_seed(&network, &seed, AccountId::try_from(0).unwrap())
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("usk derivation failed: {e}")))?;
    let uvk = usk.to_unified_full_viewing_key();
    let expected = UnifiedFullViewingKey::parse(
        &zcash_address::unified::Ufvk::try_from_items(vec![
            zcash_address::unified::Fvk::Orchard(uvk.orchard().unwrap().to_bytes()),
            zcash_address::unified::Fvk::P2pkh(
                uvk.transparent().unwrap().serialize().try_into().unwrap(),
            ),
        ])
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?,
    )
    .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?
    .encode(&network);

    println!("db ufvk  : {ufvk}");
    println!("expected : {expected}");
    assert_eq!(ufvk, expected, "stored keys do not match the device UFVK");
    std::fs::remove_file(&db).ok();
    Ok(())
}

/// Smoke test for the speculos emulator (or a device via ZEMU_HOST/ZEMU_PORT).
/// Speaks the new app-zcash protocol (CLA 0xE0) and only checks that the app
/// answers GET_FIRMWARE_VERSION with the legacy Zondax version format.
/// Run with the emulator up: `cargo test --features zemu -- --ignored`
#[tokio::test]
#[ignore]
pub async fn ledger_app_version() -> LedgerResult<()> {
    let ledger = LEDGER_ZEMU.lock().await.clone().unwrap();
    let res = ledger
        .execute(APDUCommand {
            cla: 0xE0,
            ins: 0xC4,
            p1: 0,
            p2: 0,
            data: vec![],
        })
        .await?;
    assert_eq!(res.retcode, 0x9000, "app did not answer 0x9000");
    assert_eq!(
        &res.data[..2],
        &[0x38, 0x30],
        "unexpected version format: {}",
        hex::encode(&res.data)
    );
    println!("app version response: {}", hex::encode(&res.data));
    Ok(())
}
