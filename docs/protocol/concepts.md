---
title: Concepts
---

A few general concepts that are important to know.

## State and Consensus

The state is the data that your "system" wants to track.
In a cryptocurrency, it is most of the time the association
between an *address* and a *value*, which represents the
"balance" of an "account".

- Account --> Address
- Balance --> Value

We say that the system reaches consensus when there is a
undisputed and unambigious state that participants of system
agree on.

::: tip
This is harder to achieve than it seems, especially in distributed
systems.
:::

## Initial State and Blockchain

At the beginning, the system starts with a predetermined state.
Usually, it is the state where every account has zero balance[^1].

Then as *transactions* are performed and recorded in the *Blockchain*,
the state evolves.

::: important
To maintain *consensus*, the *effects* and the *order* of the transactions
must be **deterministic**.
:::

## Centralized vs Decentralized

A system is centralized when all the transactions must go through
a single authority. Effectively, there is a **boss**. Everyone must
report to them and nothing can change the State unless it has
been checked and authorized by the boss.

The banking system follows the centralized model. Customers
interact via Online Banking, Vendor Terminals, Retail Agents, etc.
who will consult with the Bank Account Database before
authorizing the operation[^2].

The alternative is a **decentralized** system where no single
authority is in charge of the system. Instead, consensus is reached
when it can be established that "enough" participants agree and
that the state is correct and unambiguous.

Correctness is the "easier" part. Unless there are bugs
in the software, the state remains correct (by induction)
if it started correct and *every transaction* is also correct.

But, there could be multiple correct states if the participants
use different (valid) transactions.

We are not going to get into the details of how consensus
is determined. It is *difficult* and *varies between types
of Blockchains*.

## Public vs Private

Most cryptocurrencies are anonymous but not private.
Users do not register with a central authority in order
to create an account (or an address) but it is possible,
and fairly easy, to get the transaction history of
any address. With a bit of effort, the transaction
between addresses can be plotted and analyzed,
giving valuable insights into someone's spending.

When external information is added, for example the KYC
(Know Your Customer) data collected by the crypto
exchanges, one can further identify the transaction details.

Private cryptocurrencies aim to eliminate (or at least
reduce) the amount of identifiable information.

Ideally, a private system would be such that only the
people involved in a transaction know that it took
place. The recipients receive their funds
but does not know where they came from.

In real life, a cash transaction achieves that[^3].
But in the digital world, it is **extremely difficult**.

::: important
**No cryptocurrency is perfectly private** unlike what
they sometimes claim. It is always the responsibility of
the user to know what information is made available
to third parties. Also, users should be aware that
there are *tradeoffs* between **usability** and **privacy**.
:::

Unfortunately, many users overlook this aspect and
demand that their private coin behaves as well as public
coins. Progress can and will be made, but there are
limitations that are very unlikely to disappear[^4].

The goal of this documentation is to give you the
required insights into the ZK (Zero Knowledge) based
privacy coins to make educated decisions.

## Pools

Pools are groups of funds that share the same properties.
Currencies are fungible, that means 1 unit of a currency
has the same value whether it's the first one or any
other created one. In other words, 1 dollar bill is
always worth 1 dollar[^5].

It's the same thing for Cryptocurrencies.

However, it does not mean every ZEC or every USD
has the same *physical* property. For example, 100 USD
bill has the same value as 100 dollar coins or a
cashier check of 100 USD but has different practical
usability.

We say that coins form a *pool* of funds. Bank accounts
form another pool. Funds can go from one pool to another
with an exchange rate of 1 for 1 but it requires a
transaction.

Most cryptocurrencies have a single type of funds, and therefore
there is no concept of pool. But Zcash has three pools[^6]
now.

::: important
Maybe one day Zcash will eliminate pools, but right now
pools are a reality and users should know what they offer
and how they differ.
:::

### Transparent Pool

The Transparent Pool contains funds that were sent to
a transparent address. The addresses and amounts are
in plain text and visible on the Blockchain. With the
help of a block explorer, anyone can see where the funds
came from with the same details as in Bitcoin. In other
words, **the transparent pool is fully public**.

