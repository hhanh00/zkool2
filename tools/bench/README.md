# Sync benchmark: zkool vs. Slipstream

`compare_sync.py` syncs the same wallet (one seed), from the same birthday,
against the same lightwalletd, with zkool and with the Slipstream sync engine,
and prints the two times side by side. It can also run either engine alone,
which is the usual way to measure a change to zkool's sync.

## What it measures

| Engine | Number | How |
|---|---|---|
| zkool | wall-clock around `synchronizeAccount` | the GraphQL mutation blocks until synced |
| Slipstream | `total_s` from its bench JSON | measured by the engine, via `slipstream bench` |

Both exclude process and server startup and the account insert. For zkool the
report also shows where the time went (download, scanner busy, scanner waiting,
database commit) and what the multi-connection download did, read from the
server log.

This measures speed only. The synced height and balance columns are a sanity
check that both engines saw the same wallet.

## Requirements

- `cargo`. zkool builds with the toolchain pinned in `rust-toolchain.toml`.
- Python 3.10 or later, standard library only.
- For the Slipstream side: a checkout of Slipstream, by default at
  `../slipstream`, and rustc 1.90 or later (`--cargo-toolchain` selects one).
- Network access to a lightwalletd.

## Usage

zkool alone, the usual case:

```sh
python3 tools/bench/compare_sync.py --engines zkool \
  --seed "word1 word2 … word24" --birthday 2271890 \
  --server https://zec.rocks:443
```

Both engines:

```sh
python3 tools/bench/compare_sync.py \
  --seed "word1 word2 … word24" --birthday 2271890 \
  --server https://zec.rocks:443
```

| Flag | Purpose |
|---|---|
| `--engines zkool\|slipstream\|both` | Which side to run (default `both`). |
| `--skip-build` | Use the release binaries already built. |
| `--slipstream-repo PATH` | Slipstream checkout (default `../slipstream`). |
| `--zkool-url URL` | Drive a zkool server that is already running. No log is read, so no lane split. |
| `--fast` | zkool: skip the memo and detail pass. |
| `--workdir DIR` | Where the temporary wallet and the server log go. |
| `--json OUT` | Write the full result record. |

The zkool server is started with `RUST_LOG` set so the download and scan lanes
log their summaries; set `RUST_LOG` yourself to override.

## Reading the result

Run on an idle machine and repeat: on Wi-Fi the same sync varied by 10–15%
between runs. An empty wallet measures download and trial decryption; a wallet
with notes adds work this does not show.

`rust/examples/dl_bench.rs` measures the download alone, over 1 to K
connections, with no scanning.
