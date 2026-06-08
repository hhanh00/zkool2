#Requires -Version 5.1
<#
.SYNOPSIS
    Compile zkool and produce zkool-<version>.zip locally on Windows, WITHOUT
    needing Strawberry Perl / NASM / a vendored-OpenSSL MSVC build.

.DESCRIPTION
    Problem: zkool's Rust crate pulls in OpenSSL (via SQLCipher /
    `bundled-sqlcipher-vendored-openssl` and arti/tor). cargokit normally
    compiles that Rust through the MSVC toolchain during `flutter build windows`,
    which forces OpenSSL to build from C source and needs a native Windows Perl
    + NASM. That path is fragile and broke locally.

    Solution (mirrors what zwallet does): build the Rust library OURSELVES
    first, using the MSYS2 / UCRT64 GNU toolchain (gcc + the prebuilt UCRT64
    OpenSSL), then hand the finished rlz.dll to Flutter so cargokit does not
    have to compile anything. Concretely:

      1. cargo build --target x86_64-pc-windows-gnu  (uses UCRT64 OpenSSL, no Perl/NASM)
         -> rlz.dll
      2. Replace the rlz plugin's windows/CMakeLists.txt with a version that
         skips cargokit (apply_cargokit) and bundles our prebuilt rlz.dll. rlz is
         an FFI plugin, so the runner .exe loads rlz.dll at runtime and never
         links it - no import lib is needed.
      3. flutter build windows  (run inside the UCRT64 environment; no cargo runs).
      4. Copy rlz.dll + liblzma-5.dll next to zkool.exe in the Release folder.
      5. Zip the Release folder as out\zkool-<version>.zip.

    The swapped rlz windows/CMakeLists.txt is restored on exit (even on failure).

.PARAMETER Feature
    Rust cargo feature to enable. CI uses "ledger". Default: ledger.

.PARAMETER Msys2Root
    Path to the MSYS2 installation. Default: C:\msys64.

.PARAMETER SkipRustBuild
    Skip the cargo build if target\x86_64-pc-windows-gnu\release\rlz.dll already exists.

.PARAMETER SkipPubGet
    Skip `flutter pub get` (useful for repeat builds).

.PARAMETER FlutterVersion
    Flutter version (git branch/tag) to clone into build-win\flutter and use for
    the build, mirroring zwallet's build-msys2.ps1. Default: 3.44.1. Set to ""
    to use whatever `flutter` is already on PATH instead of cloning.

.PARAMETER FlutterDir
    Use an existing Flutter SDK at this path instead of cloning. Its bin\ is
    prepended to PATH for this process only. Overrides -FlutterVersion.

.EXAMPLE
    .\build-msys2.ps1
    Builds out\zkool-%VERSION%.zip.

.EXAMPLE
    .\build-msys2.ps1 -SkipRustBuild
    Reuses an already-compiled rlz.dll and just re-runs the Flutter build + zip.

.EXAMPLE
    .\build-msys2.ps1 -FlutterDir C:\flutter
    Uses an already-installed Flutter SDK instead of cloning one.
#>
[CmdletBinding()]
param(
    [string]$Feature        = "ledger",
    [string]$Msys2Root      = "C:\msys64",
    [string]$FlutterVersion = "3.44.1",
    [string]$FlutterDir     = "",
    [switch]$SkipRustBuild,
    [switch]$SkipPubGet
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

# Always operate from the script's directory (the repo root).
$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $RepoRoot

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

# Provide a Flutter SDK. Like zwallet's build-msys2.ps1, we git-clone Flutter
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

# Resolve the UCRT64 sub-tree of MSYS2 (gcc + OpenSSL live here).
$Ucrt64    = Join-Path $Msys2Root "ucrt64"
$Ucrt64Bin = Join-Path $Ucrt64 "bin"
if (-not (Test-Path (Join-Path $Ucrt64Bin "gcc.exe"))) {
    throw "Could not find UCRT64 gcc at '$Ucrt64Bin\gcc.exe'. Install the MSYS2 'mingw-w64-ucrt-x86_64-gcc' package, or pass -Msys2Root <path>."
}
if (-not (Test-Path (Join-Path $Ucrt64 "include\openssl\opensslv.h"))) {
    throw "Could not find UCRT64 OpenSSL headers under '$Ucrt64\include\openssl'. Install 'mingw-w64-ucrt-x86_64-openssl' in MSYS2."
}
Write-Info "MSYS2 UCRT64 : $Ucrt64"

# The rive_common Flutter plugin forces the MSBuild "ClangCL" platform toolset
# (VS_PLATFORM_TOOLSET ClangCL). If the LLVM/clang-cl MSBuild integration isn't
# installed, the build dies late with "MSB8020: build tools for ClangCL cannot
# be found". Check for the toolset targets up front and fail with a fix hint.
$vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vswhere) {
    $vsPath = (& $vswhere -latest -products * -property installationPath 2>$null | Select-Object -First 1)
    if ($vsPath) {
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
        Write-Info "ClangCL toolset: OK ($vsPath)"
    }
} else {
    Write-Warn "vswhere.exe not found; skipping ClangCL toolset pre-check."
}

