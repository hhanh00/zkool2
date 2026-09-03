#!/usr/bin/env bash
set -euo pipefail

# Start the speculos emulator running the built Zcash Ledger app.
# APP_DIR:     app checkout (default: ~/projects/ledger-dev/app-zcash)
# MODEL:       speculos model (nanox | nanosp | stax | flex | apex_p), default nanosp
# BUILD_MODEL: build model directory under target/ (default nanosplus for nanosp)
# SEED:        device seed (deterministic test seed by default)
#
# Endpoints:
#   http://localhost:9999  JSON APDU (POST {"apduHex": "<hex>"}) - what zkool speaks
#   http://localhost:5000  device screen / REST API

APP_DIR="${APP_DIR:-$HOME/projects/ledger-dev/app-zcash}"
MODEL="${MODEL:-nanosp}"
BUILD_MODEL="${BUILD_MODEL:-nanosplus}"
SEED="${SEED:-glory promote mansion idle axis finger extra february uncover one trip resource lawn turtle enact monster seven myth punch hobby comfort wild raise skin}"
ELF="$APP_DIR/target/$BUILD_MODEL/release/zcash"

if [ ! -f "$ELF" ]; then
  echo "Not found: $ELF (run build.sh first)" >&2
  exit 1
fi

docker rm -f speculos-zcash >/dev/null 2>&1 || true

docker run -d --name speculos-zcash \
  -p 9999:9998 -p 5000:5000 \
  -v "$APP_DIR/target":/app/target \
  -w /speculos \
  --entrypoint bash \
  ghcr.io/ledgerhq/speculos:latest -c "
    sed 's/HOST = \"127.0.0.1\"/HOST = \"0.0.0.0\"/' /speculos/tools/ledger-live-http-proxy.py > /tmp/proxy.py &&
    python ./speculos.py --model $MODEL --seed \"$SEED\" --display headless /app/target/$BUILD_MODEL/release/zcash &
    exec python /tmp/proxy.py" >/dev/null

sleep 5
docker logs speculos-zcash 2>&1 | tail -5
echo
echo "APDU endpoint : http://localhost:9999"
echo "Screen UI     : http://localhost:5000"
echo "Stop          : docker rm -f speculos-zcash"
