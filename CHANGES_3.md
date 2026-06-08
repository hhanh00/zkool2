# Changes 3

Summary of changes. These are Dart-only changes — no generated
bindings (`frb_generated.rs`, `*.freezed.dart`, `*.g.dart`) were touched, so no
codegen/RUSTFLAGS rebuild is required.

## 1. Zkool theme + theme mode selection

Added a "Zkool" theme option that restores the original pink/Material light look
that existed before the gold Zcash theme (pre-commit `8fe8f35e`), alongside the
existing gold light/dark themes.

- `lib/theme_mode.dart`
  - New `AppTheme { zkool, dark, light, system }` enum, replacing the direct use
    of Flutter's `ThemeMode` for the persisted preference (still stored under the
    `theme_mode` SharedPreferences key; default remains **dark**).
  - New `zkoolPinkTheme` (Material Pink `#E91E63` seed, light brightness).
  - Helpers `themeModeFor(AppTheme)` and `lightThemeFor(AppTheme)` map the
    selection to a `ThemeMode` + the light `ThemeData` to install.
  - `ThemeModeNotifier` / `themeModeProvider` now hold `AppTheme`.
- `lib/main.dart` — `MaterialApp.router` now derives `themeMode` and `theme`
  from the selected `AppTheme` (dark theme slot unchanged).
- `lib/settings.dart` — Theme dropdown now offers **Zkool / Dark / Light /
  System**.

## 2. "Max" button fee fix (Recipient Pays Fee)

Fixes "Not enough funds, 0.00015 more ZEC required" when sending the full
balance via the Max button. No Rust changes — reuses the existing
`recipientPaysFee` flag.

- `lib/pages/send.dart`
  - `SendPage` tracks a `maxSelected` flag: set when **Max** is clicked, cleared
    when the amount is manually edited afterwards, and reset on form clear.
  - `onSend` passes `(recipients, maxSelected)` to the Extra Options page.
  - `Send2Page` gained a `maxSelected` parameter; when true the **Recipient Pays
    Fee** switch defaults to on, so the fee is deducted from the spend amount.
- `lib/router.dart` — `/send2` route accepts either a bare `List<Recipient>` or a
  `(List<Recipient>, bool)` record.

## 3. Backup seed & keys page

- `lib/pages/account.dart`
  - New **Backup Seed & Keys** toolbar icon (`Icons.save`) between *Rescan from
    Height* and *Remove Account*; opens after `authenticate()`.
  - New `BackupPage` showing the seed phrase, passphrase, account index, unified
    viewing key, and fingerprint (reuses `getAccountSeed` / `getAccountUfvk` /
    `getAccountFingerprint` / `getAccountPools`).
- `lib/router.dart` — new `/backup` route.

## 4. Transaction TXID popup returns to account page

- `lib/pages/tx.dart` — after a successful broadcast and acknowledging the TXID
  dialog, navigate back to `/account`.

## 5. Transparent-address scan default gap limit

- `lib/store.dart` — `TransparentScan.gapLimit` default changed from **40** to
  **20**.

## 6. Transparent-scan dialog navigation + refresh

- `lib/pages/sweep.dart`
  - On **Scan Completed** and on **Close**, invalidate `getAccountsProvider` (so a
    newly imported transparent account shows immediately) and navigate to the
    account list `/` (matching the SEED PHRASE popup behavior), instead of
    `pop()`.

## 7. Light Node Server dropdown width

- `lib/settings.dart` — Light Node Server dropdown widened from `180` to `360`
  (~2×).

## 8. Block Explorer dropdown

- `lib/settings.dart`
  - Replaced the free-text Block Explorer field with a dropdown of named
    explorers; the URL text field is shown only for **Custom Explorer**:
    - `zcashexplorer.app` → `https://{net}.zcashexplorer.app/transactions/{txid}`
    - `zcashinfo.com` → `https://zcashinfo.com/tx/{txid}`
    - `cipherscan.app` → `https://cipherscan.app/tx/{txid}` **(default)**
    - Custom Explorer → reveals a free-form URL field
  - Stored in the existing `blockExplorer` property; `openBlockExplorer` already
    substitutes `{net}`/`{txid}` (templates without `{net}` are mainnet-only).
- `lib/store.dart` — default `blockExplorer` changed to
  `https://cipherscan.app/tx/{txid}`.

## 9. Confirmation count in Transaction History

- `lib/pages/account.dart`
  - `showTxHistory` computes per-tx confirmations
    (`currentHeight - tx.height + 1`, clamped ≥ 0, only for mined txs) using
    `currentHeightProvider` and passes it to each tile.
- `lib/widgets/theme.dart`
  - `TransactionTile` gained an optional `confirmations`; the title renders as
    e.g. `Sent ( 2 conf )` where the `( N conf )` suffix is **80%** of the label
    font size and slightly muted.
