---
title: Building Zkool from source
---
## Requirements

Zkool has only 2 build requirements:
- [rust](https://www.rust-lang.org/tools/install)
- [flutter](https://docs.flutter.dev/install)

The rust toolchain version is pinned in `rust-toolchain.toml`.
`rustup` installs it automatically the first time you build.

The releases are built with Flutter 3.47.4.

## Build
```
flutter build xyz
```
where `xyz` is `linux`, `macos`, `windows`, etc.

The release builds enable the Ledger support and bundle the
Sapling parameters. To build the same way, run this before
`flutter build`:
```
./misc/mkcargokit_options.sh "ledger,bundled-sapling-params" > rust/cargokit.yaml
```

## Debian
For example on Linux Debian based distribs:
- These are the package requirements for flutter
```
sudo apt install clang cmake git ninja-build pkg-config libgtk-3-dev liblzma-dev libstdc++-12-dev libudev-dev
```
- Install flutter
```
git clone https://github.com/flutter/flutter --depth 1 --branch stable
cd flutter/bin
./flutter doctor -v
export PATH=$PWD:$PATH
```
- Install rust
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
export PATH=$HOME/.cargo/bin:$PATH
```
- Build Zkool
```
cd zkool2
flutter build linux
```

## Fedora
Only the installation of the required packages differs.

```
sudo dnf install clang cmake ninja-build gtk3-devel perl openssl-devel systemd-devel
```

## Archlinux
Only the installation of the required packages differs.

```
sudo pacman -S unzip clang cmake ninja pkgconf gtk3
```

## Nix

The repository has a flake with a development shell that
provides the whole toolchain, and a package for the Linux build.
See [NIX_BUILD.md](https://github.com/hhanh00/zkool2/blob/main/NIX_BUILD.md).

## Windows

`flutter build windows` produces the application folder.
The installer of the releases is made with
[fastforge](https://fastforge.dev) and Inno Setup:
```
dart pub global activate fastforge
fastforge package --platform=windows --targets=exe,zip
```
