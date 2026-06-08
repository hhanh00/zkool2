#Requires -Version 5.1
<#
.SYNOPSIS
    Compile zkool and produce zkool-<version>.zip locally on Windows using the
    NATIVE MSVC toolchain (no MSYS2), WITHOUT needing Strawberry Perl / NASM /
    a vendored-OpenSSL from-source build.

.DESCRIPTION
    This is the MSVC alternative to build-win.ps1 (which uses the MSYS2 / UCRT64
    GNU toolchain). It builds zkool exactly the way CI does - letting Flutter /
    cargokit compile the Rust `rlz` crate during `flutter build windows` with the
    MSVC toolchain (x86_64-pc-windows-msvc) - and changes only ONE thing versus CI:
    it injects OPENSSL_* environment variables so the Rust openssl-sys crate links
    a PREBUILT vcpkg OpenSSL instead of compiling OpenSSL from source.

    The OpenSSL problem: zkool's Rust crate forces
    `libsqlite3-sys = { features = ["bundled-sqlcipher-vendored-openssl"] }`,
    whose `vendored-openssl` normally compiles OpenSSL FROM C SOURCE - which
    needs a native Windows Perl + NASM. CI gets away with it because the
    windows-latest runners ship Perl + NASM; locally we usually don't. We defeat
    it with `OPENSSL_NO_VENDOR=1`, which makes openssl-sys consume vcpkg's prebuilt
    OpenSSL instead of vendoring (the env var overrides the Cargo feature at build
    time). vcpkg ships its own Perl for ITS one-time build, so you never install
    Perl yourself.

    Why env injection is enough: cargokit runs `rustup run stable cargo build
    --target x86_64-pc-windows-msvc --features=...` and passes the current process
    environment straight through to cargo (it adds nothing for non-Android targets).
    So setting OPENSSL_* + RUSTFLAGS in this PowerShell process is all cargo needs.
    The feature list comes from rust/cargokit.yaml (cargo.release.extra_flags),
    exactly like CI's misc/mkcargokit_options.sh - this script writes that file.

    PREREQUISITE: OpenSSL must already be installed via vcpkg for the MSVC triplet.
    This script does NOT install it (see build-win.md). Install it once with:
        C:\vcpkg\vcpkg.exe install openssl:x64-windows-static-md

    Concretely:
      1. Verify vcpkg has openssl:x64-windows-static-md installed.
      2. Write rust/cargokit.yaml so cargokit builds with --features=<Feature>.
      3. Prepend the VS x64 toolchain to PATH for THIS process only (so the right
         link.exe wins even if MSYS2 etc. is on PATH), and set OPENSSL_* + RUSTFLAGS.
      4. flutter build windows --release  (cargokit compiles rlz.dll with MSVC,
         linking the vcpkg OpenSSL; no cargokit bypass, no manual cargo build).
      5. Zip the Release folder as out\zkool-<version>.zip.

.PARAMETER Feature
    Rust cargo feature to enable (written into rust/cargokit.yaml). CI uses
    "ledger". Default: ledger.

.PARAMETER VcpkgRoot
    Path to the vcpkg installation (must contain vcpkg.exe). Default: C:\vcpkg.

