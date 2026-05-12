"""Test FROST signing using GraphQL API."""

import asyncio
import os
import sqlite3
import subprocess

import httpx
import pytest
from gql import Client, GraphQLRequest, gql
from gql.transport.httpx import HTTPXAsyncTransport

from utils import mine_blocks

from test_dkg import DkgParticipant


@pytest.fixture(scope="session")
def graphql_url():
    return os.getenv("GRAPHQL_URL", "http://localhost:8000/graphql")


@pytest.fixture(scope="session")
def rpc_url():
    return os.getenv("RPC_URL", "http://127.0.0.1:18232/")


@pytest.fixture(scope="session")
def seed():
    return os.getenv("SEED", "")


@pytest.fixture(scope="session")
def zkool_binary():
    """Path to zkool_graphql binary."""
    return os.path.join(
        os.path.dirname(__file__), "..", "..", "target", "release", "zkool_graphql"
    )


@pytest.fixture(scope="session")
def gql_client_factory():
    """Factory to create GraphQL clients for different URLs."""

    async def _create_client(url: str):
        timeout = httpx.Timeout(300.0, connect=60.0)
        transport = HTTPXAsyncTransport(url=url, timeout=timeout)
        client = Client(
            transport=transport, fetch_schema_from_transport=False, execute_timeout=300.0
        )
        return client

    return _create_client


