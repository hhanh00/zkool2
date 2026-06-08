# Building the zkool Android APK on Windows (self-contained SDK under `build-win\`)

This guide explains how to compile the zkool **Android** app (APK / AAB) on your own
Windows machine with `android-win.ps1`, and — importantly — how to **run it in an
emulator on the PC** so you never have to install anything on a physical phone or
enable USB debugging.

Everything the script needs that isn't already a system-wide developer tool
(the **Android SDK**, **command-line tools**, **platform-tools**, **build-tools**,
**NDK**, a **system image**, and a pinned **Flutter SDK**) is downloaded into
**`build-win\`** — exactly like the existing `build-win.ps1` clones Flutter into
`build-win\flutter`. Nothing is installed globally and your machine's existing
`ANDROID_HOME` / Android Studio (if any) is left untouched.

> **Builds by default.** Running `.\android-win.ps1` provisions the toolchain
> (downloads the JDK/SDK/NDK, accepts licenses, optionally creates an emulator) **and
> then builds the split APKs** with the same commands CI uses. Pass **`-NoBuild`** to
> stop after provisioning if you only want the environment set up.

---

## How the Android build works (same as CI)

CI builds the Android app in `.github/actions/android/action.yml`:

```bash
flutter build apk --split-per-abi --target-platform android-arm64,android-x64
flutter build aab --target-platform android-arm64,android-x64
```

with one Rust environment variable:

```
CARGO_ENCODED_RUSTFLAGS = "--cfgzcash_unstable=\"nu7\""
```

(the ``-separated form of `RUSTFLAGS='--cfg zcash_unstable="nu7"'`, required
for **NU7** consensus support — see `CLAUDE.md`).

The native `rust/` crate (`rlz`, a `cdylib`) is **not** built with MSVC here. On
Android, **cargokit** (`rust_builder/cargokit/gradle/plugin.gradle`) compiles `rlz`
for each Android ABI using the **NDK's clang** during the Gradle build that
`flutter build apk` drives. The desktop build's **vcpkg** OpenSSL isn't used here, but
OpenSSL still matters: the script **prebuilds it per ABI** and points `openssl-sys` at
it (see "OpenSSL: prebuilt per-ABI" below). So the two Rust-side requirements are the
`nu7` flag **and** that prebuilt OpenSSL — not a from-source vendored build.

> **Cross-compiling from Windows needs no MSYS2 / system libraries.** The C side of
> the build uses the **NDK's** bundled clang + sysroot (cargokit sets `CC`/`AR`/etc.
> to the NDK toolchain — see `rust_builder/cargokit/.../android_environment.dart`).
> The only Rust prerequisite is the prebuilt **`rust-std`** for each Android target,
> installed via `rustup target add`. The catch: cargokit invokes
> `rustup run stable cargo …`, so those targets must live in the **`stable`**
> toolchain specifically — which on many Windows boxes is **not** the rustup default
> (e.g. default `stable-x86_64-pc-windows-gnu` vs. `stable` → `…-msvc`). Installing
> the targets into the wrong toolchain is what produces
> `error[E0463]: can't find crate for core`. The script installs them into the
> correct toolchain with `rustup target add --toolchain stable …`.

### OpenSSL: prebuilt per-ABI (downloaded, not built)

