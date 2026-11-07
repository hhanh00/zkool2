#!/usr/bin/env bash
# Builds the Linux deb and AppImage inside an old distro, because the build
# system's glibc, libstdc++ and GLib headers set the minimum for every user.
# Reuses the runner's Flutter, Rust and fastforge by mounting $HOME, which also
# holds the checkout, so the output lands in the workspace as usual.
# Usage: build-linux-in-container.sh
set -euo pipefail

# Built from misc/linux-build.dockerfile by the Linux build image workflow, which
# prints the digest. Pinned so each release builds from a known set of packages.
IMAGE=ghcr.io/hhanh00/zkool-linux-build@sha256:713c252256346625eb7d0fb8b87160de1b3f91e17073b824dd4c2cd8d957d635

# Build as the runner user, so later steps on the host own the output.
docker run --rm \
  --user "$(id -u):$(id -g)" \
  -v "$HOME:$HOME" \
  -w "$PWD" \
  -e HOME -e PATH -e CARGO_HOME \
  -e NEAR_INTENTS_API_KEY \
  -e APPIMAGE_EXTRACT_AND_RUN=1 \
  "$IMAGE" bash -euo pipefail -c '
    ./misc/mkcargokit_options.sh ledger,bundled-sapling-params > rust/cargokit.yaml
    fastforge package --platform=linux --targets=deb,appimage
  '
