---
title: Unconfirmed Funds
---

```graphql
query {
  unconfirmedByAccount(idAccount: 1) {
    txid
    value
  }
}
```

Returns the unconfirmed transactions that affect a given account.
