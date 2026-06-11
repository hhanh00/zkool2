"""Test account lifecycle management, notes, and memos."""

import os

import pytest
from gql import GraphQLRequest, gql

from utils import (
    cleanup_test_files,
    dump_server_log,
    get_current_height,
    kill_existing_zkool_processes,
    mine_blocks,
    start_zkool_instance,
    stop_zkool_instance,
    wait_for_blocks,
)


@pytest.mark.asyncio
async def test_account_management(gql_client_factory, rpc_url, seed, zkool_binary, lwd_url):
    """Test account lifecycle: edit, delete, reset, notes, and memos."""
    if not seed:
        pytest.skip("SEED not set")

    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    PORT = 8000
    DB_PATH = "/tmp/regtest_account_mgmt.db"
    LOG_PATH = "/tmp/graphql_account_mgmt.log"
    GRAPHQL_URL = f"http://localhost:{PORT}/graphql"

    process = None

    def _dump_log():
        """Dump server log for debugging."""
        content = dump_server_log(LOG_PATH, "SERVER LOG (account_mgmt)")
        if not content:
            print(f"(Server log empty or not found at {LOG_PATH})")

    try:
        await kill_existing_zkool_processes()

        print(f"Starting zkool_graphql on port {PORT}")
        process = await start_zkool_instance(zkool_binary, DB_PATH, PORT, lwd_url, LOG_PATH)

        # Check if process is still running
        poll_result = process.poll()
        if poll_result is not None:
            with open(LOG_PATH, "r") as f:
                log_content = f.read()
                print(f"Log content:\n{log_content}")
            pytest.fail(f"zkool_graphql failed to start with code {poll_result}")

        async with gql_client_factory(GRAPHQL_URL) as client:
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

            print("\n=== Step 2: Create test account for editing ===")
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

            print("\n=== Step 3: Test edit_account mutation ===")
            edit_account_mutation = gql(
                """
                mutation ($account: Int!, $name: String!) {
                    editAccount(idAccount: $account, updateAccount: {name: $name})
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(
                    edit_account_mutation,
                    variable_values={"account": test_account_id, "name": "RenamedAccount"}
                )
            )
            assert result["editAccount"] == True, "editAccount should return true"
            print("✓ Renamed account from 'TestAccount' to 'RenamedAccount'")

            # Verify the name change via accounts query
            accounts_query = gql(
                """
                query ($id: Int!) {
                    accounts(accountFilter: {id: $id}) {
                        id
                        name
                        height
                    }
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(accounts_query, variable_values={"id": test_account_id})
            )
            accounts = result["accounts"]
            assert len(accounts) == 1
            assert accounts[0]["name"] == "RenamedAccount", f"Expected 'RenamedAccount', got '{accounts[0]['name']}'"
            print("✓ Verified name change via accounts query")

            print("\n=== Step 4: Send funds to test account to generate notes ===")
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
                    variable_values={
                        "account": funding_id,
                        "address": test_address,
                        "amount": "0.05"
                    }
                )
            )
            txid1 = result["pay"]
            print(f"Sent 0.05 ZEC to test account, txid: {txid1}")

            # Mine blocks and sync
            height_before = await get_current_height(client)
            await mine_blocks(rpc_url, 5)
            await wait_for_blocks(client, height_before, 5)

            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": test_account_id})
            )

            print("\n=== Step 5: Test notes_by_account query ===")
            notes_query = gql(
                """
                query ($account: Int!) {
                    notesByAccount(idAccount: $account) {
                        id
                        height
                        pool
                        value
                        address
                        scope
                    }
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(notes_query, variable_values={"account": test_account_id})
            )
            notes = result["notesByAccount"]
            print(f"Found {len(notes)} notes for test account")
            assert len(notes) >= 1, "Should have at least one note"
            for note in notes[:3]:
                print(f"  - Note ID: {note['id']}, Value: {note['value']} ZEC, Pool: {note['pool']}, Scope: {note['scope']}")
            assert notes[0]["value"] == "0.05000000", f"Expected 0.05 ZEC, got {notes[0]['value']}"

            print("\n=== Step 6: Test memos_by_transaction query ===")
            # First get the transaction ID
            transactions_query = gql(
                """
                query ($account: Int!) {
                    transactionsByAccount(idAccount: $account) {
                        id
                        txid
                    }
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(transactions_query, variable_values={"account": test_account_id})
            )
            transactions = result["transactionsByAccount"]
            assert len(transactions) >= 1, "Should have at least one transaction"
            tx_id = transactions[0]["id"]

            memos_query = gql(
                """
                query ($account: Int!, $tx: Int!) {
                    memosByTransaction(idAccount: $account, idTransaction: $tx)
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(memos_query, variable_values={"account": test_account_id, "tx": tx_id})
            )
            memos = result["memosByTransaction"]
            print(f"Found {len(memos)} memos for transaction {tx_id}")
            if len(memos) > 0:
                for memo in memos:
                    print(f"  - Memo: {memo}")

            print("\n=== Step 7: Send transaction with memo ===")
            # Create another account to send to
            result = await client.execute_async(
                GraphQLRequest(create_account_mutation, variable_values={"name": "Receiver"})
            )
            receiver_id = int(result["createAccount"])
            print(f"Created receiver account: {receiver_id}")

            result = await client.execute_async(
                GraphQLRequest(address_query, variable_values={"account": receiver_id})
            )
            receiver_address = result["addressByAccount"]["orchard"]

            # Send with memo (using PaymentMemoInput)
            pay_with_memo_mutation = gql(
                """
                mutation ($account: Int!, $address: String!, $amount: BigDecimal!, $memo: String!) {
                    pay(idAccount: $account, payment: {
                        recipients: [{
                            address: $address
                            amount: $amount
                            memo: $memo
                        }]
                    })
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(
                    pay_with_memo_mutation,
                    variable_values={
                        "account": test_account_id,
                        "address": receiver_address,
                        "amount": "0.025",
                        "memo": "Test memo for account management"
                    }
                )
            )
            txid2 = result["pay"]
            print(f"Sent 0.025 ZEC with memo, txid: {txid2}")

            # Mine blocks and sync
            height_before = await get_current_height(client)
            await mine_blocks(rpc_url, 5)
            await wait_for_blocks(client, height_before, 5)

            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": receiver_id})
            )

            # Check if memo was stored
            result = await client.execute_async(
                GraphQLRequest(transactions_query, variable_values={"account": receiver_id})
            )
            receiver_txs = result["transactionsByAccount"]
            if len(receiver_txs) > 0:
                receiver_tx_id = receiver_txs[0]["id"]
                result = await client.execute_async(
                    GraphQLRequest(
                        memos_query,
                        variable_values={"account": receiver_id, "tx": receiver_tx_id}
                    )
                )
                memos = result["memosByTransaction"]
                print(f"Found {len(memos)} memos for receiver transaction")
                if len(memos) > 0:
                    print(f"  Memo: {memos[0]}")

            print("\n=== Step 8: Test reset_account mutation ===")
            # Get current sync height
            result = await client.execute_async(
                GraphQLRequest(accounts_query, variable_values={"id": test_account_id})
            )
            original_height = result["accounts"][0]["height"]
            print(f"Current sync height: {original_height}")

            reset_mutation = gql(
                """
                mutation ($account: Int!) {
                    resetAccount(idAccount: $account)
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(reset_mutation, variable_values={"account": test_account_id})
            )
            assert result["resetAccount"] == True, "resetAccount should return true"
            print("✓ Reset account synchronization")

            # Verify height was reset
            result = await client.execute_async(
                GraphQLRequest(accounts_query, variable_values={"id": test_account_id})
            )
            reset_height = result["accounts"][0]["height"]
            print(f"Height after reset: {reset_height}")
            assert reset_height < original_height, f"Height should be lower after reset, was {original_height}, now {reset_height}"

            print("\n=== Step 9: Re-sync account after reset ===")
            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": test_account_id})
            )
            result = await client.execute_async(
                GraphQLRequest(accounts_query, variable_values={"id": test_account_id})
            )
            new_height = result["accounts"][0]["height"]
            print(f"Height after re-sync: {new_height}")
            assert new_height > reset_height, "Height should increase after re-sync"

            print("\n=== Step 10: Test delete_account mutation ===")
            delete_mutation = gql(
                """
                mutation ($account: Int!) {
                    deleteAccount(idAccount: $account)
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(delete_mutation, variable_values={"account": receiver_id})
            )
            assert result["deleteAccount"] == True, "deleteAccount should return true"
            print("✓ Deleted receiver account")

            # Verify account is deleted
            result = await client.execute_async(GraphQLRequest(gql("query { accounts { id name } }")))
            accounts = result["accounts"]
            account_names = [a["name"] for a in accounts]
            assert "Receiver" not in account_names, "Receiver account should be deleted"
            print("✓ Verified receiver account is deleted")

            print("\n✅ Account management test passed!")

    finally:
        # Dump server log for debugging (includes Rust backtrace on panic)
        _dump_log()
        await stop_zkool_instance(process)
        cleanup_test_files(DB_PATH, LOG_PATH)
