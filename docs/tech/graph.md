---
title: Transaction Graph
---
## Nodes in the Transaction Graph

Each node is a transaction.

Some papers and tools also model each UTXO as a node instead (this is called the
UTXO graph).

## Edges in the graph

An edge represents the flow of value:

If a transaction A creates an output (UTXO), and a later transaction B uses that
output as an input, you draw a directed edge from A → B.


This captures the fact that B depends on A.

## Graph structure

It’s a directed acyclic graph (DAG):

Transactions only point forward in time (can’t spend future coins).

No cycles, because once an output is spent, it cannot be recreated.

All transaction paths ultimately lead back to:

Coinbase transactions (block rewards), which are the origin of all bitcoins.

## What the graph shows

Flow of coins: You can trace any satoshi (or UTXO) back to its origin.

Clustering analysis: Because outputs can be linked when spent together, analysts
can try to infer common ownership.

Network health: Graph analysis is used in forensics, compliance, and research.

5. Example

Imagine:

- Tx1: Coinbase creates 50 BTC → Output A.
- Tx2: Uses A (50 BTC) as input → sends 30 BTC to Bob, 20 BTC back to Alice.
- Tx3: Uses Bob’s 30 BTC to pay Carol 10 BTC and himself 20 BTC.

The graph would look like:

```mermaid
flowchart TD
    Tx1["Coin Base (50 BTC)"]
    Tx2["Tx1"]
    Tx3["Tx2"]

    Bob1["Bob (30 BTC)"]
    Alice["Alice (20 BTC)"]
    Carol["Carol (10 BTC)"]
    Bob2["Bob (20 BTC)"]

    Tx1 --> Tx2
    Tx2 --> Bob1
    Tx2 --> Alice
    Bob1 --> Tx3
    Tx3 --> Carol
    Tx3 --> Bob2
```

In summary:
> The transaction graph is a DAG connecting all Bitcoin transactions by their
> input/output relationships. It’s the backbone for tracing funds, analyzing
> usage patterns, and enforcing Bitcoin’s rule that each UTXO can be spent only
> once.