.PARAMETER OpensslTriplet
    vcpkg triplet for OpenSSL. Default: x64-windows-static-md (static OpenSSL libs
    linked against the dynamic CRT /MD, matching Rust-msvc's default). Do NOT use
    x64-windows-static (static CRT /MT) - it mismatches Rust and causes LNK2038.

.PARAMETER SkipPubGet
    Skip `flutter pub get` (useful for repeat builds).

.PARAMETER Clean
    Clear all build caches before building. Runs `flutter clean` and removes the
    `build\` directory plus cargokit's Rust target output. Use this when CMake
    complains the CMakeCache.txt was generated for a different source directory
    (which happens after copying/moving/cloning the project to a new path).

.PARAMETER FlutterVersion
    Flutter version (git branch/tag) to clone into build-win\flutter and use for
    the build, mirroring zwallet's build-win.ps1. Default: 3.44.1. Set to ""
    to use whatever `flutter` is already on PATH instead of cloning.

.PARAMETER FlutterDir
    Use an existing Flutter SDK at this path instead of cloning. Its bin\ is
    prepended to PATH for this process only. Overrides -FlutterVersion.

.EXAMPLE
    .\build-win.ps1
    Builds out\zkool-%VERSION%.zip (cargokit compiles the Rust during the build).

.EXAMPLE
    .\build-win.ps1 -Feature ""
    Builds with the crate's default features only.

.EXAMPLE
    .\build-win.ps1 -FlutterDir C:\flutter
    Uses an already-installed Flutter SDK instead of cloning one.
#>
[CmdletBinding()]
param(
    [string]$Feature        = "ledger",
    [string]$VcpkgRoot      = "C:\vcpkg",
    [string]$OpensslTriplet = "x64-windows-static-md",
    [string]$FlutterVersion = "3.44.1",
    [string]$FlutterDir     = "",
    [switch]$SkipPubGet,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

# Always operate from the script's directory (the repo root).
$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $RepoRoot

$Triple = "x86_64-pc-windows-msvc"

function Write-Step($msg) { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function Write-Info($msg) { Write-Host "    $msg" -ForegroundColor DarkGray }
function Write-Warn($msg) { Write-Host "    $msg" -ForegroundColor Yellow }

function Assert-Tool($name, $hint) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        throw "Required tool '$name' was not found on PATH. $hint"
    }
}

# Run an external command and throw if it returns a non-zero exit code.
function Invoke-Checked($exe, [string[]]$cmdArgs) {
    & $exe @cmdArgs
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed ($LASTEXITCODE): $exe $($cmdArgs -join ' ')"
    }
}

# ----------------------------------------------------------------------------
# 0. Prerequisites
# ----------------------------------------------------------------------------
Write-Step "Checking prerequisites"
Assert-Tool "cargo"   "Install Rust (rustup) so cargo is on PATH."

if (-not (Test-Path (Join-Path $RepoRoot "pubspec.yaml"))) {
    throw "pubspec.yaml not found. Run this script from the zkool project root."
}

# Provide a Flutter SDK. Like zwallet's build-win.ps1, we git-clone Flutter
# into build-win\flutter and prepend its bin\ to PATH for THIS process only
# (nothing is changed globally). -FlutterDir reuses an existing SDK; an empty
# -FlutterVersion falls back to whatever `flutter` is already on PATH.
if ($FlutterDir) {
    $PinnedFlutterBin = Join-Path $FlutterDir "bin"
    if (-not (Test-Path (Join-Path $PinnedFlutterBin "flutter.bat"))) {
        throw "No flutter.bat under '$PinnedFlutterBin'. Check -FlutterDir."
    }
    Write-Step "Using Flutter at $FlutterDir"
    $env:PATH = "$PinnedFlutterBin;$env:PATH"
} elseif ($FlutterVersion) {
    Assert-Tool "git" "git is needed to clone the Flutter SDK. Install from https://git-scm.com and add it to PATH."
    $FlutterDir = Join-Path $RepoRoot "build-win\flutter"
    $PinnedFlutterBin = Join-Path $FlutterDir "bin"
    if (-not (Test-Path (Join-Path $PinnedFlutterBin "flutter.bat"))) {
        Write-Step "Cloning Flutter $FlutterVersion into build-win\flutter"
        New-Item -ItemType Directory -Force -Path (Join-Path $RepoRoot "build-win") | Out-Null
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

# Now that Flutter is on PATH (cloned, pinned, or pre-existing), assert it resolves.
Assert-Tool "flutter" "Install the Flutter SDK / clone via -FlutterVersion, or add its bin\ to PATH."
Assert-Tool "dart"    "dart ships with Flutter; ensure Flutter\bin is on PATH."

# vcpkg supplies the prebuilt OpenSSL (so we never build it from source).
$VcpkgExe = Join-Path $VcpkgRoot "vcpkg.exe"
if (-not (Test-Path $VcpkgExe)) {
    throw "vcpkg.exe not found at '$VcpkgExe'. Install vcpkg (git clone https://github.com/microsoft/vcpkg; .\bootstrap-vcpkg.bat) and pass -VcpkgRoot <path>."
}
Write-Info "vcpkg: $VcpkgExe"

# Locate Visual Studio (provides cl.exe / link.exe for the msvc target).
$vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
if (-not (Test-Path $vswhere)) {
    throw "vswhere.exe not found at '$vswhere'. Install Visual Studio 2022 with the 'Desktop development with C++' workload."
}
$vsPath = (& $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null | Select-Object -First 1)
if (-not $vsPath) {
    $vsPath = (& $vswhere -latest -products * -property installationPath 2>$null | Select-Object -First 1)
}
if (-not $vsPath) {
    throw "No Visual Studio installation with the C++ toolset was found. Install VS 2022 'Desktop development with C++'."
}
Write-Info "Visual Studio: $vsPath"

# The rive_common Flutter plugin forces the MSBuild "ClangCL" platform toolset
# (VS_PLATFORM_TOOLSET ClangCL). If the LLVM/clang-cl MSBuild integration isn't
# installed, the build dies late with "MSB8020: build tools for ClangCL cannot
# be found". Check for the toolset targets up front and fail with a fix hint.
$clangToolset = Join-Path $vsPath "MSBuild\Microsoft\VC\v170\Platforms\x64\PlatformToolsets\ClangCL\Toolset.props"
if (-not (Test-Path $clangToolset)) {
    throw @"
The MSBuild 'ClangCL' platform toolset is not installed in:
    $vsPath
The rive_common plugin requires it. Install the LLVM/clang-cl components, then retry:

  & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\setup.exe" modify ``
      --installPath "$vsPath" ``
      --add Microsoft.VisualStudio.Component.VC.Llvm.Clang ``
      --add Microsoft.VisualStudio.Component.VC.Llvm.ClangToolset ``
      --quiet --norestart

(Or use the VS Installer GUI: Modify -> Individual components ->
 "C++ Clang Compiler for Windows" + "MSBuild support for LLVM (clang-cl) toolset".)
"@
}
Write-Info "ClangCL toolset: OK"

# Ensure the msvc rust target is installed (the rustup host here is gnu).
$installed = (rustup target list --installed) 2>$null
if ($installed -notcontains $Triple) {
    Write-Info "Installing rust target $Triple"
    Invoke-Checked "rustup" @("target", "add", $Triple)
}
Write-Info "Rust target: $Triple"

# ----------------------------------------------------------------------------
# 0b. Optionally clear build caches (-Clean)
# ----------------------------------------------------------------------------
# CMake records the absolute source/binary dir inside build\...\CMakeCache.txt.
# If the project is copied/moved/cloned to a NEW path, the stale cache still
# points at the OLD path and the build dies with:
#   "The current CMakeCache.txt directory ... is different than the directory
#    ... where CMakeCache.txt was created"
#   "The source ... does not match the source ... used to generate cache"
# Wiping the build output (and cargokit's Rust target) fixes it.
if ($Clean) {
    Write-Step "Clearing build caches (-Clean)"

    Write-Info "flutter clean"
    flutter clean | Out-Null   # best-effort; don't fail the whole build on this

    foreach ($dir in @(
        (Join-Path $RepoRoot "build"),
        (Join-Path $RepoRoot "rust\target"),
        (Join-Path $RepoRoot ".dart_tool")
    )) {
        if (Test-Path $dir) {
            Write-Info "Removing $dir"
            Remove-Item -Recurse -Force $dir -ErrorAction SilentlyContinue
        }
    }
    Write-Info "Build caches cleared."
}

# ----------------------------------------------------------------------------
# 1. Read version from pubspec.yaml
# ----------------------------------------------------------------------------
Write-Step "Reading version from pubspec.yaml"
$versionLine = Select-String -Path (Join-Path $RepoRoot "pubspec.yaml") -Pattern '^\s*version:\s*(.+)$' | Select-Object -First 1
if (-not $versionLine) { throw "Could not find a 'version:' line in pubspec.yaml" }
$Version = $versionLine.Matches[0].Groups[1].Value.Trim()
Write-Info "Version: $Version"
$ArtifactName = "zkool-$Version"

# ----------------------------------------------------------------------------
# 2. Verify vcpkg OpenSSL (MSVC ABI) is installed
# ----------------------------------------------------------------------------
# We point openssl-sys at this prebuilt copy (Section 4) so OpenSSL is NEVER
# built from C source - no Strawberry Perl, no NASM. The x64-windows-static-md
# triplet gives static OpenSSL libs linked against the dynamic CRT (/MD), which
# matches Rust-msvc's default CRT. (x64-windows-static would be /MT and clash.)
#
# This script does NOT install OpenSSL - it is a documented prerequisite (see
# build-win.md). Install it once with:
#     <VcpkgRoot>\vcpkg.exe install openssl:x64-windows-static-md
Write-Step "Verifying vcpkg OpenSSL ($OpensslTriplet)"
$OpensslDir = Join-Path $VcpkgRoot "installed\$OpensslTriplet"
$OpensslHdr = Join-Path $OpensslDir "include\openssl\opensslv.h"
$LibSsl     = Join-Path $OpensslDir "lib\libssl.lib"
$LibCrypto  = Join-Path $OpensslDir "lib\libcrypto.lib"

if (-not ((Test-Path $OpensslHdr) -and (Test-Path $LibSsl) -and (Test-Path $LibCrypto))) {
    throw @"
OpenSSL ($OpensslTriplet) was not found under:
    $OpensslDir
This is a prerequisite (see build-win.md). Install it once with:

    & "$VcpkgExe" install openssl:$OpensslTriplet

(One-time, ~10-20 min. vcpkg builds OpenSSL with its OWN bundled Perl, so you do
not need Perl installed. Do NOT use the x64-windows-static triplet - its /MT CRT
mismatches Rust-msvc and causes LNK2038.)
"@
}
Write-Info "OpenSSL (MSVC): $OpensslDir"

# ----------------------------------------------------------------------------
# 3. Tell cargokit which feature to build (same mechanism CI uses)
# ----------------------------------------------------------------------------
# CI runs `misc/mkcargokit_options.sh ledger > rust/cargokit.yaml`. cargokit reads
# rust/cargokit.yaml and appends cargo.release.extra_flags to its cargo build. We
# write the same file so `flutter build windows` compiles with --features=<Feature>.
Write-Step "Writing rust/cargokit.yaml (--features=$Feature)"
$CargokitYaml = Join-Path $RepoRoot "rust\cargokit.yaml"
if ([string]::IsNullOrWhiteSpace($Feature)) {
    # No feature: write an empty cargo options block (default features only).
    $cargokitContent = "cargo:`n  release:`n    extra_flags: []`n"
} else {
    $cargokitContent = "cargo:`n  release:`n    extra_flags:`n      - --features=$Feature`n"
}
Set-Content -Path $CargokitYaml -Value $cargokitContent -Encoding ASCII -NoNewline
Write-Info "Wrote $CargokitYaml"

