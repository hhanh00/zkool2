# Building zkool for Windows locally (MSVC toolchain)

This guide explains how to compile zkool and produce **`zkool-%VERSION%.zip`**
on your own Windows machine using `build-win.ps1` — the **native MSVC** path that
**does not need MSYS2**.

> Prefer MSYS2/GNU? Use `build-win.ps1` (see `build-win.md`). This MSVC variant is
> an independent alternative; pick whichever toolchain your machine already has.

## How this works (same as CI, plus prebuilt OpenSSL)

CI builds the Windows app by letting **Flutter/cargokit** compile the Rust crate
with the **MSVC** toolchain during `flutter build windows` (driven by `fastforge`).
It works on GitHub's `windows-latest` runners because they ship **Perl + NASM**, so
the `vendored-openssl` feature can build OpenSSL from C source. Locally we usually
don't have those — and Strawberry Perl in particular is incompatible with the
current Rust openssl wrapper.

`build-win.ps1` builds **exactly the way CI does** — cargokit compiles the Rust,
no bypass — and changes only **one** thing: it injects `OPENSSL_*` environment
variables so `openssl-sys` links a **prebuilt vcpkg OpenSSL** instead of compiling
it. The key facts that make this work:

- **`OPENSSL_NO_VENDOR=1`** makes `openssl-sys` consume the prebuilt copy instead of
  vendoring. That env var overrides the forced `vendored` Cargo feature at build
  time, so `Cargo.toml` stays untouched.
- cargokit runs `rustup run stable cargo build --target x86_64-pc-windows-msvc
  --features=…` and passes the **current process environment straight through** to
  cargo. So setting `OPENSSL_*` + `RUSTFLAGS` in the PowerShell process is all cargo
  needs — no bypass, no manual cargo build, no DLL planting.
- The feature list comes from **`rust/cargokit.yaml`** (`cargo.release.extra_flags`),
  exactly like CI's `misc/mkcargokit_options.sh ledger > rust/cargokit.yaml`. The
  script writes that file.

In short (the OpenSSL install is a one-time **prerequisite**, done before running
the script):

0. **Prerequisite:** `vcpkg install openssl:x64-windows-static-md` (one-time; vcpkg
   uses **its own** bundled Perl — you install nothing). The script verifies this
   is present but does **not** install it.
1. Write `rust/cargokit.yaml` so cargokit builds with `--features=ledger`.
2. Prepend the VS x64 tools to `PATH` (so cargo's link step uses MSVC's `link.exe`,
   not another `link.exe` on `PATH`), and set `OPENSSL_*` + `RUSTFLAGS` — all for the
   script's process only.
3. `flutter build windows --release` → cargokit compiles `rlz.dll` with MSVC, linking
   the vcpkg OpenSSL (**no Perl, no NASM**).
4. Zip the Release folder as `out\zkool-%VERSION%.zip`.

> The script changes **nothing globally**. The `PATH` tweak and the `OPENSSL_*` /
> `RUSTFLAGS` variables apply to its **own process only**, and `rust/cargokit.yaml`
> is the same file CI generates.

---

## 1. Prerequisites

