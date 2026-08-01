---
title: Testnet
---

# Testnet

The ZSA testnet is a distinct public network dedicated to testing ZSA. It is
not the regular Zcash testnet. It runs a ZSA-enabled version of Zebra and
lightwalletd, with NU 6.2 activated (but
not Ironwood). Anyone can issue custom assets, transfer them between
shielded addresses, and finalize them, using test funds that have no real
value. Addresses use the regtest prefix `uregtest1`, not the mainnet `u1`.

## Purpose

The ZSA testnet exists so application developers can build on ZSA and
experiment with it.

## Ecosystem for app builder

- **Faucet**: dispenses free tZEC test funds, so developers always have funds
  to issue assets and pay fees. Currently running on the ZSA testnet network at
  `faucet.zsa.methyl.cc`.
- **Wallet (zkool)**: the reference wallet, ready to issue and transfer
  assets. Use a database name containing `zsa` and connect it to a ZSA
  testnet server `https://zsa.methyl.cc`.
- **GraphQL API**: `zkool_graphql` exposes wallet functionality (accounts,
  issuance, payments) over GraphQL, so applications can integrate with ZSA
  directly. See the [GraphQL server build](graphql/build.md) guide.
