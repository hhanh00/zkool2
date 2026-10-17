"""Upgrade a legacy FROST group to ZIP 2005 and sweep its funds.

Self-contained: forms a 3-of-3 group under DKG version 1 (the legacy
derivation, through the `dkgVersion` argument of `dkgStart`), funds it, and
checks that the group is flagged as not quantum recoverable; every member
then upgrades and reports the same sibling address, the coordinator sweeps
the legacy balance into the sibling as an ordinary signing session, and the
flag clears on both accounts once the funds sit in the Ironwood pool.
"""

import asyncio
import os

import pytest
from dkg import DkgParticipant, poll_with_block_mining
from gql import GraphQLRequest, gql
from gql.transport.exceptions import TransportQueryError
from utils import get_current_height, mine_blocks, wait_for_blocks

N = 3
T = 3
LEGACY_DKG_VERSION = 1
DEFAULT_PORT = 8000
PORT_BASE = 8011
LWD_URL = "http://localhost:8137"

SYNC = gql("mutation ($account: Int!) { synchronizeAccount(idAccount: $account) }")
BALANCE = gql(
    "query ($account: Int!) { balanceByAccount(idAccount: $account) { orchard ironwood } }"
)
ADDRESS = gql("query ($account: Int!) { addressByAccount(idAccount: $account) { ironwood } }")
CREATE_ACCOUNT = gql(
    """
    mutation ($name: String!, $key: String!) {
        createAccount(newAccount: {
            name: $name
            key: $key
            aindex: 0
            useInternal: false
            birth: 1
        })
    }
    """
)
PAY = gql(
    """
    mutation ($account: Int!, $recipients: [Recipient!]!) {
        pay(idAccount: $account, payment: {recipients: $recipients})
    }
    """
)
DKG_START = gql(
    """
    mutation ($name: String!, $t: Int!, $n: Int!, $funding: Int!, $id: Int!, $version: Int!) {
        dkgStart(
            name: $name
            threshold: $t
            participants: $n
            messageAccount: $funding
            idParticipant: $id
            dkgVersion: $version
        )
    }
    """
)
DKG_SET_ADDRESS = gql(
    """
    mutation ($id: Int!, $address: String!) {
        dkgSetAddress(idParticipant: $id, address: $address)
    }
    """
)
DO_DKG = gql("mutation { doDkg }")
FROST_PARAMS = gql(
    """
    query ($account: Int!) {
        frostParams(idAccount: $account) { id n t dkgVersion sibling quantumRecoverable }
    }
    """
)
FROST_UPGRADE = gql("mutation ($account: Int!) { frostUpgrade(idAccount: $account) }")
FROST_SWEEP = gql(
    """
    mutation ($account: Int!, $coordinator: Int!, $funding: Int!) {
        frostSweep(idAccount: $account, idCoordinator: $coordinator, messageAccount: $funding)
    }
    """
)
FROST_SIGN = gql(
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


async def sync_tolerant(wallet: DkgParticipant, account: int) -> bool:
    """Sync an account; False when the protocol driver's own sync is running."""
    try:
        await wallet.execute(GraphQLRequest(SYNC, variable_values={"account": account}))
        return True
    except TransportQueryError as e:
        if "already running" not in str(e):
            raise
        return False


async def sync_until_done(wallet: DkgParticipant, account: int):
    while not await sync_tolerant(wallet, account):
        await asyncio.sleep(2)


async def balance(wallet: DkgParticipant, account: int) -> dict:
    result = await wallet.execute(GraphQLRequest(BALANCE, variable_values={"account": account}))
    return result["balanceByAccount"]


async def address(wallet: DkgParticipant, account: int) -> str:
    result = await wallet.execute(GraphQLRequest(ADDRESS, variable_values={"account": account}))
    return result["addressByAccount"]["ironwood"]


async def frost_params(wallet: DkgParticipant, account: int) -> dict:
    result = await wallet.execute(
        GraphQLRequest(FROST_PARAMS, variable_values={"account": account})
    )
    return result["frostParams"]


async def wait_ready(wallet: DkgParticipant, timeout: int = 60):
    """Wait until a freshly started server answers, instead of a fixed sleep."""
    probe = gql("query { currentHeight }")
    for _ in range(timeout):
        try:
            await wallet.execute(GraphQLRequest(probe))
            return
        except Exception:
            await wallet.close_client()
            await asyncio.sleep(1)
    raise TimeoutError(f"zkool_graphql on port {wallet.port} did not come up")


async def mine_and_wait(funder: DkgParticipant, rpc_url: str, blocks: int):
    client = await funder.get_client()
    height = await get_current_height(client)
    await mine_blocks(rpc_url, blocks)
    await wait_for_blocks(client, height, blocks)


@pytest.mark.asyncio
async def test_frost_legacy_upgrade_and_sweep(
    graphql_url, rpc_url, seed, zkool_binary, gql_client_factory
):
    if not seed:
        pytest.skip("SEED not set")
    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    wallets: list[DkgParticipant] = []

    async def cleanup():
        for w in wallets:
            await w.stop()
        for port in [DEFAULT_PORT, *range(PORT_BASE, PORT_BASE + N)]:
            log_path = f"/tmp/graphql_{port}.log"
            if os.path.exists(log_path):
                os.remove(log_path)

    try:
        from utils import kill_existing_zkool_processes

        await kill_existing_zkool_processes()

        print("=== Setting up the legacy FROST upgrade test ===")
        funder = DkgParticipant(DEFAULT_PORT, "/tmp/regtest_legacy_default.db", LWD_URL)
        funder.start(zkool_binary)
        wallets.append(funder)
        await wait_ready(funder)
        result = await funder.execute(
            GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": "Main", "key": seed})
        )
        main_wallet = int(result["createAccount"])
        await funder.execute(GraphQLRequest(SYNC, variable_values={"account": main_wallet}))
        print(f"Funding wallet balance: {(await balance(funder, main_wallet))['ironwood']}")

        print("\n=== Step 1: Form a legacy (dkg-1) group ===")
        participants: list[DkgParticipant] = []
        for i in range(1, N + 1):
            p = DkgParticipant(PORT_BASE + i - 1, f"/tmp/regtest_legacy_{i}.db", LWD_URL, id=i)
            p.start(zkool_binary)
            wallets.append(p)
            participants.append(p)
        for p in participants:
            await wait_ready(p)
        for i, p in enumerate(participants, 1):
            result = await p.execute(
                GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": "DKG-Fund", "key": ""})
            )
            p.funding_account = int(result["createAccount"])
            p.funding_address = await address(p, p.funding_account)
            result = await p.execute(
                GraphQLRequest(
                    DKG_START,
                    variable_values={
                        "name": f"Dkg-Test-{i}",
                        "t": T,
                        "n": N,
                        "funding": p.funding_account,
                        "id": i,
                        "version": LEGACY_DKG_VERSION,
                    },
                )
            )
            p.dkg_address = result["dkgStart"]
        await funder.execute(
            GraphQLRequest(
                PAY,
                variable_values={
                    "account": main_wallet,
                    "recipients": [
                        {"address": p.funding_address, "amount": "0.01"} for p in participants
                    ],
                },
            )
        )
        await mine_and_wait(funder, rpc_url, 5)
        for p in participants:
            await sync_until_done(p, p.funding_account)
        for i, sender in enumerate(participants, 1):
            for j, peer in enumerate(participants, 1):
                if i != j:
                    await sender.execute(
                        GraphQLRequest(
                            DKG_SET_ADDRESS,
                            variable_values={"id": j, "address": peer.dkg_address},
                        )
                    )
        for p in participants:
            await p.execute(GraphQLRequest(DO_DKG))

        async def dkg_done():
            return all(p.get_frost_account_id() is not None for p in participants)

        assert await poll_with_block_mining(dkg_done, rpc_url, timeout=300), "DKG timed out"
        for p in participants:
            p.frost_account = p.get_frost_account_id()
        legacy_addresses = {await address(p, p.frost_account) for p in participants}
        assert len(legacy_addresses) == 1, legacy_addresses
        legacy_address = legacy_addresses.pop()
        for p in participants:
            params = await frost_params(p, p.frost_account)
            assert params["dkgVersion"] == LEGACY_DKG_VERSION, params
            assert params["sibling"] is None, params
            assert params["quantumRecoverable"], "an empty legacy account has nothing at risk"
        print(f"Legacy group formed at {legacy_address[:24]}…")

        print("\n=== Step 2: Fund the legacy address; the group is flagged ===")
        # The funder spent a note on the participants; see its change first.
        await sync_until_done(funder, main_wallet)
        await funder.execute(
            GraphQLRequest(
                PAY,
                variable_values={
                    "account": main_wallet,
                    "recipients": [{"address": legacy_address, "amount": "0.1"}],
                },
            )
        )
        await mine_and_wait(funder, rpc_url, 5)
        for p in participants:
            await sync_until_done(p, p.frost_account)
            assert (await balance(p, p.frost_account))["ironwood"] == "0.10000000"
            params = await frost_params(p, p.frost_account)
            assert not params["quantumRecoverable"], "legacy funds must be flagged"
        print("Legacy balance 0.1 ZEC, flagged as not quantum recoverable")

        print("\n=== Step 3: Every member upgrades; one sibling address ===")
        sibling_addresses = set()
        for p in participants:
            result = await p.execute(
                GraphQLRequest(FROST_UPGRADE, variable_values={"account": p.frost_account})
            )
            p.sibling = int(result["frostUpgrade"])
            # Idempotent: a second upgrade returns the same account.
            result = await p.execute(
                GraphQLRequest(FROST_UPGRADE, variable_values={"account": p.frost_account})
            )
            assert int(result["frostUpgrade"]) == p.sibling
            sibling_addresses.add(await address(p, p.sibling))
            params = await frost_params(p, p.frost_account)
            assert params["sibling"] == p.sibling, params
            assert not params["quantumRecoverable"], "the flag stays until the sweep"
            params = await frost_params(p, p.sibling)
            assert params["dkgVersion"] == 2 and params["quantumRecoverable"], params
        assert len(sibling_addresses) == 1, sibling_addresses
        sibling_address = sibling_addresses.pop()
        assert sibling_address != legacy_address
        print(f"Sibling address {sibling_address[:24]}… agreed by every member")

        print("\n=== Step 4: Sweep the legacy balance into the sibling ===")
        coordinator = participants[1]
        result = await coordinator.execute(
            GraphQLRequest(
                FROST_SWEEP,
                variable_values={
                    "account": coordinator.frost_account,
                    "coordinator": 2,
                    "funding": coordinator.funding_account,
                },
            )
        )
        pczt = result["frostSweep"]
        for p in participants:
            if p is coordinator:
                continue
            result = await p.execute(
                GraphQLRequest(
                    FROST_SIGN,
                    variable_values={
                        "account": p.frost_account,
                        "coordinator": 2,
                        "funding": p.funding_account,
                        "pczt": pczt,
                    },
                )
            )
            assert result["frostSign"]

        async def swept():
            if not await sync_tolerant(coordinator, coordinator.frost_account):
                return False
            if not await sync_tolerant(coordinator, coordinator.sibling):
                return False
            legacy = await balance(coordinator, coordinator.frost_account)
            sibling = await balance(coordinator, coordinator.sibling)
            print(f"Legacy {legacy}, sibling {sibling}")
            return legacy["ironwood"] == "0" and float(sibling["ironwood"]) > 0

        assert await poll_with_block_mining(swept, rpc_url, timeout=600), "sweep did not complete"

        print("\n=== Step 5: The flag clears on both accounts ===")
        for p in participants:
            await sync_until_done(p, p.frost_account)
            await sync_until_done(p, p.sibling)
            sibling_balance = await balance(p, p.sibling)
            assert sibling_balance["orchard"] == "0", sibling_balance
            assert float(sibling_balance["ironwood"]) > 0.09, sibling_balance
            assert (await frost_params(p, p.frost_account))["quantumRecoverable"]
            assert (await frost_params(p, p.sibling))["quantumRecoverable"]
        print(f"Swept into the Ironwood pool: sibling holds {sibling_balance['ironwood']} ZEC")
        print("\n=== ✅ Upgrade test passed ===")
    finally:
        await cleanup()
