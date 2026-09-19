#!/usr/bin/env bash
# Build script for OpenRapoo Debian/Ubuntu (.deb) Package

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

VERSION="${VERSION:-$(grep -m1 '^version = ' "$PROJECT_ROOT/Cargo.toml" | cut -d '"' -f2)}"
if [ -z "$VERSION" ]; then
    VERSION="0.1.0"
fi

ARCH="amd64"
PKG_NAME="openrapoo_${VERSION}_${ARCH}"
DEB_DIR="${PROJECT_ROOT}/target/deb/${PKG_NAME}"
DIST_DIR="${PROJECT_ROOT}/dist"

echo "======================================================"
echo "           Building OpenRapoo Debian Package          "
echo "           Version: ${VERSION}                        "
echo "======================================================"

cd "$PROJECT_ROOT"

# 1. Build release binaries
echo "[1/4] Compiling release binaries..."
cargo build --release --workspace

# 2. Clean and create package directory structure
echo "[2/4] Preparing package directory layout..."
rm -rf "${PROJECT_ROOT}/target/deb"
mkdir -p "${DEB_DIR}/usr/bin"
mkdir -p "${DEB_DIR}/usr/share/applications"
mkdir -p "${DEB_DIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${DEB_DIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${DEB_DIR}/usr/share/pixmaps"
mkdir -p "${DEB_DIR}/usr/share/openrapoo/assets"
mkdir -p "${DEB_DIR}/lib/udev/rules.d"
mkdir -p "${DEB_DIR}/usr/lib/systemd/user"
mkdir -p "${DEB_DIR}/etc/xdg/autostart"
mkdir -p "${DEB_DIR}/DEBIAN"

# 3. Copy binaries
cp target/release/openrapoo-gui "${DEB_DIR}/usr/bin/"
cp target/release/openrapoo-daemon "${DEB_DIR}/usr/bin/"
cp target/release/openrapoo-diag "${DEB_DIR}/usr/bin/"
chmod +x "${DEB_DIR}/usr/bin/"*

# 4. Copy desktop entries
cp packaging/io.github.openrapoo.OpenRapoo.desktop "${DEB_DIR}/usr/share/applications/"
cp packaging/openrapoo-gui.desktop "${DEB_DIR}/usr/share/applications/"

# 5. Copy icons and assets
if [ -f packaging/icons/io.github.openrapoo.OpenRapoo.svg ]; then
    cp packaging/icons/io.github.openrapoo.OpenRapoo.svg "${DEB_DIR}/usr/share/icons/hicolor/scalable/apps/"
fi
if [ -f packaging/icons/icon.png ]; then
    cp packaging/icons/icon.png "${DEB_DIR}/usr/share/icons/hicolor/256x256/apps/openrapoo-gui.png"
    cp packaging/icons/icon.png "${DEB_DIR}/usr/share/icons/hicolor/256x256/apps/io.github.openrapoo.OpenRapoo.png"
    cp packaging/icons/icon.png "${DEB_DIR}/usr/share/pixmaps/openrapoo-gui.png"
fi
if [ -d crates/openrapoo-gui/assets ]; then
    cp -r crates/openrapoo-gui/assets/* "${DEB_DIR}/usr/share/openrapoo/assets/"
fi

# 6. Copy udev rules, systemd user service, and autostart
if [ -f udev/99-openrapoo.rules ]; then
    cp udev/99-openrapoo.rules "${DEB_DIR}/lib/udev/rules.d/"
fi

cat << 'EOF' > "${DEB_DIR}/usr/lib/systemd/user/openrapoo-daemon.service"
[Unit]
Description=OpenRapoo Mouse Remapping Daemon
Documentation=https://github.com/openrapoo/openrapoo
After=graphical-session.target
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/bin/openrapoo-daemon
Restart=on-failure
RestartSec=5s

# Security hardening
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=%h/.config/openrapoo %h/.local/share/openrapoo

# Allow access to input devices
SupplementaryGroups=input
DeviceAllow=/dev/input/event* rw
DeviceAllow=/dev/hidraw* rw
DeviceAllow=/dev/uinput rw

[Install]
WantedBy=graphical-session.target
EOF

if [ -f autostart/openrapoo-autostart.desktop ]; then
    cp autostart/openrapoo-autostart.desktop "${DEB_DIR}/etc/xdg/autostart/"
fi

# 7. Generate DEBIAN/control
cat << EOF > "${DEB_DIR}/DEBIAN/control"
Package: openrapoo
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: OpenRapoo Contributors <https://github.com/openrapoo/openrapoo>
Depends: libc6, udev
Description: Linux configuration tool & daemon for Rapoo MT760 Pro mouse
 OpenRapoo is a native Linux configuration suite for Rapoo mice (MT760 Pro,
 MT760 NearLink, E9050L keyboard). It provides DPI customization, polling rate
 selection, button remapping via evdev/uinput, and battery monitoring via
 UPower and BlueZ D-Bus.
EOF

# 8. Generate DEBIAN/postinst
cat << 'EOF' > "${DEB_DIR}/DEBIAN/postinst"
#!/bin/sh
set -e

if [ "$1" = "configure" ]; then
    if command -v udevadm >/dev/null 2>&1; then
        udevadm control --reload-rules || true
        udevadm trigger --subsystem-match=hidraw --subsystem-match=input || true
    fi
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications || true
    fi
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
    fi
fi
EOF
chmod +x "${DEB_DIR}/DEBIAN/postinst"

# 9. Generate DEBIAN/postrm
cat << 'EOF' > "${DEB_DIR}/DEBIAN/postrm"
#!/bin/sh
set -e

if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    if command -v udevadm >/dev/null 2>&1; then
        udevadm control --reload-rules || true
    fi
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications || true
    fi
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
    fi
fi
EOF
chmod +x "${DEB_DIR}/DEBIAN/postrm"

# 10. Fix permissions for Debian package requirements
chmod -R 0755 "${DEB_DIR}"

# 11. Build .deb package
echo "[3/4] Building .deb package with dpkg-deb..."
mkdir -p "$DIST_DIR"
dpkg-deb --build --root-owner-group "${DEB_DIR}" "${DIST_DIR}/${PKG_NAME}.deb"

echo "[4/4] Package built successfully!"
echo "  Location: ${DIST_DIR}/${PKG_NAME}.deb"
