# OpenRapoo Installation & Deployment Guide

This document details all available installation methods for OpenRapoo, including Debian `.deb` packages, standalone `AppImage`, source compilation, and user service setup.

---

## 📦 Package Installation Options

### 1. Debian / Ubuntu (.deb)

The recommended installation method for Debian-based distributions (Ubuntu 22.04 / 24.04, Debian 12, Linux Mint, Pop!_OS):

```bash
# Download latest release
wget https://github.com/openrapoo/openrapoo/releases/download/v0.1.0/openrapoo_0.1.0_amd64.deb

# Install package
sudo dpkg -i openrapoo_0.1.0_amd64.deb

# Resolve missing dependencies if necessary
sudo apt install -f
```

The package automatically installs binaries to `/usr/bin/`, desktop entries to `/usr/share/applications/`, UDev rules to `/lib/udev/rules.d/99-openrapoo.rules`, and systemd user unit to `/usr/lib/systemd/user/openrapoo-daemon.service`.

### 2. Standalone AppImage

Suitable for any Linux distribution:

```bash
# Download AppImage
wget https://github.com/openrapoo/openrapoo/releases/download/v0.1.0/OpenRapoo-0.1.0-x86_64.AppImage

# Grant execution permission
chmod +x OpenRapoo-0.1.0-x86_64.AppImage

# Launch application
./OpenRapoo-0.1.0-x86_64.AppImage
```

---

## 🔑 Setting up UDev Permissions

To ensure OpenRapoo can detect and control mouse hardware without `sudo`:

```bash
# Add user to input group
sudo usermod -aG input $USER

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Log out and log back in to apply group changes.

