{
  description = "zkool_graphql - GraphQL server for Zcash operations";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];

        buildInputs = with pkgs; [
          openssl
          sqlite
        ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
          udev
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          RUST_BACKTRACE = "1";

          shellHook = ''
            echo "🦀 Rust development environment for zkool_graphql"
            echo "Build with: cd rust && cargo build --release --bin zkool_graphql --features=graphql"
          '';
        };

      }
    );
}
