#!/usr/bin/env bash
set -euo pipefail

GRAPHQL_URL=${GRAPHQL_URL:-http://127.0.0.1:8000/graphql}
LIGHTWALLETD_URL=${LIGHTWALLETD_URL:-http://127.0.0.1:8137}
# ZKool infers the network from the database path; include "regtest".
WALLET_DB=${WALLET_DB:-e2e-ironwood-regtest-wallet.db}
SEED=${SEED:?SEED must be set}
LOG_FILE=${LOG_FILE:-e2e-zkool-graphql.log}
ZKOOL_GRAPHQL_BIN=${ZKOOL_GRAPHQL_BIN:-zkool_graphql}

server_pid=""
cleanup() {
  if [[ -n "$server_pid" ]]; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT

rm -f "$WALLET_DB"
"$ZKOOL_GRAPHQL_BIN" -d "$WALLET_DB" -l "$LIGHTWALLETD_URL" -n >"$LOG_FILE" 2>&1 &
server_pid=$!

for attempt in $(seq 1 120); do
  if curl -sf "$GRAPHQL_URL" \
    -H 'Content-Type: application/json' \
    --data-binary '{"query":"query { currentHeight }"}' \
    | jq -e '.data.currentHeight > 0' >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    echo "checked-out zkool_graphql exited during startup" >&2
    tail -200 "$LOG_FILE" >&2 || true
    exit 1
  fi
  if [[ "$attempt" == 120 ]]; then
    echo "checked-out zkool_graphql was not ready after 120 seconds" >&2
    tail -200 "$LOG_FILE" >&2 || true
    exit 1
  fi
  sleep 1
done

gql() {
  local query=$1
  local variables=$2
  local response
  response=$(curl -sf "$GRAPHQL_URL" \
    -H 'Content-Type: application/json' \
    --data-binary "$(jq -cn --arg query "$query" --argjson variables "$variables" \
      '{query: $query, variables: $variables}')")
  if ! jq -e '.errors == null' <<<"$response" >/dev/null; then
    jq . <<<"$response" >&2
    return 1
  fi
  jq -c '.data' <<<"$response"
}

account_id=$(gql 'mutation CreateAccount($account: NewAccount!) {
  createAccount(newAccount: $account)
}' "$(jq -cn --arg seed "$SEED" \
  '{account: {name: "e2e-voter", key: $seed, aindex: 0, birth: 1, useInternal: false}}')" \
  | jq -r '.createAccount')

gql 'mutation Synchronize($id: Int!) { synchronizeAccount(idAccount: $id) }' \
  "$(jq -cn --argjson id "$account_id" '{id: $id}')" >/dev/null

balance=$(gql 'query Balance($id: Int!) {
  balanceByAccount(idAccount: $id) { transparent orchard ironwood }
}' "$(jq -cn --argjson id "$account_id" '{id: $id}')")

jq . <<<"$balance"
jq -e '.balanceByAccount.ironwood | tonumber > 0' <<<"$balance" >/dev/null || {
  echo "checked-out ZKool did not discover the real Ironwood V3 note" >&2
  exit 1
}

echo "checked-out ZKool discovered a positive Ironwood balance"
