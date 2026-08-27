"""FROST DKG where participant #1 is the real Flutter app.

Same protocol as test_dkg.py, but instead of three headless zkool_graphql
instances, participant #1 is the macOS app driven through its DKG pages by
`integration_test/dkg_ui_test.dart`. This test owns the chain (funding, block
mining) and the two headless peers; it talks to the app through a JSON file
rendezvous inside the app's sandbox container.

Requires an interactive GUI session — `flutter test -d macos` opens a window.
The first run also builds the app in debug, which takes several minutes.
"""

import asyncio
import contextlib
import os
import shutil

import pytest
from dkg import DkgParticipant, Rendezvous, app_documents_dir, demand_miner, flutter_device
from gql import GraphQLRequest, gql
from utils import (
    dump_server_log,
    get_current_height,
    kill_existing_zkool_processes,
    mine_blocks,
    wait_for_blocks,
)

N = 3
T = 3
DEFAULT_PORT = 8000
PORT_BASE = 8002  # peers are participants #2 and #3
LWD_URL = "http://localhost:8137"
DKG_NAME_PREFIX = "Dkg-Test-UI"
FLUTTER_LOG = "/tmp/dkg_ui_flutter.log"
# How long the headless peers get to finish after the UI participant is done.
PEER_COMPLETION_TIMEOUT = 600

CREATE_ACCOUNT = gql(
    """
    mutation ($name: String!, $key: String!) {
        createAccount(newAccount: {
            name: $name
            key: $key
            aindex: 0
            useInternal: false
            birth: 1
        })
    }
    """
)

ADDRESS_QUERY = gql(
    """
    query ($account: Int!) {
        addressByAccount(idAccount: $account) { ironwood }
    }
    """
)

BALANCE_QUERY = gql(
    """
    query ($account: Int!) {
        balanceByAccount(idAccount: $account) { ironwood }
    }
    """
)

SYNC_MUTATION = gql(
    """
    mutation ($account: Int!) {
        synchronizeAccount(idAccount: $account)
    }
    """
)

DKG_START = gql(
    """
    mutation ($name: String!, $t: Int!, $n: Int!, $funding: Int!, $id: Int!) {
        dkgStart(
            name: $name
            threshold: $t
            participants: $n
            messageAccount: $funding
            idParticipant: $id
        )
    }
    """
)

DKG_SET_ADDRESS = gql(
    """
    mutation ($id: Int!, $address: String!) {
        dkgSetAddress(idParticipant: $id, address: $address)
    }
    """
)

DO_DKG = gql("mutation { doDkg }")

PAY = gql(
    """
    mutation ($account: Int!, $recipients: [Recipient!]!) {
        pay(idAccount: $account, payment: {recipients: $recipients})
    }
    """
)


def repo_root() -> str:
    return os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))


