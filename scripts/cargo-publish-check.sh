#!/usr/bin/env bash
# VHS-86 Cargo Publish Check Script
# Validates the crate is ready for publishing to crates.io

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${PROJECT_ROOT}"

echo "═══════════════════════════════════════════════════════════════"
echo "  VHS-86 Cargo Publish Check"
echo "  Version: $(grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\(.*\)".*/\1/')"
echo "═══════════════════════════════════════════════════════════════"
echo ""

FAILED=0

# 1. Check version consistency
echo "[1/10] Checking version consistency..."
CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\(.*\)".*/\1/')
README_VERSION=$(grep -o 'version-[0-9]\+\.[0-9]\+\.[0-9]\+' README.md | head -1 | sed 's/version-//')
MAN_VERSION=$(grep 'VHS-86(1)' man/vhs-86.1.md | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' | head -1)

echo "  Cargo.toml:  ${CARGO_VERSION}"
echo "  README.md:   ${README_VERSION:-not found}"
echo "  Man page:    ${MAN_VERSION:-not found}"

if [ -n "${README_VERSION}" ] && [ "${CARGO_VERSION}" != "${README_VERSION}" ]; then
    echo "  ⚠ WARNING: README.md version mismatch"
    FAILED=1
fi

if [ -n "${MAN_VERSION}" ] && [ "${CARGO_VERSION}" != "${MAN_VERSION}" ]; then
    echo "  ⚠ WARNING: Man page version mismatch"
    FAILED=1
fi

echo ""

# 2. Check required metadata fields
echo "[2/10] Checking Cargo.toml metadata..."
REQUIRED_FIELDS=("name" "version" "description" "license" "repository" "keywords" "categories")
for field in "${REQUIRED_FIELDS[@]}"; do
    if grep -q "^${field}" Cargo.toml; then
        echo "  ✓ ${field}"
    else
        echo "  ✗ ${field} MISSING"
        FAILED=1
    fi
done
echo ""

# 3. Check for unresolved TODO/FIXME comments
echo "[3/10] Checking for unresolved TODO/FIXME..."
TODO_COUNT=$(grep -r "TODO\|FIXME\|XXX\|HACK" src/ --include="*.rs" | grep -v "// done\|// completed\|test_" | wc -l)
if [ "${TODO_COUNT}" -gt 0 ]; then
    echo "  ⚠ Found ${TODO_COUNT} TODO/FIXME comments:"
    grep -rn "TODO\|FIXME\|XXX\|HACK" src/ --include="*.rs" | grep -v "// done\|// completed\|test_" | head -5
    if [ "${TODO_COUNT}" -gt 5 ]; then
        echo "  ... and $((TODO_COUNT - 5)) more"
    fi
else
    echo "  ✓ No unresolved TODO/FIXME comments"
fi
echo ""

# 4. Check for debug prints
echo "[4/10] Checking for debug print statements..."
DEBUG_COUNT=$(grep -rn "println!\|eprintln!" src/ --include="*.rs" | grep -v "// \|test_\|#\[cfg(test)\]" | wc -l)
if [ "${DEBUG_COUNT}" -gt 0 ]; then
    echo "  ⚠ Found ${DEBUG_COUNT} println!/eprintln! statements (excluding tests):"
    grep -rn "println!\|eprintln!" src/ --include="*.rs" | grep -v "// \|test_\|#\[cfg(test)\]" | head -5
else
    echo "  ✓ No debug print statements in production code"
fi
echo ""

# 5. Run cargo fmt check
echo "[5/10] Checking code formatting..."
if cargo fmt -- --check 2>/dev/null; then
    echo "  ✓ Code is properly formatted"
else
    echo "  ✗ Code formatting issues found. Run 'cargo fmt' to fix."
    FAILED=1
fi
echo ""

# 6. Run clippy
echo "[6/10] Running clippy..."
if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -5; then
    echo "  ✓ Clippy checks passed"
else
    echo "  ✗ Clippy found issues"
    FAILED=1
fi
echo ""

# 7. Build release
echo "[7/10] Building release binary..."
if cargo build --release 2>&1 | tail -3; then
    echo "  ✓ Release build successful"
else
    echo "  ✗ Release build failed"
    FAILED=1
fi
echo ""

# 8. Run tests
echo "[8/10] Running tests..."
if cargo test --all-features 2>&1 | tail -5; then
    echo "  ✓ All tests passed"
else
    echo "  ✗ Tests failed"
    FAILED=1
fi
echo ""

# 9. Run doc tests
echo "[9/10] Running documentation tests..."
if cargo test --doc 2>&1 | tail -3; then
    echo "  ✓ Documentation tests passed"
else
    echo "  ✗ Documentation tests failed"
    FAILED=1
fi
echo ""

# 10. Dry-run cargo publish
echo "[10/10] Dry-run cargo publish..."
if cargo publish --dry-run 2>&1 | tail -5; then
    echo "  ✓ Publish dry-run successful"
else
    echo "  ✗ Publish dry-run failed"
    FAILED=1
fi
echo ""

# Summary
echo "═══════════════════════════════════════════════════════════════"
if [ ${FAILED} -eq 0 ]; then
    echo "  ✓ ALL CHECKS PASSED"
    echo "  Crate is ready for publishing!"
    echo ""
    echo "  Next steps:"
    echo "    1. Review CHANGELOG.md"
    echo "    2. Tag release: git tag v${CARGO_VERSION}"
    echo "    3. Push tag: git push origin v${CARGO_VERSION}"
    echo "    4. Publish: cargo publish"
    echo "═══════════════════════════════════════════════════════════════"
    exit 0
else
    echo "  ✗ SOME CHECKS FAILED"
    echo "  Please fix the issues above before publishing."
    echo "═══════════════════════════════════════════════════════════════"
    exit 1
fi
