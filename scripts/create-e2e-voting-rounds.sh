#!/usr/bin/env bash
set -euo pipefail

# Create two real svoted sessions over the locally imported regtest snapshot.
# Each session has three proposals and each proposal has three choices.

SVOTED_BIN=${SVOTED_BIN:-svoted}
SVOTED_HOME=${SVOTED_HOME:?SVOTED_HOME must be set}
CHAIN_ID=${CHAIN_ID:-svote-ci}
SVOTED_NODE=${SVOTED_NODE:-tcp://127.0.0.1:26657}
SVOTED_GRPC=${SVOTED_GRPC:-127.0.0.1:9190}
GRPCURL_BIN=${GRPCURL_BIN:-grpcurl}
VOTE_SDK_PROTO_ROOT=${VOTE_SDK_PROTO_ROOT:?VOTE_SDK_PROTO_ROOT must be set}
ZEBRA_RPC_URL=${ZEBRA_RPC_URL:-http://127.0.0.1:18232}
PIR_URL=${PIR_URL:-http://127.0.0.1:3000}
PIR_DATA_DIR=${SVOTE_PIR_DATA_DIR:?SVOTE_PIR_DATA_DIR must be set}
SNAPSHOT_HEIGHT=${SVOTE_PIR_FORCE_SNAPSHOT_HEIGHT:?SVOTE_PIR_FORCE_SNAPSHOT_HEIGHT must be set}
VOTE_DURATION_SECONDS=${VOTE_DURATION_SECONDS:-120}

rpc() {
  curl -fsS -H 'Content-Type: application/json' \
    --data "{\"jsonrpc\":\"1.0\",\"id\":\"e2e\",\"method\":\"$1\",\"params\":$2}" \
    "$ZEBRA_RPC_URL"
}

snapshot_blockhash=$(rpc getblockhash "[$SNAPSHOT_HEIGHT]" | jq -er '.result')
# A round's nullifier_imt_root is the depth-29 circuit root, not the
# shallower PIR transport-tree root. Read it from the running server, then
# verify that it is the root exported for the snapshot used by this round.
pir_root_info=$(curl -fsS "$PIR_URL/root")
nullifier_root=$(jq -er '.circuit_root' <<<"$pir_root_info")
exported_nullifier_root=$(jq -er '.circuit_root' "$PIR_DATA_DIR/pir_root.json")
if [[ "$nullifier_root" != "$exported_nullifier_root" ]]; then
  echo "running PIR server circuit_root does not match pir_root.json" >&2
  echo "server:   $nullifier_root" >&2
  echo "exported: $exported_nullifier_root" >&2
  exit 1
fi

# Zebra currently exposes the local Ironwood commitment tree under its
# Orchard-compatible field. Accept both that wire form and newer Ironwood
# names, while always requiring a root from the local regtest node.
tree_state=$(rpc z_gettreestate "[\"$SNAPSHOT_HEIGHT\"]")
echo "Zebra z_gettreestate response at height $SNAPSHOT_HEIGHT:" >&2
jq . <<<"$tree_state" >&2
nc_root=$(jq -er '
  .result.ironwood.commitments.finalRoot
  // .result.orchard.commitments.finalRoot
  // error("Ironwood note-commitment root missing from z_gettreestate")' <<<"$tree_state")

[[ "$snapshot_blockhash" =~ ^[0-9a-fA-F]{64}$ ]]
[[ "$nullifier_root" =~ ^[0-9a-fA-F]{64}$ ]]
[[ "$nc_root" =~ ^[0-9a-fA-F]{64}$ ]]

proposals=$(jq -cn '[range(1; 4) | {
  id: .,
  title: ("Proposal " + tostring),
  description: ("E2E proposal " + tostring),
  options: [range(0; 3) | {
    index: ., label: ("Choice " + (. + 1 | tostring)),
    description: ("E2E choice " + (. + 1 | tostring))
  }]
}]')
proposals_hash=$(printf '%s' "$proposals" | openssl dgst -sha256 -binary | xxd -p -c 256)

create_round() {
  local number=$1 session_json result expected_nullifier_root
  session_json=$(mktemp "${RUNNER_TEMP:-/tmp}/e2e-voting-session.XXXXXX.json")
  jq -n \
    --argjson snapshot_height "$SNAPSHOT_HEIGHT" \
    --arg snapshot_blockhash "$snapshot_blockhash" \
    --arg proposals_hash "$proposals_hash" \
    --argjson vote_end_time "$(( $(date +%s) + VOTE_DURATION_SECONDS ))" \
    --arg nullifier_imt_root "$nullifier_root" \
    --arg nc_root "$nc_root" \
    --arg title "E2E Round $number" \
    --arg description "Real regtest voting session $number" \
    --argjson proposals "$proposals" \
    '{snapshot_height: $snapshot_height, snapshot_blockhash: $snapshot_blockhash,
      proposals_hash: $proposals_hash, vote_end_time: $vote_end_time,
      nullifier_imt_root: $nullifier_imt_root, nc_root: $nc_root,
      title: $title, description: $description, proposals: $proposals}' >"$session_json"

  result=$("$SVOTED_BIN" tx vote create-voting-session "$session_json" \
    --from vote-manager-1 --keyring-backend test --home "$SVOTED_HOME" \
    --chain-id "$CHAIN_ID" --node "$SVOTED_NODE" --yes --output json)
  jq -e '.code == 0' <<<"$result" >/dev/null

  for _ in $(seq 1 60); do
    rounds=$("$GRPCURL_BIN" -plaintext \
      -import-path "$VOTE_SDK_PROTO_ROOT" -proto svote/v1/query.proto \
      -d '{}' "$SVOTED_GRPC" svote.v1.Query/ListRounds)
    expected_nullifier_root=$(printf '%s' "$nullifier_root" | xxd -r -p | base64 | tr -d '\n')
    if jq -e --arg title "E2E Round $number" --arg root "$expected_nullifier_root" \
      '[.rounds[]? | select(.title == $title and .nullifierImtRoot == $root)
        | .proposals[] | select(.options | length == 3)] | length == 3' \
      <<<"$rounds" >/dev/null; then
      echo "created E2E Round $number with PIR circuit root $nullifier_root"
      return
    fi
    sleep 1
  done
  echo "E2E Round $number was not visible through svoted gRPC" >&2
  exit 1
}

wait_for_active_round() {
  local number=$1 rounds
  for _ in $(seq 1 120); do
    rounds=$("$GRPCURL_BIN" -plaintext \
      -import-path "$VOTE_SDK_PROTO_ROOT" -proto svote/v1/query.proto \
      -d '{}' "$SVOTED_GRPC" svote.v1.Query/ListRounds)
    if jq -e --arg title "E2E Round $number" \
      '.rounds[]? | select(.title == $title) | .status == "SESSION_STATUS_ACTIVE"' \
      <<<"$rounds" >/dev/null; then
      echo "E2E Round $number is active"
      return
    fi
    sleep 1
  done
  echo "E2E Round $number did not become active" >&2
  exit 1
}

create_round 1
wait_for_active_round 1
create_round 2
wait_for_active_round 2
