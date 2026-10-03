#!/usr/bin/env bash
set -euo pipefail

# Wait for the real svoted round to expire and for its automatic tally
# pipeline to publish decrypted totals. This uses the chain's gRPC API rather
# than constructing a tally locally.

SVOTED_GRPC=${SVOTED_GRPC:-127.0.0.1:9190}
GRPCURL_BIN=${GRPCURL_BIN:-grpcurl}
VOTE_SDK_PROTO_ROOT=${VOTE_SDK_PROTO_ROOT:?VOTE_SDK_PROTO_ROOT must be set}

query() {
  "$GRPCURL_BIN" -plaintext \
    -import-path "$VOTE_SDK_PROTO_ROOT" -proto svote/v1/query.proto \
    -d "$2" "$SVOTED_GRPC" "$1"
}

rounds=$(query svote.v1.Query/ListRounds '{}')
if [[ -n "${E2E_VOTED_ROUND_ID:-}" ]]; then
  voted_round_id_b64=$(printf '%s' "$E2E_VOTED_ROUND_ID" | xxd -r -p | base64 | tr -d '\n')
  round=$(jq -ce --arg round_id "$voted_round_id_b64" \
    '.rounds[] | select(.voteRoundId == $round_id)' <<<"$rounds")
else
  round=$(jq -ce '.rounds[] | select(.title == "E2E Round 1")' <<<"$rounds")
fi
round_id=$(jq -er '.voteRoundId' <<<"$round")
vote_end_time=$(jq -er '.voteEndTime | tonumber' <<<"$round")

while (( $(date +%s) <= vote_end_time )); do
  sleep 5
done

request=$(jq -cn --arg round_id "$round_id" '{vote_round_id: $round_id}')
for attempt in $(seq 1 30); do
  round_status=$(query svote.v1.Query/VoteRound "$request")
  if jq -e '.round.status == "SESSION_STATUS_FINALIZED"' <<<"$round_status" >/dev/null; then
    tally=$(query svote.v1.Query/TallyResults "$request")
    # Session finalization is committed before the asynchronous tally results
    # become queryable. Keep polling through that short interval.
    if ! jq -e '.results | type == "array" and length > 0' <<<"$tally" >/dev/null; then
      echo "svoted finalized the session; waiting for tally results"
      sleep 1
      continue
    fi
    echo "Raw svoted TallyResults response:"
    jq . <<<"$tally"
    # ZKool's voting circuit encrypts the number of 0.125 ZEC ballots, not
    # raw zatoshi. svoted currently calls this field `total_value` and labels
    # it zatoshi in its protobuf, but the decrypted value is a ballot count.
    echo "Finalized svoted tally (0.125 ZEC ballots):"
    jq -r '
      def proposal: (.proposalId // .proposal_id // 0 | tonumber);
      def choice: (.voteDecision // .vote_decision // 0 | tonumber);
      def total: (.totalValue // .total_value // 0 | tonumber);
      .results
      | sort_by([proposal, choice])[]
      | "proposal=\(proposal) choice=\(choice) ballots=\(total) zatoshi=\(total * 12500000)"
    ' <<<"$tally"
    jq -e '
      def proposal: (.proposalId // .proposal_id // 0 | tonumber);
      def choice: (.voteDecision // .vote_decision // 0 | tonumber);
      def total: (.totalValue // .total_value // 0 | tonumber);
      [.results[]
       | { proposal: proposal, choice: choice, total: total }]
       | sort_by([.proposal, .choice]) == [
          { proposal: 1, choice: 0, total: 499 },
          { proposal: 3, choice: 2, total: 499 }
        ]
    ' <<<"$tally" >/dev/null
    echo "svoted finalized the real vote with 499 ballots for choices 0 and 2; proposal 2 was skipped"
    exit 0
  fi
  sleep 1
done

echo "the submitted E2E vote did not finalize its tally" >&2
exit 1
