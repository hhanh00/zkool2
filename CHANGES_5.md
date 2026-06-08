# CHANGES_5 — Multi-network support (testnet / regtest)

Summary of the changes.

Adds the ability to run **mainnet**, **testnet**, and **regtest** side by side,
each with its **own accounts** (nothing is shared between networks). A new
**Switch Network** button on the Account List opens a dedicated `/networks` page
where the active network is chosen; switching happens **live, without an app
restart**.

Per-network isolation is achieved with **one SQLite database file per network**
(the Rust core was already network-aware via `Coin.coin` → `network()`):

| Network | DB name (base `zkool`) | coin | default server | light node |
|---------|------------------------|------|----------------|------------|
| Zcash (mainnet) | `zkool`          | 0 | `https://zec.rocks`          | yes |
| Zcash Testnet   | `zkool-testnet`  | 1 | `https://testnet.zec.rocks`  | yes |
| Zcash Regtest   | `zkool-regtest`  | 2 | `http://127.0.0.1:18232` (Zebra) | **no** |

---

## Files changed

```
 M lib/pages/accounts.dart          Switch Network button + per-network title
 M lib/pages/db.dart                pass explicit coin on new-DB open
 M lib/pages/splash.dart            restore persisted network on startup
 M lib/pages/tx_view.dart           drop dead {net} explorer substitution
 M lib/router.dart                  register /networks route
 M lib/settings.dart                network-aware servers + explorers
 M lib/store.dart                   switchNetwork + openAndWireDatabase + seeding
 M rust/src/api/coin.rs             explicit `coin` arg on open_database
 M rust/src/graphql-cli.rs          update open_database caller
 M pubspec.yaml                     bundle assets/zcash.svg
 M lib/src/rust/api/coin.dart       (regenerated FRB binding)
 M lib/src/rust/api/network.dart    (regenerated FRB binding)
 M lib/src/rust/frb_generated.dart  (regenerated FRB wire glue)
 M rust/src/frb_generated.rs        (regenerated FRB wire glue)
 M lib/store.g.dart                 (regenerated riverpod)
?? assets/zcash.svg                 theme-adaptive Zcash logo
?? lib/network.dart                 network metadata (single source of truth)
?? lib/pages/networks.dart          the /networks selection page
```

> Generated files (`frb_generated.*`, `*.g.dart`, `coin.dart`, `network.dart`
> under `lib/src/rust/`) were regenerated via `flutter_rust_bridge_codegen` /
> `build_runner`, not hand-edited.

---

## 1. Rust — explicit coin on `open_database`

**File:** `rust/src/api/coin.rs` (+ FRB regen, + `graphql-cli.rs` caller)

`Coin::open_database` gains a third argument `coin: Option<u8>`:

- **`Some(c)`** (network selection): authoritative — persisted to the DB `coin`
  prop and used as the network.
- **`None`**: falls back to the stored `coin` prop (default mainnet `0`).

The network used for `migrate_sapling_addresses` is now derived from the
**resolved** coin (after open), not the pre-open value.

`try_open` no longer **overwrites** the `coin` prop on every open from the
filename. It now seeds the prop from the filename **only when it is missing**
(first-time creation), so an explicit choice is never clobbered and a DB whose
name merely contains "testnet" can't be misclassified.

> Requires FRB regen (`flutter_rust_bridge_codegen generate` with NU7 RUSTFLAGS)
> to update `lib/src/rust/api/coin.dart` + `frb_generated.*`.

## 2. Dart — `lib/network.dart` (new)

Single source of truth for the networks:

- `enum ZNetwork { mainnet, testnet, regtest }` with a `NetworkInfo` per network
  (coin id, label, db suffix, default LWD + alternatives, default explorer,
  light-node default).
- Helpers: `networkForCoin`, `networkForName`, `baseDbName`, `dbNameForNetwork`,
  `networkTitle`, `networkAccent`, and a `ZNetworkX` extension (`net.coin`,
  `net.info`).
- Mainnet keeps the base DB name unchanged (`zkool`), so **existing installs are
  untouched**; testnet/regtest append `-testnet` / `-regtest`.
- **Regtest** defaults to a **Zebra full node** (`http://127.0.0.1:18232`,
  `defaultIsLightNode: false`) since lightwalletd regtest support is unknown.

## 3. Dart — live network switch (`lib/store.dart`)

- `SynchronizerNotifier.stop()` — tears down the in-flight sync (`cancelSync`,
  cancels the progress subscription, clears state) so no stream keeps writing to
  the previous network's DB pool.
- `openAndWireDatabase(...)` — shared open-flow (password-retry loop, seed
  per-network defaults, set LWD/Tor/proxy from the now per-DB settings, publish
  to `coinContext`). Reused by both startup and switching.