| Tool | Why | How |
|------|-----|-----|
| **Rust (rustup) + `x86_64-pc-windows-msvc` target** | Builds the native `rust/` crate for the MSVC ABI. | The script auto-runs `rustup target add x86_64-pc-windows-msvc` if it's missing. |
| **Visual Studio 2022** with **"Desktop development with C++"** | Provides `cl.exe` / `link.exe` / the Windows SDK / CMake for the msvc target **and** Flutter's runner. | Install via the VS Installer. *Community* / *Build Tools* edition is fine. |
| **VS "C++ Clang tools for Windows" (ClangCL)** | The `rive_common` plugin forces the MSBuild **ClangCL** platform toolset. Without it the build fails with `MSB8020`. | VS Installer → Modify → **Individual components** → add *"C++ Clang Compiler for Windows"* **and** *"MSBuild support for LLVM (clang-cl) toolset"*. |
| **vcpkg** | Supplies a **prebuilt OpenSSL** so we don't build it from source (no Perl/NASM). | `git clone https://github.com/microsoft/vcpkg C:\vcpkg` then `C:\vcpkg\bootstrap-vcpkg.bat`. Pass `-VcpkgRoot` if elsewhere. |
| **OpenSSL via vcpkg (`x64-windows-static-md`)** | The MSVC-ABI OpenSSL that `rlz.dll` links against. **The script verifies but does not install it.** | Run **once**: `C:\vcpkg\vcpkg.exe install openssl:x64-windows-static-md` (~10–20 min; vcpkg uses its own Perl). Use `x64-windows-static-md`, **not** `x64-windows-static`. |
| **Flutter SDK 3.44.1** | Builds the Windows app. | The script **clones Flutter 3.44.1 into `build-win\flutter`** automatically (like `zwallet`); you only need `git` on PATH. Pass `-FlutterDir` to reuse an existing SDK, or `-FlutterVersion ""` to use whatever `flutter` is on PATH. |
| **git** | Used to clone the Flutter SDK. | https://git-scm.com — add to PATH. |

> **Install OpenSSL before the first build.** From PowerShell:
> ```powershell
> C:\vcpkg\vcpkg.exe install openssl:x64-windows-static-md
> ```
> This is a one-time step. `build-win.ps1` checks that
> `<vcpkg>\installed\x64-windows-static-md\{include,lib}` exists and stops with this
> exact command if it doesn't — it never installs OpenSSL for you.

> **Why no Strawberry Perl / NASM / MSYS2 here?** Setting `OPENSSL_NO_VENDOR=1`
> makes `openssl-sys` consume vcpkg's prebuilt OpenSSL instead of compiling it from
> C source — and compiling from source is the only thing that needed Perl/NASM.
> vcpkg *does* use Perl to build OpenSSL, but it ships its own copy for that
> one-time step, so you never install one. No MSYS2 is involved at all.

Verify your setup before building (from PowerShell):

```powershell
rustup show                         # any host is fine; msvc target is auto-added
flutter doctor                      # "Windows" + "Visual Studio" checks green
cargo --version
Test-Path C:\vcpkg\vcpkg.exe        # must be True (or wherever you cloned vcpkg)
# OpenSSL prerequisite must be installed (see above):
Test-Path C:\vcpkg\installed\x64-windows-static-md\lib\libssl.lib   # must be True
```

---

## 1b. Fast Dart pre-flight (`flutter.ps1`)

Before the slow native build, run the **Dart-only** verification with
`flutter.ps1`. It compiles **no** Rust and needs **no** MSVC / vcpkg / OpenSSL —
it just runs `flutter pub get` and `flutter analyze` (and, optionally,
`dart run build_runner`). This surfaces Dart mistakes in **seconds** instead of
after a ~10-minute cargokit Rust compile.

```powershell
.\flutter.ps1                 # flutter pub get + flutter analyze
.\flutter.ps1 -BuildRunner    # + regenerate freezed/riverpod (*.g.dart/*.freezed.dart)
.\flutter.ps1 -Watch          # build_runner in watch mode (Ctrl-C to stop)
.\flutter.ps1 *>&1 | Tee-Object -FilePath build.log # flutter with logging
```

**When to use `-BuildRunner`:** only when you changed `@freezed` / `@riverpod`
declarations (or any annotation that drives generated `*.g.dart` / `*.freezed.dart`).
Adding plain functions / widgets / enums does **not** need it. To regenerate the
**flutter_rust_bridge** bindings instead, use `codegen.ps1` (see §4b) — that's a
different generator.

`flutter.ps1` provides Flutter the **same way** as `build-win.ps1` (clones the
pinned `3.44.1` into `build-win\flutter`, or `-FlutterDir <sdk>`, or
`-FlutterVersion ""` to use one on PATH), so the two scripts share the SDK.