The Rust crate forces `libsqlite3-sys = { features =
["bundled-sqlcipher-vendored-openssl"] }`. The `vendored-openssl` half would compile
OpenSSL **from C source** via `openssl-src`, whose build runs through
`/bin/sh`+`perl`+`make`. On Windows that pipeline is handed cargokit's NDK compiler
path **with backslashes** (`CC=…\clang.exe`); the MSYS `/bin/sh` treats `\` as an
escape and eats them, so the build dies with:

```
/bin/sh: line 1: C:...clang.exe: command not found
make: *** [build_libs] Error 2
cargo:warning=openssl-src: failed to build OpenSSL from source
```

The fix mirrors `build-win.ps1`'s desktop approach (prebuilt OpenSSL +
`OPENSSL_NO_VENDOR=1`). On Android we can't use vcpkg, and building OpenSSL from source
for Android requires an MSYS2 shell with perl + GNU make (OpenSSL's Android target uses
the Unix Makefile — MSVC can't target Android). So instead the script **downloads
prebuilt static OpenSSL** (`libssl.a` / `libcrypto.a`, OpenSSL 3.x) per ABI from
[KDAB/android_openssl](https://github.com/KDAB/android_openssl), normalizes it into
`build-win\openssl-android\<rust-target>\{lib,include}`, and points `openssl-sys` at it:

| Env var (set by the script during the build) | Value |
|------------------------------------------------|-------|
| `OPENSSL_NO_VENDOR` | `1` (don't vendor-build; use the prebuilt copy) |
| `AARCH64_LINUX_ANDROID_OPENSSL_DIR` | `build-win/openssl-android/aarch64-linux-android` |
| `AARCH64_LINUX_ANDROID_OPENSSL_STATIC` | `1` |
| `X86_64_LINUX_ANDROID_OPENSSL_DIR` | `build-win/openssl-android/x86_64-linux-android` |
| `X86_64_LINUX_ANDROID_OPENSSL_STATIC` | `1` |

Because OpenSSL is **prebuilt**, the backslash-mangling never happens — there's no
shell/make pipeline at all, just a download + copy in PowerShell. SQLCipher (the
`bundled-sqlcipher` half) still compiles from source via cc-rs/NDK — that part has no
shell pipeline — and links against this prebuilt OpenSSL.

> **No perl/make/MSYS needed.** The KDAB repo is cloned once into
> `build-win\openssl-android\kdab-cache` (~200 MB) and the per-ABI `.a` + headers are
> copied into `build-win\openssl-android\<abi>`, then reused. Pass `-SkipOpenssl` if
> you've wired OpenSSL up yourself. (Trade-off: you trust a third-party prebuilt
> binary rather than compiling from source.)

What cargokit needs from the Gradle project (already configured in this repo):

| Setting | Where | Value |
|---------|-------|-------|
| `ndkVersion` | `android/app/build.gradle` | `28.2.13676358` |
| `rive.ndk.version` | `android/gradle.properties` | `28.2.13676358` |
| `compileSdk` / `targetSdk` | `android/app/build.gradle` | `36` |
| ABIs built | CI command | `android-arm64`, `android-x64` |
| Gradle | `android/gradle/wrapper/...` | `8.14.3` (wrapper auto-downloads) |

So the SDK we provision below must include: **platform `android-36`**,
**build-tools** matching the Gradle Android plugin, **platform-tools** (`adb`),
**NDK `28.2.13676358`**, and (for emulation) the **emulator** + a **system image**.

---

## 1. Prerequisites

Only these must already be on your machine / `PATH` — **everything else is
auto-detected and downloaded** by the script if it isn't found:

| Tool | Why | How to get it |
|------|-----|---------------|
| **Rust (rustup)** | cargokit compiles the `rlz` crate for each Android ABI via the NDK. The Android Rust **targets** (`aarch64-linux-android`, `x86_64-linux-android`) are added by the script. | https://rustup.rs — ensure `cargo` is on `PATH`. |
| **git** | Used to clone the pinned Flutter SDK into `build-win\flutter` and the prebuilt-OpenSSL repo into `build-win\openssl-android\kdab-cache`. | https://git-scm.com — add to `PATH`. |
| **PowerShell 5.1+** | Runs the script (`.NET` `Expand-Archive` / `Invoke-WebRequest` / `Invoke-RestMethod`). | Ships with Windows 10/11. |
| **~14 GB free disk + internet** | The JDK + SDK + NDK + system image + Flutter + prebuilt OpenSSL total roughly 10–14 GB under `build-win\`. | — |

> **No MSYS2 / perl / make needed.** OpenSSL is **downloaded prebuilt** (static `.a`
> per ABI), not compiled — so the script runs in plain PowerShell. (Compiling OpenSSL
> for Android *would* require an MSYS2 shell with perl + GNU make, since OpenSSL's
> Android target uses the Unix Makefile and MSVC can't target Android; we sidestep all
> of that by using prebuilt binaries.)

Everything below is **detected first** (an existing usable copy is reused) and only
**downloaded into `build-win\` if missing**:

| Component | Detected from / Downloaded to | Notes |
|-----------|-------------------------------|-------|
| **Java JDK 17+** | Detected via `JAVA_HOME` / `java` on `PATH`; else **Temurin 17** downloaded to `build-win\jdk` | A JDK with major **≥ 17** is reused as-is. If none qualifies, the script fetches the current Temurin 17 `.zip` via the Adoptium API and sets `JAVA_HOME` for its process only. |
| **Flutter SDK** (pinned `3.41.7`, matching CI) | `build-win\flutter` | Reused if already present; pass `-FlutterDir` to point at an existing SDK, or `-FlutterVersion ""` to use whatever `flutter` is on `PATH`. |
| **Android command-line tools** | `build-win\android-sdk\cmdline-tools\latest` | Bootstraps `sdkmanager`, which installs everything below. |
| **platform-tools** (`adb`) | `build-win\android-sdk\platform-tools` | |
| **platforms;android-36** | `build-win\android-sdk\platforms` | Matches `compileSdk`/`targetSdk = 36`. |
| **build-tools;36.0.0** | `build-win\android-sdk\build-tools` | |
| **NDK `28.2.13676358`** | `build-win\android-sdk\ndk\28.2.13676358` | Must match `android/app/build.gradle` exactly. |
| **emulator** + **system-images;android-36;google_apis;x86_64** | `build-win\android-sdk\emulator`, `...\system-images` | Only when emulation is requested (default on). |

> **JDK detection rule:** the script checks `JAVA_HOME` first, then `java` on `PATH`.
> The first one whose **major version is ≥ 17** is used unchanged. Only when neither
> qualifies does it download Temurin 17 into `build-win\jdk` (Temurin is GPLv2+CE and
> freely redistributable). Either way it sets `JAVA_HOME` + `PATH` **for its own
> process only** — your machine-wide Java is never changed.

> **SDK detection rule:** the script runs `sdkmanager --list_installed` and installs
> **only the pinned packages that are missing**, so re-runs are fast and don't
> re-download what's already there.

---

## 2. SDK layout produced under `build-win\`

```
build-win\
├─ jdk\                             # ONLY if no system JDK 17+ was found
│  └─ jdk-17.x.x+x\bin\java.exe     #   (Temurin 17; JAVA_HOME for the build)
├─ flutter\                         # pinned Flutter SDK (git clone, depth 1)
├─ openssl-android\                 # prebuilt OpenSSL per ABI (for openssl-sys)
│  ├─ kdab-cache\                   #   downloaded KDAB/android_openssl checkout
│  ├─ aarch64-linux-android\        #   lib\libssl.a, libcrypto.a, include\
│  └─ x86_64-linux-android\         #   lib\libssl.a, libcrypto.a, include\
└─ android-sdk\                     # ANDROID_HOME / ANDROID_SDK_ROOT for the build
   ├─ cmdline-tools\latest\bin\     # sdkmanager.bat, avdmanager.bat
   ├─ platform-tools\               # adb.exe
   ├─ platforms\android-36\
   ├─ build-tools\36.0.0\
   ├─ ndk\28.2.13676358\
   ├─ emulator\                     # emulator.exe (when -NoEmulator is NOT passed)
   ├─ system-images\android-36\google_apis\x86_64\
   └─ avd is created under: %USERPROFILE%\.android\avd\zkool_emulator.avd
