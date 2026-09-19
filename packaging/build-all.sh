#!/usr/bin/env bash
# Master packaging script for OpenRapoo
# Builds: Release Binaries Tarball, Debian Package (.deb), and AppImage (.AppImage)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Extract single source of truth version from root Cargo.toml
VERSION=$(grep -m1 '^version = ' "$PROJECT_ROOT/Cargo.toml" | cut -d '"' -f2)
if [ -z "$VERSION" ]; then
    VERSION="0.1.0"
fi

DIST_DIR="${PROJECT_ROOT}/dist"

echo "======================================================"
echo "          OpenRapoo Complete Build Suite              "
echo "          Version: ${VERSION}                        "
echo "======================================================"
echo

cd "$PROJECT_ROOT"
mkdir -p "$DIST_DIR"

# 1. Compile release binaries
echo "[1/4] Compiling release binaries..."
cargo build --release --workspace

# 2. Package release tarball
echo "[2/4] Packaging release binaries tarball..."
TARBALL_NAME="openrapoo-${VERSION}-x86_64.tar.gz"
TARBALL_STAGING="${PROJECT_ROOT}/target/tarball_staging"
rm -rf "$TARBALL_STAGING"
mkdir -p "${TARBALL_STAGING}/bin"
mkdir -p "${TARBALL_STAGING}/scripts"
mkdir -p "${TARBALL_STAGING}/udev"
mkdir -p "${TARBALL_STAGING}/systemd"
mkdir -p "${TARBALL_STAGING}/autostart"
mkdir -p "${TARBALL_STAGING}/packaging"

cp target/release/openrapoo-gui "${TARBALL_STAGING}/bin/"
cp target/release/openrapoo-daemon "${TARBALL_STAGING}/bin/"
cp target/release/openrapoo-diag "${TARBALL_STAGING}/bin/"
cp scripts/* "${TARBALL_STAGING}/scripts/" 2>/dev/null || true
cp udev/* "${TARBALL_STAGING}/udev/" 2>/dev/null || true
cp systemd/* "${TARBALL_STAGING}/systemd/" 2>/dev/null || true
cp autostart/* "${TARBALL_STAGING}/autostart/" 2>/dev/null || true
cp packaging/io.github.openrapoo.OpenRapoo.desktop "${TARBALL_STAGING}/packaging/"
cp packaging/openrapoo-gui.desktop "${TARBALL_STAGING}/packaging/"
cp packaging/icons/icon.png "${TARBALL_STAGING}/" 2>/dev/null || true
cp README.md "${TARBALL_STAGING}/" 2>/dev/null || true

tar -czf "${DIST_DIR}/${TARBALL_NAME}" -C "$TARBALL_STAGING" .
echo "  ✓ Created ${DIST_DIR}/${TARBALL_NAME}"

# 3. Build .deb package
echo "[3/4] Building Debian/Ubuntu (.deb) package..."
"${PROJECT_ROOT}/packaging/deb/build-deb.sh"
echo "  ✓ Created .deb package in ${DIST_DIR}/"

# 4. Build AppImage
echo "[4/4] Building AppImage..."
"${PROJECT_ROOT}/packaging/appimage/build-appimage.sh" || true

echo
echo "======================================================"
echo "          Build completed successfully!              "
echo "  Artifacts generated in dist/:                       "
ls -lh "$DIST_DIR"
echo "======================================================"
echo
