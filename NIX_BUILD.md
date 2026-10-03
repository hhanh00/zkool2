# Nix Build Instructions

The flake provides a reproducible development environment for the whole
project (Flutter app, Rust `rlz` core via `flutter_rust_bridge`/`cargokit`, the
`zkool_graphql` server, and the Python test suite) plus hermetic packages for
the Linux GUI (`zkool-pure`) and the GraphQL server (`zkool-graphql`).

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
./result/bin/zkool_graphql -d zkool.db -p 8000 -l http://localhost:8137 \
  --db-password-file /path/to/db-password \
  --jwt-public-key-file /path/to/jwt-public.pem
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

### `cargo-vendor`

`nix build .#cargo-vendor` produces the vendored source tree used by the offline
shell. It is a fixed-output derivation generated from `Cargo.lock`; if the Git
dependencies change, refresh `outputHash` with the hash Nix reports on failure.

## The GUI app

### Build the GUI — `zkool-pure` (x86_64-linux)

`zkool-pure` is a normal derivation that builds the whole GUI with
`sandbox = true` and **no network**, so `nix build .#zkool-pure` works like any
other Nix package (and is cacheable). It makes cargokit sandbox-safe by
pre-vendoring every fetch:

- app + cargokit `build_tool` pub deps -> `packages.zkool-pub-cache` (a
  normalized, reproducible FOD; the git checkout's volatile `.git`/`hooks` and
  pub's version-listing cache are stripped so the output is path-independent),
- `run_build_tool.sh` is patched to `pub get --offline`,
- cargo uses `packages.cargo-vendor` with `CARGO_NET_OFFLINE=true`,
- rustup calls are served by the `rustup` shim,
- engine artifacts come from `pkgs.flutter`.

```bash
nix build .#zkool-pure
./result/bin/zkool

# Optional: install the GUI into your profile.
nix profile install .#zkool-pure
zkool
```

Dependency fetching can require network access on the first build; the GUI
compilation itself runs offline in the sandbox. Updating pub/cargo dependencies
means refreshing the fixed-output derivation (FOD) hashes in `flake.nix`.

The installed launcher sets the library path for `librlz.so` and adds
`xdg-user-dir` to `PATH`. Use `./result/bin/zkool` to launch the packaged app.
The package still depends on its Nix store closure; see portability below.

### Alternative: import a development-shell build — `zkool-store`

For a development-shell build, this helper builds the release GUI and imports
the bundle into the store:

```bash
nix run .#zkool-store      # builds --release, prints the /nix/store path
```

The result is a store path, but it is not a reproducible derivation. Use
`zkool-pure` for sandboxed builds and binary-cache distribution.

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

The flake supports both GUI and server development and packaging. Distribute
Nix-built packages through a binary cache or copy their full closure to another
Nix machine.

## Offline / hermetic cargo

```bash
nix develop .#offline
cargo build -p rlz --offline
```

`.#offline` points cargo at `packages.cargo-vendor` via a generated `CARGO_HOME`
config and sets `CARGO_NET_OFFLINE=true`.

## Release hash maintenance

The release-please workflow refreshes the GUI Cargo vendor hash, the server
Cargo dependency hash, and the pub cache hash on the release PR before pushing
its updated commit. It forces fresh dependency fetches with placeholder hashes,
then verifies the resulting hashes. No manual hash edits are needed for normal
releases. If fetching fails, the workflow fails without pushing partial updates;
resolve the failure and rerun it before merging the release PR.

To run the same refresh locally on x86_64 Linux with Nix installed:

```bash
python3 scripts/update_nix_hashes.py
```

The release-tag Nix workflow then builds and caches the GUI and server. Use
`zkool-graphql` with Cachix to download the server without compiling it; there
is no separate package that downloads a GitHub release binary.

## Binary cache (Cachix)

`.github/workflows/nix-cache.yml` builds `zkool-graphql` and `zkool-pure` on
stable release tags (`zkool-v*`, excluding `-rc` tags) and on manual dispatch,
then pushes them, with their closure, to the public Cachix cache `zkool`.

One-time setup:

- Create the cache `zkool` at https://app.cachix.org (free for public caches).
- Add a repository secret `CACHIX_AUTH_TOKEN` holding a write token for it.

Consumers then substitute instead of building:

```bash
nix build github:hhanh00/zkool2#zkool-pure   # downloads
```

once they trust the cache (`cachix use zkool`, or in `~/.config/nix/nix.conf`):

```
extra-substituters = https://zkool.cachix.org
extra-trusted-public-keys = zkool.cachix.org-1:<public key>
```

The public key is shown in the Cachix dashboard / by `cachix use zkool`. Adding
it to the flake's `nixConfig` would apply it automatically, at the cost of an
interactive trust prompt on first use.

Note: only the *runtime* closure of `zkool-pure` is pushed, so a consumer whose
`flake.lock` differs (different store paths) will rebuild and re-fetch the
pub/cargo FODs; pushing those too (`nix build .#zkool-pub-cache .#cargo-vendor`)
avoids that.

## Distribution

A `/nix/store` path is only usable by a machine with Nix and that exact closure.

- Share the flake: `nix run github:you/zkool2#zkool-graphql`.
- Binary cache: `cachix push <name> $(nix build .#zkool-graphql --print-out-paths)`,
  or `nix copy --to s3://...` / `--to ssh-ng://host`; consumers add the cache as
  a substituter.
- Air-gapped: `nix copy --to file:///mnt/usb ...` then `--from`.
- Non-Nix Linux: `nix bundle --bundler github:NixOS/bundlers#toArx .#zkool-graphql`.

## Platforms

The flake defines outputs for `x86_64-linux`, `aarch64-linux`, and
`aarch64-darwin`. The GUI packaging currently uses the Flutter
`build/linux/x64/release/bundle` path, so use `x86_64-linux` for `zkool-pure`
and `zkool-store`; the cache workflow builds on x86_64 Linux. The macOS
development shell is provided but untested. `x86_64-darwin` is excluded.
