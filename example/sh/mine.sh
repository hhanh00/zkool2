#!/usr/bin/env bash
set -euo pipefail
set -x

sed -i -e "s#miner_address = \"\"#miner_address = \"${MINER_ADDRESS}\"#" misc/zebra.toml
nohup zebrad -c misc/zebra.toml start > zebrad.log 2>&1 & disown
sleep 60
tail zebrad.log
nohup lightwalletd --no-tls-very-insecure --data-dir=./data/regtest --grpc-bind-addr=127.0.0.1:8137 --zcash-conf-path=./misc/zebra.conf --log-file=/dev/stdout &
nohup zkool_graphql -d regtest.db -l http://localhost:8137 -n &
sleep 60

curl --data-binary '{"jsonrpc": "1.0", "id":"curltest", "method": "generate", "params": [200] }' -H 'Content-type: application/json' http://127.0.0.1:18232/

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

# Get current height
HEIGHT=$(height)
echo "Height: $HEIGHT"

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
    balanceByAccount(idAccount: $id) { orchard }
}' "$(jq -n --argjson id "$WALLET" '{id: $id}')")
echo "$BALANCE"

ORCHARD=$(echo "$BALANCE" | jq -r '.balanceByAccount.orchard | tonumber')
if (( $(echo "$ORCHARD <= 0" | bc -l) )); then
    echo "Error: Expected positive orchard balance, got $ORCHARD" >&2
    exit 1
fi

pkill zkool_graphql
