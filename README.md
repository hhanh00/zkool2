## What is this?

Zkool is a multi-account wallet for Zcash.

**Linux:** You can build and run as [Flatpak](flatpak/README.md) (recommended), or use the [AppImage/Deb](.github/actions/linux) from the release artifacts.

## Feature Tables

### General

| Feature         |                                        |
|-----------------|----------------------------------------|
| Pools           | Transparent, Sapling, Orchard          |
| Accounts        | Multiple, Hidden (Protected)           |
| Database        | Encrypted on disk                      |
| Synchronization | Improved Warp, Per-account state       |
| Mempool         | Monitor incoming/outgoing transactions |
| Market Price    | ZEC/USD                                |
| Servers         | Full Nodes & Light Nodes               |

### Account
| Feature            |                                                          |
|--------------------|----------------------------------------------------------|
| Seed Phrases       | Optional passphrase, 12/15/18/21/24 words, Account Index |
| View only accounts | By Unified Viewing Key (VK), Sapling Extended VK         |
| Transparent VK     | Extended Keys (xpub, xprv), Private Key                  |
| MultiSignature     | Distributed Key Gen, FROST Signatures                    |

### Addresses
| Feature           |                                        |
|-------------------|----------------------------------------|
| Rotated Addresses | Transparent and Shielded               |
| Change Addresses  | Rotated, Internal                      |
| Notes             | Coin Control                           |
| Transparent Funds | Recover alternate addresses            |

### Transactions
| Feature                      |                                                  |
|------------------------------|--------------------------------------------------|
| Detailed Transaction Details | Metadata, Amount Transacted, Pools Input/Outputs |
| Memo                         | Text, Binary, Full text search                   |
| Multi Recipients             | "Send Many"                                      |
| Quick Shield/Unshield        | Button shortcuts                                 |
| Payment URI                  | Single/Multiple Recipients                       |
| Load/Save/Sign Transactions  | Offline Signing, Cold Wallet, PCZT               |

## What can you do with it?

- **It supports nearly every type of account**
    - From 12, 18, 21, 24 words seed phrase with an optional password
    - With internal address derivation for change (Zashi, ZIP 316)
    - From Unified Viewing Key (with complete or partial list of receivers)
    - From legacy shielded extended keys (secret and viewing) of zcashd
    - From transparent xpub/xprv keys (Electrum and BIP 32 compliant wallets)
    - From seed phrase and BIP 44 (Exodus, Ledger, and other transparent wallets)
    - From transparent secret key (any key export from a transparent wallet)
    - and of course Ywallet
- **It handles accounts individually**
    - Each account has its own synchronization state and be included or
    excluded from the global sync. This allows you to "park" accounts
    by disabling them. They do not slow down sync of your active accounts.
    If you need them later, you reenable them and bring them up to date.
    - An account can be exported and then imported in a different wallet file.
    The entire data (notes, spends, witnesses, etc) gets saved into an
    *encrypted* file.
    - Wallet files can be also encrypted.
    - Zkool supports more than one account and does not lock you to the list of accounts[^1]
- Its shielded features are as good as Ywallet. In particular, it will
    - minimize cross pool usage
    - allow you to select your pools
    - can create multi recipient payments
    - reads/writes multi recipient payment uris
    - *supports shielded multi signature accounts using FROST*
    - and more
- and its transparent privacy as good as transparent wallets
    - shielded wallets tend to handle the lack of privacy of
    transparent addresses by mandating the shielding of transparent
    funds before they can be spent (Zashi, Zingo, ...). Instead,
    Zkool supports address rotation for the users[^2]
- Your wallet file can be encrypted *at rest*
- TOR proxy and Onion services for all connections to Zcash servers
- Supports Fullnodes and Lightnodes

## What it does *not* do well

- UI is basic. There are only a few screens and nothing flashy.
On the flipside, the UI is relatively simple to understand.
- Some nice to have features are missing
    - No *address book*. Mainly because Zcash has diversified addresses
    that make address books useless if used[^3]
- No keytool
- No customization of reference currency (always USD)
- No pool transfer tool
- No third party swap integration
- No spending tracking
- No auto shielding of any sort

[^1]: Ywallet can only save and restore *all* the accounts.
[^2]: Obviously, not as good as shielding but offers *some* level
of privacy.
[^3]: The diversified address cannot be matched against the *one* address
recorded in the address book.
