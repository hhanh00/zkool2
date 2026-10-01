#!/usr/bin/env bash
set -euo pipefail

# Build a signed, CI-local voting configuration from the two active rounds on
# the real svoted chain. Nothing in this script invents a round or authority
# key: both come directly from svoted's REST API after the DKG completes.

VOTING_CONFIG_BIN=${VOTING_CONFIG_BIN:?VOTING_CONFIG_BIN must be set}
GRPCURL_BIN=${GRPCURL_BIN:-grpcurl}
SVOTED_GRPC=${SVOTED_GRPC:-127.0.0.1:9190}
VOTE_SDK_PROTO_ROOT=${VOTE_SDK_PROTO_ROOT:?VOTE_SDK_PROTO_ROOT must be set}
PIR_DATA_DIR=${SVOTE_PIR_DATA_DIR:?SVOTE_PIR_DATA_DIR must be set}
CONFIG_DIR=${CONFIG_DIR:-"$PWD/e2e-voting-config"}
PUBLIC_BASE_URL=${PUBLIC_BASE_URL:-https://localhost:8443}

command -v curl >/dev/null
command -v jq >/dev/null

mkdir -p "$CONFIG_DIR"
rounds=$("$GRPCURL_BIN" -plaintext \
  -import-path "$VOTE_SDK_PROTO_ROOT" -proto svote/v1/query.proto \
  -d '{}' "$SVOTED_GRPC" svote.v1.Query/ListRounds)
mapfile -t active_rounds < <(jq -r '
  .rounds[]
  | select(.title | startswith("E2E Round "))
  | select(.status == 1 or .status == "SESSION_STATUS_ACTIVE")
  | [.voteRoundId, .eaPk] | @tsv
' <<<"$rounds")

[[ ${#active_rounds[@]} == 2 ]] || {
  echo "expected exactly two active E2E rounds, found ${#active_rounds[@]}" >&2
  jq . <<<"$rounds" >&2
  exit 1
}

layout=$(jq -ce '.pir_layout' "$PIR_DATA_DIR/pir_root.json")
signer_id=e2e-ci
key_file="$CONFIG_DIR/e2e-ci.seed"
keygen=$($VOTING_CONFIG_BIN keygen --signer-id "$signer_id" --out "$key_file")
trusted_key=$(sed -n 's/^trusted_keys_entry: //p' <<<"$keygen")
[[ -n "$trusted_key" ]]

jq -n \
  --arg vote_url "$PUBLIC_BASE_URL/vote" \
  --arg pir_url "$PUBLIC_BASE_URL/pir" \
  --argjson layout "$layout" \
  '{config_version: 1,
    vote_servers: [{url: $vote_url, label: "local svoted"}],
    pir_endpoints: [{url: $pir_url, label: "local nf-server"}],
    pir_layout: $layout,
    supported_versions: {pir: ["v0"], vote_protocol: "v1", tally: "v0", vote_server: "v1"},
    rounds: {}}' >"$CONFIG_DIR/dynamic-voting-config.json"

for entry in "${active_rounds[@]}"; do
  IFS=$'\t' read -r round_id_b64 ea_pk <<<"$entry"
  round_id=$(printf '%s' "$round_id_b64" | base64 --decode | xxd -p -c 256)
  [[ "$round_id" =~ ^[0-9a-f]{64}$ ]] || { echo "invalid round id from svoted" >&2; exit 1; }
  "$VOTING_CONFIG_BIN" sign \
    --round-id "$round_id" --ea-pk "$ea_pk" --signer-id "$signer_id" \
    --privkey-file "$key_file" \
    --pir-depth "$(jq -r '.pir_depth' <<<"$layout")" \
    --tier0-layers "$(jq -r '.tier0_layers' <<<"$layout")" \
    --tier1-layers "$(jq -r '.tier1_layers' <<<"$layout")" \
    --poly-len "$(jq -r '.poly_len' <<<"$layout")" \
    --merge "$CONFIG_DIR/dynamic-voting-config.json"
done

jq -n --arg dynamic_url "$PUBLIC_BASE_URL/dynamic-voting-config.json" \
  --argjson trusted_key "$trusted_key" \
  '{static_config_version: 1, dynamic_config_url: $dynamic_url, trusted_keys: [$trusted_key]}' \
  >"$CONFIG_DIR/static-voting-config.json"

"$VOTING_CONFIG_BIN" verify \
  --config "$CONFIG_DIR/dynamic-voting-config.json" \
  --static-config "$CONFIG_DIR/static-voting-config.json" --json

sha256=$(sha256sum "$CONFIG_DIR/static-voting-config.json" | awk '{print $1}')
printf 'VOTING_CONFIG_URL=%s/static-voting-config.json?checksum=sha256:%s\n' \
  "$PUBLIC_BASE_URL" "$sha256" >"$CONFIG_DIR/config.env"
echo "signed configuration written to $CONFIG_DIR"
