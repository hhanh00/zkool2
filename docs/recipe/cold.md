---
title: Cold Wallet
---

> A Cold Wallet is a wallet that has secret keys but remains disconnected from the Internet **at all times**.

It is not sufficient to be disconnected _most of the time_. For instance, a wallet on a phone that it is turned on only when making transactions is NOT a cold wallet.

To make a payment, there needs to be another wallet that connects to the Zcash network and synchronizes the blockchain data. This wallet has the account _viewing key_ to properly identify the wallet transactions but is not capable of spending any funds. We'll call it the "Hot Viewing Wallet"[^1].

You have two copies of Zkool: The "Hot Viewing Wallet" (HVW) and the "Cold Wallet" (CW).

# Setup

1. Start on the CW. Create a new account. Backup the seed phrase to a safe place (do not put the seed phrase on your phone or in the Cloud).
2. Show the UFVK (Edit the account and tap the "Show Viewing Key" button)
![UFVK](./images/06.ufvk.png)
::: tip
Scan the QR code on the HVW and restore a wallet using it as the Key.
:::
3. Synchronize it.

::: important
- Make sure you keep the cold wallet offline.
- Make sure you do not have a copy of the seed phrase on a device that can
connect to the Internet.
:::

## Payments

Use the HCW to monitor incoming transactions and verify outgoing payments. To
start a payment from the account, use the HCW and enter the details of the
payment as usual.
![Save Tx](./images/07.save.png)
Once you reach the final screen with the Transaction Plan, you probably will
notice that the "SEND" icon has been replaced by a "SAVE" icon. This indicates
that the account cannot sign the transaction and is view/watch-only.

Use this to make a PCZT (Partially Created Zcash Transaction) file.

::: tip
Use a *clean* USB drive to move the PCZT file. The file does not have secret
data but contains the details of the transaction. It is better not to put it
online if the transaction has to remain absolutely private.
:::

[^1]: A hot wallet is a wallet connected to the Internet.
