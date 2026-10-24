"""Screenshots of the docs site's Voting page, taken from the real Flutter
app on regtest with the local voting stack.

The app (integration_test/docs_voting_ui_test.dart) walks the voting
screens: the config URL in the settings, the account menu's Voting item,
the rounds, the ballot, the submission preview, the submission and its end,
the vote report, and the list afterwards. This side owns the chain, the
funding wallet and the voting stack: it pays the app's account one note,
mines to a snapshot height past it, brings the stack up on that snapshot
with the docs' round (tests/tools/voting_stack.sh, which needs the binaries
its skill describes), hands the app the config URL, then draws the
annotations with tests/tools/docs_shots.py and copies the images into the
docs site ($ZKOOL_DOCS_DIR/vote/images, default ~/projects/zkool-docs/docs).
The stack's throwaway CA reaches the app through ZKOOL_EXTRA_ROOT_CERT_PATH,
from a copy inside the app's container. The stack is left up afterwards.

Run it with the regtest stack up and the SEED wallet funded (tests/README.md):

    .venv/bin/python -m pytest tests/test_docs_voting_ui.py -v -s
"""

import asyncio
import contextlib
import os
import shutil
import subprocess
import sys

import httpx
import pytest
from dkg import DkgParticipant, Rendezvous, app_documents_dir, launch_flutter_test
from gql import GraphQLRequest
from test_frost_upgrade import SYNC, wait_ready
from test_key_import import create
from test_uivk import BALANCE, PAY
from utils import dump_server_log, get_current_height, kill_existing_zkool_processes, mine_blocks, wait_for_blocks

DEFAULT_PORT = 8000
LWD_URL = "http://localhost:8137"
FLUTTER_LOG = "/tmp/docs_voting_ui_flutter.log"
FRAMES_DIR = "docs_frames_voting"
DOCS_DIR = os.getenv("ZKOOL_DOCS_DIR", os.path.expanduser("~/projects/zkool-docs/docs"))
TOOLS = os.path.join(os.path.dirname(__file__), "..", "tools")
DOCS_SHOTS = os.path.join(TOOLS, "docs_shots.py")
VOTING_STACK = os.path.join(TOOLS, "voting_stack.sh")
PROPOSALS = os.path.join(TOOLS, "docs_voting_proposals.json")
REPO_ROOT = os.path.abspath(os.path.join(TOOLS, "..", ".."))
BUILD_TIMEOUT = 2400
BUDGET_SECONDS = 2400
# The one note the app votes with.
FUNDING = "0.5"
ROUND_TITLE = "Zcash Community Poll"
# Long enough for the app to sync, fill the ballot and cast the votes (about
# four minutes), short enough for the run: the helper shares are only
# confirmed once the round ends.
VOTE_DURATION = 480


async def rpc(rpc_url: str, method: str, params: list):
    payload = {"jsonrpc": "1.0", "id": method, "method": method, "params": params}
    async with httpx.AsyncClient() as client:
        response = await client.post(rpc_url, json=payload)
        response.raise_for_status()
        return response.json()


def stack_env() -> dict[str, str]:
    """What tests/tools/voting_stack.sh wrote for its clients."""
    values = {}
    with open(os.path.join(REPO_ROOT, "data", "voting", "voting.env")) as f:
        for line in f:
            key, _, value = line.strip().partition("=")
            values[key] = value.strip("'")
    return values


