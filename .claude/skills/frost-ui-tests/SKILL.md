---
name: frost-ui-tests
description: "Run the Flutter-UI FROST integration tests (tests/test_dkg_ui.py for DKG, tests/test_frost_ui.py for signing) on a local zebra regtest chain, and optionally screen-record the app window. Use when asked to run, debug, or re-record either UI test, or to bring up the local regtest + lightwalletd + zkool_graphql stack they need."
---

# Flutter UI FROST integration tests

Two tests run the **real Flutter app** as one FROST participant while the others
are headless `zkool_graphql` instances:

| test | what it covers | runtime (warm) |
|---|---|---|
| `tests/tests/test_dkg_ui.py` | 3-of-3 DKG through `/dkg1 → /dkg2 → /dkg3` | ~3.5 min |
| `tests/tests/test_frost_ui.py` | DKG, then signing through `/frost1 → /frost2`, asserting the receiver is actually paid | ~5.5 min |

The Dart halves live in `integration_test/`, sharing `support.dart` (rendezvous,
pumping helpers, wallet setup, and `driveDkg` — signing needs a shared key
before it can start).

`test_dkg.py` / `test_frost.py` are the all-headless equivalents; run those
first to prove the stack before blaming the UI tests.

In both, the app is participant #1 and pytest owns everything else: the chain,
the funding wallet, and the peers. It spawns `flutter test` as a subprocess and
the two sides exchange values through JSON files in the app's documents
directory. In the signing test participant #2 coordinates, so the app exercises
the plain-signer path.

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
rm -f  ~/Library/Containers/cc.methyl.zkool/Data/Documents/regtest_frost_ui.db
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
export SEED="invite couch cloud pave stuff cabbage usual rigid dragon warm cable price fame warfare next swallow worth opera suggest flame patch undo position arctic"
.venv/bin/python -m pytest tests/test_dkg_ui.py -v -s     # DKG only
.venv/bin/python -m pytest tests/test_frost_ui.py -v -s   # DKG + signing
```

Expect ~3.5 min warm; the first run adds a macOS debug build of the app plus the
Rust staticlib. Logs: `/tmp/dkg_ui_flutter.log` (Flutter side, dumped by pytest on
failure) and `/tmp/graphql_8002.log` / `/tmp/graphql_8003.log` (peers).

Watch progress without touching the app's database:

```bash
sed 's/\x1b\[[0-9;]*m//g' /tmp/dkg_ui_flutter.log | grep -a "dkg-ui] status"
```

Expected DKG sequence:

```
Broadcasting participant keys
Waiting for other participants to send their keys
Broadcasting round 1 packages
Waiting for other participants to send their round 1 packages
Broadcasting round 2 packages
The shared address is: uregtest1...
```

`test_frost_ui.py` continues past that into the signing rounds. Participant #2
coordinates, so the app is a plain signer:

```
Sending our commitments to the coordinator
Waiting for the signing package from the coordinator
Sending our signature share to the coordinator
Signing completed
```

It ends by polling the receiver's balance until it reads `0.05000000`, i.e. the
multisig actually spent — not just UI states. The `flutter test` log for signing
is `/tmp/frost_ui_flutter.log`.

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
- **Never use `pumpAndSettle`** on the DKG or signing routes; DKGPage3 and
  FrostPage2 both run a 30s timer and the synchronizer keeps frames scheduled, so
  the tree never settles. Use the `pumpUntil` / `pumpFor` helpers.
- **Leave the DKG page with `go`, not `push`.** DKGPage3 cancels its timer only in
  `dispose`, so a page left mounted keeps calling `doDkg` and throws
  `get_funding_account: no rows` as soon as the selected account moves to the
  FROST account. This is why the signing test does `appRouter.go("/accounts")`
  after the DKG finishes.
- **A PCZT must be byte-identical for every signer**, and it crosses the
  app/zkool_graphql boundary in the signing test. That only works because both
  now use bincode `standard()` — the app used `legacy()` until
  `fix(pay): serialize PCZTs with bincode standard() for interop`. If a signing
  test starts failing at `unpackTransaction`, check that first.
- **FrostPage1 reads `frostParams` off the *current* account**, so the FROST
  account has to be selected (`coinContext.setAccount` plus
  `selectedAccountIdProvider`) before navigating to `/frost1`, and the PCZT is
  passed as the route `extra`.
- **FrostPage2 renders its status in a plain `Text`**, not the `CopyableText`
  DKGPage3 uses, so the two tests read the status differently.

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

`.github/workflows/wallet.yml` runs on `ubuntu-latest` and includes neither
test. Porting it needs a desktop Flutter build plus a display (`-d linux` under
`xvfb-run`, GTK dev packages) on top of the existing regtest stack.
`app_documents_dir()` and `flutter_device()` in `tests/tests/dkg.py` are already
platform-switched for this, but the Linux path is **untested** — only macOS has
been verified.
