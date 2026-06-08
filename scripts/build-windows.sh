#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=== VHS-86 Windows Build Script ==="
echo "Building for x86_64-pc-windows-msvc..."

cd "${PROJECT_ROOT}"

# Check dependencies
echo "Checking dependencies..."
command -v cargo >/dev/null 2>&1 || { echo "cargo is required but not installed."; exit 1; }

# Add Windows target if needed
rustup target add x86_64-pc-windows-msvc 2>/dev/null || {
    echo "Warning: Windows target not available. Install with:"
    echo "  rustup target add x86_64-pc-windows-msvc"
    echo ""
    echo "Attempting native build instead..."
}

# Build release binary
echo "Building release binary..."
if cargo build --release --target x86_64-pc-windows-msvc 2>/dev/null; then
    BINARY_PATH="target/x86_64-pc-windows-msvc/release/vhs-86.exe"
else
    cargo build --release
    BINARY_PATH="target/release/vhs-86.exe"
fi

# Create distribution directory
DIST_DIR="${PROJECT_ROOT}/dist/windows"
mkdir -p "${DIST_DIR}"

# Copy binary
cp "${BINARY_PATH}" "${DIST_DIR}/vhs-86.exe"

# Create zip archive
ARCHIVE_NAME="vhs-86-windows-x86_64.zip"
echo "Creating archive: ${ARCHIVE_NAME}..."
cd "${DIST_DIR}"
if command -v zip >/dev/null 2>&1; then
    zip "${PROJECT_ROOT}/dist/${ARCHIVE_NAME}" vhs-86.exe
else
    echo "Warning: zip not available. Archive not created."
    echo "Binary available at: ${DIST_DIR}/vhs-86.exe"
fi

echo ""
echo "Build complete!"
echo "Binary: ${DIST_DIR}/vhs-86.exe"
if [ -f "${PROJECT_ROOT}/dist/${ARCHIVE_NAME}" ]; then
    echo "Archive: ${PROJECT_ROOT}/dist/${ARCHIVE_NAME}"
fi
