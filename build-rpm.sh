#!/usr/bin/env bash
set -euo pipefail

echo "======================================================================"
echo "          JARVIS VOICE CHAT - FEDORA LINUX RPM BUILD & PACKAGING      "
echo "======================================================================"
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 1. Check OS
if [ -f /etc/fedora-release ]; then
    echo "[Info] Detected Fedora: $(cat /etc/fedora-release)"
elif [ -f /etc/os-release ]; then
    echo "[Info] Detected Linux: $(grep PRETTY_NAME /etc/os-release | cut -d= -f2 | tr -d '\"')"
else
    echo "[Warning] Not explicitly Fedora, proceeding with dnf/rpm build..."
fi

# 2. Install Build-Time Dependencies via dnf
echo ""
echo "[1/4] Installing required development dependencies via dnf..."
if command -v dnf &> /dev/null; then
    sudo dnf install -y \
        gcc \
        gcc-c++ \
        cmake \
        clang \
        pkg-config \
        webkit2gtk4.1-devel \
        gtk3-devel \
        libayatana-appindicator-devel \
        librsvg2-devel \
        openssl-devel \
        dbus-devel \
        alsa-lib-devel \
        speech-dispatcher-devel \
        espeak-ng \
        rpm-build \
        rpmdevtools \
        curl \
        wget \
        tar
else
    echo "[Error] 'dnf' package manager not found. Please run this script on Fedora Linux."
    exit 1
fi

# 3. Check Rust Installation
echo ""
echo "[2/4] Verifying Rust & Cargo..."
if ! command -v cargo &> /dev/null; then
    if [ -f "$HOME/.cargo/bin/cargo" ]; then
        export PATH="$HOME/.cargo/bin:$PATH"
    else
        echo "Cargo not found. Installing Rust toolchain via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
fi
echo "Rust compiler: $(rustc --version)"
echo "Cargo package manager: $(cargo --version)"

# 4. Check Tauri CLI
echo ""
echo "[3/4] Checking Tauri CLI..."
if ! command -v cargo-tauri &> /dev/null; then
    echo "Installing tauri-cli via Cargo..."
    cargo install tauri-cli --version ^2.0.0 --locked
fi
echo "Tauri CLI: $(cargo tauri --version)"

# 5. Build Release RPM Package
echo ""
echo "[4/4] Building release binary and generating RPM package..."
echo "Running: cargo tauri build"
cargo tauri build

echo ""
echo "======================================================================"
echo "BUILD SUCCEEDED!"
echo "======================================================================"

RPM_DIR="src-tauri/target/release/bundle/rpm"
if [ -d "$RPM_DIR" ]; then
    echo "RPM Package(s) generated successfully:"
    ls -lh "$RPM_DIR"/*.rpm
    echo ""
    echo "To install the RPM package on Fedora, run:"
    echo "  sudo dnf install $RPM_DIR/*.rpm"
else
    echo "[Notice] Check src-tauri/target/release/bundle/ for output packages."
fi
echo "======================================================================"