```

The script writes `android/local.properties` so Gradle finds both SDKs:

```
flutter.sdk=...\build-win\flutter
sdk.dir=...\build-win\android-sdk
```

and exports `ANDROID_HOME` / `ANDROID_SDK_ROOT` **for its own process only** —
nothing global is modified.

> **Note on the AVD location:** the emulator's virtual device (AVD) lives under
> `%USERPROFILE%\.android\avd` (the SDK doesn't support relocating it cleanly).
> Everything *downloaded* still lands under `build-win\android-sdk`. Pass
> `-NoEmulator` if you don't want any AVD created at all.

---

## 3. Quick start

```powershell
# From the repo root (where pubspec.yaml is):

# A) Provision the JDK/SDK/NDK/Flutter + create an emulator AND build the APKs.
.\android-win.ps1

# B) Only provision (no APK build); set up the toolchain + emulator and stop.
.\android-win.ps1 -NoBuild

# C) Headless: skip the emulator, still build the APKs (CI-like):
.\android-win.ps1 -NoEmulator

# D) Provision + build, then launch the emulator (handy with -Run):
.\android-win.ps1 -LaunchEmulator

# E) Build, install onto a running emulator, and launch the app in one go:
.\android-win.ps1 -LaunchEmulator -Run

.\android-win.ps1 *>&1 | Tee-Object -FilePath android.log
```

After provisioning, the script prints the exact commands to build and to launch
the emulator yourself, so you can drive the rest manually with the SDK it set up.

---

## 4. Signing — `key.properties` is required

`android/app/build.gradle` loads `key.properties` **unconditionally** (even debug
Gradle evaluation reads it), so a keystore is mandatory before any `flutter build apk`
will succeed:

```groovy
signingConfigs {
    release {
        def keystoreProperties = new Properties()
        keystoreProperties.load(new FileInputStream(rootProject.file("key.properties")))
        storeFile file(keystoreProperties['storeFile'])
        ...
    }
}
```

The real release keystore (`android/app/zkool-keystore.jks.enc`) is encrypted and
only decryptable in CI with the `JKS_PASSWORD` secret. For **local** APKs you
generate your own throwaway keystore. `android-win.ps1` does this automatically the
first time (unless `key.properties` already exists):

1. Generates `android\app\zkool-local.jks` via the JDK's `keytool` (a self-signed
   debug-grade key — fine for local installs and the emulator, **not** for Play).
2. Writes `android\key.properties`:
   ```
   storePassword=android
   keyPassword=android
   keyAlias=zkool
   storeFile=zkool-local.jks
   ```

> `key.properties`, `*.jks`, and `*.keystore` are already in `android/.gitignore`,
> so the local keystore is never committed. To use your **own** keystore, create
> `android\key.properties` yourself before running and the script leaves it alone.

> An APK signed with this local key **cannot** upgrade a Play-Store install of zkool
> (different signature). Uninstall any store build first, or use a fresh emulator.

---

## 5. Emulating on the PC (no phone, no USB debugging)

This is the whole point of running on a desktop: the **Android Emulator** runs a
full virtual device on your PC, so you never touch a physical phone.

### 5.1 What the script sets up

With emulation enabled (the default), the script installs:

- `emulator` — the Android Emulator engine.
- `system-images;android-36;google_apis;x86_64` — an **x86_64** Android 36 image.
  x86_64 runs at near-native speed on an Intel/AMD PC (an arm64 image would be
  *emulated* instruction-by-instruction and is painfully slow on x86 desktops).
- An AVD named **`zkool_emulator`** created from that image.

### 5.2 Hardware acceleration (important)

The x86_64 emulator needs a hypervisor or it falls back to glacial software
rendering:

- **Intel CPU:** install **Intel HAXM** *or* enable **Windows Hypervisor Platform**
  (WHPX). On Windows 11, WHPX is easiest: enable **"Windows Hypervisor Platform"**
  and **"Virtual Machine Platform"** in *Turn Windows features on or off*, reboot.
- **AMD CPU:** use **WHPX** (HAXM is Intel-only).
- Either way, **virtualization (VT-x / AMD-V) must be enabled in your BIOS/UEFI.**

The script does **not** install a hypervisor (it needs admin + reboot). It checks
for one with `emulator -accel-check` and warns if acceleration is unavailable, but
still lets you continue.

### 5.3 Launch the emulator

```powershell
# The script can launch it for you:
.\android-win.ps1 -LaunchEmulator

