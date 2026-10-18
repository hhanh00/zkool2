"""Export an incoming viewing key and watch an account with it.

Self-contained, one headless server: a full account from a fresh seed is
funded, its incoming viewing key is exported and imported as a second
account, and the two are compared: same receiving addresses, same notes,
same received total, and the imported one flagged incoming-only. The
transparent-only incoming key, an xpub of the external chain, imports the
same way. The full account then spends: the watcher still shows the
payment as received and nothing as spent, since an incoming key cannot see
spends, and it refuses to send.
"""

import os
from decimal import Decimal

import pytest
from dkg import DkgParticipant
from gql import GraphQLRequest, gql
from gql.transport.exceptions import TransportQueryError
from test_frost_upgrade import SYNC, address, mine_and_wait, sync_until_done, wait_ready

DEFAULT_PORT = 8000
LWD_URL = "http://localhost:8137"
# The full account's seed. Its default diversifier index on regtest is 1,
# not 0, so that matching addresses between the full account and the
# watcher proves the watcher derives the same index, not just index 0.
FULL_MNEMONIC = (
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
)

CREATE_ACCOUNT = gql(
    """
    mutation ($name: String!, $key: String!, $useInternal: Boolean!) {
        createAccount(newAccount: {
            name: $name
            key: $key
            aindex: 0
            useInternal: $useInternal
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
PREPARE = gql(
    """
    query ($account: Int!, $recipients: [Recipient!]!) {
        prepareSend(idAccount: $account, payment: {recipients: $recipients})
    }
    """
)
VIEWING_KEY = gql(
    """
    query ($account: Int!, $pools: Int, $incoming: Boolean) {
        unifiedFullViewingKey(idAccount: $account, pools: $pools, incoming: $incoming)
    }
    """
)
ACCOUNTS = gql("query { accounts { id name incomingOnly } }")
BALANCE = gql(
    """
    query ($account: Int!) {
        balanceByAccount(idAccount: $account) { transparent sapling orchard ironwood total }
    }
    """
)
ADDRESSES = gql(
    """
    query ($account: Int!) {
        addressByAccount(idAccount: $account) {
            ua transparent sapling orchard ironwood diversifierIndex
        }
    }
    """
)
NOTES = gql(
    """
    query ($account: Int!) {
        notesByAccount(idAccount: $account) { height pool scope value address }
    }
    """
)


async def create(wallet: DkgParticipant, name: str, key: str, use_internal: bool = False) -> int:
    """`use_internal` stays off for the SEED wallet, as in every other test:
    an instance created without it never scans the internal scope, so change
    sent there would vanish for the tests that run after this one."""
    result = await wallet.execute(
        GraphQLRequest(
            CREATE_ACCOUNT,
            variable_values={"name": name, "key": key, "useInternal": use_internal},
        )
    )
    return int(result["createAccount"])


async def viewing_key(wallet: DkgParticipant, account: int, pools=None, incoming=None) -> str:
    result = await wallet.execute(
        GraphQLRequest(
            VIEWING_KEY,
            variable_values={"account": account, "pools": pools, "incoming": incoming},
        )
    )
    return result["unifiedFullViewingKey"]


async def balance(wallet: DkgParticipant, account: int) -> dict:
    result = await wallet.execute(GraphQLRequest(BALANCE, variable_values={"account": account}))
    return result["balanceByAccount"]


async def addresses(wallet: DkgParticipant, account: int) -> dict:
    result = await wallet.execute(GraphQLRequest(ADDRESSES, variable_values={"account": account}))
    return result["addressByAccount"]


async def notes(wallet: DkgParticipant, account: int) -> list:
    result = await wallet.execute(GraphQLRequest(NOTES, variable_values={"account": account}))
    return sorted(result["notesByAccount"], key=lambda n: (n["height"], n["pool"], n["value"]))


async def shielded_notes(wallet: DkgParticipant, account: int) -> list:
    """Shielded notes only: an incoming key never sees these spent, while a
    transparent coin's spend is visible to anyone, by its outpoint."""
    return [n for n in await notes(wallet, account) if n["pool"] != 0]


async def incoming_only(wallet: DkgParticipant) -> dict[int, bool]:
    result = await wallet.execute(GraphQLRequest(ACCOUNTS))
    return {a["id"]: a["incomingOnly"] for a in result["accounts"]}


async def pay(wallet: DkgParticipant, rpc_url: str, account: int, to: str, amount: str):
    # The payer may have spent since it last synced; see its change first.
    await sync_until_done(wallet, account)
    await wallet.execute(
        GraphQLRequest(
            PAY,
            variable_values={"account": account, "recipients": [{"address": to, "amount": amount}]},
        )
    )
    await mine_and_wait(wallet, rpc_url, 5)


@pytest.mark.asyncio
async def test_incoming_viewing_key_export_and_import(
    graphql_url, rpc_url, seed, zkool_binary, gql_client_factory
):
    if not seed:
        pytest.skip("SEED not set")
    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    wallet = DkgParticipant(DEFAULT_PORT, "/tmp/regtest_uivk.db", LWD_URL)
    try:
        from utils import kill_existing_zkool_processes

        await kill_existing_zkool_processes()
        wallet.start(zkool_binary)
        await wait_ready(wallet)
        main = await create(wallet, "Main", seed)
        await wallet.execute(GraphQLRequest(SYNC, variable_values={"account": main}))

        print("\n=== Step 1: A full account, funded ===")
        # Internal change, so that the full account's change stays invisible
        # to the watcher: what it sees is exactly what was paid in.
        full = await create(wallet, "Full", FULL_MNEMONIC, use_internal=True)
        await pay(wallet, rpc_url, main, await address(wallet, full), "0.2")
        await sync_until_done(wallet, full)
        # The seed is fixed, so on a long-lived chain the account may carry
        # earlier runs' payments and spends too; the assertions below hold
        # either way.
        full_before = Decimal((await balance(wallet, full))["ironwood"])
        assert full_before >= Decimal("0.2"), full_before

        print("\n=== Step 2: Export the incoming key, import it as a watcher ===")
        ufvk = await viewing_key(wallet, full)
        uivk = await viewing_key(wallet, full, incoming=True)
        assert ufvk.startswith("uview") and uivk.startswith("uivk"), (ufvk, uivk)
        watch = await create(wallet, "Watch", uivk)
        await sync_until_done(wallet, watch)
        flags = await incoming_only(wallet)
        assert flags[watch] and not flags[full] and not flags[main], flags

        print("\n=== Step 3: Same addresses, every payment received, nothing else ===")
        full_addresses = await addresses(wallet, full)
        assert full_addresses["diversifierIndex"] != "0", "a seed with a non-trivial index"
        assert await addresses(wallet, watch) == full_addresses
        # The watcher holds every external shielded note the full account
        # holds, and only external ones: change (internal scope) is invisible
        # to it, and a note the full account already spent is still there.
        watched = await shielded_notes(wallet, watch)
        assert watched and all(n["scope"] == 0 for n in watched), watched
        for note in await shielded_notes(wallet, full):
            if note["scope"] == 0:
                assert note in watched, note
        received = (await balance(wallet, watch))["ironwood"]
        assert Decimal(received) >= full_before, (received, full_before)
        print(f"Watcher sees {received} ZEC received at {watched[0]['address'][:24]}…")

        print("\n=== Step 4: The transparent-only incoming key is an xpub and imports ===")
        txpub = await viewing_key(wallet, full, pools=1, incoming=True)
        assert txpub.startswith("xpub"), txpub
        twatch = await create(wallet, "Watch-T", txpub)
        t_addresses = await addresses(wallet, twatch)
        assert t_addresses["sapling"] is None and t_addresses["orchard"] is None, t_addresses
        assert (await incoming_only(wallet))[twatch]
        # A transparent-only key has no shielded key to pick a default
        # diversifier index from, so it shows external address 0, as an
        # imported xpub always has, while the full account shows its default
        # index. The two chains are the same: a payment to the watcher's
        # address lands in the full account, which registered index 0 too.
        assert t_addresses["transparent"] != full_addresses["transparent"]
        t_before = Decimal((await balance(wallet, full))["transparent"])
        await pay(wallet, rpc_url, main, t_addresses["transparent"], "0.01")
        for account in [full, twatch]:
            await sync_until_done(wallet, account)
        t_full = (await balance(wallet, full))["transparent"]
        assert Decimal(t_full) == t_before + Decimal("0.01"), t_full
        assert (await balance(wallet, twatch))["transparent"] == t_full

        print("\n=== Step 5: The full account spends; the watcher sees no spend ===")
        receiver = await create(wallet, "Receiver", "")
        await pay(wallet, rpc_url, full, await address(wallet, receiver), "0.05")
        for account in [full, watch, receiver]:
            await sync_until_done(wallet, account)
        assert (await balance(wallet, receiver))["ironwood"] == "0.05000000"
        full_balance = await balance(wallet, full)
        assert Decimal(full_balance["ironwood"]) <= full_before - Decimal("0.05"), full_balance
        assert (await balance(wallet, watch))["ironwood"] == received, "received, not spent"
        assert await shielded_notes(wallet, watch) == watched, "the spent note still shows as received"

        print("\n=== Step 6: The watcher refuses to send ===")
        with pytest.raises(TransportQueryError, match="incoming payments only"):
            await wallet.execute(
                GraphQLRequest(
                    PREPARE,
                    variable_values={
                        "account": watch,
                        "recipients": [{"address": await address(wallet, receiver), "amount": "0.01"}],
                    },
                )
            )
        print("\n=== ✅ Incoming viewing key test passed ===")
    finally:
        await wallet.stop()
        log_path = f"/tmp/graphql_{DEFAULT_PORT}.log"
        if os.path.exists(log_path):
            os.remove(log_path)
