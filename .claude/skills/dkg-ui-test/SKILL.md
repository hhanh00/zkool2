---
name: dkg-ui-test
description: "Run the Flutter-UI FROST DKG integration test (tests/test_dkg_ui.py) on a local zebra regtest chain, and optionally screen-record the app window. Use when asked to run, debug, or re-record the DKG UI test, or to bring up the local regtest + lightwalletd + zkool_graphql stack that it needs."
---

# Flutter UI DKG integration test

`tests/tests/test_dkg_ui.py` runs a 3-of-3 FROST DKG where **participant #1 is the
real Flutter app**, driven through `/dkg1 → /dkg2 → /dkg3`, while participants #2
and #3 are headless `zkool_graphql` instances. pytest owns the chain and the
peers and spawns `flutter test` as a subprocess; the two sides exchange addresses
through JSON files in the app's documents directory.

Contrast with `tests/tests/test_dkg.py`, which is the same protocol with all three
participants headless — run that first to prove the stack before blaming the UI test.

## Prerequisites

Binaries not in the repo (this machine has them under `~/projects/tools/bin`):

```bash
export PATH="$HOME/projects/tools/bin:$PATH"   # zebrad, lightwalletd
```

Build the GraphQL server used for the peers (release; ~25 min from cold):

```bash
cd rust
cargo build --release --bin zkool_graphql --features=graphql,bundled-sapling-params
```

macOS: the app must have been launched at least once so its sandbox container
exists at `~/Library/Containers/cc.methyl.zkool/Data/Documents`.

## 1. Bring up the regtest chain

Blocks are **not** mined automatically — `misc/zebra.toml` sets only
`miner_address`, zebra's internal miner is off. Every block comes from an
explicit `generate` RPC call.

```bash
cd ~/projects/zkool2
export PATH="$HOME/projects/tools/bin:$PATH"

# Full reset (chain + all test data)
pkill -9 zkool_graphql; pkill lightwalletd; pkill zebrad
rm -rf ~/Library/Caches/zebra ./data ./regtest.db
rm -f  ~/Library/Containers/cc.methyl.zkool/Data/Documents/regtest_dkg_ui.db
rm -rf ~/Library/Containers/cc.methyl.zkool/Data/Documents/dkg_ui_rendezvous

# zebra.toml needs a miner address (local edit, do not commit)
sed -i '' 's#miner_address = ""#miner_address = "tmQ1BiNRfsvT6eMkJ5n7nMZsbz1PGwCs1Zs"#' misc/zebra.toml

nohup zebrad -c misc/zebra.toml start > zebrad.log 2>&1 &
until curl -sf -m 3 --data-binary '{"jsonrpc":"1.0","id":"p","method":"getinfo","params":[]}' \
  -H 'content-type: application/json' http://127.0.0.1:18232/ >/dev/null; do sleep 2; done

mkdir -p ./data/regtest
nohup lightwalletd --no-tls-very-insecure --data-dir=./data/regtest \
  --grpc-bind-addr=127.0.0.1:8137 --zcash-conf-path=./misc/zebra.conf \
  --log-file=/dev/stdout > lightwalletd.log 2>&1 &
until nc -z -w 2 localhost 8137; do sleep 2; done

# Mine past NU6.3 / Ironwood, which activates at height 250 on regtest
curl -s --data-binary '{"jsonrpc":"1.0","id":"g","method":"generate","params":[350]}' \
  -H 'Content-type: application/json' http://127.0.0.1:18232/ | jq '.result|length'
```

## 2. Fund the SEED wallet

`example/sh/regtest_setup.sh` does chain bring-up *and* funding in one go. If the
chain is already up, run only the funding half: start `zkool_graphql` on :8000
against `regtest.db`, create the `miner` (MINER_SEED) and `wallet` (SEED)
accounts at birth 1, sync both, take the miner's notes with
`height < tip-100`, pay the total to `DESTINATION_ADDRESS` with
`recipientPaysFee: true, confirmations: 100`, mine 10, sync, and check
`balanceByAccount.ironwood > 0` (expect ~62.5).

Seeds and the destination UA are inlined at the top of `example/sh/regtest_setup.sh`.

## 3. Run the test

```bash
cd tests
SEED="invite couch cloud pave stuff cabbage usual rigid dragon warm cable price fame warfare next swallow worth opera suggest flame patch undo position arctic" \
  .venv/bin/python -m pytest tests/test_dkg_ui.py -v -s
```

