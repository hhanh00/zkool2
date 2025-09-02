---
title: Bitcoin Data Model
---
The Bitcoin transaction model is called the UTXO model (Unspent Transaction Output model).

Here’s how it works step by step:

## Transactions consume outputs, not balances

Unlike an account model (like Ethereum), Bitcoin doesn’t track “balances.”

Instead, each transaction consumes unspent outputs (UTXOs) from previous transactions and creates new outputs.

## Transaction inputs

Each input points to a previous transaction’s output that hasn’t been spent yet (a UTXO).

To spend it, the input must provide a valid unlocking script (often a digital signature proving ownership of the private key controlling that UTXO).

## Transaction outputs

Each output has:

- Value: an amount of BTC.
- Locking script (scriptPubKey): conditions under which it can be spent (e.g. "only the owner of this public key can spend it").

Outputs become UTXOs once the transaction is confirmed.

> A UTXO can only be spent once.

### Change

Since inputs must be spent in full, if you want to pay less than the value of an input, you send the difference back to yourself as “change.”

Example:

You have a UTXO worth 1 BTC. You want to pay someone 0.3 BTC.

You create a transaction with:

- Input: 1 BTC UTXO.
- Outputs:
  - 0.3 BTC to the recipient.
  - 0.7 BTC back to yourself (minus transaction fee).

## Transaction fees

Fees aren’t explicit. They’re implied:

fee = sum(inputs) – sum(outputs)

Miners collect this difference as an incentive.

## Chaining

Every transaction output can later become someone else’s input.

This creates a chain of transactions, where all coins can be traced back to the block reward outputs (coinbase transactions) of mined blocks.

> In summary: Bitcoin doesn’t have balances. It has UTXOs. Transactions consume
> UTXOs as inputs and create new UTXOs as outputs. This ensures coins aren’t
> double spent and ownership can be cryptographically proven.

