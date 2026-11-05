"""A 2-of-3 FROST group with Use Internal Change on (#1624).

Every member turns the flag on, since members are responsible for keeping it in
sync. The group spends, every member finds the change at scope 1, and a
different coordinator spends that change. A member restored from backup starts
with the flag off, turns it on, and the rescan brings it level with the others.
"""

import asyncio
import os
import sqlite3

import pytest
from dkg import DkgParticipant, poll_with_block_mining
from gql import GraphQLRequest, gql
from test_frost_restore import (
    ADDRESS,
    CREATE_ACCOUNT,
    EXPORT_BACKUP,
    RESTORE,
    SYNC,
    UFVK,
    balance,
    new_receiver,
    sign_session,
    sync_tolerant,
)
from utils import get_current_height, mine_blocks, wait_for_blocks

N = 3
T = 2
DEFAULT_PORT = 8000
PORT_BASE = 8201
RESTORED_PORT = 8204
DEFAULT_DB = "/tmp/regtest_frost_internal_default.db"
RESTORED_DB = "/tmp/regtest_frost_internal_restored.db"
LWD_URL = "http://localhost:8137"

EDIT_INTERNAL = gql(
    """
    mutation ($account: Int!, $flag: Boolean!) {
        editAccount(idAccount: $account, updateAccount: { useInternal: $flag })
    }
    """
)
DKG_START = gql(
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
DKG_SET_ADDRESS = gql(
    """
    mutation ($id: Int!, $address: String!) {
        dkgSetAddress(idParticipant: $id, address: $address)
    }
    """
)
DO_DKG = gql("mutation { doDkg }")
PAY = gql(
    """
    mutation ($account: Int!, $recipients: [Recipient!]!) {
        pay(idAccount: $account, payment: { recipients: $recipients })
    }
    """
)


def stored(wallet: DkgParticipant, sql: str, account: int) -> int:
    conn = sqlite3.connect(wallet.db_path)
    try:
        return conn.execute(sql, (account,)).fetchone()[0]
    finally:
        conn.close()


def internal_notes(wallet: DkgParticipant, account: int) -> int:
    return stored(wallet, "SELECT COUNT(*) FROM notes WHERE account = ? AND scope = 1", account)


def use_internal(wallet: DkgParticipant, account: int) -> int:
    return stored(wallet, "SELECT use_internal FROM accounts WHERE id_account = ?", account)


async def sync(wallet: DkgParticipant, account: int):
    while not await sync_tolerant(wallet, account):
        await asyncio.sleep(2)


async def balances_match(rpc_url: str, wallets: list[DkgParticipant], expected: str | None = None):
    """Mines until every wallet's group balance agrees; returns that balance."""
    seen: list[str] = []

    async def agree():
        seen.clear()
        for w in wallets:
            if not await sync_tolerant(w, w.frost_account):
                return False
            seen.append(await balance(w, w.frost_account))
        print(f"Group balances: {seen}")
        return len(set(seen)) == 1 and (expected is None or seen[0] == expected)

    assert await poll_with_block_mining(agree, rpc_url, timeout=300), (
        f"group balances never matched: {seen}, expected {expected}"
    )
    return seen[0]


@pytest.mark.asyncio
async def test_frost_internal_change(rpc_url, seed, zkool_binary):
    if not seed:
        pytest.skip("SEED not set")
    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    wallets: list[DkgParticipant] = []

    async def cleanup():
        for w in wallets:
            await w.stop()
        for port in [DEFAULT_PORT, *range(PORT_BASE, PORT_BASE + N), RESTORED_PORT]:
            log_path = f"/tmp/graphql_{port}.log"
            if os.path.exists(log_path):
                os.remove(log_path)

    try:
        from utils import kill_existing_zkool_processes

        await kill_existing_zkool_processes()

        funder = DkgParticipant(DEFAULT_PORT, DEFAULT_DB, LWD_URL)
        funder.start(zkool_binary)
        wallets.append(funder)
        await asyncio.sleep(2)
        result = await funder.execute(
            GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": "Main", "key": seed})
        )
        main_wallet = int(result["createAccount"])
        await funder.execute(GraphQLRequest(SYNC, variable_values={"account": main_wallet}))

        print("=== Step 1: Run a 2-of-3 DKG ===")
        members: list[DkgParticipant] = []
        for i in range(1, N + 1):
            p = DkgParticipant(PORT_BASE + i - 1, f"/tmp/regtest_frost_internal_{i}.db", LWD_URL, id=i)
            p.start(zkool_binary)
            wallets.append(p)
            members.append(p)
            await asyncio.sleep(2)
            p.funding_account, p.funding_address = await new_receiver(p, "DKG-Fund")
            result = await p.execute(
                GraphQLRequest(
                    DKG_START,
                    variable_values={
                        "name": f"Dkg-Test-{i}",
                        "t": T,
                        "n": N,
                        "funding": p.funding_account,
                        "id": i,
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
                        {"address": p.funding_address, "amount": "0.05"} for p in members
                    ],
                },
            )
        )
        client = await members[0].get_client()
        height = await get_current_height(client)
        await mine_blocks(rpc_url, 5)
        await wait_for_blocks(client, height, 5)
        for p in members:
            await p.execute(GraphQLRequest(SYNC, variable_values={"account": p.funding_account}))
            for other in members:
                if other is not p:
                    await p.execute(
                        GraphQLRequest(
                            DKG_SET_ADDRESS,
                            variable_values={"id": other.id, "address": other.dkg_address},
                        )
                    )
        for p in members:
            await p.execute(GraphQLRequest(DO_DKG))

        async def dkg_done():
            return all(p.get_frost_account_id() is not None for p in members)

        assert await poll_with_block_mining(dkg_done, rpc_url, timeout=300), "DKG timed out"
        for p in members:
            p.frost_account = p.get_frost_account_id()
        print("DKG completed")

        print("\n=== Step 2: Every member turns Use Internal Change on ===")
        for p in members:
            assert not use_internal(p, p.frost_account), "a FROST account starts with the flag off"
            await p.execute(
                GraphQLRequest(
                    EDIT_INTERNAL, variable_values={"account": p.frost_account, "flag": True}
                )
            )
            assert use_internal(p, p.frost_account)

        print("\n=== Step 3: Fund the group ===")
        result = await members[0].execute(
            GraphQLRequest(ADDRESS, variable_values={"account": members[0].frost_account})
        )
        group_address = result["addressByAccount"]["ironwood"]
        await mine_blocks(rpc_url, 5)
        await funder.execute(GraphQLRequest(SYNC, variable_values={"account": main_wallet}))
        await funder.execute(
            GraphQLRequest(
                PAY,
                variable_values={
                    "account": main_wallet,
                    "recipients": [{"address": group_address, "amount": "0.1"}],
                },
            )
        )
        await balances_match(rpc_url, members, "0.10000000")

        def signer(p: DkgParticipant):
            return (p, p.id, p.frost_account, p.funding_account)

        print("\n=== Step 4: Members 1 and 2 spend, coordinated by member 1 ===")
        account, address = await new_receiver(members[0], "Receiver-A")
        await sign_session(
            rpc_url,
            [signer(members[0]), signer(members[1])],
            1,
            (members[0], account, address),
            "0.01",
        )
        after_first = await balances_match(rpc_url, members)
        assert 0 < float(after_first) < 0.09, after_first
        for p in members:
            assert internal_notes(p, p.frost_account) == 1, f"member {p.id} missed the change"
        print(f"All members found the change at scope 1, balance {after_first} ZEC")

        print("\n=== Step 5: Members 2 and 3 spend the change, coordinated by member 2 ===")
        # The change is the group's only note, so this has to spend it.
        account, address = await new_receiver(members[1], "Receiver-B")
        await sign_session(
            rpc_url,
            [signer(members[1]), signer(members[2])],
            2,
            (members[1], account, address),
            "0.06",
        )
        after_second = await balances_match(rpc_url, members)
        assert float(after_second) < float(after_first) - 0.06, after_second
        for p in members:
            assert internal_notes(p, p.frost_account) == 2, f"member {p.id} missed the change"
        print(f"The group spent its internal change, balance {after_second} ZEC")

        print("\n=== Step 6: Member 3 restores from backup and turns the flag on ===")
        lost = members[2]
        result = await lost.execute(
            GraphQLRequest(EXPORT_BACKUP, variable_values={"account": lost.frost_account})
        )
        backup = result["frostExportBackup"]
        result = await lost.execute(
            GraphQLRequest(UFVK, variable_values={"account": lost.frost_account})
        )
        group_ufvk = result["unifiedFullViewingKey"]
        await lost.stop()
        wallets.remove(lost)

        restored = DkgParticipant(RESTORED_PORT, RESTORED_DB, LWD_URL, id=3)
        restored.start(zkool_binary)
        wallets.append(restored)
        await asyncio.sleep(2)
        result = await restored.execute(
            GraphQLRequest(
                RESTORE,
                variable_values={"name": "Dkg-Test-3", "backup": backup, "ufvk": group_ufvk},
            )
        )
        restored.frost_account = int(result["frostRestore"][0]["idAccount"])
        assert not use_internal(restored, restored.frost_account), "restore starts with it off"

        # Synced first, so only the reset can bring back what it scanned past.
        await sync(restored, restored.frost_account)
        await restored.execute(
            GraphQLRequest(
                EDIT_INTERNAL, variable_values={"account": restored.frost_account, "flag": True}
            )
        )
        await balances_match(rpc_url, [members[0], members[1], restored], after_second)
        assert internal_notes(restored, restored.frost_account) == 2
        print(f"Restored member's balance matches the group: {after_second} ZEC")

        print("\n=== ✅ FROST internal change test passed ===")
    finally:
        await cleanup()
