"""Everyday wallet flows with the real Flutter app, one app and no peers.

The app (integration_test/wallet_ui_test.dart) creates accounts, receives,
edits, pays two recipients in one transaction, shows fiat prices and deletes
an account. This side owns the chain and the SEED wallet that funds the app,
serves a mock CoinGecko with fixed prices, mines on demand, checks the app's
transaction on the node, and holds the run to a ten-minute budget. The app
runs with `--coverage` when ZKOOL_UI_COVERAGE is set; the lib/ line coverage
is printed and the lcov report copied to /tmp for the CI artifact.
"""

import asyncio
import contextlib
import os
import shutil
import sys
import time

import httpx
import pytest
from coingecko_mock import CoingeckoMock
from dkg import DkgParticipant, Rendezvous, app_documents_dir, demand_miner, launch_flutter_test
from gql import GraphQLRequest
from test_frost_upgrade import SYNC, wait_ready
from test_key_import import create
from test_uivk import PAY
from utils import dump_server_log, kill_existing_zkool_processes

DEFAULT_PORT = 8000
LWD_URL = "http://localhost:8137"
FLUTTER_LOG = "/tmp/wallet_ui_flutter.log"
LCOV_COPY = "/tmp/wallet_ui_lcov.info"
# The budget runs from the app's first phase: building the app (tens of
# minutes on a cold CI runner) has its own, longer wait.
BUDGET_SECONDS = 600
BUILD_TIMEOUT = 2400
TARGET_SECONDS = 360
FUND_UA = "0.5"
FUND_TADDR = "0.3"
USD_PRICE = 50.0
EUR_PRICE = 45.0


async def rpc(rpc_url: str, method: str, params: list):
    payload = {"jsonrpc": "1.0", "id": method, "method": method, "params": params}
    async with httpx.AsyncClient() as client:
        response = await client.post(rpc_url, json=payload)
        response.raise_for_status()
        return response.json()


# Hand-written code this test is not meant to reach: features with their own
# stacks or tests (voting, FROST and DKG, swaps, the cloud vault, ZSA), the
# webview pages (market, charts), and note migration, which needs a chain
# from before Ironwood activates.
OUT_OF_SCOPE = (
    "lib/pages/voting", "lib/pages/dkg.dart", "lib/pages/frost.dart",
    "lib/widgets/frost_", "lib/pages/swap", "lib/services/swap_",
    "lib/vault.dart", "lib/widgets/vault_", "lib/pages/zsa.dart",
    "lib/pages/market.dart", "lib/chart.dart", "lib/pages/migrate.dart",
)
GENERATED = ("lib/src/rust/", ".g.dart", ".freezed.dart")


def lib_line_coverage(lcov_path: str, in_scope_only: bool = False) -> tuple[int, int]:
    """(hit, found) line counts over lib/ in an lcov report. With
    `in_scope_only`, generated code and OUT_OF_SCOPE files are left out."""
    hit = found = 0
    counting = False
    with open(lcov_path) as f:
        for line in f:
            if line.startswith("SF:"):
                path = line[3:].strip()
                path = path[path.index("lib/"):] if "lib/" in path else path
                counting = path.startswith("lib/")
                if in_scope_only and counting:
                    counting = not any(g in path for g in GENERATED) and not any(
                        path.startswith(o) for o in OUT_OF_SCOPE
                    )
            elif counting and line.startswith("LH:"):
                hit += int(line[3:])
            elif counting and line.startswith("LF:"):
                found += int(line[3:])
    return hit, found


@pytest.mark.dkg_ui
@pytest.mark.asyncio
async def test_wallet_flutter_ui(rpc_url, seed, zkool_binary):
    if not seed:
        pytest.skip("SEED not set")
    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")
    docs_dir = app_documents_dir()
    if not os.path.isdir(docs_dir):
        pytest.skip(f"app documents directory {docs_dir} not found")
    if shutil.which(os.getenv("FLUTTER_BIN", "flutter")) is None:
        pytest.skip("flutter not on PATH")

    ui_db_path = os.path.join(docs_dir, "regtest_wallet_ui.db")
    rendezvous = Rendezvous(os.path.join(docs_dir, "dkg_ui_rendezvous"))
    coverage = bool(os.getenv("ZKOOL_UI_COVERAGE"))
    repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    lcov_path = os.path.join(repo_root, "coverage", "lcov.info")

    mock = CoingeckoMock({"usd": USD_PRICE, "eur": EUR_PRICE, "btc": 0.0005})
    wallet = DkgParticipant(DEFAULT_PORT, "/tmp/regtest_wallet_ui_funder.db", LWD_URL)
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
            usd_price=USD_PRICE,
            eur_price=EUR_PRICE,
            fund_ua=FUND_UA,
            fund_taddr=FUND_TADDR,
            # "Show All" is behind authenticate(): Touch ID on macOS, open on Linux.
            biometric_gate=sys.platform == "darwin",
        )
        extra = ["--coverage", "--coverage-path", lcov_path] if coverage else []
        flutter = await launch_flutter_test(
            "wallet_ui_test.dart", rendezvous, FLUTTER_LOG, extra_args=extra
        )
        # Mine only what the mempool holds: a stray idle block between the
        # app's sync and its send would raise the Sync Warning dialog.
        miner = asyncio.create_task(demand_miner(rpc_url, idle_fallback=3600))
        phase = await rendezvous.poll_ui("phase", timeout=BUILD_TIMEOUT, process=flutter)
        started = time.monotonic()
        print(f"App up (phase {phase}); the {BUDGET_SECONDS}s budget starts now")

    async def body():
        print("\n=== Step 3: Fund account A at its unified and transparent addresses ===")
        a_ua = await rendezvous.poll_ui("a_ua", timeout=BUDGET_SECONDS, process=flutter)
        a_taddr = await rendezvous.poll_ui("a_taddr", timeout=BUDGET_SECONDS, process=flutter)
        await wallet.execute(
            GraphQLRequest(
                PAY,
                variable_values={
                    "account": main,
                    "recipients": [
                        {"address": a_ua, "amount": FUND_UA},
                        {"address": a_taddr, "amount": FUND_TADDR},
                    ],
                },
            )
        )
        print(f"Paid {FUND_UA} to {a_ua[:20]}… and {FUND_TADDR} to {a_taddr}")

        print("\n=== Step 4: The app pays; the node must know the transaction ===")
        txid = await rendezvous.poll_ui("txid", timeout=BUDGET_SECONDS, process=flutter)
        result = await rpc(rpc_url, "getrawtransaction", [txid, 1])
        assert result.get("error") is None, f"node does not know {txid}: {result['error']}"
        print(f"Transaction {txid} known to the node (height {result['result'].get('height')})")
        # The demand miner mines whatever sits in the mempool; wait until it did.
        for _ in range(60):
            result = await rpc(rpc_url, "getrawtransaction", [txid, 1])
            if result["result"].get("height", -1) > 0:
                break
            await asyncio.sleep(2)
        else:
            pytest.fail(f"{txid} was not mined")
        rendezvous.write_orchestrator(mined=True)

        print("\n=== Step 5: The app finishes the remaining phases ===")
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
        print(f"\n=== ✅ Wallet UI test passed in {elapsed:.0f}s (target {TARGET_SECONDS}s) ===")
        print(f"CoinGecko mock served {len(mock.requests)} requests")
        assert mock.requests, "the app never asked the price mock"
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
