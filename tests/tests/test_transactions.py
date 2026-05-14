"""Test transaction history, account listing, address generation, and all address pools."""

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
async def test_transactions_and_addresses(gql_client_factory, rpc_url, seed, zkool_binary, lwd_url):
    """Test transaction history, account listing, address generation, and all address pools."""
    if not seed:
        pytest.skip("SEED not set")

    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    PORT = 8000
    DB_PATH = "/tmp/regtest_transactions.db"
    LOG_PATH = "/tmp/graphql_transactions.log"
    GRAPHQL_URL = f"http://localhost:{PORT}/graphql"

    process = None

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

            print("\n=== Step 2: Create test accounts ===")
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
                GraphQLRequest(create_account_mutation, variable_values={"name": "Account1"})
            )
            account1_id = int(result["createAccount"])
            print(f"Created account 1: {account1_id}")

            result = await client.execute_async(
                GraphQLRequest(create_account_mutation, variable_values={"name": "Account2"})
            )
            account2_id = int(result["createAccount"])
            print(f"Created account 2: {account2_id}")

            result = await client.execute_async(
                GraphQLRequest(create_account_mutation, variable_values={"name": "Account3"})
            )
            account3_id = int(result["createAccount"])
            print(f"Created account 3: {account3_id}")

            print("\n=== Step 3: Test accounts query (list all accounts) ===")
            accounts_query = gql(
                """
                query {
                    accounts {
                        id
                        name
                    }
                }
                """
            )
            result = await client.execute_async(GraphQLRequest(accounts_query))
            accounts = result["accounts"]
            print(f"Found {len(accounts)} accounts:")
            for account in accounts:
                print(f"  - ID: {account['id']}, Name: {account['name']}")
            assert len(accounts) == 4, f"Expected 4 accounts, got {len(accounts)}"
            account_names = [a["name"] for a in accounts]
            assert "Funding" in account_names
            assert "Account1" in account_names
            assert "Account2" in account_names
            assert "Account3" in account_names

            print("\n=== Step 4: Test accounts query with filter ===")
            accounts_filter_query = gql(
                """
                query ($name: String!) {
                    accounts(accountFilter: {name: $name}) {
                        id
                        name
                    }
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(accounts_filter_query, variable_values={"name": "Account1"})
            )
            filtered = result["accounts"]
            assert len(filtered) == 1, f"Expected 1 account, got {len(filtered)}"
            assert filtered[0]["id"] == account1_id
            assert filtered[0]["name"] == "Account1"

            print("\n=== Step 5: Get addresses for all pools ===")
            address_query = gql(
                """
                query ($account: Int!) {
                    addressByAccount(idAccount: $account) {
                        ua
                        transparent
                        sapling
                        orchard
                    }
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(address_query, variable_values={"account": account1_id})
            )
            addresses = result["addressByAccount"]
            print(f"Account 1 addresses:")
            print(f"  Unified: {addresses['ua'][:50]}..." if addresses['ua'] else "  Unified: None")
            print(f"  Transparent: {addresses['transparent'][:50]}..." if addresses['transparent'] else "  Transparent: None")
            print(f"  Sapling: {addresses['sapling'][:50]}..." if addresses['sapling'] else "  Sapling: None")
            print(f"  Orchard: {addresses['orchard'][:50]}..." if addresses['orchard'] else "  Orchard: None")

            assert addresses["orchard"], "Orchard address should be present"

            print("\n=== Step 6: Test new_addresses mutation ===")
            new_addresses_mutation = gql(
                """
                mutation ($account: Int!) {
                    newAddresses(idAccount: $account) {
                        ua
                        transparent
                        sapling
                        orchard
                        diversifierIndex
                    }
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(new_addresses_mutation, variable_values={"account": account1_id})
            )
            new_addresses = result["newAddresses"]
            print(f"Generated new addresses for account 1, diversifier index: {new_addresses['diversifierIndex']}")
            assert new_addresses["orchard"], "New Orchard address should be present"

            print("\n=== Step 7: Send transaction from funding to account 1 ===")
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
                        "address": addresses["orchard"],
                        "amount": "0.05"
                    }
                )
            )
            txid = result["pay"]
            print(f"Sent 0.05 ZEC, txid: {txid}")

            # Mine blocks
            height_before = await get_current_height(client)
            await mine_blocks(rpc_url, 5)
            await wait_for_blocks(client, height_before, 5)

            # Sync account 1
            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": account1_id})
            )

            print("\n=== Step 8: Test balance_by_account with all pools ===")
            balance_query = gql(
                """
                query ($account: Int!) {
                    balanceByAccount(idAccount: $account) {
                        height
                        transparent
                        sapling
                        orchard
                        total
                    }
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(balance_query, variable_values={"account": account1_id})
            )
            balance = result["balanceByAccount"]
            print(f"Account 1 balance:")
            print(f"  Height: {balance['height']}")
            print(f"  Transparent: {balance['transparent']} ZEC")
            print(f"  Sapling: {balance['sapling']} ZEC")
            print(f"  Orchard: {balance['orchard']} ZEC")
            print(f"  Total: {balance['total']} ZEC")
            assert balance["orchard"] == "0.05000000", f"Expected 0.05 ZEC in Orchard, got {balance['orchard']}"

            print("\n=== Step 9: Test transactions_by_account query ===")
            transactions_query = gql(
                """
                query ($account: Int!, $height: Int) {
                    transactionsByAccount(idAccount: $account, height: $height) {
                        id
                        txid
                        height
                        value
                        fee
                    }
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(transactions_query, variable_values={"account": account1_id, "height": None})
            )
            transactions = result["transactionsByAccount"]
            print(f"Found {len(transactions)} transactions for account 1")
            assert len(transactions) >= 1, "Should have at least one transaction"
            for tx in transactions[:3]:
                print(f"  - TX: {tx['txid']}, Height: {tx['height']}, Value: {tx['value']} ZEC")

            print("\n=== Step 10: Test transaction_by_id query ===")
            transaction_by_id_query = gql(
                """
                query ($account: Int!, $txid: String!) {
                    transactionById(idAccount: $account, txid: $txid) {
                        id
                        txid
                        height
                        value
                        fee
                    }
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(
                    transaction_by_id_query,
                    variable_values={"account": account1_id, "txid": txid}
                )
            )
            transaction = result["transactionById"]
            print(f"Fetched transaction by ID:")
            print(f"  TXID: {transaction['txid']}")
            print(f"  Height: {transaction['height']}")
            print(f"  Value: {transaction['value']} ZEC")
            print(f"  Fee: {transaction['fee']} ZEC")
            assert transaction["txid"] == txid.lower()

            print("\n=== Step 11: Send transaction from account 1 to account 2 ===")
            result = await client.execute_async(
                GraphQLRequest(address_query, variable_values={"account": account2_id})
            )
            account2_address = result["addressByAccount"]["orchard"]

            result = await client.execute_async(
                GraphQLRequest(
                    pay_mutation,
                    variable_values={
                        "account": account1_id,
                        "address": account2_address,
                        "amount": "0.025"
                    }
                )
            )
            txid2 = result["pay"]
            print(f"Sent 0.025 ZEC from account 1 to account 2, txid: {txid2}")

            # Mine blocks and sync
            height_before = await get_current_height(client)
            await mine_blocks(rpc_url, 5)
            await wait_for_blocks(client, height_before, 5)

            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": account1_id})
            )
            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": account2_id})
            )

            print("\n=== Step 12: Verify transaction history for both accounts ===")
            result = await client.execute_async(
                GraphQLRequest(transactions_query, variable_values={"account": account1_id, "height": None})
            )
            account1_txs = result["transactionsByAccount"]
            print(f"Account 1 has {len(account1_txs)} transactions")

            result = await client.execute_async(
                GraphQLRequest(transactions_query, variable_values={"account": account2_id, "height": None})
            )
            account2_txs = result["transactionsByAccount"]
            print(f"Account 2 has {len(account2_txs)} transactions")
            assert len(account2_txs) >= 1, "Account 2 should have at least one received transaction"

            print("\n=== Step 13: Test transactions_by_account with height filter ===")
            result = await client.execute_async(
                GraphQLRequest(
                    transactions_query,
                    variable_values={"account": account1_id, "height": 1}
                )
            )
            all_txs = result["transactionsByAccount"]

            result = await client.execute_async(GraphQLRequest(gql("query { currentHeight }")))
            current_height = result["currentHeight"]

            result = await client.execute_async(
                GraphQLRequest(
                    transactions_query,
                    variable_values={"account": account1_id, "height": current_height - 1}
                )
            )
            filtered_txs = result["transactionsByAccount"]
            print(f"Transactions from height {current_height - 1}: {len(filtered_txs)}")
            print(f"All transactions: {len(all_txs)}")
            assert len(filtered_txs) <= len(all_txs), "Filtered should have fewer or equal transactions"

            print("\n✅ Transactions and addresses test passed!")

    finally:
        await stop_zkool_instance(process)
        cleanup_test_files(DB_PATH, LOG_PATH)
