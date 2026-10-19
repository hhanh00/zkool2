"""Import every key type the account classifier accepts, on regtest.

One headless server. A full account is created from a fixed mnemonic and
every other key type derived from that mnemonic is imported beside it: the
unified full and incoming viewing keys and their transparent-only xpub forms
are exported live, the rest are the committed vectors below. Each import is
checked end to end: the addresses it shows, the pools it has, the
incoming-only flag, the balance it sees after the full account's pools are
funded, and, for the imports that hold a spending key, a spend. The two
device-only Ledger modes stay in test_ledger.py.

The vectors are printed by `print_key_import_vectors` in rust/src/account.rs:
`cargo test print_key_import_vectors -- --ignored --nocapture`.
"""

import os
from decimal import Decimal

import pytest
from dkg import DkgParticipant
from gql import GraphQLRequest, gql
from gql.transport.exceptions import TransportQueryError
from test_frost_upgrade import SYNC, address, sync_until_done, wait_ready
from test_uivk import PAY, PREPARE, addresses, balance, incoming_only, pay, viewing_key
from utils import dump_server_log, kill_existing_zkool_processes

DEFAULT_PORT = 8000
LWD_URL = "http://localhost:8137"

# ── Vectors, from print_key_import_vectors on regtest ─────────────────────────
MNEMONIC = (
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon "
    "about"
)
XPRV = (
    "xprv9xiGX2q91Zr2DSdtDHeTJHfaTPvi4JdFw4x5HjSSJcPztu96LbN8juUH4QNfS9bKYLo3jcJv9JWbUcPwbebxPX"
    "zEbu7PA3zXaCucrZSYXEK"
)
XPUB_ACCOUNT = (
    "xpub6BhcvYN2qwQKRviMKKBTfRcK1RmCTmM7JHsg67r3rwvymhUEt8gPHhnkugQaQ7UN8M5FfhEUfyVuSaK5fQzfUp"
    "vAcCxN4bAT9jyySbPGsTs"
)
XPUB_EXTERNAL = (
    "xpub6E2Z9V9KLRQ7iUGCvKrHaf79cNjtQgdhNGSWrzgcWkNMLvfc6c1fzARF9wcKsN3ZqckyhKieioFvxuozXgj6FR"
    "XprRPguJHNL2yb63mnr4C"
)
# The change chain (depth 4, child 1): not a supported import.
XPUB_CHANGE = (
    "xpub6E2Z9V9KLRQ7jFc3h7m5JJpzf6XJAFq68zzvq1LGBc8gBYzxE9LwYh9s23MT71N2p8WykfpzWDB2NWGW7Dg34e"
    "RTzh22PpuNhv5f4eGmKng"
)
WIF_COMPRESSED = "L4jNzRuAe1kHvGj7N5UXd3m9erYWRkV6EnXiHR6AoMM37dEzUkTV"
WIF_UNCOMPRESSED = "5KWzYbBuw9n9pQxJkk9xRUpADCgJzia6ie3t3Ms7JJJEAmSWSgw"
ZPK = "zpk1q2n52yu4wdfknuhvmlyzns8hwn5gaufs8hl9ktcymw4txzjnth7av4qkx6d"
SAPLING_SK = (
    "secret-extended-key-regtest1q0mkjujgqqqqpqxutgr3nmuaggzg9kp4ul62ntkkzs9q3z22mz27u8ggq67jf4"
    "e738z8yvmmv57e6vev6c3k5g9glwulzx0aqr2hpmxp2a8yc2qgzrvsqv246myr0p5ctt7nrf0zc0zlp5p5h8pveakp"
    "fwfs2jkfackwmjg84lwepmwm3uhnu0npga2h7d28djsdw9mwc0zmg8wyds3dutfmgzngnpfpxu2c5cxvf65p26tkqe"
    "gly5367qvaujcqdf34sfckh7z65qc83wpvh"
)
SAPLING_VK = (
    "zxviewregtestsapling1q0mkjujgqqqqpqxutgr3nmuaggzg9kp4ul62ntkkzs9q3z22mz27u8ggq67jf4e73x0j5"
    "4yzkdfmfgah766ja37us8xy2zf660yptz9qrpp0uwlkwm66r25r0jkfgyzydj9lv8hp4nm43htg3nwwqw22yr5jvmc"
    "g78fxu9884lwepmwm3uhnu0npga2h7d28djsdw9mwc0zmg8wyds3dutfmgzngnpfpxu2c5cxvf65p26tkqegly5367"
    "qvaujcqdf34sfckh7z65qcua7huh"
)
# The same spending key with the mainnet prefix: refused on regtest.
SAPLING_SK_MAINNET = (
    "secret-extended-key-main1q0mkjujgqqqqpqxutgr3nmuaggzg9kp4ul62ntkkzs9q3z22mz27u8ggq67jf4e73"
    "8z8yvmmv57e6vev6c3k5g9glwulzx0aqr2hpmxp2a8yc2qgzrvsqv246myr0p5ctt7nrf0zc0zlp5p5h8pveakpfwf"
    "s2jkfackwmjg84lwepmwm3uhnu0npga2h7d28djsdw9mwc0zmg8wyds3dutfmgzngnpfpxu2c5cxvf65p26tkqegly"
    "5367qvaujcqdf34sfckh7z65qcu2m4r2"
)