# Make the UCRT64 toolchain take precedence on PATH for THIS process only.
# (Nothing is changed globally; the change dies when the script exits.)
$env:PATH = "$Ucrt64Bin;$env:PATH"

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
# 2. Build the Rust library with the GNU / UCRT64 toolchain
# ----------------------------------------------------------------------------
# This is the whole point: compile rlz.dll ourselves using gcc + the prebuilt
# UCRT64 OpenSSL, so OpenSSL is NEVER built from source and Perl/NASM are not
# needed. RUSTFLAGS carries the nu7 cfg the crate requires.
$RustTarget   = "x86_64-pc-windows-gnu"
$RustOutDir   = Join-Path $RepoRoot "target\$RustTarget\release"
$RlzDll       = Join-Path $RustOutDir "rlz.dll"

Write-Step "Building Rust library for $RustTarget"

# Toolchain + OpenSSL environment (process-scoped only).
$env:RUSTFLAGS         = '--cfg zcash_unstable="nu7"'
$env:OPENSSL_DIR       = $Ucrt64.Replace('\', '/')
$env:OPENSSL_NO_VENDOR = "1"
$env:CC                = "gcc"
$env:CXX               = "g++"
Write-Info "OPENSSL_DIR=$($env:OPENSSL_DIR)  (OPENSSL_NO_VENDOR=1)"
Write-Info "RUSTFLAGS=$($env:RUSTFLAGS)"

# Ensure the gnu target is installed.
$installed = (rustup target list --installed) 2>$null
if ($installed -notcontains $RustTarget) {
    Write-Info "Installing rust target $RustTarget"
    Invoke-Checked "rustup" @("target", "add", $RustTarget)
}

if ($SkipRustBuild -and (Test-Path $RlzDll)) {
    Write-Info "SkipRustBuild set and rlz.dll exists - reusing $RlzDll"
} else {
    Invoke-Checked "cargo" @(
        "build", "--release",
        "--manifest-path", "rust/Cargo.toml",
        "--features", $Feature,
        "--target", $RustTarget
    )
}
if (-not (Test-Path $RlzDll)) { throw "Expected $RlzDll after cargo build, but it is missing." }
Write-Info "Built: $RlzDll"

# ----------------------------------------------------------------------------
# 3. Bypass cargokit entirely (no cargo run inside `flutter build windows`)
# ----------------------------------------------------------------------------
# `rlz` is a Flutter FFI plugin: the runner .exe does NOT link against it, it
# only LOADS rlz.dll at runtime. So we don't need an import lib at all - we just
# need rlz.dll copied into the Release folder. Flutter does that copy for every
# entry in the plugin's `<name>_bundled_libraries` CMake variable.
#
# We replace the rlz plugin's windows/CMakeLists.txt with a version that skips
# apply_cargokit() (which would re-run cargo + MSVC + OpenSSL) and instead points
# `rlz_bundled_libraries` straight at our prebuilt DLL. The original file is
# restored on exit.
Write-Step "Replacing rlz windows/CMakeLists.txt to use the prebuilt rlz.dll"

# Stash the prebuilt DLL at a stable, forward-slash path CMake can reference.
$PrebuiltDir = Join-Path $RepoRoot "build\zkool-prebuilt"
New-Item -ItemType Directory -Force -Path $PrebuiltDir | Out-Null
Copy-Item $RlzDll (Join-Path $PrebuiltDir "rlz.dll") -Force
$RlzDllCMake = (Join-Path $PrebuiltDir "rlz.dll").Replace('\', '/')

$RlzCMake    = Join-Path $RepoRoot "rust_builder\windows\CMakeLists.txt"
$RlzCMakeBak = "$RlzCMake.zkoolbak"
Copy-Item $RlzCMake $RlzCMakeBak -Force

$bypassCMake = @"
# TEMPORARY - generated by build-msys2.ps1 to bypass cargokit on Windows.
# The original file is restored when build-msys2.ps1 exits.
cmake_minimum_required(VERSION 3.14)
set(PROJECT_NAME "rlz")
project(`${PROJECT_NAME} LANGUAGES CXX)

# Use the Rust library we prebuilt with the MSYS2/UCRT64 GNU toolchain instead
# of compiling it here via cargokit (which would invoke cargo + MSVC + OpenSSL).
set(rlz_bundled_libraries
  "$RlzDllCMake"
  PARENT_SCOPE
)
"@
Set-Content -Path $RlzCMake -Value $bypassCMake -Encoding ASCII
Write-Info "Bypass CMakeLists written (backup: $RlzCMakeBak)"
Write-Info "Bundled DLL: $RlzDllCMake"

# Discard any stale CMake cache from earlier (cargokit-configured) runs so CMake
# re-generates from the bypass CMakeLists above. (Keeps build\zkool-prebuilt.)
$WinBuildDir = Join-Path $RepoRoot "build\windows"
if (Test-Path $WinBuildDir) {
    Write-Info "Removing stale CMake cache: $WinBuildDir"
    Remove-Item $WinBuildDir -Recurse -Force -ErrorAction SilentlyContinue
}

# ----------------------------------------------------------------------------
# 5. Flutter build + package (patch is reverted no matter what happens)
# ----------------------------------------------------------------------------
try {
    Write-Step "Ensuring Windows desktop support is enabled"
    flutter config --enable-windows-desktop | Out-Null

    if (-not $SkipPubGet) {
        Write-Step "flutter pub get"
        Invoke-Checked "flutter" @("pub", "get")
    }

    Write-Step "flutter build windows --release"
    Write-Info "cargokit is bypassed; the prebuilt rlz.dll is bundled directly (no cargo/OpenSSL)."
    Invoke-Checked "flutter" @(
        "build", "windows", "--release",
        "--dart-define", "FLUTTER_BUILD_NAME=$($Version.Split('+')[0])",
        "--dart-define", "FLUTTER_BUILD_NUMBER=$($Version.Split('+')[1])"
    )
}
finally {
    if (Test-Path $RlzCMakeBak) {
        Write-Step "Restoring original rlz windows/CMakeLists.txt"
        Move-Item $RlzCMakeBak $RlzCMake -Force
        Write-Info "Restored original CMakeLists.txt"
    }
}

# ----------------------------------------------------------------------------
# 6. Stage runtime DLLs next to zkool.exe
# ----------------------------------------------------------------------------
Write-Step "Staging runtime DLLs into the Release folder"
$ReleaseDir = Join-Path $RepoRoot "build\windows\x64\runner\Release"
if (-not (Test-Path (Join-Path $ReleaseDir "zkool.exe"))) {
    throw "Flutter build did not produce $ReleaseDir\zkool.exe"
}

# Our Rust DLL (cargokit normally bundles it, but copy explicitly to be safe).
Copy-Item $RlzDll (Join-Path $ReleaseDir "rlz.dll") -Force
Write-Info "Copied rlz.dll"

# The only external MSYS/UCRT64 runtime dependency of rlz.dll is liblzma-5.dll.
# (libgcc / libwinpthread are statically linked into rlz.dll by this build.)
$LzmaDll = Join-Path $Ucrt64Bin "liblzma-5.dll"
if (Test-Path $LzmaDll) {
    Copy-Item $LzmaDll (Join-Path $ReleaseDir "liblzma-5.dll") -Force
    Write-Info "Copied liblzma-5.dll"
} else {
    Write-Warn "liblzma-5.dll not found in $Ucrt64Bin - app may fail to start if rlz.dll needs it."
}

# ----------------------------------------------------------------------------
# 7. Package the Release folder as out\zkool-<version>.zip
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
