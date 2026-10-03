{
  description = "zkool - Flutter Zcash wallet with a Rust (flutter_rust_bridge) core";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    # x86_64-darwin is excluded: nixpkgs 26.11 dropped support for it, and
    # eachDefaultSystem would otherwise make `nix flake show`/`check` fail.
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        inherit (pkgs) lib;

        zkoolVersion = lib.removeSuffix "\n" (builtins.readFile ./version.txt);

        # Flutter must never lag the CI SDK: parse FLUTTER_VERSION from
        # build.yml (CI clones that tag) and require nixpkgs to be at least as
        # new, forcing a `nix flake update nixpkgs` when CI bumps first.
        ciFlutterVersion =
          let
            lines = lib.splitString "\n"
              (builtins.readFile ./.github/workflows/build.yml);
            line = builtins.head (builtins.filter
              (lib.hasInfix "FLUTTER_VERSION:")
              lines);
            groups = builtins.match " *FLUTTER_VERSION: \"?([0-9]+[.][0-9]+[.][0-9]+)\"?.*" line;
          in
          if groups == null then
            throw "cannot parse FLUTTER_VERSION from .github/workflows/build.yml"
          else builtins.head groups;

        flutter =
          if lib.versionAtLeast pkgs.flutter.version ciFlutterVersion then pkgs.flutter else
          throw ''
            nixpkgs flutter (${pkgs.flutter.version}) is older than the CI
            flutter (${ciFlutterVersion} from .github/workflows/build.yml).
            Run: nix flake update nixpkgs
          '';

        # Only the Cargo workspace is needed to build the Rust side. Filtering
        # keeps edits to flake.nix/docs from changing the source hash (and
        # therefore needlessly rebuilding) the Rust packages.
        cargoSrc = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            ./rust
          ];
        };

        # Pub cache covering the app AND cargokit's Dart `build_tool` (which
        # run_build_tool.sh resolves at build time). Fetched once with network
        # (legal for a FOD: the output is only Dart/git sources, no store
        # references), then reused offline by the sandboxed app build.
        pubCache = pkgs.stdenv.mkDerivation {
          # FOD outputs are substituted from binary caches by store path, and
          # the path is keyed on outputHash — not on the inputs. Suffixing the
          # name with the pubspec.lock hash changes the path whenever the lock
          # changes, so a stale pinned outputHash can no longer be silently
          # satisfied by Cachix; the FOD rebuilds and fails loudly instead.
          name = "zkool-pub-cache-${
            builtins.substring 0 12 (builtins.hashFile "sha256" ./pubspec.lock)
          }";
          src = ./.;
          nativeBuildInputs = [ flutter pkgs.git pkgs.cacert ];
          SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          GIT_SSL_CAINFO = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          outputHashMode = "recursive";
          outputHashAlgo = "sha256";
          outputHash = "sha256-i3m8X78D0Sn7i+23YKiKAN8ZtTm7J9lVXr5bnIK/K6Q=";
          buildCommand = ''
            export HOME="$TMPDIR"
            export PUB_CACHE="$out"
            cp -r "$src" source
            chmod -R u+w source
            cd source
            flutter pub get --enforce-lockfile
            ( cd rust_builder/cargokit/build_tool && flutter pub get --enforce-lockfile )

            # Normalize for reproducibility: pub's HTTP version-listing cache
            # and the git checkouts' volatile .git metadata (config embeds the
            # absolute cache path, index/logs embed timestamps) must go. The
            # bare mirrors in git/cache stay; offline pub get still resolves.
            rm -rf "$out/hosted/pub.dev/.cache" "$out/log" "$out/_temp" "$out/active_roots"
            find "$out/git" -type d -name .git -prune -exec rm -rf {} + 2>/dev/null || true
            # git's sample hooks have Nix store shebangs (bash/perl) -> store refs.
            find "$out/git" -type d -name hooks -prune -exec rm -rf {} + 2>/dev/null || true
          '';
        };

        # Toolchain follows rust-toolchain.toml — the same file CI uses — so
        # a bump there cannot drift from the Nix build. History: 1.88-1.94
        # fail to compile libcrux-psq (transitive nym dep) with E0716; 1.95.0
        # builds cleanly.
        rustToolchain =
          let
            tc = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml)).toolchain;
          in
          (pkgs.rust-bin.stable.${tc.channel} or (throw ''
            rust-toolchain.toml pins rust ${tc.channel}, which the pinned
            rust-overlay does not provide. Run: nix flake update rust-overlay
          '')).default.override {
            extensions = (tc.components or [ ]) ++ [ "rust-src" ];
          };

        # flutter_rust_bridge_codegen is version-locked to the
        # `flutter_rust_bridge = "=2.12.0"` crate and the Dart package of the
        # same version. nixpkgs ships 2.13.0, which refuses to run against a
        # 2.12.0 project, so build the matching release from crates.io.
        flutterRustBridgeCodegen = pkgs.rustPlatform.buildRustPackage rec {
          pname = "flutter_rust_bridge_codegen";
          version = "2.12.0";
          src = pkgs.fetchCrate {
            inherit pname version;
            hash = "sha256-AtFei4dkyIEzHjgDppgexTYBQ4OrPIvXdQKYjvmhnGE=";
          };
          cargoHash = "sha256-vuT6dCSewJuRwO7ni9TKpLUmoav5lFuwUVHQlfd/00s=";
          nativeBuildInputs = with pkgs; [ pkg-config ];
          # The crate's unit tests inspect the ambient toolchain and RUST_LOG,
          # which does not hold inside the Nix build sandbox.
          doCheck = false;
          meta.mainProgram = "flutter_rust_bridge_codegen";
        };

        # cargokit (rust_builder/cargokit) drives the native Rust build through
        # `rustup run stable cargo build ...` and inspects `rustup toolchain
        # list` / `rustup target list` (builder.dart, rustup.dart). Upstream
        # assumes a rustup-managed toolchain; here the compiler comes from Nix,
        # so provide a rustup-compatible shim that forwards `cargo` to the Nix
        # toolchain. Only the subcommands cargokit actually issues are handled.
        rustTarget = pkgs.stdenv.hostPlatform.rust.rustcTarget;
        cargokitRustup = pkgs.writeShellScriptBin "rustup" ''
          set -eu
          cmd="''${1:-}"
          sub="''${2:-}"
          case "$cmd" in
            toolchain)
              if [ "$sub" = "list" ]; then
                echo "stable-${rustTarget} (default)"
              fi
              ;;
            target)
              if [ "$sub" = "list" ]; then
                echo "${rustTarget}"
              fi
              ;;
            component)
              ;;
            run)
              shift 2
              # cargokit issues a plain `cargo build`. Add --locked so a stale
              # vendored tree fails loudly instead of re-resolving offline
              # against whatever versions cargo-vendor happens to carry.
              if [ "''${1:-}" = cargo ] && [ "''${2:-}" = build ]; then
                shift 2
                set -- cargo build --locked "$@"
              fi
              exec "$@"
              ;;
          esac
          exit 0
        '';

        # Toolchain the Linux desktop build needs (mirrors
        # .github/actions/linux/action.yml) plus the GTK stack Flutter and the
        # desktop plugins link against.
        linuxNativeBuildInputs = with pkgs; [ pkg-config cmake ninja clang ];
        linuxBuildInputs = with pkgs; [
          gtk3
          glib
          xz
          udev
          openssl
          sqlite
          webkitgtk_4_1
          libGL
          libepoxy
        ];

        sharedBuildInputs = with pkgs; [
          flutter
          flutterRustBridgeCodegen
          cargokitRustup
          python3
          uv
          git
        ];

        buildInputs = sharedBuildInputs
          ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux linuxBuildInputs;
        nativeBuildInputs = [ rustToolchain ]
          ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux linuxNativeBuildInputs;

        # Pure (sandboxed, no-network) GUI build. All fetches are pre-vendored:
        # pub -> pubCache, cargo -> cargo-vendor, engine -> flutter (nixpkgs,
        # required to be at least the CI FLUTTER_VERSION).
        # cargokit is made sandbox-safe by (a) running its `dart pub get` with
        # --offline against the pinned cache and (b) serving its rustup calls
        # from cargokitRustup. Being a normal derivation, referencing
        # GTK/WebKit/etc. from the store is allowed (the FOD restriction does
        # not apply).
        zkoolPure = pkgs.stdenv.mkDerivation {
          pname = "zkool-pure";
          version = zkoolVersion;
          src = ./.;
          nativeBuildInputs = [
            rustToolchain
            flutter
            cargokitRustup
          ] ++ (with pkgs; [ pkg-config cmake ninja clang git perl which coreutils cacert makeWrapper ]);
          buildInputs = linuxBuildInputs;
          SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          GIT_SSL_CAINFO = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          buildCommand = ''
            cp -r "$src" source
            chmod -R u+w source
            cd source

            # cargokit's helpers use #!/usr/bin/env bash, absent in the sandbox.
            patchShebangs rust_builder/cargokit

            export HOME="$TMPDIR"

            # pub needs a writable cache (it writes active_roots/log/.cache),
            # so start from a copy of the pinned FOD.
            export PUB_CACHE="$TMPDIR/pub-cache"
            cp -r --no-preserve=mode "${pubCache}" "$PUB_CACHE"
            chmod -R u+w "$PUB_CACHE"

            # Offline cargo through the vendored sources.
            export CARGO_NET_OFFLINE=true
            export CARGO_HOME="$TMPDIR/cargo-home"
            mkdir -p "$CARGO_HOME"
            sed "s|directory = \"vendor\"|directory = \"${self.packages.${system}.cargo-vendor}/vendor\"|" \
              "${self.packages.${system}.cargo-vendor}/config.toml" > "$CARGO_HOME/config.toml"

            # cargokit's runner resolves build_tool deps with pub: make it
            # offline. --enforce-lockfile cannot be used here: the runner is
            # generated in a temp dir with only a pubspec.yaml (a path
            # dependency on build_tool) and no pubspec.lock. build_tool's own
            # lock is still enforced when pubCache is populated.
            substituteInPlace rust_builder/cargokit/run_build_tool.sh \
              --replace-fail 'pub get --no-precompile' 'pub get --offline --no-precompile'

            flutter pub get --offline --enforce-lockfile
            flutter build linux --release --no-pub

            mkdir -p "$out/libexec/zkool" "$out/bin"
            cp -r build/linux/x64/release/bundle/. "$out/libexec/zkool/"

            # Wrap rather than symlink. The bundle dlopen()s librlz.so by bare
            # soname from the Dart VM, so $out/libexec/zkool/lib has to be on
            # the loader path; a bare symlink in $out/bin does not provide that
            # and RustLib.init() fails with "Failed to load dynamic library
            # 'librlz.so'". path_provider_linux also shells out to xdg-user-dir,
            # which must be on PATH or main() throws
            # MissingPlatformDirectoryException.
            makeWrapper "$out/libexec/zkool/zkool" "$out/bin/zkool" \
              --prefix LD_LIBRARY_PATH : "$out/libexec/zkool/lib" \
              --prefix PATH : "${lib.makeBinPath [ pkgs.xdg-user-dirs ]}"
          '';
        };

        # Alternative to zkool-pure: build in the development shell and
        # import the bundle with `nix store add-path`. See NIX_BUILD.md.
        zkoolStore = pkgs.writeShellScriptBin "zkool-store" ''
          set -euo pipefail
          root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
          cd "$root"
          nix develop "$root#default" -c bash -c '
            set -euo pipefail
            flutter build linux --release
            nix store add-path --name "zkool-${zkoolVersion}" \
              build/linux/x64/release/bundle
          '
        '';

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          RUST_BACKTRACE = "1";
          # Keep the pub cache and Flutter's internal state writable and out of
          # the read-only Nix store.
          PUB_CACHE = "$PWD/.dart_tool/pub-cache";

          shellHook = ''
            export PATH="$PWD/.dart_tool/pub-cache/bin:$PATH"
            echo "zkool dev shell"
            echo "  rustc    $(rustc --version)"
            echo "  flutter  $(flutter --version 2>/dev/null | head -1)"
          '';
        };

        # Hermetic/offline cargo sources, generated from Cargo.lock. Use
        # `nix develop .#offline` when the network or Nix sandbox blocks the git
        # dependencies.
        devShells.offline = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;
          RUST_BACKTRACE = "1";
          shellHook = ''
            VENDOR="${self.packages.${system}.cargo-vendor}"
            export CARGO_HOME="''${XDG_CACHE_HOME:-$HOME/.cache}/zkool-cargo-offline"
            mkdir -p "$CARGO_HOME"
            sed "s|directory = \"vendor\"|directory = \"$VENDOR/vendor\"|" \
              "$VENDOR/config.toml" > "$CARGO_HOME/config.toml"
            export CARGO_NET_OFFLINE=true
            echo "zkool offline dev shell (vendored cargo sources in $VENDOR)"
          '';
        };

        packages = {
          cargo-vendor = pkgs.stdenv.mkDerivation {
            # Cargo.lock hash in the name — same rationale as pubCache: a lock
            # change must not be masked by a Cachix hit on the old store path.
            name = "zkool-cargo-vendor-${
              builtins.substring 0 12 (builtins.hashFile "sha256" ./Cargo.lock)
            }";
            src = cargoSrc;
            nativeBuildInputs = [ rustToolchain pkgs.git pkgs.cacert ];
            outputHashMode = "recursive";
            outputHashAlgo = "sha256";
            outputHash = "sha256-akCtLs8ItKCvUb8Eo6sWtb2t5ksr7QiEqgOYwcWjEXs=";
            SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            GIT_SSL_CAINFO = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            buildCommand = ''
              export HOME="$TMPDIR"
              export CARGO_HOME="$TMPDIR/cargo-home"
              cp -r "$src" source
              chmod -R u+w source
              cd source
              cargo vendor --locked --versioned-dirs vendor > vendor-config.toml
              mkdir -p "$out"
              cp -r vendor "$out/vendor"
              cp vendor-config.toml "$out/config.toml"
            '';
          };

          # Pub cache FOD (app + cargokit build_tool), used by the pure build.
          zkool-pub-cache = pubCache;

          # EXPERIMENTAL pure GUI build (sandboxed, no network).
          zkool-pure = zkoolPure;

          # Hermetic build of the GraphQL server. Mirrors
          # .github/workflows/build-graphql.yml:
          #   cargo build --release --bin zkool_graphql
          #     --no-default-features --features=graphql,bundled-sapling-params
          zkool-graphql = pkgs.rustPlatform.buildRustPackage {
            pname = "zkool_graphql";
            version = zkoolVersion;
            src = cargoSrc;
            # Pinned vendor tree generated by buildRustPackage from Cargo.lock.
            cargoHash = "sha256-etZANen032i4MpIOYNMb8hrXChfA5ATa1pBmVnXfMWs=";
            cargoBuildFlags = [ "--bin" "zkool_graphql" ];
            buildNoDefaultFeatures = true;
            buildFeatures = [ "graphql" "bundled-sapling-params" ];
            # perl: libsqlite3-sys uses bundled-sqlcipher-vendored-openssl,
            # which compiles OpenSSL from source (needs perl + a C toolchain).
            nativeBuildInputs = with pkgs; [ pkg-config perl ]
              ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux [ udev ];
            # Tests spawn servers / touch the network; skip them for packaging.
            doCheck = false;
            meta = {
              description = "GraphQL server for Zcash operations";
              mainProgram = "zkool_graphql";
            };
          };
        };

        # Expose the package as `pkgs.zkool-graphql` for consumers:
        #   nixpkgs.overlays = [ inputs.zkool2.overlays.default ];
        #   environment.systemPackages = [ pkgs.zkool-graphql ];
        # Note: this pulls in the flake's own nixpkgs/rust/flutter, so it is
        # pinned to this flake rather than the consumer's nixpkgs revision.
        overlays.default = final: prev: {
          zkool-graphql = self.packages.${prev.system}.zkool-graphql;
        };

        # `nix run .#zkool-store`: build the GUI app in the dev shell and
        # import the bundle into the store (prints its /nix/store path).
        apps.zkool-store = {
          type = "app";
          program = "${zkoolStore}/bin/zkool-store";
        };

        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
