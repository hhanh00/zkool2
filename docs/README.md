---
home: true
heroText: Zkool
tagline: The swiss-army wallet for zcash
actions:
    - text: Get Started
      link: /guide/start.md
features:
    - title: Private
      details: |
        Full Zcash support with shielded transactions.
    - title: Flexible
      details: |
        Works across desktop and mobile.
        Wallet and Accounts are interoperable.
    - title: Secure
      details: |
        Your keys stay with you.
---

## Features

- supports most zcash keys[^1]
- works with transparent, sapling and orchard pools[^2],
- generates diversified shielded addresses and rotates transparent address,
- manages multiple accounts, folders and wallets[^3],
- connects to mainnet, testnet and regtest,
- wallets and accounts are interoperable between mobile and desktop versions
- speaks the light nodes[^4] and full nodes[^5] protocols
- syncs faster than any other wallet app[^6]
- encrypts communication over TOR[^7]

It can probably handle any account from any other wallets as long as you have
the secret key (unless it is Sprout)

::: warning
Zkool gives you full control over your funds. It is the only wallet app that
works in any situation, but it is also harder to use. Other wallets are much
more conservative and will make sure you stay on the established path.
:::

[^1]: 12 to 24 word mnemonics, passphrase and account index. Sapling legacy keys, viewing keys, unified keys, xpub, etc.
[^2]: all pools except Sprout
[^3]: with AES encryption
[^4]: lightwalletd and zaino
[^5]: zcashd and zebrad
[^6]: including Ywallet
[^7]: via exit nodes and onion services
