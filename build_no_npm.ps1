<#
.SYNOPSIS
    Build Rustora using only Cargo — no npm/npx/Node.js required.

.DESCRIPTION
    Uses the pre-built frontend (desktop_ui/dist/) and a standalone Tauri
    config to compile Rustora.exe without any JavaScript tooling.

    Prerequisites: Rust (stable), C++ build tools.

.EXAMPLE
    .\build_no_npm.ps1            # Build release exe + MSI installer
    .\build_no_npm.ps1 -Debug     # Build debug exe (faster compilation)
#>

param(
    [switch]$Debug
)

$ErrorActionPreference = "Stop"

function Assert-Command($Name) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        Write-Host "ERROR: '$Name' is not installed or not on PATH." -ForegroundColor Red
        Write-Host "`nPress any key to exit..." -ForegroundColor DarkGray
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 1
    }
}

try {

Write-Host "`n=== Rustora — Cargo-Only Build ===" -ForegroundColor Cyan

Write-Host "`nChecking prerequisites..."
Assert-Command "rustc"
Assert-Command "cargo"
Write-Host "  rustc : $(rustc --version)"
Write-Host "  cargo : $(cargo --version)"

$distPath = Join-Path $PSScriptRoot "desktop_ui\dist\index.html"
if (-not (Test-Path $distPath)) {
    Write-Host "ERROR: Pre-built frontend not found at desktop_ui\dist\." -ForegroundColor Red
    Write-Host "       The dist/ folder must be present in the repo for cargo-only builds." -ForegroundColor Red
    Write-Host "       Pull the latest from git, or run 'cd desktop_ui && npm run build' once." -ForegroundColor Yellow
    Write-Host "`nPress any key to exit..." -ForegroundColor DarkGray
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}
Write-Host "  Frontend dist/ found." -ForegroundColor Green

$tauriCli = Get-Command "cargo-tauri" -ErrorAction SilentlyContinue
if (-not $tauriCli) {
    Write-Host "`nInstalling Tauri CLI via Cargo (one-time)..." -ForegroundColor Yellow
    cargo install tauri-cli
    if ($LASTEXITCODE -ne 0) { throw "Failed to install tauri-cli" }
}

$confPath = "src-tauri\tauri.standalone.conf.json"

if ($Debug) {
    Write-Host "`nBuilding debug binary..." -ForegroundColor Yellow
    Write-Host "  (First build may take 10-15 minutes — DuckDB compiles from C++ source)" -ForegroundColor DarkYellow
    Push-Location "$PSScriptRoot\desktop_ui"
    try {
        cargo tauri build --debug --config $confPath
        if ($LASTEXITCODE -ne 0) { throw "Build failed" }
    } finally { Pop-Location }

    Write-Host "`nDebug build complete!" -ForegroundColor Green
    Write-Host "  Executable: target\debug\Rustora.exe"
} else {
    Write-Host "`nBuilding release binary..." -ForegroundColor Yellow
    Write-Host "  (First build may take 10-15 minutes — DuckDB compiles from C++ source)" -ForegroundColor DarkYellow
    Push-Location "$PSScriptRoot\desktop_ui"
    try {
        cargo tauri build --config $confPath
        if ($LASTEXITCODE -ne 0) { throw "Build failed" }
    } finally { Pop-Location }

    Write-Host "`nRelease build complete!" -ForegroundColor Green
    Write-Host "  Executable : target\release\Rustora.exe"
    Write-Host "  Installer  : target\release\bundle\msi\"
}

} catch {
    Write-Host "`nBUILD FAILED: $_" -ForegroundColor Red
    Write-Host $_.ScriptStackTrace -ForegroundColor DarkRed
} finally {
    Write-Host "`nPress any key to exit..." -ForegroundColor DarkGray
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
}
