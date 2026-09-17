#!/usr/bin/env bash
# OpenRapoo Service and Autostart Installer Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="$HOME/.local/bin"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"
AUTOSTART_DIR="$HOME/.config/autostart"

echo "======================================================"
echo "           OpenRapoo Service Installer               "
echo "======================================================"
echo

# 1. Ensure binaries are compiled
echo "[1/6] Building release binaries..."
cd "$SCRIPT_DIR"
cargo build --release

# 2. Copy binaries to ~/.local/bin
echo "[2/6] Installing binaries to $BIN_DIR..."
mkdir -p "$BIN_DIR"
cp -f target/release/openrapoo-daemon "$BIN_DIR/"
cp -f target/release/openrapoo-diag "$BIN_DIR/"
cp -f target/release/openrapoo-gui "$BIN_DIR/"
chmod +x "$BIN_DIR/openrapoo-daemon" "$BIN_DIR/openrapoo-diag" "$BIN_DIR/openrapoo-gui"

# 3. Install udev rules
echo "[3/6] Installing udev rules..."
if [ -f udev/99-openrapoo.rules ]; then
    sudo cp -f udev/99-openrapoo.rules /etc/udev/rules.d/
    sudo udevadm control --reload-rules
    sudo udevadm trigger
    echo "  ✓ udev rules installed and reloaded."
fi

# 4. Add user to input group
echo "[4/6] Checking 'input' group membership..."
if ! groups "$USER" | grep -q "\binput\b"; then
    echo "  Adding user $USER to group 'input'..."
    sudo usermod -aG input "$USER"
    echo "  ⚠ Note: Please log out and back in for group membership to take effect."
else
    echo "  ✓ User $USER is already in group 'input'."
fi

# 5. Install systemd user service
echo "[5/6] Installing systemd user service..."
mkdir -p "$SYSTEMD_USER_DIR"
cp -f systemd/openrapoo-daemon.service "$SYSTEMD_USER_DIR/"
systemctl --user daemon-reload || true
systemctl --user enable openrapoo-daemon.service || true
echo "  ✓ Systemd user service installed and enabled."

# 6. Install XDG autostart entry
echo "[6/6] Installing XDG autostart entry..."
mkdir -p "$AUTOSTART_DIR"
cp -f autostart/openrapoo-autostart.desktop "$AUTOSTART_DIR/"
echo "  ✓ XDG autostart entry installed."

echo
echo "======================================================"
echo "   Installation completed successfully!              "
echo "   To start the service now, run:                   "
echo "     systemctl --user start openrapoo-daemon         "
echo "======================================================"
echo