# …or do it yourself with the provisioned SDK:
$env:ANDROID_HOME = "$PWD\build-win\android-sdk"
& "$env:ANDROID_HOME\emulator\emulator.exe" -avd zkool_emulator
```

Wait until the home screen appears, then confirm the device is visible:

```powershell
& "$env:ANDROID_HOME\platform-tools\adb.exe" devices
# emulator-5554   device
flutter devices
# sees "Android SDK built for x86_64 (emulator-5554)"
```

### 5.4 Run / install zkool on the emulator

```powershell
# Hot-run from source onto the running emulator:
flutter run -d emulator-5554

# …or install a built APK (produced by the default build, see §6):
& "$env:ANDROID_HOME\platform-tools\adb.exe" install -r `
    build\app\outputs\flutter-apk\app-x86_64-release.apk
```

> Install the **x86_64** APK on the x86_64 emulator — an `arm64-v8a` APK will fail
> to install with `INSTALL_FAILED_NO_MATCHING_ABIS`. `flutter run`/`flutter install`
> pick the right ABI automatically; manual `adb install` does not.

---

## 6. Building the APK / AAB (default)

Building runs **by default** (pass `-NoBuild` to skip it). The script runs the same
commands CI uses, after exporting `CARGO_ENCODED_RUSTFLAGS` (the `nu7` flag) for its
process:

