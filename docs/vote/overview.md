---
title: Overview
---

Zkool can vote in **coin weighted polls**: your vote counts for the ZEC your
account held at the poll's *snapshot height*. Nothing is spent. The vote
proves that you own the notes without revealing them, and the poll's
organizer never learns your address.

## How it works

- The organizer publishes a signed **voting config** that lists the rounds,
  the voting servers and the keys that sign them. Zkool verifies it before
  it shows you a round.
- A **round** has a snapshot height and one or more proposals, each with its
  options. Only the ZEC held in the **Ironwood pool** at the snapshot height
  counts. Funds received later, or held in the other pools, do not.
- Your notes at the snapshot are grouped into **bundles**. For each bundle,
  Zkool proves that it owns the notes and casts your ballot on the voting
  chain. Proving takes a few minutes per note.
- A part of each vote, a **helper share**, goes to helper servers, which
  reveal the shares when the round ends so that the votes can be tallied.
  Zkool keeps confirming the shares with the helpers until every one of them
  is in. A vote whose shares were not confirmed may not be counted.
- A note can vote only once in a round, and votes are final once the
  submission starts.

## What you need

- An account with its ZEC in the Ironwood pool before the snapshot height,
  synchronized past that height. If you are new to Zkool, see how to
  [install it](../guide/start.md), [restore an account](../recipe/restore.md)
  and [synchronize](../guide/sync.md).
- The voting config URL the organizer sends you.
- A network connection while the votes are cast and the shares confirmed.
  The submission continues in the background if you leave the page, and
  resumes by itself when you open the round again.

## Privacy

The proofs show that the notes exist and belong to you, without revealing
which notes they are. Some rounds withhold a few small notes from the vote
to keep the voting weight from identifying you; the eligibility preview says
how much is withheld.

Ready? See [How to vote](./howto.md).
