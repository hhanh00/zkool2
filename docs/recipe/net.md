---
title: Mainnet, Testnet, Regtest
---
## Networks
Every database (wallet file) belongs to one network, chosen when it is
created. By default, Zkool creates mainnet wallets.

In the Database Manager, tap the **+** button. The *Create New Database* dialog
has a **Network** dropdown with Mainnet, Testnet, Regtest and ZSA Regtest. It
starts on the network of the wallet currently open; pick another one for the
new wallet.

The network is stored in the wallet file and cannot be changed afterwards: to
move to another network, create a new database and open it.

::: tip
Older wallets were put on testnet or regtest by their file name (`testnet` or
`regtest` in the name). They keep the network they were created on; the name no
longer matters.
:::

## Server and explorer

A new wallet starts with a server of its network: `https://zec.rocks` on
mainnet and `https://testnet.zec.rocks` on testnet. On the regtests, it points
at a local lightwalletd (`http://localhost:8137`) or at the ZSA testnet server;
change it in the Settings page to match your setup.

The server field of the Settings page has a button that opens a list of
known servers of the wallet's network, with their status.

The block explorer link of the transaction page follows the network too:
`zecblock.com` on mainnet and `testnet.zecblock.com` on testnet. The regtests
have no explorer and show no link.
