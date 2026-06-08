#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=== VHS-86 macOS Build Script ==="
echo "Building for x86_64-apple-darwin and aarch64-apple-darwin..."

cd "${PROJECT_ROOT}"

# Check dependencies
echo "Checking dependencies..."
command -v cargo >/dev/null 2>&1 || { echo "cargo is required but not installed."; exit 1; }

# Install dependencies if needed
if command -v brew >/dev/null 2>&1; then
    echo "Installing build dependencies..."
    brew install libgit2 || true
fi

# Build for Intel Macs
echo "Building for x86_64-apple-darwin..."
rustup target add x86_64-apple-darwin 2>/dev/null || true
cargo build --release --target x86_64-apple-darwin 2>/dev/null || cargo build --release

# Build for Apple Silicon
echo "Building for aarch64-apple-darwin..."
rustup target add aarch64-apple-darwin 2>/dev/null || true
cargo build --release --target aarch64-apple-darwin 2>/dev/null || echo "Warning: Apple Silicon build may require cross-compilation setup"

# Create distribution directory
DIST_DIR="${PROJECT_ROOT}/dist/macos"
mkdir -p "${DIST_DIR}"

# Copy binaries
if [ -f "target/x86_64-apple-darwin/release/vhs-86" ]; then
    cp "target/x86_64-apple-darwin/release/vhs-86" "${DIST_DIR}/vhs-86-x86_64"
    strip "${DIST_DIR}/vhs-86-x86_64" 2>/dev/null || true
fi

if [ -f "target/aarch64-apple-darwin/release/vhs-86" ]; then
    cp "target/aarch64-apple-darwin/release/vhs-86" "${DIST_DIR}/vhs-86-aarch64"
    strip "${DIST_DIR}/vhs-86-aarch64" 2>/dev/null || true
fi

# Fallback to universal binary if single arch
if [ -f "target/release/vhs-86" ] && [ ! -f "${DIST_DIR}/vhs-86-x86_64" ]; then
    cp "target/release/vhs-86" "${DIST_DIR}/vhs-86"
    strip "${DIST_DIR}/vhs-86" 2>/dev/null || true
fi

# Generate man page if possible
if command -v pandoc >/dev/null 2>&1; then
    echo "Generating man page..."
    mkdir -p "${DIST_DIR}/man"
    pandoc -s -t man "${PROJECT_ROOT}/man/vhs-86.1.md" -o "${DIST_DIR}/man/vhs-86.1"
fi

# Create tarball
ARCHIVE_NAME="vhs-86-macos.tar.gz"
echo "Creating archive: ${ARCHIVE_NAME}..."
cd "${DIST_DIR}"
tar -czf "${PROJECT_ROOT}/dist/${ARCHIVE_NAME}" -- *

echo ""
echo "Build complete!"
echo "Output directory: ${DIST_DIR}"
echo "Archive: ${PROJECT_ROOT}/dist/${ARCHIVE_NAME}"
