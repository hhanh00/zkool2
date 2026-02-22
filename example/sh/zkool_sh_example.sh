#!/bin/sh

CREATE_ACCOUNT='mutation {
    createAccount(newAccount: {
    name: "Test"
    useInternal: true
    key: ""
    aindex: 0
    })
}'
ACCOUNT_ID=$(jq -n --arg q "$CREATE_ACCOUNT" '{query: $q}' | \
curl -s -X POST http://localhost:8000/graphql \
  -H "Content-Type: application/json" \
  -d @- | jq -r .data.createAccount)

GET_UA='
query ($idAccount: Int!) {
    addressByAccount(idAccount: $idAccount) {
    ua
    }
}'
UA=$(jq -n --arg q "$GET_UA" --argjson id "$ACCOUNT_ID" '{query: $q, variables: {idAccount: $id}}' | \
curl -s -X POST http://localhost:8000/graphql \
  -H "Content-Type: application/json" \
  -d @- | jq -r .data.addressByAccount.ua )
echo $UA
