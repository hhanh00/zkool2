"""End-to-end tests for Ledger account import through zkool_graphql."""

import asyncio
import os

import pytest
from gql import GraphQLRequest, gql

from utils import (
    cleanup_test_files,
    dump_server_log,
    kill_existing_zkool_processes,
    start_zkool_instance,
    stop_zkool_instance,
)


@pytest.mark.asyncio
async def test_import_ledger_account_via_graphql(gql_client_factory, zkool_binary):
    """Import an Official Ledger account without supplying a key or pools."""
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
            "http://localhost:8137",
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
    except Exception:
        dump_server_log(log_path, "ZKOOL GRAPHQL LOG")
        raise
    finally:
        if process is not None:
            await stop_zkool_instance(process)
        cleanup_test_files(db_path, log_path)