For example, by following this link: [t1PEp2GJLSdhDfCKqc2J211WKDUS1NfoQNy](https://mainnet.zcashexplorer.app/address/t1PEp2GJLSdhDfCKqc2J211WKDUS1NfoQNy)
you get its history of transactions.

Transparent addresses begin with `t`.

### Shielded Pools

There are three shielded pools:
- Sprout,
- Sapling,
- Orchard.

Sprout is deprecated. By now, it is only supported by `zcashd` and only
for withdrawals. Sapling and Orchard are active.

Address beginning in `zs` are Sapling Shielded addresses.
Orchard addresses begin with a `u`.

::: warning
Not all addresses that begin in `u` are Orchard! Instead they
are Unified Addresses (UA) and may contain multiple addresses
(called receivers in this context), **including transparent**
ones.

**Do not assume that an address in `u` is only shielded.**
:::

Transactions can involve funds from multiple pools.

## Account vs Outputs Model

Transactions capture the exchange of funds in two
possible ways:
- They can say: "Transfer this amount from this address
to that address", that's the *Account Model*,
- or they can say: "Here's some coins and bills", that's
the *Output Model*[^7].

In real life, we use both models. When we use cash, ie
physical money, we go with the Output Model. When we
do an online transaction, we use the Account Model.

From the perspective of someone who wants to
verify a transaction, the two models differ.

In any case, the verifier must check
that the account has enough funds.

With the Account Model, the verifier keeps track of the balance
of every account since any account can initiate a transaction.
It does not have to keep the *history* of the account though.
That's one value per account. It's small but it is **global**
public data.

With the Output Model, the verifier must verify that
the funds that are spent in the transaction were not
previous spent. That's like tracking every bills and coins.
It is more data but verification only requires **local**
public data. The verifier does not need to know the balance
of the account, just that the funds involved in the transaction
are valid.

::: tip
Private systems use the (unspent transaction) output model.
:::

We don't have a technology that allows verification
of global state privately. Essentially, we cannot
verify that the account balance is greater than the value
spent without revealing information about the account,
the balance or the transaction amount.

## Transactions

A transaction takes several[^8] *Input* notes
and creates *Output* notes.

A note represents an amount of currency at a given address.
A note is associated with a single address but an address
may be associated with multiple notes (or none).

There are Transparent notes, Sapling notes and Orchard notes.
They use different cryptography, therefore they don't have
the same address type. Their amount is interchangeable though.

- Inputs must refer to previous outputs.
    - You cannot "borrow" money from a future transaction.
    - You cannot spend an output twice.
- Inputs come with a digital signature that authorizes
their spending.
- Outputs do not have signatures. As long as the address
appears valid, it can be used. This means that the system
does not verify that the recipient has the secret key.
- The total amount of the inputs must exceed the total amounts
of the outputs. The difference is the transaction fee that
the miner can claim in the block reward[^9].
- A transaction can involve notes from multiple pools.
- Since amount are interchangeable, you can have
for example, transparent input notes and shielded output notes
as long as the transaction does not break any rules.
- Shielded notes hide their amount and their address.

::: important
All these rules must be verifiable by a third-party
not involved in the transaction. We'll come back
to them when we revisit consensus in the following sections.
:::

## Blockchain

The blockchain is a series of blocks that are linked by a
back reference `prev_hash`. Essentially every block is hashes
by a cryptographic hash function which guarantees that its
contents cannot be modified without resulting in a different
hash value.

Every block contains the hash of its predecessor thus
forming a "chain" where each block links to the previous block.

No one can modify a block without "breaking" the chain.

## Blocks

Blocks contain a list of transactions. Their position in the list
determines the order in which they should be applied[^10] to
the State.

They are produced by either *miners* or *validators* (depending on
the type of blockchain). Regardless of the mechanism, the block
production has several goals:
- ensure that blocks are produced correctly and at the
desired interval
- avoid or reduce the chances of having several blocks produced
simultaneously, since it leads to an ambiguous state.

## Mempool

Before they get included in a block, transactions flow from
node to node. The mempool stores the pending transactions,
waiting for a block producer to include them. Until then,
they can be evicted since a conflicting transaction may get
chosen instead.

In a decentralized system, nodes have different transactions
in their mempool. The block producers have their own mempool too.
They will emit blocks based on the transactions they have.

## Footnotes

[^1]: Some cryptocurrencies start with funds in a set of
predetermined addresses. This is called premining.
[^2]: In practice, for performance reasons there are
operations that do not require permission but the
bank can revert them at a later time.
[^3]: Assuming no other witnesses, unmarked bills, etc.
[^4]: Basically, it would take a miracle.
[^5]: Excluding collector's value
[^6]: Four if you count the first shielded pool Sprout
which has fallen out of usage.
[^7]: Called UTXO model because Bitcoin came with this name
first.
[^8]: Or none if it is the mining reward.
[^9]: Mining and fees are described later. They are not
critical for understanding the protocol, but are important
for the economics and security of the cryptocurrency.
[^10]: Changing the order can result in a different state since
a transaction cannot use funds from later transactions.

