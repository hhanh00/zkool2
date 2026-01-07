---
title: Balance
---

## Current Balance

```graphql
query {
  balanceByAccount(idAccount: 1) {
    transparent
    sapling
    orchard
    total
  }
}
```

Get the balance of an account. You must specify which pools you want.

```json
{
  "data": {
    "balanceByAccount": {
      "transparent": "0",
      "sapling": "0",
      "orchard": "0.00075157",
      "total": "0.00075157"
    }
  }
}
```

## Past Balance

You can ask for the balance at a given height in the past.

```graphql
query {
  balanceByAccount(idAccount: 1 height: 3100000) {
    height
    transparent
    sapling
    orchard
    total
  }
}
```

## Notes

```graphql
query {
  notesByAccount(idAccount: 1) {
    height value pool tx { txid }
  }
}
```

```json
{
  "data": {
    "notesByAccount": [
      {
        "height": 3190954,
        "value": "0.00035000",
        "pool": 2,
        "tx": {
          "txid": "39fbca22ca0bb38cad862d260e0c4589a79d7873ea42756aa80ac89a0b77b99d"
        }
      },
      {
        "height": 3190954,
        "value": "0.00040157",
        "pool": 2,
        "tx": {
          "txid": "39fbca22ca0bb38cad862d260e0c4589a79d7873ea42756aa80ac89a0b77b99d"
        }
      }
    ]
  }
}
```

:::info
This is an example of a query that uses a connection (or edge) between objects.
The note refers to a Transaction. You can query the transaction fields that
correspond to the notes or leave the transaction out of the results.
:::
