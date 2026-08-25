#!/usr/bin/env bash
#
# build_app.sh - build and run the egui-app template (Linux / macOS)
#
# Usage:
#   ./build_app.sh                 build (debug) and run
#   ./build_app.sh --release       build (release) and run
#   ./build_app.sh --build-only    build without launching the app
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

usage() {
    cat <<'EOF'
Usage: build_app.sh [--release] [--build-only] [--help]

  --release      build with the release profile (target/release)
  --build-only   build but do not launch the app
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

if [ "$RUN" -eq 0 ]; then
    echo "Build succeeded. Binary: target/$PROFILE/egui_app"
    exit 0
fi

echo "[2/2] Launching egui_app..."
"target/$PROFILE/egui_app"
