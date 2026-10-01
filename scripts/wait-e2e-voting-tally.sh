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
round=$(jq -ce '.rounds[] | select(.title == "E2E Round 1")' <<<"$rounds")
round_id=$(jq -er '.voteRoundId' <<<"$round")
vote_end_time=$(jq -er '.voteEndTime | tonumber' <<<"$round")

while (( $(date +%s) <= vote_end_time )); do
  sleep 5
done

request=$(jq -cn --arg round_id "$round_id" '{vote_round_id: $round_id}')
for attempt in $(seq 1 120); do
  round_status=$(query svote.v1.Query/VoteRound "$request")
  if jq -e '.round.status == "SESSION_STATUS_FINALIZED"' <<<"$round_status" >/dev/null; then
    tally=$(query svote.v1.Query/TallyResults "$request")
    echo "Raw svoted TallyResults response:"
    jq . <<<"$tally"
    echo "Finalized svoted tally (zatoshi):"
    jq -r '
      def proposal: (.proposalId // .proposal_id // 0 | tonumber);
      def choice: (.voteDecision // .vote_decision // 0 | tonumber);
      def total: (.totalValue // .total_value // 0 | tonumber);
      .results
      | sort_by([proposal, choice])[]
      | "proposal=\(proposal) choice=\(choice) total_zatoshi=\(total)"
    ' <<<"$tally"
    jq -e '
      def proposal: (.proposalId // .proposal_id // 0 | tonumber);
      def choice: (.voteDecision // .vote_decision // 0 | tonumber);
      def total: (.totalValue // .total_value // 0 | tonumber);
      [.results[]
       | select(choice == 0 and total > 0)
       | proposal]
      | sort == [1, 2, 3]
    ' <<<"$tally" >/dev/null
    echo "svoted finalized the real vote with a positive tally for choice 0 of all proposals"
    exit 0
  fi
  sleep 1
done

echo "E2E Round 1 did not finalize its tally" >&2
exit 1