```powershell
flutter build apk --split-per-abi --target-platform android-arm64,android-x64
# and, with -Aab:
flutter build aab --target-platform android-arm64,android-x64
```

Outputs:

| Artifact | Path |
|----------|------|
| Split APKs | `build\app\outputs\flutter-apk\app-arm64-v8a-release.apk`, `app-x86_64-release.apk` |
| App bundle (`-Aab`) | `build\app\outputs\bundle\release\app-release.aab` |

`--split-per-abi` produces one APK per ABI (smaller installs). The `arm64-v8a` APK
is for real phones; the `x86_64` APK is the one to install on the desktop emulator.

> **First build is slow.** cargokit cross-compiles the `rlz` Rust crate (incl.
> vendored OpenSSL) for **both** ABIs with the NDK, and the Gradle wrapper downloads
> Gradle 8.14.3 on first run. Expect 15–30 min cold; subsequent builds are cached.

The matching environment variable, set by the script for the build process only:

```
CARGO_ENCODED_RUSTFLAGS = "--cfg zcash_unstable=\"nu7\""
```

This is the encoded equivalent of `RUSTFLAGS='--cfg zcash_unstable="nu7"'` from
`CLAUDE.md`; the encoded form avoids the space-splitting ambiguity that breaks the
plain `RUSTFLAGS` form when a value contains spaces.

---

## 7. Verifying the setup

```powershell
$env:ANDROID_HOME = "$PWD\build-win\android-sdk"
& "$env:ANDROID_HOME\cmdline-tools\latest\bin\sdkmanager.bat" --list_installed
& "$env:ANDROID_HOME\platform-tools\adb.exe" --version
& "$env:ANDROID_HOME\emulator\emulator.exe" -list-avds      # -> zkool_emulator
flutter doctor -v                                            # Android toolchain green
```

`flutter doctor` should show the **Android toolchain** pointing at
`build-win\android-sdk` with licenses accepted. If it complains about licenses, run:

```powershell
& "$env:ANDROID_HOME\cmdline-tools\latest\bin\sdkmanager.bat" --licenses
```

(The script accepts licenses non-interactively during provisioning.)

---

## 8. Parameters

