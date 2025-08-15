---
title: Building Zkool from source
---
## Requirements

Zkool has only 2 build requirements:
- [rust](https://www.rust-lang.org/tools/install)
- [flutter](https://docs.flutter.dev/install)

## Build
```
flutter build xyz
```
where `xyz` is `linux`, `macos`, etc.

## Debian
For example on Linux Debian based distribs:
- These are the package requirements for flutter
```
sudo apt install clang cmake git ninja-build pkg-config libgtk-3-dev liblzma-dev
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
