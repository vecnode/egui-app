@echo off
setlocal

REM ============================================================================
REM  build_app.bat - build and run the egui-app template (Windows)
REM
REM  Usage:
REM    build_app.bat                 build (debug) and run
REM    build_app.bat --release       build (release) and run
REM    build_app.bat --build-only    build without launching the app
REM    build_app.bat --help          show this help
REM
REM  Exit codes: 0 on success, 1 on argument/build errors, otherwise the
REM  exit code of the launched application is propagated.
REM ============================================================================

set "SCRIPT_DIR=%~dp0"
pushd "%SCRIPT_DIR%" >nul 2>&1 || (
    echo [ERROR] Could not enter script directory: %SCRIPT_DIR%
    exit /b 1
)

set "PROFILE=debug"
set "CARGO_FLAGS="
set "RUN=1"

:parse_args
if "%~1"=="" goto :done_args
if /I "%~1"=="--release" (
    set "PROFILE=release"
    set "CARGO_FLAGS=--release"
    shift
    goto :parse_args
)
if /I "%~1"=="--build-only" (
    set "RUN=0"
    shift
    goto :parse_args
)
if /I "%~1"=="--help" (
    goto :show_help
)
echo [ERROR] Unknown argument: %~1
goto :show_help_error

:done_args

where cargo >nul 2>nul
if errorlevel 1 (
    echo [ERROR] cargo was not found on PATH.
    echo         Install the Rust toolchain from https://rustup.rs/ and retry.
    exit /b 1
)

echo [1/2] Building egui_app (%PROFILE%)...
cargo build %CARGO_FLAGS%
if errorlevel 1 (
    echo [ERROR] Build failed. See the cargo output above.
    exit /b 1
)

if "%RUN%"=="0" (
    echo Build succeeded. Binary: target\%PROFILE%\egui_app.exe
    exit /b 0
)

echo [2/2] Launching egui_app...
"target\%PROFILE%\egui_app.exe"
set "APP_EXIT=%ERRORLEVEL%"
echo App exited with code %APP_EXIT%.
exit /b %APP_EXIT%

:show_help
echo Usage: build_app.bat [--release] [--build-only] [--help]
echo.
echo   --release      build with the release profile (target\release\)
echo   --build-only   build but do not launch the app
echo   --help         show this help
exit /b 0

:show_help_error
echo Usage: build_app.bat [--release] [--build-only] [--help]
echo.
echo   --release      build with the release profile (target\release\)
echo   --build-only   build but do not launch the app
echo   --help         show this help
exit /b 1
