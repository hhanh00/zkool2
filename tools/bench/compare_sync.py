"""A/B sync benchmark: zkool vs. the Slipstream sync engine.

Both engines sync the *same wallet* (one BIP-39 seed), from the *same birthday*,
against the *same lightwalletd*, so the only variable is the engine. The harness:

  1. builds both release binaries (unless --skip-build),
  2. derives the Slipstream UFVK from the seed (so you supply ONE seed),
  3. runs `slipstream bench` -> reads `total_s` from its BenchSummary JSON,
  4. starts `zkool_graphql`, restores the account, times the blocking
     `synchronizeAccount` mutation, reads the final balance, tears the server down,
  5. prints a side-by-side table (optionally writes the full result as JSON).

The two headline numbers are each engine's *sync pass*, excluding one-time
process/server startup and account insertion:
  - Slipstream: `total_s` from bench.json (engine-measured).
  - zkool: wall-clock around the `synchronizeAccount` GraphQL call.

This is NOT a correctness oracle. Slipstream has its own byte-identical golden
oracle; zkool runs a ZSA librustzcash fork. The balance/height columns are a
sanity cross-check that both engines found the same funds, nothing more.

Stdlib only — no pip installs. Requires `cargo` and, to run the Slipstream side,
a checkout of the Slipstream repo (default: next to this one).
"""

import argparse
import json
import os
import signal
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request
from dataclasses import dataclass, field, asdict
from pathlib import Path

# ── small utilities ────────────────────────────────────────────────────────────


def log(msg: str) -> None:
    print(f"[bench] {msg}", flush=True)


def die(msg: str) -> "None":
    print(f"[bench] error: {msg}", file=sys.stderr, flush=True)
    sys.exit(1)


def run(cmd: list[str], cwd: Path | None = None, env: dict | None = None) -> str:
    """Run a command to completion, streaming nothing, returning stdout. Dies on failure."""
    log("$ " + " ".join(cmd) + (f"   (cwd={cwd})" if cwd else ""))
    proc = subprocess.run(
        cmd, cwd=cwd, env=env, capture_output=True, text=True
    )
    if proc.returncode != 0:
        sys.stderr.write(proc.stdout)
        sys.stderr.write(proc.stderr)
        die(f"command failed ({proc.returncode}): {' '.join(cmd)}")
    return proc.stdout


def free_port(preferred: int) -> int:
    """Return `preferred` if bindable, else an ephemeral free port."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        try:
            s.bind(("127.0.0.1", preferred))
            return preferred
        except OSError:
            s.bind(("127.0.0.1", 0))
            return s.getsockname()[1]


def normalize_server(server: str) -> str:
    """Slipstream's parse_server requires scheme://host:port. Fill in the default port."""
    if "://" not in server:
        die(f"--server must start with http:// or https:// (got {server!r})")
    scheme, rest = server.split("://", 1)
    host = rest.rstrip("/")
    if ":" not in host.split("/")[0]:
        host = host + (":443" if scheme == "https" else ":9067")
    return f"{scheme}://{host}"


# ── result records ─────────────────────────────────────────────────────────────


@dataclass
class EngineRun:
    engine: str
    sync_s: float | None = None          # the headline sync-pass wall time
    wall_s: float | None = None          # full wall-clock incl. process start (context)
    synced_height: int | None = None
    blocks: int | None = None
    balance_zat: int | None = None       # total balance, if the engine reports one
    extra: dict = field(default_factory=dict)
    error: str | None = None


# ── Slipstream side ────────────────────────────────────────────────────────────


def slipstream_bin(repo: Path) -> Path:
    return repo / "target" / "release" / "slipstream"


def build_slipstream(repo: Path, toolchain: str | None) -> None:
    log("building slipstream-cli (release) …")
    cargo = ["cargo"] + ([f"+{toolchain}"] if toolchain else [])
    run(cargo + ["build", "--release", "-p", "slipstream-cli"], cwd=repo)


def derive_ufvk(repo: Path, seed: str, network: str) -> str:
    out = run(
        [str(slipstream_bin(repo)), "derive-ufvk", "--mnemonic", seed, "--network", network]
    )
    for line in out.splitlines():
        line = line.strip()
        if line:
            return line
    die("derive-ufvk produced no UFVK on stdout")


