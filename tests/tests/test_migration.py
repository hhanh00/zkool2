"""End-to-end note migration on regtest: Orchard → split → SD → Ironwood.

Migration is driven headlessly through the GraphQL `stepMigration` mutation,
which runs one step of the pipeline per call.

Two properties of the system shape this test:

* **Orchard can only be funded before Ironwood activates.** After NU6.3 there
  is no pool-restricted address — `addressByAccount` returns the same string
  for `orchard` and `ironwood` — so a payment lands in Ironwood and legacy
  Orchard notes can no longer be created. The chain must therefore activate
  NU6.3 above the funding height, and a chain cannot be rewound, so both
  subjects are funded up front in a single test rather than one per test.
* **Migration does not synchronize.** The wallet's autosync does that in the
  app, so this driver calls `synchronizeAccount` itself between steps and
  parks the wallet on an anchor boundary before an O→I step can run.

Requires a fresh regtest chain mined into the window between coinbase
maturity (100) and NU6.3 activation (250), so there is spendable coinbase and
Ironwood is not yet live:

    zebrad -c misc/zebra.toml start
    curl --data-binary '{"jsonrpc":"1.0","method":"generate","params":[150]}' \
        -H 'Content-type: application/json' http://127.0.0.1:18232/
    lightwalletd --no-tls-very-insecure --data-dir=./data/regtest \
        --grpc-bind-addr=127.0.0.1:8137 --zcash-conf-path=./misc/zebra.conf
    pytest tests/test_migration.py
"""

import os

import pytest
from gql import GraphQLRequest, gql

from utils import (
    cleanup_test_files,
    dump_server_log,
    create_account,
    get_address,
    get_balance,
    get_current_height,
    kill_existing_zkool_processes,
    mine_blocks,
    start_zkool_instance,
    stop_zkool_instance,
    sync_account,
    wait_for_blocks,
)

# Must match crate::migrate::ANCHOR_BUCKET_SIZE, which step_migration uses.
ANCHOR_BUCKET_SIZE = 144
# Must match crate::migrate::MIN_SD (100 * COST_PER_ACTION), in zats.
MIN_SD = 500_000
# The smallest standard denomination, 10^5 + SD_FEE_PAD.
SD_MIN_DENOM = 120_000
# Must match crate::migrate::SD_FEE_PAD (4 * COST_PER_ACTION). Every Orchard SD
# note carries this to pay for its own O→I hop, so each migrate step should
# cost exactly this much.
SD_FEE_PAD = 20_000
# Migration must not eat an unreasonable share of the balance. Loose, because
# the per-transaction cost is fixed and so weighs heavily on a small wallet.
MAX_FEE_FRACTION = 0.25
# --coin 2: regtest. Passed explicitly; the server would otherwise infer the
# network from the database filename.
REGTEST_COIN = 2
ZATS_PER_ZEC = 100_000_000
# Coinbase maturity: the miner's notes are only spendable this far back.
MATURITY = 100

# NU6.3 (Ironwood) activation. This must agree with BOTH misc/zebra.toml and
# the hardcoded regtest parameters in rust/src/api/coin.rs (nu6_3: 250) — the
# wallet picks its consensus branch id from its own copy, so a chain that
# disagrees rejects every transaction it builds.
NU6_3_HEIGHT = int(os.getenv("NU6_3_HEIGHT", "250"))
# Owns the coinbase: matches miner_address in misc/zebra.toml.
MINER_SEED = os.getenv(
    "MINER_SEED",
    "burger voice warrior danger satoshi you solid atom elite alcohol category "
    "layer able debate culture talk tissue language hip surge fiction paddle "
    "stove voyage",
)

STEP_MIGRATION_MUTATION = gql(
    """
    mutation ($account: Int!) {
        stepMigration(idAccount: $account) {
            event
            fee
            message
        }
    }
    """
)

NOTES_QUERY = gql(
    """
    query ($account: Int!) {
        notesByAccount(idAccount: $account) {
            pool
            value
        }
    }
    """
)

