<#
.SYNOPSIS
    Bootstrap script for Rustora with npm and non-npm paths.

.DESCRIPTION
    Supports two workflows:
    - Full dev workflow with Node.js/npm (installs frontend deps, starts dev).
    - Cargo-only build workflow (no npm/npx required) for restricted environments.

.EXAMPLE
    .\setup.ps1                 # Full setup + launch dev server
    .\setup.ps1 -Build          # Full setup + produce release .exe / MSI
    .\setup.ps1 -Build -NoNpm   # Force cargo-only build path
#>

param(
    [switch]$Build,
    [switch]$NoNpm
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

function Test-CommandAvailable($Name) {
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Install-TauriCliIfMissing {
    if (-not (Test-CommandAvailable "cargo-tauri")) {
        Write-Host "`nInstalling Tauri CLI via Cargo (one-time)..." -ForegroundColor Yellow
        cargo install tauri-cli
        if ($LASTEXITCODE -ne 0) { throw "Failed to install tauri-cli" }
    }
}

$didPushLocation = $false

try {
    Write-Host "`n=== Rustora Setup ===" -ForegroundColor Cyan

    Write-Host "`nChecking prerequisites..."
    Assert-Command "rustc"
    Assert-Command "cargo"

    $hasNode = Test-CommandAvailable "node"
    $hasNpm = Test-CommandAvailable "npm"
    $useNoNpmPath = $NoNpm -or -not ($hasNode -and $hasNpm)

    Write-Host "  rustc  : $(rustc --version)"
    Write-Host "  cargo  : $(cargo --version)"
    if ($hasNode) {
        Write-Host "  node   : $(node --version)"
    } else {
        Write-Host "  node   : not found" -ForegroundColor DarkYellow
    }
    if ($hasNpm) {
        Write-Host "  npm    : $(npm --version)"
    } else {
        Write-Host "  npm    : not found" -ForegroundColor DarkYellow
    }

    if ($useNoNpmPath) {
        if (-not $Build) {
            throw "Node.js/npm are unavailable (or -NoNpm was set). Dev mode requires npm. Use '.\setup.ps1 -Build -NoNpm' (or '.\build_no_npm.ps1') to build without npm/npx."
        }

        Write-Host "`nUsing cargo-only build path (no npm/npx)." -ForegroundColor Yellow
        $noNpmBuildScript = Join-Path $PSScriptRoot "build_no_npm.ps1"
        if (-not (Test-Path $noNpmBuildScript)) {
            throw "Missing build_no_npm.ps1 at repo root."
        }

        & $noNpmBuildScript
        if ($LASTEXITCODE -ne 0) { throw "cargo-only build failed" }
        exit 0
    }

    Push-Location "$PSScriptRoot\desktop_ui"
    $didPushLocation = $true

    if (-not (Test-Path "node_modules")) {
        Write-Host "`nInstalling frontend dependencies..." -ForegroundColor Yellow
        npm install
        if ($LASTEXITCODE -ne 0) { throw "npm install failed" }
    } else {
        Write-Host "`nFrontend dependencies already installed." -ForegroundColor Green
    }

    if (-not (Test-Path "src-tauri\icons")) {
        Write-Host "`nGenerating application icons..." -ForegroundColor Yellow
        Install-TauriCliIfMissing
        cargo tauri icon src-tauri\icons\icon.png 2>$null
        if (-not (Test-Path "src-tauri\icons")) {
            Write-Host "  Icon generation requires a source image. Using default Tauri icon." -ForegroundColor DarkYellow
        }
    } else {
        Write-Host "Application icons already present." -ForegroundColor Green
    }

    if ($Build) {
        Write-Host "`nBuilding release binary..." -ForegroundColor Yellow
        Write-Host "  (First build may take 10-15 minutes - DuckDB compiles from C++ source)" -ForegroundColor DarkYellow
        Install-TauriCliIfMissing
        cargo tauri build
        if ($LASTEXITCODE -ne 0) { throw "cargo tauri build failed" }
        Write-Host "`nBuild complete!" -ForegroundColor Green
        Write-Host "  Executable : ..\target\release\Rustora.exe"
        Write-Host "  Installer  : ..\target\release\bundle\msi\"
    } else {
        Write-Host "`nLaunching Rustora in development mode..." -ForegroundColor Yellow
        Write-Host "  (First launch compiles the Rust backend - this may take several minutes)" -ForegroundColor DarkYellow
        Install-TauriCliIfMissing
        cargo tauri dev
    }
} catch {
    Write-Host "`nSETUP FAILED: $_" -ForegroundColor Red
    Write-Host $_.ScriptStackTrace -ForegroundColor DarkRed
    Write-Host "`nPress any key to exit..." -ForegroundColor DarkGray
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
} finally {
    if ($didPushLocation) {
        Pop-Location
    }
}
