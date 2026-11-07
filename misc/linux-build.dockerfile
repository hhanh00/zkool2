# Build environment for Linux releases. The base distro's glibc, libstdc++ and
# GLib headers set the minimum for every user, so changing it changes the limits
# in check-bundle-abi.sh. Flutter, Rust and fastforge come from the runner.
FROM ubuntu:22.04

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    build-essential ca-certificates clang cmake curl file git libgtk-3-dev \
    liblzma-dev libstdc++-12-dev libudev-dev locate ninja-build perl \
    pkg-config python3 unzip wget xz-utils zip \
  && rm -rf /var/lib/apt/lists/*

RUN wget -q -O /usr/local/lib/appimagetool \
    "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-$(uname -m).AppImage" \
  && chmod +x /usr/local/lib/appimagetool

# fastforge calls appimagetool with ARCH=x86_64 on every architecture, which
# gives aarch64 AppImages an x86_64 runtime that cannot start.
RUN printf '#!/bin/sh\nARCH="$(uname -m)" exec /usr/local/lib/appimagetool "$@"\n' \
    > /usr/local/bin/appimagetool \
  && chmod +x /usr/local/bin/appimagetool