# External transparent address 0, which every transparent import shows, and
# the address of the same key with an uncompressed public key.
T0_ADDRESS = "tmF1xjfhsSzhy55dmhorzTnKjtHhZmPKzts"
T0_UNCOMPRESSED_ADDRESS = "tmFmdZZ4Yx7q6fEYAC3FyKpLnWGtfXbJ9Ap"
# The seed's default diversifier index on regtest is 1, so its default
# transparent address differs from the imports' index 0.
SEED_DINDEX = "1"
SEED_T1_ADDRESS = "tmV1zYhR2xisn6VWdCNKHpeD4S7L1U1nPH6"
SAPLING_ADDRESS = (
    "zregtestsapling1j9s20dpe62kr0jhfvnqua3j8wv873ccy7u3l3fuwgvc8l4yy0dqpnaknkp3g0uaj0jynj459np"
    "t"
)
SEED_UA = (
    "uregtest1cylemdn3wy8urmyq26w023d0xw2v8pnntjm2gqhwxg77aj4da4pw5tj09hqzyvdffx4xz8dsl7ddjauzu"
    "qzelg4r5khhlqau32cw45tlmgmwvjp4u72fp6xnwhxg2rj8hxs0wcpqyhfr0uemapxhp3xs48r5l9ragvldx5g5nqj"
    "5q4c4"
)
# The Zondax Ledger app derives Sapling differently from the same phrase.
ZONDAX_SAPLING_ADDRESS = (
    "zregtestsapling15n6hmmukrkdgjal84g0mtnl2fj2kf80yfnjnxz6778znjcp9vgs8grul5a2uf5pgxaxnzm8je0"
    "g"
)

HW_ZONDAX = 1
# The imports that all show external transparent address 0 and share its coins.
TRANSPARENT_T0 = {
    "xprv", "xpub", "xpub-exported", "xpub-external", "xpub-external-exported", "wif", "zpk",
}
FUND_IRONWOOD = "0.3"
FUND_SAPLING = "0.2"
FUND_T0_EACH = "0.1"  # three coins, so three transparent signers can each spend one
FUND_T0_UNCOMPRESSED = "0.1"
FUND_ZONDAX = "0.05"
SPEND = "0.05"
# What the uncompressed WIF account moves to itself in Step 7: by then it holds
# FUND_T0_UNCOMPRESSED less the Step 6 payment and its fee, under SPEND.
SELF_SPEND = "0.02"

CREATE_ACCOUNT = gql(
    """
    mutation ($name: String!, $key: String!, $hw: Int) {
        createAccount(newAccount: {
            name: $name
            key: $key
            aindex: 0
            birth: 1
            useInternal: false
            hw: $hw
        })
    }
    """
)


