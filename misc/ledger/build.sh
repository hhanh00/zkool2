#!/usr/bin/env bash
set -euo pipefail

# Build the Zcash Ledger app in the dockerized toolchain.
# APP_DIR: app checkout (default: ~/projects/ledger-dev/app-zcash)
# MODEL:   build model (nanox | nanosplus | stax | flex | apex_p), default nanosplus

APP_DIR="${APP_DIR:-$HOME/projects/ledger-dev/app-zcash}"
MODEL="${MODEL:-nanosplus}"

docker run --rm -v "$APP_DIR":/app \
  ghcr.io/ledgerhq/ledger-app-builder/ledger-app-dev-tools:latest \
  cargo ledger build "$MODEL"

echo "Built: $APP_DIR/target/$MODEL/release/zcash"
