{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    pkg-config
    openssl
    sqlite
  ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
    udev
  ];

  RUST_BACKTRACE = "1";

  shellHook = ''
    echo "🦀 Rust development environment for zkool_graphql"
    echo "Build with: cargo build --release --bin zkool_graphql --features=graphql"
    cd rust
  '';
}
