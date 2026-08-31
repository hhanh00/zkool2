"""Test 2-of-3 FROST DKG using GraphQL API.

The DKG is run by ALL n participants regardless of the threshold: rounds
advance only when every participant's package has arrived, so each wallet
materializes the same complete key set and any t-subset is valid for signing
later. This test proves the DKG completes for n=3, t=2.
"""

import asyncio
import os

import pytest
from gql import GraphQLRequest, gql

from conftest import gql_client_factory
from dkg import DkgParticipant, poll_with_block_mining
from utils import get_current_height, mine_blocks, wait_for_blocks


@pytest.mark.asyncio
async def test_dkg_2_of_3(graphql_url, rpc_url, seed, zkool_binary, gql_client_factory):
    """Test 2-of-3 FROST DKG: all 3 participants run the DKG, t=2 for signing."""
    if not seed:
        pytest.skip("SEED not set")

    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    N = 3
    T = 2
    DEFAULT_PORT = 8000
    PORT_BASE = 8101
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
        default_log = "/tmp/graphql_default_2of3.log"
        if os.path.exists(default_log):
            os.remove(default_log)

    try:
        from utils import kill_existing_zkool_processes

        await kill_existing_zkool_processes()

        print("=== Setting up 2-of-3 FROST DKG Test ===")
        print(f"Starting default instance on port {DEFAULT_PORT}")
        default_db = "/tmp/regtest_dkg_default_2of3.db"
        default_participant = DkgParticipant(DEFAULT_PORT, default_db, LWD_URL)
        default_participant.start(zkool_binary)
        await asyncio.sleep(2)

        print("\n=== Step 1: Start participant instances ===")
        for i in range(1, N + 1):
            port = PORT_BASE + i - 1
            db_path = f"/tmp/regtest_dkg_2of3_{i}.db"
            participant = DkgParticipant(port, db_path, LWD_URL)
            participant.start(zkool_binary)
            participants.append(participant)
            print(f"Started participant {i} on port {port}")
            await asyncio.sleep(2)

        print("\n=== Step 2: Create funded wallet on default instance ===")
        async with gql_client_factory(graphql_url) as client:
            create_account_mutation = gql(
                """
                mutation ($main: String!) {
                    createAccount(newAccount: {
                        name: "Main"
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
            main_wallet = int(result["createAccount"])
            print(f"Funding wallet: {main_wallet}")

            sync_mutation = gql(
                """
                mutation ($account: Int!) {
                    synchronizeAccount(idAccount: $account)
                }
                """
            )
            await client.execute_async(
                GraphQLRequest(sync_mutation, variable_values={"account": main_wallet})
            )

            balance_query = gql(
                """
                query ($account: Int!) {
                    balanceByAccount(idAccount: $account) {
                        ironwood
                    }
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(balance_query, variable_values={"account": main_wallet})
            )
            funding_balance = result["balanceByAccount"]["ironwood"]
            print(f"Funding wallet balance: {funding_balance}")

        print("\n=== Step 3: Initialize DKG for each participant (n=3, t=2) ===")
        for i, participant in enumerate(participants, 1):
            create_account_mutation = gql(
                """
                mutation {
                    createAccount(newAccount: {
                        name: "DKG-Fund"
                        key: ""
                        aindex: 0
                        useInternal: false
                        birth: 1
                    })
                }
                """
            )
            result = await participant.execute(GraphQLRequest(create_account_mutation))
            participant.funding_account = int(result["createAccount"])

            address_query = gql(
                """
                query ($account: Int!) {
                    addressByAccount(idAccount: $account) {
                        ironwood
                    }
                }
                """
            )
            result = await participant.execute(
                GraphQLRequest(
                    address_query, variable_values={"account": participant.funding_account}
                )
            )
            participant.funding_address = result["addressByAccount"]["ironwood"]

            print(f"Participant {i} funding account: {participant.funding_account}")
            print(f"Participant {i} funding address: {participant.funding_address}")

            dkg_start_mutation = gql(
                """
                mutation ($name: String!, $t: Int!, $n: Int!, $funding: Int!, $id: Int!) {
                    dkgStart(
                        name: $name
                        threshold: $t
                        participants: $n
                        messageAccount: $funding
                        idParticipant: $id
                    )
                }
                """
            )
            result = await participant.execute(
                GraphQLRequest(
                    dkg_start_mutation,
                    variable_values={
                        "name": f"Dkg-Test-2of3-{i}",
                        "t": T,
                        "n": N,
                        "funding": participant.funding_account,
                        "id": i,
                    },
                )
            )
            participant.dkg_address = result["dkgStart"]
            print(f"Participant {i} DKG address: {participant.dkg_address}")

        print("\n=== Step 4: Fund each participant's funding address ===")
        async with gql_client_factory(graphql_url) as client:
            recipients = [{"address": p.funding_address, "amount": "0.01"} for p in participants]

            pay_mutation = gql(
                """
                mutation ($account: Int!, $recipients: [Recipient!]!) {
                    pay(idAccount: $account, payment: {recipients: $recipients})
                }
                """
            )
            result = await client.execute_async(
                GraphQLRequest(
                    pay_mutation,
                    variable_values={"account": main_wallet, "recipients": recipients},
                )
            )
            txid = result["pay"]
            print(f"Funding transaction: {txid}")

        print("\n=== Step 5: Mine blocks for confirmation ===")
        client = await participants[0].get_client()
        height = await get_current_height(client)
        await mine_blocks(rpc_url, 5)
        await wait_for_blocks(client, height, 5)
        print("Blocks mined")

        print("\n=== Step 6: Synchronize funding accounts ===")
        sync_mutation = gql(
            """
            mutation ($account: Int!) {
                synchronizeAccount(idAccount: $account)
            }
            """
        )
        for i, participant in enumerate(participants, 1):
            await participant.execute(
                GraphQLRequest(
                    sync_mutation, variable_values={"account": participant.funding_account}
                )
            )
            print(f"Synchronized participant {i} funding account")

        print("\n=== Step 7: Verify funding accounts received funds ===")
        balance_query = gql(
            """
            query ($account: Int!) {
                balanceByAccount(idAccount: $account) {
                    ironwood
                }
            }
            """
        )
        for i, participant in enumerate(participants, 1):
            result = await participant.execute(
                GraphQLRequest(
                    balance_query, variable_values={"account": participant.funding_account}
                )
            )
            balance = result["balanceByAccount"]["ironwood"]
            print(f"Participant {i} funding account balance: {balance}")
            assert balance and balance != "0", f"Participant {i} has insufficient balance"

        print("\n=== Step 8: Exchange DKG addresses between participants ===")
        set_address_mutation = gql(
            """
            mutation ($id: Int!, $address: String!) {
                dkgSetAddress(idParticipant: $id, address: $address)
            }
            """
        )
        for i, sender in enumerate(participants, 1):
            for j, receiver in enumerate(participants, 1):
                if i == j:
                    continue

                target_address = participants[j - 1].dkg_address
                await sender.execute(
                    GraphQLRequest(
                        set_address_mutation, variable_values={"id": j, "address": target_address}
                    )
                )
                print(f"Participant {i} set address for participant {j}")

        print("\n=== Step 9: Execute DKG on all participants ===")
        do_dkg_mutation = gql(
            """
            mutation {
                doDkg
            }
            """
        )
        for i, participant in enumerate(participants, 1):
            await participant.execute(GraphQLRequest(do_dkg_mutation))
            print(f"Initiated DKG on participant {i}")

        print("\n=== Step 10: Wait for DKG completion ===")

        async def all_dkg_completed():
            return all(p.get_frost_account_id() is not None for p in participants)

        success = await poll_with_block_mining(all_dkg_completed, rpc_url, timeout=300)
        if not success:
            pytest.fail("DKG timed out")
        print("All participants completed DKG successfully")

        print("\n=== Step 11: Verify shared address is same for all participants ===")
        shared_address = None
        address_query = gql(
            """
            query ($account: Int!) {
                addressByAccount(idAccount: $account) {
                    ironwood
                }
            }
            """
        )
        for i, participant in enumerate(participants, 1):
            frost_account = participant.get_frost_account_id()
            assert frost_account is not None, f"Participant {i} has no FROST account"
            result = await participant.execute(
                GraphQLRequest(address_query, variable_values={"account": frost_account})
            )
            addr = result["addressByAccount"]["ironwood"]
            print(f"Participant {i} shared address: {addr}")
            if shared_address is None:
                shared_address = addr
            else:
                assert addr == shared_address, f"Participant {i} has a different shared address"

        print(f"\n=== ✅ 2-of-3 DKG Test Passed! ===")
        print(f"Shared FROST address: {shared_address}")
        print("All 3 participants completed the DKG despite t=2:")
        print("  - every wallet materialized the same complete key set")

    finally:
        await cleanup()