Expect ~3.5 min warm; the first run adds a macOS debug build of the app plus the
Rust staticlib. Logs: `/tmp/dkg_ui_flutter.log` (Flutter side, dumped by pytest on
failure) and `/tmp/graphql_8002.log` / `/tmp/graphql_8003.log` (peers).

Watch progress without touching the app's database:

```bash
sed 's/\x1b\[[0-9;]*m//g' /tmp/dkg_ui_flutter.log | grep -a "dkg-ui] status"
```

Expected sequence:

```
Broadcasting participant keys
Waiting for other participants to send their keys
Broadcasting round 1 packages
Waiting for other participants to send their round 1 packages
Broadcasting round 2 packages
The shared address is: uregtest1...
```

## Things that will bite you

- **Never query the app's SQLite file while the test runs.** It is in rollback-journal
  mode, and a concurrent reader causes `database is locked` errors inside the app.
- **The db filename must contain `regtest`** — that substring is what selects
  `Network::Regtest` (`rust/src/api/coin.rs:266`). Without it the app runs mainnet
  parameters.
- **The macOS app is sandboxed** and cannot read or write `/tmp`; the rendezvous dir
  and the UI participant's db must live in the container's Documents directory.
- **Do not `osascript -e 'tell application "zkool" to activate'`** — it launches
  `/Applications/zkool.app` (the real mainnet wallet), which shares the bundle id
  and container with the Debug build and breaks the run (`flutter test` exits 79).
  Raise the test app by process only: `tell application "System Events" to set
  frontmost of process "zkool" to true`, which needs Accessibility for Terminal.
- **Mining is on demand**, driven by `demand_miner()` in `tests/tests/dkg.py`: it
  polls `getrawmempool` and mines one block whenever a transaction is waiting, with
  a 60s idle fallback. Do not go back to a fixed-interval miner — it makes the run
  non-deterministic.
- **The app polls every 30s** (`Timer.periodic` in `DKGPage3`, `lib/pages/dkg.dart`)
  while the peers advance on every block, so the app is usually a round behind. The
  test waits up to `PEER_COMPLETION_TIMEOUT` (600s) for the peers to finish *after*
  the app is done — without that wait the test fails with "No FROST account for
  participant 2".
- **Never use `pumpAndSettle`** on the DKG route; the 30s timer and the synchronizer
  keep frames scheduled so it never settles. Use the `pumpUntil` / `pumpFor` helpers.

## Screen-recording the run (macOS)

Requires Accessibility for Terminal (System Settings → Privacy & Security →
Accessibility) so the window can be raised. A local, uncommitted block in
`macos/Runner/MainFlutterWindow.swift` forces the window to 1100x820 and sets
`isRestorable = false`; it must run **after** `super.awakeFromNib()`, or macOS
state restoration reapplies the old near-fullscreen frame.

```bash
# once the app is up
osascript -e 'tell application "System Events" to set frontmost of process "zkool" to true'
osascript -e 'tell application "System Events" to tell process "zkool" to get {position, size} of front window'
# -> e.g. 305, 75, 1100, 850   (points, not pixels)

screencapture -v -R305,75,1100,850 /tmp/rec.mp4 &   # region capture: app window only
# re-raise every few seconds while the run proceeds, then:
kill -INT <screencapture pid>                        # SIGINT finalizes the mp4

ffmpeg -i /tmp/rec.mp4 -vf "scale=1100:-2,fps=12" -c:v libx264 -crf 30 \
  -pix_fmt yuv420p /tmp/dkg_ui.mp4
```

Trim the tail: when the test ends the app closes and the recorded region exposes
whatever was behind it. The Dart test holds the finished screen for 6s
(`pumpFor`) so the Finalize step and shared address are visible.

## Teardown

```bash
pkill -f zkool_graphql; pkill lightwalletd; pkill zebrad
rm -rf ~/Library/Caches/zebra ./data ./regtest.db
git checkout misc/zebra.toml     # drops the local miner_address edit
```

## CI

`.github/workflows/wallet.yml` runs on `ubuntu-latest` and does not include this
test. Porting it needs a desktop Flutter build plus a display (`-d linux` under
`xvfb-run`, GTK dev packages) on top of the existing regtest stack.
`app_documents_dir()` and `flutter_device()` in `tests/tests/dkg.py` are already
platform-switched for this, but the Linux path is **untested** — only macOS has
been verified.
