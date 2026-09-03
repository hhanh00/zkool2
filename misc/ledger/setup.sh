#!/usr/bin/env bash
set -euo pipefail

# One-time setup: clone the Zcash Ledger app and pull the Docker images.
# APP_DIR overrides where the app is cloned (default: ~/projects/ledger-dev/app-zcash)

APP_DIR="${APP_DIR:-$HOME/projects/ledger-dev/app-zcash}"

mkdir -p "$(dirname "$APP_DIR")"
if [ ! -d "$APP_DIR" ]; then
  git clone --depth 1 https://github.com/LedgerHQ/app-zcash.git "$APP_DIR"
fi

docker pull ghcr.io/ledgerhq/ledger-app-builder/ledger-app-dev-tools:latest
docker pull ghcr.io/ledgerhq/speculos:latest
