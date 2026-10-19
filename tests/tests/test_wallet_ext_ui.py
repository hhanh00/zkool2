"""The remaining wallet pages with the real Flutter app, one app and no peers.

The app (integration_test/wallet_ext_ui_test.dart) opens the pages the
everyday wallet test (test_wallet_ui.py) leaves out: settings and its
sub-pages, the account view tabs, contacts, the address list, the error
display and the database manager. This side owns the chain and the SEED
wallet, funds the app's account at every pool in one transaction (with a
dust note and a memo, which those pages need), serves a mock CoinGecko so
the price settings stay offline, mines on demand, and holds the run to a
ten-minute budget. Coverage is gathered and printed as in test_wallet_ui.py.

Not in the UI workflow's default list: it runs when named in the `tests`
input.
"""

import asyncio
import contextlib
import os
import shutil
import sys
import time

import pytest
from coingecko_mock import CoingeckoMock
from dkg import DkgParticipant, Rendezvous, app_documents_dir, demand_miner, launch_flutter_test
from gql import GraphQLRequest
from test_frost_upgrade import SYNC, wait_ready
from test_key_import import create
from test_uivk import PAY
from test_wallet_ui import BUDGET_SECONDS, BUILD_TIMEOUT, lib_line_coverage
from utils import dump_server_log, kill_existing_zkool_processes

DEFAULT_PORT = 8000
LWD_URL = "http://localhost:8137"
FLUTTER_LOG = "/tmp/wallet_ext_ui_flutter.log"
LCOV_COPY = "/tmp/wallet_ext_ui_lcov.info"
TARGET_SECONDS = 360
FUND_TADDR = "0.3"
FUND_SADDR = "0.2"
FUND_OADDR = "0.4"
# Below the notes tab's dust threshold (5000 zats).
FUND_DUST = "0.00003"
MEMO = "wallet-ext memo"


