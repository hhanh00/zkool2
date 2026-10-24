"""Screenshots of the docs site's MultiSig Accounts / FROST pages, taken from
the real Flutter app on regtest.

The app (integration_test/docs_frost_ui_test.dart) is participant #1 and
the coordinator of a 2-of-3 group whose other members are headless
zkool_graphql instances. It goes through the DKG wizard, shows the group's
account, viewing keys and receive pages and the restore form, plans a
payment, saves it for the other signers, loads it back and coordinates the
signing while participant #2 signs the saved file headless. This side owns
the chain, the funding wallet and the peers: it funds every participant and
the shared address, passes the DKG addresses around, provides the recipient,
checks that the recipient is paid, then draws the annotations with
tests/tools/docs_shots.py and copies the images into the docs site
($ZKOOL_DOCS_DIR/frost/images, default ~/projects/zkool-docs/docs).

Run it with the regtest stack up and the SEED wallet funded (tests/README.md):

    .venv/bin/python -m pytest tests/test_docs_frost_ui.py -v -s
"""

import asyncio
import contextlib
import os
import shutil
import subprocess
import sys

import pytest
from dkg import DkgParticipant, Rendezvous, app_documents_dir, demand_miner, launch_flutter_test
from gql import GraphQLRequest
from gql.transport.exceptions import TransportQueryError
from test_dkg_ui import (
    ADDRESS_QUERY,
    BALANCE_QUERY,
    CREATE_ACCOUNT,
    DKG_SET_ADDRESS,
    DKG_START,
    DO_DKG,
    LWD_URL,
    PAY,
    PEER_COMPLETION_TIMEOUT,
    SYNC_MUTATION,
)
from test_frost_coordinator_ui import FROST_SIGN
from utils import (
    dump_server_log,
    get_current_height,
    kill_existing_zkool_processes,
    mine_blocks,
    wait_for_blocks,
)

DEFAULT_PORT = 8000
PORT_BASE = 8002
# A 2-of-3 group, as in the docs' example: the app is participant #1 and
# coordinates, participant #2 signs with it, participant #3 sits this one out.
N = 3
T = 2
APP_ID = 1
SIGNER_ID = 2
GROUP_NAME = "Treasury"
FLUTTER_LOG = "/tmp/docs_frost_ui_flutter.log"
FRAMES_DIR = "docs_frames_frost"
DOCS_DIR = os.getenv("ZKOOL_DOCS_DIR", os.path.expanduser("~/projects/zkool-docs/docs"))
DOCS_SHOTS = os.path.join(os.path.dirname(__file__), "..", "tools", "docs_shots.py")
BUILD_TIMEOUT = 2400
# Each participant's regular account, which pays for the protocol memos.
PARTICIPANT_FUNDING = "0.01"
SHARED_FUNDING = "0.1"
AMOUNT = "0.05"
EXPECTED = "0.05000000"


