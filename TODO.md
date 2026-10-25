# Documentation TODO

Features of Zkool 6.35.0 that the docs site does not cover, as of 2026-10-25.
A feature counts as documented when a Guide, Recipe, Report, FROST, GraphQL,
ZSA, Voting, Vault or Zcash tech page explains it. The What's new pages are
release notes, so a feature that appears only there is listed as a gap.

Legend: `[ ]` open, `[~]` in progress, `[x]` done.

## Only in What's new

| | Feature | Where in the app | Only mention | Note |
| --- | --- | --- | --- | --- |
| [ ] | Swaps (NEAR Intents) | Account menu → Swaps | news/6.34 | The 6.34 page still lacks its swaps screenshot |
| [ ] | Memo plugins and Plugin Manager | Settings → Plugin Manager | news/6.21 | Rhai scripts that turn structured memos into tables |
| [ ] | Themes and Dark Mode | Settings → Theme | news/6.17 | |

## Not mentioned anywhere

| | Feature | Where in the app | Note |
| --- | --- | --- | --- |
| [~] | Contacts page | Account manager menu → Contacts | guide/pay says to type `@` for a contact, nothing on adding, editing, importing or exporting one. Also covers OpenAlias `#alias` resolution. Planned as recipe/contacts.md |
| [ ] | Light Node vs Full Node server | Settings | The toggle changes the server field label and the sync path |
| [ ] | Log viewer | Account menu → Log | |
| [ ] | Clear and reset to birth height | Edit Account | Rewind is documented in recipe/account and recipe/sync, reset is not |
| [ ] | Transaction table mode | Account menu toggle | Switches the history between list and table |
| [ ] | Cancelling Synchronization | Settings | |
| [ ] | File transmission via QR codes | Settings | QR size, error correction level, duration between codes, repair packets. recipe/cold uses animated QR but never explains the knobs |
| [ ] | Block Explorer | Settings | protocol/concepts mentions explorers in general only |
| [ ] | Sweep page | lib/pages/sweep.dart | The only "sweep" in the docs is the FROST ZIP 2005 upgrade sweep |

## Mentioned but too thin

| | Feature | Where it is mentioned | What is missing |
| --- | --- | --- | --- |
| [ ] | Note Migration to Ironwood | One glossary bullet in guide/account | No walkthrough of the Migration page or when to use it. No dedicated page on Ironwood or NU7 although a dozen pages mention them |
| [ ] | Expert mode | vault/howto and vault/overview | It also gates items on the account page |
| [ ] | Edit Account options | Spread across recipe/account and recipe/restore | Folder assignment, Full vs Incoming viewing key, export to encrypted file. No single reference |

## Stub pages in the repo

- [ ] docs/overview.md is a bare outline for a Protocol section and is commented out of the sidebar.
- [ ] docs/notes.md is empty.
- [ ] docs/protocol/concepts.md exists but is not in the sidebar.

## Suggested order

1. Contacts (in progress)
2. Swaps
3. Note Migration and Ironwood
4. A Settings reference page covering the server, theme, QR, explorer, cancel-sync and plugin options in one place
5. The rest
