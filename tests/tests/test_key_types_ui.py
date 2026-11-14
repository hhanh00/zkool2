"""Every key type other than a seed, in the real app.

The app (integration_test/key_types_ui_test.dart) imports each key, syncs, and
does what the key allows. This side derives the full and incoming viewing keys
from a fixed mnemonic, funds what they all watch, mines on demand, and checks
each payment the app broadcasts on the node. Seeds are covered by
test_wallet_ui.py and the Ledger modes by test_ledger.py.
"""

import asyncio
import contextlib
import os
import shutil
import sys
from decimal import Decimal

import pytest
from dkg import DkgParticipant, Rendezvous, app_documents_dir, demand_miner, launch_flutter_test
from gql import GraphQLRequest
from test_frost_upgrade import SYNC, address, wait_ready
from test_key_import import (
    FUND_IRONWOOD,
    FUND_SAPLING,
    FUND_T0_EACH,
    FUND_T0_UNCOMPRESSED,
    MNEMONIC,
    SAPLING_ADDRESS,
    SAPLING_SK,
    SAPLING_VK,
    SPEND,
    T0_ADDRESS,
    T0_UNCOMPRESSED_ADDRESS,
    WIF_COMPRESSED,
    WIF_UNCOMPRESSED,
    XPRV,
    XPUB_ACCOUNT,
    XPUB_EXTERNAL,
    ZPK,
    create,
)
from test_uivk import addresses, pay, viewing_key
from test_wallet_ui import rpc
from utils import dump_server_log, kill_existing_zkool_processes

DEFAULT_PORT = 8000
LWD_URL = "http://localhost:8137"
FLUTTER_LOG = "/tmp/key_types_ui_flutter.log"
BUILD_TIMEOUT = 2400
BUDGET_SECONDS = 1200
T0_COINS = 3
T0_TOTAL = str(T0_COINS * Decimal(FUND_T0_EACH))


def key_cases(ufvk: str, uivk: str, sapling_ufvk: str, full: dict) -> list[dict]:
    """What each import must show. `minimum` is by pool, in the order
    transparent, Sapling, Orchard, Ironwood. On a long-lived chain the fixed
    keys carry earlier runs' coins too, so these are lower bounds. `backup` is
    the key the backup block shows, the imported one unless the import loses
    part of it."""

    def case(
        name,
        key,
        mode,
        taddr=None,
        saddr=None,
        orchard=False,
        minimum=("0", "0", "0", "0"),
        sync_warning=False,
        backup=None,
    ):
        return {
            "name": name,
            "key": key,
            "mode": mode,
            "taddr": taddr,
            "saddr": saddr,
            "orchard": orchard,
            "minimum": list(minimum),
            "sync_warning": sync_warning,
            "backup": backup or key,
        }

    t0_only = (T0_TOTAL, "0", "0", "0")
    every_pool = (T0_TOTAL, FUND_SAPLING, "0", FUND_IRONWOOD)
    return [
        case("T-xprv", XPRV, "spend", taddr=T0_ADDRESS, minimum=t0_only),
        case("T-wif", WIF_COMPRESSED, "spend", taddr=T0_ADDRESS, minimum=t0_only),
        case(
            "T-wif-u",
            WIF_UNCOMPRESSED,
            "spend",
            taddr=T0_UNCOMPRESSED_ADDRESS,
            minimum=(FUND_T0_UNCOMPRESSED, "0", "0", "0"),
        ),
        # Sent one block behind the tip, so its Sync Warning takes syncIfNeeded
        # through the manual path.
        case(
            "S-sk",
            SAPLING_SK,
            "spend",
            saddr=SAPLING_ADDRESS,
            minimum=("0", FUND_SAPLING, "0", "0"),
            sync_warning=True,
        ),
        case("T-xpub", XPUB_ACCOUNT, "watch", taddr=T0_ADDRESS, minimum=t0_only),
        case("T-zpk", ZPK, "watch", taddr=T0_ADDRESS, minimum=t0_only),
        case(
            "S-vk",
            SAPLING_VK,
            "watch",
            saddr=SAPLING_ADDRESS,
            minimum=("0", FUND_SAPLING, "0", "0"),
            # Only the diversifiable key is stored, so the account backs up as
            # the Sapling-only unified key of the same key.
            backup=sapling_ufvk,
        ),
        case(
            "U-fvk",
            ufvk,
            "watch",
            taddr=full["transparent"],
            saddr=full["sapling"],
            orchard=True,
            minimum=every_pool,
        ),
        case("T-xpub-ext", XPUB_EXTERNAL, "incoming", taddr=T0_ADDRESS, minimum=t0_only),
        case(
            "U-ivk",
            uivk,
            "incoming",
            taddr=full["transparent"],
            saddr=full["sapling"],
            orchard=True,
            minimum=every_pool,
        ),
    ]


