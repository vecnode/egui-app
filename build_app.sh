#!/usr/bin/env bash
#
# build_app.sh - build and run the egui-app template (Linux / macOS)
#
# Usage:
#   ./build_app.sh                 build (debug) and run
#   ./build_app.sh --release       build (release) and run
#   ./build_app.sh --build-only    build without launching the app
#   ./build_app.sh --bundle        build and wrap in a macOS .app bundle
#   ./build_app.sh --help          show this help
#
# Exit codes: 0 on success, 1 on argument/build errors, otherwise the
# exit code of the launched application is propagated.
#
# NOTE: on Linux, building eframe/egui may require system libraries such as
# libxcb, libxkbcommon and libgtk-3; see the README "Prerequisites" section.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

PROFILE="debug"
CARGO_FLAGS=()
RUN=1
BUNDLE=0

usage() {
    cat <<'EOF'
Usage: build_app.sh [--release] [--build-only] [--bundle] [--help]

  --release      build with the release profile (target/release)
  --build-only   build but do not launch the app
  --bundle       macOS only: also wrap the binary in egui_app.app with the
                 app icon (assets/icon.icns) and Info.plist
  --help         show this help
EOF
}

for arg in "$@"; do
    case "$arg" in
        --release)
            PROFILE="release"
            CARGO_FLAGS+=(--release)
            ;;
        --build-only)
            RUN=0
            ;;
        --bundle)
            BUNDLE=1
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            echo "[ERROR] Unknown argument: $arg" >&2
            usage >&2
            exit 1
            ;;
    esac
done

if ! command -v cargo >/dev/null 2>&1; then
    echo "[ERROR] cargo was not found on PATH." >&2
    echo "        Install the Rust toolchain from https://rustup.rs/ and retry." >&2
    exit 1
fi

echo "[1/2] Building egui_app ($PROFILE)..."
cargo build "${CARGO_FLAGS[@]}"

if [ "$BUNDLE" -eq 1 ]; then
    if [ "$(uname -s)" != "Darwin" ]; then
        echo "[WARN] --bundle is only supported on macOS; skipping the .app bundle." >&2
    else
        APP_BUNDLE="target/$PROFILE/egui_app.app"
        mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources"
        cp "target/$PROFILE/egui_app" "$APP_BUNDLE/Contents/MacOS/egui_app"
        cp assets/icon.icns "$APP_BUNDLE/Contents/Resources/icon.icns"
        cp assets/macos/Info.plist "$APP_BUNDLE/Contents/Info.plist"
        chmod +x "$APP_BUNDLE/Contents/MacOS/egui_app"
        echo "Bundle created: $APP_BUNDLE"
    fi
fi

if [ "$RUN" -eq 0 ]; then
    echo "Build succeeded. Binary: target/$PROFILE/egui_app"
    exit 0
fi

echo "[2/2] Launching egui_app..."
"target/$PROFILE/egui_app"
