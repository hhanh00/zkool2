#Requires -Version 5.1
<#
.SYNOPSIS
    Provision a SELF-CONTAINED Android SDK + NDK + emulator under build-win\ and
    (optionally) build / run the zkool Android APK on Windows.

.DESCRIPTION
    This is the Android counterpart to build-win.ps1. Where build-win.ps1 builds the
    Windows desktop app with MSVC + vcpkg OpenSSL, this script targets ANDROID, where
    the native rust/ crate (rlz) is cross-compiled by cargokit using the Android NDK's
    clang during `flutter build apk`. Two Rust-side requirements: the NU7 flag
    (CARGO_ENCODED_RUSTFLAGS), and a PREBUILT OpenSSL per ABI - because the crate's
    `bundled-sqlcipher-vendored-openssl` would otherwise build OpenSSL from C source
    via /bin/sh+perl+make, which mangles the NDK clang path's backslashes on Windows
    and fails. This script DOWNLOADS prebuilt static OpenSSL (.a) per ABI into
    build-win\openssl-android (from KDAB/android_openssl) and points openssl-sys at it
    (OPENSSL_NO_VENDOR=1 + <TARGET>_OPENSSL_DIR), mirroring how build-win.ps1 uses a
    prebuilt vcpkg OpenSSL on desktop. No perl / make / MSYS2 required.

    Like build-win.ps1 clones Flutter into build-win\flutter, this script downloads
    EVERYTHING it needs that isn't a system developer tool into build-win\:
      * Android command-line tools  -> build-win\android-sdk\cmdline-tools\latest
      * platform-tools (adb)        -> build-win\android-sdk\platform-tools
      * platforms;android-36        -> build-win\android-sdk\platforms
      * build-tools;36.0.0          -> build-win\android-sdk\build-tools
      * NDK 28.2.13676358           -> build-win\android-sdk\ndk\28.2.13676358
      * emulator + x86_64 system image (unless -NoEmulator)
      * Flutter SDK (pinned)        -> build-win\flutter
    Nothing global is modified: ANDROID_HOME / ANDROID_SDK_ROOT and PATH are set for
    THIS process only, and android\local.properties points Gradle at both SDKs.

    BY DEFAULT THIS SCRIPT PROVISIONS THE TOOLCHAIN AND THEN BUILDS the APK with the
    same commands CI uses. Pass -NoBuild to stop after provisioning (just download /
    verify the JDK, SDK, NDK, Flutter and optionally the emulator, without compiling).

    PREREQUISITES (must be on PATH; the JDK is auto-downloaded if missing):
      * Rust (rustup) with cargo on PATH             (cargokit builds rlz per-ABI)
      * git on PATH                                  (clones the Flutter SDK)
      * Java JDK 17+ is DETECTED, or downloaded into build-win\jdk if not found.

.PARAMETER NoBuild
    Stop after provisioning - do NOT run `flutter build apk`. (Building is the
    default.)

.PARAMETER Aab
    Additionally build the .aab app bundle (as CI does). Ignored with -NoBuild.

.PARAMETER Run
    Install the freshly built APK onto the running emulator and launch it. Ignored
    with -NoBuild.

.PARAMETER LaunchEmulator
    Boot the zkool_emulator AVD after provisioning.

.PARAMETER NoEmulator
    Do not install the emulator / system image and do not create an AVD.

