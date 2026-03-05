<#
.SYNOPSIS
    Build Rustora (egui native UI) using only Cargo.

.DESCRIPTION
    Builds the pure-Rust desktop application with eframe/egui.
    No npm, Node.js, Tauri CLI, or JavaScript tooling required.

    Prerequisites: Rust (stable), C++ build tools.

.EXAMPLE
    .\build_egui.ps1            # Build release exe
    .\build_egui.ps1 -Debug     # Build debug exe (faster compilation)
#>

param(
    [switch]$Debug
)

$ErrorActionPreference = "Stop"

function Assert-Command($Name) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        Write-Host "ERROR: '$Name' is not installed or not on PATH." -ForegroundColor Red
        exit 1
    }
}

function Wait-IfInteractive {
    if (-not [Environment]::UserInteractive) { return }
    if ($env:CI -eq "true") { return }
    try {
        Write-Host "`nPress any key to exit..." -ForegroundColor DarkGray
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    } catch {}
}

try {

Write-Host "`n=== Rustora - Native egui Build ===" -ForegroundColor Cyan

Write-Host "`nChecking prerequisites..."
Assert-Command "rustc"
Assert-Command "cargo"
Write-Host "  rustc : $(rustc --version)"
Write-Host "  cargo : $(cargo --version)"

if ($Debug) {
    Write-Host "`nBuilding debug binary (desktop_egui)..." -ForegroundColor Yellow
    Write-Host "  (First build may take 10-15 minutes - DuckDB compiles from C++ source)" -ForegroundColor DarkYellow
    cargo build -p desktop_egui
    if ($LASTEXITCODE -ne 0) { throw "Build failed" }
    Write-Host "`nDebug build complete!" -ForegroundColor Green
    Write-Host "  Executable: target\debug\Rustora.exe"
} else {
    Write-Host "`nBuilding release binary (desktop_egui)..." -ForegroundColor Yellow
    Write-Host "  (First build may take 10-15 minutes - DuckDB compiles from C++ source)" -ForegroundColor DarkYellow
    cargo build --release -p desktop_egui
    if ($LASTEXITCODE -ne 0) { throw "Build failed" }
    Write-Host "`nRelease build complete!" -ForegroundColor Green
    Write-Host "  Executable: target\release\Rustora.exe"
}

} catch {
    Write-Host "`nBUILD FAILED: $_" -ForegroundColor Red
    Write-Host $_.ScriptStackTrace -ForegroundColor DarkRed
} finally {
    Wait-IfInteractive
}
