#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=== VHS-86 Linux Build Script ==="
echo "Building for x86_64-unknown-linux-gnu..."

cd "${PROJECT_ROOT}"

# Check dependencies
echo "Checking dependencies..."
command -v cargo >/dev/null 2>&1 || { echo "cargo is required but not installed."; exit 1; }

# Install dependencies if needed
if command -v apt-get >/dev/null 2>&1; then
    echo "Installing build dependencies..."
    sudo apt-get update
    sudo apt-get install -y libssl-dev pkg-config libgit2-dev
elif command -v dnf >/dev/null 2>&1; then
    echo "Installing build dependencies..."
    sudo dnf install -y openssl-devel pkgconfig libgit2-devel
elif command -v pacman >/dev/null 2>&1; then
    echo "Installing build dependencies..."
    sudo pacman -S --needed openssl pkgconf libgit2
fi

# Build release binary
echo "Building release binary..."
cargo build --release

# Strip binary
if command -v strip >/dev/null 2>&1; then
    echo "Stripping binary..."
    strip target/release/vhs-86
fi

# Create distribution directory
DIST_DIR="${PROJECT_ROOT}/dist/linux"
mkdir -p "${DIST_DIR}"

# Copy binary
cp target/release/vhs-86 "${DIST_DIR}/"

# Generate man page if possible
if command -v pandoc >/dev/null 2>&1; then
    echo "Generating man page..."
    mkdir -p "${DIST_DIR}/man"
    pandoc -s -t man "${PROJECT_ROOT}/man/vhs-86.1.md" -o "${DIST_DIR}/man/vhs-86.1"
fi

# Create tarball
ARCHIVE_NAME="vhs-86-linux-x86_64.tar.gz"
echo "Creating archive: ${ARCHIVE_NAME}..."
cd "${DIST_DIR}"
tar -czf "${PROJECT_ROOT}/dist/${ARCHIVE_NAME}" vhs-86

echo ""
echo "Build complete!"
echo "Binary: ${DIST_DIR}/vhs-86"
echo "Archive: ${PROJECT_ROOT}/dist/${ARCHIVE_NAME}"