@pytest.mark.dkg_ui
@pytest.mark.asyncio
async def test_wallet_ext_flutter_ui(rpc_url, seed, zkool_binary):
    if not seed:
        pytest.skip("SEED not set")
    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")
    docs_dir = app_documents_dir()
    if not os.path.isdir(docs_dir):
        pytest.skip(f"app documents directory {docs_dir} not found")
    if shutil.which(os.getenv("FLUTTER_BIN", "flutter")) is None:
        pytest.skip("flutter not on PATH")

    ui_db_path = os.path.join(docs_dir, "regtest_wallet_ext_ui.db")
    rendezvous = Rendezvous(os.path.join(docs_dir, "dkg_ui_rendezvous"))
    coverage = bool(os.getenv("ZKOOL_UI_COVERAGE"))
    repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    lcov_path = os.path.join(repo_root, "coverage", "lcov.info")

    mock = CoingeckoMock()
    wallet = DkgParticipant(DEFAULT_PORT, "/tmp/regtest_wallet_ext_ui_funder.db", LWD_URL)
    flutter = None
    miner = None
    started = time.monotonic()
    main = None

    async def launch():
        nonlocal flutter, miner, started, main
        print("=== Step 1: Chain, funding wallet and price mock ===")
        mock.start()
        wallet.start(zkool_binary)
        await wait_ready(wallet)
        main = await create(wallet, "Main", seed)
        await wallet.execute(GraphQLRequest(SYNC, variable_values={"account": main}))

        print("\n=== Step 2: Launch the app ===")
        rendezvous.write_config(
            db_path=ui_db_path,
            lwd=LWD_URL,
            coingecko_url=mock.url,
            fund_taddr=FUND_TADDR,
            fund_saddr=FUND_SADDR,
            fund_oaddr=FUND_OADDR,
            fund_dust=FUND_DUST,
            # The lock screen is behind authenticate(): Touch ID on macOS, open on Linux.
            biometric_gate=sys.platform == "darwin",
        )
        extra = ["--coverage", "--coverage-path", lcov_path] if coverage else []
        flutter = await launch_flutter_test(
            "wallet_ext_ui_test.dart", rendezvous, FLUTTER_LOG, extra_args=extra
        )
        miner = asyncio.create_task(demand_miner(rpc_url, idle_fallback=3600))
        phase = await rendezvous.poll_ui("phase", timeout=BUILD_TIMEOUT, process=flutter)
        started = time.monotonic()
        print(f"App up (phase {phase}); the {BUDGET_SECONDS}s budget starts now")

    async def body():
        print("\n=== Step 3: Fund account A at every pool, plus a dust note ===")
        taddr = await rendezvous.poll_ui("a_taddr", timeout=BUDGET_SECONDS, process=flutter)
        saddr = await rendezvous.poll_ui("a_saddr", timeout=BUDGET_SECONDS, process=flutter)
        oaddr = await rendezvous.poll_ui("a_oaddr", timeout=BUDGET_SECONDS, process=flutter)
        await wallet.execute(
            GraphQLRequest(
                PAY,
                variable_values={
                    "account": main,
                    "recipients": [
                        {"address": taddr, "amount": FUND_TADDR},
                        {"address": saddr, "amount": FUND_SADDR},
                        {"address": oaddr, "amount": FUND_OADDR, "memo": MEMO},
                        {"address": oaddr, "amount": FUND_DUST},
                    ],
                },
            )
        )
        print(f"Paid {FUND_TADDR} to {taddr}, {FUND_SADDR} to {saddr[:20]}…, "
              f"{FUND_OADDR} and {FUND_DUST} to {oaddr[:20]}…")

        print("\n=== Step 4: The app works through the pages ===")
        await rendezvous.poll_ui("done", timeout=BUDGET_SECONDS, process=flutter)
        returncode = await asyncio.wait_for(flutter.wait(), timeout=300)
        assert returncode == 0, f"flutter test failed with {returncode}, see {FLUTTER_LOG}"

    try:
        await kill_existing_zkool_processes()
        rendezvous.setup()
        for stale in (ui_db_path, FLUTTER_LOG, LCOV_COPY):
            if os.path.exists(stale):
                os.remove(stale)
        await launch()
        try:
            await asyncio.wait_for(body(), timeout=BUDGET_SECONDS)
        except asyncio.TimeoutError:
            ui = rendezvous.read_ui()
            pytest.fail(
                f"the run exceeded {BUDGET_SECONDS}s "
                f"(last phase: {ui.get('phase')}, last step: {ui.get('step')})"
            )
        elapsed = time.monotonic() - started
        print(f"\n=== ✅ Extended wallet UI test passed in {elapsed:.0f}s (target {TARGET_SECONDS}s) ===")
        if coverage and os.path.exists(lcov_path):
            shutil.copy(lcov_path, LCOV_COPY)
            for label, only in (("lib/", False), ("in-scope lib/", True)):
                hit, found = lib_line_coverage(lcov_path, in_scope_only=only)
                pct = 100.0 * hit / found if found else 0.0
                print(f"{label} line coverage: {hit}/{found} = {pct:.1f}%")
            print(f"lcov report copied to {LCOV_COPY}")
    except Exception:
        ui = rendezvous.read_ui()
        print(f"\nLast phase: {ui.get('phase')}\nLast step: {ui.get('step')}")
        dump_server_log(FLUTTER_LOG, "FLUTTER TEST LOG")
        dump_server_log(f"/tmp/graphql_{DEFAULT_PORT}.log", "FUNDING WALLET")
        raise
    finally:
        if miner is not None:
            miner.cancel()
            await asyncio.gather(miner, return_exceptions=True)
        if flutter is not None and flutter.returncode is None:
            flutter.terminate()
            with contextlib.suppress(asyncio.TimeoutError):
                await asyncio.wait_for(flutter.wait(), timeout=30)
        await wallet.stop()
        mock.stop()
        rendezvous.cleanup()
        if os.path.exists(ui_db_path):
            os.remove(ui_db_path)
