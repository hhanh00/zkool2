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

:::important
You MUST have a configuration file `zkool.toml`.
:::

It needs to specify the path to the database file (which will be automatically
created) and the URL to the LightwalletD server.

```toml
db_path = "zkool.db"
lwd_url = "https://zec.rocks"
```
