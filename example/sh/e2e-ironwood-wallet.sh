#!/usr/bin/env bash
set -euo pipefail

GRAPHQL_URL=${GRAPHQL_URL:-http://127.0.0.1:8000/graphql}
LIGHTWALLETD_URL=${LIGHTWALLETD_URL:-http://127.0.0.1:8137}
# ZKool infers the network from the database path; include "regtest".
WALLET_DB=${WALLET_DB:-e2e-ironwood-regtest-wallet.db}
SEED=${SEED:?SEED must be set}
LOG_FILE=${LOG_FILE:-e2e-zkool-graphql.log}
ZKOOL_GRAPHQL_BIN=${ZKOOL_GRAPHQL_BIN:-zkool_graphql}
VOTING_CONFIG_URL=${VOTING_CONFIG_URL:-}

server_pid=""
start_server() {
  "$ZKOOL_GRAPHQL_BIN" -d "$WALLET_DB" -l "$LIGHTWALLETD_URL" -n >"$LOG_FILE" 2>&1 &
  server_pid=$!
}

wait_for_server() {
  for attempt in $(seq 1 120); do
    if curl -sf "$GRAPHQL_URL" \
      -H 'Content-Type: application/json' \
      --data-binary '{"query":"query { currentHeight }"}' \
      | jq -e '.data.currentHeight > 0' >/dev/null 2>&1; then
      return
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
}

cleanup() {
  if [[ -n "$server_pid" ]]; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT

rm -f "$WALLET_DB"
start_server
wait_for_server

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

if [[ -n "$VOTING_CONFIG_URL" ]]; then
  gql 'mutation SetVotingConfigUrl($url: String!) {
    setVotingConfigUrl(url: $url)
  }' "$(jq -cn --arg url "$VOTING_CONFIG_URL" '{url: $url}')" \
    | jq -e '.setVotingConfigUrl == true' >/dev/null
fi

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

if [[ -n "$VOTING_CONFIG_URL" ]]; then
  rounds=$(gql 'query VotingRounds($id: Int!) {
    votingRounds(idAccount: $id) { roundId title proposals { proposalId options } }
  }' "$(jq -cn --argjson id "$account_id" '{id: $id}')")
  jq -e '
    (.votingRounds | length) == 2
    and all(.votingRounds[]; (.proposals | length) == 3)
    and all(.votingRounds[].proposals[]; (.options | length) == 3)
  ' <<<"$rounds" >/dev/null
  echo "ZKool resolved two signed voting rounds with three proposals and choices each"

  round_id=$(jq -er '.votingRounds[0].roundId' <<<"$rounds")
  selections=$(jq -cn --argjson round "$rounds" '
    $round.votingRounds[0].proposals
    | map({proposalId: .proposalId, choice: 0})
  ')
  submission=$(gql 'mutation SubmitVote($id: Int!, $round: String!, $selections: [VotingSelectionInput!]!) {
    submitVote(idAccount: $id, roundId: $round, selections: $selections) {
      roundId running completedProposals totalProposals remainingObligations
      sharesConfirmed sharesTotal failures
    }
  }' "$(jq -cn \
    --argjson id "$account_id" \
    --arg round "$round_id" \
    --argjson selections "$selections" \
    '{id: $id, round: $round, selections: $selections}')")
  jq -e '.submitVote.failures | length == 0' <<<"$submission" >/dev/null

  for attempt in $(seq 1 240); do
    status=$(gql 'query VotingSubmissionStatus($id: Int!, $round: String!) {
      votingSubmissionStatus(idAccount: $id, roundId: $round) {
        running completedProposals totalProposals remainingObligations
        sharesConfirmed sharesTotal failures
      }
    }' "$(jq -cn --argjson id "$account_id" --arg round "$round_id" '{id: $id, round: $round}')")
    jq . <<<"$status"
    if jq -e '
      .votingSubmissionStatus as $status
      | ($status.running | not)
      and $status.completedProposals == 3
      and $status.totalProposals == 3
      and $status.sharesTotal > 0
      and $status.sharesConfirmed == $status.sharesTotal
      and ($status.failures | length == 0)
    ' <<<"$status" >/dev/null; then
      echo "ZKool submitted a vote and all helper shares were confirmed"
      break
    fi
    if [[ "$attempt" == 240 ]]; then
      echo "vote submission or helper-share confirmation did not finish" >&2
      exit 1
    fi
    sleep 1
  done
fi
