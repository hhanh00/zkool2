"""Smoke test for zkool GraphQL API including subscriptions."""

import asyncio
import os

import pytest
from gql import Client, GraphQLRequest, gql
from gql.transport.websockets import WebsocketsTransport

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
        await kill_existing_zkool_processes()
        await asyncio.sleep(2)  # Give time for processes to fully terminate

        print(f"Starting zkool_graphql on port {PORT}")
        process = await start_zkool_instance(zkool_binary, DB_PATH, PORT, LWD_URL, LOG_PATH)
        await asyncio.sleep(3)  # Give server more time to fully start

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
        await stop_zkool_instance(process)
        cleanup_test_files(DB_PATH, LOG_PATH)


@pytest.mark.asyncio
async def test_subscriptions(gql_client_factory, rpc_url, seed, zkool_binary, ws_url):
    """Test WebSocket subscriptions for transaction and block events."""
    if not seed:
        pytest.skip("SEED not set")

    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    PORT = 8000
    LWD_URL = "http://localhost:8137"
    DB_PATH = "/tmp/regtest_subscriptions.db"
    LOG_PATH = "/tmp/graphql_subscriptions.log"
    GRAPHQL_URL = f"http://localhost:{PORT}/graphql"

    process = None

    try:
        await kill_existing_zkool_processes()
        await asyncio.sleep(2)  # Give time for processes to fully terminate

        print(f"Starting zkool_graphql on port {PORT}")
        process = await start_zkool_instance(zkool_binary, DB_PATH, PORT, LWD_URL, LOG_PATH)
        await asyncio.sleep(3)  # Give server more time to fully start

        async with gql_client_factory(GRAPHQL_URL) as client:
            # Create funded wallet
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

            # Create two receiver accounts
            create_account_mutation2 = gql(
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
                GraphQLRequest(create_account_mutation2, variable_values={"name": "Receiver1"})
            )
            receiver1_id = int(result["createAccount"])
            print(f"Created receiver 1: {receiver1_id}")

            result = await client.execute_async(
                GraphQLRequest(create_account_mutation2, variable_values={"name": "Receiver2"})
            )
            receiver2_id = int(result["createAccount"])
            print(f"Created receiver 2: {receiver2_id}")

            # Get addresses
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
                GraphQLRequest(address_query, variable_values={"account": receiver1_id})
            )
            receiver1_address = result["addressByAccount"]["orchard"]
            print(f"Receiver 1 address: {receiver1_address}")

            result = await client.execute_async(
                GraphQLRequest(address_query, variable_values={"account": receiver2_id})
            )
            receiver2_address = result["addressByAccount"]["orchard"]
            print(f"Receiver 2 address: {receiver2_address}")

            print("\n=== Test 1: Outgoing transaction subscription ===")

            # Use WebsocketsTransport with ping handling disabled
            transport = WebsocketsTransport(
                url=ws_url,
                connect_timeout=60,
                close_timeout=60,
                ping_interval=None,  # Disable client-side pings
            )
            ws_client = Client(transport=transport, fetch_schema_from_transport=False)

            async with ws_client as ws:
                # Subscribe to events for funding account
                events_subscription = gql(
                    """
                    subscription ($id_account: Int!) {
                        events(idAccount: $id_account) {
                            type
                            height
                            txid
                            value
                            dkgAccount
                            notes {
                                address
                                value
                                memo
                            }
                        }
                    }
                    """
                )

                events_received = []

                async def event_receiver():
                    req = GraphQLRequest(events_subscription, variable_values={"id_account": funding_id})
                    async for event in ws.subscribe(req):
                        events_received.append(event["events"])
                        print(f"Received event: type={event['events']['type']}, txid={event['events'].get('txid', 'N/A')}, height={event['events']['height']}")
                        # Collect at least 2 events (Tx and Block)
                        if len(events_received) >= 2:
                            break

                # Start receiving events in background
                receiver_task = asyncio.create_task(event_receiver())

                # Give subscription time to establish
                await asyncio.sleep(1)

                # Send transaction (should trigger Tx event BEFORE mining)
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
                        variable_values={"account": funding_id, "address": receiver1_address, "amount": "1.0"},
                    )
                )
                txid = result["pay"]
                print(f"Sent transaction, txid: {txid}")

                # Wait a moment for the Tx event (should arrive before mining)
                await asyncio.sleep(2)

                # Mine a block (should trigger Block event)
                height_before = await get_current_height(client)
                await mine_blocks(rpc_url, 1)

                # Wait for subscription events
                try:
                    await asyncio.wait_for(receiver_task, timeout=10)
                except asyncio.TimeoutError:
                    print(f"Received {len(events_received)} events before timeout")
                    if events_received:
                        for i, e in enumerate(events_received):
                            print(f"  Event {i+1}: {e}")
                    pytest.fail("Did not receive expected events before timeout")

                assert len(events_received) >= 1, "Should receive at least one event"

                # Check first event should be Tx type with height < 0 (unconfirmed)
                tx_event = None
                block_event = None
                for event in events_received:
                    if event["type"] == "Tx":
                        tx_event = event
                    elif event["type"] == "Block":
                        block_event = event

                if tx_event:
                    print(f"Tx event: txid={tx_event['txid']}, height={tx_event['height']}, value={tx_event['value']}")
                    assert tx_event["txid"].lower() == txid.lower(), "Tx event txid should match sent txid"
                    assert tx_event["height"] < 0, f"Tx height should be negative (unconfirmed), got {tx_event['height']}"

                if block_event:
                    print(f"Block event: height={block_event['height']}")
                    assert block_event["height"] == height_before + 1, f"Block height should be {height_before + 1}"

            print("✅ Outgoing transaction subscription test passed!")

            print("\n=== Test 2: Incoming transaction subscription ===")
            transport = WebsocketsTransport(
                url=ws_url,
                connect_timeout=60,
                close_timeout=60,
                ping_interval=None,
            )
            ws_client = Client(transport=transport, fetch_schema_from_transport=False)

            async with ws_client as ws:
                # Subscribe to events for receiver2 (incoming)
                events_received = []

                async def receiver2_listener():
                    req = GraphQLRequest(events_subscription, variable_values={"id_account": receiver2_id})
                    async for event in ws.subscribe(req):
                        events_received.append(event["events"])
                        print(f"Receiver2 event: type={event['events']['type']}, txid={event['events'].get('txid', 'N/A')}")
                        if len(events_received) >= 2:  # Tx + Block
                            break

                receiver_task = asyncio.create_task(receiver2_listener())
                await asyncio.sleep(1)

                # Send transaction to receiver2
                result = await client.execute_async(
                    GraphQLRequest(
                        pay_mutation,
                        variable_values={"account": funding_id, "address": receiver2_address, "amount": "0.5"},
                    )
                )
                txid2 = result["pay"]
                print(f"Sent transaction to receiver2, txid: {txid2}")

                await asyncio.sleep(2)

                # Mine a block
                height_before = await get_current_height(client)
                await mine_blocks(rpc_url, 1)

                # Wait for events
                try:
                    await asyncio.wait_for(receiver_task, timeout=10)
                except asyncio.TimeoutError:
                    print(f"Received {len(events_received)} events before timeout")
                    pytest.fail("Did not receive incoming transaction events before timeout")

                assert len(events_received) >= 1, "Should receive at least one event"

                tx_event = next((e for e in events_received if e["type"] == "Tx"), None)
                if tx_event:
                    print(f"Incoming Tx event: txid={tx_event['txid']}, height={tx_event['height']}, value={tx_event['value']}")
                    assert tx_event["txid"].lower() == txid2.lower(), "Tx event txid should match sent txid"

            print("✅ Incoming transaction subscription test passed!")

            print("\n=== Test 3: Block events subscription ===")
            transport = WebsocketsTransport(
                url=ws_url,
                connect_timeout=60,
                close_timeout=60,
                ping_interval=None,
            )
            ws_client = Client(transport=transport, fetch_schema_from_transport=False)

            async with ws_client as ws:
                blocks_received = []

                async def block_listener():
                    req = GraphQLRequest(events_subscription, variable_values={"id_account": funding_id})
                    async for event in ws.subscribe(req):
                        event_data = event["events"]
                        if event_data["type"] == "Block":
                            blocks_received.append(event_data)
                            print(f"Block event: height={event_data['height']}")
                            if len(blocks_received) >= 3:
                                break

                receiver_task = asyncio.create_task(block_listener())
                await asyncio.sleep(1)

                # Mine 3 blocks
                height_before = await get_current_height(client)
                print(f"Height before mining: {height_before}")

                await mine_blocks(rpc_url, 3)
                print("Mined 3 blocks")

                # Wait for block events
                try:
                    await asyncio.wait_for(receiver_task, timeout=15)
                except asyncio.TimeoutError:
                    print(f"Received {len(blocks_received)} block events before timeout")
                    pytest.fail("Did not receive all block events before timeout")

                assert len(blocks_received) >= 3, f"Should receive at least 3 block events, got {len(blocks_received)}"

                # Verify block heights
                for i, block in enumerate(blocks_received):
                    expected_height = height_before + i + 1
                    assert block["height"] == expected_height, (
                        f"Block {i+1} height should be {expected_height}, got {block['height']}"
                    )

            print("✅ Block events subscription test passed!")

            print("\n=== Test 4: Multiple accounts subscription ===")
            # Subscribe to both receiver1 and receiver2 simultaneously
            transport = WebsocketsTransport(
                url=ws_url,
                connect_timeout=60,
                close_timeout=60,
                ping_interval=None,
            )
            ws_client = Client(transport=transport, fetch_schema_from_transport=False)

            async with ws_client as ws:
                receiver1_events = []
                receiver2_events = []

                async def receiver1_listener():
                    req = GraphQLRequest(events_subscription, variable_values={"id_account": receiver1_id})
                    async for event in ws.subscribe(req):
                        event_data = event["events"]
                        if event_data["type"] == "Tx":
                            receiver1_events.append(event_data)
                            print(f"Receiver1 Tx event: txid={event_data['txid']}")
                            break

                async def receiver2_listener():
                    req = GraphQLRequest(events_subscription, variable_values={"id_account": receiver2_id})
                    async for event in ws.subscribe(req):
                        event_data = event["events"]
                        if event_data["type"] == "Tx":
                            receiver2_events.append(event_data)
                            print(f"Receiver2 Tx event: txid={event_data['txid']}")
                            break

                # Start both listeners
                task1 = asyncio.create_task(receiver1_listener())
                task2 = asyncio.create_task(receiver2_listener())
                await asyncio.sleep(1)

                # Send to both accounts in a single transaction
                pay_multiple_mutation = gql(
                    """
                    mutation ($account: Int!, $recipients: [Recipient!]!) {
                        pay(idAccount: $account, payment: {
                            recipients: $recipients
                        })
                    }
                    """
                )
                result = await client.execute_async(
                    GraphQLRequest(
                        pay_multiple_mutation,
                        variable_values={
                            "account": funding_id,
                            "recipients": [
                                {"address": receiver1_address, "amount": "0.1"},
                                {"address": receiver2_address, "amount": "0.1"},
                            ],
                        },
                    )
                )
                txid3 = result["pay"]
                print(f"Sent transaction to both receivers, txid: {txid3}")

                # Wait for both subscription events
                try:
                    await asyncio.wait_for(asyncio.gather(task1, task2), timeout=10)
                except asyncio.TimeoutError:
                    print(f"Receiver1 events: {len(receiver1_events)}, Receiver2 events: {len(receiver2_events)}")
                    pytest.fail("Did not receive all transaction events before timeout")

                assert len(receiver1_events) >= 1, "Receiver1 should receive event"
                assert len(receiver2_events) >= 1, "Receiver2 should receive event"

                # Both should have the same txid (same transaction)
                assert receiver1_events[0]["txid"].lower() == txid3.lower()
                assert receiver2_events[0]["txid"].lower() == txid3.lower()

                print(f"Receiver1 got: {receiver1_events[0]}")
                print(f"Receiver2 got: {receiver2_events[0]}")

            print("✅ Multiple accounts subscription test passed!")

            print("\n=== Test 5: Transaction before mining ===")
            # Verify that transaction events arrive BEFORE the transaction is mined
            transport = WebsocketsTransport(
                url=ws_url,
                connect_timeout=60,
                close_timeout=60,
                ping_interval=None,
            )
            ws_client = Client(transport=transport, fetch_schema_from_transport=False)

            async with ws_client as ws:
                events_in_order = []

                async def order_tracker():
                    req = GraphQLRequest(events_subscription, variable_values={"id_account": funding_id})
                    async for event in ws.subscribe(req):
                        event_data = event["events"]
                        events_in_order.append((event_data["type"], event_data["height"]))
                        print(f"Event: type={event_data['type']}, height={event_data['height']}")
                        if len(events_in_order) >= 2:
                            break

                receiver_task = asyncio.create_task(order_tracker())
                await asyncio.sleep(1)

                # Send transaction
                result = await client.execute_async(
                    GraphQLRequest(
                        pay_mutation,
                        variable_values={"account": funding_id, "address": receiver1_address, "amount": "0.05"},
                    )
                )
                print("Transaction sent, waiting for Tx event...")

                # Wait for Tx event (should be before mining)
                await asyncio.sleep(3)

                # Now mine a block
                await mine_blocks(rpc_url, 1)

                try:
                    await asyncio.wait_for(receiver_task, timeout=10)
                except asyncio.TimeoutError:
                    print(f"Received events in order: {events_in_order}")
                    pytest.fail("Did not receive events in expected order")

                # Verify order: Tx event should come before Block event
                assert len(events_in_order) >= 2, "Should receive at least 2 events"

                # First event should be Tx with negative height (unconfirmed)
                first_type, first_height = events_in_order[0]
                assert first_type == "Tx", f"First event should be Tx, got {first_type}"
                assert first_height < 0, f"Tx event should have negative height (unconfirmed), got {first_height}"

                print(f"✅ Events received in correct order: {events_in_order}")

            print("\n✅ All subscription tests passed!")

    finally:
        await stop_zkool_instance(process)
        cleanup_test_files(DB_PATH, LOG_PATH)
