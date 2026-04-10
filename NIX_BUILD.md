# Nix Build Instructions for zkool_graphql

This directory contains Nix build scripts for building `zkool_graphql`.

## Platform Support

✅ **macOS** - Fully supported
✅ **Linux** - Fully supported

## Quick Start

**Recommended: Use `shell.nix`**
```bash
nix-shell --run 'cd rust && cargo build --release --bin zkool_graphql --features=graphql'
```

The binary will be at `rust/target/release/zkool_graphql`.

## Alternative: Interactive Development Shell

```bash
nix-shell
# Then build manually:
cd rust
cargo build --release --bin zkool_graphql --features=graphql
```

## Running the Binary

```bash
./rust/target/release/zkool_graphql -d zkool.db -p 8000 -l http://localhost:8137
```

## Why shell.nix instead of nix-build?

This project has git dependencies that don't work well with Nix's strict sandboxing. Using `nix-shell` provides:
- ✅ Same Nix-managed dependencies
- ✅ Works with git dependencies
- ✅ Real-time build progress
- ✅ No sandboxing issues

## Dependencies

Nix automatically provides:
- Rust toolchain (rustc, cargo)
- pkg-config
- OpenSSL
- SQLite
- Platform-specific dependencies (udev on Linux)

## Features

The build includes the `graphql` feature which enables:
- Juniper GraphQL server
- Warp web framework with WebSocket support
- JWT authentication
- SQLite database support
