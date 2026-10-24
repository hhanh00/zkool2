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
# What the rounds say. CI keeps the defaults; a docs or local run names its
# own round and hands a proposals JSON file (the array the session takes).
ROUND_COUNT=${ROUND_COUNT:-2}
ROUND_TITLE_PREFIX=${ROUND_TITLE_PREFIX:-"E2E Round "}
# An exact title, for a single round that should not carry a number.
ROUND_TITLE=${ROUND_TITLE:-}
ROUND_DESCRIPTION_PREFIX=${ROUND_DESCRIPTION_PREFIX:-"Real regtest voting session "}
PROPOSALS_JSON=${PROPOSALS_JSON:-}

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

if [[ -n "$PROPOSALS_JSON" ]]; then
  proposals=$(jq -c . "$PROPOSALS_JSON")
else
  proposals=$(jq -cn '[range(1; 4) | {
    id: .,
    title: ("Proposal " + tostring),
    description: ("E2E proposal " + tostring),
    options: [range(0; 3) | {
      index: ., label: ("Choice " + (. + 1 | tostring)),
      description: ("E2E choice " + (. + 1 | tostring))
    }]
  }]')
fi
# The option count of each proposal, to recognize the round once listed.
expected_options=$(jq -c '[.[] | (.options | length)]' <<<"$proposals")
proposals_hash=$(printf '%s' "$proposals" | openssl dgst -sha256 -binary | xxd -p -c 256)

round_title() {
  if [[ -n "$ROUND_TITLE" ]]; then printf '%s' "$ROUND_TITLE"; else printf '%s%s' "$ROUND_TITLE_PREFIX" "$1"; fi
}
create_round() {
  local number=$1 session_json result expected_nullifier_root
  session_json=$(mktemp "${RUNNER_TEMP:-/tmp}/e2e-voting-session.XXXXXX")
  jq -n \
    --argjson snapshot_height "$SNAPSHOT_HEIGHT" \
    --arg snapshot_blockhash "$snapshot_blockhash" \
    --arg proposals_hash "$proposals_hash" \
    --argjson vote_end_time "$(( $(date +%s) + VOTE_DURATION_SECONDS ))" \
    --arg nullifier_imt_root "$nullifier_root" \
    --arg nc_root "$nc_root" \
    --arg title "$(round_title "$number")" \
    --arg description "$ROUND_DESCRIPTION_PREFIX$number" \
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
    if jq -e --arg title "$(round_title "$number")" --arg root "$expected_nullifier_root" \
      --argjson expected "$expected_options" \
      '[.rounds[]? | select(.title == $title and .nullifierImtRoot == $root)
        | .proposals[] | (.options | length)] == $expected' \
      <<<"$rounds" >/dev/null; then
      echo "created $(round_title "$number") with PIR circuit root $nullifier_root"
      return
    fi
    sleep 1
  done
  echo "$(round_title "$number") was not visible through svoted gRPC" >&2
  exit 1
}

wait_for_active_round() {
  local number=$1 rounds
  for _ in $(seq 1 120); do
    rounds=$("$GRPCURL_BIN" -plaintext \
      -import-path "$VOTE_SDK_PROTO_ROOT" -proto svote/v1/query.proto \
      -d '{}' "$SVOTED_GRPC" svote.v1.Query/ListRounds)
    if jq -e --arg title "$(round_title "$number")" \
      '.rounds[]? | select(.title == $title) | .status == "SESSION_STATUS_ACTIVE"' \
      <<<"$rounds" >/dev/null; then
      echo "$(round_title "$number") is active"
      return
    fi
    sleep 1
  done
  echo "$(round_title "$number") did not become active" >&2
  exit 1
}

for number in $(seq 1 "$ROUND_COUNT"); do
  create_round "$number"
  wait_for_active_round "$number"
done
