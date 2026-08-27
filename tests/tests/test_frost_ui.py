"""FROST signing where one signer is the real Flutter app.

Extends test_dkg_ui.py: the app (participant #1) runs the DKG wizard to get a
shared key, then drives the signing pages while participants #2 and #3 stay
headless. Participant #2 coordinates, mirroring test_frost.py.

The shared PCZT crosses the app/zkool_graphql boundary, which only works
because both now serialize it with bincode `standard()` (rust/src/api/pay.rs).
"""

import asyncio
import contextlib
import os
import shutil

import pytest
from dkg import DkgParticipant, Rendezvous, app_documents_dir, demand_miner, flutter_device
from gql import GraphQLRequest, gql
from test_dkg_ui import (
    ADDRESS_QUERY,
    BALANCE_QUERY,
    CREATE_ACCOUNT,
    DKG_NAME_PREFIX,
    DKG_SET_ADDRESS,
    DKG_START,
    DO_DKG,
    LWD_URL,
    PAY,
    PEER_COMPLETION_TIMEOUT,
    SYNC_MUTATION,
    N,
    T,
    repo_root,
)
from utils import (
    dump_server_log,
    get_current_height,
    kill_existing_zkool_processes,
    mine_blocks,
    wait_for_blocks,
)

DEFAULT_PORT = 8000
PORT_BASE = 8002
COORDINATOR_ID = 2
FLUTTER_LOG = "/tmp/frost_ui_flutter.log"
SHARED_FUNDING = "0.1"
SEND_AMOUNT = "0.05"
EXPECTED_RECEIVED = "0.05000000"

PREPARE_SEND = gql(
    """
    query ($account: Int!, $address: String!, $amount: BigDecimal!) {
        prepareSend(
            idAccount: $account
            payment: {recipients: [{address: $address, amount: $amount}]}
        )
    }
    """
)

FROST_SIGN = gql(
    """
    mutation ($account: Int!, $coordinator: Int!, $funding: Int!, $pczt: String!) {
        frostSign(
            idAccount: $account
            idCoordinator: $coordinator
            messageAccount: $funding
            pczt: $pczt
        )
    }
    """
)