@pytest.mark.dkg_ui
@pytest.mark.asyncio
async def test_key_types_flutter_ui(rpc_url, seed, zkool_binary):
    if not seed:
        pytest.skip("SEED not set")
    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")
    docs_dir = app_documents_dir()
    if not os.path.isdir(docs_dir):
        pytest.skip(f"app documents directory {docs_dir} not found")
    if shutil.which(os.getenv("FLUTTER_BIN", "flutter")) is None:
        pytest.skip("flutter not on PATH")

    ui_db_path = os.path.join(docs_dir, "regtest_key_types_ui.db")
    rendezvous = Rendezvous(os.path.join(docs_dir, "dkg_ui_rendezvous"))
    wallet = DkgParticipant(DEFAULT_PORT, "/tmp/regtest_key_types_ui_funder.db", LWD_URL)
    flutter = None
    miner = None

    async def serve_mine_request(request: str):
        """Mine one block when the app asks for `request`."""
        for _ in range(BUDGET_SECONDS):
            if rendezvous.read_ui().get("mine") == request:
                result = await rpc(rpc_url, "generate", [1])
                assert result.get("error") is None, result
                return
            if flutter.returncode is not None:
                pytest.fail(f"flutter test exited with {flutter.returncode}, see {FLUTTER_LOG}")
            await asyncio.sleep(1)
        pytest.fail(f"the app never asked to mine for {request}")

    try:
        await kill_existing_zkool_processes()
        rendezvous.setup()
        for stale in (ui_db_path, FLUTTER_LOG):
            if os.path.exists(stale):
                os.remove(stale)

        print("=== Step 1: Derive the viewing keys and fund what they watch ===")
        wallet.start(zkool_binary)
        await wait_ready(wallet)
        main = await create(wallet, "Main", seed)
        await wallet.execute(GraphQLRequest(SYNC, variable_values={"account": main}))
        full = await create(wallet, "Seed", MNEMONIC)
        full_addresses = await addresses(wallet, full)
        ufvk = await viewing_key(wallet, full)
        uivk = await viewing_key(wallet, full, incoming=True)
        sapling_ufvk = await viewing_key(wallet, full, pools=2)
        await pay(wallet, rpc_url, main, full_addresses["ua"], FUND_IRONWOOD)
        await pay(wallet, rpc_url, main, full_addresses["sapling"], FUND_SAPLING)
        for _ in range(T0_COINS):
            await pay(wallet, rpc_url, main, T0_ADDRESS, FUND_T0_EACH)
        await pay(wallet, rpc_url, main, T0_UNCOMPRESSED_ADDRESS, FUND_T0_UNCOMPRESSED)
        recipient = await address(wallet, main)

        print("\n=== Step 2: Launch the app ===")
        cases = key_cases(ufvk, uivk, sapling_ufvk, full_addresses)
        rendezvous.write_config(
            db_path=ui_db_path,
            lwd=LWD_URL,
            recipient=recipient,
            spend=SPEND,
            cases=cases,
            # The backup block is behind authenticate(): Touch ID on macOS, open on Linux.
            biometric_gate=sys.platform == "darwin",
        )
        flutter = await launch_flutter_test("key_types_ui_test.dart", rendezvous, FLUTTER_LOG)
        await rendezvous.poll_ui("phase", timeout=BUILD_TIMEOUT, process=flutter)

        print("\n=== Step 3: The app imports and syncs every key ===")
        await serve_mine_request("auto-sync")
        # Mine only what the mempool holds from here on, so the spends settle
        # without moving the tip under the app's send pages.
        miner = asyncio.create_task(demand_miner(rpc_url, idle_fallback=3600))

        print("\n=== Step 4: Each spending key's payment must reach the node ===")
        for case in cases:
            if case["mode"] != "spend":
                continue
            if case["sync_warning"]:
                await serve_mine_request(f"sync-warning-{case['name']}")
            txid = await rendezvous.poll_ui(
                f"txid_{case['name']}", timeout=BUDGET_SECONDS, process=flutter
            )
            result = await rpc(rpc_url, "getrawtransaction", [txid, 1])
            assert result.get("error") is None, f"node does not know {txid}: {result['error']}"
            print(f"{case['name']} paid: {txid}")

        print("\n=== Step 5: The app finishes ===")
        await rendezvous.poll_ui("done", timeout=BUDGET_SECONDS, process=flutter)
        returncode = await asyncio.wait_for(flutter.wait(), timeout=300)
        assert returncode == 0, f"flutter test failed with {returncode}, see {FLUTTER_LOG}"
        print(f"\n=== ✅ {len(cases)} key types passed ===")
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
        rendezvous.cleanup()
        if os.path.exists(ui_db_path):
            os.remove(ui_db_path)