PAY_FROM_COINBASE_MUTATION = gql(
    """
    mutation ($account: Int!, $address: String!, $amount: BigDecimal!, $confirmations: Int!) {
        pay(
            idAccount: $account
            payment: {
                recipients: [{address: $address, amount: $amount}]
                srcPools: 1
                confirmations: $confirmations
            }
        )
    }
    """
)


def _missing(reason: str):
    """Fail in CI, skip locally: a missing prerequisite must never be green."""
    if os.getenv("CI"):
        pytest.fail(f"prerequisite not met: {reason}")
    pytest.skip(reason)


def is_iw_sd(value: int) -> bool:
    """A pure standard denomination, 10^k for k >= 5 — mirrors crate::migrate::is_iw_sd.

    Ironwood notes are minted at the bare denomination; the SD_FEE_PAD an
    Orchard SD note carried is consumed by the hop that mints it.
    """
    if value < 100_000 or value % 100_000:
        return False
    x = value // 100_000
    while x % 10 == 0:
        x //= 10
    return x == 1


async def assert_migrated(
    client, account_id: int, funded: int, steps: list[tuple[str, int]], label: str
):
    """Check the shape of a finished migration, not just that it finished."""
    fees = sum(fee for _, fee in steps)
    ironwood_notes = await pool_note_values(client, account_id, pool=3)
    leftover_notes = await orchard_zec_values(client, account_id)
    ironwood = sum(ironwood_notes)
    leftover = sum(leftover_notes)
    print(f"{label}: ironwood={ironwood_notes} leftover={leftover_notes} fees={fees}")

    # Ironwood notes exist afterwards, each at a standard denomination.
    assert ironwood_notes, f"{label}: no Ironwood notes after migration"
    for value in ironwood_notes:
        assert is_iw_sd(value), f"{label}: Ironwood note {value} is not a denomination"

    # Whatever is left in Orchard is below the threshold worth splitting.
    assert leftover < MIN_SD, f"{label}: {leftover} zats still migratable in Orchard"

    # Migration was not expensive. Each O→I hop is prefunded by the SD_FEE_PAD
    # baked into its note, so it must cost exactly that; the split is the only
    # step whose fee varies with the wallet's shape.
    for event, fee in steps:
        if event == "MigrateComplete":
            assert fee == SD_FEE_PAD, f"{label}: O→I cost {fee}, expected {SD_FEE_PAD}"
    assert fees < funded * MAX_FEE_FRACTION, (
        f"{label}: migration spent {fees} of {funded} "
        f"({fees / funded:.1%}, limit {MAX_FEE_FRACTION:.0%})"
    )

    # Nothing lost, nothing invented.
    assert ironwood + leftover + fees == funded, (
        f"{label}: {ironwood} + {leftover} + {fees} != {funded} funded "
        f"(off by {ironwood + leftover + fees - funded})"
    )


async def orchard_zec_values(client, account_id: int) -> list[int]:
    return await pool_note_values(client, account_id, pool=2)


async def pool_note_values(client, account_id: int, pool: int) -> list[int]:
    """Unspent ZEC note values in `pool` (2 = Orchard, 3 = Ironwood), in zats.

    `notesByAccount` reports values in ZEC, not zats. It exposes no asset
    field to filter ZSA notes by, but ZSA is NU7 and cannot be active on an
    Ironwood (NU6.3) chain, so pool 2 here is always ZEC.
    """
    result = await client.execute_async(
        GraphQLRequest(NOTES_QUERY, variable_values={"account": account_id})
    )
    return [
        round(float(n["value"]) * ZATS_PER_ZEC)
        for n in result["notesByAccount"]
        if int(n["pool"]) == pool
    ]