> **Analyze is scoped to `lib\` (+ `test\`).** It deliberately does **not** analyze
> `build-win\flutter` (the cloned SDK) or `rust_builder\cargokit` (cargokit's
> internal `build_tool`, which has unresolved deps). Analyzing the whole tree from
> the repo root otherwise reports **thousands** of bogus issues from those
> third-party trees. `analysis_options.yaml` excludes them too, so editor/IDE
> analysis stays clean as well.

Recommended order whenever you've touched Dart and/or regenerated bindings:

```powershell
.\flutter.ps1        # 1. cheap: pub get + analyze (fix any Dart errors here)
.\build-win.ps1      # 2. full native build once analyze is clean
```

> If you only changed Dart (no Rust API change), you can skip `codegen.ps1` and go
> straight `flutter.ps1` → `build-win.ps1`.

**Options:**

```powershell
.\flutter.ps1 -SkipPubGet                 # repeat runs; skip pub get
.\flutter.ps1 -SkipAnalyze -BuildRunner   # only regenerate code, don't analyze
.\flutter.ps1 -FlutterDir C:\flutter      # reuse an existing SDK
.\flutter.ps1 -FlutterVersion ""          # use whatever 'flutter' is on PATH
```

If PowerShell blocks the script, run it for the current process only:

```powershell
powershell -ExecutionPolicy Bypass -File .\flutter.ps1
```

---

## 2. Build

Open **PowerShell**, `cd` into the project root (where `pubspec.yaml` is), and run:

```powershell
.\build-win.ps1
```

If PowerShell blocks the script with an execution-policy error, run it for the
current process only:

```powershell
powershell -ExecutionPolicy Bypass -File .\build-win.ps1
```

The script will:

1. Clone Flutter (`3.44.1`) into `build-win\flutter` (first run only) and prepend
   its `bin\` to PATH for this process only. (Use `-FlutterDir` / `-FlutterVersion ""`
   to skip cloning — see §4.)
2. Check tools, locate Visual Studio (via `vswhere`), verify the ClangCL toolset,
   and add the `x86_64-pc-windows-msvc` Rust target if needed.
3. Read the version (`%VERSION%`) from `pubspec.yaml`.
4. **Verify** `openssl:x64-windows-static-md` is already installed via vcpkg (it
   stops with the install command if not — it does **not** install it).
5. Write `rust/cargokit.yaml` with `--features=ledger` (same as CI's
   `mkcargokit_options.sh`).
6. Prepend the VS x64 tools to `PATH` and set `OPENSSL_NO_VENDOR=1` +
   `OPENSSL_DIR`/`OPENSSL_STATIC`/`OPENSSL_LIBS` + `RUSTFLAGS` — process-scoped only.
7. `flutter build windows --release` — cargokit compiles the Rust
   (`rustup run stable cargo build --target x86_64-pc-windows-msvc --features=ledger`),
   linking the vcpkg OpenSSL. cargokit bundles `rlz.dll` into the Release folder.
8. Zip the Release folder to **`out\zkool-%VERSION%.zip`**.

The **first build is slow** — cargokit compiles the whole zcash dependency tree
(~10+ minutes). Subsequent builds reuse the cargo cache. (The one-time
`vcpkg install openssl` is done separately as a prerequisite, see §1.)

---

## 3. Output

When it finishes you'll see:

```
out\zkool-%VERSION%.zip
```

That zip's root contains the runnable Windows build — `zkool.exe`, its DLLs
(including `rlz.dll`), and the `data\` folder. Unzip it anywhere and run
`zkool.exe`. Because OpenSSL and SQLCipher are **statically** linked into
`rlz.dll`, there are no extra OpenSSL DLLs to ship.

---

## 4. Options

```powershell
# Skip `flutter pub get` on repeat builds:
.\build-win.ps1 -SkipPubGet

# vcpkg installed somewhere other than C:\vcpkg:
.\build-win.ps1 -VcpkgRoot D:\vcpkg

