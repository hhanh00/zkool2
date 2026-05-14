"""DKG participant class and utilities for FROST testing."""

import os
import sqlite3
import subprocess

import httpx
from gql import Client, GraphQLRequest, gql
from gql.transport.httpx import HTTPXAsyncTransport

from utils import mine_blocks


class DkgParticipant:
    """Represents a participant in a FROST DKG protocol."""

    def __init__(self, port: int, db_path: str, lwd_url: str):
        self.port = port
        self.db_path = db_path
        self.lwd_url = lwd_url
        self.url = f"http://localhost:{port}/graphql"
        self.process: subprocess.Popen | None = None
        self.funding_account: int | None = None
        self.funding_address: str | None = None
        self.dkg_address: str | None = None
        self.frost_account: int | None = None
        self._client: Client | None = None

    async def get_client(self) -> Client:
        """Get or create the GraphQL client for this participant."""
        if self._client is None:
            timeout = httpx.Timeout(300.0, connect=60.0)
            transport = HTTPXAsyncTransport(url=self.url, timeout=timeout)
            self._client = Client(
                transport=transport, fetch_schema_from_transport=False, execute_timeout=300.0
            )
        return self._client

    async def close_client(self):
        """Close the GraphQL client."""
        if self._client:
            await self._client.close_async()
            self._client = None

    async def execute(self, request: GraphQLRequest):
        """Execute a GraphQL request on this participant's server."""
        client = await self.get_client()
        return await client.execute_async(request)

    def start(self, zkool_binary: str, remove_db=True):
        """Start the zkool_graphql instance."""
        if remove_db and os.path.exists(self.db_path):
            os.remove(self.db_path)

        log_path = f"/tmp/graphql_{self.port}.log"
        self.process = subprocess.Popen(
            [
                zkool_binary,
                "-d", self.db_path,
                "-p", str(self.port),
                "-l", self.lwd_url,
            ],
            stdout=open(log_path, "w"),
            stderr=subprocess.STDOUT,
        )

    async def stop(self):
        """Stop the zkool_graphql instance and close client."""
        await self.close_client()
        if self.process:
            self.process.terminate()
            self.process.wait(timeout=10)
            self.process = None

    def get_frost_account_id(self) -> int | None:
        """Get the FROST account ID from the database."""
        if not os.path.exists(self.db_path):
            return None

        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            cursor.execute("SELECT id_account FROM accounts WHERE name LIKE 'Dkg-Test-%'")
            result = cursor.fetchone()
            conn.close()
            return result[0] if result else None
        except Exception:
            return None

    def get_funding_account_id(self) -> int | None:
        """Get the funding account ID from the database."""
        if not os.path.exists(self.db_path):
            return None

        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            cursor.execute("SELECT id_account FROM accounts WHERE name = 'DKG-Fund'")
            result = cursor.fetchone()
            conn.close()
            return result[0] if result else None
        except Exception:
            return None


async def poll_with_block_mining(
    condition,
    rpc_url: str,
    timeout: int = 300,
    interval: int = 10,
    blocks_per_interval: int = 1,
) -> bool:
    """Poll a condition with block mining after each interval.

    Args:
        condition: Async function that returns True when condition is met
        rpc_url: RPC URL for mining blocks
        timeout: Total timeout in seconds
        interval: Polling interval in seconds
        blocks_per_interval: Number of blocks to mine after each interval

    Returns:
        True if condition was met, False if timeout occurred
    """
    elapsed = 0
    while elapsed < timeout:
        if await condition():
            return True

        import asyncio

        await asyncio.sleep(interval)
        await mine_blocks(rpc_url, blocks_per_interval)
        elapsed += interval

    return False
