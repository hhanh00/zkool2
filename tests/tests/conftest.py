import os
from pathlib import Path

import httpx
import pytest
from gql import Client
from gql.transport.websockets import WebsocketsTransport
from gql.transport.httpx import HTTPXAsyncTransport


@pytest.fixture(scope="session")
def graphql_url():
    return os.getenv("GRAPHQL_URL", "http://localhost:8000/graphql")


@pytest.fixture(scope="session")
def ws_url():
    return os.getenv("WS_URL", "ws://localhost:8000/subscriptions")


@pytest.fixture(scope="session")
def rpc_url():
    return os.getenv("RPC_URL", "http://127.0.0.1:18232/")


@pytest.fixture
async def gql_client(graphql_url):
    timeout = httpx.Timeout(300.0, connect=60.0)
    transport = HTTPXAsyncTransport(url=graphql_url, timeout=timeout)
    async with Client(
        transport=transport,
        fetch_schema_from_transport=False,
        execute_timeout=300.0,
    ) as client:
        yield client


@pytest.fixture
async def ws_client(ws_url):
    transport = WebsocketsTransport(url=ws_url, connect_timeout=60, close_timeout=60)
    async with Client(transport=transport, fetch_schema_from_transport=False) as client:
        yield client


@pytest.fixture(scope="session")
def seed():
    seed_path = Path(__file__).parent.parent.parent / "example" / "sh" / "seed"
    if seed_path.exists():
        return seed_path.read_text().strip()
    return os.getenv("SEED", "")
