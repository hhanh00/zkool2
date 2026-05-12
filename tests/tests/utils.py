"""Shared utilities for zkool tests."""

import asyncio

import httpx
from gql import GraphQLRequest, gql


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
