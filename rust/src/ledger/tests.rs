use crate::ledger::transport::{APDUCommand, Device, LEDGER_ZEMU};

use super::*;

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
