"""Test blockchain reorganization handling."""

import asyncio
import os

import pytest
from gql import GraphQLRequest, gql

from utils import (
    cleanup_test_files,
    get_current_height,
    kill_existing_zkool_processes,
    mine_blocks,
    start_zkool_instance,
    stop_zkool_instance,
    wait_for_blocks,
)


@pytest.mark.asyncio
async def test_reorg(gql_client_factory, rpc_url, seed, zkool_binary, lwd_url):
    """Test that wallet correctly handles blockchain reorganization."""
    if not seed:
        pytest.skip("SEED not set")

    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    PORT = 8000
    DB_PATH = "/tmp/regtest_reorg.db"
    LOG_PATH = "/tmp/graphql_reorg.log"
    GRAPHQL_URL = f"http://localhost:{PORT}/graphql"

    process = None

    try:
        await kill_existing_zkool_processes()

        # Clear mempool to avoid conflicts from previous test runs
        print("Clearing mempool...")
        import httpx
        async with httpx.AsyncClient() as rpc_client:
            await rpc_client.post(
                rpc_url,
                json={"jsonrpc": "2.0", "id": 1, "method": "clearmempool"}
            )
            print("Mempool cleared")

        print(f"Starting zkool_graphql on port {PORT}")
        process = await start_zkool_instance(zkool_binary, DB_PATH, PORT, lwd_url, LOG_PATH)

        async with gql_client_factory(GRAPHQL_URL) as client:
            # Mine a new block on the valid tip to confirm the reorg
            print("\n=== Step 0: Mine a block on the new tip ===")
            await mine_blocks(rpc_url, 1)

            print("\n=== Step 1: Create funding wallet ===")
            create_account_mutation = gql(
                """
                mutation ($main: String!) {
                    createAccount(newAccount: {
                        name: "Funding"
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
            funding_id = int(result["createAccount"])
            print(f"Created funding wallet: {funding_id}")

            sync_mutation = gql(
                """
                mutation ($account: Int!) {
                    synchronizeAccount(idAccount: $account)
                }
                """
            )
            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": funding_id})
            )

            print("\n=== Step 2: Create test account ===")
            create_account_mutation = gql(
                """
                mutation ($name: String!) {
                    createAccount(newAccount: {
                        name: $name
                        key: ""
                        aindex: 0
                        useInternal: false
                        birth: 1
                    })
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(create_account_mutation, variable_values={"name": "TestAccount"})
            )
            test_account_id = int(result["createAccount"])
            print(f"Created test account: {test_account_id}")

            # Get test account address
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
                GraphQLRequest(address_query, variable_values={"account": test_account_id})
            )
            test_address = result["addressByAccount"]["orchard"]
            print(f"Test account address: {test_address}")

            print("\n=== Step 3: Send funds to test account ===")
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
                    variable_values={"account": funding_id, "address": test_address, "amount": "0.1"}
                )
            )
            txid = result["pay"]
            print(f"Sent 0.1 ZEC to test account, txid: {txid}")

            print("\n=== Step 4: Get current height and block hash ===")
            # Get the current block hash (we'll invalidate this later)
            getblock_hash_query = gql(
                """
                query {
                    currentHeight
                }
                """
            )
            result = await client.execute_async(GraphQLRequest(getblock_hash_query))
            height_before = result["currentHeight"]
            print(f"Current height: {height_before}")

            # Get block hash via RPC (GraphQL doesn't expose block hashes)
            import httpx
            async with httpx.AsyncClient() as rpc_client:
                response = await rpc_client.post(
                    rpc_url,
                    json={
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "getblockhash",
                        "params": [height_before]
                    }
                )
                block_hash = response.json().get("result")
                print(f"Block hash at height {height_before}: {block_hash}")

            print("\n=== Step 5: Mine a few blocks ===")
            height_before_mining = await get_current_height(client)
            await mine_blocks(rpc_url, 5)
            await wait_for_blocks(client, height_before_mining, 5)
            print("Mined 5 blocks")

            print("\n=== Step 6: Sync test account ===")
            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": test_account_id})
            )

            print("\n=== Step 7: Check balance (should be > 0) ===")
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
                GraphQLRequest(balance_query, variable_values={"account": test_account_id})
            )
            balance_before_reorg = result["balanceByAccount"]["orchard"]
            print(f"Test account balance before reorg: {balance_before_reorg} ZEC")

            assert float(balance_before_reorg) > 0, f"Balance should be > 0, got {balance_before_reorg}"
            print("✓ Balance is > 0 as expected")

            print("\n=== Step 8: Invalidate block to trigger reorg ===")
            # Check height before invalidation
            height_before_invalidate = await get_current_height(client)
            print(f"Height before invalidate: {height_before_invalidate}")

            # Invalidate the block to trigger a chain reorganization
            async with httpx.AsyncClient() as rpc_client:
                response = await rpc_client.post(
                    rpc_url,
                    json={
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "invalidateblock",
                        "params": [block_hash]
                    }
                )
                print(f"Invalidate block response: {response.json()}")
            # Mine 10 blocks to ensure that the new chain is longer
            await mine_blocks(rpc_url, 10)

            # Check height after invalidation
            await asyncio.sleep(1)
            height_after_invalidate = await get_current_height(client)
            print(f"Height after invalidate: {height_after_invalidate}")

            # Clear mempool so the transaction doesn't get re-mined in the new chain
            print("Clearing mempool to prevent transaction from being re-included...")
            async with httpx.AsyncClient() as rpc_client:
                await rpc_client.post(
                    rpc_url,
                    json={"jsonrpc": "2.0", "id": 1, "method": "clearmempool"}
                )
            print("Mempool cleared")

            await asyncio.sleep(2)  # Give time for reorg to propagate
            height_after_new_block = await get_current_height(client)
            print(f"Height after mining new block: {height_after_new_block}")

            print("\n=== Step 10: Sync test account after reorg ===")
            # After reorg, we need to sync again to pick up the new chain state
            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": test_account_id})
            )
            # Debug: Check wallet height
            current_height = await get_current_height(client)
            print(f"Current wallet height: {current_height}")

            # Debug: Check if transaction is still in mempool or confirmed
            print("\n=== Debug: Check transaction status ===")
            async with httpx.AsyncClient() as rpc_client:
                response = await rpc_client.post(
                    rpc_url,
                    json={"jsonrpc": "2.0", "id": 1, "method": "getrawmempool"}
                )
                mempool = response.json().get("result", [])
                print(f"Mempool: {len(mempool)} transactions")
                if txid in mempool:
                    print(f"  TX {txid} is in mempool")
                else:
                    print(f"  TX {txid} is NOT in mempool")

                response = await rpc_client.post(
                    rpc_url,
                    json={"jsonrpc": "2.0", "id": 1, "method": "getrawtransaction", "params": [txid, 1]}
                )
                tx_result = response.json()
                if "result" in tx_result:
                    tx_info = tx_result["result"]
                    confirmations = tx_info.get("confirmations", 0)
                    print(f"TX confirmations: {confirmations}")
                    if confirmations > 0:
                        print(f"  TX is STILL CONFIRMED in new chain!")
                        print(f"  Block hash: {tx_info.get('blockhash')}")

            print("\n=== Step 11: Check balance (should be = 0) ===")
            result = await client.execute_async(
                GraphQLRequest(balance_query, variable_values={"account": test_account_id})
            )
            balance_after_reorg = result["balanceByAccount"]["orchard"]
            print(f"Test account balance after reorg: {balance_after_reorg} ZEC")

            assert float(balance_after_reorg) == 0, f"Balance should be 0 after reorg, got {balance_after_reorg}"
            print("✓ Balance is 0 as expected (transaction was in orphaned chain)")

            print("\n✅ Reorganization test passed!")

    finally:
        await stop_zkool_instance(process)
        #cleanup_test_files(DB_PATH, LOG_PATH)
