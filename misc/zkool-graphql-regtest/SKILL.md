---
name: zkool-graphql-regtest
description: "Set up a Zkool GraphQL server on Zcash regtest, restore a faucet account, create a test account, and fund it."
---

# Zkool GraphQL Regtest Setup

Spin up a Zkool GraphQL Docker container, restore a faucet account from seed, create a test account, and send test funds.

## Prerequisites

- Docker running (Colima or Docker Desktop)
- Docker Hub image `hhanh00/zkool-graphql:latest`
- Faucet seed phrase

## Workflow

### 1. Pull the image

```bash
docker pull hhanh00/zkool-graphql:latest
```

### 2. Run the container

```bash
docker rm -f zkool-graphql 2>/dev/null
docker run -d \
  --name zkool-graphql \
  -v zkool-data:/data \
  -p 8080:8080 \
  hhanh00/zkool-graphql:latest \
  --db-path /data/zsa-regtest \
  --lwd-url https://zsa.methyl.cc \
  --port 8080
```

Verify:

```bash
curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/
# Expect 405 (normal for GraphQL endpoint without query)
```

### 3. Restore the Faucet account

```bash
curl -s -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation($new: NewAccount!) { createAccount(newAccount: $new) }",
    "variables": {
      "new": {
        "name": "Faucet",
        "key": "equal clock rain latin plastic toss scrub modify clarify fold armor exchange gesture erase habit plug state forward demise demand limb risk only document",
        "passphrase": "",
        "aindex": 0,
        "birth": 5,
        "pools": 7,
        "useInternal": false
      }
    }
  }'
```

**Pool bitmask:** `1`=transparent, `2`=sapling, `4`=orchard. `7` = all pools.

Verify and get addresses:

```bash
curl -s -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ accounts { id name seed aindex dindex birth height balance } addressByAccount(idAccount: 1) { transparent sapling orchard ua } }"}'
```

### 4. Sync the Faucet

```bash
curl -s -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation($id: Int!) { synchronizeAccount(idAccount: $id, fast: false) }",
    "variables": { "id": 1 }
  }'
```

Check balance:

```bash
curl -s -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ balanceByAccount(idAccount: 1) { height transparent sapling orchard total } }"}'
```

### 5. Create a test account

```bash
curl -s -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation($new: NewAccount!) { createAccount(newAccount: $new) }",
    "variables": {
      "new": {
        "name": "<Test Account Name>",
        "key": "",
        "passphrase": "",
        "aindex": 0,
        "birth": 5,
        "pools": 7,
        "useInternal": false
      }
    }
  }'
```

Get the test account's transparent address:

```bash
curl -s -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ addressByAccount(idAccount: <ACCOUNT_ID>) { transparent } }"}'
```

### 6. Send funds from Faucet to test account

```bash
curl -s -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation($id: Int!, $pay: Payment!) { pay(idAccount: $id, payment: $pay) }",
    "variables": {
      "id": <FAUCET_ID>,
      "pay": {
        "recipients": [{
          "address": "<TEST_TRANSPARENT_ADDRESS>",
          "amount": "1"
        }],
        "srcPools": <POOL_BITMASK>
      }
    }
  }'
```

Returns a txid (string).

### 7. Check transaction confirmation

```bash
# Unconfirmed
curl -s -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ unconfirmedByAccount(idAccount: <ACCOUNT_ID>) { txid } }"}'

# Confirmed transactions
curl -s -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ transactionsByAccount(idAccount: <ACCOUNT_ID>, height: 0) { id txid height } }"}'

# Balances
curl -s -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ balanceByAccount(idAccount: <ACCOUNT_ID>) { height transparent sapling orchard total } }"}'
```

**Regtest note:** Transactions sit in the mempool until the node mines a block that includes them. This may take a few minutes.

## GraphQL API Reference

### Queries

| Query | Description |
|---|---|
| `accounts(accountFilter)` | List accounts |
| `balanceByAccount(idAccount, height)` | Get balance breakdown |
| `addressByAccount(idAccount, pools)` | Get addresses per pool |
| `transactionsByAccount(idAccount, height)` | List transactions |
| `unconfirmedByAccount(idAccount)` | Pending transactions |
| `assets(idAccount)` | List issued ZSA assets |
| `currentHeight` | Latest chain height |

### Mutations

| Mutation | Returns | Description |
|---|---|---|
| `createAccount(newAccount)` | `Int` | Restore an account (returns ID) |
| `deleteAccount(idAccount)` | `Boolean` | Delete an account |
| `editAccount(idAccount, updateAccount)` | `Boolean` | Update name/birth |
| `synchronizeAccount(idAccount, fast)` | `Int` | Sync to chain tip |
| `pay(idAccount, payment)` | `String` | Send ZEC (returns txid) |
| `issueAsset(idAccount, assetName, amount, firstIssuance, finalize)` | `String` | Issue a ZSA asset |
| `setAssetName(idAsset, name)` | `String` | Rename asset |
| `newAddresses(idAccount)` | `Addresses` | Generate new addresses |

### Input Types

**NewAccount:** `name!`, `key!` (seed), `passphrase`, `aindex!` (usually `0`), `birth`, `pools` (bitmask), `useInternal!`

**Payment:** `recipients!` (list), `srcPools` (bitmask), `recipientPaysFee`, `confirmations`

**Recipient:** `address!`, `amount!` (string in ZEC), `memo`, `assetDesc` (for ZSA transfers)

**UpdateAccount:** `name`, `birth`

## Safety Notes

- Server runs **without authentication** — restrict to localhost/LAN.
- Seed phrases stored in plaintext in the container's database.
- Transparent addresses only generated when `pools` includes `1`.

## Remarks
- When polling for transaction confirmation, synchronize the receiving account to update its balance and transaction history.
