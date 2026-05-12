import asyncio
import contextlib
import os
import subprocess

import httpx
import pytest
from gql import Client, GraphQLRequest, gql
from gql.transport.httpx import HTTPXAsyncTransport

from utils import get_current_height, mine_blocks, wait_for_blocks


@pytest.fixture(scope="session")
def zkool_binary():
    """Path to zkool_graphql binary."""
    return os.path.join(os.path.dirname(__file__), "..", "..", "target", "release", "zkool_graphql")


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
async def test_transfer_to_new_wallet(gql_client_factory, rpc_url, seed, zkool_binary):
    """Port of example/sh/smoke.sh smoke test."""
    if not seed:
        pytest.skip("SEED not set")

    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    PORT = 8000
    LWD_URL = "http://localhost:8137"
    DB_PATH = "/tmp/regtest_smoke.db"
    LOG_PATH = "/tmp/graphql_smoke.log"
    GRAPHQL_URL = f"http://localhost:{PORT}/graphql"

    process = None

    try:
        # Kill any existing zkool_graphql processes
        print("Killing any existing zkool_graphql processes...")
        subprocess.run(["pkill", "-9", "zkool_graphql"], stderr=subprocess.DEVNULL)
        await asyncio.sleep(1)

        # Remove existing database
        if os.path.exists(DB_PATH):
            os.remove(DB_PATH)

        # Start zkool_graphql instance
        print(f"Starting zkool_graphql on port {PORT}")
        process = subprocess.Popen(
            [zkool_binary, "-d", DB_PATH, "-p", str(PORT), "-l", LWD_URL],
            stdout=open(LOG_PATH, "w"),
            stderr=subprocess.STDOUT,
        )
        await asyncio.sleep(2)

        # Create client and run test
        async with gql_client_factory(GRAPHQL_URL) as client:
            # Import funded wallet
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
            wallet_id = int(result["createAccount"])
            assert wallet_id > 0
            print(f"Created funding wallet: {wallet_id}")

            # Synchronize the funded wallet
            sync_mutation = gql(
                """
                mutation ($account: Int!) {
                    synchronizeAccount(idAccount: $account)
                }
                """
            )
            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": wallet_id})
            )
            print("Synchronized funding wallet")

            # Create new wallet
            create_account_mutation2 = gql(
                """
                mutation {
                    createAccount(newAccount: {
                        name: "A"
                        key: ""
                        aindex: 0
                        useInternal: false
                        birth: 1
                    })
                }
                """
            )
            result = await client.execute_async(create_account_mutation2)
            a2_id = int(result["createAccount"])
            assert a2_id > 0
            print(f"Created recipient wallet: {a2_id}")

            # Get new wallet address
            address_query = gql(
                """
                query ($account: Int!) {
                    addressByAccount(idAccount: $account) {
                        orchard
                    }
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(address_query, variable_values={"account": a2_id})
            )
            address = result["addressByAccount"]["orchard"]
            assert address
            assert address.startswith("uregtest1") or address.startswith("zrays")
            print(f"Recipient address: {address}")

            # Synchronize both accounts
            sync_both_mutation = gql(
                """
                mutation ($accounts: [Int!]!) {
                    synchronize(idAccounts: $accounts)
                }
                """
            )
            await client.execute_async(
                GraphQLRequest(sync_both_mutation, variable_values={"accounts": [wallet_id, a2_id]})
            )

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
                GraphQLRequest(balance_query, variable_values={"account": wallet_id})
            )
            funding_balance = result["balanceByAccount"]["orchard"]
            print(f"Funding wallet balance: {funding_balance}")

            # Send funds
            pay_mutation = gql(
                """
                mutation ($account: Int!, $address: String!, $amount: BigDecimal!) {
                    pay(idAccount: $account, payment: {
                        recipients: [{
                            address: $address
                            amount: $amount
                        }]
                    })
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(
                    pay_mutation,
                    variable_values={"account": wallet_id, "address": address, "amount": "10.5"},
                )
            )
            txid = result["pay"]
            assert txid
            print(f"Sent funds, txid: {txid}")

            # Wait for transaction to propagate
            await asyncio.sleep(5)

            # Mine blocks
            height_before = await get_current_height(client)
            print(f"Height before mining: {height_before}")

            await mine_blocks(rpc_url, 10)
            await wait_for_blocks(client, height_before, 10)
            print("Blocks mined")

            # Synchronize the recipient account
            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": a2_id})
            )

            # Check final balance
            result = await client.execute_async(
                GraphQLRequest(balance_query, variable_values={"account": a2_id})
            )
            final_balance = result["balanceByAccount"]["orchard"]
            print(f"Final balance: {final_balance}")

            assert final_balance == "10.50000000", f"Expected 10.50000000, got {final_balance}"
            print("✅ Smoke test passed!")

    finally:
        # Cleanup
        if process:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()

        if os.path.exists(DB_PATH):
            os.remove(DB_PATH)
        if os.path.exists(LOG_PATH):
            os.remove(LOG_PATH)
