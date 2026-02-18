@echo off
echo.
echo  🦀  Starting Rustora...
echo.

cd /d "%~dp0desktop_ui"

:: Install dependencies if node_modules missing
if not exist "node_modules" (
    echo  📦  Installing frontend dependencies...
    call npm install
    echo.
)

echo  ⚡  Launching Tauri dev server...
echo     (First launch compiles Rust — may take a few minutes)
echo.
call npm run tauri dev
