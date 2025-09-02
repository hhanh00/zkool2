---
title: Privacy
---
With a public Transaction Graph, one can verify the validity and consistency of
the system as a representation of a currency. For instance, if a transaction
tries to spend a non existent output, the nodes will detect that there is no
previous transaction that matches the input.

However, the Transaction Graph also reveals a lot of information about the
spending of bitcoins that can fairly be analyzed by tracking companies.

Privacy Coins such as Zcash focus on **hiding** the transaction graph while
preserving its functionality.

The Transaction Graph still exists. But it is *globally* private, yet *locally*
public.

- The participants of a transaction know its details. Obviously the sender must
  know the address and quantity sent to the recipient, and the recipient must be
  able to recognize incoming funds.
- But, the other users should not be able to know (explicitly or by deduction) any of that.
