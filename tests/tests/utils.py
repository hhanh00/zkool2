"""Shared utilities for zkool tests."""

import asyncio
import contextlib
import os
import subprocess

import httpx
from gql import Client, GraphQLRequest, gql
from gql.transport.httpx import HTTPXAsyncTransport


async def get_current_height(client) -> int:
    """Get current blockchain height.

    Args:
        client: GraphQL client instance

    Returns:
        Current blockchain height as integer
    """
    query = gql("query { currentHeight }")
    request = GraphQLRequest(query)
    result = await client.execute_async(request)
    return int(result["currentHeight"])


async def wait_for_blocks(client, start_height: int, num_blocks: int):
    """Wait for blocks to be mined.

    Args:
        client: GraphQL client instance
        start_height: Starting block height
        num_blocks: Number of blocks to wait for
    """
    target = start_height + num_blocks
    while await get_current_height(client) < target:
        await asyncio.sleep(1)


async def mine_blocks(rpc_url: str, num_blocks: int):
    """Mine blocks using the RPC endpoint.

    Args:
        rpc_url: RPC endpoint URL
        num_blocks: Number of blocks to mine

    Returns:
        RPC response
    """
    payload = {
        "jsonrpc": "1.0",
        "id": "curltest",
        "method": "generate",
        "params": [num_blocks],
    }
    async with httpx.AsyncClient() as client:
        response = await client.post(rpc_url, json=payload)
        response.raise_for_status()
        return response.json()


@contextlib.asynccontextmanager
async def gql_client(url: str, timeout: float = 300.0):
    """Create a GraphQL client for the given URL.

    Args:
        url: GraphQL endpoint URL
        timeout: Request timeout in seconds

    Yields:
        GraphQL client instance
    """
    http_timeout = httpx.Timeout(timeout, connect=60.0)
    transport = HTTPXAsyncTransport(url=url, timeout=http_timeout)
    client = Client(
        transport=transport, fetch_schema_from_transport=False, execute_timeout=timeout
    )
    try:
        yield client
    finally:
        await client.close_async()


async def kill_existing_zkool_processes():
    """Kill any existing zkool_graphql processes."""
    subprocess.run(["pkill", "-9", "zkool_graphql"], stderr=subprocess.DEVNULL)
    await asyncio.sleep(1)


async def start_zkool_instance(
    zkool_binary: str, db_path: str, port: int, lwd_url: str, log_path: str | None = None
) -> subprocess.Popen:
    """Start a zkool_graphql instance.

    Args:
        zkool_binary: Path to zkool_graphql binary
        db_path: Path to database file
        port: Port to run on
        lwd_url: Light wallet daemon URL
        log_path: Optional path for log file

    Returns:
        Subprocess object
    """
    if log_path is None:
        log_path = f"/tmp/graphql_{port}.log"

    # Remove existing database
    if os.path.exists(db_path):
        os.remove(db_path)

    process = subprocess.Popen(
        [zkool_binary, "-d", db_path, "-p", str(port), "-l", lwd_url],
        stdout=open(log_path, "w"),
        stderr=subprocess.STDOUT,
    )
    await asyncio.sleep(2)
    return process


async def stop_zkool_instance(process: subprocess.Popen, timeout: int = 10):
    """Stop a zkool_graphql instance.

    Args:
        process: Subprocess object
        timeout: Timeout in seconds
    """
    if process:
        process.terminate()
        try:
            process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            process.kill()


def cleanup_test_files(db_path: str | None = None, log_path: str | None = None):
    """Remove test database and log files.

    Args:
        db_path: Path to database file
        log_path: Path to log file
    """
    if db_path and os.path.exists(db_path):
        os.remove(db_path)
    if log_path and os.path.exists(log_path):
        os.remove(log_path)


# GraphQL mutations and queries
CREATE_ACCOUNT_MUTATION = gql(
    """
    mutation ($name: String!, $key: String!, $aindex: Int!, $useInternal: Boolean!, $birth: Int!) {
        createAccount(newAccount: {
            name: $name
            key: $key
            aindex: $aindex
            useInternal: $useInternal
            birth: $birth
        })
    }
    """
)


SYNC_ACCOUNT_MUTATION = gql(
    """
    mutation ($account: Int!) {
        synchronizeAccount(idAccount: $account)
    }
    """
)


ADDRESS_BY_ACCOUNT_QUERY = gql(
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


BALANCE_BY_ACCOUNT_QUERY = gql(
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


PAY_MUTATION = gql(
    """
    mutation ($account: Int!, $recipients: [Recipient!]!) {
        pay(idAccount: $account, payment: {recipients: $recipients})
    }
    """
)


async def create_account(
    client, name: str, key: str = "", aindex: int = 0, use_internal: bool = False, birth: int = 1
) -> int:
    """Create a new account.

    Args:
        client: GraphQL client
        name: Account name
        key: Private key (empty for new account)
        aindex: Account index
        use_internal: Whether to use internal address
        birth: Birth height

    Returns:
        Account ID
    """
    result = await client.execute_async(
        GraphQLRequest(
            CREATE_ACCOUNT_MUTATION,
            variable_values={
                "name": name,
                "key": key,
                "aindex": aindex,
                "useInternal": use_internal,
                "birth": birth,
            },
        )
    )
    return int(result["createAccount"])


async def sync_account(client, account_id: int):
    """Synchronize an account.

    Args:
        client: GraphQL client
        account_id: Account ID to synchronize
    """
    await client.execute_async(
        GraphQLRequest(SYNC_ACCOUNT_MUTATION, variable_values={"account": account_id})
    )


async def get_address(client, account_id: int, pool: str = "orchard") -> str:
    """Get an address for an account.

    Args:
        client: GraphQL client
        account_id: Account ID
        pool: Address pool (orchard, sapling, transparent, ua)

    Returns:
        Address string
    """
    result = await client.execute_async(
        GraphQLRequest(ADDRESS_BY_ACCOUNT_QUERY, variable_values={"account": account_id})
    )
    return result["addressByAccount"][pool]


async def get_balance(client, account_id: int, pool: str = "orchard") -> str:
    """Get balance for an account.

    Args:
        client: GraphQL client
        account_id: Account ID
        pool: Balance pool (orchard, sapling, transparent, total)

    Returns:
        Balance as string
    """
    result = await client.execute_async(
        GraphQLRequest(BALANCE_BY_ACCOUNT_QUERY, variable_values={"account": account_id})
    )
    return result["balanceByAccount"][pool]


async def pay(client, account_id: int, recipients: list[dict]) -> str:
    """Send funds from an account.

    Args:
        client: GraphQL client
        account_id: Account ID to send from
        recipients: List of {address, amount} dicts

    Returns:
        Transaction ID
    """
    result = await client.execute_async(
        GraphQLRequest(PAY_MUTATION, variable_values={"account": account_id, "recipients": recipients})
    )
    return result["pay"]
