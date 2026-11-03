---
title: Build
---

`zkool_graphql` is *only* release from source. As a 100% Rust app, it is easy to
build for many architectures and therefore, we don't provide pre-built binaries.

## Clone from github

```sh
git clone https://github.com/hhanh00/zkool2
```

## Build

Requirement: Install [Rust](https://rust-lang.org/)

Then build with `cargo`.

```sh
cd rust
cargo b -r --features=graphql --bin zkool_graphql
```

This creates the executable `zkool_graphql` in the `target/release` directory.

## Configuration

The config file needs to specify the path to the database file (which will be
automatically created), the URL to the LightwalletD server and the listening port.

```toml
db_path = "zkool.db"
lwd_url = "https://zec.rocks"
port = 8000
```

Alternatively, you can specify another config file or set the values on the
command line.

```text
Usage: zkool_graphql [OPTIONS]

Options:
  -c, --config-path <CONFIG_PATH>
  -d, --db-path <DB_PATH>
  -l, --lwd-url <LWD_URL>
  -p, --port <PORT>
  -C, --coin <COIN>                Coin type for a database being created:
                                   0=mainnet, 1=testnet, 2=regtest, 3=ZSA regtest
  -h, --help                       Print help
```

## Using Testnet or Regtest

Pass `--coin` when the database is created: `1` for testnet, `2` for regtest,
`3` for the ZSA regtest. An existing database keeps the network it was created
on, whatever the flag says.

Without `--coin`, the file name decides for a new database: `testnet` in the
name means testnet, `regtest` means regtest, `zsa` the ZSA regtest, and
anything else mainnet.

:::important
You must also use a lightwalletd that connects to the proper network.
:::
