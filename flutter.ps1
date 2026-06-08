#Requires -Version 5.1
<#
.SYNOPSIS
    Run the fast, Dart-only verification for zkool on Windows: `flutter pub get`,
    optionally `dart run build_runner` (freezed / riverpod / FRB-adjacent codegen),
    and `flutter analyze`.

.DESCRIPTION
    This is the cheap pre-flight to run BEFORE the slow native build
    (`build-win.ps1`). It compiles nothing in Rust and needs no MSVC / vcpkg /
    OpenSSL - it only exercises the Dart analyzer, so it surfaces Dart mistakes in
    seconds instead of after a ~10-minute cargokit Rust compile.

    Typical flow when you've changed Dart and/or regenerated FRB bindings:
        .\flutter.ps1            # pub get + analyze
        .\build-win.ps1          # full native build once analyze is clean

    Use -BuildRunner when you've changed @freezed / @riverpod declarations (or any
    annotation that drives generated *.g.dart / *.freezed.dart) and need those
    regenerated. Plain functions / widgets / enums do NOT need it.

    Flutter is provided the same way as build-win.ps1: this script clones a pinned
    Flutter into build-win\flutter (first run), or reuses one via -FlutterDir, or
    falls back to whatever `flutter` is on PATH with -FlutterVersion "".

.PARAMETER BuildRunner
    Run `dart run build_runner build --delete-conflicting-outputs` before analyze
    to regenerate freezed / riverpod outputs.

.PARAMETER Watch
    Run build_runner in watch mode (`dart run build_runner watch`) instead of a
    one-shot build. Implies -BuildRunner. Analyze is skipped in watch mode (the
    command runs until you Ctrl-C it).

.PARAMETER SkipPubGet
    Skip `flutter pub get` (useful for repeat runs).

.PARAMETER SkipAnalyze
    Skip `flutter analyze` (e.g. when you only want to regenerate code).

.PARAMETER FlutterVersion
    Flutter version (git branch/tag) to clone into build-win\flutter, mirroring
    build-win.ps1. Default: 3.44.1. Set to "" to use whatever `flutter` is on PATH.

.PARAMETER FlutterDir
    Use an existing Flutter SDK at this path instead of cloning. Its bin\ is
    prepended to PATH for this process only. Overrides -FlutterVersion.

.EXAMPLE
    .\flutter.ps1
    flutter pub get + flutter analyze.

.EXAMPLE
    .\flutter.ps1 -BuildRunner
    pub get + regenerate freezed/riverpod + analyze.

.EXAMPLE
    .\flutter.ps1 -FlutterDir C:\flutter -SkipPubGet
    Use an existing SDK, skip pub get, just analyze.
#>
[CmdletBinding()]
param(
    [switch]$BuildRunner,
    [switch]$Watch,
    [switch]$SkipPubGet,
    [switch]$SkipAnalyze,
    [string]$FlutterVersion = "3.44.1",
    [string]$FlutterDir     = ""
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

# Always operate from the script's directory (the repo root).
$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $RepoRoot

function Write-Step($msg) { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function Write-Info($msg) { Write-Host "    $msg" -ForegroundColor DarkGray }

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
if (-not (Test-Path (Join-Path $RepoRoot "pubspec.yaml"))) {
    throw "pubspec.yaml not found. Run this script from the zkool project root."
}

# Provide a Flutter SDK exactly like build-win.ps1: clone a pinned version into
# build-win\flutter, reuse one via -FlutterDir, or use whatever is on PATH.
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

Assert-Tool "flutter" "Install the Flutter SDK / clone via -FlutterVersion, or add its bin\ to PATH."
Assert-Tool "dart"    "dart ships with Flutter; ensure Flutter\bin is on PATH."

# ----------------------------------------------------------------------------
# 1. flutter pub get
# ----------------------------------------------------------------------------
if (-not $SkipPubGet) {
    Write-Step "flutter pub get"
    Invoke-Checked "flutter" @("pub", "get")
}

# ----------------------------------------------------------------------------
# 2. dart run build_runner (optional) - freezed / riverpod codegen
# ----------------------------------------------------------------------------
# Only needed when @freezed / @riverpod declarations changed. This regenerates
# *.g.dart / *.freezed.dart; it does NOT touch the FRB bindings (use codegen.ps1
# for those).
if ($Watch) {
    Write-Step "dart run build_runner watch (Ctrl-C to stop)"
    Write-Info "Watch mode runs until interrupted; analyze is skipped."
    Invoke-Checked "dart" @("run", "build_runner", "watch", "--delete-conflicting-outputs")
    return
}
if ($BuildRunner) {
    Write-Step "dart run build_runner build"
    Invoke-Checked "dart" @("run", "build_runner", "build", "--delete-conflicting-outputs")
}

# ----------------------------------------------------------------------------
# 3. flutter analyze (scoped to OUR code)
# ----------------------------------------------------------------------------
# Analyze only the app's own directories, not the cloned Flutter SDK
# (build-win\flutter) or cargokit's build_tool (rust_builder) - those are
# third-party trees with unresolved deps that produce thousands of bogus issues.
# analysis_options.yaml also excludes them, but passing explicit targets keeps
# the scope correct regardless of how analyze is invoked.
if (-not $SkipAnalyze) {
    $analyzeTargets = @("lib")
    if (Test-Path (Join-Path $RepoRoot "test")) { $analyzeTargets += "test" }
    Write-Step "flutter analyze $($analyzeTargets -join ' ')"
    Invoke-Checked "flutter" (@("analyze") + $analyzeTargets)
}

Write-Step "Done"
Write-Host "`nDart verification passed. You can now run .\build-win.ps1 for the full native build." -ForegroundColor Green