def bench_slipstream(
    repo: Path, network: str, server: str, ufvk: str, birthday: int, workdir: Path
) -> EngineRun:
    r = EngineRun(engine="slipstream")
    json_path = workdir / "slipstream_bench.json"
    cmd = [
        str(slipstream_bin(repo)), "bench",
        "--network", network,
        "--server", server,
        "--ufvk", ufvk,
        "--birthday", str(birthday),
        "--json", str(json_path),
    ]
    log("slipstream: fresh-restore bench …")
    t0 = time.monotonic()
    proc = subprocess.run(cmd, cwd=repo, capture_output=True, text=True)
    r.wall_s = time.monotonic() - t0
    if proc.returncode != 0:
        sys.stderr.write(proc.stdout)
        sys.stderr.write(proc.stderr)
        r.error = f"slipstream bench exited {proc.returncode}"
        return r
    try:
        summary = json.loads(json_path.read_text())
    except (OSError, json.JSONDecodeError) as e:
        r.error = f"could not read bench.json: {e}"
        return r
    r.sync_s = summary.get("total_s")
    r.blocks = summary.get("blocks")
    r.extra = {
        "fetch_s": summary.get("fetch_s"),
        "scan_s": summary.get("scan_s"),
        "enhance_s": summary.get("enhance_s"),
        "wire_mbps_avg": summary.get("wire_mbps_avg"),
        "engine_build": summary.get("engine_build"),
    }
    return r


# ── zkool side ─────────────────────────────────────────────────────────────────


def zkool_bin(repo: Path) -> Path:
    return repo / "target" / "release" / "zkool_graphql"


def build_zkool(repo: Path, bundled_params: bool) -> None:
    feats = "graphql,bundled-sapling-params" if bundled_params else "graphql"
    log(f"building zkool_graphql (release, --features {feats}) … (first build is slow)")
    run(
        ["cargo", "build", "--release", "--bin", "zkool_graphql", "--features", feats],
        cwd=repo,
    )


class Graphql:
    """Minimal GraphQL-over-HTTP client (POST /graphql)."""

    def __init__(self, url: str):
        self.url = url

    def call(self, query: str, variables: dict | None = None, timeout: float = 86400.0):
        body = json.dumps({"query": query, "variables": variables or {}}).encode()
        req = urllib.request.Request(
            self.url, data=body, headers={"Content-Type": "application/json"}
        )
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            payload = json.loads(resp.read().decode())
        if payload.get("errors"):
            raise RuntimeError(f"graphql errors: {payload['errors']}")
        return payload["data"]


def wait_until_ready(gql: Graphql, deadline_s: float) -> None:
    """Poll `currentHeight` until the server answers (covers Sapling-param download)."""
    start = time.monotonic()
    last_err = None
    while time.monotonic() - start < deadline_s:
        try:
            data = gql.call("{ currentHeight }", timeout=10.0)
            log(f"zkool: server ready (currentHeight={data.get('currentHeight')})")
            return
        except (urllib.error.URLError, OSError, RuntimeError) as e:
            last_err = e
            time.sleep(1.0)
    die(f"zkool server not ready after {deadline_s:.0f}s (last: {last_err})")


def start_zkool_server(
    repo: Path, db_path: Path, lwd_url: str, port: int, coin: int, log_file: Path
) -> subprocess.Popen:
    cmd = [
        str(zkool_bin(repo)),
        "--db-path", str(db_path),
        "--lwd-url", lwd_url,
        "--port", str(port),
        "--coin", str(coin),
        "--allow-unauthenticated",
        "--allow-unencrypted-database",
        "--no-mempool",
    ]
    log("$ " + " ".join(cmd))
    fh = open(log_file, "w")
    # start_new_session so we can signal the whole process group on teardown.
    # The download and scan lanes report at debug level; ask for them unless
    # the caller chose a log filter of their own.
    env = dict(os.environ)
    env.setdefault(
        "RUST_LOG",
        "info,rlz::sync::fetch=debug,rlz::warp::sync=debug,rlz::sync::shielded=debug",
    )
    return subprocess.Popen(
        cmd, cwd=repo, env=env, stdout=fh, stderr=subprocess.STDOUT, start_new_session=True
    )


