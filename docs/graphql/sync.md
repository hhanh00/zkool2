---
title: Synchronization
---

Synchronize one or several accounts.

```graphql
mutation {
  synchronize(idAccounts: [1 2 3])
}
```

- idAccounts (required): *List* of account IDs to synchronize

:::important
Batching synchronization is much more efficient than synchronizing
accounts one by one.
:::
