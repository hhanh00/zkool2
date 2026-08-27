"""DKG participant class and utilities for FROST testing."""

import asyncio
import json
import os
import shutil
import sqlite3
import subprocess
import sys

import httpx
from gql import Client, GraphQLRequest
from gql.transport.httpx import HTTPXAsyncTransport
from utils import mine_blocks


class DkgParticipant:
    """Represents a participant in a FROST DKG protocol."""

    def __init__(self, port: int, db_path: str, lwd_url: str, id: int | None = None):
        self.port = port
        self.db_path = db_path
        self.lwd_url = lwd_url
        self.id = id
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
                "-d",
                self.db_path,
                "-p",
                str(self.port),
                "-l",
                self.lwd_url,
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


MACOS_BUNDLE_ID = "cc.methyl.zkool"


def app_documents_dir() -> str:
    """Whatever `getApplicationDocumentsDirectory()` returns for the app.

    On macOS the app is sandboxed (com.apple.security.app-sandbox), so it can
    neither read nor write /tmp: the UI participant's database and the
    rendezvous files have to live inside its container. On Linux there is no
    sandbox and path_provider returns ~/Documents.
    """
    if sys.platform == "darwin":
        return os.path.expanduser(f"~/Library/Containers/{MACOS_BUNDLE_ID}/Data/Documents")
    return os.path.expanduser("~/Documents")


def flutter_device() -> str:
    """The `flutter test -d <device>` desktop target for this platform."""
    return {"darwin": "macos", "linux": "linux", "win32": "windows"}[sys.platform]


class Rendezvous:
    """File-based handshake with the Flutter integration test.

    pytest cannot talk to the app over GraphQL, and both sides need addresses
    from the other while the `flutter test` subprocess is running, so they
    exchange JSON documents in a shared directory. Every write goes to a temp
    file and is renamed, so a reader never sees a partial document.
    """

    def __init__(self, directory: str):
        self.dir = directory

    @property
    def config_path(self) -> str:
        return os.path.join(self.dir, "config.json")

    @property
    def ui_path(self) -> str:
        return os.path.join(self.dir, "ui.json")

    @property
    def peers_path(self) -> str:
        return os.path.join(self.dir, "peers.json")

    def setup(self):
        shutil.rmtree(self.dir, ignore_errors=True)
        os.makedirs(self.dir, exist_ok=True)

    def cleanup(self):
        shutil.rmtree(self.dir, ignore_errors=True)

    def _write(self, path: str, payload: dict):
        tmp = path + ".tmp"
        with open(tmp, "w") as f:
            json.dump(payload, f)
        os.replace(tmp, path)

    def write_config(self, **payload):
        self._write(self.config_path, payload)

    def write_peers(self, peers: dict[int, str]):
        self._write(self.peers_path, {str(k): v for k, v in peers.items()})

    def read_ui(self) -> dict:
        if not os.path.exists(self.ui_path):
            return {}
        try:
            with open(self.ui_path) as f:
                return json.load(f)
        except json.JSONDecodeError:
            return {}

    async def poll_ui(self, key: str, timeout: int = 600, interval: int = 2, process=None):
        """Wait until the Flutter test publishes `key`, or fail loudly.

        If `process` is given and it exits before the key shows up, raise
        immediately instead of waiting out the whole timeout.
        """
        elapsed = 0
        while elapsed < timeout:
            value = self.read_ui().get(key)
            if value:
                return value
            if process is not None and process.returncode is not None:
                raise RuntimeError(
                    f"flutter test exited with {process.returncode} before publishing '{key}'"
                )
            await asyncio.sleep(interval)
            elapsed += interval
        raise TimeoutError(f"timed out waiting for the UI participant to publish '{key}'")


async def get_raw_mempool(rpc_url: str) -> list:
    """Transaction ids currently sitting in the node's mempool."""
    payload = {"jsonrpc": "1.0", "id": "mempool", "method": "getrawmempool", "params": []}
    async with httpx.AsyncClient() as client:
        response = await client.post(rpc_url, json=payload)
        response.raise_for_status()
        return response.json().get("result") or []


async def demand_miner(rpc_url: str, poll: int = 2, idle_fallback: int = 60):
    """Mine a block whenever there is something to confirm, until cancelled.

    The DKG advances by passing packages in transaction memos, so a round only
    completes once the sender's transaction is mined. Mining on a fixed timer
    makes the run non-deterministic — blocks land at arbitrary points relative
    to the protocol. Watching the mempool instead ties each block to the
    broadcast that needs it, so the chain moves exactly when the protocol does.

    `idle_fallback` is a safety valve only: if nothing has been broadcast for
    that long the tip is nudged forward once, so a participant waiting on a new
    tip (rather than on a transaction) cannot wedge the run.
    """
    loop = asyncio.get_running_loop()
    last_mine = loop.time()
    try:
        while True:
            try:
                if await get_raw_mempool(rpc_url):
                    await mine_blocks(rpc_url, 1)
                    last_mine = loop.time()
                elif loop.time() - last_mine > idle_fallback:
                    await mine_blocks(rpc_url, 1)
                    last_mine = loop.time()
            except Exception as e:  # a transient RPC hiccup must not kill the run
                print(f"[miner] {e}")
            await asyncio.sleep(poll)
    except asyncio.CancelledError:
        pass