def stop_server(proc: subprocess.Popen) -> None:
    if proc.poll() is not None:
        return
    try:
        os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        proc.wait(timeout=10)
    except subprocess.TimeoutExpired:
        os.killpg(os.getpgid(proc.pid), signal.SIGKILL)


def parse_zkool_lanes(log_path: Path) -> dict:
    """Extract the instrumented per-lane timings from the zkool server log.

    The patched engine logs two INFO lines on sync completion — one for the scan
    lane (busy=decrypt+tree, wait=starved waiting for downloaded blocks) and one
    for the write lane (busy=SQLite commit). Field order is formatter-dependent,
    so we scrape keys by regex wherever they appear.
    """
    import re
    keys = ("download_wait_ms", "reader_wall_ms", "scan_busy_ms", "scan_wait_ms",
            "scan_chunks", "write_busy_ms", "write_commits",
            # the multi-connection download's own summary
            "connections", "peak_connections", "pieces", "cuts", "timeouts",
            "steals", "replaced", "budget_raises", "max_buffered_bytes")
    out: dict = {}
    try:
        text = log_path.read_text(errors="replace")
    except OSError:
        return out
    for k in keys:
        m = re.search(rf"\b{k}=(\d+)", text)
        if m:
            out[k] = int(m.group(1))
    return out


def zec_to_zat(value) -> int | None:
    """zkool reports balances as decimal-ZEC strings; convert to integer zatoshis."""
    if value is None:
        return None
    try:
        return round(float(value) * 100_000_000)
    except (TypeError, ValueError):
        return None


def bench_zkool_over_graphql(
    gql: Graphql, seed: str, birthday: int, pools: int, fast: bool
) -> EngineRun:
    r = EngineRun(engine="zkool")

    log("zkool: createAccount (restore from seed) …")
    acct = gql.call(
        "mutation($new: NewAccount!) { createAccount(newAccount: $new) }",
        {
            "new": {
                "name": "bench",
                "key": seed,
                "passphrase": "",
                "aindex": 0,
                "birth": birthday,
                "pools": pools,
                "useInternal": False,
            }
        },
    )
    account_id = acct["createAccount"]
    log(f"zkool: account id = {account_id}")

    log(f"zkool: synchronizeAccount(fast={fast}) — timing the blocking sync …")
    t0 = time.monotonic()
    synced = gql.call(
        "mutation($id: Int!, $fast: Boolean) { synchronizeAccount(idAccount: $id, fast: $fast) }",
        {"id": account_id, "fast": fast},
    )
    r.sync_s = time.monotonic() - t0
    r.wall_s = r.sync_s
    r.synced_height = synced["synchronizeAccount"]

    try:
        bal = gql.call(
            "query($id: Int!) { balanceByAccount(idAccount: $id) "
            "{ height transparent sapling orchard total } }",
            {"id": account_id},
        )["balanceByAccount"]
        r.balance_zat = zec_to_zat(bal.get("total"))
        r.extra = {"balance": bal}
    except RuntimeError as e:
        r.extra = {"balance_error": str(e)}
    return r


# ── reporting ──────────────────────────────────────────────────────────────────


def fmt_dur(seconds: float | None) -> str:
    if seconds is None:
        return "—"
    m, s = divmod(seconds, 60)
    return f"{int(m)}:{s:05.2f}" if m else f"{s:.2f}s"


def fmt_zec(zat: int | None) -> str:
    if zat is None:
        return "—"
    sign = "-" if zat < 0 else ""
    a = abs(zat)
    return f"{sign}{a // 100_000_000}.{a % 100_000_000:08d}"


