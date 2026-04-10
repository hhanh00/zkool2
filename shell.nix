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
  RUSTFLAGS = pkgs.lib.optionals pkgs.stdenv.isLinux "-C link-arg=-fuse-ld=lld";

  shellHook = ''
    echo "🦀 Rust development environment for zkool_graphql"
    echo "Build with: cd rust && cargo build --release --bin zkool_graphql --features=graphql"
  '';
}
