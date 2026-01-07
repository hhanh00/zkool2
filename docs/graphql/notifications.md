---
title: Notifications / Subscriptions
---

If your client supports it, it can subscribe to events sent by the server
when a new block is received or when new transactions that affect a given
account are detected in the mempool.

```graphql
subscription {
  events(idAccount: 1) {
    type
    txid
    height
  }
}
```

Subscribe to events for account 1. Events are either:

- new block for which height is set,
- new transaction for which the txid is set
- type indicates the type of the event BLOCK or TX