def print_report(runs: list[EngineRun]) -> None:
    by = {r.engine: r for r in runs}
    print()
    print("=" * 68)
    print("  Slipstream vs. zkool — sync benchmark")
    print("=" * 68)
    header = f"  {'engine':<12}{'sync pass':>14}{'synced height':>16}{'balance (ZEC)':>18}"
    print(header)
    print("  " + "-" * 64)
    for name in ("slipstream", "zkool"):
        r = by.get(name)
        if not r:
            continue
        if r.error:
            print(f"  {name:<12}{'FAILED: ' + r.error}")
            continue
        print(
            f"  {name:<12}{fmt_dur(r.sync_s):>14}"
            f"{(str(r.synced_height) if r.synced_height is not None else '—'):>16}"
            f"{fmt_zec(r.balance_zat):>18}"
        )
    slip, zk = by.get("slipstream"), by.get("zkool")
    if slip and zk and slip.sync_s and zk.sync_s and not slip.error and not zk.error:
        faster, slower = sorted([slip, zk], key=lambda r: r.sync_s)
        ratio = slower.sync_s / faster.sync_s
        print("  " + "-" * 64)
        print(f"  => {faster.engine} is {ratio:.2f}× faster "
              f"({fmt_dur(faster.sync_s)} vs {fmt_dur(slower.sync_s)})")
    if slip and not slip.error and slip.extra:
        e = slip.extra
        print(f"  slipstream split: fetch {fmt_dur(e.get('fetch_s'))} | "
              f"scan {fmt_dur(e.get('scan_s'))} | enhance {fmt_dur(e.get('enhance_s'))}"
              f"  ({e.get('wire_mbps_avg', 0) or 0:.1f} MB/s avg)")
    if zk and not zk.error and zk.extra.get("lanes"):
        ln = zk.extra["lanes"]
        sb, sw = ln.get("scan_busy_ms"), ln.get("scan_wait_ms")
        wb = ln.get("write_busy_ms")
        def _s(ms):
            return fmt_dur(ms / 1000.0) if ms is not None else "—"
        dl = ln.get("download_wait_ms")
        print(f"  zkool split:      download {_s(dl)} | scan-busy {_s(sb)} | "
              f"scan-wait {_s(sw)} | db-commit {_s(wb)}")
        if sb is not None and sw is not None:
            verdict = "download-bound" if sw > sb else "compute-bound"
            print(f"  zkool is {verdict} (scan-wait {_s(sw)} vs scan-busy {_s(sb)})")
        if ln.get("pieces") is not None:
            mb = (ln.get("max_buffered_bytes") or 0) / 1048576
            print(f"  zkool download:   {ln.get('peak_connections')} streams at peak, "
                  f"{ln.get('connections')} opened | {ln.get('pieces')} slices, "
                  f"{ln.get('cuts')} over bytes, {ln.get('timeouts')} over time, "
                  f"{ln.get('steals')} moved | {ln.get('replaced')} replaced | "
                  f"{mb:.0f} MB buffered at peak")
    print("=" * 68)
    print()


# ── main ───────────────────────────────────────────────────────────────────────


COIN_BY_NETWORK = {"main": 0, "test": 1}


