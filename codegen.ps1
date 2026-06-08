#Requires -Version 5.1
<#
.SYNOPSIS
    Regenerate the flutter_rust_bridge (FRB) bindings on Windows using the NATIVE
    MSVC toolchain, with the same Rust environment build-win.ps1 uses.

.DESCRIPTION
    `flutter_rust_bridge_codegen generate` runs `cargo expand` over the Rust crate
    to discover the API surface. That means it must COMPILE the crate - so it needs
    exactly the same toolchain + environment as a real build:

      * RUSTFLAGS='--cfg zcash_unstable="nu7"'  (NU7 consensus; required by CLAUDE.md)
      * the MSVC link.exe on PATH (so the link step uses MSVC, not MSYS2's link.exe)
      * OPENSSL_* so openssl-sys links the PREBUILT vcpkg OpenSSL instead of compiling
        OpenSSL from C source (which needs Strawberry Perl + NASM).

    Without the OPENSSL_* injection, codegen fails the same way a from-source build
    does (cc / gcc errors while vendoring OpenSSL/secp256k1). This script mirrors
    build-win.ps1's prerequisite checks and process-scoped environment, then runs
    the codegen instead of `flutter build windows`.

    It regenerates:
      lib/src/rust/**           (Dart bindings, incl. api/coin.dart)
      rust/src/frb_generated.rs (Rust wire glue)

    PREREQUISITE: OpenSSL must already be installed via vcpkg for the MSVC triplet
    (same as build-win.ps1). Install it once with:
        C:\vcpkg\vcpkg.exe install openssl:x64-windows-static-md

.PARAMETER VcpkgRoot
    Path to the vcpkg installation (must contain vcpkg.exe). Default: C:\vcpkg.

.PARAMETER OpensslTriplet
    vcpkg triplet for OpenSSL. Default: x64-windows-static-md (static OpenSSL libs
    linked against the dynamic CRT /MD, matching Rust-msvc's default). Do NOT use
    x64-windows-static (static CRT /MT) - it mismatches Rust and causes LNK2038.

.PARAMETER FrbVersion
    flutter_rust_bridge_codegen version to use. Must match the pinned dependency in
    pubspec.yaml / rust/Cargo.toml. Default: 2.12.0.

.PARAMETER CodegenRoot
    Directory to install flutter_rust_bridge_codegen into LOCALLY (cargo --root).
    Default: build-win\frb (alongside the cloned Flutter SDK). Its bin\ is prepended
    to PATH for THIS process only - nothing is installed globally into ~/.cargo/bin.

.PARAMETER InstallCodegen
    Force a (re)install of the matching flutter_rust_bridge_codegen binary into
    CodegenRoot. If omitted, the script auto-installs locally only when the binary
    isn't already present under CodegenRoot.

.EXAMPLE
    .\codegen.ps1
    Auto-installs flutter_rust_bridge_codegen into build-win\frb on first run
    (local, not global), then regenerates the bindings.

.EXAMPLE
    .\codegen.ps1 -InstallCodegen
    Forces a fresh local install of flutter_rust_bridge_codegen 2.12.0 into
    build-win\frb, then regenerates.

.EXAMPLE
    .\codegen.ps1 -CodegenRoot D:\tools\frb
    Installs/uses the codegen binary under D:\tools\frb instead of build-win\frb.
#>
[CmdletBinding()]
param(
    [string]$VcpkgRoot      = "C:\vcpkg",
    [string]$OpensslTriplet = "x64-windows-static-md",
    [string]$FrbVersion     = "2.12.0",
    [string]$CodegenRoot    = "",
    [switch]$InstallCodegen
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

# Always operate from the script's directory (the repo root).
$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $RepoRoot

# Where to install flutter_rust_bridge_codegen LOCALLY (cargo --root). Defaults to
# build-win\frb, alongside the cloned Flutter SDK, so nothing is installed globally.
if ([string]::IsNullOrWhiteSpace($CodegenRoot)) {
    $CodegenRoot = Join-Path $RepoRoot "build-win\frb"
}

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
Assert-Tool "cargo"  "Install Rust (rustup) so cargo is on PATH."
Assert-Tool "rustup" "Install Rust (rustup) so rustup is on PATH."

if (-not (Test-Path (Join-Path $RepoRoot "pubspec.yaml"))) {
    throw "pubspec.yaml not found. Run this script from the zkool project root."
}
if (-not (Test-Path (Join-Path $RepoRoot "flutter_rust_bridge.yaml"))) {
    throw "flutter_rust_bridge.yaml not found. Run this script from the zkool project root."
}

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

# Ensure the msvc rust target is installed (the rustup host here is gnu).
$installed = (rustup target list --installed) 2>$null
if ($installed -notcontains $Triple) {
    Write-Info "Installing rust target $Triple"
    Invoke-Checked "rustup" @("target", "add", $Triple)
}
Write-Info "Rust target: $Triple"

# ----------------------------------------------------------------------------
# 1. Verify vcpkg OpenSSL (MSVC ABI) is installed
# ----------------------------------------------------------------------------
# codegen's `cargo expand` compiles the crate, so openssl-sys must link the
# prebuilt vcpkg OpenSSL (Section 3) - NEVER built from C source (no Perl/NASM).
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
# 2. Ensure flutter_rust_bridge_codegen is available (matching the pinned version)
# ----------------------------------------------------------------------------
# The codegen binary version MUST match the flutter_rust_bridge dependency
# (pubspec.yaml + rust/Cargo.toml = $FrbVersion); a mismatch produces bindings
# that don't agree with the runtime and break at load time.
#
# We install LOCALLY under $CodegenRoot (cargo --root) - NOT globally into
# ~/.cargo/bin - and prepend $CodegenRoot\bin to PATH for THIS process only. This
# keeps the codegen toolchain self-contained inside build-win\, like the cloned
# Flutter SDK, so nothing leaks into the user's global cargo install set.
$LocalFrbBin = Join-Path $CodegenRoot "bin"
$LocalFrbExe = Join-Path $LocalFrbBin "flutter_rust_bridge_codegen.exe"

# Prepend the local bin first so a matching local install always wins over any
# global one already on PATH.
if (Test-Path $LocalFrbBin) {
    $env:PATH = "$LocalFrbBin;$env:PATH"
}

# Decide whether to (re)install locally:
#   -InstallCodegen        -> always (re)install at the pinned version
#   no local exe present   -> auto-install locally (self-contained, low risk)
$needInstall = $InstallCodegen.IsPresent -or (-not (Test-Path $LocalFrbExe))
if ($needInstall) {
    Assert-Tool "cargo" "Install Rust (rustup) so cargo is on PATH."
    Write-Step "Installing flutter_rust_bridge_codegen $FrbVersion into $CodegenRoot"
    Write-Info "Local install (cargo --root) - nothing is installed globally."
    New-Item -ItemType Directory -Force -Path $CodegenRoot | Out-Null
    Invoke-Checked "cargo" @(
        "install", "flutter_rust_bridge_codegen",
        "--version", $FrbVersion, "--locked",
        "--root", $CodegenRoot, "--force"
    )
    # Ensure the freshly-installed bin is on PATH (it may not have existed above).
    if ($env:PATH -notlike "*$LocalFrbBin*") {
        $env:PATH = "$LocalFrbBin;$env:PATH"
    }
}

Assert-Tool "flutter_rust_bridge_codegen" "Local install failed. Re-run with -InstallCodegen, or check cargo output above."
$frbResolved = (Get-Command flutter_rust_bridge_codegen).Source
$frbActual   = (& flutter_rust_bridge_codegen --version) 2>$null
Write-Info "flutter_rust_bridge_codegen: $frbActual"
Write-Info "  resolved from: $frbResolved (pinned dependency: $FrbVersion)"
if ($frbResolved -notlike "$LocalFrbBin*") {
    Write-Warn "Using a flutter_rust_bridge_codegen that is NOT the local build-win\frb copy."
    Write-Warn "Re-run with -InstallCodegen to install the pinned $FrbVersion locally."
}
if ($frbActual -and ($frbActual -notmatch [regex]::Escape($FrbVersion))) {
    Write-Warn "Codegen binary version does not contain '$FrbVersion'. Bindings may mismatch the runtime. Re-run with -InstallCodegen to fix."
}

# ----------------------------------------------------------------------------
# 3. Toolchain PATH + OpenSSL environment (THIS process only)
# ----------------------------------------------------------------------------
# Mirror build-win.ps1: prepend the VS x64 toolchain so MSVC's link.exe wins on
# PATH, and inject OPENSSL_* + RUSTFLAGS so the crate compiles during codegen.
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

# Force cargo to build for the MSVC triple during codegen (the rustup host here
# is gnu, which is what failed the from-source OpenSSL/secp256k1 build).
$env:CARGO_BUILD_TARGET  = $Triple

$env:RUSTFLAGS           = '--cfg zcash_unstable="nu7"'
$env:OPENSSL_NO_VENDOR   = "1"
$env:OPENSSL_STATIC      = "1"
$env:OPENSSL_DIR         = $OpensslDir.Replace('\', '/')
$env:OPENSSL_LIB_DIR     = (Join-Path $OpensslDir "lib").Replace('\', '/')
$env:OPENSSL_INCLUDE_DIR = (Join-Path $OpensslDir "include").Replace('\', '/')
# vcpkg names the libs libssl.lib / libcrypto.lib; openssl-sys wants the base
# names COLON-separated (not semicolon).
$env:OPENSSL_LIBS        = "libssl:libcrypto"
Write-Info "CARGO_BUILD_TARGET=$($env:CARGO_BUILD_TARGET)"
Write-Info "OPENSSL_DIR=$($env:OPENSSL_DIR)  (OPENSSL_NO_VENDOR=1, OPENSSL_STATIC=1)"
Write-Info "OPENSSL_LIBS=$($env:OPENSSL_LIBS)"
Write-Info "RUSTFLAGS=$($env:RUSTFLAGS)"

# ----------------------------------------------------------------------------
# 4. Run the codegen
# ----------------------------------------------------------------------------
Write-Step "flutter_rust_bridge_codegen generate"
Write-Info "Regenerates lib/src/rust/** and rust/src/frb_generated.rs from crate::api."
Invoke-Checked "flutter_rust_bridge_codegen" @("generate")

# ----------------------------------------------------------------------------
# 5. Format the generated Dart (backstop for FRB's internal `dart format`)
# ----------------------------------------------------------------------------
# FRB runs `dart format` on its output as a final step, but that step can SILENTLY
# fail - e.g. if `dart` isn't on PATH at codegen time, or (the trap that bit us) if
# analysis_options.yaml is unparseable by dart_style's YAML reader. An unquoted glob
# like `- **/*_generated*` is read as a YAML *alias* and crashes the formatter, so the
# files land RAW (long one-liners, stray indentation). We re-run `dart format`
# explicitly here so the committed output is always formatted regardless.
#
# Width 80 matches the existing committed FRB style (FRB uses the default dart_style
# width, NOT the project's formatter.page_width). Keep this at 80 so the diff stays
# minimal and consistent with CI-generated bindings.
Write-Step "dart format lib\src\rust (line length 80, matches committed FRB style)"
# Prefer the pinned Flutter's dart; fall back to whatever `dart` is on PATH.
$DartExe = $null
$PinnedDart = Join-Path $RepoRoot "build-win\flutter\bin\dart.bat"
if (Test-Path $PinnedDart) {
    $DartExe = $PinnedDart
} elseif (Get-Command "dart" -ErrorAction SilentlyContinue) {
    $DartExe = "dart"
}
if ($DartExe) {
    & $DartExe format --line-length 80 (Join-Path $RepoRoot "lib\src\rust")
    if ($LASTEXITCODE -ne 0) {
        Write-Warn "dart format exited with $LASTEXITCODE. If it complains about analysis_options.yaml,"
        Write-Warn "check for an unquoted glob starting with '*' (must be quoted, e.g. \"**/*_generated*\")."
    }
} else {
    Write-Warn "No 'dart' found (pinned or on PATH) - skipping format. The generated files may be"
    Write-Warn "UNFORMATTED. Run: dart format --line-length 80 lib\src\rust"
}

Write-Step "Done"
Write-Host "`nBindings regenerated + formatted. Review the diff under lib/src/rust/ and rust/src/frb_generated.rs." -ForegroundColor Green