# ----------------------------------------------------------------------------
# 4. Toolchain PATH + OpenSSL environment (THIS process only)
# ----------------------------------------------------------------------------
# We do NOT bypass cargokit. cargokit will run
#   rustup run stable cargo build --target x86_64-pc-windows-msvc --features=...
# during `flutter build windows`, inheriting THIS process's environment. So all
# we inject is:
#   * the VS x64 toolchain on PATH (so cargo's link step uses MSVC's link.exe,
#     not some other link.exe that may be ahead on PATH, e.g. MSYS2's), and
#   * OPENSSL_* so openssl-sys links the prebuilt vcpkg OpenSSL.
#
# OPENSSL_NO_VENDOR=1 is the linchpin: openssl-sys checks it at build time and
# skips vendoring even though `bundled-sqlcipher-vendored-openssl` turns the
# `vendored` feature on. So OpenSSL is linked from the vcpkg copy, not compiled.
Write-Step "Setting up the MSVC toolchain PATH + OpenSSL environment (process-scoped)"

# Prepend the VS x64 host tools (link.exe, cl.exe) to PATH for this process only.
$VsHostBin = Join-Path $vsPath "VC\Tools\MSVC"
if (Test-Path $VsHostBin) {
    $msvcVer = Get-ChildItem $VsHostBin -Directory | Sort-Object Name -Descending | Select-Object -First 1
    if ($msvcVer) {
        $hostX64 = Join-Path $msvcVer.FullName "bin\Hostx64\x64"
        if (Test-Path (Join-Path $hostX64 "link.exe")) {
            $env:PATH = "$hostX64;$env:PATH"
            Write-Info "MSVC tools on PATH: $hostX64"
        }
    }
}
# Note: cargo's cc-rs locates the full MSVC INCLUDE/LIB via the registry on its
# own, so a complete vcvars import is not required - we only need the right
# link.exe to win on PATH.

