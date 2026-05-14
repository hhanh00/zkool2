"""Test FROST signing using GraphQL API."""

import os

import pytest
from gql import GraphQLRequest, gql

from dkg import DkgParticipant, poll_with_block_mining
from utils import mine_blocks


@pytest.mark.asyncio
async def test_frost_sign_3_of_3(graphql_url, rpc_url, seed, zkool_binary, gql_client_factory):
    """Test 3-out-of-3 FROST signing using GraphQL API."""
    if not seed:
        pytest.skip("SEED not set")

    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    N = 3
    DEFAULT_PORT = 8000
    PORT_BASE = 8001
    LWD_URL = "http://localhost:8137"

    participants = []
    default_participant = None

    async def cleanup():
        for p in participants:
            await p.stop()
        if default_participant:
            await default_participant.stop()
        for i in range(1, N + 1):
            log_path = f"/tmp/graphql_{PORT_BASE + i - 1}.log"
            if os.path.exists(log_path):
                os.remove(log_path)
        default_log = "/tmp/graphql_default.log"
        if os.path.exists(default_log):
            os.remove(default_log)

    try:
        from utils import kill_existing_zkool_processes
        await kill_existing_zkool_processes()

        print("=== Setting up 3-out-of-3 FROST SIGN Test ===")

        dkg_missing = False
        for i in range(1, N + 1):
            db_path = f"/tmp/regtest_dkg_test_{i}.db"
            if not os.path.exists(db_path):
                print(f"DKG database not found at {db_path}")
                dkg_missing = True
            else:
                frost_account = DkgParticipant(0, db_path, "").get_frost_account_id()
                if not frost_account:
                    print(f"DKG account not found in {db_path}")
                    dkg_missing = True

        if dkg_missing:
            pytest.skip("DKG not completed. Run test_dkg first to create shared accounts.")

        print(f"Starting default instance on port {DEFAULT_PORT}")
        default_db = "/tmp/regtest_dkg_default.db"
        default_participant = DkgParticipant(DEFAULT_PORT, default_db, LWD_URL)
        default_participant.start(zkool_binary)
        import asyncio
        await asyncio.sleep(2)

        print("\n=== Step 1: Start participant instances ===")
        for i in range(1, N + 1):
            port = PORT_BASE + i - 1
            db_path = f"/tmp/regtest_dkg_test_{i}.db"
            participant = DkgParticipant(port, db_path, LWD_URL)
            participant.start(zkool_binary, remove_db=False)
            participants.append(participant)
            print(f"Started participant {i} on port {port}")
            await asyncio.sleep(2)

        print("\n=== Step 2: Find shared account IDs ===")
        frost_account_ids = {}
        for i, participant in enumerate(participants, 1):
            frost_account = participant.get_frost_account_id()
            if not frost_account:
                pytest.fail(f"No FROST account found for participant {i}")
            frost_account_ids[i] = frost_account
            participant.frost_account = frost_account
            print(f"Participant {i} FROST account ID: {frost_account}")

        coordinator = participants[1]
        coordinator_frost_account = frost_account_ids[2]
        print(f"Coordinator (participant #2) FROST account ID: {coordinator_frost_account}")

        print("\n=== Step 3: Create receiver account ===")
        create_account_mutation = gql(
            """
            mutation {
                createAccount(newAccount: {
                    name: "FROST-Receiver"
                    key: ""
                    aindex: 0
                    useInternal: false
                    birth: 1
                })
            }
            """
        )
        result = await coordinator.execute(GraphQLRequest(create_account_mutation))
        receiver_account = int(result["createAccount"])
        print(f"Receiver account ID: {receiver_account}")

        address_query = gql(
            """
            query ($account: Int!) {
                addressByAccount(idAccount: $account) {
                    orchard
                }
            }
            """
        )
        result = await coordinator.execute(
            GraphQLRequest(address_query, variable_values={"account": receiver_account})
        )
        receiver_address = result["addressByAccount"]["orchard"]
        print(f"Receiver address: {receiver_address}")

        print("\n=== Step 4: Synchronize FROST account ===")
        sync_mutation = gql(
            """
            mutation ($account: Int!) {
                synchronizeAccount(idAccount: $account)
            }
            """
        )
        await coordinator.execute(
            GraphQLRequest(sync_mutation, variable_values={"account": coordinator_frost_account})
        )
        print("FROST account synchronized")

        print("\n=== Step 5: Prepare payment ===")
        prepare_query = gql(
            """
            query ($account: Int!, $address: String!, $amount: BigDecimal!) {
                prepareSend(
                    idAccount: $account
                    payment: {
                        recipients: [{
                            address: $address
                            amount: $amount
                        }]
                    }
                )
            }
            """
        )
        result = await coordinator.execute(
            GraphQLRequest(
                prepare_query,
                variable_values={
                    "account": coordinator_frost_account,
                    "address": receiver_address,
                    "amount": "0.05"
                }
            )
        )
        pczt = result["prepareSend"]
        print(f"PCZT prepared: {pczt[:50]}...")

        print("\n=== Step 6: FROST signing round ===")
        sign_mutation = gql(
            """
            mutation ($account: Int!, $coordinator: Int!, $funding: Int!, $pczt: String!) {
                frostSign(
                    idAccount: $account
                    idCoordinator: $coordinator
                    messageAccount: $funding
                    pczt: $pczt
                )
            }
            """
        )
        for i, participant in enumerate(participants, 1):
            frost_account = frost_account_ids[i]
            funding_account = participant.get_funding_account_id()
            if not funding_account:
                pytest.fail(f"No funding account found for participant {i}")

            print(f"Participant {i} signing with funding account {funding_account}...")
            result = await participant.execute(
                GraphQLRequest(
                    sign_mutation,
                    variable_values={
                        "account": frost_account,
                        "coordinator": 2,
                        "funding": funding_account,
                        "pczt": pczt
                    }
                )
            )
            sign_result = result["frostSign"]
            print(f"Participant {i} sign result: {sign_result}")

            if not sign_result:
                pytest.fail(f"Participant {i} signing failed")

        print("All participants completed signing round")

        balance_query = gql(
            """
            query ($account: Int!) {
                balanceByAccount(idAccount: $account) {
                    orchard
                }
            }
            """
        )
        result = await coordinator.execute(
            GraphQLRequest(balance_query, variable_values={"account": coordinator_frost_account})
        )
        frost_balance = result["balanceByAccount"]["orchard"]
        print(f"FROST shared account balance after signing: {frost_balance} ZEC")

        print("\n=== Step 7: Verify transaction completed ===")
        expected_amount = "0.05000000"

        async def check_receiver_balance():
            sync_mutation = gql(
                """
                mutation ($account: Int!) {
                    synchronizeAccount(idAccount: $account)
                }
            """
            )
            await coordinator.execute(
                GraphQLRequest(sync_mutation, variable_values={"account": receiver_account})
            )

            result = await coordinator.execute(
                GraphQLRequest(balance_query, variable_values={"account": receiver_account})
            )
            receiver_balance = result["balanceByAccount"]["orchard"]
            print(f"Receiver balance: {receiver_balance} ZEC (waiting for {expected_amount} ZEC)")
            return receiver_balance == expected_amount

        success = await poll_with_block_mining(check_receiver_balance, rpc_url, timeout=300)
        if success:
            print("\n=== ✅ Transaction completed successfully! ===")
            print(f"Receiver account received {expected_amount} ZEC from FROST shared account")
            print("\n=== Test completed successfully ===")
        else:
            result = await coordinator.execute(
                GraphQLRequest(balance_query, variable_values={"account": receiver_account})
            )
            final_balance = result["balanceByAccount"]["orchard"]
            pytest.fail(
                f"Transaction did not complete. Final receiver balance: {final_balance} ZEC "
                f"(expected {expected_amount} ZEC)"
            )

    finally:
        await cleanup()
