"""A stand-in for the CoinGecko endpoints the wallet calls, with fixed prices.

The wallet reads its API base from the `coingecko_url` prop, so a test points
it here and can assert exact fiat amounts. Three endpoints are served:
`simple/price`, `simple/supported_vs_currencies` and `coins/zcash/market_chart`.
The chart returns one quote per hour at the fixed price, from `days` ago to an
hour from now, so every transaction time finds a quote.
"""

import json
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

DEFAULT_PRICES = {"usd": 50.0, "eur": 45.0, "btc": 0.0005}


class CoingeckoMock:
    def __init__(self, prices: dict[str, float] | None = None):
        self.prices = dict(prices or DEFAULT_PRICES)
        self.requests: list[str] = []
        mock = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_):  # keep pytest output clean
                pass

            def do_GET(self):
                url = urlparse(self.path)
                query = {k: v[0] for k, v in parse_qs(url.query).items()}
                mock.requests.append(url.path)
                body = mock.respond(url.path, query)
                if body is None:
                    self.send_response(404)
                    self.end_headers()
                    return
                data = json.dumps(body).encode()
                self.send_response(200)
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
        print(f"CoinGecko mock at {self.url} with prices {self.prices}")

    def stop(self):
        self.server.shutdown()
        self.server.server_close()

    def respond(self, path: str, query: dict[str, str]):
        if path == "/api/v3/simple/price":
            wanted = query.get("vs_currencies", "usd").split(",")
            return {"zcash": {c: self.prices[c] for c in wanted if c in self.prices}}
        if path == "/api/v3/simple/supported_vs_currencies":
            return sorted(self.prices)
        if path == "/api/v3/coins/zcash/market_chart":
            currency = query.get("vs_currency", "usd")
            days = int(query.get("days", "1"))
            price = self.prices[currency]
            now = int(time.time())
            start = now - days * 86400
            return {"prices": [[t * 1000, price] for t in range(start, now + 3600, 3600)]}
        return None