def main() -> None:
    here = Path(__file__).resolve()
    zkool_root = here.parents[2]  # tools/bench/ -> repo root

    p = argparse.ArgumentParser(
        description="Benchmark Slipstream against the zkool sync engine on one wallet.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    p.add_argument("--seed", required=True, help="BIP-39 mnemonic for the wallet to sync (both engines).")
    p.add_argument("--birthday", type=int, required=True, help="Wallet birthday height.")
    p.add_argument("--server", required=True,
                   help="lightwalletd URL for BOTH engines, e.g. https://zec.rocks:443")
    p.add_argument("--network", choices=["main", "test"], default="main")
    p.add_argument("--engines", choices=["both", "slipstream", "zkool"], default="both")

    p.add_argument("--slipstream-repo", type=Path, default=zkool_root.parent / "slipstream",
                   help="Path to the Slipstream checkout.")
    p.add_argument("--zkool-repo", type=Path, default=zkool_root,
                   help="Path to the zkool checkout (default: this one).")
    p.add_argument("--zkool-url", default=None,
                   help="Use an already-running zkool GraphQL endpoint (skips build+spawn), "
                        "e.g. http://127.0.0.1:8000/graphql")
    p.add_argument("--zkool-lwd-url", default=None,
                   help="Override the lightwalletd URL passed to zkool (default: derived from --server).")
    p.add_argument("--zkool-port", type=int, default=8137)
    p.add_argument("--pools", type=int, default=7, help="zkool pool bitmask (1=t,2=s,4=o; 7=all).")
    p.add_argument("--fast", action="store_true",
                   help="zkool: skip the memo/detail pass (synchronizeAccount fast=true).")
    p.add_argument("--bundled-params", action="store_true",
                   help="Build zkool with bundled Sapling params (no runtime download).")
    p.add_argument("--ready-timeout", type=float, default=900.0,
                   help="Seconds to wait for the zkool server (covers Sapling-param download).")

    p.add_argument("--cargo-toolchain", default=None,
                   help="rustup toolchain for the Slipstream build (e.g. 1.99.0), when the default "
                        "toolchain is older than the repo's 1.90 minimum. zkool uses its own pin.")
    p.add_argument("--skip-build", action="store_true", help="Assume both binaries are already built.")
    p.add_argument("--workdir", type=Path, default=None,
                   help="Directory for temp wallets/logs (default: a fresh temp dir, kept).")
    p.add_argument("--json", type=Path, default=None, help="Write the full result record here.")
    args = p.parse_args()

    server = normalize_server(args.server)
    slip_repo = args.slipstream_repo.resolve()
    do_slip = args.engines in ("both", "slipstream")
    do_zkool = args.engines in ("both", "zkool")

    if args.workdir:
        workdir = args.workdir.resolve()
        workdir.mkdir(parents=True, exist_ok=True)
    else:
        import tempfile
        workdir = Path(tempfile.mkdtemp(prefix="slipstream-bench-"))
    log(f"workdir: {workdir}")

    runs: list[EngineRun] = []

    # ── Slipstream ──
    if do_slip:
        if not args.skip_build:
            build_slipstream(slip_repo, args.cargo_toolchain)
        if not slipstream_bin(slip_repo).exists():
            die(f"slipstream binary not found at {slipstream_bin(slip_repo)} (build it or drop --skip-build)")
        ufvk = derive_ufvk(slip_repo, args.seed, args.network)
        log(f"slipstream UFVK: {ufvk[:24]}… (birthday {args.birthday})")
        runs.append(bench_slipstream(slip_repo, args.network, server, ufvk, args.birthday, workdir))

    # ── zkool ──
    if do_zkool:
        if args.zkool_url:
            gql = Graphql(args.zkool_url)
            log(f"zkool: using existing endpoint {args.zkool_url}")
            wait_until_ready(gql, 30.0)
            runs.append(bench_zkool_over_graphql(gql, args.seed, args.birthday, args.pools, args.fast))
        else:
            zkool_repo = args.zkool_repo.resolve()
            if not args.skip_build:
                if not zkool_repo.exists():
                    die(f"zkool repo not found at {zkool_repo} (pass --zkool-repo or --zkool-url)")
                build_zkool(zkool_repo, args.bundled_params)
            if not zkool_bin(zkool_repo).exists():
                die(f"zkool_graphql not found at {zkool_bin(zkool_repo)} (build it or pass --zkool-url)")
            port = free_port(args.zkool_port)
            lwd = args.zkool_lwd_url or server
            db_path = workdir / "zkool.db"
            server_log = workdir / "zkool_server.log"
            proc = start_zkool_server(
                zkool_repo, db_path, lwd, port, COIN_BY_NETWORK[args.network], server_log
            )
            gql = Graphql(f"http://127.0.0.1:{port}/graphql")
            try:
                wait_until_ready(gql, args.ready_timeout)
                zr = bench_zkool_over_graphql(gql, args.seed, args.birthday, args.pools, args.fast)
                runs.append(zr)
            finally:
                log("zkool: stopping server")
                stop_server(proc)
                log(f"zkool: server log at {server_log}")
            lanes = parse_zkool_lanes(server_log)
            if lanes:
                zr.extra["lanes"] = lanes

    print_report(runs)

    if args.json:
        args.json.write_text(json.dumps({
            "config": {
                "seed_fingerprint": None,  # seed intentionally not persisted
                "birthday": args.birthday,
                "server": server,
                "network": args.network,
                "fast": args.fast,
            },
            "runs": [asdict(r) for r in runs],
        }, indent=2))
        log(f"wrote {args.json}")

    if any(r.error for r in runs):
        sys.exit(2)


if __name__ == "__main__":
    main()