@pytest.mark.dkg_ui
@pytest.mark.asyncio
async def test_frost_sign_flutter_ui(graphql_url, rpc_url, seed, zkool_binary, gql_client_factory):
    """3-of-3 FROST signing with the Flutter app as a signer."""
    if not seed:
        pytest.skip("SEED not set")
    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    docs_dir = app_documents_dir()
    if not os.path.isdir(docs_dir):
        pytest.skip(f"app documents directory {docs_dir} not found")
    if shutil.which(os.getenv("FLUTTER_BIN", "flutter")) is None:
        pytest.skip("flutter not on PATH")

    ui_db_path = os.path.join(docs_dir, "regtest_frost_ui.db")
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

        print("=== Step 1: Start the funding instance and the headless peers ===")
        default_participant = DkgParticipant(
            DEFAULT_PORT, "/tmp/regtest_frost_ui_default.db", LWD_URL
        )
        default_participant.start(zkool_binary)
        await asyncio.sleep(2)

        for i in range(2, N + 1):
            participant = DkgParticipant(
                PORT_BASE + i - 2, f"/tmp/regtest_frost_ui_{i}.db", LWD_URL, id=i
            )
            participant.start(zkool_binary)
            participants.append(participant)
            print(f"Started participant {i} on port {participant.port}")
            await asyncio.sleep(2)

        print("\n=== Step 2: Create the funded wallet ===")
        async with gql_client_factory(graphql_url) as client:
            result = await client.execute_async(
                GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": "Main", "key": seed})
            )
            main_wallet = int(result["createAccount"])
            await client.execute_async(
                GraphQLRequest(SYNC_MUTATION, variable_values={"account": main_wallet})
            )

        print("\n=== Step 3: Initialize DKG on the headless peers ===")
        for participant in participants:
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
                        "name": f"{DKG_NAME_PREFIX}-{participant.id}",
                        "t": T,
                        "n": N,
                        "funding": participant.funding_account,
                        "id": participant.id,
                    },
                )
            )
            participant.dkg_address = result["dkgStart"]

        print("\n=== Step 4: Launch the Flutter integration test ===")
        rendezvous.write_config(
            db_path=ui_db_path,
            lwd=LWD_URL,
            n=N,
            t=T,
            my_id=1,
            coordinator=COORDINATOR_ID,
            name=f"{DKG_NAME_PREFIX}-1",
        )
        flutter_log = open(FLUTTER_LOG, "w")
        flutter = await asyncio.create_subprocess_exec(
            os.getenv("FLUTTER_BIN", "flutter"),
            "test",
            "integration_test/frost_ui_test.dart",
            "-d",
            flutter_device(),
            f"--dart-define=ZKOOL_TEST_RENDEZVOUS={rendezvous.dir}",
            cwd=repo_root(),
            stdout=flutter_log,
            stderr=asyncio.subprocess.STDOUT,
        )
        print(f"flutter test pid {flutter.pid}, log {FLUTTER_LOG}")

        print("\n=== Step 5: Fund every participant ===")
        ui_funding_address = await rendezvous.poll_ui(
            "funding_address", timeout=2400, process=flutter
        )
        recipients = [{"address": ui_funding_address, "amount": "0.01"}]
        recipients += [{"address": p.funding_address, "amount": "0.01"} for p in participants]
        async with gql_client_factory(graphql_url) as client:
            await client.execute_async(
                GraphQLRequest(
                    PAY, variable_values={"account": main_wallet, "recipients": recipients}
                )
            )

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

        print("\n=== Step 6: Exchange DKG addresses and run the DKG ===")
        ui_dkg_address = await rendezvous.poll_ui("dkg_address", timeout=900, process=flutter)
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
                GraphQLRequest(
                    ADDRESS_QUERY, variable_values={"account": participant.frost_account}
                )
            )
            assert result["addressByAccount"]["ironwood"] == shared_address

        print("\n=== Step 7: Fund the shared FROST address ===")
        async with gql_client_factory(graphql_url) as client:
            await client.execute_async(
                GraphQLRequest(SYNC_MUTATION, variable_values={"account": main_wallet})
            )
            await client.execute_async(
                GraphQLRequest(
                    PAY,
                    variable_values={
                        "account": main_wallet,
                        "recipients": [{"address": shared_address, "amount": SHARED_FUNDING}],
                    },
                )
            )
        height = await get_current_height(peer_client)
        await mine_blocks(rpc_url, 5)
        await wait_for_blocks(peer_client, height, 5)

        coordinator = next(p for p in participants if p.id == COORDINATOR_ID)
        await coordinator.execute(
            GraphQLRequest(
                SYNC_MUTATION, variable_values={"account": coordinator.frost_account}
            )
        )
        result = await coordinator.execute(
            GraphQLRequest(
                BALANCE_QUERY, variable_values={"account": coordinator.frost_account}
            )
        )
        print(f"Shared account balance: {result['balanceByAccount']['ironwood']}")

        print("\n=== Step 8: Coordinator prepares the payment ===")
        result = await coordinator.execute(
            GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": "FROST-Receiver", "key": ""})
        )
        receiver_account = int(result["createAccount"])
        result = await coordinator.execute(
            GraphQLRequest(ADDRESS_QUERY, variable_values={"account": receiver_account})
        )
        receiver_address = result["addressByAccount"]["ironwood"]

        result = await coordinator.execute(
            GraphQLRequest(
                PREPARE_SEND,
                variable_values={
                    "account": coordinator.frost_account,
                    "address": receiver_address,
                    "amount": SEND_AMOUNT,
                },
            )
        )
        pczt = result["prepareSend"]
        print(f"PCZT prepared ({len(pczt) // 2} bytes)")

        # Hand the same PCZT to the app; it decodes with unpackTransaction.
        rendezvous.write_orchestrator(pczt=pczt)

        print("\n=== Step 9: Sign on the headless participants ===")
        for participant in participants:
            funding_account = participant.get_funding_account_id()
            result = await participant.execute(
                GraphQLRequest(
                    FROST_SIGN,
                    variable_values={
                        "account": participant.frost_account,
                        "coordinator": COORDINATOR_ID,
                        "funding": funding_account,
                        "pczt": pczt,
                    },
                )
            )
            assert result["frostSign"], f"participant {participant.id} failed to sign"
            print(f"Participant {participant.id} signed")

        print("\n=== Step 10: Wait for the app to report signing complete ===")
        await rendezvous.poll_ui("signing_status", timeout=1800, process=flutter)
        returncode = await asyncio.wait_for(flutter.wait(), timeout=300)
        assert returncode == 0, f"flutter test failed with {returncode}, see {FLUTTER_LOG}"

        print("\n=== Step 11: Verify the receiver was paid ===")
        elapsed = 0
        receiver_balance = "0"
        while elapsed < 600:
            await coordinator.execute(
                GraphQLRequest(SYNC_MUTATION, variable_values={"account": receiver_account})
            )
            result = await coordinator.execute(
                GraphQLRequest(BALANCE_QUERY, variable_values={"account": receiver_account})
            )
            receiver_balance = result["balanceByAccount"]["ironwood"]
            print(f"Receiver balance: {receiver_balance} (want {EXPECTED_RECEIVED})")
            if receiver_balance == EXPECTED_RECEIVED:
                break
            await asyncio.sleep(10)
            elapsed += 10
        else:
            pytest.fail(
                f"transaction did not land; receiver balance {receiver_balance}, "
                f"expected {EXPECTED_RECEIVED}"
            )

        print("\n=== ✅ FROST UI signing test passed ===")
        print(f"Shared address: {shared_address}")
        print(f"Receiver received {receiver_balance} from the multisig")

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
