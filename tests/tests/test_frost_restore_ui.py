"""Restore from a backup string where the restored member is the real Flutter app.

Three headless `zkool_graphql` peers form a group and the orchestrator funds
it. Participant #3 exports its `zkfrost` backup string and the group's
viewing key and is stopped. The app restores that member through the
new-account page and then signs as a participant. Two shapes:

- **legacy**: the group is dkg-1. The app shows the not-recoverable banner,
  taps Upgrade (creating the ZIP 2005 sibling), and signs the sweep that
  participant #2 coordinates. The orchestrator checks that every member
  derived the same sibling address and that the sweep landed in the
  sibling's Ironwood pool.
- **ZIP 2005**: the group is dkg-2. The app shows no banner and signs an
  ordinary payment participant #2 coordinates; the orchestrator checks the
  receiver is paid.
"""

import asyncio
import contextlib
import os
import shutil
import sys

import pytest
from dkg import DkgParticipant, Rendezvous, app_documents_dir, demand_miner, launch_flutter_test
from gql import GraphQLRequest, gql
from gql.transport.exceptions import TransportQueryError
from test_dkg_ui import (
    ADDRESS_QUERY,
    BALANCE_QUERY,
    CREATE_ACCOUNT,
    DKG_SET_ADDRESS,
    DO_DKG,
    LWD_URL,
    PAY,
    SYNC_MUTATION,
    N,
    T,
)
from utils import (
    dump_server_log,
    get_current_height,
    kill_existing_zkool_processes,
    mine_blocks,
    wait_for_blocks,
)

DEFAULT_PORT = 8000
PORT_BASE = 8002  # peers are participants #1, #2 and #3
RESTORED_ID = 3
COORDINATOR_ID = 2
DKG_NAME = "Dkg-Test-Restore-UI"  # get_frost_account_id looks for Dkg-Test-%
FLUTTER_LOG = "/tmp/frost_restore_ui_flutter.log"
SHARED_FUNDING = "0.1"
PAYMENT = "0.01"
PAYMENT_RECEIVED = "0.01000000"