@pytest.mark.asyncio
async def test_frost_sign_3_of_3(graphql_url, rpc_url, seed, zkool_binary):
    """Test 3-out-of-3 FROST signing using GraphQL API."""
    if not seed:
        pytest.skip("SEED not set")

    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    N = 3  # Number of participants
    T = 3  # Threshold
    DEFAULT_PORT = 8000
    PORT_BASE = 8001
    LWD_URL = "http://localhost:8137"

    # Cleanup function
    participants = []
    default_participant = None

    async def cleanup():
        for p in participants:
            await p.stop()
        if default_participant:
            await default_participant.stop()
        # Clean up log files (databases are kept for further testing)
        for i in range(1, N + 1):
            log_path = f"/tmp/graphql_{PORT_BASE + i - 1}.log"
            if os.path.exists(log_path):
                os.remove(log_path)
        default_log = "/tmp/graphql_default.log"
        if os.path.exists(default_log):
            os.remove(default_log)

    try:
        # Kill any existing zkool_graphql processes
        subprocess.run(["pkill", "-9", "zkool_graphql"], stderr=subprocess.DEVNULL)

        print("=== Setting up 3-out-of-3 FROST SIGN Test ===")

        # Check if DKG databases exist; if not, skip the test
        dkg_missing = False
        for i in range(1, N + 1):
            db_path = f"/tmp/regtest_dkg_test_{i}.db"
            if not os.path.exists(db_path):
                print(f"DKG database not found at {db_path}")
                dkg_missing = True
            else:
                # Verify the DKG account exists
                conn = sqlite3.connect(db_path)
                cursor = conn.cursor()
                cursor.execute("SELECT id_account FROM accounts WHERE name LIKE 'Dkg-Test-%'")
                result = cursor.fetchone()
                conn.close()
                if not result:
                    print(f"DKG account not found in {db_path}")
                    dkg_missing = True

        if dkg_missing:
            pytest.skip("DKG not completed. Run test_dkg first to create shared accounts.")

        # Start default instance
        print(f"Starting default instance on port {DEFAULT_PORT}")
        default_db = "/tmp/regtest_dkg_default.db"
        default_participant = DkgParticipant(DEFAULT_PORT, default_db, LWD_URL)
        default_participant.start(zkool_binary)
        await asyncio.sleep(2)

        # Start participant instances (they already have DKG databases)
        print("\n=== Step 1: Start participant instances ===")
        for i in range(1, N + 1):
            port = PORT_BASE + i - 1
            db_path = f"/tmp/regtest_dkg_test_{i}.db"
            participant = DkgParticipant(port, db_path, LWD_URL)
            participant.start(zkool_binary, remove_db=False)
            participants.append(participant)
            print(f"Started participant {i} on port {port}")
            await asyncio.sleep(2)

        print("\n=== Step 2: Find shared account IDs ===")

        # Get FROST account IDs for all participants from their DKG databases
        frost_account_ids = {}
        for i, participant in enumerate(participants, 1):
            frost_account = participant.get_frost_account_id()
            if not frost_account:
                pytest.fail(f"No FROST account found for participant {i}")
            frost_account_ids[i] = frost_account
            participant.frost_account = frost_account
            print(f"Participant {i} FROST account ID: {frost_account}")

        # Use participant #2 as coordinator
        coordinator = participants[1]  # Index 1 is participant #2
        coordinator_frost_account = frost_account_ids[2]
        print(f"Coordinator (participant #2) FROST account ID: {coordinator_frost_account}")

        # Create a receiver account
        print("\n=== Step 3: Create receiver account ===")
        create_account_mutation = gql(
            """
            mutation {
                createAccount(newAccount: {
                    name: "FROST-Receiver"
                    key: ""
                    aindex: 0
                    useInternal: false
                    birth: 1
                })
            }
            """
        )
        result = await coordinator.execute(GraphQLRequest(create_account_mutation))
        receiver_account = int(result["createAccount"])
        print(f"Receiver account ID: {receiver_account}")

        # Get receiver address
        address_query = gql(
            """
            query ($account: Int!) {
                addressByAccount(idAccount: $account) {
                    orchard
                }
            }
            """
        )
        result = await coordinator.execute(
            GraphQLRequest(address_query, variable_values={"account": receiver_account})
        )
        receiver_address = result["addressByAccount"]["orchard"]
        print(f"Receiver address: {receiver_address}")

        # Synchronize coordinator's FROST account
        print("\n=== Step 4: Synchronize FROST account ===")
        sync_mutation = gql(
            """
            mutation ($account: Int!) {
                synchronizeAccount(idAccount: $account)
            }
            """
        )
        await coordinator.execute(
            GraphQLRequest(sync_mutation, variable_values={"account": coordinator_frost_account})
        )
        print("FROST account synchronized")

        # Prepare payment from FROST shared account to receiver
        print("\n=== Step 5: Prepare payment ===")
        prepare_query = gql(
            """
            query ($account: Int!, $address: String!, $amount: BigDecimal!) {
                prepareSend(
                    idAccount: $account
                    payment: {
                        recipients: [{
                            address: $address
                            amount: $amount
                        }]
                    }
                )
            }
            """
        )
        result = await coordinator.execute(
            GraphQLRequest(
                prepare_query,
                variable_values={
                    "account": coordinator_frost_account,
                    "address": receiver_address,
                    "amount": "0.05"
                }
            )
        )
        pczt = result["prepareSend"]
        print(f"PCZT prepared: {pczt[:50]}...")

        # Each participant performs FROST signing
        print("\n=== Step 6: FROST signing round ===")
        for i, participant in enumerate(participants, 1):
            frost_account = frost_account_ids[i]

            # Get funding account for this participant
            funding_account = None
            conn = sqlite3.connect(participant.db_path)
            cursor = conn.cursor()
            cursor.execute("SELECT id_account FROM accounts WHERE name = 'DKG-Fund'")
            result = cursor.fetchone()
            conn.close()
            if result:
                funding_account = result[0]
            else:
                pytest.fail(f"No funding account found for participant {i}")

            print(f"Participant {i} signing with funding account {funding_account}...")

            sign_mutation = gql(
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
            result = await participant.execute(
                GraphQLRequest(
                    sign_mutation,
                    variable_values={
                        "account": frost_account,
                        "coordinator": 2,
                        "funding": funding_account,
                        "pczt": pczt
                    }
                )
            )
            sign_result = result["frostSign"]
            print(f"Participant {i} sign result: {sign_result}")

            if not sign_result:
                pytest.fail(f"Participant {i} signing failed")

        print("All participants completed signing round")

        # Check FROST shared account balance
        balance_query = gql(
            """
            query ($account: Int!) {
                balanceByAccount(idAccount: $account) {
                    orchard
                }
            }
            """
        )
        result = await coordinator.execute(
            GraphQLRequest(balance_query, variable_values={"account": coordinator_frost_account})
        )
        frost_balance = result["balanceByAccount"]["orchard"]
        print(f"FROST shared account balance after signing: {frost_balance} ZEC")

        # Verify transaction completed and receiver received funds
        print("\n=== Step 7: Verify transaction completed ===")
        TIMEOUT = 300
        ELAPSED = 0
        expected_amount = "0.05000000"

        while ELAPSED < TIMEOUT:
            # Synchronize receiver account
            sync_mutation = gql(
                """
                mutation ($account: Int!) {
                    synchronizeAccount(idAccount: $account)
                }
                """
            )
            await coordinator.execute(
                GraphQLRequest(sync_mutation, variable_values={"account": receiver_account})
            )

            # Check receiver balance
            result = await coordinator.execute(
                GraphQLRequest(balance_query, variable_values={"account": receiver_account})
            )
            receiver_balance = result["balanceByAccount"]["orchard"]
            print(f"Receiver balance: {receiver_balance} ZEC (waiting for {expected_amount} ZEC)")

            # Check if balance matches expected amount
            if receiver_balance == expected_amount:
                print("\n=== ✅ Transaction completed successfully! ===")
                print(f"Receiver account received {expected_amount} ZEC from FROST shared account")
                print("\n=== Test completed successfully ===")
                return

            await asyncio.sleep(10)
            ELAPSED += 10

        pytest.fail(f"Transaction did not complete within {TIMEOUT} seconds. Final receiver balance: {receiver_balance} ZEC (expected {expected_amount} ZEC)")

    finally:
        await cleanup()
