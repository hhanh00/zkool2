"""The native market page, on the real Flutter app.

The app (integration_test/market_ui_test.dart) opens the page on a fresh
wallet whose `klines_url` prop points at the klines mock served here, walks
the chart styles, intervals, the volume and log toggles, the crosshair and
the go-to-latest button, and waits for the poll of the forming bar. This side
owns the mock and checks its request log: the history request on open and on
an interval change, and the two-bar poll. No chain is needed; the lightwalletd
URL is never contacted.

Run it with Flutter on PATH:
    .venv/bin/python -m pytest tests/test_market_ui.py -v -s
"""

import asyncio
import contextlib
import os
import shutil

import pytest
from dkg import Rendezvous, app_documents_dir, launch_flutter_test
from klines_mock import KlinesMock
from utils import dump_server_log, kill_existing_zkool_processes

LWD_URL = "http://localhost:8137"
FLUTTER_LOG = "/tmp/market_ui_flutter.log"
FRAMES_DIR = "market_frames"
BUILD_TIMEOUT = 2400
BUDGET_SECONDS = 600


@pytest.mark.dkg_ui
@pytest.mark.asyncio
async def test_market_flutter_ui():
    docs_dir = app_documents_dir()
    if not os.path.isdir(docs_dir):
        pytest.skip(f"app documents directory {docs_dir} not found")
    if shutil.which(os.getenv("FLUTTER_BIN", "flutter")) is None:
        pytest.skip("flutter not on PATH")

    ui_db_path = os.path.join(docs_dir, "regtest_market_ui.db")
    frames_dir = os.path.join(docs_dir, FRAMES_DIR)
    rendezvous = Rendezvous(os.path.join(docs_dir, "dkg_ui_rendezvous"))
    mock = KlinesMock()
    flutter = None

    try:
        await kill_existing_zkool_processes()
        rendezvous.setup()
        shutil.rmtree(frames_dir, ignore_errors=True)
        for stale in (ui_db_path, FLUTTER_LOG):
            if os.path.exists(stale):
                os.remove(stale)

        print("=== Step 1: Klines mock and app ===")
        mock.start()
        rendezvous.write_config(
            db_path=ui_db_path,
            lwd=LWD_URL,
            klines_url=mock.url,
            frames_dir=FRAMES_DIR,
        )
        flutter = await launch_flutter_test("market_ui_test.dart", rendezvous, FLUTTER_LOG)

        print("\n=== Step 2: The page walkthrough ===")
        await rendezvous.poll_ui("done", timeout=BUILD_TIMEOUT + BUDGET_SECONDS, process=flutter)
        returncode = await asyncio.wait_for(flutter.wait(), timeout=300)
        assert returncode == 0, f"flutter test failed with {returncode}, see {FLUTTER_LOG}"

        print(f"\n=== Step 3: Requests: {mock.requests} ===")
        histories = [r for r in mock.requests if "limit=1500" in r]
        polls = [r for r in mock.requests if "limit=2" in r]
        assert any("interval=1d" in r for r in histories), "the daily history on open"
        assert any("interval=1h" in r for r in histories), "the hourly history on the interval change"
        assert polls, "the two-bar poll of the forming bar"
        assert all("symbol=ZECUSDC" in r for r in mock.requests)
        print(f"\n=== ✅ Market page: {len(mock.requests)} requests, frames in {frames_dir} ===")
    except Exception:
        print(f"\nLast published: {rendezvous.read_ui()}")
        dump_server_log(FLUTTER_LOG, "FLUTTER TEST LOG")
        raise
    finally:
        if flutter is not None and flutter.returncode is None:
            flutter.terminate()
            with contextlib.suppress(asyncio.TimeoutError):
                await asyncio.wait_for(flutter.wait(), timeout=30)
        mock.stop()
        rendezvous.cleanup()
        if os.path.exists(ui_db_path):
            os.remove(ui_db_path)
