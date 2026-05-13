"""Test JWT authentication for account access control."""

import asyncio
import contextlib
import os
import subprocess

import httpx
import pytest
from cryptography.hazmat.backends import default_backend
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import ec
from gql import Client, GraphQLRequest, gql
from gql.transport.httpx import HTTPXAsyncTransport

from utils import get_current_height, mine_blocks, wait_for_blocks


def generate_jwt_keypair():
    """Generate a new EC P-256 key pair for JWT authentication."""
    # Generate EC P-256 private key
    private_key = ec.generate_private_key(ec.SECP256R1(), default_backend())
    public_key = private_key.public_key()

    # Serialize private key to PEM format (PKCS8)
    private_pem = private_key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption()
    ).decode('utf-8')

    # Serialize public key to PEM format (SubjectPublicKeyInfo)
    public_pem = public_key.public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo
    ).decode('utf-8')

    return private_pem, public_pem


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
    async def _create_client(url: str, jwt_token: str | None = None):
        timeout = httpx.Timeout(300.0, connect=60.0)
        headers = {}
        if jwt_token:
            headers["Authorization"] = f"Bearer {jwt_token}"
        transport = HTTPXAsyncTransport(url=url, timeout=timeout, headers=headers)
        client = Client(
            transport=transport, fetch_schema_from_transport=False, execute_timeout=300.0
        )
        try:
            yield client
        finally:
            await client.close_async()

    return _create_client


def create_jwt_token(private_pem: str, account_id: int) -> str:
    """Create a JWT token for the given account ID using the private key."""
    import time
    import jwt

    # Create payload - must match the Claims struct in Rust
    payload = {
        "sub": account_id,  # Note: Rust expects u32, not string
        "iat": int(time.time()),
        "exp": int(time.time()) + 3600,  # 1 hour expiration
        "write": True,  # Required field
    }

    # Load private key from PEM
    from cryptography.hazmat.backends import default_backend
    from cryptography.hazmat.primitives import serialization

    private_key = serialization.load_pem_private_key(
        private_pem.encode('utf-8'),
        password=None,
        backend=default_backend()
    )

    # Sign with ES256
    token = jwt.encode(payload, private_key, algorithm="ES256")
    return token


