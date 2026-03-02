<#
.SYNOPSIS
    Bootstrap script for Rustora — installs dependencies and launches the app.

.DESCRIPTION
    Checks prerequisites (Rust, Node.js), installs npm packages, generates
    Tauri icons if missing, then starts the development server.

.EXAMPLE
    .\setup.ps1          # Full setup + launch dev server
    .\setup.ps1 -Build   # Full setup + produce release .exe / MSI
#>

param(
    [switch]$Build
)

$ErrorActionPreference = "Stop"

function Assert-Command($Name) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        Write-Host "ERROR: '$Name' is not installed or not on PATH." -ForegroundColor Red
        exit 1
    }
}

Write-Host "`n=== Rustora Setup ===" -ForegroundColor Cyan

Write-Host "`nChecking prerequisites..."
Assert-Command "rustc"
Assert-Command "cargo"
Assert-Command "node"
Assert-Command "npm"

Write-Host "  rustc  : $(rustc --version)"
Write-Host "  cargo  : $(cargo --version)"
Write-Host "  node   : $(node --version)"
Write-Host "  npm    : $(npm --version)"

Push-Location "$PSScriptRoot\desktop_ui"

try {
    if (-not (Test-Path "node_modules")) {
        Write-Host "`nInstalling frontend dependencies..." -ForegroundColor Yellow
        npm install
        if ($LASTEXITCODE -ne 0) { throw "npm install failed" }
    } else {
        Write-Host "`nFrontend dependencies already installed." -ForegroundColor Green
    }

    if (-not (Test-Path "src-tauri\icons")) {
        Write-Host "`nGenerating application icons..." -ForegroundColor Yellow
        npx tauri icon src-tauri\icons\icon.png 2>$null
        if (-not (Test-Path "src-tauri\icons")) {
            Write-Host "  Icon generation requires a source image. Using default Tauri icon." -ForegroundColor DarkYellow
        }
    } else {
        Write-Host "Application icons already present." -ForegroundColor Green
    }

    if ($Build) {
        Write-Host "`nBuilding release binary..." -ForegroundColor Yellow
        Write-Host "  (First build may take 10-15 minutes — DuckDB compiles from C++ source)" -ForegroundColor DarkYellow
        npm run tauri build
        if ($LASTEXITCODE -ne 0) { throw "tauri build failed" }
        Write-Host "`nBuild complete!" -ForegroundColor Green
        Write-Host "  Executable : ..\target\release\Rustora.exe"
        Write-Host "  Installer  : ..\target\release\bundle\msi\"
    } else {
        Write-Host "`nLaunching Rustora in development mode..." -ForegroundColor Yellow
        Write-Host "  (First launch compiles the Rust backend — this may take several minutes)" -ForegroundColor DarkYellow
        npm run tauri dev
    }
} finally {
    Pop-Location
}
