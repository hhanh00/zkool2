# Building zkool for Windows locally

This guide explains how to compile zkool and produce **`zkool-%VERSION%.zip`**
on your own Windows machine using `build-msys2.ps1`.

## How this differs from CI

CI compiles the Rust crate *through* Flutter/cargokit with the **MSVC** toolchain.
That forces OpenSSL (pulled in by SQLCipher and arti/tor) to be built from C
source, which needs a native Windows Perl + NASM and is fragile locally.

`build-msys2.ps1` takes the approach `zwallet` uses instead: it **builds the
Rust library first**, by itself, using your **MSYS2 / UCRT64 GNU toolchain**
(gcc + the prebuilt UCRT64 OpenSSL), then **bypasses cargokit entirely** so
Flutter never compiles Rust (and never touches OpenSSL).

The key fact that makes this clean: `rlz` is a Flutter **FFI** plugin. The runner
`.exe` does **not** link against it — it `LoadLibrary`s `rlz.dll` at runtime. So
all Flutter needs is `rlz.dll` copied into the Release folder; no import lib, no
link step. The script swaps the `rlz` plugin's `windows/CMakeLists.txt` for a
one-liner that points Flutter at our prebuilt DLL.

In short:

1. `cargo build --target x86_64-pc-windows-gnu` → `rlz.dll` (uses UCRT64 OpenSSL; **no Perl, no NASM**).
2. Replace `rust_builder/windows/CMakeLists.txt` so it skips `apply_cargokit()`
   and bundles our prebuilt `rlz.dll` instead (no cargo, no MSVC, no OpenSSL).
3. `flutter build windows` (run inside the UCRT64 environment).
4. Copy `rlz.dll` + `liblzma-5.dll` next to `zkool.exe`.
5. Zip the Release folder as `out\zkool-%VERSION%.zip`.

> The script changes **nothing globally**. It only prepends the UCRT64 `bin\` to
> `PATH` for its own process, and the swap of the `rlz` `CMakeLists.txt` is
> reverted on exit (even if the build fails).

---

## 1. Prerequisites

You already have **Rust (rustup) with the `x86_64-pc-windows-gnu` GNU toolchain
default**, and **MSYS2** with the **UCRT64** environment. That's the hard part.
The full list:

| Tool | Why | How |
|------|-----|-----|
| **Rust (stable, gnu host)** | Compiles the native `rust/` crate for `x86_64-pc-windows-gnu`. | ✅ Already installed. Verify: `rustup show`. |
| **MSYS2 UCRT64 toolchain** | Provides `gcc`, the prebuilt **OpenSSL** (headers + libs), and `liblzma-5.dll`. | In an MSYS2 shell: `pacman -S mingw-w64-ucrt-x86_64-gcc mingw-w64-ucrt-x86_64-openssl`. Default location `C:\msys64`. |
| **Flutter SDK 3.44.1** | Builds the Windows app. | The script **clones Flutter 3.44.1 into `build-win\flutter`** automatically (like `zwallet`); you only need `git` on PATH. Pass `-FlutterDir` to reuse an existing SDK, or `-FlutterVersion ""` to use whatever `flutter` is on PATH. |
| **git** | Used to clone the Flutter SDK. | https://git-scm.com — add to PATH. |
| **Visual Studio 2022** with **"Desktop development with C++"** | Flutter's runner `.exe` is built with MSVC (Flutter needs `cl.exe`, the Windows SDK and CMake). | Install via the VS Installer. *Community* / *Build Tools* edition is fine. |
| **VS "C++ Clang tools for Windows" (ClangCL)** | The `rive_common` plugin forces the MSBuild **ClangCL** platform toolset. Without it the build fails with `MSB8020`. | VS Installer → Modify → **Individual components** → add *"C++ Clang Compiler for Windows"* **and** *"MSBuild support for LLVM (clang-cl) toolset"*. |

> **Why no Strawberry Perl / NASM / NuGet here?** Because we never run cargokit's
> MSVC OpenSSL-from-source build. OpenSSL comes prebuilt from UCRT64
> (`C:\msys64\ucrt64`), so none of those are required.

Verify your setup before building (from PowerShell):

```powershell
rustup show                         # host should be ...-pc-windows-gnu
flutter doctor                      # "Windows" + "Visual Studio" checks green
cargo --version
C:\msys64\ucrt64\bin\gcc.exe --version
Test-Path C:\msys64\ucrt64\include\openssl\opensslv.h   # must be True
```

---

## 2. Build

Open **PowerShell**, `cd` into the project root (where `pubspec.yaml` is), and run:

```powershell
.\build-msys2.ps1
```

If PowerShell blocks the script with an execution-policy error, run it for the
current process only:

```powershell
powershell -ExecutionPolicy Bypass -File .\build-msys2.ps1
```

The script will:

1. Clone Flutter (`3.44.1`) into `build-win\flutter` (first run only) and prepend
   its `bin\` to PATH for this process only. (Use `-FlutterDir` / `-FlutterVersion ""`
   to skip cloning — see §4.)
2. Read the version (`%VERSION%`) from `pubspec.yaml`.
3. Prepend `C:\msys64\ucrt64\bin` to PATH (this process only) and set
   `OPENSSL_DIR` + `RUSTFLAGS='--cfg zcash_unstable="nu7"'`.
4. `cargo build --release --target x86_64-pc-windows-gnu --features ledger`
   → `target\x86_64-pc-windows-gnu\release\rlz.dll`.
5. Swap the `rlz` plugin's `windows/CMakeLists.txt` for a version that bundles
   the prebuilt `rlz.dll` (skips cargokit), and clear the stale CMake cache.
6. `flutter build windows --release` (no cargo / OpenSSL compile happens).
7. Copy `rlz.dll` + `liblzma-5.dll` into the Release folder.
8. Zip it to **`out\zkool-%VERSION%.zip`** and restore the original `CMakeLists.txt`.

First build is slow — step 4 compiles the entire Rust/zcash dependency tree
(~12 minutes on a typical machine). Subsequent builds reuse the cargo cache.

---

## 3. Output

When it finishes you'll see:

```
out\zkool-%VERSION%.zip
```

That zip's root contains the runnable Windows build — `zkool.exe`, its DLLs
(including `rlz.dll` and `liblzma-5.dll`), and the `data\` folder. Unzip it
anywhere and run `zkool.exe`.

---

## 4. Options

```powershell
# Reuse an already-compiled rlz.dll and just re-run the Flutter build + zip:
.\build-msys2.ps1 -SkipRustBuild

