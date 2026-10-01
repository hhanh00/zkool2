#!/usr/bin/env bash
set -euo pipefail

# Read the local regtest height and emit settings consumed by nf-server. The
# PIR dataset must be generated from the same regtest chain as the wallet.

REGTEST_RPC_URL=${REGTEST_RPC_URL:-http://127.0.0.1:18232}
REGTEST_LWD_URL=${REGTEST_LWD_URL:-http://127.0.0.1:8137}
PIR_DATA_DIR=${PIR_DATA_DIR:-"$PWD/pir-data/regtest"}
PIR_PORT=${PIR_PORT:-3000}
LWD_URLS=${LWD_URLS:-$REGTEST_LWD_URL}
OUTPUT_ENV=${OUTPUT_ENV:-"$PWD/pir-server.env"}
NF_SERVER_BIN=${NF_SERVER_BIN:-nf-server}
IMPORT_PIR=${IMPORT_PIR:-1}

command -v curl >/dev/null || { echo "curl is required" >&2; exit 127; }
command -v jq >/dev/null || { echo "jq is required" >&2; exit 127; }

if [[ -z "${SNAPSHOT_HEIGHT:-}" ]]; then
snapshot_height=$(curl --fail --silent --show-error \
    -H 'Content-Type: application/json' \
    --data '{"jsonrpc":"1.0","id":"height","method":"getblockcount","params":[]}' \
    "$REGTEST_RPC_URL" | jq -er '.result | numbers')
else
  snapshot_height=$SNAPSHOT_HEIGHT
fi

[[ "$snapshot_height" =~ ^[0-9]+$ ]] || {
  echo "snapshot height must be a non-negative integer" >&2
  exit 2
}
if (( snapshot_height % 10 != 0 )); then
  echo "regtest snapshot height must be a multiple of 10 (got $snapshot_height)" >&2
  exit 2
fi

mkdir -p "$PIR_DATA_DIR"
cat >"$OUTPUT_ENV" <<EOF
SVOTE_ZCASH_NETWORK=regtest
SVOTE_PIR_CONFIG_URL=
SVOTE_PIR_FORCE_SNAPSHOT_HEIGHT=$snapshot_height
SVOTE_PIR_DATA_DIR=$PIR_DATA_DIR
SVOTE_PIR_PORT=$PIR_PORT
LWD_URLS=$LWD_URLS
EOF

if [[ -n "${GITHUB_ENV:-}" ]]; then
  cat "$OUTPUT_ENV" >>"$GITHUB_ENV"
fi

if [[ "$IMPORT_PIR" == 1 ]]; then
  command -v "$NF_SERVER_BIN" >/dev/null || {
    echo "nf-server is required to import the regtest snapshot" >&2
    exit 127
  }
  # nf-server reads LWD_URLS, SVOTE_ZCASH_NETWORK, and the data directory
  # from its environment. Importing at --max-height prevents the PIR dataset
  # from racing ahead of the wallet's selected snapshot.
  set -a
  source "$OUTPUT_ENV"
  set +a
  "$NF_SERVER_BIN" sync \
    --pir-data-dir "$SVOTE_PIR_DATA_DIR" \
    --max-height "$SVOTE_PIR_FORCE_SNAPSHOT_HEIGHT" \
    --non-interactive
fi

echo "PIR environment: regtest"
echo "PIR snapshot: $snapshot_height"
echo "configuration written to: $OUTPUT_ENV"
