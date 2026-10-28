"""Screenshots of the docs site's Cloud Vault pages, taken from the real
Flutter app.

The app (integration_test/docs_vault_ui_test.dart) opens Settings, shows
the version line whose long press turns expert mode on, and the vault rows
once it is on. The vault needs no chain and no Google account for these two
screens. Turning the switch on runs through the app's unlock gate, which is
a Touch ID prompt on most Macs, so the "Enable Vault" dialog is only shot
with ZKOOL_DOCS_VAULT_DIALOG=1 and someone at the keyboard to pass it.

This side draws the annotations with tests/tools/docs_shots.py and copies
the images into the docs site ($ZKOOL_DOCS_DIR/vault/images, default
~/projects/zkool-docs/docs).

    .venv/bin/python -m pytest tests/test_docs_vault_ui.py -v -s
"""

import asyncio
import contextlib
import os
import shutil
import subprocess
import sys

import pytest
from dkg import Rendezvous, app_documents_dir, launch_flutter_test
from utils import dump_server_log, kill_existing_zkool_processes

LWD_URL = "http://localhost:8137"
FLUTTER_LOG = "/tmp/docs_vault_ui_flutter.log"
FRAMES_DIR = "docs_frames_vault"
DOCS_DIR = os.getenv("ZKOOL_DOCS_DIR", os.path.expanduser("~/projects/zkool-docs/docs"))
DOCS_SHOTS = os.path.join(os.path.dirname(__file__), "..", "tools", "docs_shots.py")
BUILD_TIMEOUT = 2400


@pytest.mark.dkg_ui
@pytest.mark.asyncio
async def test_docs_vault_flutter_ui():
    docs_dir = app_documents_dir()
    if not os.path.isdir(docs_dir):
        pytest.skip(f"app documents directory {docs_dir} not found")
    if shutil.which(os.getenv("FLUTTER_BIN", "flutter")) is None:
        pytest.skip("flutter not on PATH")
    if not os.path.isdir(DOCS_DIR):
        pytest.skip(f"docs site not found at {DOCS_DIR} (ZKOOL_DOCS_DIR)")

    ui_db_path = os.path.join(docs_dir, "regtest_docs_vault.db")
    frames_dir = os.path.join(docs_dir, FRAMES_DIR)
    rendezvous = Rendezvous(os.path.join(docs_dir, "dkg_ui_rendezvous"))
    enable_dialog = os.getenv("ZKOOL_DOCS_VAULT_DIALOG") == "1"
    flutter = None

    try:
        await kill_existing_zkool_processes()
        rendezvous.setup()
        shutil.rmtree(frames_dir, ignore_errors=True)
        for stale in (ui_db_path, FLUTTER_LOG):
            if os.path.exists(stale):
                os.remove(stale)

        print("=== Step 1: Launch the app ===")
        rendezvous.write_config(db_path=ui_db_path, lwd=LWD_URL, frames_dir=FRAMES_DIR, enable_dialog=enable_dialog)
        flutter = await launch_flutter_test("docs_vault_ui_test.dart", rendezvous, FLUTTER_LOG)
        if enable_dialog:
            print("The app will ask to unlock when it taps the Cloud Vault switch: pass the prompt.")

        await rendezvous.poll_ui("done", timeout=BUILD_TIMEOUT, process=flutter)
        returncode = await asyncio.wait_for(flutter.wait(), timeout=300)
        assert returncode == 0, f"flutter test failed with {returncode}, see {FLUTTER_LOG}"

        print("\n=== Step 2: Annotate ===")
        subprocess.run([sys.executable, "-I", DOCS_SHOTS, frames_dir, "--out", DOCS_DIR], check=True)
        print(f"\n=== ✅ Cloud Vault screenshots written under {DOCS_DIR} ===")
    except Exception:
        print(f"\nLast published: {rendezvous.read_ui()}")
        dump_server_log(FLUTTER_LOG, "FLUTTER TEST LOG")
        raise
    finally:
        if flutter is not None and flutter.returncode is None:
            flutter.terminate()
            with contextlib.suppress(asyncio.TimeoutError):
                await asyncio.wait_for(flutter.wait(), timeout=30)
        rendezvous.cleanup()
        if os.path.exists(ui_db_path):
            os.remove(ui_db_path)
