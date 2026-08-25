#!/usr/bin/env bash
#
# distribute_app.sh - build and package egui_app for Linux / macOS
#
# Produces (from the repository root):
#   Linux:  dist/egui-app-<version>-linux-<arch>.tar.gz   (+ .sha256)
#   macOS:  dist/egui-app-<version>-macos-<arch>.zip      (+ .sha256)
#           (a proper .app bundle with the icon and Info.plist)
#
# Security notes:
#   - SHA-256 checksums are written next to every archive for integrity.
#   - The macOS .app is unsigned: Gatekeeper will warn until the bundle is
#     signed and notarized (CI signing can be added later). Linux binaries
#     need no signature by default.
#   - Nothing is bundled except the binary, the license and the README; no
#     user data or build artifacts leave the machine.
#
# Exit codes: 0 on success, 1 on any error.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

VER="$(awk -F'"' '/^version = /{print $2; exit}' Cargo.toml)"
if [ -z "$VER" ]; then
    echo "[ERROR] could not parse version from Cargo.toml" >&2
    exit 1
fi
echo "Version: $VER"

if ! command -v cargo >/dev/null 2>&1; then
    echo "[ERROR] cargo was not found on PATH." >&2
    echo "        Install the Rust toolchain from https://rustup.rs/ and retry." >&2
    exit 1
fi

echo "[1/3] Building release..."
cargo build --release

case "$(uname -s)" in
    Linux)  PLATFORM="linux"; ARCH="$(uname -m)" ;;
    Darwin) PLATFORM="macos"; ARCH="$(uname -m)" ;;
    *) echo "[ERROR] unsupported OS: $(uname -s)" >&2; exit 1 ;;
esac

STAGE="dist/staging/egui-app-$VER"
rm -rf dist/staging
mkdir -p "$STAGE"

echo "[2/3] Packaging ($PLATFORM/$ARCH)..."
if [ "$PLATFORM" = "linux" ]; then
    cp target/release/egui_app "$STAGE/egui_app"
    cp LICENSE README.md "$STAGE/"
    cp assets/icon.png assets/linux/egui-app.desktop "$STAGE/"
    chmod 755 "$STAGE/egui_app"
    (cd dist/staging && tar -czf "../egui-app-$VER-linux-$ARCH.tar.gz" "egui-app-$VER")
    OUT="dist/egui-app-$VER-linux-$ARCH.tar.gz"
else
    APP_BUNDLE="$STAGE/egui_app.app"
    mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources"
    cp target/release/egui_app "$APP_BUNDLE/Contents/MacOS/egui_app"
    cp assets/icon.icns "$APP_BUNDLE/Contents/Resources/icon.icns"
    cp assets/macos/Info.plist "$APP_BUNDLE/Contents/Info.plist"
    chmod +x "$APP_BUNDLE/Contents/MacOS/egui_app"
    cp LICENSE README.md "$STAGE/"
    (cd dist/staging && zip -rqy "../egui-app-$VER-macos-$ARCH.zip" "egui-app-$VER")
    OUT="dist/egui-app-$VER-macos-$ARCH.zip"
fi

echo "[3/3] Checksums..."
(cd dist && shasum -a 256 "$(basename "$OUT")" > "$(basename "$OUT").sha256")

rm -rf dist/staging
echo "Done:"
echo "  $OUT"
echo "  $OUT.sha256"
