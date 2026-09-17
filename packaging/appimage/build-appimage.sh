#!/usr/bin/env bash
# Build script for OpenRapoo AppImage

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/target/appdir"

echo "======================================================"
echo "           Building OpenRapoo AppImage                "
echo "======================================================"

cd "$PROJECT_ROOT"
cargo build --release --workspace

rm -rf "$BUILD_DIR"
mkdir -p "${BUILD_DIR}/usr/bin"
mkdir -p "${BUILD_DIR}/usr/share/applications"
mkdir -p "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps"

cp target/release/openrapoo-gui "${BUILD_DIR}/usr/bin/"
cp target/release/openrapoo-daemon "${BUILD_DIR}/usr/bin/"
cp target/release/openrapoo-diag "${BUILD_DIR}/usr/bin/"

cp packaging/io.github.openrapoo.OpenRapoo.desktop "${BUILD_DIR}/usr/share/applications/"
cp packaging/io.github.openrapoo.OpenRapoo.desktop "${BUILD_DIR}/"
cp packaging/icons/io.github.openrapoo.OpenRapoo.svg "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps/"
cp packaging/icons/io.github.openrapoo.OpenRapoo.svg "${BUILD_DIR}/io.github.openrapoo.OpenRapoo.svg"
cp packaging/appimage/AppRun "${BUILD_DIR}/AppRun"

chmod +x "${BUILD_DIR}/AppRun"
chmod +x "${BUILD_DIR}/usr/bin/"*

echo "AppDir structure prepared at: ${BUILD_DIR}"
echo "To generate the final AppImage file using appimagetool:"
echo "  appimagetool ${BUILD_DIR} OpenRapoo-x86_64.AppImage"
