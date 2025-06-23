# General

## Pools

Zkool is a shielded and transparent wallet. It supports
every active pool: Transparent, Sapling, Orchard.

It does not support Sprout that has been deprecated and only
supported by `zcashd`.

## Accounts

Zkool manages multiple accounts completely independently.
Unlike Ywallet, synchronizing or resetting an account
does not affect other accounts.

For better performance, it is preferable to synchronize
multiple accounts concurrently but it is not a requirement.

## Database

Wallets are backed by
a database files (Sqlite). This database file can be
encrypted on disk. This prevents attackers from reading
the secret keys even if they manage to steal the wallet file.

Zkool allows creating and using different databases,
each with their own optional password.

## Synchronization

The synchronization algorithm is an improved version of
the Warp Sync used in Ywallet. The new version
is faster and uses less memory. It can also keep each
account synchronization data separated.

## Mempool

Zkool monitors the mempool and can recognize incoming
and outgoing transactions that are not yet included in
a block. The mempool monitor shows all the mempool transactions
and how they will update the account balances once confirmed.

## Market Price

On request, the average exchange rate between ZEC and USD
can be fetched from Coingecko.

## Servers

Zkool can work with both Light servers and Full Nodes
(`zcashd` and `zebrad`). However synchronizing from a full
node is much slower and resource intensive because the full
blocks must be sent and processed.

We recommend using a light node if possible.
