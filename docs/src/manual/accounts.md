# Accounts

## Keys

Accounts can be imported from various types of keys. The most common
one being the seed phrase.

### Seed Phrase

Zkool supports Seed phrases of 12, 15, 18, 21, and 24 words. New
accounts have a 24 word seed phrase.

We recommend using 24 words if possible.

Optionally, the user can add a password or a passphrase to the
seed phrase[^1].

Also the account can be derived at a given index in case the user
wants to have more than one account with the same seed phrase.

### Other Keys

Alternatively, accounts can be made using a unified viewing key
(see next section), a legacy Sapling extended key[^2],
an extended transparent key[^3] and finally a straight transparent
private key[^4]

## View Only Accounts

View Only Accounts are accounts that do not have secret keys but have
the view keys. They can detect incoming and outgoing transactions
and show the balance. But they absolutely cannot make new payments
because they cannot sign.

View Only Accounts typically work in conjunction with a second device
that has the secret keys in a secure location.

## MultiSig

Zkool supports the creation of multisig accounts M/N using
a Distributed Key Generation protocol (DKG).

Spending from a MultiSig account is possible using the
FROST protocol.

The DKG and FROST protocols are implemented without the
usage of a third party server.

[^1]: Like in Trezor and Ledger
[^2]: Sapling keys exported by zcashd
[^3]: These are common in the Bitcoin world and corresponds to the BIP-32
derivation standard (`xprv` and `xpub` keys)
[^4]: It is the exported secret key. They begin with 'K' or 'L'