@pytest.mark.dkg_ui
@pytest.mark.asyncio
async def test_dkg_3_of_3_flutter_ui(graphql_url, rpc_url, seed, zkool_binary, gql_client_factory):
    """3-out-of-3 DKG with the Flutter app as participant #1."""
    if not seed:
        pytest.skip("SEED not set")
    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    docs_dir = app_documents_dir()
    if not os.path.isdir(docs_dir):
        pytest.skip(
            f"app documents directory {docs_dir} not found — "
            "launch the app once before running this test"
        )
    if shutil.which(os.getenv("FLUTTER_BIN", "flutter")) is None:
        pytest.skip("flutter not on PATH")

    # The db filename must contain "regtest": that is what selects
    # Network::Regtest in rust/src/api/coin.rs.
    ui_db_path = os.path.join(docs_dir, "regtest_dkg_ui.db")
    rendezvous = Rendezvous(os.path.join(docs_dir, "dkg_ui_rendezvous"))

    participants: list[DkgParticipant] = []
    default_participant = None
    flutter = None
    miner = None

    try:
        await kill_existing_zkool_processes()
        rendezvous.setup()
        for stale in (ui_db_path, FLUTTER_LOG):
            if os.path.exists(stale):
                os.remove(stale)

        print("=== Step 1: Start the default (funding) instance and the headless peers ===")
        default_participant = DkgParticipant(
            DEFAULT_PORT, "/tmp/regtest_dkg_ui_default.db", LWD_URL
        )
        default_participant.start(zkool_binary)
        await asyncio.sleep(2)

        for i in range(2, N + 1):
            port = PORT_BASE + i - 2
            participant = DkgParticipant(port, f"/tmp/regtest_dkg_ui_{i}.db", LWD_URL, id=i)
            participant.start(zkool_binary)
            participants.append(participant)
            print(f"Started participant {i} on port {port}")
            await asyncio.sleep(2)

        print("\n=== Step 2: Create the funded wallet on the default instance ===")
        async with gql_client_factory(graphql_url) as client:
            result = await client.execute_async(
                GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": "Main", "key": seed})
            )
            main_wallet = int(result["createAccount"])
            await client.execute_async(
                GraphQLRequest(SYNC_MUTATION, variable_values={"account": main_wallet})
            )
            result = await client.execute_async(
                GraphQLRequest(BALANCE_QUERY, variable_values={"account": main_wallet})
            )
            print(f"Funding wallet {main_wallet} balance: {result['balanceByAccount']['ironwood']}")

        print("\n=== Step 3: Initialize DKG on the headless peers ===")
        for participant in participants:
            i = participant.id
            result = await participant.execute(
                GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": "DKG-Fund", "key": ""})
            )
            participant.funding_account = int(result["createAccount"])
            result = await participant.execute(
                GraphQLRequest(
                    ADDRESS_QUERY, variable_values={"account": participant.funding_account}
                )
            )
            participant.funding_address = result["addressByAccount"]["ironwood"]
            result = await participant.execute(
                GraphQLRequest(
                    DKG_START,
                    variable_values={
                        "name": f"{DKG_NAME_PREFIX}-{i}",
                        "t": T,
                        "n": N,
                        "funding": participant.funding_account,
                        "id": i,
                    },
                )
            )
            participant.dkg_address = result["dkgStart"]
            print(f"Participant {i} funding {participant.funding_address}")
            print(f"Participant {i} DKG address {participant.dkg_address}")

        print("\n=== Step 4: Launch the Flutter integration test ===")
        rendezvous.write_config(
            db_path=ui_db_path,
            lwd=LWD_URL,
            n=N,
            t=T,
            my_id=1,
            name=f"{DKG_NAME_PREFIX}-1",
        )
        flutter_log = open(FLUTTER_LOG, "w")
        flutter = await asyncio.create_subprocess_exec(
            os.getenv("FLUTTER_BIN", "flutter"),
            "test",
            "integration_test/dkg_ui_test.dart",
            "-d",
            flutter_device(),
            f"--dart-define=ZKOOL_TEST_RENDEZVOUS={rendezvous.dir}",
            cwd=repo_root(),
            stdout=flutter_log,
            stderr=asyncio.subprocess.STDOUT,
        )
        print(f"flutter test pid {flutter.pid}, log {FLUTTER_LOG}")

        print("\n=== Step 5: Wait for the UI participant's funding address ===")
        # Generous: the first run builds the macOS app and the Rust staticlib.
        ui_funding_address = await rendezvous.poll_ui(
            "funding_address", timeout=2400, process=flutter
        )
        print(f"UI funding address {ui_funding_address}")

        print("\n=== Step 6: Fund every participant ===")
        recipients = [{"address": ui_funding_address, "amount": "0.01"}]
        recipients += [{"address": p.funding_address, "amount": "0.01"} for p in participants]
        async with gql_client_factory(graphql_url) as client:
            result = await client.execute_async(
                GraphQLRequest(
                    PAY, variable_values={"account": main_wallet, "recipients": recipients}
                )
            )
            print(f"Funding transaction: {result['pay']}")

        print("\n=== Step 7: Mine and synchronize the peers ===")
        peer_client = await participants[0].get_client()
        height = await get_current_height(peer_client)
        await mine_blocks(rpc_url, 5)
        await wait_for_blocks(peer_client, height, 5)
        for participant in participants:
            await participant.execute(
                GraphQLRequest(
                    SYNC_MUTATION, variable_values={"account": participant.funding_account}
                )
            )
            result = await participant.execute(
                GraphQLRequest(
                    BALANCE_QUERY, variable_values={"account": participant.funding_account}
                )
            )
            balance = result["balanceByAccount"]["ironwood"]
            print(f"Participant {participant.id} funding balance: {balance}")
            assert balance and balance != "0", f"Participant {participant.id} was not funded"

        print("\n=== Step 8: Exchange DKG addresses ===")
        ui_dkg_address = await rendezvous.poll_ui("dkg_address", timeout=900, process=flutter)
        print(f"UI DKG address {ui_dkg_address}")

        all_addresses = {1: ui_dkg_address}
        all_addresses.update({p.id: p.dkg_address for p in participants})
        for participant in participants:
            for other_id, address in all_addresses.items():
                if other_id == participant.id:
                    continue
                await participant.execute(
                    GraphQLRequest(
                        DKG_SET_ADDRESS, variable_values={"id": other_id, "address": address}
                    )
                )
            print(f"Participant {participant.id} knows every peer address")

        rendezvous.write_peers({p.id: p.dkg_address for p in participants})

        print("\n=== Step 9: Run the DKG ===")
        miner = asyncio.create_task(demand_miner(rpc_url))
        for participant in participants:
            await participant.execute(GraphQLRequest(DO_DKG))
            print(f"Initiated DKG on participant {participant.id}")

        shared_address = await rendezvous.poll_ui("shared_address", timeout=1800, process=flutter)
        print(f"UI shared address: {shared_address}")

        print("\n=== Step 10: Wait for the Flutter test to finish ===")
        returncode = await asyncio.wait_for(flutter.wait(), timeout=300)
        assert returncode == 0, f"flutter test failed with {returncode}, see {FLUTTER_LOG}"

        print("\n=== Step 11: Wait for the headless peers to finish their rounds ===")
        # The UI participant can reach the shared address a round ahead of the
        # peers: they still need its round-2 packages to be mined and synced.
        # The background miner is still running, so just wait them out.
        elapsed = 0
        while elapsed < PEER_COMPLETION_TIMEOUT:
            if all(p.get_frost_account_id() for p in participants):
                break
            await asyncio.sleep(10)
            elapsed += 10
        else:
            pending = [p.id for p in participants if not p.get_frost_account_id()]
            pytest.fail(
                f"participants {pending} did not complete the DKG within "
                f"{PEER_COMPLETION_TIMEOUT}s of the UI participant finishing"
            )

        print("\n=== Step 12: Verify every participant derived the same shared address ===")
        for participant in participants:
            participant.frost_account = participant.get_frost_account_id()
            assert participant.frost_account, f"No FROST account for participant {participant.id}"
            result = await participant.execute(
                GraphQLRequest(
                    ADDRESS_QUERY, variable_values={"account": participant.frost_account}
                )
            )
            peer_address = result["addressByAccount"]["ironwood"]
            print(f"Participant {participant.id} shared address: {peer_address}")
            assert peer_address == shared_address, (
                f"Participant {participant.id} derived a different shared address! "
                f"{peer_address} != {shared_address}"
            )

        print("\n=== ✅ Flutter UI DKG Test Passed! ===")
        print(f"Shared FROST address: {shared_address}")

    except Exception:
        dump_server_log(FLUTTER_LOG, "FLUTTER TEST LOG")
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
        if default_participant:
            await default_participant.stop()
        rendezvous.cleanup()
        if os.path.exists(ui_db_path):
            os.remove(ui_db_path)
