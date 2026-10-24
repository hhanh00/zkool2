"""A stand-in for the Binance USDⓈ-M futures klines endpoint the market chart calls.

The wallet reads its klines API base from the `klines_url` prop, so a test
points it here. One endpoint is served, `/fapi/v1/klines`, in Binance's
format: an array of 12-field arrays with the open time in milliseconds and
open, high, low, close and volume as decimal strings.

Candles are a seeded random walk, so a given (seed, interval, limit) always
produces the same series and tests can assert on exact values. Open times are
aligned to the interval, and the last candle is the one forming now.
"""

import json
import random
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

INTERVAL_SECONDS = {
    "1m": 60,
    "3m": 180,
    "5m": 300,
    "15m": 900,
    "30m": 1800,
    "1h": 3600,
    "2h": 7200,
    "4h": 14400,
    "6h": 21600,
    "8h": 28800,
    "12h": 43200,
    "1d": 86400,
    "3d": 259200,
    "1w": 604800,
    "1M": 2592000,
}
DEFAULT_LIMIT = 500
MAX_LIMIT = 1500


def make_candles(interval: str, limit: int, seed: int = 1, start_price: float = 50.0, now: int | None = None):
    """The `limit` most recent candles of `interval`, oldest first, as
    (open_time_seconds, open, high, low, close, volume) tuples."""
    step = INTERVAL_SECONDS[interval]
    now = int(time.time()) if now is None else now
    last_open = now - now % step
    rng = random.Random(f"{seed}:{interval}")
    price = start_price
    candles = []
    for i in range(limit):
        open_time = last_open - (limit - 1 - i) * step
        open_ = price
        close = max(0.01, open_ * (1 + rng.uniform(-0.03, 0.03)))
        high = max(open_, close) * (1 + rng.uniform(0, 0.01))
        low = min(open_, close) * (1 - rng.uniform(0, 0.01))
        volume = rng.uniform(1000, 100000)
        candles.append((open_time, open_, high, low, close, volume))
        price = close
    return candles


def binance_rows(candles, interval: str):
    step = INTERVAL_SECONDS[interval]
    rows = []
    for open_time, open_, high, low, close, volume in candles:
        quote_volume = volume * (open_ + close) / 2
        rows.append(
            [
                open_time * 1000,
                f"{open_:.2f}",
                f"{high:.2f}",
                f"{low:.2f}",
                f"{close:.2f}",
                f"{volume:.3f}",
                (open_time + step) * 1000 - 1,
                f"{quote_volume:.2f}",
                int(volume / 10),
                f"{volume / 2:.3f}",
                f"{quote_volume / 2:.2f}",
                "0",
            ]
        )
    return rows


class KlinesMock:
    def __init__(self, seed: int = 1, start_price: float = 50.0):
        self.seed = seed
        self.start_price = start_price
        self.requests: list[str] = []
        mock = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_):  # keep pytest output clean
                pass

            def do_GET(self):
                url = urlparse(self.path)
                query = {k: v[0] for k, v in parse_qs(url.query).items()}
                mock.requests.append(self.path)
                status, body = mock.respond(url.path, query)
                data = json.dumps(body).encode()
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)

        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)

    @property
    def url(self) -> str:
        return f"http://127.0.0.1:{self.server.server_address[1]}"

    def start(self):
        self.thread.start()
        print(f"Klines mock at {self.url} with seed {self.seed}")

    def stop(self):
        self.server.shutdown()
        self.server.server_close()

    def candles(self, interval: str, limit: int):
        """The series a request for `interval` and `limit` returns, for assertions."""
        return make_candles(interval, limit, self.seed, self.start_price)

    def respond(self, path: str, query: dict[str, str]):
        if path != "/fapi/v1/klines":
            return 404, {"code": -1, "msg": "Not found"}
        interval = query.get("interval", "")
        if interval not in INTERVAL_SECONDS:
            return 400, {"code": -1120, "msg": "Invalid interval."}
        if query.get("symbol", "") != "ZECUSDC":
            return 400, {"code": -1121, "msg": "Invalid symbol."}
        try:
            limit = int(query.get("limit", DEFAULT_LIMIT))
        except ValueError:
            return 400, {"code": -1100, "msg": "Illegal characters found in parameter 'limit'."}
        if limit < 1 or limit > MAX_LIMIT:
            return 400, {"code": -1130, "msg": "Data sent for parameter 'limit' is not valid."}
        return 200, binance_rows(self.candles(interval, limit), interval)
