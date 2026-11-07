#!/usr/bin/env bash
# Launches a Linux bundle and its AppImage on another distro under Xvfb. Each
# passes if it is still running when the timeout stops it, since a loader failure
# exits at once. The AppImage is tested separately because it ships its own
# libstdc++ and puts it ahead of the system's, which can break on newer distros.
# Usage: smoke-linux-bundle.sh <bundle-dir> <appimage> <docker-image>
set -euo pipefail

usage="usage: smoke-linux-bundle.sh <bundle-dir> <appimage> <docker-image>"
bundle=$(cd "${1:?$usage}" && pwd)
appimage=$(cd "$(dirname "${2:?$usage}")" && pwd)/$(basename "$2")
image=${3:?$usage}

docker run --rm -v "$bundle:/app:ro" -v "$appimage:/zkool.AppImage:ro" "$image" bash -c '
  set -u
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -qq
  apt-get install -y -qq --no-install-recommends \
    libgtk-3-0 liblzma5 libudev1 libegl1 libgl1 libgles2 \
    libegl-mesa0 libgl1-mesa-dri xvfb xauth >/dev/null

  # librlz.so is loaded at runtime, so plain startup alone may not reach it.
  missing=$(for f in /app/zkool /app/lib/*.so; do
    LD_LIBRARY_PATH=/app/lib ldd "$f" 2>&1
  done | grep "not found" || true)
  if [ -n "$missing" ]; then
    echo "$missing"
    exit 1
  fi

  # There is no GPU in the container, so Mesa renders in software. Mesa also
  # needs libstdc++, so a version mismatch only shows up in the output.
  launch() {
    local name=$1
    shift
    xvfb-run -a -s "-screen 0 1280x1024x24" timeout 30 "$@" 2>&1 | tee /tmp/output
    local rc=${PIPESTATUS[0]}
    if [ "$rc" -ne 124 ]; then
      echo "$name exited early with status $rc"
      exit 1
    fi
    if grep -E "GLIBC(XX)?_[0-9.]+.* not found" /tmp/output; then
      echo "$name has a missing library version"
      exit 1
    fi
    echo "$name ran for 30s"
  }

  launch bundle /app/zkool
  # Containers have no FUSE, so the AppImage unpacks itself to run.
  APPIMAGE_EXTRACT_AND_RUN=1 launch AppImage /zkool.AppImage
'