@pytest.mark.dkg_ui
@pytest.mark.asyncio
async def test_docs_voting_flutter_ui(rpc_url, seed, zkool_binary):
    if not seed:
        pytest.skip("SEED not set")
    if not os.path.exists(zkool_binary):
        pytest.skip(f"zkool_graphql binary not found at {zkool_binary}")
    docs_dir = app_documents_dir()
    if not os.path.isdir(docs_dir):
        pytest.skip(f"app documents directory {docs_dir} not found")
    if shutil.which(os.getenv("FLUTTER_BIN", "flutter")) is None:
        pytest.skip("flutter not on PATH")
    if not os.path.isdir(DOCS_DIR):
        pytest.skip(f"docs site not found at {DOCS_DIR} (ZKOOL_DOCS_DIR)")
    for binary in ("svoted", "nf-server", "voting-config"):
        if not os.path.exists(os.path.join(REPO_ROOT, "tools", "voting", binary)):
            pytest.skip(f"tools/voting/{binary} missing: see .claude/skills/voting-stack/SKILL.md")

    ui_db_path = os.path.join(docs_dir, "regtest_docs_voting.db")
    frames_dir = os.path.join(docs_dir, FRAMES_DIR)
    rendezvous = Rendezvous(os.path.join(docs_dir, "dkg_ui_rendezvous"))
    ca_copy = os.path.join(docs_dir, "voting-ca.crt")

    wallet = DkgParticipant(DEFAULT_PORT, "/tmp/regtest_docs_voting_funder.db", LWD_URL)
    flutter = None
    main = None
    saved_ca_env = os.environ.get("ZKOOL_EXTRA_ROOT_CERT_PATH")

    def stack(*args) -> str:
        return subprocess.run([VOTING_STACK, *args], check=True, capture_output=True, text=True, cwd=REPO_ROOT).stdout

    async def mine_to_snapshot() -> int:
        """Mines the payment, then on to the next multiple of ten, which the
        PIR import requires, and waits for lightwalletd to serve it."""
        client = await wallet.get_client()
        height = await get_current_height(client)
        target = (height // 10 + 1) * 10
        await mine_blocks(rpc_url, target - height)
        await wait_for_blocks(client, height, target - height)
        return target

    try:
        await kill_existing_zkool_processes()
        rendezvous.setup()
        shutil.rmtree(frames_dir, ignore_errors=True)
        for stale in (ui_db_path, FLUTTER_LOG):
            if os.path.exists(stale):
                os.remove(stale)

        print("=== Step 1: The funding wallet, and the stack's CA for the app ===")
        shutil.copyfile(stack("ca").strip(), ca_copy)
        os.environ["ZKOOL_EXTRA_ROOT_CERT_PATH"] = ca_copy
        wallet.start(zkool_binary)
        await wait_ready(wallet)
        main = await create(wallet, "Main", seed)
        await wallet.execute(GraphQLRequest(SYNC, variable_values={"account": main}))
        balance = (await wallet.execute(GraphQLRequest(BALANCE, variable_values={"account": main})))["balanceByAccount"]
        if float(balance["total"]) < float(FUNDING) + 0.01:
            pytest.fail(f"the SEED wallet holds {balance['total']} ZEC on this chain: fund it first (tests/README.md)")

        print("\n=== Step 2: Launch the app ===")
        rendezvous.write_config(db_path=ui_db_path, lwd=LWD_URL, frames_dir=FRAMES_DIR, fund=FUNDING)
        flutter = await launch_flutter_test("docs_voting_ui_test.dart", rendezvous, FLUTTER_LOG)

        print("\n=== Step 3: Fund the account, snapshot past it, open the round ===")
        a_ua = await rendezvous.poll_ui("a_ua", timeout=BUILD_TIMEOUT, process=flutter)
        result = await wallet.execute(
            GraphQLRequest(PAY, variable_values={"account": main, "recipients": [{"address": a_ua, "amount": FUNDING}]})
        )
        print(f"Paid {FUNDING} to the app ({result['pay'][:12]}…)")
        snapshot = await mine_to_snapshot()
        print(f"Snapshot height {snapshot}")
        stack("up", "--snapshot", str(snapshot), "--title", ROUND_TITLE, "--proposals", PROPOSALS, "--duration", str(VOTE_DURATION))
        env = stack_env()
        print(f"Round \"{env['VOTING_ROUND_TITLE']}\" at {env['VOTING_CONFIG_URL']}")
        rendezvous.write_orchestrator(config_url=env["VOTING_CONFIG_URL"], round_title=env["VOTING_ROUND_TITLE"], snapshot=snapshot)

        await rendezvous.poll_ui("done", timeout=BUDGET_SECONDS, process=flutter)
        returncode = await asyncio.wait_for(flutter.wait(), timeout=300)
        assert returncode == 0, f"flutter test failed with {returncode}, see {FLUTTER_LOG}"

        print("\n=== Step 4: Annotate ===")
        subprocess.run([sys.executable, "-I", DOCS_SHOTS, frames_dir, "--out", DOCS_DIR], check=True)
        print(f"\n=== ✅ Voting screenshots written under {DOCS_DIR} ===")
    except subprocess.CalledProcessError as e:
        print(f"\n{e.cmd} failed:\n{e.stdout}\n{e.stderr}")
        raise
    except Exception:
        ui = rendezvous.read_ui()
        print(f"\nLast published: {ui}")
        dump_server_log(FLUTTER_LOG, "FLUTTER TEST LOG")
        dump_server_log(f"/tmp/graphql_{DEFAULT_PORT}.log", "FUNDING WALLET")
        raise
    finally:
        if saved_ca_env is None:
            os.environ.pop("ZKOOL_EXTRA_ROOT_CERT_PATH", None)
        else:
            os.environ["ZKOOL_EXTRA_ROOT_CERT_PATH"] = saved_ca_env
        if flutter is not None and flutter.returncode is None:
            flutter.terminate()
            with contextlib.suppress(asyncio.TimeoutError):
                await asyncio.wait_for(flutter.wait(), timeout=30)
        await wallet.stop()
        rendezvous.cleanup()
        if os.path.exists(ui_db_path):
            os.remove(ui_db_path)
        if os.path.exists(ui_db_path + ".voting"):
            os.remove(ui_db_path + ".voting")
