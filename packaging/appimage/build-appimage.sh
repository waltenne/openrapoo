#!/usr/bin/env bash
# Build script for OpenRapoo AppImage

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/target/appdir"
DIST_DIR="${PROJECT_ROOT}/dist"

VERSION="${VERSION:-$(grep -m1 '^version = ' "$PROJECT_ROOT/Cargo.toml" | cut -d '"' -f2)}"
if [ -z "$VERSION" ]; then
    VERSION="0.1.0"
fi

APPIMAGE_NAME="OpenRapoo-${VERSION}-x86_64.AppImage"

echo "======================================================"
echo "           Building OpenRapoo AppImage                "
echo "           Version: ${VERSION}                        "
echo "======================================================"

cd "$PROJECT_ROOT"

# 1. Build release binaries
echo "[1/4] Compiling release binaries..."
cargo build --release --workspace

# 2. Prepare AppDir directory structure
echo "[2/4] Preparing AppDir structure..."
rm -rf "$BUILD_DIR"
mkdir -p "${BUILD_DIR}/usr/bin"
mkdir -p "${BUILD_DIR}/usr/share/applications"
mkdir -p "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${BUILD_DIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${BUILD_DIR}/usr/share/openrapoo/assets"
mkdir -p "$DIST_DIR"

cp target/release/openrapoo-gui "${BUILD_DIR}/usr/bin/"
cp target/release/openrapoo-daemon "${BUILD_DIR}/usr/bin/"
cp target/release/openrapoo-diag "${BUILD_DIR}/usr/bin/"

cp packaging/io.github.openrapoo.OpenRapoo.desktop "${BUILD_DIR}/usr/share/applications/"
cp packaging/io.github.openrapoo.OpenRapoo.desktop "${BUILD_DIR}/"

if [ -f packaging/icons/io.github.openrapoo.OpenRapoo.svg ]; then
    cp packaging/icons/io.github.openrapoo.OpenRapoo.svg "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps/"
    cp packaging/icons/io.github.openrapoo.OpenRapoo.svg "${BUILD_DIR}/io.github.openrapoo.OpenRapoo.svg"
fi

if [ -f packaging/icons/icon.png ]; then
    cp packaging/icons/icon.png "${BUILD_DIR}/usr/share/icons/hicolor/256x256/apps/openrapoo-gui.png"
    cp packaging/icons/icon.png "${BUILD_DIR}/openrapoo-gui.png"
    cp packaging/icons/icon.png "${BUILD_DIR}/.DirIcon"
fi

if [ -d crates/openrapoo-gui/assets ]; then
    cp -r crates/openrapoo-gui/assets/* "${BUILD_DIR}/usr/share/openrapoo/assets/"
fi

cp packaging/appimage/AppRun "${BUILD_DIR}/AppRun"

chmod +x "${BUILD_DIR}/AppRun"
chmod +x "${BUILD_DIR}/usr/bin/"*

echo "[3/4] AppDir structure ready at: ${BUILD_DIR}"

# 3. Download appimagetool if not available in PATH
echo "[4/4] Generating AppImage file..."
APPIMAGETOOL=""

if command -v appimagetool >/dev/null 2>&1; then
    APPIMAGETOOL="appimagetool"
elif [ -f "${PROJECT_ROOT}/target/appimagetool-x86_64.AppImage" ]; then
    APPIMAGETOOL="${PROJECT_ROOT}/target/appimagetool-x86_64.AppImage"
else
    echo "  Downloading appimagetool..."
    TOOL_URL="https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"
    curl -sSL -o "${PROJECT_ROOT}/target/appimagetool-x86_64.AppImage" "$TOOL_URL" || true
    if [ -f "${PROJECT_ROOT}/target/appimagetool-x86_64.AppImage" ]; then
        chmod +x "${PROJECT_ROOT}/target/appimagetool-x86_64.AppImage"
        APPIMAGETOOL="${PROJECT_ROOT}/target/appimagetool-x86_64.AppImage"
    fi
fi

if [ -n "$APPIMAGETOOL" ] && [ -x "$APPIMAGETOOL" ]; then
    ARCH=x86_64 "$APPIMAGETOOL" "$BUILD_DIR" "${DIST_DIR}/${APPIMAGE_NAME}" || {
        # If running inside sandbox/container without FUSE
        ARCH=x86_64 "$APPIMAGETOOL" --appimage-extract-and-run "$BUILD_DIR" "${DIST_DIR}/${APPIMAGE_NAME}" || true
    }
fi

if [ -f "${DIST_DIR}/${APPIMAGE_NAME}" ]; then
    echo "AppImage built successfully at: ${DIST_DIR}/${APPIMAGE_NAME}"
else
    echo "AppDir prepared. Run appimagetool manually:"
    echo "  appimagetool ${BUILD_DIR} ${DIST_DIR}/${APPIMAGE_NAME}"
fi
