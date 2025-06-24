# Recovery

One of the distinctive features of Zkool is its ability to
recover accounts from various types of keys.

The main key type is the **seed phrase**, but Zkool supports
more a half dozen types of keys. They usually come from
older wallet apps that didn't support seed phrases,
or used for specific purposes.

## Keys

Let's see what keys are available:

- A Seed phrase. They come in lengths of 12, 15, 18, 21 or 24 words.
Currently, the 24 word variant is the most commonly used.
An extra **password** (usually a single word) can be added. It defaults
to empty.

Zkool supports any of these lengths. From a seed phrase, the password and the
**account index[^1]**, Zkool derives the Transparent, Sapling and Orchard
**extended secret keys**.

```admonish info
Extended Secret Keys are secret keys, i.e. keys that allow the
spending of funds, that are extended to allow for the derivation
of additional addresses.
```

- Transparent or Sapling[^2] extended secret keys. The former
begins with `xprv`, while the later begins with `secret-extended-key`.
These allow the creation of an account with a single pool.

- Transparent has an extended public key format. They begin
with `xpub`. It allows
the derivation of multiple transparent addresses, but they cannot be used
to spend funds.

- Sapling has a legacy format for extended viewing keys. They
begin with `zxviews`. They give access to the entire transaction
history (past and future), but they cannot be used to spend funds.

- Unified Viewing keys (UVK) combine one or many different types of
viewing keys[^3]. This is the better format for viewing keys.
For example, a UVK can be a group of a transparent and sapling
viewing key. Or it could be an Orchard viewing key by itself.
UVK begins with `uview`.

These keys offer **address diversification**.

The last type of key supported is the transparent private key.
This is used by old wallets that do not feature
either address rotation or shielded transactions.
A private key begins with `K` or `L`.

A private key corresponds to a single transparent address.

```admonish info
In any case, do not forget to enter the birth height
as it can greatly speed up the account synchronization.
```

If you do not know the birth height exactly, it is still
good to input an estimate as long as it *lower* than the
real birth height.

[^1]: The account index enables the derivation of multiple
independent accounts from the same seed phrase. By default,
wallets use the account index 0. Wallets that support multiple
accounts increment the account index for each additional account.
[^2]: Orchard does not expose a standard format for their
extended secret keys therefore they cannot be supported.
[^3]: Any combination of Transparent, Sapling and Orchard
viewing keys is possible *except* for the Transparent key by itself.
