# Changes — Summary (CHANGES_1)

Overview of the changes.

## Files changed
```
 M lib/main.dart
 M lib/pages/account.dart
 M lib/pages/accounts.dart
 M lib/pages/new_account.dart
 M lib/settings.dart
 M lib/utils.dart
 M lib/widgets/theme.dart
?? lib/theme_mode.dart    (new)
```

---

## 1. Biometric "NoHardware" no longer blocks Settings
**File:** `lib/utils.dart` (`authenticate()`)

- Added `case "NoHardware":` and `case auth_error.biometricOnlyNotSupported:` to the
  error switch. On devices with no biometric hardware, instead of showing a blocking
  error dialog and denying access, it now shows a non-blocking snackbar
  ("Biometric lock unavailable on this device - access not protected") and returns
  `true` so the user can still open Settings.
- Genuine failures (cancellation, lockout, unknown codes) still hit `default` and deny.

## 2. Theme system: Zcash dark-yellow + theme switcher
**Files:** `lib/theme_mode.dart` (new), `lib/main.dart`, `lib/settings.dart`

- **`lib/theme_mode.dart` (new):**
  - `themeModeProvider` — a non-generated `NotifierProvider<ThemeMode>` that defaults
    to **Dark** on first launch and persists the user's choice via
    `SharedPreferences` (`theme_mode` key), honored on subsequent launches.
  - `zcashDarkTheme` — charcoal surfaces (`#121212`/`#1C1C1C`) with the Zcash gold
    accent **`#F4B728`** applied to AppBar foreground, dividers, tab bar, progress
    indicators, switches and icons.
  - `zcashLightTheme` — `ColorScheme.fromSeed` on the same gold for brand consistency.
- **`lib/main.dart`:** `MaterialApp.router` is now wrapped in a `Consumer`; its
  `themeMode` watches `themeModeProvider`, with `theme: zcashLightTheme` /
  `darkTheme: zcashDarkTheme` (replacing the previous hard-coded
  `ThemeMode.system` + `ThemeData.light()/dark()`).
- **`lib/settings.dart`:** Added a **Theme** dropdown (Dark / Light / System).
  It is placed at the **top** of the settings form (above "Light Node").

## 3. Dark-mode account cards: outline instead of gradient
**File:** `lib/widgets/theme.dart` (`DisplayPanel`)

- In dark mode the panel now renders a flat `surface` card with a gold accent
  **outline** (no gradient, no shadow). Light mode keeps the original gradient/shadow.
- Affects the account cards on the launch page and other panels using `DisplayPanel`.

## 4. Account list: long-press context menu + card spacing
**File:** `lib/pages/accounts.dart`

- **Long-press menu:** Long-pressing an account row (`onLongPressStart`, anchored at
  the press position via `showMenu`) opens a popup menu with:
  - **Rename Account** — `updateAccount(name:)` + vault sync.
  - **Freeze / Unfreeze Account** — toggles `enabled` (frozen accounts are excluded
    from auto-sync and "sync all").
  - **Rescan from Height** — height-only dialog (prefilled with birth height);
    sets `birth` then `resetSync(id)` to clear tx history + UTXO/notes and re-sync.
  - **Remove Account** — confirm dialog → `deleteAccount`.
- **Spacing:** Each account card now has `8px` vertical padding (~16px gap between
  cards). The reorder `ValueKey` moved to the wrapping `Padding` so drag-and-drop
  reordering still works.
- Added `import 'package:flutter/services.dart';` (for `FilteringTextInputFormatter`).

## 5. Account view page: Rescan & Remove actions in the AppBar
**File:** `lib/pages/account.dart`

- Added **Rescan from Height** (`restart_alt`) and **Remove Account** (red `delete`)
  as dedicated AppBar icon buttons, positioned **between "Sync this account" and
  "Receive Funds"** (deliberately away from "Send Funds" to avoid disrupting the
  send flow). Both are disabled until account data loads.
- `onRescan(account)` — same set-birth + `resetSync` flow as the list menu.
- `onRemove(account)` — confirm → `deleteAccount` → navigate back to the account list
  (`GoRouter.go("/")`).
- The overflow (⋮) menu now holds only Fetch Tx Prices / Export / Charts / ZSA.

## 6. New account: default Restore birth height to latest block
**File:** `lib/pages/new_account.dart` (`onSave`)

- When "Restore" is on and the birth-height field is left empty, it now fetches the
  current tip height fresh via `getCurrentHeight` (and updates `currentHeightProvider`)
  instead of falling back to a low/stale value, then uses that as the birth height.
- Added `import 'package:zkool/src/rust/api/network.dart';` for `getCurrentHeight`.

---

## Verification
- `flutter analyze` on all touched `lib/` files reports **0 errors / 0 warnings**;
  remaining items are pre-existing `info`-level style lints (trailing commas,
  `use_build_context_synchronously`) consistent with the rest of the codebase.
- No Rust or code-generation changes required (theme uses a plain `NotifierProvider`;
  all wallet operations use existing FRB APIs: `updateAccount`, `resetSync`,
  `deleteAccount`, `getCurrentHeight`).
