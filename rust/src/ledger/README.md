# Ledger emulator dev setup

How to build the Zcash Ledger app, run it in the speculos emulator, and verify
it from zkool's Rust test suite. Everything runs in Docker; no local toolchain
or physical Ledger is needed.

The helper scripts live in `misc/ledger/` (`setup.sh`, `build.sh`,
`run-emulator.sh`). Their knobs (`APP_DIR`, `MODEL`, `BUILD_MODEL`, `SEED`)
are documented at the top of each script.

## Architecture

```
zkool (cargo test --features zemu)
  |  HTTP POST {"apduHex": "<hex>"}          (ledger_transport_zemu::TransportZemuHttp)
  v
host port 9999 --> container port 9998: JSON-HTTP proxy (ledger-live-http-proxy.py)
                        |  binary length-prefixed frames
                        v
                  container port 9999: speculos APDU server
                        v
                  speculos (QEMU, model nanosp) running app-zcash
```

The app under emulation is the Rust rewrite of
[LedgerHQ/app-zcash](https://github.com/LedgerHQ/app-zcash) (v3.9.3+). It
speaks CLA `0xE0` (Bitcoin-app style) with Zcash extensions (INS `0x50` VK,
`0x51` shielded address, `0x52`-`0x59` PCZT/Ironwood). zkool's app-layer code
(`builder.rs`, `fvk.rs`, ...) still speaks the legacy Zondax CLA `0x85`
protocol and needs migration; only the version smoke test speaks `0xE0` today.
The transport layer is protocol-agnostic and unchanged.

## Account types (`accounts.hw`)

| hw | App | Pools | Device protocol |
| --- | --- | --- | --- |
| 0 | software | all | n/a |
| 1 | Zondax | transparent + sapling | CLA `0x85` |
| 2 | Official | transparent + ironwood | CLA `0xE0` (not implemented yet) |

v1 Official behavior: accounts are created from the recovery seed with
standard ZIP-32 derivation (no device needed); device operations return clear
errors via `ledger::mock::StubLedger`. Ironwood shares its keys with Orchard
(see `api::account::get_account_pools`), so Official accounts are initialized
with the Orchard key set. Pool masks are validated at creation
(`account.rs`). Selection UI lives in the New Account form
(`lib/pages/new_account.dart`).

## Prerequisites

- Docker Desktop installed and running (`docker info` succeeds).
- Git.

Do NOT pull the images with `--platform linux/amd64` on Apple Silicon: both
images are multi-arch and a plain pull selects the native arm64 variant.
Forcing amd64 fails at runtime with `exec format error` unless Rosetta is
enabled in Docker Desktop.

## 1. One-time setup

```bash
misc/ledger/setup.sh
```

Clones the app to `~/projects/ledger-dev/app-zcash` (override with `APP_DIR`)
and pulls both images.

## 2. Build the app

```bash
misc/ledger/build.sh
```

Build model names: `nanox`, `nanosplus`, `stax`, `flex`, `apex_p` (default
`nanosplus`). Output: `~/projects/ledger-dev/app-zcash/target/nanosplus/release/zcash`.

## 3. Run the emulator

```bash
misc/ledger/run-emulator.sh
```

Why the container is started the way it is (do not simplify):

- Build model `nanosplus` maps to speculos model `nanosp`.
- The JSON-HTTP proxy (`ledger-live-http-proxy.py`) is what serves the
  `{"apduHex": ...}` protocol zkool's transport speaks; speculos' own port
  9999 is binary-framed.
- The proxy binds `127.0.0.1` by default, which Docker port-forwarding cannot
  reach — hence the `sed` to `0.0.0.0` inside the container.
- `-p 9999:9998`: host 9999 (what zkool connects to) -> proxy on container 9998.
- The seed is a fixed test seed so derived addresses are reproducible.

The device screen is available for approvals at `http://localhost:5000`
(open `/swagger`, use the screenshot/button endpoints).

## 4. Verify the emulator

```bash
curl -s -m 5 -X POST http://127.0.0.1:9999 \
  -H 'Content-Type: application/json' -d '{"apduHex": "e0c4000000"}'
```

Expected:

```json
{"data": "38300309030100039000", "error": null}
```

`e0c4000000` is `GET_FIRMWARE_VERSION` (CLA 0xE0, INS 0xC4); the answer is the
legacy Zondax version format (`0x38 0x30`, version 3.9.3) plus SW `0x9000`.

## 5. Run the zkool test

From `rust/`:

```bash
cargo test --features zemu -- --ignored --nocapture ledger_app_version
```

Expected:

```
app version response: 3830030903010003
test ledger::tests::ledger_app_version ... ok
```

The test lives in `rust/src/ledger/tests.rs` and is `#[ignore]`-gated so
regular `cargo test` runs do not require the emulator.

- `ZEMU_HOST` / `ZEMU_PORT` env vars override the endpoint (defaults
  `127.0.0.1:9999`), e.g. to point at an emulator on another machine.
- On macOS this hits the local speculos container by design; a physical
  device is not part of this setup.

## 6. Stop

```bash
docker rm -f speculos-zcash
```

## Troubleshooting

| Symptom | Cause / fix |
| --- | --- |
| `exec /usr/local/bin/python: exec format error` | amd64 image pulled on Apple Silicon: re-pull without `--platform`, or enable Rosetta in Docker Desktop |
| `invalid choice: 'nanosplus'` from speculos | speculos model is `nanosp`; build model is `nanosplus` |
| curl connects but empty reply | proxy not running or still bound to `127.0.0.1` (see step 3), or wrong port mapping (`9999:9998`, not `9999:9999`) |
| `Connection refused` on container 9998 | the proxy crashed; check `docker logs speculos-zcash` |
| test hangs | emulator not up: `docker ps --filter name=speculos-zcash`, then re-run step 4 |
