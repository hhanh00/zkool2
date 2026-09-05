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

/// Block until the app is back on its home screen. A device operation is not
/// really finished when the host gets its answer: the app needs a moment to
/// return to ready, and APDUs sent in that window are dropped.
fn wait_until_ready(timeout: std::time::Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if screen_text().to_lowercase().contains("app is ready") {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    false
}

/// Drive the NBGL review: advance pages with the right button. The post-review
/// "Address verified" status and the "Sign transaction" approval page need a
/// both-button press; "Reject transaction" means the approval page was
/// overshot, so step back with left. The driver waits for the review to start,
/// and retires as soon as the app is back on the home screen, so it never
/// outlives the interaction it was spawned for.
fn spawn_approval_driver() {
    std::thread::spawn(|| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        while std::time::Instant::now() < deadline {
            if !screen_text().to_lowercase().contains("app is ready") {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        while std::time::Instant::now() < deadline {
            let text = screen_text();
            if text.to_lowercase().contains("app is ready") {
                return;
            }
            if text.contains("Reject transaction") {
                eprintln!("driver: left (screen: {text:?})");
                ui_post("/button/left", r#"{"action": "press-and-release"}"#);
            } else if text.contains("Sign transaction")
                || text.contains("Confirm")
                || text.contains("verified")
            {
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
    if !wait_until_ready(std::time::Duration::from_secs(30)) {
        return Err(LedgerError::Protocol(
            "device did not return to the home screen".into(),
        ));
    }
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
    if !wait_until_ready(std::time::Duration::from_secs(30)) {
        return Err(LedgerError::Protocol(
            "device did not return to the home screen".into(),
        ));
    }
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

/// Sign an ironwood-to-ironwood v6 transaction with the Official Ledger PCZT
/// protocol against the emulator. The transaction is planned locally from the
/// emulator seed (one ironwood note spend, one recipient output, one hidden
/// internal change note), so the device must derive the same ask and produce
/// spend authorization signatures that verify against zkool's own v6 digest —
/// which is exactly what the PCZT signer checks when they are applied.
/// Needs the emulator up, seeded with EMULATOR_SEED:
/// `cargo test --features zemu -- --ignored --nocapture ledger_official_sign`
#[tokio::test]
#[ignore]
pub async fn ledger_official_sign() -> LedgerResult<()> {
    use std::str::FromStr as _;

    use orchard::{
        note::AssetBase,
        tree::MerkleHashOrchard,
    };
    use pczt::roles::{creator::Creator, io_finalizer::IoFinalizer};
    use rand_core::OsRng;
    use sqlx::Connection as _;
    use zcash_keys::{
        encoding::AddressCodec as _,
        keys::UnifiedSpendingKey,
    };
    use zcash_primitives::transaction::{
        builder::{BuildConfig, Builder, BundlePadding},
        fees::zip317::FeeRule,
    };
    use zcash_protocol::{
        consensus::{BlockHeight, BranchId},
        memo::MemoBytes,
        value::Zatoshis,
    };
    use zip32::AccountId;

    const EMULATOR_SEED: &str = "display accident enable raw glimpse engine know fog bubble price bunker minimum entry tuna joy motor rate tennis evolve october verb jelly indoor dance";

    let network = Network::Main;
    let ledger = LEDGER_ZEMU.lock().await.clone().unwrap();

    // ── keys from the emulator seed ───────────────────────────────────────
    let mnemonic = bip39::Mnemonic::from_str(EMULATOR_SEED)
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("bad seed: {e}")))?;
    let seed = mnemonic.to_seed("");
    let usk = UnifiedSpendingKey::from_seed(&network, &seed, AccountId::try_from(0).unwrap())
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("usk derivation failed: {e}")))?;
    let tsk = usk.transparent();
    let tvk = tsk.to_account_pubkey();
    let (tpk, taddr) = crate::account::derive_transparent_address(&tvk, 0, 0, false)
        .map_err(|e| LedgerError::Anyhow(e))?;
    let address_string = taddr.encode(&network);

    // ── plan an ironwood-to-ironwood v6 tx (spend → recipient + change) ───
    let height = BlockHeight::from_u32(3_500_000);
    assert_eq!(
        BranchId::for_height(&network, height),
        BranchId::Nu6_3,
        "test height must be past NU6.3"
    );

    // Synthetic ironwood note owned by the account (external scope), with a
    // witness at position 0 of an otherwise-empty tree: the siblings are the
    // empty subtrees of each height.
    use incrementalmerkletree::Hashable as _;
    let fvk = orchard::keys::FullViewingKey::from(usk.orchard());
    let spend_recipient = fvk.address_at(zip32::DiversifierIndex::from(0u32), orchard::keys::Scope::External);
    let change_recipient = fvk.address_at(zip32::DiversifierIndex::from(0u32), orchard::keys::Scope::Internal);
    let rho = orchard::note::Rho::from_bytes(&[9u8; 32])
        .into_option()
        .ok_or_else(|| LedgerError::Protocol("bad rho".into()))?;
    let rseed = orchard::note::RandomSeed::from_bytes([7u8; 32], &rho)
        .into_option()
        .ok_or_else(|| LedgerError::Protocol("bad rseed".into()))?;
    let note = orchard::note::Note::from_parts(
        spend_recipient,
        orchard::value::NoteValue::from_raw(100_000),
        AssetBase::zatoshi(),
        rho,
        rseed,
        orchard::note::NoteVersion::V3,
    )
    .into_option()
    .ok_or_else(|| LedgerError::Protocol("bad note".into()))?;
    let cmx = orchard::note::ExtractedNoteCommitment::from(note.commitment());
    let mut auth_path = [MerkleHashOrchard::empty_leaf(); 32];
    let mut state = MerkleHashOrchard::empty_leaf();
    for (l, sibling) in auth_path.iter_mut().enumerate() {
        *sibling = state;
        state = MerkleHashOrchard::combine(
            incrementalmerkletree::Level::from(l as u8),
            &state,
            &state,
        );
    }
    let merkle_path = orchard::tree::MerklePath::from_parts(0, auth_path);
    let anchor = merkle_path.root(cmx);

    let config = BuildConfig::Standard {
        sapling_anchor: None,
        orchard_anchor: None,
        ironwood_anchor: Some(anchor),
        orchard_padding: BundlePadding::DEFAULT,
        ironwood_padding: BundlePadding::DEFAULT,
    };
    let mut builder = Builder::new(&network, height, config);
    builder
        .add_ironwood_spend::<std::convert::Infallible>(fvk.clone(), note, merkle_path)
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("add spend: {e:?}")))?;
    builder
        .add_ironwood_output::<std::convert::Infallible>(
            Some(fvk.to_ovk(orchard::keys::Scope::External)),
            spend_recipient,
            Zatoshis::const_from_u64(40_000),
            MemoBytes::empty(),
        )
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("add output: {e:?}")))?;
    builder
        .add_ironwood_output::<std::convert::Infallible>(
            Some(fvk.to_ovk(orchard::keys::Scope::External)),
            change_recipient,
            Zatoshis::const_from_u64(50_000),
            MemoBytes::empty(),
        )
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("add change: {e:?}")))?;

    let r = builder
        .build_for_pczt(OsRng, &FeeRule::standard(), |_: &AssetBase| false)
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("build_for_pczt: {e:?}")))?;
    let pczt = Creator::build_from_parts(r.pczt_parts)
        .ok_or_else(|| LedgerError::Protocol("creator returned no pczt".into()))?;

    let (pczt, _) = IoFinalizer::new(pczt)
        .finalize_io()
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("io finalizer: {e:?}")))?;

    let ironwood_index = r.ironwood_meta.spend_action_index(0).unwrap();
    let package = crate::api::pay::PcztPackage {
        pczt: pczt
            .serialize()
            .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("serialize: {e:?}")))?,
        n_spends: [0, 0, 0, 1],
        sapling_indices: vec![],
        orchard_indices: vec![],
        ironwood_indices: vec![ironwood_index],
        can_sign: true,
        can_broadcast: false,
        price: None,
        category: None,
        is_issuance: false,
    };

    // ── sign on the emulator ──────────────────────────────────────────────
    let mut db = std::env::temp_dir();
    db.push(format!("zkool_ol_sign_{}.db", std::process::id()));
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

    // Seed the OL account directly in the DB instead of importing it from the
    // device: the import flow is covered by ledger_account_import, and this
    // test only needs the signing protocol against the emulator.
    let account = crate::db::store_account_metadata(
        &mut connection,
        "ol-sign",
        &None,
        &None,
        u32::from(height),
        false,
        false,
    )
    .await
    .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?;
    crate::db::store_account_hw(&mut connection, account, HwKind::Official as u8, 0)
        .await
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?;
    crate::db::init_account_transparent(&mut connection, account, u32::from(height))
        .await
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?;
    crate::db::store_account_transparent_vk(&mut connection, account, &tvk)
        .await
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?;
    crate::db::store_account_transparent_addr(
        &mut connection,
        account,
        0,
        0,
        None,
        &tpk,
        &address_string,
        false,
    )
    .await
    .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?;
    crate::db::init_account_orchard(&network, &mut connection, account, u32::from(height))
        .await
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?;
    crate::db::store_account_orchard_vk(
        &mut connection,
        account,
        &orchard::keys::FullViewingKey::from(usk.orchard()),
    )
        .await
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?;
    crate::db::update_dindex(&mut connection, account, 0, true)
        .await
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("{e}")))?;

    // The device review runs on the last ironwood packet; the driver approves it.
    spawn_approval_driver();

    let signed = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        crate::ledger::official_sign::sign_transaction(
            &network,
            &mut connection,
            account,
            &package,
            None::<&()>,
            &ledger,
        ),
    )
    .await
    .map_err(|_| LedgerError::Protocol("timeout signing on the device".into()))
    .and_then(|r| r.map_err(LedgerError::Anyhow))?;

    // Reaching this point means the device-derived signature verified against
    // zkool's ZIP-244 sighash for the declared derivation path.
    let _signed = pczt::Pczt::parse(&signed.pczt)
        .map_err(|e| LedgerError::Anyhow(anyhow::anyhow!("reparse: {e:?}")))?;
    if !wait_until_ready(std::time::Duration::from_secs(30)) {
        return Err(LedgerError::Protocol(
            "device did not return to the home screen".into(),
        ));
    }
    std::fs::remove_file(&db).ok();
    println!("official ledger signing OK");
    Ok(())
}
