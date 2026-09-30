# Nix Build Instructions

The flake provides a reproducible development environment for the whole
project (Flutter app, Rust `rlz` core via `flutter_rust_bridge`/`cargokit`, the
`zkool_graphql` server, and the Python test suite) plus a hermetic package for
the GraphQL server.

## Prerequisites

Flakes must be enabled. Add this to `/etc/nix/nix.conf` or
`~/.config/nix/nix.conf`:

```
experimental-features = nix-command flakes
```

## Development shell

```bash
nix develop          # Rust 1.95.0 + Flutter 3.47.4 + native deps
```

| Tool                          | Version | Notes                                        |
| ----------------------------- | ------- | -------------------------------------------- |
| `rustc` / `cargo`             | 1.95.0  | matches `rust-toolchain.toml` (1.88-1.94 fail) |
| `flutter` / Dart              | 3.47.4 / 3.13.3 | matches `.github/workflows/build.yml` |
| `flutter_rust_bridge_codegen` | 2.12.0  | matches the `=2.12.0` crate/Dart pin         |
| `uv` / `python3`              |         | for `tests/`                                 |

Plus `cmake`, `ninja`, `clang`, `pkg-config`, GTK 3, WebKitGTK, OpenSSL, SQLite
and `libudev` (the Linux desktop toolchain, mirroring
`.github/actions/linux/action.yml`).

### Build in the shell

```bash
flutter build linux --debug        # -> build/linux/x64/debug/bundle/zkool
flutter build linux --release
cargo build -p rlz                 # Rust core
```

`flutter pub get`, `cargo fetch` and `flutter build linux` work out of the box.

### cargokit and the rustup shim

`cargokit` (`rust_builder/cargokit`) hard-codes `rustup run stable cargo build`
and inspects `rustup toolchain list` / `rustup target list`. Since the compiler
here comes from Nix (not rustup), the shell puts a small **`rustup` shim** on
`PATH` that forwards those calls to the Nix toolchain. It only handles the
subcommands cargokit issues, so it is **not** a general rustup and only supports
host/native builds.

## Packages

### `zkool-graphql` (hermetic)

Server binary, mirroring `.github/workflows/build-graphql.yml`:

```bash
nix build .#zkool-graphql
./result/bin/zkool_graphql -d zkool.db -p 8000 -l http://localhost:8137
```

Built with the vendored cargo sources (`cargoHash`) so it needs no network.

Consumers can also get it as `pkgs.zkool-graphql` via the exported overlay:

```nix
# in another flake
nixpkgs.overlays = [ inputs.zkool2.overlays.default ];
environment.systemPackages = [ pkgs.zkool-graphql ];
```

or by reference, without an overlay:
`inputs.zkool2.packages.${pkgs.system}.zkool-graphql`.

### `zkool-graphql-bin` (prebuilt, x86_64-linux)

If you do not want to compile Rust, this downloads the `zkool_graphql` binary
that CI attaches to the GitHub release and makes it runnable on NixOS
(`autoPatchelfHook`):

```bash
nix build .#zkool-graphql-bin
./result/bin/zkool_graphql --help
nix profile install .#zkool-graphql-bin   # into your profile
```

The release is produced by `.github/workflows/build-graphql.yml`; bump `version`
in the URL and the `hash` when a new tag is published.

### `cargo-vendor`

`nix build .#cargo-vendor` produces the vendored source tree used by the offline
shell. It is a fixed-output derivation generated from `Cargo.lock`; if the Git
dependencies change, refresh `outputHash` with the hash Nix reports on failure.

## The GUI app

A pure `nix build` of the GUI app is not possible in this setup:

- Flutter downloads engine/pub artifacts and cargokit shells out to cargo, so
  the build needs network. `__noChroot`/`--option sandbox false` require a
  trusted user, which a normal user is not.
- A fixed-output derivation is not an option either: Nix 2.34 rejects store-path
  references in FOD outputs, and the bundle links GTK/WebKit/etc. from the store.

Instead, build it in the dev shell and import the result into the store:

```bash
nix run .#zkool-store      # builds --release, prints the /nix/store path
```

This is equivalent to:

```bash
nix develop -c bash -c '
  flutter build linux --release
  nix store add-path --name zkool-6.31.0 build/linux/x64/release/bundle
'
```

The result is a real store path (references recorded), but it is not a
reproducible derivation.

### Portability: why a Nix-built GUI crashes on other distros

A binary produced through the Nix toolchain is hard-wired to `/nix/store`, so
it only runs on a machine that has that same closure. For the GUI bundle this
is not a GTK ABI problem (even though the missing libraries are GTK), it is the
Nix store:

```
$ readelf -l  build/linux/x64/release/bundle/zkool | grep interpreter
  [Requesting program interpreter: /nix/store/...-glibc-2.42-84/lib/ld-linux-x86-64.so.2]

$ readelf -d  build/linux/x64/release/bundle/zkool | grep -i runpath
  Library runpath: [/nix/store/...-gtk+3-3.24.52/lib:/nix/store/...-pango.../lib:...:$ORIGIN/lib]
```

On another distro the kernel cannot find the Nix `ld-linux` interpreter, so the
process dies at startup (`No such file or directory`) before GTK is loaded.
Copying the bundle to another machine without `/nix/store` therefore cannot
work.

For other distros, use the artifacts CI builds on Ubuntu (or build them
locally on a normal distro):

- **`.AppImage`** — bundles its own GTK and dependencies; portable single file
  (needs FUSE, or run with `--appimage-extract-and-run`).
- **`.deb`** — Debian/Ubuntu.
- **`.flatpak`** — uses the `org.freedesktop.Platform//25.08` runtime, which
  supplies its own GTK/WebKit; the most robust option. Build locally with
  `flatpak-builder --force-clean flatpak-build flatpak/cc.methyl.Zkool.yml`.

See `.github/actions/linux/action.yml` (via `fastforge`) for the CI pipeline.

The Nix flake is intended as a **development environment** for the GUI, and for
producing the server (`zkool-graphql` / `zkool-graphql-bin`), which is a single
static-ish binary and *is* portable across Nix machines.

## Offline / hermetic cargo

```bash
nix develop .#offline
cargo build -p rlz --offline
```

`.#offline` points cargo at `packages.cargo-vendor` via a generated `CARGO_HOME`
config and sets `CARGO_NET_OFFLINE=true`.

## Distribution

A `/nix/store` path is only usable by a machine with Nix and that exact closure.

- Share the flake: `nix run github:you/zkool2#zkool-graphql`.
- Binary cache: `cachix push <name> $(nix build .#zkool-graphql --print-out-paths)`,
  or `nix copy --to s3://...` / `--to ssh-ng://host`; consumers add the cache as
  a substituter.
- Air-gapped: `nix copy --to file:///mnt/usb ...` then `--from`.
- Non-Nix Linux: `nix bundle --bundler github:NixOS/bundlers#toArx .#zkool-graphql`.

## Platforms

The dev shell is defined for all default systems (`flake-utils`). Linux is
fully exercised; the macOS shell is provided but untested.