async def fund_orchard(client, rpc_url: str, miner_id: int, address: str, amount: str):
    """Pay `amount` from mature coinbase and confirm it."""
    height = await get_current_height(client)
    result = await client.execute_async(
        GraphQLRequest(
            PAY_FROM_COINBASE_MUTATION,
            variable_values={
                "account": miner_id,
                "address": address,
                "amount": amount,
                "confirmations": MATURITY,
            },
        )
    )
    # `pay` reports broadcast failures in its String result rather than as a
    # GraphQL error, so a txid is the only success signal.
    txid = result["pay"]
    assert "failed" not in txid.lower(), f"pay failed: {txid}"
    await mine_blocks(rpc_url, 3)
    await wait_for_blocks(client, height, 3)
    # Re-sync the payer, or the next payment reselects the UTXO just spent.
    await sync_account(client, miner_id)


async def mine_to_anchor_boundary(client, rpc_url: str) -> int:
    """Mine so the tip lands exactly on an anchor boundary, and return it.

    An O→I is only built when the wallet checkpoint sits on a shared boundary,
    so a caller syncs immediately after this.
    """
    height = await get_current_height(client)
    remainder = height % ANCHOR_BUCKET_SIZE
    needed = ANCHOR_BUCKET_SIZE - remainder if remainder else 0
    if needed:
        await mine_blocks(rpc_url, needed)
        await wait_for_blocks(client, height, needed)
    boundary = await get_current_height(client)
    assert boundary % ANCHOR_BUCKET_SIZE == 0, f"tip {boundary} is not a boundary"
    return boundary


async def drive_migration(
    client, rpc_url: str, account_id: int, max_steps: int = 40
) -> list[tuple[str, int]]:
    """Run the migration to completion, returning (event, fee) per step.

    Between steps this does what the app's autosync would: park the wallet on
    an anchor boundary, sync, and confirm each broadcast.
    """
    steps: list[tuple[str, int]] = []

    for _ in range(max_steps):
        boundary = await mine_to_anchor_boundary(client, rpc_url)
        await sync_account(client, account_id)

        result = await client.execute_async(
            GraphQLRequest(STEP_MIGRATION_MUTATION, variable_values={"account": account_id})
        )
        step = result["stepMigration"]
        event = step["event"]
        steps.append((event, step["fee"] or 0))
        print(f"  step -> {event} (fee={step['fee']}) at boundary {boundary}")

        if event == "Complete":
            return steps
        if event == "Error":
            pytest.fail(f"migration reported an error: {step['message']}")
        if event in ("SplitComplete", "MigrateComplete"):
            height = await get_current_height(client)
            await mine_blocks(rpc_url, 3)
            await wait_for_blocks(client, height, 3)
            await sync_account(client, account_id)

    pytest.fail(f"migration did not finish in {max_steps} steps: {[e for e, _ in steps]}")