$env:RUSTFLAGS           = '--cfg zcash_unstable="nu7"'
$env:OPENSSL_NO_VENDOR   = "1"
$env:OPENSSL_STATIC      = "1"
$env:OPENSSL_DIR         = $OpensslDir.Replace('\', '/')
$env:OPENSSL_LIB_DIR     = (Join-Path $OpensslDir "lib").Replace('\', '/')
$env:OPENSSL_INCLUDE_DIR = (Join-Path $OpensslDir "include").Replace('\', '/')
# vcpkg names the libs libssl.lib / libcrypto.lib; openssl-sys wants the base
# names COLON-separated (not semicolon).
$env:OPENSSL_LIBS        = "libssl:libcrypto"
Write-Info "OPENSSL_DIR=$($env:OPENSSL_DIR)  (OPENSSL_NO_VENDOR=1, OPENSSL_STATIC=1)"
Write-Info "OPENSSL_LIBS=$($env:OPENSSL_LIBS)"
Write-Info "RUSTFLAGS=$($env:RUSTFLAGS)"

# ----------------------------------------------------------------------------
# 5. Flutter build (cargokit compiles rlz.dll with MSVC, linking vcpkg OpenSSL)
# ----------------------------------------------------------------------------
Write-Step "Ensuring Windows desktop support is enabled"
flutter config --enable-windows-desktop | Out-Null

if (-not $SkipPubGet) {
    Write-Step "flutter pub get"
    Invoke-Checked "flutter" @("pub", "get")
}

Write-Step "flutter build windows --release"
Write-Info "cargokit compiles the Rust during this step (no bypass); OpenSSL is linked from vcpkg."
Invoke-Checked "flutter" @(
    "build", "windows", "--release",
    "--dart-define", "FLUTTER_BUILD_NAME=$($Version.Split('+')[0])",
    "--dart-define", "FLUTTER_BUILD_NUMBER=$($Version.Split('+')[1])"
)

$ReleaseDir = Join-Path $RepoRoot "build\windows\x64\runner\Release"
if (-not (Test-Path (Join-Path $ReleaseDir "zkool.exe"))) {
    throw "Flutter build did not produce $ReleaseDir\zkool.exe"
}
if (-not (Test-Path (Join-Path $ReleaseDir "rlz.dll"))) {
    throw "Flutter build did not bundle rlz.dll into $ReleaseDir (cargokit build may have failed)."
}
Write-Info "Release folder ready: $ReleaseDir"

# ----------------------------------------------------------------------------
# 6. Package the Release folder as out\zkool-<version>.zip
# ----------------------------------------------------------------------------
Write-Step "Creating out\$ArtifactName.zip"
$OutDir = Join-Path $RepoRoot "out"
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$ZipPath = Join-Path $OutDir "$ArtifactName.zip"
if (Test-Path $ZipPath) { Remove-Item $ZipPath -Force }

# Zip the *contents* of the Release folder (so the zip root holds zkool.exe).
Compress-Archive -Path (Join-Path $ReleaseDir "*") -DestinationPath $ZipPath -CompressionLevel Optimal

if (-not (Test-Path $ZipPath)) { throw "Failed to create $ZipPath" }

Write-Step "Done"
Get-Item $ZipPath | Format-Table Name, @{N="SizeMB";E={[math]::Round($_.Length/1MB,1)}}, LastWriteTime
Write-Host "`nArtifact ready: $ZipPath" -ForegroundColor Green