# Skip `flutter pub get` on repeat builds:
.\build-msys2.ps1 -SkipPubGet

# MSYS2 installed somewhere other than C:\msys64:
.\build-msys2.ps1 -Msys2Root D:\msys64

# Build with a different cargo feature (default: ledger):
.\build-msys2.ps1 -Feature ""

# Use an already-installed Flutter SDK instead of cloning one:
.\build-msys2.ps1 -FlutterDir C:\flutter

# Clone a different Flutter version (branch/tag; default 3.44.1):
.\build-msys2.ps1 -FlutterVersion 3.27.0

# Use whatever 'flutter' is already on PATH (no clone):
.\build-msys2.ps1 -FlutterVersion ""
```

> The script clones Flutter into `build-win\flutter` on first run; delete that
> folder to force a fresh re-clone. Your globally-installed Flutter (if any) is
> left untouched.

---

## 5. Troubleshooting

- **`flutter` / `cargo` not found** — the tool isn't on PATH. Re-open PowerShell
  after editing PATH so the change takes effect.
- **`git` not found / Flutter clone fails** — the script clones Flutter into
  `build-win\flutter` and needs `git` on PATH (https://git-scm.com). To skip the
  clone, pass `-FlutterDir <existing-sdk>` or `-FlutterVersion ""` to use a
  Flutter already on PATH. Delete `build-win\flutter` to force a clean re-clone.
- **"Could not find UCRT64 gcc / OpenSSL headers"** — install the UCRT64
  packages in MSYS2 (`pacman -S mingw-w64-ucrt-x86_64-gcc
  mingw-w64-ucrt-x86_64-openssl`), or pass `-Msys2Root` if MSYS2 is not at
  `C:\msys64`.
- **`openssl-sys` tries to build from source / asks for Perl** — that means
  `OPENSSL_DIR` wasn't seen by cargo. The script sets it; if you build manually,
  export `OPENSSL_DIR=C:/msys64/ucrt64` and `OPENSSL_NO_VENDOR=1` first.
- **Build still tries to run cargo / hits `link.exe` errors like `___chkstk_ms`
  or `__isnan`** — that's cargokit compiling Rust with a mismatched host
  toolchain. It means the bypass `CMakeLists.txt` wasn't used, usually due to a
  stale CMake cache. The script deletes `build\windows` before building to
  prevent this; if you bypassed that, delete `build\windows` manually and rerun.
- **`MSB8020: The build tools for ClangCL ... cannot be found`** — the
  `rive_common` plugin requires the MSBuild ClangCL toolset. Install the VS
  components *"C++ Clang Compiler for Windows"* and *"MSBuild support for LLVM
  (clang-cl) toolset"*, e.g.:
  ```powershell
  & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\setup.exe" modify `
      --installPath "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools" `
      --add Microsoft.VisualStudio.Component.VC.Llvm.Clang `
      --add Microsoft.VisualStudio.Component.VC.Llvm.ClangToolset `
      --quiet --norestart
  ```
  The script pre-checks for this and fails early with the same hint.
- **App starts then exits / `rlz.dll` fails to load** — confirm both `rlz.dll`
  and `liblzma-5.dll` are next to `zkool.exe` in the Release folder / zip.
- **Rust build fails on a zcash crate** — make sure `RUSTFLAGS` is set. The
  script sets `--cfg zcash_unstable="nu7"`; if building manually you must export it.
- **Clean rebuild** — delete `build\` and `target\x86_64-pc-windows-gnu\`, then rerun.
