"""Screenshots of the docs site's Guide pages, taken from the real Flutter
app on regtest.

The app (integration_test/docs_guide_ui_test.dart) walks the flows the Guide
pages describe: first run, a new and a restored account, the account list,
the account page before and after funds arrive, the receive page, a payment
through the extra options and the transaction plan, and a multi-recipient
queue. It rasterizes each screen the pages show, with the rectangles of the
widgets their text points at. This side owns the chain and the funding
wallet: it pays the app's account a few times so the history has rows, mines
on demand, then draws the annotations with tests/tools/docs_shots.py and
copies the images into the docs site ($ZKOOL_DOCS_DIR/guide/images, default
~/projects/zkool-docs/docs).

Run it with the regtest stack up and the SEED wallet funded (tests/README.md):

    .venv/bin/python -m pytest tests/test_docs_guide_ui.py -v -s
"""

import asyncio
import contextlib
import os
import shutil
import subprocess
import sys

import httpx
import pytest
from dkg import (
    DkgParticipant,
    Rendezvous,
    app_documents_dir,
    demand_miner,
    get_raw_mempool,
    launch_flutter_test,
)
from gql import GraphQLRequest
from test_frost_upgrade import SYNC, wait_ready
from test_key_import import create
from test_uivk import ADDRESSES, BALANCE, PAY
from utils import dump_server_log, kill_existing_zkool_processes

DEFAULT_PORT = 8000
LWD_URL = "http://localhost:8137"
FLUTTER_LOG = "/tmp/docs_guide_ui_flutter.log"
FRAMES_DIR = "docs_frames_guide"
DOCS_DIR = os.getenv("ZKOOL_DOCS_DIR", os.path.expanduser("~/projects/zkool-docs/docs"))
DOCS_SHOTS = os.path.join(os.path.dirname(__file__), "..", "tools", "docs_shots.py")
BUILD_TIMEOUT = 2400
BUDGET_SECONDS = 900
# What the app's account receives before it pays back, so the history and
# the memos have rows: amounts and memos.
FUNDING = [("0.3", "Welcome to Zkool"), ("0.15", None), ("0.05", "Coffee money")]


async def rpc(rpc_url: str, method: str, params: list):
    payload = {"jsonrpc": "1.0", "id": method, "method": method, "params": params}
    async with httpx.AsyncClient() as client:
        response = await client.post(rpc_url, json=payload)
        response.raise_for_status()
        return response.json()


async def wait_mined(rpc_url: str, txid: str):
    for _ in range(60):
        result = await rpc(rpc_url, "getrawtransaction", [txid, 1])
        if result.get("error") is None and result["result"].get("height", -1) > 0:
            return
        await asyncio.sleep(2)
    pytest.fail(f"{txid} was not mined")


async def wait_mempool_empty(rpc_url: str):
    """The demand miner mines whatever waits; this waits for it to have done so."""
    for _ in range(60):
        if not await get_raw_mempool(rpc_url):
            return
        await asyncio.sleep(2)
    pytest.fail("the mempool did not drain")


