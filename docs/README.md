---
home: true
heroText: Zkool
tagline: The swiss-army wallet for zcash
---
Zkool:
- supports most zcash keys[^1]
- works with transparent, sapling and orchard pools[^2],
- generates diversified shielded addresses and rotates transparent address,
- manages multiple accounts, folders and wallets[^3],
- connects to mainnet, testnet and regtest,
- speaks the light nodes[^4] and full nodes[^5] protocols
- syncs faster than any other wallet app[^6]
- encrypts communication over TOR[^7]

It can probably handle any account from any other wallets as long as you have
the secret key (unless it is Sprout)

::: tip
Zkool is like a Formula 1 wallet. It drives faster and gives you full control but
you can drive off the track. Other wallets are much more conservative and will
make sure you stay on the established path.
:::

[^1]: 12 to 24 word mnemonics, passphrase and account index. Sapling legacy keys, viewing keys, unified keys, xpub, etc.
[^2]: all pools except Sprout
[^3]: with AES encryption
[^4]: lightwalletd and zaino
[^5]: zcashd and zebrad
[^6]: including Ywallet
[^7]: via exit nodes and onion services
