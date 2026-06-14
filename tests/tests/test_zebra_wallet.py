"""Test wallet synchronization under zebra mode.

Starts zkool_graphql with --zebra pointing directly to the zebra JSON-RPC
endpoint (bypassing lightwalletd), then syncs the funded wallet and verifies
the balance matches what the mine.sh setup script deposited.
"""

import asyncio
import os

import pytest
from gql import GraphQLRequest, gql

from utils import (
    cleanup_test_files,
    get_balance,
    kill_existing_zkool_processes,
    start_zkool_instance,
    stop_zkool_instance,
)


@pytest.mark.asyncio
async def test_zebra_wallet_sync(gql_client_factory, rpc_url, seed, zkool_binary):
    """Sync a funded wallet using zebra JSON-RPC backend and verify balance."""
    if not seed:
        pytest.skip("SEED not set")

    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    PORT = 8000
    ZEBRA_RPC_URL = rpc_url  # zebra JSON-RPC endpoint (default: http://127.0.0.1:18232)
    DB_PATH = "/tmp/regtest_zebra_wallet.db"
    LOG_PATH = "/tmp/graphql_zebra_wallet.log"
    GRAPHQL_URL = f"http://localhost:{PORT}/graphql"

    process = None

    try:
        await kill_existing_zkool_processes()
        await asyncio.sleep(2)

        print(f"Starting zkool_graphql in zebra mode on port {PORT}")
        print(f"Zebra RPC URL: {ZEBRA_RPC_URL}")
        process = await start_zkool_instance(
            zkool_binary, DB_PATH, PORT, ZEBRA_RPC_URL, LOG_PATH, zebra=True
        )
        await asyncio.sleep(3)

        # Check process is still alive
        poll_result = process.poll()
        if poll_result is not None:
            with open(LOG_PATH, "r") as f:
                log_content = f.read()
                print(f"Server log:\n{log_content}")
            pytest.fail(f"zkool_graphql (zebra mode) failed to start with code {poll_result}")

        async with gql_client_factory(GRAPHQL_URL) as client:
            # Import the funded wallet using the seed from mine.sh setup
            create_account_mutation = gql(
                """
                mutation ($seed: String!) {
                    createAccount(newAccount: {
                        name: "ZebraWallet"
                        key: $seed
                        aindex: 0
                        useInternal: false
                        birth: 1
                    })
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(create_account_mutation, variable_values={"seed": seed})
            )
            wallet_id = int(result["createAccount"])
            assert wallet_id > 0
            print(f"Created wallet (zebra mode): {wallet_id}")

            # Sync the wallet via zebra backend
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
            print("Wallet synchronized via zebra backend")

            # Verify balance — mine.sh sent funds to this wallet's address
            orchard_balance = await get_balance(client, wallet_id, "orchard")
            print(f"Orchard balance: {orchard_balance} ZEC")

            orchard_val = float(orchard_balance)
            assert orchard_val > 0, (
                f"Expected positive orchard balance (funded by mine.sh), got {orchard_balance}"
            )

            # Also verify total balance is consistent
            total_balance = await get_balance(client, wallet_id, "total")
            print(f"Total balance: {total_balance} ZEC")
            assert float(total_balance) > 0

            # Check that current height is advancing (zebra RPC is working)
            height_query = gql("query { currentHeight }")
            result = await client.execute_async(GraphQLRequest(height_query))
            current_height = int(result["currentHeight"])
            print(f"Current height: {current_height}")
            assert current_height > 100, f"Expected height > 100, got {current_height}"

            print("✅ Zebra wallet sync test passed!")

    finally:
        await stop_zkool_instance(process)
        cleanup_test_files(DB_PATH, LOG_PATH)
