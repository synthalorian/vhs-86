#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== VHS-86 Cross-Platform Build ==="
echo ""

# Detect OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     PLATFORM=linux;;
    Darwin*)    PLATFORM=macos;;
    CYGWIN*|MINGW*|MSYS*) PLATFORM=windows;;
    *)          PLATFORM=unknown;;
esac

echo "Detected platform: ${PLATFORM}"
echo ""

# Run platform-specific build
if [ -f "${SCRIPT_DIR}/build-${PLATFORM}.sh" ]; then
    "${SCRIPT_DIR}/build-${PLATFORM}.sh"
else
    echo "No build script for platform: ${PLATFORM}"
    echo "Available build scripts:"
    ls -1 "${SCRIPT_DIR}"/build-*.sh | sed 's/.*\//  /'
    exit 1
fi
