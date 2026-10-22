"""Screenshots of the docs site's Recipe pages, taken from the real Flutter
app on regtest.

The app (integration_test/docs_recipe_ui_test.dart) walks the screens the
Recipe pages show: the account list and its menu, the mempool, the account
menu and viewing keys, the edit page, the receive page, the notes tab, the
payment URI, the new account page with the Ledger options, the settings,
the database manager, and the cold wallet round trip between a watch-only
account and the account with the keys. This side owns the chain and the
funding wallet: it pays the app's account so the pages have data, holds
one payment unmined while the app shoots the mempool, mines on demand, then
draws the annotations with tests/tools/docs_shots.py and copies the images
into the docs site ($ZKOOL_DOCS_DIR/recipe/images, default
~/projects/zkool-docs/docs).

Run it with the regtest stack up and the SEED wallet funded (tests/README.md):

    .venv/bin/python -m pytest tests/test_docs_recipe_ui.py -v -s
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
from gql.transport.exceptions import TransportQueryError
from test_frost_upgrade import SYNC, wait_ready
from test_key_import import create
from test_uivk import ADDRESSES, BALANCE, PAY
from utils import dump_server_log, kill_existing_zkool_processes

DEFAULT_PORT = 8000
LWD_URL = "http://localhost:8137"
FLUTTER_LOG = "/tmp/docs_recipe_ui_flutter.log"
FRAMES_DIR = "docs_frames_recipe"
DOCS_DIR = os.getenv("ZKOOL_DOCS_DIR", os.path.expanduser("~/projects/zkool-docs/docs"))
DOCS_SHOTS = os.path.join(os.path.dirname(__file__), "..", "tools", "docs_shots.py")
BUILD_TIMEOUT = 2400
BUDGET_SECONDS = 1200
# What the app's account receives up front: amounts and memos.
FUNDING = [("0.3", "Welcome to Zkool"), ("0.15", None), ("0.05", "Coffee money")]
# The payment left in the mempool for its screenshot.
MEMPOOL_AMOUNT = "0.02"


async def rpc(rpc_url: str, method: str, params: list):
    payload = {"jsonrpc": "1.0", "id": method, "method": method, "params": params}
    async with httpx.AsyncClient() as client:
        response = await client.post(rpc_url, json=payload)
        response.raise_for_status()
        return response.json()


async def wait_mined(rpc_url: str, txid: str):
    """Waits for the demand miner to have mined `txid`: the next payment
    from the same wallet would otherwise reuse its inputs and be rejected."""
    for _ in range(60):
        result = await rpc(rpc_url, "getrawtransaction", [txid, 1])
        if result.get("error") is None and result["result"].get("height", -1) > 0:
            return
        await asyncio.sleep(2)
    pytest.fail(f"{txid} was not mined")


@pytest.mark.dkg_ui
@pytest.mark.asyncio
async def test_docs_recipe_flutter_ui(rpc_url, seed, zkool_binary):
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

    ui_db_path = os.path.join(docs_dir, "regtest_docs.db")
    frames_dir = os.path.join(docs_dir, FRAMES_DIR)
    rendezvous = Rendezvous(os.path.join(docs_dir, "dkg_ui_rendezvous"))

    wallet = DkgParticipant(DEFAULT_PORT, "/tmp/regtest_docs_recipe_funder.db", LWD_URL)
    flutter = None
    miner = None
    main = None

    def start_miner():
        return asyncio.create_task(demand_miner(rpc_url, idle_fallback=3600))

    async def stop_miner():
        nonlocal miner
        if miner is not None:
            miner.cancel()
            await asyncio.gather(miner, return_exceptions=True)
            miner = None

    async def pay(amount: str, memo: str | None, address: str) -> str:
        """Pays from the funding wallet and returns the txid.

        While the app keeps a mempool stream open, zebra answers the
        submission with a spend conflict against the very transaction it
        just accepted, so the wallet reports a failure and keeps the notes
        unlocked. The mempool is empty before every payment here, so the
        one transaction it then holds is this payment.
        """
        recipient = {"address": address, "amount": amount}
        if memo:
            recipient["memo"] = memo
        try:
            result = await wallet.execute(
                GraphQLRequest(PAY, variable_values={"account": main, "recipients": [recipient]})
            )
            txid = result["pay"]
        except TransportQueryError as e:
            if "same effects" not in str(e):
                raise
            for _ in range(10):
                pending = await get_raw_mempool(rpc_url)
                if pending:
                    break
                await asyncio.sleep(1)
            else:
                raise
            txid = pending[0]
            print("zebra reported a spend conflict but holds the payment")
        print(f"Paid {amount} to {address[:20]}… ({txid[:12]}…)")
        return txid

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
        needed = sum(float(amount) for amount, _ in FUNDING) + float(MEMPOOL_AMOUNT) + 0.01
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
            fund=str(sum(float(amount) for amount, _ in FUNDING) + float(MEMPOOL_AMOUNT)),
        )
        flutter = await launch_flutter_test("docs_recipe_ui_test.dart", rendezvous, FLUTTER_LOG)

    async def body():
        nonlocal miner
        print("\n=== Step 3: A payment held in the mempool ===")
        a_ua = await rendezvous.poll_ui("a_ua", timeout=BUILD_TIMEOUT, process=flutter)
        await rendezvous.poll_ui("mempool_ready", timeout=BUDGET_SECONDS, process=flutter)
        # Nothing mines until the app has shot the mempool page. The app's
        # account has no history yet, so the row shows no amount: once it
        # has synced, its mempool stream no longer reports anything on
        # regtest, which is not understood.
        txid = await pay(MEMPOOL_AMOUNT, "Still in the mempool", a_ua)
        await rendezvous.poll_ui("mempool_done", timeout=BUDGET_SECONDS, process=flutter)
        miner = start_miner()
        await wait_mined(rpc_url, txid)
        await wallet.execute(GraphQLRequest(SYNC, variable_values={"account": main}))

        print("\n=== Step 4: Fund the account ===")
        for amount, memo in FUNDING:
            txid = await pay(amount, memo, a_ua)
            # Each payment is mined before the next, so they never contend
            # for the same notes.
            await wait_mined(rpc_url, txid)
            await wallet.execute(GraphQLRequest(SYNC, variable_values={"account": main}))

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
        print(f"\n=== ✅ Recipe screenshots written under {DOCS_DIR} ===")
    except Exception:
        ui = rendezvous.read_ui()
        print(f"\nLast published: {ui}")
        dump_server_log(FLUTTER_LOG, "FLUTTER TEST LOG")
        dump_server_log(f"/tmp/graphql_{DEFAULT_PORT}.log", "FUNDING WALLET")
        raise
    finally:
        await stop_miner()
        if flutter is not None and flutter.returncode is None:
            flutter.terminate()
            with contextlib.suppress(asyncio.TimeoutError):
                await asyncio.wait_for(flutter.wait(), timeout=30)
        await wallet.stop()
        rendezvous.cleanup()
        if os.path.exists(ui_db_path):
            os.remove(ui_db_path)
