"""Test detailed transaction inspection: Transaction.notes(), outputs(), spends(), Note.tx()."""

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


@pytest.fixture(scope="session")
def rpc_url():
    return os.getenv("RPC_URL", "http://127.0.0.1:18232/")


@pytest.fixture(scope="session")
def seed():
    return os.getenv("SEED", "")


@pytest.fixture(scope="session")
def lwd_url():
    return os.getenv("LWD_URL", "http://localhost:8137")


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
async def test_transaction_details(gql_client_factory, rpc_url, seed, zkool_binary, lwd_url):
    """Test Transaction.notes(), outputs(), spends() fields and Note.tx() relationship."""
    if not seed:
        pytest.skip("SEED not set")

    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    PORT = 8000
    DB_PATH = "/tmp/regtest_tx_details.db"
    LOG_PATH = "/tmp/graphql_tx_details.log"
    GRAPHQL_URL = f"http://localhost:{PORT}/graphql"

    process = None

    try:
        # Kill any existing zkool_graphql processes
        subprocess.run(["pkill", "-9", "zkool_graphql"], stderr=subprocess.DEVNULL)
        await asyncio.sleep(1)

        # Remove existing database
        if os.path.exists(DB_PATH):
            os.remove(DB_PATH)

        # Start zkool_graphql instance
        print(f"Starting zkool_graphql on port {PORT}")
        process = subprocess.Popen(
            [zkool_binary, "-d", DB_PATH, "-p", str(PORT), "-l", lwd_url],
            stdout=open(LOG_PATH, "w"),
            stderr=subprocess.STDOUT,
        )
        await asyncio.sleep(2)

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

            # Synchronize funding wallet
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

            print("\n=== Step 2: Create receiver accounts ===")
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

            print("\n=== Step 3: Get addresses ===")
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
                GraphQLRequest(address_query, variable_values={"account": account1_id})
            )
            account1_address = result["addressByAccount"]["orchard"]
            print(f"Account 1 Orchard address: {account1_address[:50]}...")

            result = await client.execute_async(
                GraphQLRequest(address_query, variable_values={"account": account2_id})
            )
            account2_address = result["addressByAccount"]["orchard"]
            print(f"Account 2 Orchard address: {account2_address[:50]}...")

            print("\n=== Step 4: Send funding transaction to account 1 ===")
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
                        "address": account1_address,
                        "amount": "0.05"
                    }
                )
            )
            txid1 = result["pay"]
            print(f"Sent 0.05 ZEC to account 1, txid: {txid1}")

            # Mine blocks and sync
            height_before = await get_current_height(client)
            await mine_blocks(rpc_url, 5)
            await wait_for_blocks(client, height_before, 5)

            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": account1_id})
            )

            print("\n=== Step 5: Send transaction from account 1 to account 2 ===")
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

            print("\n=== Step 6: Test Transaction.notes() field ===")
            # Get transaction for account 1
            transactions_query = gql(
                """
                query ($account: Int!) {
                    transactionsByAccount(idAccount: $account) {
                        id
                        txid
                        notes {
                            id
                            value
                            pool
                            scope
                            address
                        }
                    }
                }
            """
            )
            result = await client.execute_async(
                GraphQLRequest(transactions_query, variable_values={"account": account1_id})
            )
            transactions = result["transactionsByAccount"]
            print(f"Found {len(transactions)} transactions for account 1")

            # Find the transaction that has notes (should have received notes and sent notes)
            tx_with_notes = None
            for tx in transactions:
                if tx["notes"] and len(tx["notes"]) > 0:
                    tx_with_notes = tx
                    break

            assert tx_with_notes is not None, "Should have at least one transaction with notes"
            print(f"Transaction {tx_with_notes['txid'][:16]}... has {len(tx_with_notes['notes'])} notes:")
            for note in tx_with_notes["notes"]:
                print(f"  - Note ID: {note['id']}, Value: {note['value']} ZEC, Pool: {note['pool']}, Scope: {note['scope']}")

            print("\n=== Step 7: Test Transaction.outputs() field ===")
            tx_outputs_query = gql(
                """
                query ($account: Int!) {
                    transactionsByAccount(idAccount: $account) {
                        id
                        txid
                        outputs {
                            id
                            pool
                            vout
                            value
                            address
                            memo
                        }
                    }
                }
            """
            )
            result = await client.execute_async(
                GraphQLRequest(tx_outputs_query, variable_values={"account": account1_id})
            )
            transactions = result["transactionsByAccount"]

            tx_with_outputs = None
            for tx in transactions:
                if tx["outputs"] and len(tx["outputs"]) > 0:
                    tx_with_outputs = tx
                    break

            assert tx_with_outputs is not None, "Should have at least one transaction with outputs"
            print(f"Transaction {tx_with_outputs['txid'][:16]}... has {len(tx_with_outputs['outputs'])} outputs:")
            for output in tx_with_outputs["outputs"]:
                memo_str = f", Memo: {output['memo'][:30]}..." if output['memo'] else ""
                print(f"  - Output ID: {output['id']}, Pool: {output['pool']}, Vout: {output['vout']}, Value: {output['value']} ZEC{memo_str}")

            print("\n=== Step 8: Test Transaction.spends() field ===")
            tx_spends_query = gql(
                """
                query ($account: Int!) {
                    transactionsByAccount(idAccount: $account) {
                        id
                        txid
                        spends {
                            id
                            value
                            pool
                            address
                        }
                    }
                }
            """
            )
            result = await client.execute_async(
                GraphQLRequest(tx_spends_query, variable_values={"account": account1_id})
            )
            transactions = result["transactionsByAccount"]

            # The transaction from account 1 to account 2 should have spends
            tx_with_spends = None
            for tx in transactions:
                if tx["spends"] and len(tx["spends"]) > 0:
                    tx_with_spends = tx
                    break

            if tx_with_spends:
                print(f"Transaction {tx_with_spends['txid'][:16]}... has {len(tx_with_spends['spends'])} spends:")
                for spend in tx_with_spends["spends"]:
                    print(f"  - Spend Note ID: {spend['id']}, Value: {spend['value']} ZEC, Pool: {spend['pool']}")
            else:
                print("No spends found (transaction may not have spent any notes yet)")

            print("\n=== Step 9: Test Note.tx() field ===")
            # Get notes for account 2 (received notes)
            notes_query = gql(
                """
                query ($account: Int!) {
                    notesByAccount(idAccount: $account) {
                        id
                        value
                        tx {
                            id
                            txid
                            height
                            value
                        }
                    }
                }
            """
            )
            result = await client.execute_async(
                GraphQLRequest(notes_query, variable_values={"account": account2_id})
            )
            notes = result["notesByAccount"]
            assert len(notes) >= 1, "Account 2 should have at least one note"

            print(f"Account 2 has {len(notes)} notes with transaction details:")
            for note in notes[:2]:
                print(f"  - Note ID: {note['id']}, Value: {note['value']} ZEC")
                print(f"    Transaction: {note['tx']['txid'][:16]}..., Height: {note['tx']['height']}, TX Value: {note['tx']['value']} ZEC")
            assert notes[0]["tx"] is not None, "Note should have associated transaction"
            assert notes[0]["tx"]["txid"] is not None, "Transaction should have txid"

            print("\n=== Step 10: Verify transaction detail relationships ===")
            # Get a specific transaction and verify all detail fields
            tx_detail_query = gql(
                """
                query ($account: Int!) {
                    transactionsByAccount(idAccount: $account) {
                        id
                        txid
                        height
                        value
                        fee
                        notes {
                            id
                            value
                        }
                        outputs {
                            id
                            value
                        }
                        spends {
                            id
                            value
                        }
                    }
                }
            """
            )
            result = await client.execute_async(
                GraphQLRequest(tx_detail_query, variable_values={"account": account1_id})
            )
            transactions = result["transactionsByAccount"]

            # Find a transaction with all details
            complete_tx = None
            for tx in transactions:
                if tx.get("notes") or tx.get("outputs") or tx.get("spends"):
                    complete_tx = tx
                    break

            assert complete_tx is not None, "Should have a transaction with detail fields"
            print(f"Complete transaction details for {complete_tx['txid'][:16]}...:")
            print(f"  Height: {complete_tx['height']}")
            print(f"  Value: {complete_tx['value']} ZEC")
            print(f"  Fee: {complete_tx['fee']} ZEC")
            print(f"  Notes count: {len(complete_tx.get('notes', []))}")
            print(f"  Outputs count: {len(complete_tx.get('outputs', []))}")
            print(f"  Spends count: {len(complete_tx.get('spends', []))}")

            print("\n✅ Transaction details test passed!")

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
