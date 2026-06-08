# CHANGES_4 — Receive Funds: derivation paths + "Previous Set of Addresses"

Summary of the changes.

Two features were added to the **Receive Funds** page:

1. Each displayed address now shows its **BIP-44 / ZIP-32 derivation path**.
2. A new **"Previous Set of Addresses"** button (between *Sweep* and *Next Set
   of Addresses*) that moves the account's active diversifier index backward,
   the inverse of the existing *Next Set of Addresses*.

---

## Files changed

```
 M lib/pages/receive.dart            +95  -5    (UI: paths + Prev button)
 M lib/src/rust/api/account.dart     +3   -0    (Dart wrapper: generatePrevDindex)
 M lib/src/rust/frb_generated.dart   +29  -0    (FRB binding)
 M rust/src/account.rs               +116 -0    (generate_prev_dindex)
 M rust/src/api/account.rs           +7   -0    (API wrapper)
 M rust/src/frb_generated.rs         +39  -0    (FRB wire impl + dispatch)
 6 files changed, 284 insertions(+), 5 deletions(-)
```

---

## 1. Derivation paths on the Receive Funds page

**File:** `lib/pages/receive.dart`

Added a small, monospace, hint-colored line under each address showing how it
was derived. Computed entirely in Dart from data already available on the page
(`account.aindex`, `addresses.diversifierIndex`, and the network name), so **no
Rust change was required** for this part.

- Detects the coin type from the active network (`getNetworkName`): mainnet →
  `133'`, test/regtest → `1'`.
- **Transparent** path: `m/44'/{coinType}'/{aindex}'/0/{diversifierIndex}`
  (matches `derive_transparent_address` in `rust/src/account.rs`).
- **Shielded** (Sapling / Orchard / Unified) path: `m/32'/{coinType}'/{aindex}'`,
  with the diversifier index shown separately (the diversifier selects the
  receiver rather than being a path level; matches `usk.sapling()` /
  `usk.orchard()`).
- Wired into all four address tiles: Unified Address, Orchard-only, Sapling,
  and Transparent (the transparent tile uses the BIP-44 line; the rest use the
  ZIP-32 line).

New imports/helpers: `getNetworkName`, `coinType` state field,
`transparentPath()`, `shieldedPath()`, `derivationLabel()`, `derivationInfo()`.

---

## 2. "Previous Set of Addresses" button

### UI — `lib/pages/receive.dart`

- New `derivePrevID` showcase key.
- New AppBar `IconButton` (`Icons.skip_previous`, tooltip
  *"Previous Set of Addresses"*) placed **between** the Sweep button and the
  Next Set button.
- New `onPrevAddress()` handler: guards against going below index 0 (shows
  *"Already at the first set of addresses"* snackbar), otherwise calls
  `generatePrevDindex(c:)`, refreshes the displayed addresses, and rebuilds.
- Added `derivePrevID` to the tutorial showcase list.

### Dart wrapper — `lib/src/rust/api/account.dart`

```dart
Future<int> generatePrevDindex({required Coin c}) =>
    RustLib.instance.api.crateApiAccountGeneratePrevDindex(c: c);
```

### Rust API wrapper — `rust/src/api/account.rs`

```rust
#[cfg_attr(feature = "flutter", frb)]
pub async fn generate_prev_dindex(c: &Coin) -> Result<u32> {
    let mut connection = c.get_connection().await?;
    crate::account::generate_prev_dindex(&c.network(), &mut connection, c.account).await
}
```

### Rust core — `rust/src/account.rs`

Added `generate_prev_dindex`, the inverse of `generate_next_dindex`:

- Reads `(aindex, dindex)` for the account.
- If `dindex == 0`, returns `0` unchanged (already at the first set).
- For Sapling-enabled accounts, scans **downward** from `dindex - 1` for the
  closest **valid** Sapling diversifier (skipping invalid indices), floored at
  0, and re-points the stored Sapling address. On Ledger (`hw != 0`) it accepts
  the index via `get_hw_next_diversifier_address` since validity can't be
  cheaply tested locally.
- For accounts without Sapling, simply decrements by 1.
- Persists the new `dindex`, then ensures the matching transparent receive
  address row exists (derives/stores it, including the Ledger path).

This mirrors `generate_next_dindex`'s structure and DB updates exactly, in
reverse.

---

## 3. flutter_rust_bridge bindings

> **Note:** the FRB generator (`flutter_rust_bridge_codegen generate`) runs
> `cargo expand`, which requires the Rust crate to compile. **These bindings must
> be regenerated with
> `RUSTFLAGS='--cfg zcash_unstable="nu7"' flutter_rust_bridge_codegen generate`
> on a working build environment before shipping**, per the repo rule that
> generated files (`frb_generated.rs`, etc.) are not edited by hand.

- **`rust/src/frb_generated.rs`**:
  - `wire__crate__api__account__generate_prev_dindex_impl` (mirrors the
    `generate_next_dindex` wire impl).
  - Dispatcher arm `135 => wire__...generate_prev_dindex_impl(...)`.
- **`lib/src/rust/frb_generated.dart`**:
  - Abstract method `crateApiAccountGeneratePrevDindex({required Coin c})`.
  - Impl using `funcId: 135`, `sse_decode_u_32` success / `AnyhowException`
    error, and `const meta` with `debugName: "generate_prev_dindex"`.

**funcId choice:** `135` was used because the current maximum funcId is `134`.
Assigning the next free integer (rather than inserting mid-sequence) means **no
existing function indices had to be renumbered**, keeping the wire
mapping self-consistent across the Dart and Rust dispatchers. The new function
takes only a `Coin` argument and returns `u32`, both of which reuse existing
serializers (`sse_encode_box_autoadd_coin`, `sse_decode_u_32`) — so no new wire
types were needed.

---

## Verification status

- `flutter analyze` on the touched Dart files: **0 errors** (only pre-existing
  `info`-level lints — trailing commas in generated code, `use_build_context_synchronously`
  in unchanged handlers).
- `cargo check` / full build: should be verified with a real build + codegen on
  a working environment. The Rust changes were written to mirror existing,
  compiling code.

### Follow-up required before commit/ship

1. Run `flutter_rust_bridge_codegen generate` on a working build env and confirm
   it produces the same `generate_prev_dindex` bindings (funcId may differ — let
   codegen own it).
2. `cargo build` / `cargo check` with `RUSTFLAGS='--cfg zcash_unstable="nu7"'`.
3. Manually test Next → Previous round-trips, including the Sapling
   invalid-diversifier skip case and the index-0 floor.