.PARAMETER FlutterVersion
    Flutter git branch/tag to clone into build-win\flutter. Default 3.41.7 (CI's pin).
    Set to "" to use whatever `flutter` is already on PATH.

.PARAMETER FlutterDir
    Reuse an existing Flutter SDK at this path instead of cloning. Overrides
    -FlutterVersion.

.PARAMETER AndroidSdkDir
    Where to place the self-contained Android SDK. Default: build-win\android-sdk.

.PARAMETER RustToolchain
    The rustup toolchain cargokit builds with. cargokit uses the literal "stable"
    unless overridden, so the Android rust-std targets MUST be installed into THIS
    toolchain (not merely the rustup default). Default: stable. Change it only if you
    have configured cargokit (rust/cargokit.yaml or an override) to use a different
    toolchain.

.PARAMETER Feature
    Cargo feature(s) cargokit builds rlz with, written to rust/cargokit.yaml. DEFAULT
    IS EMPTY for Android - the `ledger` feature pulls in `hidapi`, which has no Android
    target_os support (it's desktop USB-HID only) and fails to compile for
    *-linux-android. CI's android action likewise builds with no extra features. Do NOT
    pass "ledger" here. (build-win.ps1 uses ledger because that's a desktop build.)

.PARAMETER SkipOpenssl
    Skip the per-ABI OpenSSL prebuild + env wiring. Use only if you have already built
    OpenSSL for Android into build-win\openssl-android and exported the *_OPENSSL_DIR
    variables yourself, or if you reverted the crate to a non-vendored OpenSSL.

.PARAMETER Clean
    Remove build\ and cargokit's rust\target before building.

.PARAMETER SkipPubGet
    Skip `flutter pub get`.

.EXAMPLE
    .\android-win.ps1
    Provision the JDK/SDK/NDK/Flutter, create an emulator, AND build the split APKs.

.EXAMPLE
    .\android-win.ps1 -NoBuild
    Only download + set up the toolchain and emulator; do not compile an APK.

.EXAMPLE
    .\android-win.ps1 -LaunchEmulator -Run
    Provision, boot the emulator, build the APK, install and launch it.

.EXAMPLE
    .\android-win.ps1 -NoEmulator -Aab
    Headless: provision (no emulator), build split APKs and the .aab.
#>
[CmdletBinding()]
param(
    [switch]$NoBuild,
    [switch]$Aab,
    [switch]$Run,
    [switch]$LaunchEmulator,
    [switch]$NoEmulator,
    [string]$FlutterVersion = "3.41.7",
    [string]$FlutterDir     = "",
    [string]$AndroidSdkDir  = "",
    [string]$RustToolchain  = "stable",
    [string]$Feature        = "",
    [switch]$SkipOpenssl,
    [switch]$Clean,
    [switch]$SkipPubGet
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

# Building is ON by default; -NoBuild stops after provisioning. $Build is the internal
# flag the rest of the script reads.
$Build = -not $NoBuild

# Ensure TLS 1.2 on Windows PowerShell 5.1 (its default is TLS 1.0, which breaks the
# HTTPS downloads). PowerShell 7+ (.NET 5+) negotiates TLS via the OS, so skip it
# there - and note the enum is [Net.SecurityProtocolType], not [Net.SecurityProtocol].
if ($PSVersionTable.PSVersion.Major -lt 6) {
    [Net.ServicePointManager]::SecurityProtocol =
        [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
}

# Always operate from the script's directory (the repo root).
$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $RepoRoot

# --- Versions pinned to match the repo (android/app/build.gradle, gradle.properties,
#     and .github/actions/android/action.yml). Keep these in sync with those files. ---
$NdkVersion      = "28.2.13676358"          # android/app/build.gradle ndkVersion
$PlatformApi     = "36"                      # compileSdk / targetSdk
$BuildToolsVer   = "36.0.0"
$SysImage        = "system-images;android-$PlatformApi;google_apis;x86_64"
$AvdName         = "zkool_emulator"
# Android command-line tools (Windows) - update the build number if it 404s.
$CmdlineToolsUrl = "https://dl.google.com/android/repository/commandlinetools-win-11076708_latest.zip"

# JDK: the minimum major version Gradle 8.14.3 accepts, and the Temurin feature
# release to download into build-win\jdk when no usable JDK is found. Adoptium's
# "latest GA" redirect API resolves the exact build, so we never hardcode a patch.
$JdkMinMajor     = 17
$JdkFeature      = "17"
$JdkApiUrl       = "https://api.adoptium.net/v3/assets/latest/$JdkFeature/hotspot?architecture=x64&image_type=jdk&os=windows&vendor=eclipse"

# OpenSSL for Android. We DOWNLOAD prebuilt static OpenSSL (.a) per ABI and point
# openssl-sys at it (OPENSSL_NO_VENDOR=1 + per-target *_OPENSSL_DIR), the same way
# build-win.ps1 points the desktop build at a prebuilt vcpkg OpenSSL. This sidesteps
# the crate's `bundled-sqlcipher-vendored-openssl`, whose vendored OpenSSL-from-source
# build runs through /bin/sh+perl+make and mangles the backslashes in the Windows NDK
# `CC=...\clang.exe` path (-> "clang.exe: command not found", build fails).
#
# We use prebuilt binaries (no perl/make/MSYS) from KDAB's android_openssl, which ships
# static libssl.a/libcrypto.a per ABI (ssl_3 = OpenSSL 3.x) plus shared headers. Map
# each Rust Android target to KDAB's Android ABI directory name.
$OpensslKdabRef  = "master"                  # KDAB/android_openssl branch (ssl_3 = OpenSSL 3.x)
$OpensslAbis     = @{
    "aarch64-linux-android" = "arm64-v8a"
    "x86_64-linux-android"  = "x86_64"
}

function Write-Step($msg) { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function Write-Info($msg) { Write-Host "    $msg" -ForegroundColor DarkGray }
function Write-Warn($msg) { Write-Host "    $msg" -ForegroundColor Yellow }

function Assert-Tool($name, $hint) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        throw "Required tool '$name' was not found on PATH. $hint"
    }
}

# Run an external command and throw on non-zero exit.
function Invoke-Checked($exe, [string[]]$cmdArgs) {
    & $exe @cmdArgs
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed ($LASTEXITCODE): $exe $($cmdArgs -join ' ')"
    }
}

# Return the major version of the JDK at $javaExe (e.g. 17), or 0 if it can't be read.
function Get-JavaMajor($javaExe) {
    try {
        $out = (& $javaExe -version 2>&1 | Out-String)
    } catch {
        return 0
    }
    if ($out -match 'version "(\d+)') { return [int]$Matches[1] }
    return 0
}

# Find a usable JDK (major >= $JdkMinMajor). Checks, in order: $env:JAVA_HOME, then
# `java` on PATH. Returns the JDK HOME dir (the folder whose bin\ holds java.exe), or
# $null if none qualifies. Does NOT download - the caller decides whether to.
function Find-UsableJdk {
    # 1. JAVA_HOME, if it points at a real JDK of the right version.
    if ($env:JAVA_HOME) {
        $jh = $env:JAVA_HOME
        $je = Join-Path $jh "bin\java.exe"
        if ((Test-Path $je) -and ((Get-JavaMajor $je) -ge $JdkMinMajor)) {
            return $jh
        }
    }
    # 2. `java` on PATH.
    $cmd = Get-Command "java" -ErrorAction SilentlyContinue
    if ($cmd) {
        $je = $cmd.Source
        if ((Get-JavaMajor $je) -ge $JdkMinMajor) {
            # JDK HOME is the parent of bin\.
            return (Split-Path -Parent (Split-Path -Parent $je))
        }
    }
    return $null
}

# Download + extract Temurin JDK into build-win\jdk and return its HOME dir. Reuses an
# existing extracted JDK if one of the right major version is already there.
function Install-LocalJdk($buildWin) {
    $jdkRoot = Join-Path $buildWin "jdk"
    New-Item -ItemType Directory -Force -Path $jdkRoot | Out-Null

    # Reuse an already-extracted JDK of the right version (folder name like jdk-17.x.x+x).
    $existing = Get-ChildItem $jdkRoot -Directory -ErrorAction SilentlyContinue |
        Where-Object { Test-Path (Join-Path $_.FullName "bin\java.exe") } |
        Where-Object { (Get-JavaMajor (Join-Path $_.FullName "bin\java.exe")) -ge $JdkMinMajor } |
        Select-Object -First 1
    if ($existing) {
        Write-Info "Reusing local JDK at $($existing.FullName)"
        return $existing.FullName
    }

    # Resolve the exact .zip via Adoptium's JSON API (more robust than following the
    # 'latest' 302 redirect under Windows PowerShell 5.1).
    Write-Step "Downloading Temurin JDK $JdkFeature into build-win\jdk"
    Write-Info "Querying $JdkApiUrl"
    $asset = Invoke-RestMethod -Uri $JdkApiUrl -UseBasicParsing
    $pkg = ($asset | Select-Object -First 1).binary.package
    if (-not $pkg -or -not $pkg.link) {
        throw "Could not resolve a Temurin JDK $JdkFeature download from the Adoptium API."
    }
    $zipUrl  = $pkg.link
    $relName = ($asset | Select-Object -First 1).release_name
    Write-Info "GET $zipUrl"

    $tmpZip = Join-Path $env:TEMP "temurin-jdk-$JdkFeature.zip"
    Invoke-WebRequest -Uri $zipUrl -OutFile $tmpZip -UseBasicParsing

    Write-Info "Extracting to $jdkRoot"
    Expand-Archive -Path $tmpZip -DestinationPath $jdkRoot -Force
    Remove-Item -Force $tmpZip -ErrorAction SilentlyContinue

    # The zip contains a single top-level jdk-<version>\ folder.
    $jdkHomeDir = Get-ChildItem $jdkRoot -Directory -ErrorAction SilentlyContinue |
        Where-Object { Test-Path (Join-Path $_.FullName "bin\java.exe") } |
        Select-Object -First 1
    if (-not $jdkHomeDir) {
        throw "Temurin JDK extraction did not produce a bin\java.exe under $jdkRoot."
    }
    Write-Info "Local JDK ready: $($jdkHomeDir.FullName) ($relName)"
    return $jdkHomeDir.FullName
}

# Fetch PREBUILT OpenSSL-for-Android (static libs) from the KDAB android_openssl repo
# and assemble a normalized per-ABI tree that openssl-sys understands. NO perl / make /
# MSYS required - we never compile OpenSSL; we just download the .a archives + headers.
#
# Source layout in KDAB/android_openssl (branch $OpensslKdabRef):
#   ssl_3/<android-abi>/libssl.a, libcrypto.a        (per-ABI static archives)
#   ssl_3/include/openssl/*.h                         (headers, shared across ABIs)
# openssl-sys wants <DIR>/lib/lib{ssl,crypto}.a and <DIR>/include/openssl/opensslv.h,
# so we copy the per-ABI .a into <installDir>\lib and the shared headers into
# <installDir>\include. The shared OpenSSL source/headers are downloaded once and reused.
function Get-OpenSslForAndroid {
    param(
        [string]$rustTarget,    # e.g. aarch64-linux-android
        [string]$androidAbi,    # e.g. arm64-v8a       (KDAB directory name)
        [string]$opensslRoot,   # build-win\openssl-android
        [string]$cacheDir       # where the raw KDAB checkout is cached
    )
    $installDir = Join-Path $opensslRoot $rustTarget
    $libDir     = Join-Path $installDir "lib"
    $incDir     = Join-Path $installDir "include"
    $libssl     = Join-Path $libDir "libssl.a"
    $libcrypto  = Join-Path $libDir "libcrypto.a"
    if ((Test-Path $libssl) -and (Test-Path $libcrypto) -and (Test-Path (Join-Path $incDir "openssl\opensslv.h"))) {
        Write-Info "OpenSSL for $rustTarget already present ($installDir)"
        return $installDir
    }

    # Cache the KDAB repo once (sparse/shallow clone keeps it small; full repo ~200MB).
    if (-not (Test-Path (Join-Path $cacheDir "ssl_3"))) {
        Write-Step "Downloading prebuilt OpenSSL (KDAB android_openssl @ $OpensslKdabRef)"
        if (Test-Path $cacheDir) { Remove-Item -Recurse -Force $cacheDir }
        Invoke-Checked "git" @(
            "clone", "--depth", "1", "-b", $OpensslKdabRef,
            "https://github.com/KDAB/android_openssl.git", $cacheDir
        )
    } else {
        Write-Info "Prebuilt OpenSSL cache present at $cacheDir"
    }

    $srcAbiDir = Join-Path $cacheDir "ssl_3\$androidAbi"
    $srcInc    = Join-Path $cacheDir "ssl_3\include"
    $srcSsl    = Join-Path $srcAbiDir "libssl.a"
    $srcCrypto = Join-Path $srcAbiDir "libcrypto.a"
    if (-not ((Test-Path $srcSsl) -and (Test-Path $srcCrypto))) {
        throw "Prebuilt static libs not found at $srcAbiDir (libssl.a/libcrypto.a). Check the KDAB ref / ABI name."
    }
    if (-not (Test-Path (Join-Path $srcInc "openssl\opensslv.h"))) {
        throw "Prebuilt OpenSSL headers not found at $srcInc\openssl. Check the KDAB layout."
    }

    Write-Step "Assembling OpenSSL for $rustTarget ($androidAbi) into build-win\openssl-android"
    New-Item -ItemType Directory -Force -Path $libDir | Out-Null
    if (Test-Path $incDir) { Remove-Item -Recurse -Force $incDir }
    New-Item -ItemType Directory -Force -Path $incDir | Out-Null

    Copy-Item -Force $srcSsl    $libssl
    Copy-Item -Force $srcCrypto $libcrypto
    # Headers are shared; copy the whole openssl\ (and crypto\ if present) header tree.
    Copy-Item -Recurse -Force (Join-Path $srcInc "openssl") (Join-Path $incDir "openssl")
    $srcCryptoInc = Join-Path $srcInc "crypto"
    if (Test-Path $srcCryptoInc) {
        Copy-Item -Recurse -Force $srcCryptoInc (Join-Path $incDir "crypto")
    }

    if (-not ((Test-Path $libssl) -and (Test-Path $libcrypto) -and (Test-Path (Join-Path $incDir "openssl\opensslv.h")))) {
        throw "Failed to assemble OpenSSL for $rustTarget under $installDir."
    }
    Write-Info "OpenSSL for $rustTarget ready: $installDir"
    return $installDir
}

# ----------------------------------------------------------------------------
# 0. Prerequisites (NOT downloaded - must already be installed)
# ----------------------------------------------------------------------------
Write-Step "Checking prerequisites"

if (-not (Test-Path (Join-Path $RepoRoot "pubspec.yaml"))) {
    throw "pubspec.yaml not found. Run this script from the zkool project root."
}

Assert-Tool "cargo" "Install Rust (rustup) so cargo is on PATH."
Assert-Tool "git"   "git is needed to clone the Flutter SDK and download tooling. Install from https://git-scm.com."

# Repo root in Windows-native form (Set-Location may report a PSDrive path).
$RepoRootFull = (Get-Item $RepoRoot).FullName
$BuildWin     = Join-Path $RepoRootFull "build-win"
New-Item -ItemType Directory -Force -Path $BuildWin | Out-Null

# Java JDK 17+ - Gradle 8.14.3 and the Android Gradle Plugin require it, and the
# sdkmanager/avdmanager/keytool scripts need a JRE. DETECT a usable JDK; if none is
# found, DOWNLOAD Temurin into build-win\jdk (self-contained, like the SDK/Flutter).
Write-Step "Resolving a JDK (>= $JdkMinMajor)"
$JdkHome = Find-UsableJdk
if ($JdkHome) {
    Write-Info "Using existing JDK at $JdkHome (major $(Get-JavaMajor (Join-Path $JdkHome 'bin\java.exe')))"
} else {
    Write-Warn "No usable JDK $JdkMinMajor+ found on JAVA_HOME or PATH - downloading one into build-win\jdk."
    $JdkHome = Install-LocalJdk $BuildWin
}
# Make this JDK authoritative for every child process (gradle, sdkmanager, keytool),
# for THIS process only.
$env:JAVA_HOME = $JdkHome
$env:PATH      = "$(Join-Path $JdkHome 'bin');$env:PATH"
Write-Info "JAVA_HOME=$env:JAVA_HOME"

# ----------------------------------------------------------------------------
# 1. Provide a Flutter SDK (clone pinned into build-win\flutter, like build-win.ps1)
# ----------------------------------------------------------------------------
if ($FlutterDir) {
    $PinnedFlutterBin = Join-Path $FlutterDir "bin"
    if (-not (Test-Path (Join-Path $PinnedFlutterBin "flutter.bat"))) {
        throw "No flutter.bat under '$PinnedFlutterBin'. Check -FlutterDir."
    }
    Write-Step "Using Flutter at $FlutterDir"
    $env:PATH = "$PinnedFlutterBin;$env:PATH"
} elseif ($FlutterVersion) {
    $FlutterDir = Join-Path $BuildWin "flutter"
    $PinnedFlutterBin = Join-Path $FlutterDir "bin"
    if (-not (Test-Path (Join-Path $PinnedFlutterBin "flutter.bat"))) {
        Write-Step "Cloning Flutter $FlutterVersion into build-win\flutter"
        Invoke-Checked "git" @(
            "clone", "-b", $FlutterVersion, "--depth", "1",
            "https://github.com/flutter/flutter.git", $FlutterDir
        )
    } else {
        Write-Info "Flutter already cloned at $FlutterDir (delete it to re-clone)."
    }
    $env:PATH = "$PinnedFlutterBin;$env:PATH"
} else {
    Write-Info "FlutterVersion empty - using whatever 'flutter' is on PATH."
}
Assert-Tool "flutter" "Install the Flutter SDK / clone via -FlutterVersion, or add its bin\ to PATH."
Assert-Tool "dart"    "dart ships with Flutter; ensure Flutter\bin is on PATH."

# ----------------------------------------------------------------------------
# 2. Android Rust targets (cargokit cross-compiles rlz for these ABIs)
# ----------------------------------------------------------------------------
# CRITICAL: cargokit builds with `rustup run <toolchain> cargo ...`, where the
# toolchain defaults to the literal "stable" (see cargokit build_tool builder.dart:
# `_toolchain => ... ?? 'stable'`). On a machine whose *default* toolchain differs
# from "stable" (e.g. default = stable-x86_64-pc-windows-gnu, but `stable` resolves
# to stable-x86_64-pc-windows-msvc), a plain `rustup target add` installs the Android
# std into the DEFAULT toolchain, NOT into the one cargokit uses. The build then dies
# with:  error[E0463]: can't find crate for `core` ... target may not be installed.
#
# So we install the Android targets into the EXACT toolchain cargokit invokes,
# selectable via -RustToolchain (default "stable"), using `rustup target add
# --toolchain <tc>`. This is also why no MSYS2 / system libraries are needed: Android
# cross-compilation uses the NDK's clang + sysroot for C, and Rust only needs the
# rust-std component for each Android target present in the right toolchain.
Write-Step "Ensuring Android Rust targets are installed for toolchain '$RustToolchain'"
$wantTargets = @("aarch64-linux-android", "x86_64-linux-android")
# `rustup +<tc> target list --installed` reports only that toolchain's targets.
$installedTargets = (& rustup "+$RustToolchain" target list --installed) 2>$null
foreach ($t in $wantTargets) {
    if ($installedTargets -notcontains $t) {
        Write-Info "Installing rust target $t into '$RustToolchain'"
        Invoke-Checked "rustup" @("target", "add", "--toolchain", $RustToolchain, $t)
    } else {
        Write-Info "Rust target present in '$RustToolchain': $t"
    }
}

# ----------------------------------------------------------------------------
# 3. Lay down the self-contained Android SDK under build-win\android-sdk
# ----------------------------------------------------------------------------
if (-not $AndroidSdkDir) { $AndroidSdkDir = Join-Path $BuildWin "android-sdk" }
New-Item -ItemType Directory -Force -Path $AndroidSdkDir | Out-Null
$AndroidSdkDir = (Get-Item $AndroidSdkDir).FullName
Write-Step "Provisioning Android SDK at $AndroidSdkDir"

# Make the SDK visible to every child process (sdkmanager, flutter, gradle) for THIS
# process only - nothing global is touched.
$env:ANDROID_HOME       = $AndroidSdkDir
$env:ANDROID_SDK_ROOT   = $AndroidSdkDir

$CmdlineRoot   = Join-Path $AndroidSdkDir "cmdline-tools"
$CmdlineLatest = Join-Path $CmdlineRoot "latest"
$SdkManager    = Join-Path $CmdlineLatest "bin\sdkmanager.bat"
$AvdManager    = Join-Path $CmdlineLatest "bin\avdmanager.bat"

# 3a. Bootstrap the command-line tools (which provide sdkmanager) if absent.
if (-not (Test-Path $SdkManager)) {
    Write-Step "Downloading Android command-line tools"
    $tmpZip = Join-Path $env:TEMP "cmdline-tools.zip"
    Write-Info "GET $CmdlineToolsUrl"
    Invoke-WebRequest -Uri $CmdlineToolsUrl -OutFile $tmpZip -UseBasicParsing

    # The zip extracts a top-level "cmdline-tools\" folder; the SDK requires it to
    # live at cmdline-tools\latest\, so extract to a temp dir then move into place.
    $tmpExtract = Join-Path $env:TEMP "cmdline-tools-extract"
    if (Test-Path $tmpExtract) { Remove-Item -Recurse -Force $tmpExtract }
    Expand-Archive -Path $tmpZip -DestinationPath $tmpExtract -Force

    New-Item -ItemType Directory -Force -Path $CmdlineRoot | Out-Null
    if (Test-Path $CmdlineLatest) { Remove-Item -Recurse -Force $CmdlineLatest }
    Move-Item -Path (Join-Path $tmpExtract "cmdline-tools") -Destination $CmdlineLatest

    Remove-Item -Force $tmpZip -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force $tmpExtract -ErrorAction SilentlyContinue
    Write-Info "cmdline-tools installed at $CmdlineLatest"
} else {
    Write-Info "cmdline-tools already present."
}
if (-not (Test-Path $SdkManager)) {
    throw "sdkmanager.bat not found at $SdkManager after install. Check the cmdline-tools URL."
}

# 3b. Accept licenses non-interactively (writes to <sdk>\licenses).
Write-Step "Accepting Android SDK licenses"
# `yes` isn't on Windows; feed a stream of "y" to sdkmanager --licenses via stdin.
$ys = ("y`n" * 50)
$ys | & $SdkManager --sdk_root="$AndroidSdkDir" --licenses | Out-Null

# 3c. Determine which of the pinned packages this repo needs are MISSING, and install
#     only those. sdkmanager is itself idempotent, but querying --list_installed lets
#     us skip the slow network round-trip entirely when everything is already present
#     and report exactly what (if anything) is being downloaded.
$packages = @(
    "platform-tools",
    "platforms;android-$PlatformApi",
    "build-tools;$BuildToolsVer",
    "ndk;$NdkVersion"
)
if (-not $NoEmulator) {
    $packages += "emulator"
    $packages += $SysImage
}

Write-Step "Checking installed SDK packages"
$installedList = (& $SdkManager --sdk_root="$AndroidSdkDir" --list_installed 2>$null | Out-String)
$missing = @()
foreach ($p in $packages) {
    # --list_installed prints the package path as the first column of each row.
    if ($installedList -match ("(?m)^\s*" + [regex]::Escape($p) + "\s")) {
        Write-Info "present : $p"
    } else {
        Write-Warn "missing : $p"
        $missing += $p
    }
}

if ($missing.Count -gt 0) {
    Write-Step "Installing missing SDK packages ($($missing.Count))"
    # Install in one invocation so shared deps resolve once.
    $ys | & $SdkManager --sdk_root="$AndroidSdkDir" @missing
    if ($LASTEXITCODE -ne 0) {
        throw "sdkmanager failed to install one or more packages: $($missing -join ', ')"
    }
} else {
    Write-Info "All required SDK packages already installed."
}

# Sanity-check the NDK landed at the exact version build.gradle pins.
$NdkPath = Join-Path $AndroidSdkDir "ndk\$NdkVersion"
if (-not (Test-Path (Join-Path $NdkPath "source.properties"))) {
    throw "NDK $NdkVersion was not installed at $NdkPath. cargokit requires this exact version (android/app/build.gradle)."
}
Write-Info "NDK ready: $NdkPath"

# ----------------------------------------------------------------------------
# 3d. Provide prebuilt OpenSSL per ABI (so openssl-sys does NOT vendor-build it)
# ----------------------------------------------------------------------------
# cargokit builds rlz with CC_<target>=...\clang.exe. The crate's
# `bundled-sqlcipher-vendored-openssl` would otherwise compile OpenSSL from C source
# through /bin/sh+perl+make, which strips the backslashes in that CC path and dies with
# "clang.exe: command not found". We avoid that entirely by DOWNLOADING prebuilt static
# OpenSSL (.a) per ABI and pointing openssl-sys at it via OPENSSL_NO_VENDOR=1 +
# per-target *_OPENSSL_DIR (set in the build step). No perl/make/MSYS needed.
# $OpensslInstalls maps rustTarget -> install dir for that wiring.
$OpensslRoot     = Join-Path $BuildWin "openssl-android"
$OpensslCache    = Join-Path $OpensslRoot "kdab-cache"
$OpensslInstalls = @{}
if ($Build -and -not $SkipOpenssl) {
    Write-Step "Providing prebuilt OpenSSL for Android (per ABI) into build-win\openssl-android"
    foreach ($rustTarget in $OpensslAbis.Keys) {
        $androidAbi = $OpensslAbis[$rustTarget]
        $OpensslInstalls[$rustTarget] = Get-OpenSslForAndroid `
            -rustTarget $rustTarget `
            -androidAbi $androidAbi `
            -opensslRoot $OpensslRoot `
            -cacheDir $OpensslCache
    }
} elseif ($SkipOpenssl) {
    Write-Warn "Skipping OpenSSL provisioning (-SkipOpenssl). openssl-sys must find OpenSSL some other way."
}

# ----------------------------------------------------------------------------
# 4. Point Gradle at both SDKs via android\local.properties
# ----------------------------------------------------------------------------
Write-Step "Writing android\local.properties"
# Gradle's java.util.Properties reads these; backslashes must be escaped (or use /).
$flutterForward = $FlutterDir.Replace('\', '/')
$sdkForward     = $AndroidSdkDir.Replace('\', '/')
$localProps = @"
flutter.sdk=$flutterForward
sdk.dir=$sdkForward
"@
Set-Content -Path (Join-Path $RepoRootFull "android\local.properties") -Value $localProps -Encoding ASCII
Write-Info "flutter.sdk=$flutterForward"
Write-Info "sdk.dir=$sdkForward"

# ----------------------------------------------------------------------------
# 5. Signing: ensure android\key.properties + a keystore exist
# ----------------------------------------------------------------------------
# android/app/build.gradle loads key.properties UNCONDITIONALLY, so a keystore is
# mandatory even to evaluate the project. The real release keystore is the encrypted
# CI one; for local builds we generate a throwaway debug-grade key (gitignored).
$KeyProps = Join-Path $RepoRootFull "android\key.properties"
if (Test-Path $KeyProps) {
    Write-Info "android\key.properties already exists - leaving it untouched."
} else {
    Write-Step "Generating a local signing keystore (android\app\zkool-local.jks)"
    $keytool = "keytool"
    if ($env:JAVA_HOME -and (Test-Path (Join-Path $env:JAVA_HOME "bin\keytool.exe"))) {
        $keytool = Join-Path $env:JAVA_HOME "bin\keytool.exe"
    }
    $jks = Join-Path $RepoRootFull "android\app\zkool-local.jks"
    if (-not (Test-Path $jks)) {
        Invoke-Checked $keytool @(
            "-genkeypair", "-v",
            "-keystore", $jks,
            "-storepass", "android", "-keypass", "android",
            "-alias", "zkool",
            "-keyalg", "RSA", "-keysize", "2048", "-validity", "10000",
            "-dname", "CN=zkool-local, OU=dev, O=zkool, L=, S=, C=US"
        )
    }
    $kp = @"
storePassword=android
keyPassword=android
keyAlias=zkool
storeFile=zkool-local.jks
"@
    Set-Content -Path $KeyProps -Value $kp -Encoding ASCII
    Write-Info "Wrote android\key.properties (local debug-grade key; gitignored)."
    Write-Warn "This key is for local installs/emulator ONLY - it cannot publish to Play."
}

# ----------------------------------------------------------------------------
# 6. Create the emulator AVD (unless -NoEmulator)
# ----------------------------------------------------------------------------
if (-not $NoEmulator) {
    Write-Step "Creating emulator AVD '$AvdName' (if missing)"
    $existingAvds = (& $AvdManager list avd 2>$null | Out-String)
    if ($existingAvds -match [regex]::Escape($AvdName)) {
        Write-Info "AVD '$AvdName' already exists."
    } else {
        # avdmanager prompts "Do you wish to create a custom hardware profile? [no]";
        # feed "no" on stdin.
        "no" | & $AvdManager create avd `
            --name $AvdName `
            --package $SysImage `
            --device "pixel_6" `
            --force
        if ($LASTEXITCODE -ne 0) {
            throw "avdmanager failed to create AVD '$AvdName'."
        }
        Write-Info "AVD '$AvdName' created from $SysImage."
    }

    # Warn if hardware acceleration isn't available - the x86_64 emulator is unusable
    # without WHPX/HAXM + BIOS virtualization.
    $EmulatorExe = Join-Path $AndroidSdkDir "emulator\emulator.exe"
    if (Test-Path $EmulatorExe) {
        $accel = (& $EmulatorExe -accel-check 2>&1 | Out-String)
        if ($accel -match "is installed and usable|accel is working|HAXM|WHPX") {
            Write-Info "Emulator acceleration: available."
        } else {
            Write-Warn "Emulator hardware acceleration may be UNAVAILABLE:"
            Write-Warn ($accel.Trim())
            Write-Warn "Enable Windows Hypervisor Platform (WHPX) + BIOS virtualization; see android-win.md section 5.2."
        }
    }
}

# ----------------------------------------------------------------------------
# 7. Optional clean
# ----------------------------------------------------------------------------
if ($Clean) {
    Write-Step "Clearing build caches (-Clean)"
    flutter clean | Out-Null
    foreach ($dir in @(
        (Join-Path $RepoRootFull "build"),
        (Join-Path $RepoRootFull "rust\target")
    )) {
        if (Test-Path $dir) {
            Write-Info "Removing $dir"
            Remove-Item -Recurse -Force $dir -ErrorAction SilentlyContinue
        }
    }
}

# ----------------------------------------------------------------------------
# 8. pub get (cheap; do it during provisioning so the project is ready)
# ----------------------------------------------------------------------------
if (-not $SkipPubGet) {
    Write-Step "flutter pub get"
    Invoke-Checked "flutter" @("pub", "get")
}

# ----------------------------------------------------------------------------
# 9. Launch the emulator (optional)
# ----------------------------------------------------------------------------
$EmulatorExe = Join-Path $AndroidSdkDir "emulator\emulator.exe"
$Adb         = Join-Path $AndroidSdkDir "platform-tools\adb.exe"
if ($LaunchEmulator -and -not $NoEmulator) {
    Write-Step "Launching emulator '$AvdName'"
    if (-not (Test-Path $EmulatorExe)) { throw "emulator.exe not found at $EmulatorExe." }
    # Start detached so the script can continue; the emulator keeps running.
    Start-Process -FilePath $EmulatorExe -ArgumentList @("-avd", $AvdName)
    Write-Info "Waiting for the device to come online (adb wait-for-device)..."
    & $Adb start-server | Out-Null
    & $Adb wait-for-device
    # Poll sys.boot_completed so we don't try to install before Android is up.
    for ($i = 0; $i -lt 60; $i++) {
        $booted = (& $Adb shell getprop sys.boot_completed 2>$null | Out-String).Trim()
        if ($booted -eq "1") { break }
        Start-Sleep -Seconds 5
    }
    Write-Info "Emulator booted."
}

# ----------------------------------------------------------------------------
# 10. Build (default; skipped with -NoBuild). Mirrors .github/actions/android/action.yml.
# ----------------------------------------------------------------------------
if ($Build) {
    Write-Step "Building Android APK (--split-per-abi, android-arm64,android-x64)"

    # Write rust/cargokit.yaml with the Android feature set. cargokit reads this file
    # and appends cargo.release.extra_flags to its cargo build. CRITICAL: this file is
    # SHARED with build-win.ps1, which writes `--features=ledger` for the DESKTOP build.
    # A stale ledger entry here makes the Android build pull in `hidapi`, which has no
    # Android target_os support and fails with "unresolved import `hidapi`". So we
    # (over)write it for Android - default empty, matching CI's android action.
    $CargokitYaml = Join-Path $RepoRootFull "rust\cargokit.yaml"
    if ([string]::IsNullOrWhiteSpace($Feature)) {
        $cargokitContent = "cargo:`n  release:`n    extra_flags: []`n"
        Write-Info "rust\cargokit.yaml: no extra features (Android default)."
    } else {
        $cargokitContent = "cargo:`n  release:`n    extra_flags:`n      - --features=$Feature`n"
        Write-Info "rust\cargokit.yaml: --features=$Feature"
    }
    Set-Content -Path $CargokitYaml -Value $cargokitContent -Encoding ASCII -NoNewline

    # NU7 consensus flag in the ENCODED form (unit-separator-joined), exactly as CI
    # sets CARGO_ENCODED_RUSTFLAGS. The encoded form avoids space-splitting in the
    # `--cfg zcash_unstable="nu7"` value. 0x1F is the ASCII unit separator.
    $US = [char]0x1F
    $env:CARGO_ENCODED_RUSTFLAGS = "--cfg${US}zcash_unstable=`"nu7`""
    Write-Info "CARGO_ENCODED_RUSTFLAGS set (nu7)."

    # Point openssl-sys at our prebuilt per-ABI OpenSSL instead of vendoring it from
    # source. OPENSSL_NO_VENDOR=1 defeats the crate's vendored-openssl feature; the
    # PER-TARGET vars (<RUST_TARGET_UPPER_UNDERSCORED>_OPENSSL_*) let each ABI link its
    # own static libs. openssl-sys reads e.g. AARCH64_LINUX_ANDROID_OPENSSL_DIR.
    if (-not $SkipOpenssl) {
        $env:OPENSSL_NO_VENDOR = "1"
        foreach ($rustTarget in $OpensslInstalls.Keys) {
            $dir = $OpensslInstalls[$rustTarget]
            $prefix = ($rustTarget.ToUpper() -replace '-', '_')   # aarch64-linux-android -> AARCH64_LINUX_ANDROID
            Set-Item -Path "Env:${prefix}_OPENSSL_DIR"    -Value ($dir.Replace('\','/'))
            Set-Item -Path "Env:${prefix}_OPENSSL_STATIC" -Value "1"
            # vcpkg/Windows name libs *.lib; ours are libssl.a/libcrypto.a (unix), so
            # the default lib names work - no *_OPENSSL_LIBS override needed.
            Write-Info "${prefix}_OPENSSL_DIR=$($dir.Replace('\','/'))  (STATIC=1)"
        }
        Write-Info "OPENSSL_NO_VENDOR=1 (using prebuilt OpenSSL, no source vendoring)."
    }

    Invoke-Checked "flutter" @(
        "build", "apk", "--split-per-abi",
        "--target-platform", "android-arm64,android-x64"
    )
    if ($Aab) {
        Write-Step "Building Android App Bundle (.aab)"
        Invoke-Checked "flutter" @(
            "build", "aab",
            "--target-platform", "android-arm64,android-x64"
        )
    }

    $ApkDir = Join-Path $RepoRootFull "build\app\outputs\flutter-apk"
    Write-Step "Build artifacts"
    Get-ChildItem $ApkDir -Filter *.apk -ErrorAction SilentlyContinue |
        Format-Table Name, @{N="SizeMB";E={[math]::Round($_.Length/1MB,1)}}, LastWriteTime

    # 10a. Copy + rename APKs (and .aab) to ./out with version-stamped names like
    #   zkool-6.17.1+221-arm64-v8a-release.apk
    # The version comes from pubspec.yaml `version: <semver>+<build>` (e.g. 6.17.1+221).
    # Flutter emits split APKs as app-<abi>-release.apk; we map those to the desired name.
    $PubspecVersion = $null
    $pubspecPath = Join-Path $RepoRootFull "pubspec.yaml"
    if (Test-Path $pubspecPath) {
        $verLine = Select-String -Path $pubspecPath -Pattern '^\s*version:\s*(.+?)\s*$' |
            Select-Object -First 1
        if ($verLine) { $PubspecVersion = $verLine.Matches[0].Groups[1].Value.Trim() }
    }
    if (-not $PubspecVersion) {
        Write-Warn "Could not read version from pubspec.yaml; skipping ./out copy."
    } else {
        $OutDir = Join-Path $RepoRootFull "out"
        New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
        Write-Info "Copying artifacts to $OutDir (version $PubspecVersion)."

        # Map Flutter's split-APK names -> ABI label used in the output filename.
        $apkAbiMap = @{
            "app-arm64-v8a-release.apk" = "arm64-v8a"
            "app-x86_64-release.apk"    = "x86_64"
            "app-armeabi-v7a-release.apk" = "armeabi-v7a"
        }
        $copied = @()
        foreach ($srcName in $apkAbiMap.Keys) {
            $src = Join-Path $ApkDir $srcName
            if (Test-Path $src) {
                $abi  = $apkAbiMap[$srcName]
                $dest = Join-Path $OutDir "zkool-$PubspecVersion-$abi-release.apk"
                Copy-Item -Path $src -Destination $dest -Force
                $copied += $dest
            }
        }
        # Fallback for a non-split (universal) APK build.
        $universal = Join-Path $ApkDir "app-release.apk"
        if ((Test-Path $universal) -and $copied.Count -eq 0) {
            $dest = Join-Path $OutDir "zkool-$PubspecVersion-universal-release.apk"
            Copy-Item -Path $universal -Destination $dest -Force
            $copied += $dest
        }

        # The .aab (if -Aab) lives in a different folder.
        if ($Aab) {
            $aabSrc = Join-Path $RepoRootFull "build\app\outputs\bundle\release\app-release.aab"
            if (Test-Path $aabSrc) {
                $dest = Join-Path $OutDir "zkool-$PubspecVersion-release.aab"
                Copy-Item -Path $aabSrc -Destination $dest -Force
                $copied += $dest
            }
        }

        if ($copied.Count -gt 0) {
            Write-Step "Copied to ./out"
            Get-ChildItem $OutDir |
                Format-Table Name, @{N="SizeMB";E={[math]::Round($_.Length/1MB,1)}}, LastWriteTime
        } else {
            Write-Warn "No APK/AAB artifacts found to copy into ./out."
        }
    }

    # 10b. Install + run on the emulator (-Run).
    if ($Run) {
        if ($NoEmulator) { throw "-Run needs an emulator, but -NoEmulator was passed." }
        Write-Step "Installing + launching zkool on the emulator"
        # `flutter install` picks the matching ABI (x86_64 for the emulator) for us.
        & $Adb wait-for-device
        Invoke-Checked "flutter" @("install", "-d", "emulator-5554")
        Write-Info "Installed. To hot-run from source instead: flutter run -d emulator-5554"
    }
}

# ----------------------------------------------------------------------------
# 11. Summary + next steps
# ----------------------------------------------------------------------------
Write-Step "Done"
Write-Host ""
Write-Host "Android SDK : $AndroidSdkDir" -ForegroundColor Green
Write-Host "Flutter SDK : $FlutterDir"    -ForegroundColor Green
Write-Host "NDK         : $NdkPath"        -ForegroundColor Green
if (-not $NoEmulator) {
    Write-Host "Emulator AVD: $AvdName"     -ForegroundColor Green
}
Write-Host ""
if (-not $Build) {
    Write-Host "Provisioning only - no APK was built (-NoBuild). Omit -NoBuild to compile." -ForegroundColor Yellow
}
Write-Host "Next steps (run in THIS PowerShell session, or re-run the script):" -ForegroundColor Cyan
Write-Host "  `$env:ANDROID_HOME = `"$AndroidSdkDir`""
if (-not $NoEmulator) {
    Write-Host "  & `"$EmulatorExe`" -avd $AvdName        # boot the emulator"
}
Write-Host "  flutter devices                              # confirm the emulator is seen"
Write-Host "  flutter run -d emulator-5554                 # hot-run from source"
Write-Host "  .\android-win.ps1 -NoBuild                   # provision only (no APK)"
Write-Host "  .\android-win.ps1 -LaunchEmulator -Run       # build + install + launch"