| Parameter | Default | Meaning |
|-----------|---------|---------|
| `-NoBuild` | off | Stop after provisioning; do **not** run `flutter build apk`. (Building is the default.) |
| `-Aab` | off | Also build the `.aab` app bundle (ignored with `-NoBuild`). |
| `-Run` | off | Install + launch the app on the running emulator (ignored with `-NoBuild`). |
| `-LaunchEmulator` | off | Boot the `zkool_emulator` AVD after provisioning. |
| `-NoEmulator` | off | Do **not** install the emulator/system-image or create an AVD. |
| `-FlutterVersion` | `3.41.7` | Flutter branch/tag to clone into `build-win\flutter`. `""` = use `flutter` on `PATH`. |
| `-FlutterDir` | `""` | Reuse an existing Flutter SDK at this path instead of cloning. |
| `-AndroidSdkDir` | `build-win\android-sdk` | Where to place the Android SDK. |
| `-RustToolchain` | `stable` | rustup toolchain cargokit builds with; the Android `rust-std` targets are installed into **this** toolchain. Change only if cargokit is configured for a different one. |
| `-Feature` | `""` (none) | Cargo feature(s) written to `rust\cargokit.yaml`. Empty by default for Android (matches CI). **Do not pass `ledger`** — `hidapi` has no Android support and won't compile. |
| `-SkipOpenssl` | off | Skip the per-ABI OpenSSL prebuild + env wiring (use only if you've supplied OpenSSL yourself or de-vendored the crate). |
| `-Clean` | off | Remove `build\` and cargokit's `rust\target` before building. |
| `-SkipPubGet` | off | Skip `flutter pub get` (repeat builds). |

---

## 9. Troubleshooting

| Symptom | Fix |
|---------|-----|
| `FileNotFoundException: ...key.properties` during Gradle | You skipped signing setup. Let the script generate a local keystore, or create `android\key.properties` yourself (see §4). |
| `error[E0463]: can't find crate for core` / `target may not be installed` | The Android `rust-std` isn't in the toolchain **cargokit** uses. cargokit builds with `rustup run stable …`, so the targets must be in the `stable` toolchain — which may differ from your rustup *default*. Fix: `rustup target add --toolchain stable aarch64-linux-android x86_64-linux-android` (the script now does this; pass `-RustToolchain` if cargokit is configured for a different toolchain). **Not** an NDK/MSYS2 problem. |
| `clang.exe: command not found` / `openssl-src: failed to build OpenSSL from source` | The vendored OpenSSL-from-source build mangled the NDK `CC` path's backslashes under MSYS `/bin/sh`. The script now **downloads prebuilt OpenSSL per ABI** and sets `OPENSSL_NO_VENDOR=1` + `*_OPENSSL_DIR` to avoid vendoring (see "OpenSSL: prebuilt per-ABI"). If you still see it, the env vars didn't reach cargo — confirm you didn't pass `-SkipOpenssl`, and that `build-win\openssl-android\<abi>\lib\libssl.a` exists. |
| `Prebuilt static libs not found` / OpenSSL download fails | The KDAB ref or ABI dir changed, or the clone was interrupted. Delete `build-win\openssl-android\kdab-cache` and re-run so it re-clones. |
| `error[E0432]: unresolved import hidapi` / `cannot find module hidapi` | The `ledger` Cargo feature is enabled for Android, but `hidapi` (USB-HID) has no Android support. The script writes `rust\cargokit.yaml` with **no** features for Android; this error means a stale `--features=ledger` (left by `build-win.ps1`) leaked in. Re-run `android-win.ps1` (it overwrites `cargokit.yaml`), or set it manually to `extra_flags: []`. Do **not** pass `-Feature ledger`. |
| `NDK did not have a source.properties` / NDK mismatch | The installed NDK must be exactly `28.2.13676358`. Re-run provisioning; check `build-win\android-sdk\ndk\`. |
| `INSTALL_FAILED_NO_MATCHING_ABIS` | You `adb install`ed the `arm64-v8a` APK on the x86_64 emulator. Install `app-x86_64-release.apk`, or use `flutter install`. |
| Emulator boots to a black screen / extremely slow | Hardware acceleration is off. Enable WHPX/HAXM + BIOS virtualization (§5.2). Check with `emulator -accel-check`. |
| `Unsupported class file major version` / Gradle JDK error | A too-old Java is ahead on `PATH`. The script downloads Temurin 17 into `build-win\jdk` when it can't find a JDK ≥ 17 — delete a stale `JAVA_HOME` pointing at JDK 8/11, or remove `build-win\jdk` to force a fresh download. |
| `cmdline-tools component is missing` in `flutter doctor` | The cmdline-tools must live under `cmdline-tools\latest\` (the script lays it out correctly). |
| `adb` not found | Use the provisioned one: `build-win\android-sdk\platform-tools\adb.exe`. |
| License not accepted | `sdkmanager.bat --licenses` (see §7). |

---

## 10. Relationship to the other build scripts

| Script | Target | Toolchain |
|--------|--------|-----------|
| `flutter.ps1` | — (Dart pre-flight) | analyzer only, no native compile |
| `build-win.ps1` | Windows desktop `.zip` | MSVC + vcpkg OpenSSL |
| `build-msys2.ps1` | Windows desktop | MSYS2 / UCRT64 GNU |
| **`android-win.ps1`** | **Android APK / AAB** | **NDK (cargokit) + self-contained SDK** |

Run `flutter.ps1` first to catch Dart errors fast, then `android-win.ps1` for the
Android artifacts.
