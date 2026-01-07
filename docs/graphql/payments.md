---
title: Sending Funds
---

```graphql
mutation {
  pay(idAccount: 1,
  payment:  {
     recipients:  [{
        address: "u1p0j6xazlcmwsxq0z8rcavyx4edgrsqwc5aj4efsdxn8adc2url0eypkttd0608hfdx592jxt78ar062urudm3wctx72vg89v25zktnpp"
        amount: "0.00040157"
     }]
     srcPools: 7
  })
}
```

This mutation makes a payment from account 1 to the address `u1p0...`
for an amount of 0.00040157 ZEC. The transaction can use any source of funds
available regardless of the pool because of the srcPools is set at 7.

You can send to more than one recipient with the `pay` mutation.

Other flags are:

- recipientPaysFee: true if the transaction fee should be deducted from the
  amount sent out

> The transaction builder will prioritize using shielded funds, but may use
> transparent funds if allowed by the srcPools value. To force a t2t
> transaction, set srcPools to 1 (See [Pools](./account.md#pools)).
