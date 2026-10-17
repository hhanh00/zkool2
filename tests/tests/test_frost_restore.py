"""Restore a FROST member from its zkfrost backup string and sign with it.

Runs after test_dkg.py (and test_frost.py) on the databases they leave behind:
participant 3 exports its backup string, a fresh headless wallet restores
from the string and the group's UFVK with no peer addresses entered, and
the group then signs twice, once with the restored wallet as a plain
participant and once with it as the coordinator. A string checked against
another key is refused.
"""

import asyncio
import os

import pytest
from dkg import DkgParticipant, poll_with_block_mining
from gql import GraphQLRequest, gql
from gql.transport.exceptions import TransportQueryError
from utils import get_current_height, mine_blocks, wait_for_blocks

SYNC = gql("mutation ($account: Int!) { synchronizeAccount(idAccount: $account) }")
BALANCE = gql("query ($account: Int!) { balanceByAccount(idAccount: $account) { ironwood } }")
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
UFVK = gql("query ($account: Int!) { unifiedFullViewingKey(idAccount: $account) }")
EXPORT_BACKUP = gql("mutation ($account: Int!) { frostExportBackup(idAccount: $account) }")
RESTORE = gql(
    """
    mutation ($name: String!, $backup: String!, $ufvk: String!) {
        frostRestore(name: $name, backup: $backup, ufvk: $ufvk, zip2005: true) {
            idAccount dkgVersion
        }
    }
    """
)
PREPARE = gql(
    """
    query ($account: Int!, $address: String!, $amount: BigDecimal!) {
        prepareSend(
            idAccount: $account
            payment: { recipients: [{ address: $address, amount: $amount }] }
        )
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
FROST_CANCEL = gql("mutation { frostCancel }")

N = 4
DEFAULT_PORT = 8000
PORT_BASE = 8001
RESTORED_PORT = 8005
RESTORED_DB = "/tmp/regtest_frost_restored.db"
LWD_URL = "http://localhost:8137"


async def sync_tolerant(wallet: DkgParticipant, account: int) -> bool:
    """Sync an account; False when the protocol driver's own sync is running."""
    try:
        await wallet.execute(GraphQLRequest(SYNC, variable_values={"account": account}))
        return True
    except TransportQueryError as e:
        if "already running" not in str(e):
            raise
        print("Sync busy (driver pass in progress), retrying on the next poll")
        return False


async def balance(wallet: DkgParticipant, account: int) -> str:
    result = await wallet.execute(GraphQLRequest(BALANCE, variable_values={"account": account}))
    return result["balanceByAccount"]["ironwood"]


async def new_receiver(wallet: DkgParticipant, name: str) -> tuple[int, str]:
    result = await wallet.execute(
        GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": name, "key": ""})
    )
    account = int(result["createAccount"])
    result = await wallet.execute(GraphQLRequest(ADDRESS, variable_values={"account": account}))
    return account, result["addressByAccount"]["ironwood"]


async def sign_session(
    rpc_url: str,
    signers: list[tuple[DkgParticipant, int, int, int]],
    coordinator_id: int,
    receiver: tuple[DkgParticipant, int, str],
    amount: str,
):
    """One signing session: `signers` is (wallet, participant id, frost
    account, funding account), `receiver` is (wallet, account, address); the
    coordinator's wallet prepares the payment. Ends when the receiver's
    balance reads `amount`."""
    coordinator = next(s for s in signers if s[1] == coordinator_id)
    wallet, _, frost_account, _ = coordinator

    # Every wallet starts the session clean; a member keeps its finished
    # session's state until told otherwise.
    for w, *_ in signers:
        await w.execute(GraphQLRequest(FROST_CANCEL))

    while not await sync_tolerant(wallet, frost_account):
        await asyncio.sleep(2)
    result = await wallet.execute(
        GraphQLRequest(
            PREPARE,
            variable_values={"account": frost_account, "address": receiver[2], "amount": amount},
        )
    )
    pczt = result["prepareSend"]

    for w, pid, account, funding in signers:
        result = await w.execute(
            GraphQLRequest(
                FROST_SIGN,
                variable_values={
                    "account": account,
                    "coordinator": coordinator_id,
                    "funding": funding,
                    "pczt": pczt,
                },
            )
        )
        assert result["frostSign"], f"participant {pid} failed to start signing"
        print(f"Participant {pid} started signing (coordinator {coordinator_id})")

    receiver_wallet, receiver_account, _ = receiver
    expected = f"{float(amount):.8f}"

    async def paid():
        if not await sync_tolerant(receiver_wallet, receiver_account):
            return False
        got = await balance(receiver_wallet, receiver_account)
        print(f"Receiver balance: {got} ZEC (waiting for {expected} ZEC)")
        return got == expected

    assert await poll_with_block_mining(paid, rpc_url, timeout=600), (
        f"session coordinated by participant {coordinator_id} did not pay the receiver"
    )


@pytest.mark.asyncio
async def test_frost_restore_and_sign(graphql_url, rpc_url, seed, zkool_binary, gql_client_factory):
    if not seed:
        pytest.skip("SEED not set")
    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")

    for i in range(1, N + 1):
        db_path = f"/tmp/regtest_dkg_test_{i}.db"
        if not os.path.exists(db_path) or not DkgParticipant(0, db_path, "").get_frost_account_id():
            pytest.skip("DKG not completed. Run test_dkg first to create shared accounts.")

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

        print("=== Setting up the FROST restore test ===")
        funder = DkgParticipant(DEFAULT_PORT, "/tmp/regtest_dkg_default.db", LWD_URL)
        funder.start(zkool_binary)
        wallets.append(funder)
        await asyncio.sleep(2)
        result = await funder.execute(
            GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": "Main", "key": seed})
        )
        main_wallet = int(result["createAccount"])
        await funder.execute(GraphQLRequest(SYNC, variable_values={"account": main_wallet}))
        print(f"Funding wallet balance: {await balance(funder, main_wallet)}")

        print("\n=== Step 1: Start the original participants ===")
        originals: dict[int, DkgParticipant] = {}
        for i in range(1, N + 1):
            p = DkgParticipant(PORT_BASE + i - 1, f"/tmp/regtest_dkg_test_{i}.db", LWD_URL, id=i)
            p.start(zkool_binary, remove_db=False)
            wallets.append(p)
            originals[i] = p
            p.frost_account = p.get_frost_account_id()
            p.funding_account = p.get_funding_account_id()
            await asyncio.sleep(2)

        print("\n=== Step 2: Participant 3 exports its backup string, then goes away ===")
        lost = originals[3]
        result = await lost.execute(
            GraphQLRequest(EXPORT_BACKUP, variable_values={"account": lost.frost_account})
        )
        backup = result["frostExportBackup"]
        assert backup.startswith("zkfrost1"), backup
        result = await lost.execute(
            GraphQLRequest(UFVK, variable_values={"account": lost.frost_account})
        )
        group_ufvk = result["unifiedFullViewingKey"]
        print(f"Backup string: {backup[:24]}… ({len(backup)} chars)")
        print(f"Group UFVK: {group_ufvk[:24]}…")
        await lost.stop()
        wallets.remove(lost)

        print("\n=== Step 3: A fresh wallet restores from the string and the UFVK ===")
        restored = DkgParticipant(RESTORED_PORT, RESTORED_DB, LWD_URL, id=3)
        restored.start(zkool_binary)
        wallets.append(restored)
        await asyncio.sleep(2)

        # The restored wallet pays for its own protocol memos.
        result = await restored.execute(
            GraphQLRequest(CREATE_ACCOUNT, variable_values={"name": "DKG-Fund", "key": ""})
        )
        restored.funding_account = int(result["createAccount"])
        result = await restored.execute(
            GraphQLRequest(ADDRESS, variable_values={"account": restored.funding_account})
        )
        restored.funding_address = result["addressByAccount"]["ironwood"]
        pay = gql(
            """
            mutation ($account: Int!, $recipients: [Recipient!]!) {
                pay(idAccount: $account, payment: {recipients: $recipients})
            }
            """
        )
        await funder.execute(
            GraphQLRequest(
                pay,
                variable_values={
                    "account": main_wallet,
                    "recipients": [{"address": restored.funding_address, "amount": "0.02"}],
                },
            )
        )
        client = await funder.get_client()
        height = await get_current_height(client)
        await mine_blocks(rpc_url, 5)
        await wait_for_blocks(client, height, 5)
        await restored.execute(
            GraphQLRequest(SYNC, variable_values={"account": restored.funding_account})
        )
        funding_balance = await balance(restored, restored.funding_account)
        print(f"Restored wallet funding balance: {funding_balance}")

        # A string checked against a key from another group is refused.
        result = await funder.execute(
            GraphQLRequest(UFVK, variable_values={"account": main_wallet})
        )
        other_ufvk = result["unifiedFullViewingKey"]
        with pytest.raises(TransportQueryError, match="does not belong"):
            await restored.execute(
                GraphQLRequest(
                    RESTORE,
                    variable_values={"name": "Wrong", "backup": backup, "ufvk": other_ufvk},
                )
            )
        print("Restore with another group's key refused")

        result = await restored.execute(
            GraphQLRequest(
                RESTORE,
                variable_values={"name": "Dkg-Test-3", "backup": backup, "ufvk": group_ufvk},
            )
        )
        (restored_account,) = result["frostRestore"]
        restored.frost_account = int(restored_account["idAccount"])
        assert restored_account["dkgVersion"] == 2, result
        print(f"Restored FROST account {restored.frost_account}, DKG version 2")

        # The restored account sees the group's balance after a rescan from
        # Orchard activation.
        while not await sync_tolerant(restored, restored.frost_account):
            await asyncio.sleep(2)
        while not await sync_tolerant(originals[2], originals[2].frost_account):
            await asyncio.sleep(2)
        restored_balance = await balance(restored, restored.frost_account)
        assert restored_balance == await balance(originals[2], originals[2].frost_account)
        assert float(restored_balance) > 0, "the group has no funds to sign with"
        print(f"Restored FROST balance: {restored_balance} ZEC, matches the group")

        # Every member signs: the group is t-of-t, with the restored wallet
        # standing in for participant 3.
        signers = [
            (originals[i], i, originals[i].frost_account, originals[i].funding_account)
            if i != 3
            else (restored, 3, restored.frost_account, restored.funding_account)
            for i in range(1, N + 1)
        ]

        print("\n=== Step 4: Sign with the restored wallet as a participant ===")
        account, address = await new_receiver(originals[2], "Receiver-A")
        await sign_session(rpc_url, signers, 2, (originals[2], account, address), "0.01")

        print("\n=== Step 5: Sign with the restored wallet as the coordinator ===")
        account, address = await new_receiver(restored, "Receiver-B")
        await sign_session(rpc_url, signers, 3, (restored, account, address), "0.01")

        print("\n=== ✅ Restore test passed ===")
    finally:
        await cleanup()
