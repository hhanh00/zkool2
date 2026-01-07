---
title: Transaction History
---

```graphql
query {
  transactionsByAccount(idAccount: 1) {
    txid
    time
    value
    notes { pool value }
  }
}
```

```json
{
  "data": {
    "transactionsByAccount": [
      {
        "txid": "39fbca22ca0bb38cad862d260e0c4589a79d7873ea42756aa80ac89a0b77b99d",
        "time": "2026-01-02T16:51:34",
        "value": "-0.00010000",
        "notes": [
          {
            "pool": 2,
            "value": "0.00035000"
          },
          {
            "pool": 2,
            "value": "0.00040157"
          }
        ]
      },
      {
        "txid": "224cd18bb829debf623baaa614360caa6dde0e12cca6081f7827f3571b051c7a",
        "time": "2026-01-02T16:44:36",
        "value": "-0.00010000",
        "notes": [
          {
            "pool": 2,
            "value": "0.00040157"
          },
          {
            "pool": 2,
            "value": "0.00045000"
          }
        ]
      },
      {
        "txid": "773039284130067ec1ec0c4cd8d134a593f30f99b2c54c20c3b76d9ad1566486",
        "time": "2026-01-02T16:40:58",
        "value": "-0.00010000",
        "notes": [
          {
            "pool": 2,
            "value": "0.00040157"
          },
          {
            "pool": 2,
            "value": "0.00055000"
          }
        ]
      },
      {
        "txid": "08d162c85e9f3e56e1fe30bce8a2aa08e7c83200d079817f9b417d5fde2afda2",
        "time": "2026-01-02T16:36:49",
        "value": "-0.00010000",
        "notes": [
          {
            "pool": 2,
            "value": "0.00040157"
          },
          {
            "pool": 2,
            "value": "0.00065000"
          }
        ]
      },
      {
        "txid": "4975f4c818f8c33c128423b426075e697aa839036db5af346e2183ca4dab7ea2",
        "time": "2026-01-02T12:53:55",
        "value": "-0.00010000",
        "notes": [
          {
            "pool": 2,
            "value": "0.00040157"
          },
          {
            "pool": 2,
            "value": "0.00075000"
          }
        ]
      },
      {
        "txid": "7e448c44e8610d451e18e3cc88b8c46b07637a8e2fc6dd5b80690074b061ef8d",
        "time": "2026-01-02T12:49:27",
        "value": "-0.00020000",
        "notes": [
          {
            "pool": 2,
            "value": "0.00040157"
          },
          {
            "pool": 2,
            "value": "0.00085000"
          }
        ]
      },
      {
        "txid": "587e2f83b3b1a2f7e7bf54aedd548f19b3d1cb10a9c0b806e18ce17bf7654008",
        "time": "2026-01-02T12:35:27",
        "value": "-0.00015000",
        "notes": [
          {
            "pool": 0,
            "value": "0.00040157"
          },
          {
            "pool": 2,
            "value": "0.00014843"
          }
        ]
      },
      {
        "txid": "8cf7fd5e98db790fec62fe1e740225380f31e71d7512c1cf96c5b6ed67ca7fec",
        "time": "2026-01-01T16:14:13",
        "value": "-0.00015000",
        "notes": [
          {
            "pool": 2,
            "value": "0.00070000"
          },
          {
            "pool": 0,
            "value": "0.00090157"
          }
        ]
      },
      {
        "txid": "ee5fc4ccbc7b586ea814016ce588fa13b48d71185033c8b57d1f96b869f70e7a",
        "time": "2026-01-01T16:00:10",
        "value": "-0.00015000",
        "notes": [
          {
            "pool": 0,
            "value": "0.00090157"
          },
          {
            "pool": 2,
            "value": "0.00085000"
          }
        ]
      },
      {
        "txid": "9ec52927528d03b5840796760096ce43c53f1e071b79431a813643595d185327",
        "time": "2026-01-01T14:41:33",
        "value": "0.00190157",
        "notes": [
          {
            "pool": 2,
            "value": "0.00190157"
          }
        ]
      }
    ]
  }
}
```

:::info
Transactions can be filtered by height.
:::