# Use a different OpenSSL triplet (NOT recommended — see troubleshooting):
.\build-win.ps1 -OpensslTriplet x64-windows-static-md

# Build with the crate's default features only (writes empty extra_flags):
.\build-win.ps1 -Feature ""

# Use an already-installed Flutter SDK instead of cloning one:
.\build-win.ps1 -FlutterDir C:\flutter

# Clone a different Flutter version (branch/tag; default 3.44.1):
.\build-win.ps1 -FlutterVersion 3.27.0

# Use whatever 'flutter' is already on PATH (no clone):
.\build-win.ps1 -FlutterVersion ""
```

> The script clones Flutter into `build-win\flutter` on first run; delete that
> folder to force a fresh re-clone. Your globally-installed Flutter (if any) is
> left untouched.

> To force a full Rust recompile, delete `target\x86_64-pc-windows-msvc\` (or the
> whole `build\` + `target\`) before running.

---

## 4b. Regenerating the flutter_rust_bridge bindings (`codegen.ps1`)

When you change the Rust API surface under `rust/src/api/` — adding/removing a
`pub` function, or changing a signature like `Coin::open_database` — you must
regenerate the **flutter_rust_bridge (FRB)** bindings so the Dart side matches the
Rust side. The generated files are:

- `lib/src/rust/**` (Dart bindings, e.g. `lib/src/rust/api/coin.dart`)
- `rust/src/frb_generated.rs` (Rust wire glue)

> **Do not hand-edit the generated files** (see `CLAUDE.md`). Regenerate them with
> `codegen.ps1` instead.

`flutter_rust_bridge_codegen generate` runs `cargo expand` over the crate to
discover its API, which means it must **compile** the crate — so it needs the
**same toolchain + environment as a real build**. In particular, without the
`OPENSSL_*` injection it fails the same way a from-source build does (`cc` / `gcc`
errors while vendoring OpenSSL/secp256k1). `codegen.ps1` mirrors `build-win.ps1`'s
prerequisite checks and **process-scoped** environment, then runs the codegen
instead of `flutter build windows`:

- `RUSTFLAGS='--cfg zcash_unstable="nu7"'` — the NU7 flag (required by `CLAUDE.md`).
- Prepends the VS x64 tools to `PATH` so MSVC's `link.exe` wins.
- Sets `CARGO_BUILD_TARGET=x86_64-pc-windows-msvc` (the rustup host here is GNU,
  which is what fails the from-source OpenSSL build).
- Injects `OPENSSL_NO_VENDOR=1` + `OPENSSL_DIR`/`OPENSSL_STATIC`/`OPENSSL_LIBS`/… so
  `openssl-sys` links the **prebuilt vcpkg OpenSSL** (no Perl, no NASM).

**Prerequisites:** the same as the build — Rust, Visual Studio (C++), vcpkg, and
the one-time `vcpkg install openssl:x64-windows-static-md` (see §1). It also needs
the **`flutter_rust_bridge_codegen` binary**, whose version **must match** the
`flutter_rust_bridge` dependency pinned in `pubspec.yaml` / `rust/Cargo.toml`
(currently **`2.12.0`**) — a mismatch produces bindings that break at load time.

You don't install that binary yourself: `codegen.ps1` installs it **locally** into
**`build-win\frb`** (via `cargo install --root`) on first run and prepends its `bin\`
to `PATH` for the script's process only. **Nothing is installed globally** into
`~/.cargo/bin`, and `build-win\` is gitignored — the codegen toolchain stays
self-contained next to the cloned Flutter SDK.

Run it from the project root (where `pubspec.yaml` is):

```powershell
# First run auto-installs the codegen into build-win\frb (local), then regenerates:
.\codegen.ps1

# Force a fresh local (re)install of the pinned 2.12.0, then regenerate:
.\codegen.ps1 -InstallCodegen
```

If PowerShell blocks the script, run it for the current process only:

```powershell
powershell -ExecutionPolicy Bypass -File .\codegen.ps1
```

After it finishes, review the diff under `lib/src/rust/` and
`rust/src/frb_generated.rs`, then build as usual with `build-win.ps1`.

**Options:**

```powershell
# vcpkg installed somewhere other than C:\vcpkg:
.\codegen.ps1 -VcpkgRoot D:\vcpkg

# Pin a different FRB codegen version (must match pubspec.yaml / rust/Cargo.toml):
.\codegen.ps1 -FrbVersion 2.12.0 -InstallCodegen

# Install/use the codegen binary somewhere other than build-win\frb:
.\codegen.ps1 -CodegenRoot D:\tools\frb
```

> Delete `build-win\frb` to force a clean reinstall of the codegen binary (or just
> pass `-InstallCodegen`). It's gitignored along with the rest of `build-win\`.

> Like `build-win.ps1`, this script changes **nothing globally** — the `PATH` tweak
> and `OPENSSL_*` / `RUSTFLAGS` / `CARGO_BUILD_TARGET` variables apply to its **own
> process only**.

The codegen-specific troubleshooting items are noted in §5 below (they share the
OpenSSL / `link.exe` / `RUSTFLAGS` causes with the build).

---

## 5. Troubleshooting

- **`flutter` / `cargo` / `vcpkg.exe` not found** — the tool isn't on PATH (or
  `-VcpkgRoot` is wrong). Re-open PowerShell after editing PATH; pass `-VcpkgRoot`
  if vcpkg isn't at `C:\vcpkg`.
- **`git` not found / Flutter clone fails** — the script clones Flutter into
  `build-win\flutter` and needs `git` on PATH (https://git-scm.com). To skip the
  clone, pass `-FlutterDir <existing-sdk>` or `-FlutterVersion ""` to use a
  Flutter already on PATH. Delete `build-win\flutter` to force a clean re-clone.
- **`LNK2038: mismatch detected for 'RuntimeLibrary' ('MT_StaticRelease' vs
  'MD_DynamicRelease')`** — wrong OpenSSL triplet. Rust-msvc uses the **dynamic**
  CRT (`/MD`), so OpenSSL must be `x64-windows-static-md`, **not**
  `x64-windows-static` (which is `/MT`). Re-run with the default triplet.
- **`unresolved external symbol` from OpenSSL during the Rust link** — ensure
  `OPENSSL_STATIC=1` and `OPENSSL_LIBS=libssl:libcrypto` (colon-separated). The
  script sets these; if building manually, export them and confirm
  `libssl.lib` / `libcrypto.lib` exist in
  `<vcpkg>\installed\x64-windows-static-md\lib`.
- **`openssl-sys` tries to build from source / asks for Perl** — `OPENSSL_NO_VENDOR`
  wasn't seen by cargo. The script sets it; if you build manually, export
  `OPENSSL_NO_VENDOR=1` and `OPENSSL_DIR=<vcpkg>/installed/x64-windows-static-md`.
  Do **not** install Strawberry Perl to "fix" this — it's incompatible with the
  current openssl wrapper.
- **`link.exe` errors / wrong `link.exe` (e.g. `___chkstk_ms`, MSYS2 link)** — a
  non-MSVC `link.exe` won on `PATH`. The script prepends the VS x64 tools
  (`VC\Tools\MSVC\<ver>\bin\Hostx64\x64`) to `PATH` so MSVC's `link.exe` wins;
  confirm with `(Get-Command link.exe).Source` — it should point into the VS
  install, not `C:\msys64`. cargo finds the MSVC `INCLUDE`/`LIB` via the registry
  on its own, so a full `vcvars64.bat` import isn't needed.
- **`MSB8020: The build tools for ClangCL ... cannot be found`** — the
  `rive_common` plugin requires the MSBuild ClangCL toolset. Install the VS
  components *"C++ Clang Compiler for Windows"* and *"MSBuild support for LLVM
  (clang-cl) toolset"*, e.g.:
  ```powershell
  & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\setup.exe" modify `
      --installPath "C:\Program Files\Microsoft Visual Studio\2022\Community" `
      --add Microsoft.VisualStudio.Component.VC.Llvm.Clang `
      --add Microsoft.VisualStudio.Component.VC.Llvm.ClangToolset `
      --quiet --norestart
  ```
  The script pre-checks for this and fails early with the same hint.
- **Script stops with "OpenSSL ... was not found"** — the prerequisite isn't
  installed. Run the one-time `C:\vcpkg\vcpkg.exe install openssl:x64-windows-static-md`
  (see §1), then re-run the build.
- **`vcpkg install openssl` is very slow** — expected one-time cost; it only runs
  once. After it completes, every build reuses the installed copy.
- **Build finishes but `rlz.dll` is missing from the Release folder** — cargokit's
  Rust build failed (scroll up in the log for the cargo error). The script checks
  for `rlz.dll` and stops if it's absent. With static OpenSSL there's no other
  native dependency beyond the MSVC runtime that Flutter already bundles.
- **Rust build fails on a zcash crate** — make sure `RUSTFLAGS` is set. The script
  sets `--cfg zcash_unstable="nu7"`; if building manually you must export it.
- **Wrong feature compiled / `ledger` not enabled** — check `rust\cargokit.yaml`.
  The script writes it each run from `-Feature`; cargokit reads
  `cargo.release.extra_flags` from it (same file CI generates).
- **Clean rebuild** — delete `build\` and `target\x86_64-pc-windows-msvc\`, then rerun.

### `flutter.ps1` (Dart pre-flight)

- **`flutter analyze` reports a missing generated symbol** (e.g. an undefined
  `*Provider` or `_$...` mixin) — a `@freezed` / `@riverpod` declaration changed but
  its generated file wasn't refreshed. Re-run `flutter.ps1 -BuildRunner` to
  regenerate `*.g.dart` / `*.freezed.dart`.
- **`flutter analyze` reports an undefined FRB symbol** (e.g. a Rust function or a
  changed signature like `openDatabase(... coin:)`) — the flutter_rust_bridge
  bindings are stale. Regenerate them with `codegen.ps1` (§4b), not `-BuildRunner`.
- **`git` not found / Flutter clone fails** — same as the build: pass
  `-FlutterDir <existing-sdk>` or `-FlutterVersion ""` to avoid cloning.

### `codegen.ps1` (FRB binding regeneration)

- **`flutter_rust_bridge_codegen` not found / local install failed** — `codegen.ps1`
  installs it **locally** into `build-win\frb` (via `cargo install --root`) and needs
  `cargo` on PATH to do so. If the install failed, scroll up for the cargo error;
  re-run with `-InstallCodegen` to force a fresh local install. The version must
  match the `flutter_rust_bridge` dependency in `pubspec.yaml` / `rust/Cargo.toml`.
- **"Using a flutter_rust_bridge_codegen that is NOT the local build-win\frb copy"** —
  a global codegen on PATH was used instead of the local one. Re-run with
  `-InstallCodegen` to install the pinned version into `build-win\frb`; the script
  prepends that bin\ to PATH so the local copy wins.
- **`Error: cargo expand returned empty output`** — the crate failed to compile during
  codegen, almost always for the **same reasons as a failed build**: missing
  `RUSTFLAGS`, a non-MSVC `link.exe` winning on `PATH`, or `openssl-sys` trying to
  vendor OpenSSL from source (`cc` / `gcc` errors). `codegen.ps1` sets
  `RUSTFLAGS`, prepends the MSVC tools, and injects `OPENSSL_NO_VENDOR=1` + the
  `OPENSSL_*` vars to fix all three — see the OpenSSL / `link.exe` / `RUSTFLAGS`
  items above.
- **Bindings regenerate but the app crashes on launch / FRB hash mismatch** — the
  codegen binary version didn't match the pinned `flutter_rust_bridge` dependency.
  Re-run `codegen.ps1 -InstallCodegen` to install `2.12.0`, regenerate, and rebuild.