async def create(wallet: DkgParticipant, name: str, key: str, hw: int | None = None) -> int:
    result = await wallet.execute(
        GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": name, "key": key, "hw": hw})
    )
    return int(result["createAccount"])


async def refuse(wallet: DkgParticipant, name: str, key: str, message: str):
    with pytest.raises(TransportQueryError, match=message):
        await create(wallet, name, key)


def pools_of(addrs: dict) -> set[str]:
    return {pool for pool in ("transparent", "sapling", "orchard") if addrs[pool] is not None}


def dec(value: str | None) -> Decimal:
    return Decimal(value or "0")


TRANSACTIONS = gql(
    """
    query ($account: Int!) {
        transactionsByAccount(idAccount: $account) { height value fee }
    }
    """
)


def request_transactions(account: int) -> GraphQLRequest:
    return GraphQLRequest(TRANSACTIONS, variable_values={"account": account})


@pytest.mark.asyncio
async def test_import_every_key_type(rpc_url, seed, zkool_binary):
    if not seed:
        pytest.skip("SEED not set")
    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    wallet = DkgParticipant(DEFAULT_PORT, "/tmp/regtest_key_import.db", LWD_URL)
    try:
        await kill_existing_zkool_processes()
        wallet.start(zkool_binary)
        await wait_ready(wallet)
        main = await create(wallet, "Main", seed)
        await wallet.execute(GraphQLRequest(SYNC, variable_values={"account": main}))

        print("\n=== Step 1: The full account and the keys exported from it ===")
        full = await create(wallet, "Seed", MNEMONIC)
        full_addresses = await addresses(wallet, full)
        assert full_addresses["diversifierIndex"] == SEED_DINDEX, full_addresses
        assert full_addresses["ua"] == SEED_UA
        assert full_addresses["transparent"] == SEED_T1_ADDRESS
        assert full_addresses["sapling"] == SAPLING_ADDRESS
        ufvk = await viewing_key(wallet, full)
        uivk = await viewing_key(wallet, full, incoming=True)
        assert ufvk.startswith("uview") and uivk.startswith("uivk"), (ufvk, uivk)
        # The exported xpubs carry a placeholder parent fingerprint, so they
        # are not byte-equal to the derived vectors; both import below.
        txpub_full = await viewing_key(wallet, full, pools=1)
        txpub_incoming = await viewing_key(wallet, full, pools=1, incoming=True)
        assert txpub_full.startswith("xpub") and txpub_incoming.startswith("xpub")

        print("\n=== Step 2: Import every key type ===")
        imports = {
            "xprv": await create(wallet, "T-xprv", XPRV),
            "xpub": await create(wallet, "T-xpub", XPUB_ACCOUNT),
            "xpub-exported": await create(wallet, "T-xpub-exp", txpub_full),
            "xpub-external": await create(wallet, "T-xpub-ext", XPUB_EXTERNAL),
            "xpub-external-exported": await create(wallet, "T-xpub-ext-exp", txpub_incoming),
            "wif": await create(wallet, "T-wif", WIF_COMPRESSED),
            "wif-uncompressed": await create(wallet, "T-wif-u", WIF_UNCOMPRESSED),
            "zpk": await create(wallet, "T-zpk", ZPK),
            "sapling-sk": await create(wallet, "S-sk", SAPLING_SK),
            "sapling-vk": await create(wallet, "S-vk", SAPLING_VK),
            "ufvk": await create(wallet, "U-fvk", ufvk),
            "uivk": await create(wallet, "U-ivk", uivk),
            "zondax": await create(wallet, "Zondax", MNEMONIC, hw=HW_ZONDAX),
        }
        flags = await incoming_only(wallet)
        for kind, account in imports.items():
            addrs = await addresses(wallet, account)
            expected_incoming = kind in ("xpub-external", "xpub-external-exported", "uivk")
            assert flags[account] == expected_incoming, (kind, flags[account])
            if kind in TRANSPARENT_T0:
                assert pools_of(addrs) == {"transparent"}, (kind, addrs)
                assert addrs["transparent"] == T0_ADDRESS, (kind, addrs)
            elif kind == "wif-uncompressed":
                assert pools_of(addrs) == {"transparent"}, (kind, addrs)
                assert addrs["transparent"] == T0_UNCOMPRESSED_ADDRESS, (kind, addrs)
            elif kind in ("sapling-sk", "sapling-vk"):
                assert pools_of(addrs) == {"sapling"}, (kind, addrs)
                assert addrs["sapling"] == SAPLING_ADDRESS, (kind, addrs)
            elif kind in ("ufvk", "uivk"):
                assert addrs == full_addresses, (kind, addrs)
            elif kind == "zondax":
                assert pools_of(addrs) == {"sapling"}, (kind, addrs)
                assert addrs["sapling"] == ZONDAX_SAPLING_ADDRESS, (kind, addrs)
            shown = addrs["ua"] or addrs["transparent"] or addrs["sapling"]
            print(f"{kind:22} account {account} {shown}")
        assert not flags[full] and not flags[main]

        print("\n=== Step 3: Keys that must be refused ===")
        await refuse(wallet, "Bad-xpub", XPUB_CHANGE, "Unsupported xpub at depth 4, child 1")
        await refuse(wallet, "Bad-sapling", SAPLING_SK_MAINNET, "Unsupported key")
        await refuse(wallet, "Bad-garbage", "garbage", "Unsupported key")

        print("\n=== Step 4: Fund every pool the full account has ===")
        await pay(wallet, rpc_url, main, full_addresses["ua"], FUND_IRONWOOD)
        await pay(wallet, rpc_url, main, full_addresses["sapling"], FUND_SAPLING)
        for _ in range(3):
            await pay(wallet, rpc_url, main, T0_ADDRESS, FUND_T0_EACH)
        await pay(wallet, rpc_url, main, T0_UNCOMPRESSED_ADDRESS, FUND_T0_UNCOMPRESSED)
        await pay(wallet, rpc_url, main, ZONDAX_SAPLING_ADDRESS, FUND_ZONDAX)

        print("\n=== Step 5: Each import sees exactly what its pools allow ===")
        for account in [full, *imports.values()]:
            await sync_until_done(wallet, account)
        full_balance = await balance(wallet, full)
        print(f"full account: {full_balance}")
        # The mnemonic is fixed, so on a long-lived chain the accounts carry
        # earlier runs' payments and spends too; compare the imports to the
        # full account rather than to the amounts paid in.
        assert dec(full_balance["transparent"]) >= Decimal(FUND_T0_EACH) * 3
        assert dec(full_balance["sapling"]) >= Decimal(FUND_SAPLING)
        assert dec(full_balance["ironwood"]) >= Decimal(FUND_IRONWOOD)
        balances = {kind: await balance(wallet, account) for kind, account in imports.items()}
        for kind, b in balances.items():
            print(f"{kind:22} {b}")
        for kind in TRANSPARENT_T0:
            b = balances[kind]
            assert b["transparent"] == full_balance["transparent"], (kind, b)
            assert dec(b["sapling"]) == 0 and dec(b["ironwood"]) == 0, (kind, b)
        assert dec(balances["wif-uncompressed"]["transparent"]) >= Decimal(FUND_T0_UNCOMPRESSED)
        for kind in ("sapling-sk", "sapling-vk"):
            b = balances[kind]
            assert b["sapling"] == full_balance["sapling"], (kind, b)
            assert dec(b["transparent"]) == 0 and dec(b["ironwood"]) == 0, (kind, b)
        assert balances["ufvk"] == full_balance, balances["ufvk"]
        # An incoming key sees received shielded notes but never their spends,
        # so after earlier runs it may show more than the full account.
        uivk_balance = balances["uivk"]
        assert uivk_balance["transparent"] == full_balance["transparent"], uivk_balance
        assert dec(uivk_balance["sapling"]) >= dec(full_balance["sapling"]), uivk_balance
        assert dec(uivk_balance["ironwood"]) >= dec(full_balance["ironwood"]), uivk_balance
        zondax_balance = balances["zondax"]
        assert dec(zondax_balance["sapling"]) >= Decimal(FUND_ZONDAX), zondax_balance
        assert dec(zondax_balance["transparent"]) == 0 and dec(zondax_balance["ironwood"]) == 0

        print("\n=== Step 6: Every spending key spends; the watchers cannot ===")
        receiver = await create(wallet, "Receiver", "")
        receiver_address = await address(wallet, receiver)
        spenders = ["xprv", "wif", "wif-uncompressed", "sapling-sk"]
        before = {kind: dec((await balance(wallet, imports[kind]))["total"]) for kind in spenders}
        before["full"] = dec(full_balance["total"])
        for kind in spenders:
            await pay(wallet, rpc_url, imports[kind], receiver_address, SPEND)
            print(f"{kind} paid {SPEND}")
        await pay(wallet, rpc_url, full, receiver_address, SPEND)
        print(f"full account paid {SPEND}")
        for account in [full, receiver, *imports.values()]:
            await sync_until_done(wallet, account)
        received = (await balance(wallet, receiver))["ironwood"]
        assert Decimal(received) == Decimal(SPEND) * (len(spenders) + 1), received
        for kind in spenders:
            after = dec((await balance(wallet, imports[kind]))["total"])
            assert after <= before[kind] - Decimal(SPEND), (kind, before[kind], after)
        after_full = dec((await balance(wallet, full))["total"])
        assert after_full <= before["full"] - Decimal(SPEND), (before["full"], after_full)
        # The watchers of the same keys follow the spends: a transparent
        # coin's spend is visible to anyone, by its outpoint.
        t_after = (await balance(wallet, imports["xprv"]))["transparent"]
        for kind in TRANSPARENT_T0 - {"xprv", "wif"}:
            assert (await balance(wallet, imports[kind]))["transparent"] == t_after, kind
        # An incoming key cannot even plan a payment: it has no change address.
        # A full viewing key plans one, for an offline signer, but cannot sign.
        recipients = [{"address": receiver_address, "amount": SPEND}]
        def request(query, kind):
            return GraphQLRequest(
                query, variable_values={"account": imports[kind], "recipients": recipients}
            )

        for kind in ("xpub-external", "xpub-external-exported", "uivk"):
            with pytest.raises(TransportQueryError, match="incoming payments only"):
                await wallet.execute(request(PREPARE, kind))
        for kind in ("xpub", "xpub-exported", "zpk", "sapling-vk", "ufvk"):
            await wallet.execute(request(PREPARE, kind))
            with pytest.raises(TransportQueryError):
                await wallet.execute(request(PAY, kind))

        print("\n=== Step 7: A self-transfer from the uncompressed key reads as one ===")
        # An uncompressed-key coin's spend is bigger, so ZIP 317 charges more
        # for it. The fee the wallet records for the transaction must be the
        # fee it paid, or a self-transfer shows up as a payment to someone.
        legacy = imports["wif-uncompressed"]
        own_address = (await addresses(wallet, legacy))["transparent"]
        await pay(wallet, rpc_url, legacy, own_address, SELF_SPEND)
        await sync_until_done(wallet, legacy)
        txs = (await wallet.execute(request_transactions(legacy)))["transactionsByAccount"]
        latest = max(txs, key=lambda tx: tx["height"])
        assert Decimal(latest["value"]) == -Decimal(latest["fee"]), latest
        print(f"self-transfer: value {latest['value']}, fee {latest['fee']}")
        print("\n=== ✅ Key import test passed ===")
        print(f"Receiver got {received} from {len(spenders) + 1} signers")
    except Exception:
        dump_server_log(f"/tmp/graphql_{DEFAULT_PORT}.log", "SERVER LOG")
        raise
    finally:
        await wallet.stop()
        log_path = f"/tmp/graphql_{DEFAULT_PORT}.log"
        if os.path.exists(log_path):
            os.remove(log_path)
