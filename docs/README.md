This is the documentation website for ZKool.

Zkool is the swiss-army wallet for zcash.

Its features include
- most zcash keys[^1]
- transparent, sapling and orchard pools[^2],
- diversified shielded addresses and transparent address rotation,
- multi accounts and multi wallets[^3],
- mainnet, testnet and regtest,
- light nodes[^4] and full nodes[^5]
- TOR[^6]

::: tip
It can probably handle any account from any other wallets as long as you have
the secret key (unless it is Sprout)
:::

[^1]: 12 to 24 word mnemonics, passphrase and account index. Sapling legacy keys, viewing keys, unified keys, xpub, etc.
[^2]: all pools except sprout
[^3]: with AES encryption
[^4]: lightwalletd and zaino
[^5]: zcashd and zebrad
[^6]: via exit nodes and onion services
