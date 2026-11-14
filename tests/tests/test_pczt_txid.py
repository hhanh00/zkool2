"""decodePczt tells the txid of a prepared transaction before it is signed.

Since ZIP 244 a txid commits to a transaction's effects only, not to its
proofs and signatures, so the PCZT prepareSend returns already fixes it.
One headless server: an account is funded in the transparent, Sapling and
Ironwood pools, then spends from each pool in turn through prepareSend,
decodePczt, signTx and broadcastTx. The txid decodePczt gave must be the
one broadcastTx returns and the one the node knows the transaction by.
"""

import os

import pytest
from dkg import DkgParticipant
from gql import GraphQLRequest, gql
from test_frost_upgrade import SYNC, address, mine_and_wait, sync_until_done, wait_ready
from test_uivk import addresses, create, pay
from test_wallet_ui import rpc

DEFAULT_PORT = 8000
LWD_URL = "http://localhost:8137"
# A fixed seed: on a long-lived chain the account keeps earlier runs'
# coins, which only adds to what each pool can spend.
SPENDER_MNEMONIC = (
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
)
FUND = "0.05"
SPEND = "0.001"
POOLS = [(1, "transparent"), (2, "sapling"), (8, "ironwood")]

PREPARE = gql(
    """
    query ($account: Int!, $address: String!, $amount: BigDecimal!, $srcPools: Int!) {
        prepareSend(
            idAccount: $account
            payment: { recipients: [{ address: $address, amount: $amount }], srcPools: $srcPools }
        )
    }
    """
)
DECODE = gql("query ($pczt: String!) { decodePczt(pczt: $pczt) { txid fee } }")
SIGN = gql("query ($account: Int!, $pczt: String!) { signTx(idAccount: $account, pczt: $pczt) }")
BROADCAST = gql(
    """
    query ($tx: String!, $account: Int!) { broadcastTx(txBytes: $tx, idAccount: $account) }
    """
)


@pytest.mark.asyncio
async def test_decode_pczt_tells_the_txid_before_signing(rpc_url, seed, zkool_binary):
    if not seed:
        pytest.skip("SEED not set")
    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    wallet = DkgParticipant(DEFAULT_PORT, "/tmp/regtest_pczt_txid.db", LWD_URL)
    try:
        from utils import kill_existing_zkool_processes

        await kill_existing_zkool_processes()
        wallet.start(zkool_binary)
        await wait_ready(wallet)
        main = await create(wallet, "Main", seed)
        await wallet.execute(GraphQLRequest(SYNC, variable_values={"account": main}))

        print("\n=== Step 1: Fund one account in three pools ===")
        spender = await create(wallet, "Spender", SPENDER_MNEMONIC)
        spender_addresses = await addresses(wallet, spender)
        for _, pool in POOLS:
            await pay(wallet, rpc_url, main, spender_addresses[pool], FUND)
        await sync_until_done(wallet, spender)
        recipient = await address(wallet, main)

        for src_pools, pool in POOLS:
            print(f"\n=== Spend from {pool} ===")
            prepared = await wallet.execute(
                GraphQLRequest(
                    PREPARE,
                    variable_values={
                        "account": spender,
                        "address": recipient,
                        "amount": SPEND,
                        "srcPools": src_pools,
                    },
                )
            )
            pczt = prepared["prepareSend"]
            decoded = await wallet.execute(GraphQLRequest(DECODE, variable_values={"pczt": pczt}))
            txid = decoded["decodePczt"]["txid"]
            # Null only for a PCZT whose effects are not final; prepareSend's are.
            assert txid is not None and len(txid) == 64, (pool, txid)

            signed = await wallet.execute(
                GraphQLRequest(SIGN, variable_values={"account": spender, "pczt": pczt})
            )
            sent = await wallet.execute(
                GraphQLRequest(
                    BROADCAST, variable_values={"tx": signed["signTx"], "account": spender}
                )
            )
            assert sent["broadcastTx"] == txid, (pool, sent["broadcastTx"], txid)

            # The node holds the transaction under that id.
            raw = await rpc(rpc_url, "getrawtransaction", [txid, 0])
            assert raw.get("error") is None, (pool, raw)
            assert raw["result"] == signed["signTx"], pool
            print(f"{pool}: {txid}")

            await mine_and_wait(wallet, rpc_url, 1)
            await sync_until_done(wallet, spender)
    finally:
        await wallet.stop()
