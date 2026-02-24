#!/bin/sh

# Start zkool_graphql with -j ../example/sh/test-jwt-secret
# to enable JWT authorization
# Create a token with the following Claim
# {
#   "sub": account_id,
#   "exp": expiration_timestamp
# }
# Admin commands need to have account_id = 0

ADMIN_TOKEN=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE4MDM0NjM4ODksInN1YiI6MH0.r-3XHaAifOmdSusOsSiMgkQV0qpfh4DHUQX0wn9MHos
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
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d @- | jq -r .data.createAccount)
echo $ACCOUNT_ID

USER_TOKEN=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE4MDM0NjM4ODksInN1YiI6MX0.sg7JuWtFqgNeZMk10C3-J0n__a9MLmb9_14oBsSHyeU
GET_UA='
query ($idAccount: Int!) {
    addressByAccount(idAccount: $idAccount) {
    ua
    }
}'
UA=$(jq -n --arg q "$GET_UA" --argjson id "$ACCOUNT_ID" '{query: $q, variables: {idAccount: $id}}' | \
curl -s -X POST http://localhost:8000/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $USER_TOKEN" \
  -d @- | jq -r .data.addressByAccount.ua )
echo $UA