- `_seedNetworkDefaults(...)` — on first open of a network DB, seeds `lwd`,
  `is_light_node`, and `block_explorer` props from `NetworkInfo` when absent.
- `switchNetwork(ref, net, askPassword:)` — orchestrates the live switch: stop
  sync + mempool → reset selected account → open the per-network DB → persist
  `database` + `network` prefs → invalidate all network-scoped providers →
  restart auto-sync and the mempool listener.

## 4. Dart — `/networks` page + Account List button

- **`lib/pages/networks.dart` (new):** radio-style selector (Zcash / Zcash
  Testnet / Zcash Regtest). Selecting a different network **switches
  immediately** (no confirmation dialog), prompting for a password only if the
  target DB is encrypted. The current network shows a "Current network"
  subtitle; the server URL is not shown.
- **`lib/router.dart`:** registers `/networks`.
- **`lib/pages/accounts.dart`:** a **Switch Network** icon button (globe) as the
  first item in the Account List actions — i.e. between the `+` New Account
  button and Settings.

## 5. Dart — per-network UI polish

- **App / page title (`accounts.dart`):** the Account List AppBar shows
  `zkool` on mainnet, `zkool (testnet)` / `zkool (regtest)` otherwise
  (`networkTitle`).
- **Theme-adaptive logo (`assets/zcash.svg`, new):** a Zcash "Ƶ" glyph drawn with
  `currentColor`, tinted per network via a flutter_svg `colorFilter`, so it reads
  on any theme. Replaces the temporary `misc/icon.png`.
- **Network-aware servers (`settings.dart`):** the Light Node Server dropdown
  uses the bundled `servers.json` on mainnet and the network's defaults on
  testnet/regtest.
- **Network-aware explorers (`settings.dart`):** added `kTestnetBlockExplorers`
  and a `blockExplorersFor(net)` selector; removed the `{net}` placeholder from
  `kBlockExplorers` (now literal mainnet URLs) and from the testnet default
  (now literal `testnet.cipherscan.app`), so testnet explorers appear as **named
  dropdown entries** instead of "Custom Explorer". The dead `{net}` substitution
  in `tx_view.dart` was removed.

## 6. Startup & new-DB callers

- **`splash.dart`:** reads the persisted `network` pref and passes it as the
  explicit `coin` to `openDatabase`, restoring the last-used network on launch
  (falls back to `null` → stored prop for existing single-network installs).
- **`db.dart`:** the Database Manager's "new database" opens with the
  currently-active network's coin.

---

## Build / regen notes

- Rust changes require `RUSTFLAGS='--cfg zcash_unstable="nu7"'` and an FRB regen
  (`flutter_rust_bridge_codegen generate`) to refresh the bindings.
- No new `@freezed` / `@riverpod` declarations were added, so `build_runner` is
  only needed if `store.g.dart` drifts.
- Verify with `flutter pub get` + `flutter analyze`, then `flutter build windows`.

## Open items / things to confirm during testing

- **Regtest Zebra RPC port** defaults to `18232`; adjust in Settings (or change
  the default in `lib/network.dart`) if your Zebra regtest listens elsewhere.
- Switching networks tears down and restarts the global sync/mempool listeners;
  watch for any stream still bound to the old DB during a fast switch.

---

## Addendum — Mobile responsive fix

A narrow-screen layout pass so the Account View and Settings render cleanly on
mobile widths. The breakpoint is a simple `MediaQuery` width check
(`width < 600` ⇒ mobile); no new dependencies.

**Files:** `lib/pages/account.dart`, `lib/settings.dart`

### `lib/pages/account.dart` — collapse AppBar actions on mobile

- The crowded AppBar action row (Rescan from Height, Backup Seed & Keys, Remove
  Account) no longer overflows on narrow screens.
- On **mobile** those three `IconButton`s are hidden (`if (!isMobile)`) and
  surfaced instead as items in the existing `PopupMenuButton` (`rescan` /
  `backup` / `remove` cases), so the same actions stay reachable from the
  overflow menu. "Remove Account" keeps its error-color styling.
- On **desktop/wide** the icon buttons remain as before.

### `lib/settings.dart` — stack the Light Node Server field on mobile

- The "Light Node Server" control was a fixed `Row` (label + a 360px-wide
  dropdown), which got cramped on small screens.
- It's now wrapped in a `Builder` that branches on the same `isMobile` check:
  - **Mobile:** the dropdown is returned on its own, full-width, with the label
    moved inline as the field's `labelText` (no separate label column).
  - **Desktop/wide:** the label-plus-dropdown `Row` is kept; the dropdown box
    shrank slightly (360 → 324px) to fit alongside the label.