DKG_START_VERSIONED = gql(
    """
    mutation ($name: String!, $t: Int!, $n: Int!, $funding: Int!, $id: Int!, $version: Int!) {
        dkgStart(
            name: $name
            threshold: $t
            participants: $n
            messageAccount: $funding
            idParticipant: $id
            dkgVersion: $version
        )
    }
    """
)
UFVK = gql("query ($account: Int!) { unifiedFullViewingKey(idAccount: $account) }")
EXPORT_BACKUP = gql("mutation ($account: Int!) { frostExportBackup(idAccount: $account) }")
FROST_UPGRADE = gql("mutation ($account: Int!) { frostUpgrade(idAccount: $account) }")
FROST_SWEEP = gql(
    """
    mutation ($account: Int!, $coordinator: Int!, $funding: Int!) {
        frostSweep(idAccount: $account, idCoordinator: $coordinator, messageAccount: $funding)
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


async def sync_tolerant(wallet: DkgParticipant, account: int) -> bool:
    try:
        await wallet.execute(GraphQLRequest(SYNC_MUTATION, variable_values={"account": account}))
        return True
    except TransportQueryError as e:
        if "already running" not in str(e):
            raise
        return False


async def sync_until_done(wallet: DkgParticipant, account: int):
    while not await sync_tolerant(wallet, account):
        await asyncio.sleep(2)


async def balance(wallet: DkgParticipant, account: int) -> str:
    result = await wallet.execute(
        GraphQLRequest(BALANCE_QUERY, variable_values={"account": account})
    )
    return result["balanceByAccount"]["ironwood"]


async def address(wallet: DkgParticipant, account: int) -> str:
    result = await wallet.execute(
        GraphQLRequest(ADDRESS_QUERY, variable_values={"account": account})
    )
    return result["addressByAccount"]["ironwood"]


async def mine_and_wait(client, rpc_url: str, blocks: int):
    height = await get_current_height(client)
    await mine_blocks(rpc_url, blocks)
    await wait_for_blocks(client, height, blocks)


async def wait_until(condition, timeout: int, interval: int = 10, failure: str = "timed out"):
    elapsed = 0
    while elapsed < timeout:
        if await condition():
            return
        await asyncio.sleep(interval)
        elapsed += interval
    pytest.fail(failure)


async def _run(dkg_version: int, graphql_url, rpc_url, seed, zkool_binary, gql_client_factory):
    if not seed:
        pytest.skip("SEED not set")
    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    docs_dir = app_documents_dir()
    if not os.path.isdir(docs_dir):
        pytest.skip(f"app documents directory {docs_dir} not found")
    if shutil.which(os.getenv("FLUTTER_BIN", "flutter")) is None:
        pytest.skip("flutter not on PATH")

    legacy = dkg_version < 2
    ui_db_path = os.path.join(docs_dir, "regtest_frost_restore_ui.db")
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
            DEFAULT_PORT, "/tmp/regtest_frost_restore_ui_default.db", LWD_URL
        )
        default_participant.start(zkool_binary)
        await asyncio.sleep(2)
        for i in range(1, N + 1):
            participant = DkgParticipant(
                PORT_BASE + i - 1, f"/tmp/regtest_frost_restore_ui_{i}.db", LWD_URL, id=i
            )
            participant.start(zkool_binary)
            participants.append(participant)
            await asyncio.sleep(2)
        by_id = {p.id: p for p in participants}

        print("\n=== Step 2: Create the funded wallet ===")
        async with gql_client_factory(graphql_url) as client:
            result = await client.execute_async(
                GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": "Main", "key": seed})
            )
            main_wallet = int(result["createAccount"])
            await client.execute_async(
                GraphQLRequest(SYNC_MUTATION, variable_values={"account": main_wallet})
            )

        async def pay_from_main(recipients):
            async with gql_client_factory(graphql_url) as client:
                await client.execute_async(
                    GraphQLRequest(SYNC_MUTATION, variable_values={"account": main_wallet})
                )
                await client.execute_async(
                    GraphQLRequest(
                        PAY, variable_values={"account": main_wallet, "recipients": recipients}
                    )
                )

        print(f"\n=== Step 3: A dkg-{dkg_version} group among the headless peers ===")
        for p in participants:
            result = await p.execute(
                GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": "DKG-Fund", "key": ""})
            )
            p.funding_account = int(result["createAccount"])
            p.funding_address = await address(p, p.funding_account)
            result = await p.execute(
                GraphQLRequest(
                    DKG_START_VERSIONED,
                    variable_values={
                        "name": DKG_NAME,
                        "t": T,
                        "n": N,
                        "funding": p.funding_account,
                        "id": p.id,
                        "version": dkg_version,
                    },
                )
            )
            p.dkg_address = result["dkgStart"]
        await pay_from_main(
            [{"address": p.funding_address, "amount": "0.01"} for p in participants]
        )
        peer_client = await participants[0].get_client()
        await mine_and_wait(peer_client, rpc_url, 5)
        for p in participants:
            await sync_until_done(p, p.funding_account)
        for p in participants:
            for other in participants:
                if other is not p:
                    await p.execute(
                        GraphQLRequest(
                            DKG_SET_ADDRESS,
                            variable_values={"id": other.id, "address": other.dkg_address},
                        )
                    )
        miner = asyncio.create_task(demand_miner(rpc_url))
        for p in participants:
            await p.execute(GraphQLRequest(DO_DKG))

        async def dkg_done():
            return all(p.get_frost_account_id() for p in participants)

        await wait_until(dkg_done, 600, failure="the headless peers did not complete the DKG")
        for p in participants:
            p.frost_account = p.get_frost_account_id()
        shared_addresses = {await address(p, p.frost_account) for p in participants}
        assert len(shared_addresses) == 1, shared_addresses
        shared_address = shared_addresses.pop()
        print(f"Group at {shared_address[:24]}…")

        print("\n=== Step 4: Fund the group's address ===")
        await pay_from_main([{"address": shared_address, "amount": SHARED_FUNDING}])
        await mine_and_wait(peer_client, rpc_url, 5)

        print("\n=== Step 5: Participant 3 exports its backup string, then goes away ===")
        lost = by_id[RESTORED_ID]
        result = await lost.execute(
            GraphQLRequest(EXPORT_BACKUP, variable_values={"account": lost.frost_account})
        )
        backup = result["frostExportBackup"]
        result = await lost.execute(
            GraphQLRequest(UFVK, variable_values={"account": lost.frost_account})
        )
        group_ufvk = result["unifiedFullViewingKey"]
        await lost.stop()
        participants.remove(lost)
        print(f"Backup string {backup[:24]}… ({len(backup)} chars)")

        print("\n=== Step 6: Launch the Flutter test, which restores the member ===")
        rendezvous.write_config(
            db_path=ui_db_path,
            lwd=LWD_URL,
            name=DKG_NAME,
            backup=backup,
            ufvk=group_ufvk,
            dkg_version=dkg_version,
            my_id=RESTORED_ID,
            coordinator=COORDINATOR_ID,
            # The backup string page sits behind biometric authentication,
            # which cannot be answered from a test on a Mac with Touch ID.
            check_backup_page=sys.platform != "darwin",
        )
        flutter = await launch_flutter_test("frost_restore_ui_test.dart", rendezvous, FLUTTER_LOG)

        ui_funding_address = await rendezvous.poll_ui(
            "funding_address", timeout=2400, process=flutter
        )
        await pay_from_main([{"address": ui_funding_address, "amount": "0.01"}])
        await mine_and_wait(peer_client, rpc_url, 5)

        restored_account = await rendezvous.poll_ui("restored", timeout=900, process=flutter)
        print(f"The app restored FROST account {restored_account}")

        coordinator = by_id[COORDINATOR_ID]
        await sync_until_done(coordinator, coordinator.frost_account)

        if legacy:
            print("\n=== Step 7: The app upgrades; the peers upgrade to the same address ===")
            ui_sibling_address = await rendezvous.poll_ui(
                "sibling_address", timeout=1200, process=flutter
            )
            for p in participants:
                result = await p.execute(
                    GraphQLRequest(FROST_UPGRADE, variable_values={"account": p.frost_account})
                )
                p.sibling = int(result["frostUpgrade"])
                assert await address(p, p.sibling) == ui_sibling_address, (
                    f"participant {p.id} derived another sibling address"
                )
            print(f"Sibling address {ui_sibling_address[:24]}… agreed by every member")

            print("\n=== Step 8: Participant 2 coordinates the sweep; the app signs ===")
            result = await coordinator.execute(
                GraphQLRequest(
                    FROST_SWEEP,
                    variable_values={
                        "account": coordinator.frost_account,
                        "coordinator": COORDINATOR_ID,
                        "funding": coordinator.funding_account,
                    },
                )
            )
            pczt = result["frostSweep"]
            paid_account = coordinator.sibling

            async def landed():
                if not (
                    await sync_tolerant(coordinator, coordinator.frost_account)
                    and await sync_tolerant(coordinator, coordinator.sibling)
                ):
                    return False
                legacy_balance = await balance(coordinator, coordinator.frost_account)
                sibling_balance = await balance(coordinator, coordinator.sibling)
                print(f"Legacy {legacy_balance}, sibling {sibling_balance}")
                return legacy_balance == "0" and float(sibling_balance) > 0

        else:
            print("\n=== Step 7: Participant 2 prepares a payment; the app signs ===")
            result = await coordinator.execute(
                GraphQLRequest(
                    CREATE_ACCOUNT, variable_values={"name": "FROST-Receiver", "key": ""}
                )
            )
            paid_account = int(result["createAccount"])
            receiver_address = await address(coordinator, paid_account)
            result = await coordinator.execute(
                GraphQLRequest(
                    PREPARE_SEND,
                    variable_values={
                        "account": coordinator.frost_account,
                        "address": receiver_address,
                        "amount": PAYMENT,
                    },
                )
            )
            pczt = result["prepareSend"]

            async def landed():
                if not await sync_tolerant(coordinator, paid_account):
                    return False
                received = await balance(coordinator, paid_account)
                print(f"Receiver balance: {received} (want {PAYMENT_RECEIVED})")
                return received == PAYMENT_RECEIVED

        rendezvous.write_orchestrator(pczt=pczt)
        # frostSweep already started the coordinator's session; prepareSend
        # does not, so for a payment the coordinator starts it like anyone.
        for p in participants:
            if legacy and p is coordinator:
                continue
            result = await p.execute(
                GraphQLRequest(
                    FROST_SIGN,
                    variable_values={
                        "account": p.frost_account,
                        "coordinator": COORDINATOR_ID,
                        "funding": p.funding_account,
                        "pczt": pczt,
                    },
                )
            )
            assert result["frostSign"]
        await rendezvous.poll_ui("signing_status", timeout=1800, process=flutter)

        print("\n=== Step 9: Wait for the transaction to land ===")
        await wait_until(landed, 900, failure="the transaction did not land")
        rendezvous.write_orchestrator(pczt=pczt, mined=True)

        print("\n=== Step 10: The app checks its accounts ===")
        await rendezvous.poll_ui("done", timeout=1200, process=flutter)
        returncode = await asyncio.wait_for(flutter.wait(), timeout=300)
        assert returncode == 0, f"flutter test failed with {returncode}, see {FLUTTER_LOG}"
        print(f"\n=== ✅ FROST restore UI test (dkg-{dkg_version}) passed ===")

    except Exception:
        dump_server_log(FLUTTER_LOG, "FLUTTER TEST LOG")
        for p in participants:
            dump_server_log(f"/tmp/graphql_{p.port}.log", f"PARTICIPANT {p.id}")
        raise

    finally:
        if miner is not None:
            miner.cancel()
            await asyncio.gather(miner, return_exceptions=True)
        if flutter is not None and flutter.returncode is None:
            flutter.terminate()
            with contextlib.suppress(asyncio.TimeoutError):
                await asyncio.wait_for(flutter.wait(), timeout=30)
        for p in participants:
            await p.stop()
        if default_participant:
            await default_participant.stop()
        rendezvous.cleanup()
        if os.path.exists(ui_db_path):
            os.remove(ui_db_path)


@pytest.mark.dkg_ui
@pytest.mark.asyncio
async def test_frost_restore_legacy_flutter_ui(
    graphql_url, rpc_url, seed, zkool_binary, gql_client_factory
):
    """A legacy member restored in the app: banner, upgrade, sweep."""
    await _run(1, graphql_url, rpc_url, seed, zkool_binary, gql_client_factory)


@pytest.mark.dkg_ui
@pytest.mark.asyncio
async def test_frost_restore_zip2005_flutter_ui(
    graphql_url, rpc_url, seed, zkool_binary, gql_client_factory
):
    """A ZIP 2005 member restored in the app: no banner, an ordinary payment."""
    await _run(2, graphql_url, rpc_url, seed, zkool_binary, gql_client_factory)
