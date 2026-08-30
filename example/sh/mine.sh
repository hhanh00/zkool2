#!/usr/bin/env bash
set -euo pipefail
set -x

# Number of blocks to mine. The note-migration tests need a chain that stops
# below the Ironwood (NU6.3) activation height in misc/zebra.toml, because
# after activation there is no pool-restricted address — addressByAccount
# returns the same string for orchard and ironwood — so a payment lands in
# Ironwood and legacy Orchard notes can no longer be created. They pass 150:
# above coinbase maturity (100), below activation (250).
BLOCKS=${BLOCKS:-300}

# Whether to shield the miner's coinbase to DESTINATION_ADDRESS afterwards.
# Set to 0 for callers that fund their own accounts; the shielding step below
# requires Ironwood to be active and so cannot run on a pre-activation chain.
FUND=${FUND:-1}

sed -i -e "s#miner_address = \"\"#miner_address = \"${MINER_ADDRESS}\"#" misc/zebra.toml
nohup zebrad -c misc/zebra.toml start > zebrad.log 2>&1 & disown
sleep 60
tail zebrad.log
nohup lightwalletd --no-tls-very-insecure --data-dir=./data/regtest --grpc-bind-addr=127.0.0.1:8137 --zcash-conf-path=./misc/zebra.conf --log-file=/dev/stdout &
nohup zkool_graphql -d regtest.db -l http://localhost:8137 -n &
sleep 60

curl --data-binary "{\"jsonrpc\": \"1.0\", \"id\":\"curltest\", \"method\": \"generate\", \"params\": [${BLOCKS}] }" -H 'Content-type: application/json' http://127.0.0.1:18232/

GRAPHQL_URL="http://localhost:8000/graphql"
MATURITY_THRESHOLD=100
MAX_NOTES=10

gql() {
    local query="$1"
    local variables="$2"
    curl -sf "$GRAPHQL_URL" \
        -H "Content-Type: application/json" \
        -d "$(jq -n --argjson vars "$variables" --arg q "$query" \
            '{query: $q, variables: $vars}')" \
        | jq -r '.data'
}

height() {
  gql "query { currentHeight }" {} | jq -r '.currentHeight'
}

wait_zaino() {
  local target=$(($1 + $2))
  while (( $(height) < $target )); do
    sleep 1
  done
}

# Wait for lightwalletd to ingest what was just mined. currentHeight is
# served through it, so this also confirms the whole stack is answering.
for _ in $(seq 1 120); do
  H=$(height || true)
  case "$H" in '' | null) H=0 ;; esac
  if [ "$H" -ge "$BLOCKS" ]; then
    break
  fi
  sleep 2
done

# Get current height
HEIGHT=$(height)
echo "Height: $HEIGHT"

if [ "$FUND" != "1" ]; then
  echo "FUND=0: leaving the chain unfunded at height $HEIGHT"
  pkill zkool_graphql
  exit 0
fi

# Create miner account
MINER=$(gql 'mutation CreateAccount($account: NewAccount!) {
    createAccount(newAccount: $account)
}' "$(jq -n \
    --arg key "$MINER_SEED" \
    '{account: {name: "miner", key: $key, aindex: 0, birth: 1, useInternal: false}}')" \
    | jq -r '.createAccount')
echo "Miner id: $MINER"

# Create wallet account
WALLET=$(gql 'mutation CreateAccount($account: NewAccount!) {
    createAccount(newAccount: $account)
}' "$(jq -n \
    --arg key "$SEED" \
    '{account: {name: "wallet", key: $key, aindex: 0, birth: 1, useInternal: false}}')" \
    | jq -r '.createAccount')
echo "Wallet id: $WALLET"

# Synchronize both accounts
gql 'mutation Synchronize($ids: [Int!]!) { synchronize(idAccounts: $ids) }' \
    "$(jq -n --argjson m "$MINER" --argjson w "$WALLET" '{ids: [$m, $w]}')" > /dev/null

# Get mature notes
MATURE_HEIGHT=$((HEIGHT - MATURITY_THRESHOLD))
NOTES=$(gql 'query NotesByAccount($id: Int!) {
    notesByAccount(idAccount: $id) { id height value }
}' "$(jq -n --argjson id "$MINER" '{id: $id}')" \
    | jq --argjson mh "$MATURE_HEIGHT" --argjson max "$MAX_NOTES" \
        '[.notesByAccount[] | select(.height < $mh)] | .[:$max]')

NOTE_COUNT=$(echo "$NOTES" | jq 'length')
if [ "$NOTE_COUNT" -eq 0 ]; then
    echo "Error: No sufficiently mature notes found." >&2
    exit 1
fi
echo "Selected $NOTE_COUNT mature note(s)"

# Sum note values
TOTAL=$(echo "$NOTES" | jq '[.[].value | tonumber] | add')
echo "Total: $TOTAL"

# Pay
TXID=$(gql 'mutation Pay($id: Int!, $payment: Payment!) {
    pay(idAccount: $id, payment: $payment)
}' "$(jq -n \
    --argjson id "$MINER" \
    --arg addr "$DESTINATION_ADDRESS" \
    --arg total "$TOTAL" \
    --argjson thresh "$MATURITY_THRESHOLD" \
    '{id: $id, payment: {
        recipients: [{address: $addr, amount: $total}],
        recipientPaysFee: true,
        confirmations: $thresh
    }}')" | jq -r '.pay')
echo "Done. txid: $TXID"

HEIGHT=$(height)
curl --data-binary '{"jsonrpc": "1.0", "id":"curltest", "method": "generate", "params": [10] }' -H 'Content-type: application/json' http://127.0.0.1:18232/
wait_zaino $HEIGHT 10

# Sync wallet
gql 'mutation Synchronize($id: Int!) { synchronizeAccount(idAccount: $id) }' \
    "$(jq -n --argjson id "$WALLET" '{id: $id}')" > /dev/null

# Check balance
BALANCE=$(gql 'query GetBalance($id: Int!) {
    balanceByAccount(idAccount: $id) { transparent orchard ironwood }
}' "$(jq -n --argjson id "$WALLET" '{id: $id}')")
echo "$BALANCE"

IRONWOOD=$(echo "$BALANCE" | jq -r '.balanceByAccount.ironwood | tonumber')
if (( $(echo "$IRONWOOD <= 0" | bc -l) )); then
    echo "Error: Expected positive orchard balance, got $IRONWOOD" >&2
    exit 1
fi

pkill zkool_graphql
