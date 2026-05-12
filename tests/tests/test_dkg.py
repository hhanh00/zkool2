import asyncio
import contextlib
import os
import sqlite3
import subprocess

import httpx
import pytest
from gql import Client, GraphQLRequest, gql
from gql.transport.httpx import HTTPXAsyncTransport

from utils import get_current_height, mine_blocks, wait_for_blocks


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


class DkgParticipant:
    def __init__(self, port: int, db_path: str, lwd_url: str):
        self.port = port
        self.db_path = db_path
        self.lwd_url = lwd_url
        self.url = f"http://localhost:{port}/graphql"
        self.process: subprocess.Popen | None = None
        self.funding_account: int | None = None
        self.funding_address: str | None = None
        self.dkg_address: str | None = None
        self.frost_account: int | None = None
        self._client: Client | None = None

    async def get_client(self) -> Client:
        """Get or create the GraphQL client for this participant."""
        if self._client is None:
            timeout = httpx.Timeout(300.0, connect=60.0)
            transport = HTTPXAsyncTransport(url=self.url, timeout=timeout)
            self._client = Client(
                transport=transport, fetch_schema_from_transport=False, execute_timeout=300.0
            )
        return self._client

    async def close_client(self):
        """Close the GraphQL client."""
        if self._client:
            await self._client.close_async()
            self._client = None

    async def execute(self, request: GraphQLRequest):
        """Execute a GraphQL request on this participant's server."""
        client = await self.get_client()
        return await client.execute_async(request)

    def start(self, zkool_binary: str, remove_db=True):
        """Start the zkool_graphql instance."""
        # Remove existing database
        if remove_db and os.path.exists(self.db_path):
            os.remove(self.db_path)

        log_path = f"/tmp/graphql_{self.port}.log"
        self.process = subprocess.Popen(
            [
                zkool_binary,
                "-d", self.db_path,
                "-p", str(self.port),
                "-l", self.lwd_url,
            ],
            stdout=open(log_path, "w"),
            stderr=subprocess.STDOUT,
        )

    async def stop(self):
        """Stop the zkool_graphql instance and close client."""
        await self.close_client()
        if self.process:
            self.process.terminate()
            self.process.wait(timeout=10)
            self.process = None

    def get_frost_account_id(self) -> int | None:
        """Get the FROST account ID from the database."""
        if not os.path.exists(self.db_path):
            return None

        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            cursor.execute("SELECT id_account FROM accounts WHERE name LIKE 'Dkg-Test-%'")
            result = cursor.fetchone()
            conn.close()
            return result[0] if result else None
        except Exception:
            return None


@pytest.fixture
def gql_client_factory():
    """Factory to create GraphQL clients for different URLs."""

    @contextlib.asynccontextmanager
    async def _create_client(url: str):
        timeout = httpx.Timeout(300.0, connect=60.0)
        transport = HTTPXAsyncTransport(url=url, timeout=timeout)
        client = Client(
            transport=transport, fetch_schema_from_transport=False, execute_timeout=300.0
        )
        try:
            yield client
        finally:
            await client.close_async()

    return _create_client