@pytest.mark.asyncio
async def test_migration_orchard_to_ironwood(gql_client_factory, rpc_url, zkool_binary, lwd_url):
    """Orchard notes split into standard denominations and migrate to Ironwood.

    Covers two subjects on one chain:

    1. A wallet with an ordinary Orchard balance migrates and terminates.
    2. A wallet whose Orchard total lands in [MIN_SD, MIN_SD + SD_MIN_DENOM)
       also splits. A flat MIN_SD fee reserve used to be subtracted before
       denominating, which removed every planned output for totals in that
       window: no split was broadcast, no note was spent, and the runner span
       forever on an unchanging wallet while reporting the splitting phase.
    """
    # A skip is a green build. Anything that means the fixture is wrong must
    # fail in CI rather than quietly passing.
    if not os.path.exists(zkool_binary):
        _missing(f"zkool_graphql binary not found at {zkool_binary}")

    PORT = 8003
    DB_PATH = "/tmp/regtest_migration.db"
    LOG_PATH = "/tmp/graphql_migration.log"
    GRAPHQL_URL = f"http://localhost:{PORT}/graphql"
    process = None

    try:
        await kill_existing_zkool_processes()
        cleanup_test_files(DB_PATH, LOG_PATH)
        process = await start_zkool_instance(
            zkool_binary, DB_PATH, PORT, lwd_url, LOG_PATH, coin=REGTEST_COIN
        )
        assert process.poll() is None, "zkool_graphql failed to start"

        async with gql_client_factory(GRAPHQL_URL) as client:
            height = await get_current_height(client)
            if height >= NU6_3_HEIGHT:
                _missing(
                    f"chain is at {height}, past NU6.3 activation ({NU6_3_HEIGHT}): "
                    "Orchard can no longer be funded. Bring the chain up with "
                    "BLOCKS below the activation height."
                )
            assert height > MATURITY, f"need mature coinbase, chain is only at {height}"

            print("\n=== Fund in Orchard, before Ironwood activates ===")
            miner_id = await create_account(client, "Miner", key=MINER_SEED)
            await sync_account(client, miner_id)
            miner_balance = await get_balance(client, miner_id, pool="transparent")
            print(f"Miner coinbase: {miner_balance}")
            assert float(miner_balance) > 0, "miner has no coinbase to spend"

            ordinary_id = await create_account(client, "Ordinary", key="")
            dead_zone_id = await create_account(client, "DeadZone", key="")

            # 0.0055 ZEC = 550_000 zats decomposes to 4 x 120_000 with 70_000
            # left over: the exact shape from the reported stall.
            for account_id, amount in ((ordinary_id, "0.05"), (dead_zone_id, "0.0055")):
                address = await get_address(client, account_id, pool="orchard")
                await fund_orchard(client, rpc_url, miner_id, address, amount)
                await sync_account(client, account_id)

            ordinary_notes = await orchard_zec_values(client, ordinary_id)
            dead_zone_notes = await orchard_zec_values(client, dead_zone_id)
            print(f"Ordinary Orchard notes: {ordinary_notes}")
            print(f"Dead-zone Orchard notes: {dead_zone_notes}")

            # Orchard notes exist before migration — otherwise there is
            # nothing to migrate and every later check is vacuous.
            assert ordinary_notes, "funding did not land in Orchard"
            assert dead_zone_notes, "dead-zone funding did not land in Orchard"
            dead_zone_total = sum(dead_zone_notes)
            assert MIN_SD <= dead_zone_total < MIN_SD + SD_MIN_DENOM, (
                f"total {dead_zone_total} is outside the dead zone; adjust the amount"
            )

            print(f"\n=== Mine past NU6.3 activation ({NU6_3_HEIGHT}) ===")
            height = await get_current_height(client)
            needed = NU6_3_HEIGHT - height + 1
            await mine_blocks(rpc_url, needed)
            await wait_for_blocks(client, height, needed)
            print(f"Height now {await get_current_height(client)}")

            print("\n=== Subject 1: ordinary balance ===")
            steps = await drive_migration(client, rpc_url, ordinary_id)
            events = [event for event, _ in steps]
            print(f"Events: {events}")
            assert "SplitComplete" in events, "no split was ever built"
            assert "MigrateComplete" in events, "nothing reached Ironwood"
            assert events[-1] == "Complete"
            await assert_migrated(client, ordinary_id, sum(ordinary_notes), steps, "ordinary")

            print("\n=== Subject 2: dead zone [500_000, 620_000) ===")
            steps = await drive_migration(client, rpc_url, dead_zone_id)
            events = [event for event, _ in steps]
            print(f"Events: {events}")
            assert "SplitComplete" in events, "dead-zone total planned no split"
            assert "MigrateComplete" in events, "dead-zone split never reached Ironwood"
            assert events[-1] == "Complete", "migration did not terminate"
            await assert_migrated(client, dead_zone_id, dead_zone_total, steps, "dead-zone")
    except Exception:
        dump_server_log(LOG_PATH, "MIGRATION SERVER LOG")
        raise
    finally:
        if process:
            await stop_zkool_instance(process)
        cleanup_test_files(DB_PATH, LOG_PATH)
