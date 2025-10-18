---
home: true
heroText: Zkool
tagline: The swiss-army wallet for Zcash
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
        Multi Accounts / Folders /
        Databases
    - title: Secure
      details: |
        Encrypted Data Files /
        Your keys stay with you.

---

::: info
Zkool is the successor to Ywallet. It has been rearchitectured and rewritten
from scratch with a more modern technology stack. It is faster and more powerful
than before.
:::

Some of the features of Ywallet were removed and replaced by something equivalent
or better.

> If you have a feature of Ywallet that you think should be in Zkool, please
> contact the author by opening an [issue](https://github.com/hhanh00/zkool2/issues)
> on the project github.

## Features

- supports most [zcash keys](recipe/restore)[^1]
- works with transparent, sapling and orchard pools[^2],
- generates [diversified shielded addresses](guide/addresses) and rotates transparent address,
- manages [multiple accounts](guide/accounts), [folders](recipe/folder) and [wallets](recipe/database)[^3],
- supports multi signature shielded accounts by leveraging the [FROST](frost/guide) threshold
  signature algorithm
- connects to [mainnet, testnet and regtest](recipe/net), via full nodes[^4] and lightwalletd
  servers[^5]
- wallets and accounts are [interoperable](recipe/database) between mobile and desktop versions
- syncs faster than any other wallet app[^6] with Warp Sync 2
- [encrypts communication](recipe/tor) over TOR[^7]
- interfaces with the [Shielded Zcash Ledger](recipe/ledger) app (for Nano S+,
  NanoX, etc) and can be setup as a [Cold Wallet](recipe/cold)
- helps you [organize your finances](report/overview) by categories, and generates reports and charts
- integrates [Trading View charts](guide/other)

It can probably handle any account from any other wallets as long as you have
the secret key (unless it is Sprout)

::: warning
Zkool gives you full control over your funds. It is the only wallet
app that works in any situation, but it is also harder to use. Other wallets are
much more conservative and will make sure you stay on the established path.
:::

## Special Abilities

Zkool is currently the only wallet[^8]

- that can recover multiple keys concurrently. This is a great time saver when
  you have several wallets you want to import.
- support account indices, password on seeds. These variants are sometimes used
  internally by other wallet apps[^9].
- pre-sapling activation. Other wallets won't accept a birth height before
  ~2018.
- optimized scanning. If your key does not have a certain pool, Zkool won't
  spend time trying to synchronize it. Consequently, recovering a private key is
  nearly *instantaneous* because blocks aren't scanned.
- FROST multi signature accounts with Distributed Key Generation and no
  centralized coordination
- assign categories and store fiat exchange rates per transaction, allowing you
to track your spending and income.

[^1]: 12 to 24 word mnemonics, passphrase and account index. Sapling legacy
    keys, viewing keys, unified keys, xpub, etc.
[^2]: all pools except Sprout
[^3]: with AES encryption
[^4]: zcashd and zebrad
[^5]: lightwalletd and zaino
[^6]: including Ywallet
[^7]: via exit nodes and onion services
[^8]: besides Ywallet
[^9]: such as Ledger and ZecWallet