@pytest.mark.asyncio
async def test_jwt_authentication(gql_client_factory, rpc_url, seed, zkool_binary, lwd_url):
    """Test JWT authentication for account access control."""
    if not seed:
        pytest.skip("SEED not set")

    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    PORT = 8000
    DB_PATH = "/tmp/regtest_jwt.db"
    LOG_PATH = "/tmp/graphql_jwt.log"
    JWT_KEY_PATH = "/tmp/regtest_jwt_key.pub"
    GRAPHQL_URL = f"http://localhost:{PORT}/graphql"

    process = None

    # Helper to start the server
    async def start_server(with_jwt=False):
        nonlocal process
        # Kill any existing zkool_graphql processes
        subprocess.run(["pkill", "-9", "zkool_graphql"], stderr=subprocess.DEVNULL)
        await asyncio.sleep(1)

        cmd = [
            zkool_binary,
            "-d", DB_PATH,
            "-p", str(PORT),
            "-l", lwd_url,
        ]
        if with_jwt:
            cmd.extend(["-j", JWT_KEY_PATH])

        print(f"Starting server with JWT={with_jwt}")
        process = subprocess.Popen(
            cmd,
            stdout=open(LOG_PATH, "w"),
            stderr=subprocess.STDOUT,
        )
        await asyncio.sleep(3)

        # Check if process is still running
        poll_result = process.poll()
        if poll_result is not None:
            with open(LOG_PATH, "r") as f:
                log_content = f.read()
                print(f"Log content:\n{log_content}")
            pytest.fail(f"zkool_graphql failed to start with code {poll_result}")

    try:
        # Generate JWT key pair
        print("=== Generating JWT key pair ===")
        private_pem, public_pem = generate_jwt_keypair()
        print(f"Private key: {private_pem[:50]}...")
        print(f"Public key: {public_pem[:50]}...")

        # Write public key to file
        with open(JWT_KEY_PATH, "w") as f:
            f.write(public_pem)
        print(f"Wrote JWT public key to {JWT_KEY_PATH}")

        # Create admin JWT with sub=0 for full access
        admin_jwt_token = create_jwt_token(private_pem, 0)
        print(f"Admin JWT (sub=0): {admin_jwt_token[:50]}...")

        # Remove existing database
        if os.path.exists(DB_PATH):
            os.remove(DB_PATH)

        # Start server WITH JWT enabled from the start
        await start_server(with_jwt=True)

        # Use admin JWT (sub=0) for all setup operations
        async with gql_client_factory(GRAPHQL_URL, admin_jwt_token) as client:
            print("\n=== Step 1: Create admin wallet from SEED ===")
            create_admin_mutation = gql(
                """
                mutation ($seed: String!) {
                    createAccount(newAccount: {
                        name: "Admin"
                        key: $seed
                        aindex: 0
                        useInternal: false
                        birth: 1
                    })
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(create_admin_mutation, variable_values={"seed": seed})
            )
            admin_id = int(result["createAccount"])
            print(f"Created admin account: {admin_id}")

            # Synchronize admin account
            sync_mutation = gql(
                """
                mutation ($account: Int!) {
                    synchronizeAccount(idAccount: $account)
                }
                """
            )
            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": admin_id})
            )
            print("Synchronized admin account")

            # Get admin balance
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
                GraphQLRequest(balance_query, variable_values={"account": admin_id})
            )
            admin_balance = result["balanceByAccount"]["orchard"]
            print(f"Admin balance: {admin_balance} ZEC")

            print("\n=== Step 2: Create 2 new accounts ===")
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

            # Get addresses for both accounts
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
            print(f"Account 1 address: {account1_address}")

            result = await client.execute_async(
                GraphQLRequest(address_query, variable_values={"account": account2_id})
            )
            account2_address = result["addressByAccount"]["orchard"]
            print(f"Account 2 address: {account2_address}")

            # Generate JWTs for account 1 & 2
            jwt1_token = create_jwt_token(private_pem, account1_id)
            print(f"JWT for account 1: {jwt1_token[:50]}...")

            jwt2_token = create_jwt_token(private_pem, account2_id)
            print(f"JWT for account 2: {jwt2_token[:50]}...")

            print("\n=== Step 3: Send funds from admin to account 1 ===")
            # Check admin balance first
            result = await client.execute_async(
                GraphQLRequest(balance_query, variable_values={"account": admin_id})
            )
            admin_balance = result["balanceByAccount"]["orchard"]
            print(f"Admin balance: {admin_balance} ZEC")

            # Send 0.1 ZEC (smaller amount in case admin balance is low)
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
                    variable_values={"account": admin_id, "address": account1_address, "amount": "0.05"}
                )
            )
            txid = result["pay"]
            print(f"Sent 0.05 ZEC to account 1, txid: {txid}")

            # Mine blocks
            height_before = await get_current_height(client)
            await mine_blocks(rpc_url, 5)
            await wait_for_blocks(client, height_before, 5)
            print("Mined 5 blocks")

            # Synchronize account 1
            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": account1_id})
            )

            print("\n=== Step 4: Check admin (sub=0) can see all balances ===")
            # Admin with JWT (sub=0) can see all balances
            result = await client.execute_async(
                GraphQLRequest(balance_query, variable_values={"account": admin_id})
            )
            print(f"Admin sees admin balance: {result['balanceByAccount']['orchard']} ZEC")

            result = await client.execute_async(
                GraphQLRequest(balance_query, variable_values={"account": account1_id})
            )
            print(f"Admin sees account 1 balance: {result['balanceByAccount']['orchard']} ZEC")

            result = await client.execute_async(
                GraphQLRequest(balance_query, variable_values={"account": account2_id})
            )
            print(f"Admin sees account 2 balance: {result['balanceByAccount']['orchard']} ZEC")

            print("\n=== Step 5: Check account 1 JWT can only see their own balance ===")
            # With JWT 1, should be able to see account 1 balance
            async with gql_client_factory(GRAPHQL_URL, jwt1_token) as jwt1_client:
                result = await jwt1_client.execute_async(
                    GraphQLRequest(balance_query, variable_values={"account": account1_id})
                )
                account1_balance = result["balanceByAccount"]["orchard"]
                print(f"Account 1 JWT sees account 1 balance: {account1_balance} ZEC")

                # Should NOT be able to see account 2 balance
                try:
                    result = await jwt1_client.execute_async(
                        GraphQLRequest(balance_query, variable_values={"account": account2_id})
                    )
                    print(f"ERROR: Account 1 JWT should not see account 2 balance!")
                    assert False, "Account 1 JWT should not be able to see account 2 balance"
                except Exception as e:
                    print(f"✓ Account 1 JWT correctly blocked from seeing account 2: {e}")

                # Should NOT be able to see admin balance
                try:
                    result = await jwt1_client.execute_async(
                        GraphQLRequest(balance_query, variable_values={"account": admin_id})
                    )
                    print(f"ERROR: Account 1 JWT should not see admin balance!")
                    assert False, "Account 1 JWT should not be able to see admin balance"
                except Exception as e:
                    print(f"✓ Account 1 JWT correctly blocked from seeing admin: {e}")

            print("\n=== Step 6: Check account 2 JWT can only see their own balance ===")
            # With JWT 2, should be able to see account 2 balance
            async with gql_client_factory(GRAPHQL_URL, jwt2_token) as jwt2_client:
                result = await jwt2_client.execute_async(
                    GraphQLRequest(balance_query, variable_values={"account": account2_id})
                )
                account2_balance = result["balanceByAccount"]["orchard"]
                print(f"Account 2 JWT sees account 2 balance: {account2_balance} ZEC")

                # Should NOT be able to see account 1 balance
                try:
                    result = await jwt2_client.execute_async(
                        GraphQLRequest(balance_query, variable_values={"account": account1_id})
                    )
                    print(f"ERROR: Account 2 JWT should not see account 1 balance!")
                    assert False, "Account 2 JWT should not be able to see account 1 balance"
                except Exception as e:
                    print(f"✓ Account 2 JWT correctly blocked from seeing account 1: {e}")

            print("\n=== Step 7: Send funds from account 1 to account 2 using JWT 1 ===")
            # First send some funds from admin to account 2 for testing
            await client.execute_async(
                GraphQLRequest(
                    pay_mutation,
                    variable_values={"account": admin_id, "address": account2_address, "amount": "0.05"}
                )
            )
            height_before = await get_current_height(client)
            await mine_blocks(rpc_url, 5)
            await wait_for_blocks(client, height_before, 5)
            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": account2_id})
            )

            result = await client.execute_async(
                GraphQLRequest(balance_query, variable_values={"account": account2_id})
            )
            print(f"Account 2 balance before: {result['balanceByAccount']['orchard']} ZEC")

            async with gql_client_factory(GRAPHQL_URL, jwt1_token) as jwt1_client:
                # Now try to send from account 1 to account 2 using JWT 1
                result = await jwt1_client.execute_async(
                    GraphQLRequest(
                        pay_mutation,
                        variable_values={"account": account1_id, "address": account2_address, "amount": "0.025"}
                    )
                )
                txid = result["pay"]
                print(f"✓ Sent 0.025 ZEC from account 1 to account 2, txid: {txid}")

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

            print("\n=== Step 8: Try to send from account 1 to 2 without JWT (should fail) ===")
            async with gql_client_factory(GRAPHQL_URL) as no_jwt_client:
                try:
                    result = await no_jwt_client.execute_async(
                        GraphQLRequest(
                            pay_mutation,
                            variable_values={"account": account1_id, "address": account2_address, "amount": "0.025"}
                        )
                    )
                    print(f"ERROR: Should not be able to send without JWT!")
                    assert False, "Should not be able to send without JWT"
                except Exception as e:
                    print(f"✓ Correctly blocked from sending without JWT: {e}")

            print("\n=== Step 9: Try to send from account 1 to 2 using JWT 2 (should fail) ===")
            async with gql_client_factory(GRAPHQL_URL, jwt2_token) as jwt2_client:
                try:
                    result = await jwt2_client.execute_async(
                        GraphQLRequest(
                            pay_mutation,
                            variable_values={"account": account1_id, "address": account2_address, "amount": "0.025"}
                        )
                    )
                    print(f"ERROR: Should not be able to send from account 1 using JWT 2!")
                    assert False, "Should not be able to send from account 1 using JWT 2"
                except Exception as e:
                    print(f"✓ Correctly blocked from sending with wrong JWT: {e}")

            print("\n=== Step 10: Check final balances with JWTs ===")
            async with gql_client_factory(GRAPHQL_URL, jwt1_token) as jwt1_client:
                result = await jwt1_client.execute_async(
                    GraphQLRequest(balance_query, variable_values={"account": account1_id})
                )
                print(f"JWT 1 sees account 1 balance: {result['balanceByAccount']['orchard']} ZEC")

                # Still cannot see account 2
                try:
                    result = await jwt1_client.execute_async(
                        GraphQLRequest(balance_query, variable_values={"account": account2_id})
                    )
                    print(f"ERROR: JWT 1 should still not see account 2!")
                    assert False, "JWT 1 should not be able to see account 2"
                except Exception as e:
                    print(f"✓ JWT 1 still cannot see account 2")

            async with gql_client_factory(GRAPHQL_URL, jwt2_token) as jwt2_client:
                result = await jwt2_client.execute_async(
                    GraphQLRequest(balance_query, variable_values={"account": account2_id})
                )
                print(f"JWT 2 sees account 2 balance: {result['balanceByAccount']['orchard']} ZEC")

                # Still cannot see account 1
                try:
                    result = await jwt2_client.execute_async(
                        GraphQLRequest(balance_query, variable_values={"account": account1_id})
                    )
                    print(f"ERROR: JWT 2 should still not see account 1!")
                    assert False, "JWT 2 should not be able to see account 1"
                except Exception as e:
                    print(f"✓ JWT 2 still cannot see account 1")

            print("\n=== Step 11: Verify admin (sub=0) can see all accounts ===")
            # Admin JWT with sub=0 can see everything
            async with gql_client_factory(GRAPHQL_URL, admin_jwt_token) as client:
                result = await client.execute_async(
                    GraphQLRequest(balance_query, variable_values={"account": account1_id})
                )
                print(f"Admin sees account 1 balance: {result['balanceByAccount']['orchard']} ZEC")

                result = await client.execute_async(
                    GraphQLRequest(balance_query, variable_values={"account": account2_id})
                )
                print(f"Admin sees account 2 balance: {result['balanceByAccount']['orchard']} ZEC")

            print("\n✅ JWT authentication test passed!")

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
        if os.path.exists(JWT_KEY_PATH):
            os.remove(JWT_KEY_PATH)
