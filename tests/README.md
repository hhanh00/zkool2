# zkool-tests

Python test utilities and integration tests for zkool.

## Setup (uv)

```bash
cd tests
uv sync
```

## Environment Variables

```bash
export GRAPHQL_URL="http://localhost:8000/graphql"
export WS_URL="ws://localhost:8000/subscriptions"
export RPC_URL="http://127.0.0.1:18232/"
export SEED="invite couch cloud pave stuff cabbage usual rigid dragon warm cable price fame warfare next swallow worth opera suggest flame patch undo position arctic"
```

## Running Tests

```bash
# Run all tests
.venv/bin/uv run pytest

# Run smoke test
.venv/bin/uv run pytest tests/test_smoke.py -v

# Run with output
.venv/bin/uv run pytest tests/test_smoke.py -v -s
```

## Setup (pip)

```bash
cd tests
python3 -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
pytest
```

## Flutter UI DKG test

`tests/test_dkg_ui.py` runs the same 3-of-3 FROST DKG as `test_dkg.py`, but
participant #1 is the **real Flutter app** instead of a headless
`zkool_graphql` instance: pytest starts the funding instance and participants
#2/#3, then spawns

```bash
flutter test integration_test/dkg_ui_test.dart -d <macos|linux> \
  --dart-define=ZKOOL_TEST_RENDEZVOUS=<dir>
```

and drives the app's `/dkg1 → /dkg2 → /dkg3` pages. Both sides exchange addresses through JSON
files in the app's documents directory — on macOS
`~/Library/Containers/cc.methyl.zkool/Data/Documents/dkg_ui_rendezvous`, since
the sandboxed app cannot use `/tmp`; on Linux `~/Documents/dkg_ui_rendezvous`.

```bash
.venv/bin/uv run pytest tests/test_dkg_ui.py -v -s
```

Requirements beyond the usual regtest stack:
- a desktop session — `flutter test -d macos|linux` opens a window (under CI on
  Linux this needs `xvfb-run`)
- the app's documents directory must exist; on macOS that means the app has
  been launched at least once so its sandbox container is created
- the first run builds the desktop app and the Rust staticlib in debug (slow)

The test writes `regtest_dkg_ui.db` into the app's Documents directory and
temporarily overrides the `pin_lock`, `offline` and `vault` preferences,
restoring them when it finishes. It never touches the developer's own wallet
database.
