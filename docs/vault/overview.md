---
title: Overview
---

The **Cloud Vault** keeps an encrypted copy of your account keys in your
Google Drive, so that a lost or broken device does not mean lost accounts.
It stores the seed phrase of each account with its account index, its birth
height, its change setting and its name: everything Zkool needs to recreate
the account. Nothing else leaves the device, and nothing in the vault can be
read without your master password or one of your passkeys.

::: warning
The vault is a feature for advanced users. It is hidden behind
[expert mode](./howto.md#expert-mode), and it is a backup of your keys, not
a replacement for writing the seed phrases down.
:::

## What it protects

- **Only you can read it.** The accounts are encrypted on the device before
  they are uploaded. Google holds the encrypted bytes and cannot decrypt them,
  and neither can Zkool's author.
- **It lives in the app data folder of your Drive.** The files are not
  visible in Drive, do not count against your view of it, and are removed if
  you remove Zkool from your Google account.
- **Two ways in.** A master password, chosen when the vault is created,
  always opens it. On a device with passkeys, a passkey registered for the
  vault opens it with Face ID or Touch ID instead.
- **It follows your accounts.** With the vault on, a new or edited account is
  written to the vault as soon as it is saved. The accounts that existed
  before the vault was turned on get in the first time they are edited.
  Accounts restored from a short seed phrase are left out, because the vault
  only stores full seeds.

## How it is protected

The vault is a log of entries that Zkool only appends to. The first entry
holds an X25519 key pair: the public key in clear, the secret key encrypted
with ChaCha20-Poly1305 under a key derived from your master password with
Argon2id and a random salt. Each device that registers a passkey adds an
entry with the secret key encrypted again under the passkey's PRF output, a
secret that only that passkey can produce. Each account is an entry
encrypted to the public key with a fresh ephemeral key, so writing to the
vault never needs the master password. Recovering decrypts the secret key
with the password or the passkey, then every account entry with it.

## What it does not protect against

- Losing the master password **and** every device with a registered passkey.
  There is no reset. Keep the seed phrases written down.
- A weak master password. Anyone who gets into your Google account can
  download the vault and try passwords offline. Argon2id slows them down,
  it does not stop them.
- Accounts without a seed: viewing-only accounts, private key imports,
  Ledger accounts and multisig accounts are not in the vault. Back them up
  with their own files.

## Platforms

| | Android | iOS | macOS | Linux, Windows |
|---|---|---|---|---|
| Cloud Vault | yes | yes | yes | no |
| Passkeys | yes | no | no | no |

Google sign-in is built into the Android, iOS and macOS apps only. Passkeys
are available on Android; the other platforms open the vault with the
master password.
