---
title: Receiving Funds
---

## Addresses

```graphql
query {
  addressByAccount(idAccount: 1) {
    ua
    transparent
    orchard
  }
}
```

```json
{
  "data": {
    "addressByAccount": {
      "ua": "u18ayll5t9fv97nqldrtxgllk69snemz0xx9f2cv4l8h6ss9x29dqd0wgvthpn8u572zmk4tlrt56s6akzq3a0u9c0p7wmx57sgzsgt3973nmg2wltdeepe4l335uc6kvk6llu2kl9wnecdek6lazlch0gprg3vdumhlc7e6lzxyjcz70s",
      "transparent": "t1JiAXyZDse71mxeTmW5hymXWZzY5xRRock",
      "orchard": "u1p0j6xazlcmwsxq0z8rcavyx4edgrsqwc5aj4efsdxn8adc2url0eypkttd0608hfdx592jxt78ar062urudm3wctx72vg89v25zktnpp"
    }
  }
}
```

:::info
By default, the UA contains Sapling and Orchard receivers. To get a UA with a different set of receivers, use the pool parameter with a value calculated by adding:

- 1 for transparent,
- 2 for sapling,
- 4 for orchard.

:::

Ex: UA that includes every receiver

```graphql
query {
  addressByAccount(idAccount: 1 pools: 7) {
    ua
    transparent
    orchard
  }
}
```

## Diversified Addresses

To generate a new set of addresses:

```graphql
mutation {
  newAddresses(idAccount: 1) {
    ua
    transparent
    sapling
    orchard
  }
}
```

:::info
The previous addresses are still valid and can receive funds.
:::

## Unconfirmed Funds
