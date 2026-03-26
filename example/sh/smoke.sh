#!/usr/bin/env bash
set -euo pipefail
set -x
GRAPHQL_URL="http://localhost:8000/graphql"

height() {
  gq $GRAPHQL_URL -q '
  query h {
    currentHeight
  }' | jq -r '.data.currentHeight'
}

wait_zaino() {
  local target=$(($1 + $2))
  while (( $(height) < $target )); do
    sleep 1
  done
}

echo "Test transfer to new wallet"

echo "Import funded wallet"
WALLET=$(gq $GRAPHQL_URL \
  -q 'mutation ($main: String!) {
  createAccount(newAccount: {
    name: "Main"
    key: $main
    aindex: 0
    useInternal: false
    birth: 1
  })
}' -v "main=$SEED" | jq -r '.data.createAccount')

echo "Synchronize"
gq $GRAPHQL_URL \
-q 'mutation ($account: Int!) {
  synchronizeAccount(idAccount: $account)
}' -v "account=$WALLET" | jq -r '.data.synchronizeAccount'

echo "Create new wallet"
A2=$(gq $GRAPHQL_URL \
  -q 'mutation {
  createAccount(newAccount: {
    name: "A"
    key: ""
    aindex: 0
    useInternal: false
    birth: 1
  })
}' | jq -r '.data.createAccount')

echo "Get new wallet address"
ADDRESS=$(gq $GRAPHQL_URL \
  -q 'query ($account: Int!) {
  addressByAccount(idAccount: $account) {
    orchard
  }
}' -v "account=$A2" | jq -r '.data.addressByAccount.orchard')

echo "Synchronize"
gq $GRAPHQL_URL \
-q 'mutation ($accounts: [Int!]!) {
  synchronize(idAccounts: $accounts)
}' -v "accounts=[$WALLET,$A2]" | jq -r '.data.synchronize'

echo "Show funding balance"
gq $GRAPHQL_URL \
-q 'query ($account: Int!) {
  balanceByAccount(idAccount: $account) {
    orchard
  }
}' -v "account=$WALLET" | jq -r '.data.balanceByAccount.orchard'

echo "Send funds"
TXID=$(gq $GRAPHQL_URL \
-q 'mutation ($account: Int!, $address: String!, $amount: BigDecimal!){
  pay(idAccount: $account
  payment:  {
    recipients:  [{
      address: $address
      amount: $amount
    }]
  })
}' -v "account=$WALLET" -v "address=$ADDRESS" \
-v 'amount=10.5' | jq -r '.data.pay')

echo "Mine a few blocks"
HEIGHT=$(height)
curl --data-binary '{"jsonrpc": "1.0", "id":"curltest", "method": "generate", "params": [10] }' -H 'Content-type: application/json' http://127.0.0.1:18232/
wait_zaino $HEIGHT 10

echo "Synchronize"
gq $GRAPHQL_URL \
-q 'mutation ($account: Int!) {
  synchronizeAccount(idAccount: $account)
}' -v "account=$A2" | jq -r '.data.synchronizeAccount'

echo "Show new wallet balance"
FINAL_BAL=$(gq $GRAPHQL_URL \
-q 'query ($account: Int!) {
  balanceByAccount(idAccount: $account) {
    orchard
  }
}' -v "account=$A2" | jq -r '.data.balanceByAccount.orchard')

echo $FINAL_BAL

if [ "$FINAL_BAL" != "10.50000000" ]; then
    echo "Error: expected 10.50000000, got $FINAL_BAL" >&2
    exit 1
fi