@pytest.mark.asyncio
async def test_dkg_3_of_3(graphql_url, rpc_url, seed, zkool_binary, gql_client_factory):
    """Test 3-out-of-3 FROST DKG using GraphQL API."""
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
        # Clean up log files (databases are kept for test_frost.py)
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

        # Start default instance
        print("=== Setting up 3-out-of-3 FROST DKG Test ===")
        print(f"Starting default instance on port {DEFAULT_PORT}")
        default_db = "/tmp/regtest_dkg_default.db"
        default_participant = DkgParticipant(DEFAULT_PORT, default_db, LWD_URL)
        default_participant.start(zkool_binary)
        await asyncio.sleep(2)

        # Start participant instances
        print("\n=== Step 1: Start participant instances ===")
        for i in range(1, N + 1):
            port = PORT_BASE + i - 1
            db_path = f"/tmp/regtest_dkg_test_{i}.db"
            participant = DkgParticipant(port, db_path, LWD_URL)
            participant.start(zkool_binary)
            participants.append(participant)
            print(f"Started participant {i} on port {port}")
            await asyncio.sleep(2)

        print("\n=== Step 2: Create funded wallet on default instance ===")
        async with gql_client_factory(graphql_url) as client:
            # Create funded wallet
            create_account_mutation = gql(
                """
                mutation ($main: String!) {
                    createAccount(newAccount: {
                        name: "Main"
                        key: $main
                        aindex: 0
                        useInternal: false
                        birth: 1
                    })
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(create_account_mutation, variable_values={"main": seed})
            )
            main_wallet = int(result["createAccount"])
            print(f"Funding wallet: {main_wallet}")

            # Synchronize wallet
            sync_mutation = gql(
                """
                mutation ($account: Int!) {
                    synchronizeAccount(idAccount: $account)
                }
                """
            )
            await client.execute_async(GraphQLRequest(sync_mutation, variable_values={"account": main_wallet}))

            # Get funding wallet balance
            balance_query = gql(
                """
                query ($account: Int!) {
                    balanceByAccount(idAccount: $account) {
                        orchard
                    }
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(balance_query, variable_values={"account": main_wallet})
            )
            funding_balance = result["balanceByAccount"]["orchard"]
            print(f"Funding wallet balance: {funding_balance}")

        print("\n=== Step 3: Initialize DKG for each participant ===")
        for i, participant in enumerate(participants, 1):
            # Create funding account for DKG
            create_account_mutation = gql(
                """
                mutation {
                    createAccount(newAccount: {
                        name: "DKG-Fund"
                        key: ""
                        aindex: 0
                        useInternal: false
                        birth: 1
                    })
                }
                """
            )
            result = await participant.execute(GraphQLRequest(create_account_mutation))
            participant.funding_account = int(result["createAccount"])

            # Get funding address
            address_query = gql(
                """
                query ($account: Int!) {
                    addressByAccount(idAccount: $account) {
                        orchard
                    }
                }
                """
            )
            result = await participant.execute(
                GraphQLRequest(address_query, variable_values={"account": participant.funding_account})
            )
            participant.funding_address = result["addressByAccount"]["orchard"]

            print(f"Participant {i} funding account: {participant.funding_account}")
            print(f"Participant {i} funding address: {participant.funding_address}")

            # Initialize DKG
            dkg_start_mutation = gql(
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
            result = await participant.execute(
                GraphQLRequest(
                    dkg_start_mutation,
                    variable_values={
                        "name": f"Dkg-Test-{i}",
                        "t": T,
                        "n": N,
                        "funding": participant.funding_account,
                        "id": i,
                    },
                )
            )
            participant.dkg_address = result["dkgStart"]
            print(f"Participant {i} DKG address: {participant.dkg_address}")

        print("\n=== Step 4: Fund each participant's funding address ===")
        async with gql_client_factory(graphql_url) as client:
            # Build recipients list
            recipients = [
                {"address": p.funding_address, "amount": "0.01"} for p in participants
            ]

            pay_mutation = gql(
                """
                mutation ($account: Int!, $recipients: [Recipient!]!) {
                    pay(idAccount: $account, payment: {recipients: $recipients})
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(
                    pay_mutation,
                    variable_values={"account": main_wallet, "recipients": recipients},
                )
            )
            txid = result["pay"]
            print(f"Funding transaction: {txid}")

        print("\n=== Step 5: Mine blocks for confirmation ===")
        client = await participants[0].get_client()
        height = await get_current_height(client)
        await mine_blocks(rpc_url, 5)
        await wait_for_blocks(client, height, 5)
        print("Blocks mined")

        print("\n=== Step 6: Synchronize funding accounts ===")
        for i, participant in enumerate(participants, 1):
            sync_mutation = gql(
                """
                mutation ($account: Int!) {
                    synchronizeAccount(idAccount: $account)
                }
                """
            )
            await participant.execute(
                GraphQLRequest(sync_mutation, variable_values={"account": participant.funding_account})
            )
            print(f"Synchronized participant {i} funding account")

        print("\n=== Step 7: Verify funding accounts received funds ===")
        for i, participant in enumerate(participants, 1):
            balance_query = gql(
                """
                query ($account: Int!) {
                    balanceByAccount(idAccount: $account) {
                        orchard
                    }
                }
                """
            )
            result = await participant.execute(
                GraphQLRequest(balance_query, variable_values={"account": participant.funding_account})
            )
            balance = result["balanceByAccount"]["orchard"]
            print(f"Participant {i} funding account balance: {balance}")
            assert balance and balance != "0", f"Participant {i} has insufficient balance"

        print("\n=== Step 8: Exchange DKG addresses between participants ===")
        for i, sender in enumerate(participants, 1):
            for j, receiver in enumerate(participants, 1):
                if i == j:
                    continue

                target_address = participants[j - 1].dkg_address
                set_address_mutation = gql(
                    """
                    mutation ($id: Int!, $address: String!) {
                        dkgSetAddress(idParticipant: $id, address: $address)
                    }
                    """
                )
                await sender.execute(
                    GraphQLRequest(set_address_mutation, variable_values={"id": j, "address": target_address})
                )
                print(f"Participant {i} set address for participant {j}")

        print("\n=== Step 9: Execute DKG on all participants ===")
        for i, participant in enumerate(participants, 1):
            do_dkg_mutation = gql(
                """
                mutation {
                    doDkg
                }
                """
            )
            await participant.execute(GraphQLRequest(do_dkg_mutation))
            print(f"Initiated DKG on participant {i}")

        # Wait for DKG to complete (poll for account creation)
        print("\n=== Step 10: Wait for DKG completion ===")
        TIMEOUT = 300  # 5 minutes
        ELAPSED = 0
        while ELAPSED < TIMEOUT:
            all_completed = True
            for participant in participants:
                if participant.get_frost_account_id() is None:
                    all_completed = False
                    break

            if all_completed:
                print("All participants completed DKG successfully")
                break

            await asyncio.sleep(10)
            ELAPSED += 10

        if ELAPSED >= TIMEOUT:
            pytest.fail("DKG timed out")

        print("\n=== Step 11: Verify shared address is same for all participants ===")
        shared_address = None
        for i, participant in enumerate(participants, 1):
            # Get FROST account ID from database
            participant.frost_account = participant.get_frost_account_id()
            assert participant.frost_account, f"No FROST account found for participant {i}"

            address_query = gql(
                """
                query ($account: Int!) {
                    addressByAccount(idAccount: $account) {
                        orchard
                    }
                }
                """
            )
            result = await participant.execute(
                GraphQLRequest(address_query, variable_values={"account": participant.frost_account})
            )
            frost_address = result["addressByAccount"]["orchard"]
            print(f"Participant {i} shared address: {frost_address}")

            if shared_address is None:
                shared_address = frost_address
            else:
                assert (
                    shared_address == frost_address
                ), f"Participants generated different shared addresses! {shared_address} != {frost_address}"

        print(f"\n=== Step 12: Fund shared FROST address ===")
        async with gql_client_factory(graphql_url) as client:
            # Sync main wallet
            sync_mutation = gql(
                """
                mutation ($account: Int!) {
                    synchronizeAccount(idAccount: $account)
                }
                """
            )
            await client.execute_async(GraphQLRequest(sync_mutation, variable_values={"account": main_wallet}))

            # Get balance
            balance_query = gql(
                """
                query ($account: Int!) {
                    balanceByAccount(idAccount: $account) {
                        orchard
                    }
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(balance_query, variable_values={"account": main_wallet})
            )
            funding_balance = result["balanceByAccount"]["orchard"]
            print(f"Funding wallet balance: {funding_balance}")

            # Send to shared address
            pay_mutation = gql(
                """
                mutation ($account: Int!, $address: String!, $amount: BigDecimal!) {
                    pay(idAccount: $account, payment: {
                        recipients: [{address: $address, amount: $amount}]
                    })
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(
                    pay_mutation,
                    variable_values={"account": main_wallet, "address": shared_address, "amount": "0.1"},
                )
            )
            txid = result["pay"]
            print(f"Funding transaction: {txid}")

        print("\n=== Step 13: Mine blocks and synchronize ===")
        client = await participants[0].get_client()
        height = await get_current_height(client)
        await mine_blocks(rpc_url, 5)
        await wait_for_blocks(client, height, 5)
        print("Blocks mined")

        print("\n=== Step 14: Synchronize FROST accounts ===")
        for i, participant in enumerate(participants, 1):
            sync_mutation = gql(
                """
                mutation ($account: Int!) {
                    synchronizeAccount(idAccount: $account)
                }
                """
            )
            await participant.execute(
                GraphQLRequest(sync_mutation, variable_values={"account": participant.frost_account})
            )
            print(f"Synchronized participant {i} FROST account")

        print("\n=== Step 15: Verify shared address balance ===")
        for i, participant in enumerate(participants, 1):
            balance_query = gql(
                """
                query ($account: Int!) {
                    balanceByAccount(idAccount: $account) {
                        orchard
                    }
                }
                """
            )
            result = await participant.execute(
                GraphQLRequest(balance_query, variable_values={"account": participant.frost_account})
            )
            final_balance = result["balanceByAccount"]["orchard"]
            print(f"Participant {i} FROST balance: {final_balance}")
            assert final_balance == "0.10000000", f"Expected 0.10000000, got {final_balance}"

        print("\n=== ✅ DKG Test Passed! ===")
        print(f"Shared FROST address: {shared_address}")
        print(f"All {N} participants successfully:")
        print("  - Generated the same shared address")
        print("  - Received funding of 0.1 ZEC")
        print("  - Synchronized independently")

    finally:
        await cleanup()
