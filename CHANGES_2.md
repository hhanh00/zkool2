# Commit "Change 2" (6ccaa12) — Summary (CHANGES_2)

Overview of the changes in the latest commit. This commit implements six
feature requests plus supporting plumbing (proxy routing for the
Zebra full-node client, a portable-data-directory helper, and a small
server-list sync tool).

## Files changed
```
 Cargo.lock
 assets/tor.svg                     (new)
 lib/main.dart
 lib/pages/account.dart
 lib/pages/accounts.dart
 lib/pages/db.dart
 lib/pages/new_account.dart
 lib/settings.dart
 lib/src/rust/api/coin.dart
 lib/src/rust/api/coin.freezed.dart
 lib/src/rust/api/network.dart
 lib/src/rust/api/transaction.dart
 lib/src/rust/frb_generated.dart
 lib/store.dart
 lib/store.freezed.dart
 lib/store.g.dart
 lib/utils.dart
 lib/vault.dart
 lib/widgets/input_amount.dart
 pubspec.lock
 pubspec.yaml
 rust/Cargo.toml
 rust/src/account.rs
 rust/src/api/coin.rs
 rust/src/api/network.rs
 rust/src/api/transaction.rs
 rust/src/budget.rs
 rust/src/frb_generated.rs
 rust/src/net/zebra.rs
 servers/.gitignore                 (new)
 servers/README.md                  (new)
 servers/package.json               (new)
 servers/sync.ts                    (new)
 servers/tsconfig.json              (new)
 windows/CMakeLists.txt
```

---

## 1. Mnemonic import used the wrong receive-address index
**Files:** `rust/src/account.rs`, `rust/src/frb_generated.rs` (regen)

- When importing a wallet by mnemonic, the first receive address came out as
  `m/44'/133'/0'/0/3` (or `.../0/7`) instead of `.../0/0`.
- **Root cause:** the diversifier index (`dindex`) was derived from the *unified*
  full viewing key's `default_address()`. The unified address is dominated by the
  Orchard receiver, whose default valid diversifier is a non-zero value (3, 7, …),
  so the Sapling receive address inherited that non-zero index.
- **Fix:** force diversifier index **0**. `default_address()` is wrong here too
  because it searches *forward* from 0 for the first valid index — for Sapling
  (where ~half of indices are invalid) it routinely returns a non-zero value.
  Instead:
  - **Transparent & Orchard** addresses are valid at every index, so they always
    use index `0`.
  - **Sapling** uses `find_address(0)`, which returns index `0` whenever it is
    valid for the key (the common case) and only falls back to the next valid
    index in the rare case that diversifier 0 is invalid.
  - The shared account diversifier index stored via `update_dindex` (and used by
    `get_addresses` for all pools) is therefore `0` in the normal case, giving a
    receive address of `m/44'/133'/aindex'/0/0`. The BIP44 account index
    (`aindex'`) was already correct and follows the Account Index field
    (default 0).
- **NOTE:** this is a Rust change, so it only takes effect after the Rust crate is
  recompiled (`cargo build` / `flutter build`). Re-importing the mnemonic with the
  old (un-recompiled) binary will still show the old `.../0/3` address.

## 2. socks5h:// proxy support (remote DNS) + Tor default
**Files:** `rust/src/api/coin.rs`, `rust/src/net/zebra.rs`, `lib/settings.dart`

- **`coin.rs` — light-node (gRPC) path:** the SOCKS handling is split so the two
  schemes behave correctly:
  - `socks5h://` — the hostname is sent to the proxy, which resolves DNS
    *remotely*. This is what makes `.onion` light-wallet servers reachable.
  - `socks5://` — the host is resolved *locally* (`tokio::net::lookup_host`) and the
    resulting IP is handed to the SOCKS proxy.
- **`zebra.rs` — full-node (JSON-RPC) path:** `ZebraClient::new` now takes a
  `proxy` argument. When set, the `reqwest::Client` is built with
  `Proxy::all(proxy)`; reqwest natively understands `socks5`, `socks5h`, `http`
  and `https` proxy URLs, so `.onion` Zebra endpoints work through Tor as well.
- **`settings.dart` — Tor default:** the Tor button now fills in a `socks5h://`
  URL by default (`socks5h://127.0.0.1:9150` on Windows, `:9050` elsewhere) so
  `.onion` servers resolve correctly out of the box.

## 3. Tor onion logo (SVG icon) on Settings
**Files:** `assets/tor.svg` (new), `lib/settings.dart`, `pubspec.yaml`

- Added **`assets/tor.svg`** — an onion-mark icon (concentric onion rings with a
  vertical stem) drawn with `stroke="currentColor"` so it can be tinted to match
  the theme.
- Added the `flutter_svg` dependency and registered `assets/tor.svg` in the asset
  bundle (`pubspec.yaml`).
- In **`settings.dart`** the proxy button now renders the Tor onion via
  `SvgPicture.asset("assets/tor.svg", colorFilter: ColorFilter.mode(primary,
  BlendMode.srcIn))` instead of the previous Material icon.

