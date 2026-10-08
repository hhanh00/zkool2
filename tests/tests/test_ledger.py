"""Ledger import and consensus-valid payment tests through zkool_graphql."""

import asyncio
import os
from decimal import Decimal

import pytest
from gql import GraphQLRequest, gql
from gql.transport.exceptions import TransportQueryError

from utils import (
    cleanup_test_files,
    dump_server_log,
    kill_existing_zkool_processes,
    get_current_height,
    mine_blocks,
    wait_for_blocks,
    start_zkool_instance,
    stop_zkool_instance,
)


@pytest.mark.asyncio
async def test_ledger_import_and_payments_via_graphql(
    gql_client_factory, zkool_binary, rpc_url, lwd_url
):
    """Import, sign, broadcast and mine payments from the mock Ledger."""
    if not os.path.exists(zkool_binary):
        pytest.fail(f"zkool_graphql binary not found at {zkool_binary}")
    expected_address = os.environ.get("EXPECTED_LEDGER_ADDRESS")
    if not expected_address:
        pytest.fail("EXPECTED_LEDGER_ADDRESS is not set")
    expected_transparent = os.environ.get("EXPECTED_LEDGER_TRANSPARENT")
    if not expected_transparent:
        pytest.fail("EXPECTED_LEDGER_TRANSPARENT is not set")

    port = 8000
    db_path = "/tmp/regtest_ledger.db"
    log_path = "/tmp/graphql_ledger.log"
    graphql_url = f"http://localhost:{port}/graphql"
    process = None

    try:
        await kill_existing_zkool_processes()
        process = await start_zkool_instance(
            zkool_binary,
            db_path,
            port,
            lwd_url,
            log_path,
            coin=2,
        )
        await asyncio.sleep(3)

        async with gql_client_factory(graphql_url) as client:
            create_account = gql(
                """
                mutation {
                    createAccount(newAccount: {
                        name: "Mock Ledger"
                        aindex: 0
                        useInternal: false
                        birth: 1
                        hw: 2
                    })
                }
                """
            )
            result = await client.execute_async(create_account)
            account_id = int(result["createAccount"])
            assert account_id > 0

            get_addresses = gql(
                """
                query ($account: Int!) {
                    addressByAccount(idAccount: $account) {
                        transparent
                        ironwood
                        diversifierIndex
                    }
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(get_addresses, variable_values={"account": account_id})
            )
            addresses = result["addressByAccount"]
            assert int(addresses["diversifierIndex"]) == 0
            assert addresses["transparent"] == expected_transparent
            assert addresses["ironwood"] == expected_address

            # Compare the device-imported viewing key with the same public key
            # derived in software from the mock device's seed.
            seed = os.environ["REGTEST_SEED"]
            result = await client.execute_async(GraphQLRequest(gql("""
                mutation ($seed: String!) {
                    createAccount(newAccount: {
                        name: "Software reference"
                        key: $seed
                        aindex: 0
                        useInternal: false
                        birth: 1
                        pools: 9
                    })
                }
            """), variable_values={"seed": seed}))
            software_id = int(result["createAccount"])
            viewing_key = gql("""
                query ($account: Int!, $pools: Int!) {
                    unifiedFullViewingKey(idAccount: $account, pools: $pools)
                }
            """)
            for pools in (1, 4, 5, 8, 9, 12, 13):
                device = await client.execute_async(GraphQLRequest(
                    viewing_key, variable_values={"account": account_id, "pools": pools}
                ))
                software = await client.execute_async(GraphQLRequest(
                    viewing_key, variable_values={"account": software_id, "pools": pools}
                ))
                assert device == software
                assert device["unifiedFullViewingKey"].startswith(
                    "xpub" if pools == 1 else "uviewregtest1"
                )
            with pytest.raises(TransportQueryError, match="unavailable"):
                await client.execute_async(GraphQLRequest(
                    viewing_key, variable_values={"account": account_id, "pools": 3}
                ))
            # The regtest bootstrap funds the imported account's Ironwood address.
            # It has no spending key in zkool: pay must obtain signatures over APDU.
            sync = gql("""
                mutation ($account: Int!) { synchronizeAccount(idAccount: $account) }
            """)
            balance = gql("""
                query ($account: Int!) {
                    balanceByAccount(idAccount: $account) { ironwood transparent total }
                }
            """)
            async def sync_balance(account):
                await client.execute_async(GraphQLRequest(sync, variable_values={"account": account}))
                result = await client.execute_async(GraphQLRequest(balance, variable_values={"account": account}))
                return result["balanceByAccount"]

            funded = await sync_balance(account_id)
            assert Decimal(funded["ironwood"]) > Decimal("0.1"), "Bootstrap must fund the Ledger account"
            result = await client.execute_async(GraphQLRequest(gql("""
                mutation ($seed: String!) {
                    createAccount(newAccount: {
                        name: "Payment recipient"
                        key: $seed
                        aindex: 1
                        useInternal: false
                        birth: 1
                        pools: 9
                    })
                }
            """), variable_values={"seed": seed}))
            recipient_id = int(result["createAccount"])
            result = await client.execute_async(GraphQLRequest(
                get_addresses, variable_values={"account": recipient_id}
            ))
            recipient = result["addressByAccount"]["ironwood"]
            pay = gql("""
                mutation ($account: Int!, $address: String!, $amount: BigDecimal!, $pools: Int) {
                    pay(idAccount: $account, payment: {
                        recipients: [{address: $address, amount: $amount}]
                        srcPools: $pools
                    })
                }
            """)
            async def send_and_mine(address, amount, pools=None):
                result = await client.execute_async(GraphQLRequest(pay, variable_values={
                    "account": account_id, "address": address, "amount": amount, "pools": pools,
                }))
                txid = result["pay"]
                assert len(txid) == 64
                height = await get_current_height(client)
                mined = await mine_blocks(rpc_url, 5)
                assert not mined.get("error"), mined
                await asyncio.wait_for(wait_for_blocks(client, height, 5), timeout=120)
                # A returned txid alone does not prove consensus acceptance.
                result = await client.execute_async(GraphQLRequest(gql("""
                    mutation ($account: Int!) { synchronizeAccount(idAccount: $account) }
                """), variable_values={"account": account_id}))
                result = await client.execute_async(GraphQLRequest(gql("""
                    query ($account: Int!, $txid: String!) {
                        transactionById(idAccount: $account, txid: $txid) { txid height }
                    }
                """), variable_values={"account": account_id, "txid": txid}))
                tx = result["transactionById"]
                assert tx["txid"] == txid.lower()
                assert int(tx["height"]) > height
                return txid

            recipient_before = await sync_balance(recipient_id)
            recipient_initial = Decimal(recipient_before["ironwood"])
            await send_and_mine(recipient, "0.025")
            received = await sync_balance(recipient_id)
            assert Decimal(received["ironwood"]) == recipient_initial + Decimal("0.025")
            remaining = await sync_balance(account_id)
            assert Decimal(remaining["total"]) < Decimal(funded["total"]) - Decimal("0.025")

            # Move change into a transparent output, then spend it through ECDSA.
            transparent_before = await sync_balance(account_id)
            await send_and_mine(addresses["transparent"], "0.05")
            transparent = await sync_balance(account_id)
            assert Decimal(transparent["transparent"]) == Decimal(transparent_before["transparent"]) + Decimal("0.05")
            # Pool restriction prevents the planner from selecting Ironwood notes.
            await send_and_mine(recipient, "0.02", pools=1)
            received = await sync_balance(recipient_id)
            assert Decimal(received["ironwood"]) == recipient_initial + Decimal("0.045")
            spent = await sync_balance(account_id)
            assert Decimal(spent["transparent"]) < Decimal(transparent["transparent"]) - Decimal("0.02")
    except Exception:
        dump_server_log(log_path, "ZKOOL GRAPHQL LOG")
        raise
    finally:
        if process is not None:
            await stop_zkool_instance(process)
        cleanup_test_files(db_path, log_path)
