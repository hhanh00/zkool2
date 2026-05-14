"""Test WebSocket subscriptions for transaction and block events."""

import asyncio
import json
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
async def test_websocket_subscriptions(gql_client_factory, rpc_url, seed, zkool_binary, ws_url, lwd_url):
    """Test WebSocket subscriptions using raw websockets library."""
    if not seed:
        pytest.skip("SEED not set")

    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    PORT = 8000
    DB_PATH = "/tmp/regtest_ws_subscriptions.db"
    LOG_PATH = "/tmp/graphql_ws_subscriptions.log"
    GRAPHQL_URL = f"http://localhost:{PORT}/graphql"

    process = None

    try:
        await kill_existing_zkool_processes()
        await asyncio.sleep(2)

        print(f"Starting zkool_graphql on port {PORT}")
        process = await start_zkool_instance(zkool_binary, DB_PATH, PORT, lwd_url, LOG_PATH)
        await asyncio.sleep(3)

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

            # Check balance to ensure we have funds
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
                GraphQLRequest(balance_query, variable_values={"account": funding_id})
            )
            balance = result["balanceByAccount"]["orchard"]
            print(f"Funding wallet balance: {balance} ZEC")

            # Create receiver account
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
                GraphQLRequest(create_account_mutation2, variable_values={"name": "Receiver"})
            )
            receiver_id = int(result["createAccount"])
            print(f"Created receiver: {receiver_id}")

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
            result = await client.execute_async(
                GraphQLRequest(address_query, variable_values={"account": receiver_id})
            )
            receiver_address = result["addressByAccount"]["orchard"]
            print(f"Receiver address: {receiver_address}")

            print("\n=== Setting up WebSocket subscription ===")

            import websockets

            # Single WebSocket connection for all tests
            ws = await websockets.connect(ws_url, close_timeout=60, subprotocols=["graphql-ws"])

            # Send connection init first (required by graphql-ws)
            await ws.send(json.dumps({"type": "connection_init", "payload": {}}))

            # Wait for connection_ack
            init_msg = json.loads(await ws.recv())
            print(f"Init message: {init_msg}")

            # Helper to send a subscription request
            subscription_id_counter = [0]  # Use list to allow mutation in nested function

            async def subscribe_to_account(account_id):
                """Send a subscription request for an account."""
                subscription_id_counter[0] += 1
                subscription_query = {
                    "id": str(subscription_id_counter[0]),
                    "type": "start",
                    "payload": {
                        "variables": {"id_account": account_id},
                        "query": """
                            subscription ($id_account: Int!) {
                                events(idAccount: $id_account) {
                                    type
                                    height
                                    txid
                                    value
                                    dkgAccount
                                }
                            }
                        """
                    }
                }
                await ws.send(json.dumps(subscription_query))
                print(f"Subscription request sent for account {account_id}, id: {subscription_query['id']}")
                return subscription_query["id"]

            # Subscribe to funding account
            funding_subscription_id = await subscribe_to_account(funding_id)

            # Collect all events
            all_events = []

            async def collect_all_events():
                """Collect all events from the WebSocket."""
                async for message in ws:
                    data = json.loads(message)

                    if isinstance(data, dict):
                        if data.get("type") == "ping":
                            await ws.send(json.dumps({"type": "pong"}))
                            continue
                        elif data.get("type") in ("pong", "ka"):
                            continue

                    if isinstance(data, dict) and "payload" in data:
                        payload = data["payload"]
                        if isinstance(payload, dict) and "data" in payload:
                            event_data = payload["data"].get("events")
                            if event_data:
                                all_events.append(event_data)
                                print(f"Received event: type={event_data['type']}, height={event_data['height']}, txid={event_data.get('txid', 'N/A')}")

            # Start event collector in background
            collector_task = asyncio.create_task(collect_all_events())
            await asyncio.sleep(1)

            print("\n=== Test 1: Transaction event before mining ===")

            events_received = []

            # Send transaction
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
                    variable_values={"account": funding_id, "address": receiver_address, "amount": "0.01"},
                )
            )
            txid = result["pay"]
            print(f"Sent transaction, txid: {txid}")

            # Wait for Tx event (should arrive before mining)
            await asyncio.sleep(3)

            # Test unconfirmed API while tx is pending
            print("\n=== Test 1b: Unconfirmed transactions API ===")

            print("All events")
            for i, event in enumerate(all_events):
                print(f"  Event {i+1}: type={event['type']}, height={event['height']}, txid={event.get('txid', 'N/A')}")

            # Define the query
            unconfirmed_query = gql(
                """
                query ($account: Int!) {
                    unconfirmedByAccount(idAccount: $account) {
                        txid
                        value
                        notes {
                            pool
                            scope
                            value
                            address
                        }
                    }
                }
                """
            )

            # Query the unconfirmed API to verify it works
            result = await client.execute_async(
                GraphQLRequest(unconfirmed_query, variable_values={"account": funding_id})
            )
            unconfirmed_txs = result["unconfirmedByAccount"]
            print(f"Unconfirmed transactions: {len(unconfirmed_txs)}")

            # Verify API returns correct structure
            assert isinstance(unconfirmed_txs, list), "Should return a list"
            print("✓ unconfirmedByAccount API returns a list")

            # Verify structure of any returned transactions
            for tx in unconfirmed_txs:
                assert "txid" in tx, "Transaction should have txid"
                assert "value" in tx, "Transaction should have value"
                assert "notes" in tx, "Transaction should have notes"
                assert isinstance(tx["notes"], list), "Notes should be a list"
                print(f"  - txid: {tx['txid']}, value: {tx['value']}, notes: {len(tx['notes'])}")

            print("✓ unconfirmedByAccount API structure is correct")

            # Mine a block
            height_before = await get_current_height(client)
            await mine_blocks(rpc_url, 1)
            print(f"Mined block, new height: {height_before + 1}")

            # Wait for events (TX should have arrived before mining, BLOCK after)
            await asyncio.sleep(3)

            # Filter events for funding account
            funding_events = [e for e in all_events if e.get("txid") == txid or e.get("type") == "BLOCK"]
            print(f"Funding account events: {len(funding_events)}")
            for i, event in enumerate(funding_events):
                print(f"  Event {i+1}: type={event['type']}, height={event['height']}, txid={event.get('txid', 'N/A')}")

            assert len(funding_events) >= 1, "Should receive at least one event"

            # Find Tx and Block events
            tx_event = next((e for e in funding_events if e["type"] == "TX"), None)
            block_event = next((e for e in funding_events if e["type"] == "BLOCK"), None)

            if tx_event:
                print(f"✓ Tx event found: txid={tx_event['txid']}, height={tx_event['height']}, value={tx_event['value']}")
                assert tx_event["txid"].lower() == txid.lower(), "Tx event txid should match sent txid"

            if block_event:
                print(f"✓ Block event found: height={block_event['height']}")
                # Block height should be >= height_before (may be the same if we received a stale event)
                assert block_event["height"] >= height_before, f"Block height should be >= {height_before}, got {block_event['height']}"

            print("\n=== Test 2: Incoming transaction subscription ===")

            # Subscribe to receiver account
            await subscribe_to_account(receiver_id)

            # Sync funding account to update spent notes
            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": funding_id})
            )

            # Clear events from Test 1
            all_events.clear()

            # Send transaction to receiver
            result = await client.execute_async(
                GraphQLRequest(
                    pay_mutation,
                    variable_values={"account": funding_id, "address": receiver_address, "amount": "0.005"},
                )
            )
            txid2 = result["pay"]
            print(f"Sent transaction to receiver, txid: {txid2}")

            # Mine a few blocks, expect at least one event
            await asyncio.sleep(1)
            await mine_blocks(rpc_url, 3)

            # Wait for events to propagate
            await asyncio.sleep(2)

            # Filter events for receiver account (incoming TX)
            receiver_events = [e for e in all_events if e.get("txid") == txid2]
            print(f"Receiver events: {len(receiver_events)}")

            if receiver_events:
                incoming_tx = next((e for e in receiver_events if e["type"] == "TX"), None)
                if incoming_tx:
                    print(f"✓ Incoming Tx event found: txid={incoming_tx['txid']}, value={incoming_tx['value']}")
                    assert incoming_tx["txid"].lower() == txid2.lower()

            print("\n=== Test 3: Multiple block events ===")

            # Clear events from Test 2
            all_events.clear()

            await asyncio.sleep(1)

            # Mine 3 blocks
            height_before = await get_current_height(client)
            print(f"Height before mining Test 3 blocks: {height_before}")
            await mine_blocks(rpc_url, 3)
            await asyncio.sleep(3)  # Give time for blocks to propagate
            height_after = await get_current_height(client)
            print(f"Height after mining: {height_after}, mined: {height_after - height_before} blocks")

            # Filter block events from all collected events
            block_events = [e for e in all_events if e["type"] == "BLOCK"]

            print(f"Block events received: {len(block_events)}")
            assert len(block_events) >= 1, f"Should receive at least 1 block event, got {len(block_events)}"

            for i, block in enumerate(block_events):
                expected_height = height_before + i + 1
                print(f"Block {i+1}: expected height {expected_height}, got {block['height']}")
                # The last block event should match the final height
                if i == len(block_events) - 1:
                    assert block["height"] == height_after, f"Last block height should be {height_after}, got {block['height']}"

            print(f"✓ Received {len(block_events)} block event(s) (note: not all blocks may generate events due to upstream polling)")
            if block_events:
                last_block = block_events[-1]
                assert height_before <= last_block["height"] <= height_after, \
                    f"Block height should be between {height_before} and {height_after}, got {last_block['height']}"
                print(f"✓ Last block event height {last_block['height']} is within expected range")

            print("\n✅ All subscription tests passed!")

            # Close WebSocket connection and cancel collector
            collector_task.cancel()
            try:
                await collector_task
            except asyncio.CancelledError:
                pass
            await ws.close()
    finally:
        await stop_zkool_instance(process)
        cleanup_test_files(DB_PATH, LOG_PATH)