## 4. Fiat currency conversion + currency dropdown
**Files:** `rust/src/api/network.rs`, `rust/src/budget.rs`,
`rust/src/api/transaction.rs`, `rust/src/frb_generated.rs`,
`lib/src/rust/api/network.dart`, `lib/src/rust/api/transaction.dart`,
`lib/src/rust/frb_generated.dart`, `lib/store.dart`, `lib/store.freezed.dart`,
`lib/store.g.dart`, `lib/utils.dart`, `lib/settings.dart`,
`lib/widgets/input_amount.dart`, `lib/pages/accounts.dart`, `lib/pages/account.dart`

- **Rust price fetch (`network.rs`):** `get_coingecko_price` now takes a
  `currency` argument, lowercases it, requests
  `…?ids=zcash&vs_currencies={cur}&x_cg_demo_api_key={api}`, and reads the price
  out of the dynamic JSON via `.pointer("/zcash/{currency}")`. The old fixed
  `Usd`/`ZcashUSD` structs were removed.
- **Historical / tx prices (`budget.rs`, `transaction.rs`):**
  `get_historical_prices_all`, `get_historical_prices` and `fill_missing_tx_prices`
  thread the selected `currency` through (CoinGecko `vs_currency={currency}`).
- **FRB bindings:** `get_coingecko_price` and `fill_missing_tx_prices` gained a
  second `String currency` argument, wired through the generated Dart and Rust
  bindings (encode/decode in arg order: `api` → `currency` [→ `c`]).
- **Settings model (`store.dart`):** `AppSettings` gained an `fxCurrency` field,
  persisted via SharedPreferences (`fx_currency`, default `usd`) and pushed into
  the `PriceNotifier` auto-fetch.
- **Settings UI (`settings.dart`):** added a small **Market Price Currency**
  dropdown (compact, `onAccountMenu`-style) listing the 15 supported currencies
  (BTC, USD, CNY, EUR, JPY, GBP, INR, RUB, BRL, CAD, AUD, MXN, KRW, TRY, VND).
- **Display (`utils.dart`, `input_amount.dart`, `accounts.dart`, `account.dart`):**
  added `fxCurrencies` + an `fxSymbol()` helper; all hard-coded "USD"/"$" labels
  now use the selected currency code and symbol (amount-entry label, account-list
  price button + fiat value, and per-tx fiat values).

## 5. Light Node Server dropdown made compact
**File:** `lib/settings.dart`

- The Light Node Server dropdown was oversized. It is now a `Row` with an
  `Expanded` "Light Node Server" label plus a fixed-width (~180px) dense
  `FormBuilderDropdown` (`isExpanded: true`, `OutlineInputBorder`, ellipsized
  items + a "Custom…" entry), matching the small `onAccountMenu`-style sizing used
  by the new currency dropdown.

## 6. Portable build: ship `zkool_portable.exe`
**Files:** `windows/CMakeLists.txt`, `lib/utils.dart`, `lib/main.dart`,
`lib/pages/db.dart`, `lib/vault.dart`

- **`windows/CMakeLists.txt`:** after `install(TARGETS …)`, an `install(CODE …)`
  step runs `${CMAKE_COMMAND} -E copy_if_different` to copy `zkool.exe` →
  `zkool_portable.exe` in the same output directory, so `flutter build windows`
  produces both binaries side by side. (Uses `execute_process` + `copy_if_different`
  for CMake 3.14 compatibility rather than `file(COPY_FILE)`, which needs 3.21.)
- **`utils.dart`:** added `isPortable` (true when the running executable is named
  `zkool_portable…`), `getDataDirectory()` and a `joinPath()` helper. The portable
  build stores its data in a local `./db` directory next to the executable; the
  normal build keeps using the OS application-documents directory.
- **`main.dart`, `db.dart`, `vault.dart`:** switched from
  `getApplicationDocumentsDirectory()` to the new `getDataDirectory()` so the data
  dir, the DB-manager listing and the vault key files all honor portable mode.

---

## Supporting tooling: `servers/` (new)
**Files:** `servers/README.md`, `servers/package.json`, `servers/sync.ts`,
`servers/tsconfig.json`, `servers/.gitignore`

- A small standalone TypeScript tool that fetches the current Zcash light-wallet
  server list from `hosh.zec.rocks` and writes it to `servers.json` (with each
  server's full URL plus its uptime-table metadata).
- Supports `--online-only` (only currently-online servers) and `--no-tor`
  (exclude `.onion` servers); run via `npm run sync` / `tsx sync.ts` or compiled
  with `npm run build && npm start`. This is build-time/maintenance tooling and is
  not part of the shipped app.

---

## Verification
- All touched `lib/` files pass `dart analyze` with **0 errors**; remaining items
  are pre-existing `info`-level style lints (trailing commas, etc.).
- **Rust + FRB bindings (account.rs dindex, coin.rs/zebra.rs proxy,
  network.rs/budget.rs/transaction.rs currency, and the regenerated
  `frb_generated` wire) and the new CMake portable-exe install step should be
  verified with a full `cargo check --features flutter` / `flutter build windows`**
  on a working build environment.
