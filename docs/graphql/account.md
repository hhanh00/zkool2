---
title: Account Management
---

## Create

```graphql
mutation {
  createAccount(newAccount:  {
     name: "New Test Account"
     useInternal: true
     key: ""
     aindex: 0
  })
}
```

- name (required): Name of the account,
- useInternal (required): Enable compatibility with ZIP-316 (Zashi, Zingo) by
  separating the change in an *internal* address
- key (required): seed phrase, private key or viewing key. Leave empty for new
  accounts, a random seed phrase will be generated
- aindex (required): account index. Typically 0 unless you are deriving
  additional accounts from the same seed phrase
- birth: block height on the blockchain at which the account was created or
  first used. Highly recommended when recovering wallets because scanning skips
  blocks before the birth height
- passphrase: additional password associated with the seed phrase
- pools: See below [Pools](./account.md#pools)

Returns the account ID.

```json
{
  "data": {
    "createAccount": 1
  }
}
```

## Pools

The value of pool is the combination of transparent (1), sapling (2) and orchard
(4) pool enabled on this account. Add the values of each pool together. For
example, to enable sapling and orchard but not transparent, use 2 + 4 = 6. By
default, every pool is enabled.

## List Accounts

```graphql
query {
  accounts {
    id
    name
  }
}
```

## Account Info

```graphql
query {
  accountById(idAccount: 1) {
    name
    seed
  }
}
```

:::important
When you create a new account, do not forget to get the seed phrase
and make a backup.
:::

## Change Name / Birth Height

```graphql
mutation {
  editAccount(idAccount: 1 updateAccount:  {
     name: "New Name"
     birth: 3100000
  })
}
```

- name: New name
- birth: New birth height

Both fields are optional. Leave it out if you do not want to change its value.
> If you change the birth height, you probably need to reset and synchronize again.

## Reset

Clears the wallet synchronization data. You may want to [synchronize](./sync.md) afterward.

```graphql
mutation {
  resetAccount(idAccount: 1)
}
```

## Delete

```graphql
mutation {
  deleteAccount(idAccount: 1)
}
```
