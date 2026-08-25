@echo off
setlocal

REM ============================================================================
REM  distribute_app.bat - build and package egui_app for Windows distribution
REM
REM  Produces (from the repository root):
REM    dist\egui-app-<version>-windows-x86_64.zip
REM    dist\egui-app-<version>-windows-x86_64.zip.sha256   (SHA-256 checksum)
REM
REM  The zip is portable: no installer and no admin rights are required.
REM
REM  Security notes:
REM    - The .exe is unsigned, so SmartScreen may warn on first run; sign the
REM      binary before broad distribution (CI signing can be added later).
REM    - The SHA-256 checksum lets users verify download integrity.
REM    - Nothing is bundled except the executable, the license and the README;
REM      no user data or build artifacts leave the machine.
REM ============================================================================

set "SCRIPT_DIR=%~dp0"
pushd "%SCRIPT_DIR%" >nul 2>&1 || exit /b 1

where cargo >nul 2>nul
if errorlevel 1 (
    echo [ERROR] cargo was not found on PATH. Install Rust from https://rustup.rs/ and retry.
    exit /b 1
)

for /f "tokens=2 delims==" %%V in ('findstr /b "version" Cargo.toml') do set "VER=%%V"
set "VER=%VER: =%"
set "VER=%VER:"=%"
if not defined VER (
    echo [ERROR] could not parse version from Cargo.toml
    exit /b 1
)
echo Version: %VER%

echo [1/3] Building release...
cargo build --release
if errorlevel 1 (
    echo [ERROR] release build failed. See the cargo output above.
    exit /b 1
)

set "ARCH=x86_64"
set "STAGE=dist\staging\egui-app-%VER%"
set "OUT=dist\egui-app-%VER%-windows-%ARCH%.zip"

echo [2/3] Packaging...
if exist "dist\staging" rmdir /s /q "dist\staging"
mkdir "%STAGE%" >nul 2>&1
if errorlevel 1 (
    echo [ERROR] could not create %STAGE%
    exit /b 1
)
copy /y "target\release\egui_app.exe" "%STAGE%\egui_app.exe" >nul
copy /y "LICENSE" "%STAGE%\LICENSE" >nul
copy /y "README.md" "%STAGE%\README.md" >nul
copy /y "assets\icon.ico" "%STAGE%\icon.ico" >nul

echo [3/3] Compressing and checksumming...
powershell -NoProfile -ExecutionPolicy Bypass -Command "Compress-Archive -Path '%STAGE%' -DestinationPath '%OUT%' -Force"
if errorlevel 1 (
    echo [ERROR] could not create the zip archive
    exit /b 1
)
powershell -NoProfile -ExecutionPolicy Bypass -Command "$h = (Get-FileHash -Algorithm SHA256 -Path '%OUT%').Hash; Set-Content -Path '%OUT%.sha256' -Value ($h + '  ' + (Split-Path '%OUT%' -Leaf))"
if errorlevel 1 (
    echo [ERROR] could not write the checksum file
    exit /b 1
)
rmdir /s /q "dist\staging"

echo Done:
echo   %OUT%
echo   %OUT%.sha256