@pytest.mark.dkg_ui
@pytest.mark.asyncio
async def test_docs_frost_flutter_ui(rpc_url, seed, zkool_binary):
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

    ui_db_path = os.path.join(docs_dir, "regtest_docs_frost.db")
    frames_dir = os.path.join(docs_dir, FRAMES_DIR)
    rendezvous = Rendezvous(os.path.join(docs_dir, "dkg_ui_rendezvous"))

    wallet = DkgParticipant(DEFAULT_PORT, "/tmp/regtest_docs_frost_funder.db", LWD_URL)
    participants: list[DkgParticipant] = []
    flutter = None
    miner = None
    told: dict = {}

    def tell(**values):
        """Adds [values] to what the app reads from us."""
        told.update(values)
        rendezvous.write_orchestrator(**told)

    async def mine_and_wait(blocks: int):
        client = await participants[0].get_client()
        height = await get_current_height(client)
        await mine_blocks(rpc_url, blocks)
        await wait_for_blocks(client, height, blocks)

    try:
        await kill_existing_zkool_processes()
        rendezvous.setup()
        shutil.rmtree(frames_dir, ignore_errors=True)
        for stale in (ui_db_path, FLUTTER_LOG):
            if os.path.exists(stale):
                os.remove(stale)

        print("=== Step 1: The funding wallet and the headless peers ===")
        wallet.start(zkool_binary)
        await asyncio.sleep(2)
        for i in range(2, N + 1):
            participant = DkgParticipant(PORT_BASE + i - 2, f"/tmp/regtest_docs_frost_{i}.db", LWD_URL, id=i)
            participant.start(zkool_binary)
            participants.append(participant)
            print(f"Started participant {i} on port {participant.port}")
            await asyncio.sleep(2)
        result = await wallet.execute(
            GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": "Main", "key": seed})
        )
        main = int(result["createAccount"])
        await wallet.execute(GraphQLRequest(SYNC_MUTATION, variable_values={"account": main}))

        print("\n=== Step 2: Initialize the DKG on the headless peers ===")
        # The peers' account names never show in the shots; they are the ones
        # DkgParticipant looks up by.
        for participant in participants:
            result = await participant.execute(
                GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": "DKG-Fund", "key": ""})
            )
            participant.funding_account = int(result["createAccount"])
            result = await participant.execute(
                GraphQLRequest(ADDRESS_QUERY, variable_values={"account": participant.funding_account})
            )
            participant.funding_address = result["addressByAccount"]["ironwood"]
            result = await participant.execute(
                GraphQLRequest(
                    DKG_START,
                    variable_values={
                        "name": f"Dkg-Test-Docs-{participant.id}",
                        "t": T,
                        "n": N,
                        "funding": participant.funding_account,
                        "id": participant.id,
                    },
                )
            )
            participant.dkg_address = result["dkgStart"]

        print("\n=== Step 3: Launch the app ===")
        rendezvous.write_config(
            db_path=ui_db_path,
            lwd=LWD_URL,
            frames_dir=FRAMES_DIR,
            n=N,
            t=T,
            my_id=APP_ID,
            name=GROUP_NAME,
            participant_funding=PARTICIPANT_FUNDING,
            shared_funding=SHARED_FUNDING,
            amount=AMOUNT,
        )
        flutter = await launch_flutter_test("docs_frost_ui_test.dart", rendezvous, FLUTTER_LOG)

        print("\n=== Step 4: Fund every participant ===")
        app_funding_address = await rendezvous.poll_ui("funding_address", timeout=BUILD_TIMEOUT, process=flutter)
        recipients = [{"address": app_funding_address, "amount": PARTICIPANT_FUNDING}]
        recipients += [{"address": p.funding_address, "amount": PARTICIPANT_FUNDING} for p in participants]
        await wallet.execute(GraphQLRequest(PAY, variable_values={"account": main, "recipients": recipients}))
        await mine_and_wait(5)
        for participant in participants:
            await participant.execute(
                GraphQLRequest(SYNC_MUTATION, variable_values={"account": participant.funding_account})
            )

        print("\n=== Step 5: Exchange the DKG addresses and run the DKG ===")
        app_dkg_address = await rendezvous.poll_ui("dkg_address", timeout=900, process=flutter)
        all_addresses = {APP_ID: app_dkg_address}
        all_addresses.update({p.id: p.dkg_address for p in participants})
        for participant in participants:
            for other_id, address in all_addresses.items():
                if other_id != participant.id:
                    await participant.execute(
                        GraphQLRequest(DKG_SET_ADDRESS, variable_values={"id": other_id, "address": address})
                    )
        rendezvous.write_peers({p.id: p.dkg_address for p in participants})
        miner = asyncio.create_task(demand_miner(rpc_url))
        for participant in participants:
            await participant.execute(GraphQLRequest(DO_DKG))

        shared_address = await rendezvous.poll_ui("shared_address", timeout=1800, process=flutter)
        print(f"Shared FROST address: {shared_address}")
        elapsed = 0
        while elapsed < PEER_COMPLETION_TIMEOUT:
            if all(p.get_frost_account_id() for p in participants):
                break
            await asyncio.sleep(10)
            elapsed += 10
        else:
            pytest.fail("the headless peers did not complete the DKG")
        for participant in participants:
            participant.frost_account = participant.get_frost_account_id()
            result = await participant.execute(
                GraphQLRequest(ADDRESS_QUERY, variable_values={"account": participant.frost_account})
            )
            assert result["addressByAccount"]["ironwood"] == shared_address

        print("\n=== Step 6: Fund the shared address; a recipient for the payment ===")
        await wallet.execute(GraphQLRequest(SYNC_MUTATION, variable_values={"account": main}))
        await wallet.execute(
            GraphQLRequest(
                PAY,
                variable_values={
                    "account": main,
                    "recipients": [{"address": shared_address, "amount": SHARED_FUNDING}],
                },
            )
        )
        await mine_and_wait(5)
        result = await wallet.execute(
            GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": "Recipient", "key": ""})
        )
        recipient = int(result["createAccount"])
        result = await wallet.execute(GraphQLRequest(ADDRESS_QUERY, variable_values={"account": recipient}))
        tell(receiver=result["addressByAccount"]["ironwood"])

        print("\n=== Step 7: The app's file, signed by participant 2 ===")
        pczt_file = await rendezvous.poll_ui("pczt_file", timeout=1800, process=flutter)
        with open(pczt_file, "rb") as f:
            pczt = f.read().hex()
        print(f"Unsigned transaction saved by the app ({len(pczt) // 2} bytes)")
        await rendezvous.poll_ui("signing", timeout=900, process=flutter)
        signer = next(p for p in participants if p.id == SIGNER_ID)
        await signer.execute(GraphQLRequest(SYNC_MUTATION, variable_values={"account": signer.frost_account}))
        result = await signer.execute(
            GraphQLRequest(
                FROST_SIGN,
                variable_values={
                    "account": signer.frost_account,
                    "coordinator": APP_ID,
                    "funding": signer.get_funding_account_id(),
                    "pczt": pczt,
                },
            )
        )
        assert result["frostSign"], f"participant {signer.id} failed to sign"
        print(f"Participant {signer.id} signed")

        print("\n=== Step 8: The transaction, and the recipient paid ===")
        status = await rendezvous.poll_ui("signing_status", timeout=1800, process=flutter)
        print(f"App: {status}")
        elapsed, balance = 0, "0"
        while elapsed < 600:
            try:
                await wallet.execute(GraphQLRequest(SYNC_MUTATION, variable_values={"account": recipient}))
            except TransportQueryError as e:
                if "already running" not in str(e):
                    raise
            result = await wallet.execute(GraphQLRequest(BALANCE_QUERY, variable_values={"account": recipient}))
            balance = result["balanceByAccount"]["ironwood"]
            print(f"Recipient balance: {balance} (want {EXPECTED})")
            if balance == EXPECTED:
                break
            await asyncio.sleep(10)
            elapsed += 10
        else:
            pytest.fail(f"recipient balance {balance}, expected {EXPECTED}")
        tell(paid=True)

        await rendezvous.poll_ui("done", timeout=900, process=flutter)
        returncode = await asyncio.wait_for(flutter.wait(), timeout=300)
        assert returncode == 0, f"flutter test failed with {returncode}, see {FLUTTER_LOG}"

        print("\n=== Step 9: Annotate ===")
        subprocess.run([sys.executable, "-I", DOCS_SHOTS, frames_dir, "--out", DOCS_DIR], check=True)
        print(f"\n=== ✅ FROST screenshots written under {DOCS_DIR} ===")
    except Exception:
        ui = rendezvous.read_ui()
        print(f"\nLast published: {ui}")
        dump_server_log(FLUTTER_LOG, "FLUTTER TEST LOG")
        dump_server_log(f"/tmp/graphql_{DEFAULT_PORT}.log", "FUNDING WALLET")
        for participant in participants:
            dump_server_log(f"/tmp/graphql_{participant.port}.log", f"PARTICIPANT {participant.id}")
        raise
    finally:
        if miner is not None:
            miner.cancel()
            await asyncio.gather(miner, return_exceptions=True)
        if flutter is not None and flutter.returncode is None:
            flutter.terminate()
            with contextlib.suppress(asyncio.TimeoutError):
                await asyncio.wait_for(flutter.wait(), timeout=30)
        for participant in participants:
            await participant.stop()
        await wallet.stop()
        rendezvous.cleanup()
        if os.path.exists(ui_db_path):
            os.remove(ui_db_path)
