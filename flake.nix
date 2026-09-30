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

        # Must match rust-toolchain.toml: 1.88-1.94 fail to compile
        # libcrux-psq (transitive nym dep) with E0716; 1.95.0 builds cleanly.
        rustVersion = "1.95.0";

        rustToolchain = pkgs.rust-bin.stable.${rustVersion}.default.override {
          extensions = [ "rust-analyzer" "rust-src" ];
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

        # The Flutter GUI cannot be a normal Nix package here: Flutter fetches
        # engine/pub artifacts and cargokit shells out to cargo (network), and
        # a fixed-output derivation is not an option because Nix 2.34 rejects
        # store-path references in FOD outputs while the bundle links
        # GTK/WebKit/etc. from the store. Instead `nix run .#zkool-store`
        # builds in the dev shell and imports the result with `nix store
        # add-path`. See NIX_BUILD.md.
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
            name = "zkool-cargo-vendor";
            src = cargoSrc;
            nativeBuildInputs = [ rustToolchain pkgs.git pkgs.cacert ];
            outputHashMode = "recursive";
            outputHashAlgo = "sha256";
            outputHash = "sha256-Cw1RksSwVtODd0Z/D19LXfd6gXDSvlwI9HhZkp9J4wY=";
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
        } // lib.optionalAttrs (pkgs.stdenv.hostPlatform.system == "x86_64-linux") {
          # Prebuilt server binary from the GitHub release, so `nix build`/
          # `nix profile install` downloads instead of compiling Rust. The
          # release is produced by .github/workflows/build-graphql.yml.
          zkool-graphql-bin = pkgs.stdenv.mkDerivation {
            pname = "zkool_graphql-bin";
            version = zkoolVersion;
            src = pkgs.fetchurl {
              url = "https://github.com/hhanh00/zkool2/releases/download/zkool-v${zkoolVersion}/zkool_graphql";
              hash = "sha256-IRUizrR6z/n1uOxAYWxeNcXgHwrvQdGkMlGeo6+bEOM=";
            };
            dontUnpack = true;
            nativeBuildInputs = [ pkgs.autoPatchelfHook ];
            buildInputs = [ pkgs.stdenv.cc.cc.lib pkgs.glibc ];
            installPhase = ''
              runHook preInstall
              install -Dm755 "$src" "$out/bin/zkool_graphql"
              runHook postInstall
            '';
            meta = {
              description = "GraphQL server for Zcash operations (prebuilt)";
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
