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
        Full Zcash support with shielded transactions
    - title: Flexible
      details: |
        Works across desktop and mobile /
        Interoperable Wallet and Accounts
    - title: Spending Reports
      details: |
        Income & Expense Charts /
        Categories
    - title: Zcash Tech
      details: |
        Transparent and Shielded Pools /
        Diversified Addresses /
    - title: Multi Accounts
      details: |
        Multi Signature FROST Accounts /
        Multi Accounts /
        Multi Databases
    - title: Secure
      details: |
        Encrypted Data Files /
        Your keys stay with you.

---

## Features

- supports most zcash keys[^1]
- works with transparent, sapling and orchard pools[^2],
- generates diversified shielded addresses and rotates transparent address,
- manages multiple accounts, folders and wallets[^3],
- supports multi signature shielded accounts by leveraging the FROST threshold signature algorithm
- connects to mainnet, testnet and regtest, via full nodes[^4] and lightwalletd servers[^5]
- wallets and accounts are interoperable between mobile and desktop versions
- syncs faster than any other wallet app[^6]
- encrypts communication over TOR[^7]

It can probably handle any account from any other wallets as long as you have
the secret key (unless it is Sprout)

::: warning
Zkool gives you full control over your funds. It is the only wallet app that
works in any situation, but it is also harder to use. Other wallets are much
more conservative and will make sure you stay on the established path.
:::

## Special Abilities

Zkool is currently the only wallet[^8]
- that can recover multiple keys concurrently. This is a great time saver when you have several wallets you want to import.
- support account indices, password on seeds. These variants are sometimes used internally by other wallet apps[^9].
- pre-sapling activation. Other wallets won't accept a birth height before ~2018.
- optimized scanning. If your key does not have a certain pool, Zkool won't spend time trying to synchronize it. Consequently, recovering a private key is nearly *instantaneous* because blocks aren't scanned.
- assign categories and store fiat exchange rates per transaction, allowing you
to track your spending and income.

[^1]: 12 to 24 word mnemonics, passphrase and account index. Sapling legacy keys, viewing keys, unified keys, xpub, etc.
[^2]: all pools except Sprout
[^3]: with AES encryption
[^4]: zcashd and zebrad
[^5]: lightwalletd and zaino
[^6]: including Ywallet
[^7]: via exit nodes and onion services
[^8]: besides Ywallet
[^9]: such as Ledger and ZecWallet