@pytest.mark.dkg_ui
@pytest.mark.asyncio
async def test_docs_guide_flutter_ui(rpc_url, seed, zkool_binary):
    if not seed:
        pytest.skip("SEED not set")
    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")
    docs_dir = app_documents_dir()
    if not os.path.isdir(docs_dir):
        pytest.skip(f"app documents directory {docs_dir} not found")
    if shutil.which(os.getenv("FLUTTER_BIN", "flutter")) is None:
        pytest.skip("flutter not on PATH")
    if not os.path.isdir(DOCS_DIR):
        pytest.skip(f"docs site not found at {DOCS_DIR} (ZKOOL_DOCS_DIR)")

    ui_db_path = os.path.join(docs_dir, "regtest_docs_guide_ui.db")
    frames_dir = os.path.join(docs_dir, FRAMES_DIR)
    rendezvous = Rendezvous(os.path.join(docs_dir, "dkg_ui_rendezvous"))

    wallet = DkgParticipant(DEFAULT_PORT, "/tmp/regtest_docs_guide_funder.db", LWD_URL)
    flutter = None
    miner = None
    main = None

    async def launch():
        nonlocal flutter, miner, main
        print("=== Step 1: Chain and funding wallet ===")
        wallet.start(zkool_binary)
        await wait_ready(wallet)
        main = await create(wallet, "Main", seed)
        await wallet.execute(GraphQLRequest(SYNC, variable_values={"account": main}))
        by_account = {"account": main}
        addresses = (await wallet.execute(GraphQLRequest(ADDRESSES, variable_values=by_account)))[
            "addressByAccount"
        ]
        balance = (await wallet.execute(GraphQLRequest(BALANCE, variable_values=by_account)))[
            "balanceByAccount"
        ]
        needed = sum(float(amount) for amount, _ in FUNDING) + 0.01
        if float(balance["total"]) < needed:
            pytest.fail(
                f"the SEED wallet holds {balance['total']} ZEC on this chain, {needed} needed: "
                "fund it first (tests/README.md)"
            )

        print("\n=== Step 2: Launch the app ===")
        rendezvous.write_config(
            db_path=ui_db_path,
            lwd=LWD_URL,
            frames_dir=FRAMES_DIR,
            destination=addresses["ironwood"],
            transparent=addresses["transparent"],
            sapling=addresses["sapling"],
            fund=str(sum(float(amount) for amount, _ in FUNDING)),
        )
        flutter = await launch_flutter_test("docs_guide_ui_test.dart", rendezvous, FLUTTER_LOG)
        miner = asyncio.create_task(demand_miner(rpc_url, idle_fallback=3600))

    async def body():
        print("\n=== Step 3: Fund the account ===")
        a_ua = await rendezvous.poll_ui("a_ua", timeout=BUILD_TIMEOUT, process=flutter)
        for amount, memo in FUNDING:
            recipient = {"address": a_ua, "amount": amount}
            if memo:
                recipient["memo"] = memo
            await wallet.execute(
                GraphQLRequest(PAY, variable_values={"account": main, "recipients": [recipient]})
            )
            print(f"Paid {amount} to {a_ua[:20]}…")
            # Each payment is mined before the next, so they never contend
            # for the same notes.
            await wait_mempool_empty(rpc_url)
            await wallet.execute(GraphQLRequest(SYNC, variable_values={"account": main}))

        print("\n=== Step 4: The app pays back ===")
        txid = await rendezvous.poll_ui("txid", timeout=BUDGET_SECONDS, process=flutter)
        await wait_mined(rpc_url, txid)
        rendezvous.write_orchestrator(mined=True)

        await rendezvous.poll_ui("done", timeout=BUDGET_SECONDS, process=flutter)
        returncode = await asyncio.wait_for(flutter.wait(), timeout=300)
        assert returncode == 0, f"flutter test failed with {returncode}, see {FLUTTER_LOG}"

    try:
        await kill_existing_zkool_processes()
        rendezvous.setup()
        shutil.rmtree(frames_dir, ignore_errors=True)
        for stale in (ui_db_path, FLUTTER_LOG):
            if os.path.exists(stale):
                os.remove(stale)
        await launch()
        await asyncio.wait_for(body(), timeout=BUILD_TIMEOUT + BUDGET_SECONDS)

        print("\n=== Step 5: Annotate ===")
        subprocess.run(
            [sys.executable, "-I", DOCS_SHOTS, frames_dir, "--out", DOCS_DIR], check=True
        )
        print(f"\n=== ✅ Guide screenshots written under {DOCS_DIR} ===")
    except Exception:
        ui = rendezvous.read_ui()
        print(f"\nLast published: {ui}")
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
        rendezvous.cleanup()
        if os.path.exists(ui_db_path):
            os.remove(ui_db_path)
