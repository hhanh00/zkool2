#!/usr/bin/env bash
# The local voting stack on the running regtest chain, for the voting UI runs.
#
#   tests/tools/voting_stack.sh up [--snapshot H] [--title T] [--proposals FILE] [--duration S]
#   tests/tools/voting_stack.sh down
#   tests/tools/voting_stack.sh status
#   tests/tools/voting_stack.sh ca        # prints the CA certificate's path, making it if needed
#
# `up` starts from nothing every time (it runs `down` first): a fresh svoted
# chain, the PIR import of the regtest nullifiers at the snapshot height,
# nf-server, one active voting round, and the signed configuration served
# over TLS under a throwaway CA, as the SDK insists on https. It mirrors
# .github/workflows/e2e-voting.yml through the same scripts under scripts/,
# and writes what a client needs to data/voting/voting.env: the config URL,
# the CA certificate (the wallet takes it through ZKOOL_EXTRA_ROOT_CERT_PATH,
# so nothing is added to the system trust store), the round title and the
# snapshot height.
#
# Binaries live under the untracked tools/voting (svoted, nf-server,
# voting-config, the vote-sdk checkout for its init script and protos);
# .claude/skills/voting-stack/SKILL.md says how to get them. State and logs
# live under the untracked data/voting; `down` stops the services and
# removes it. The regtest chain and lightwalletd must already be up.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
TOOLS=$ROOT/tools/voting
DATA=$ROOT/data/voting
LOGS=$DATA/logs
SVOTED_HOME=$DATA/svoted
PIR_DATA_DIR=$DATA/pir
CONFIG_DIR=$DATA/config
CERT_DIR=$DATA/tls
# The CA outlives `down`, so a client can be given it before `up` runs.
CA_DIR=$ROOT/data/voting-ca
CHAIN_ID=svote-local
CONFIG_PORT=${CONFIG_PORT:-8443}
PIR_PORT=${PIR_PORT:-3000}
REGTEST_RPC_URL=${REGTEST_RPC_URL:-http://127.0.0.1:18232}
REGTEST_LWD_URL=${REGTEST_LWD_URL:-http://127.0.0.1:8137}
export PATH="$TOOLS:$PATH"

log() { printf '[voting-stack] %s\n' "$*" >&2; }
die() { log "$*"; exit 1; }

need() {
  for bin in "$@"; do
    command -v "$bin" >/dev/null || die "$bin not found; see .claude/skills/voting-stack/SKILL.md"
  done
}

stop_pid() {
  local file=$DATA/$1.pid pid
  [[ -f "$file" ]] || return 0
  pid=$(cat "$file")
  if kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
    for _ in $(seq 1 20); do kill -0 "$pid" 2>/dev/null || break; sleep 0.5; done
    kill -9 "$pid" 2>/dev/null || true
  fi
  rm -f "$file"
}

wait_for() {
  local what=$1 tries=$2; shift 2
  for _ in $(seq 1 "$tries"); do
    if "$@" >/dev/null 2>&1; then return 0; fi
    sleep 1
  done
  die "$what did not come up"
}

# The throwaway CA, made once and kept: prints the certificate's path.
ca() {
  if [[ ! -f "$CA_DIR/ca.crt" ]]; then
    mkdir -p "$CA_DIR"
    openssl req -x509 -new -nodes -sha256 -days 365 -subj '/CN=ZKool local voting CA' \
      -keyout "$CA_DIR/ca.key" -out "$CA_DIR/ca.crt" >/dev/null 2>&1 || die "issuing the CA failed"
  fi
  echo "$CA_DIR/ca.crt"
}

down() {
  stop_pid config-server
  stop_pid nf-server
  stop_pid svoted
  rm -rf "$DATA"
  log "down"
}

status() {
  local ok=0
  curl -fsS "http://127.0.0.1:1317/cosmos/base/tendermint/v1beta1/node_info" >/dev/null 2>&1 && log "svoted: up" || { log "svoted: down"; ok=1; }
  curl -fsS "http://127.0.0.1:$PIR_PORT/root" >/dev/null 2>&1 && log "nf-server: up" || { log "nf-server: down"; ok=1; }
  curl -fsS --cacert "$CA_DIR/ca.crt" "https://localhost:$CONFIG_PORT/static-voting-config.json" >/dev/null 2>&1 && log "config server: up" || { log "config server: down"; ok=1; }
  [[ -f "$DATA/voting.env" ]] && cat "$DATA/voting.env"
  return $ok
}

up() {
  local snapshot="" title="Zcash Community Poll" proposals="" duration=3600
  while [[ $# -gt 0 ]]; do
    case $1 in
      --snapshot) snapshot=$2; shift 2 ;;
      --title) title=$2; shift 2 ;;
      --proposals) proposals=$(cd "$(dirname "$2")" && pwd)/$(basename "$2"); shift 2 ;;
      --duration) duration=$2; shift 2 ;;
      *) die "unknown option $1" ;;
    esac
  done
  need svoted nf-server voting-config grpcurl jq xxd openssl curl python3
  [[ -d "$TOOLS/vote-sdk/proto" ]] || die "tools/voting/vote-sdk is missing"
  curl -fsS -H 'Content-Type: application/json' \
    --data '{"jsonrpc":"1.0","id":"h","method":"getblockcount","params":[]}' "$REGTEST_RPC_URL" >/dev/null \
    || die "zebra is not answering at $REGTEST_RPC_URL"

  down
  mkdir -p "$LOGS" "$PIR_DATA_DIR" "$CONFIG_DIR" "$CERT_DIR"

  log "issuing the config server's certificate"
  ca
  (
    cd "$CERT_DIR"
    openssl req -new -nodes -sha256 -subj '/CN=localhost' -addext 'subjectAltName=DNS:localhost,IP:127.0.0.1' -keyout server.key -out server.csr
    openssl x509 -req -sha256 -days 365 -in server.csr -CA "$CA_DIR/ca.crt" -CAkey "$CA_DIR/ca.key" -CAcreateserial -out server.crt \
      -extfile <(printf 'subjectAltName=DNS:localhost,IP:127.0.0.1')
  ) >"$LOGS/tls.log" 2>&1 || { tail -20 "$LOGS/tls.log" >&2; die "issuing the certificates failed"; }

  log "initializing the svoted chain"
  SVOTED_HOME=$SVOTED_HOME CHAIN_ID=$CHAIN_ID VM_PRIVKEYS=$(openssl rand -hex 32) \
    SVOTE_USE_DEFAULT_GENESIS_VOTE_MANAGERS=false \
    bash "$TOOLS/vote-sdk/scripts/init.sh" >"$LOGS/svoted-init.log" 2>&1 \
    || { tail -30 "$LOGS/svoted-init.log" >&2; die "svoted init failed"; }
  SVOTE_PIR_URL=disabled nohup svoted start --home "$SVOTED_HOME" >"$LOGS/svoted.log" 2>&1 &
  echo $! >"$DATA/svoted.pid"
  wait_for "svoted" 60 curl -fsS "http://127.0.0.1:1317/cosmos/base/tendermint/v1beta1/node_info"

  if [[ -z "$snapshot" ]]; then
    local height
    height=$(curl -fsS -H 'Content-Type: application/json' \
      --data '{"jsonrpc":"1.0","id":"h","method":"getblockcount","params":[]}' "$REGTEST_RPC_URL" | jq -er .result)
    snapshot=$(( height / 10 * 10 ))
  fi
  log "importing the regtest nullifiers up to height $snapshot"
  REGTEST_RPC_URL=$REGTEST_RPC_URL REGTEST_LWD_URL=$REGTEST_LWD_URL PIR_DATA_DIR=$PIR_DATA_DIR \
    PIR_PORT=$PIR_PORT OUTPUT_ENV=$DATA/pir-server.env NF_SERVER_BIN=nf-server SNAPSHOT_HEIGHT=$snapshot \
    bash "$ROOT/scripts/configure-pir-server.sh" >"$LOGS/pir-import.log" 2>&1 \
    || { tail -30 "$LOGS/pir-import.log" >&2; die "the PIR import failed"; }
  set -a; source "$DATA/pir-server.env"; set +a
  nohup nf-server serve --zcash-network regtest --pir-data-dir "$SVOTE_PIR_DATA_DIR" --port "$SVOTE_PIR_PORT" \
    >"$LOGS/nf-server.log" 2>&1 &
  echo $! >"$DATA/nf-server.pid"
  wait_for "nf-server" 60 curl -fsS "http://127.0.0.1:$SVOTE_PIR_PORT/root"

  log "creating the round \"$title\""
  SVOTED_BIN=svoted SVOTED_HOME=$SVOTED_HOME CHAIN_ID=$CHAIN_ID SVOTED_GRPC=127.0.0.1:9190 \
    VOTE_SDK_PROTO_ROOT=$TOOLS/vote-sdk/proto ZEBRA_RPC_URL=$REGTEST_RPC_URL PIR_URL=http://127.0.0.1:$SVOTE_PIR_PORT \
    VOTE_DURATION_SECONDS=$duration ROUND_COUNT=1 ROUND_TITLE="$title" ROUND_TITLE_PREFIX="$title" ROUND_DESCRIPTION_PREFIX="$title " \
    PROPOSALS_JSON=$proposals \
    bash "$ROOT/scripts/create-e2e-voting-rounds.sh" >"$LOGS/rounds.log" 2>&1 \
    || { tail -30 "$LOGS/rounds.log" >&2; die "creating the round failed"; }

  log "signing and serving the configuration"
  VOTING_CONFIG_BIN=voting-config SVOTED_GRPC=127.0.0.1:9190 VOTE_SDK_PROTO_ROOT=$TOOLS/vote-sdk/proto \
    CONFIG_DIR=$CONFIG_DIR PUBLIC_BASE_URL=https://localhost:$CONFIG_PORT ROUND_COUNT=1 ROUND_TITLE_PREFIX="$title" \
    bash "$ROOT/scripts/configure-e2e-voting-config.sh" >"$LOGS/config.log" 2>&1 \
    || { tail -30 "$LOGS/config.log" >&2; die "signing the configuration failed"; }
  nohup python3 "$ROOT/scripts/serve-e2e-voting-config.py" --directory "$CONFIG_DIR" --port "$CONFIG_PORT" \
    --cert "$CERT_DIR/server.crt" --key "$CERT_DIR/server.key" \
    --grpcurl "$(command -v grpcurl)" --proto-root "$TOOLS/vote-sdk/proto" >"$LOGS/config-server.log" 2>&1 &
  echo $! >"$DATA/config-server.pid"
  wait_for "the config server" 30 curl -fsS --cacert "$CA_DIR/ca.crt" "https://localhost:$CONFIG_PORT/static-voting-config.json"

  # Quoted, so the file can be sourced whatever the title holds.
  {
    sed "s/^VOTING_CONFIG_URL=\(.*\)$/VOTING_CONFIG_URL='\1'/" "$CONFIG_DIR/config.env"
    printf "VOTING_CA_CERT='%s'\n" "$CA_DIR/ca.crt"
    printf "VOTING_ROUND_TITLE='%s'\n" "$title"
    printf 'VOTING_SNAPSHOT_HEIGHT=%s\n' "$snapshot"
  } >"$DATA/voting.env"
  log "up"
  cat "$DATA/voting.env"
}

case ${1:-} in
  up) shift; up "$@" ;;
  down) down ;;
  status) status ;;
  ca) ca ;;
  *) echo "usage: $0 up [--snapshot H] [--title T] [--proposals FILE] [--duration S] | down | status | ca" >&2; exit 2 ;;
esac
