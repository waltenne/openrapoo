#!/usr/bin/env bash
# Helper script to install OpenRapoo desktop entry and icon for Linux desktops

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

ICON_SRC="$ROOT_DIR/icon.png"
DESKTOP_SRC="$ROOT_DIR/packaging/io.github.openrapoo.OpenRapoo.desktop"

LOCAL_APPS="$HOME/.local/share/applications"
LOCAL_ICONS="$HOME/.local/share/icons/hicolor/256x256/apps"
LOCAL_PIXMAPS="$HOME/.local/share/pixmaps"
LOCAL_OPENRAPOO_ASSETS="$HOME/.local/share/openrapoo/assets"

mkdir -p "$LOCAL_APPS" "$LOCAL_ICONS" "$LOCAL_PIXMAPS" "$LOCAL_OPENRAPOO_ASSETS"

if [ -f "$ICON_SRC" ]; then
    for icon_name in "io.github.openrapoo.OpenRapoo.png" "openrapoo-gui.png" "openrapoo_gui.png" "openrapoo.png" "icon.png"; do
        cp "$ICON_SRC" "$LOCAL_ICONS/$icon_name"
        cp "$ICON_SRC" "$LOCAL_PIXMAPS/$icon_name"
    done
    echo "Installed icon.png aliases to $LOCAL_ICONS and $LOCAL_PIXMAPS"
fi

GUI_ASSETS_DIR="$ROOT_DIR/crates/openrapoo-gui/assets"
if [ -d "$GUI_ASSETS_DIR" ]; then
    cp -r "$GUI_ASSETS_DIR"/* "$LOCAL_OPENRAPOO_ASSETS/" 2>/dev/null || true
    echo "Installed GUI assets to $LOCAL_OPENRAPOO_ASSETS"
fi

if [ -f "$DESKTOP_SRC" ]; then
    for desktop_name in "io.github.openrapoo.OpenRapoo.desktop" "openrapoo-gui.desktop" "openrapoo_gui.desktop" "openrapoo.desktop"; do
        cp "$DESKTOP_SRC" "$LOCAL_APPS/$desktop_name"
    done
    echo "Installed desktop files to $LOCAL_APPS"
fi

update-desktop-database "$LOCAL_APPS" 2>/dev/null || true
gtk-update-icon-cache "$HOME/.local/share/icons/hicolor" 2>/dev/null || true

echo "Desktop integration updated successfully!"

