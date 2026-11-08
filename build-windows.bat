@echo off
setlocal enabledelayedexpansion

echo ======================================================================
echo           JARVIS VOICE CHAT - WINDOWS BUILD ^& PACKAGING
echo ======================================================================
echo.

cd /d "%~dp0"

:: 1. Check for Rust / Cargo
echo [1/4] Checking Rust installation...
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
        set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
        echo Found Cargo in %USERPROFILE%\.cargo\bin.
    ) else (
        echo Rust/Cargo was not found in PATH.
        echo Attempting to download and install Rust via rustup-init.exe...
        powershell -NoProfile -Command "Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile '%TEMP%\rustup-init.exe'"
        if exist "%TEMP%\rustup-init.exe" (
            echo Running rustup installer (default profile)...
            "%TEMP%\rustup-init.exe" -y --default-toolchain stable
            set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
            del "%TEMP%\rustup-init.exe"
        ) else (
            echo ERROR: Failed to download rustup-init.exe.
            echo Please install Rust manually from https://rustup.rs and re-run this script.
            pause
            exit /b 1
        )
    )
)
cargo --version
rustc --version
echo Rust is ready.
echo.

:: 2. Check for CMake (whisper.cpp build requirement)
echo [2/4] Checking CMake...
where cmake >nul 2>&1
if %errorlevel% neq 0 (
    if exist "C:\Program Files\CMake\bin\cmake.exe" (
        set "PATH=C:\Program Files\CMake\bin;%PATH%"
        echo Found CMake in C:\Program Files\CMake\bin.
    ) else (
        echo WARNING: CMake was not detected in PATH.
        echo whisper.cpp compilation requires CMake and C++ build tools.
        echo If the build fails, install CMake from https://cmake.org/download/
    )
) else (
    cmake --version | findstr /i "version"
    echo CMake is ready.
)
echo.

:: 3. Check for Tauri CLI
echo [3/4] Checking Tauri CLI (cargo-tauri)...
where cargo-tauri >nul 2>&1
if %errorlevel% neq 0 (
    echo Installing tauri-cli via Cargo (this may take a few minutes on first run)...
    cargo install tauri-cli --version ^2.0.0 --locked
    if %errorlevel% neq 0 (
        echo ERROR: Failed to install tauri-cli.
        pause
        exit /b 1
    )
) else (
    echo Tauri CLI is already installed.
)
echo.

:: 4. Build Windows App Bundle (MSI & EXE)
echo [4/4] Building release binary and MSI package...
echo Running: cargo tauri build
cargo tauri build

if %errorlevel% neq 0 (
    echo.
    echo ======================================================================
    echo BUILD FAILED!
    echo Please check the error messages above.
    echo Ensure Visual Studio C++ Build Tools and CMake are installed.
    echo ======================================================================
    pause
    exit /b %errorlevel%
)

echo.
echo ======================================================================
echo BUILD SUCCEEDED!
echo.
echo Output installers generated:
if exist "src-tauri\target\release\bundle\msi" (
    echo MSI Installer:
    dir /b /s "src-tauri\target\release\bundle\msi\*.msi"
)
if exist "src-tauri\target\release\bundle\nsis" (
    echo NSIS Installer:
    dir /b /s "src-tauri\target\release\bundle\nsis\*.exe"
)
echo ======================================================================
pause
