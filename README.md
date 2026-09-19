# OpenRapoo

<p align="center">
  <strong>Native Linux Configuration Suite & Daemon for Rapoo MT760 Pro & Rapoo Devices</strong>
</p>

<p align="center">
  <a href="README.md"><strong>English</strong></a> |
  <a href="README.pt-BR.md"><strong>Português do Brasil</strong></a>
</p>

<p align="center">
  <img src="assets/screenshots/openrapoo-gui.png" alt="OpenRapoo GPUI Interface" width="800" />
</p>

---

OpenRapoo is an open-source Linux software suite developed in **Rust** with **GPUI** to provide full feature support, DPI customization, polling rate adjustment, button remapping, and battery telemetry for Rapoo mice and keyboards—specifically the **Rapoo MT760 Pro**.

This project was developed using **Google Antigravity** to solve the complete absence of official Linux configuration software from the manufacturer.

Inspired by [OpenLogi](https://github.com/AprilNEA/OpenLogi).

---

## 🎯 Problem OpenRapoo Solves
> [!WARNING]
> **Active Development & Potential Bugs Disclaimer**:
> OpenRapoo is currently at **Version 0.1.0** and under active open-source development. Because it was created to bridge the lack of official vendor software on Linux, **the software may contain bugs** or unexpected behavior depending on your Linux distribution, desktop environment (Wayland/X11), or kernel version. If you find any bug or issue, please [report it on GitHub Issues](https://github.com/waltenne/openrapoo/issues)!

Rapoo does not provide official software for Linux operating systems. Users connecting Rapoo mice (such as the MT760 Pro) to Linux face several issues:
---

## 🎯 Why OpenRapoo Was Created

Rapoo **does not provide official configuration software for Linux operating systems**. Users connecting Rapoo mice (such as the MT760 Pro) to Linux face several issues:
1. Inability to remap extra side buttons or thumb scroll wheels.
2. Inability to adjust hardware DPI sensitivity levels or USB polling rates.
3. Lack of reliable battery status monitoring across USB, 2.4 GHz Dongle, and Bluetooth connections.
4. Reliance on proprietary Windows executables.
4. Total reliance on proprietary Windows-only `.exe` executables.

OpenRapoo bridges this gap by providing a native, lightweight, non-root Linux GUI and user daemon that interfaces cleanly with `evdev`, `uinput`, `hidraw`, `UPower`, and `BlueZ` D-Bus APIs.

---

## 🔍 Current Project Status (v0.1.0)
## 💻 Compatibility Matrix

OpenRapoo is currently at **Version 0.1.0** with a functional GPUI desktop application, background uinput daemon, multi-provider battery detection, and native Linux packaging (`.deb` and `AppImage`).
### Supported Hardware
- **Rapoo MT760 Pro Mouse**: Vendor ID `0x24AE`, Product ID `0x186A` (USB/2.4GHz Dongle) & `0x4510` (Bluetooth).
- **Rapoo E9050L Keyboard**: Vendor ID `0x24AE`, Product ID `0x1008`.
- **Other Rapoo Wireless/HID Devices**: Multi-device identification with collision isolation.

### Tested Hardware
- **Rapoo MT760 Pro Mouse**: Vendor ID `0x24AE`, Product ID `0x186A` (USB/Dongle) / `0x4510` (Bluetooth).
- **Rapoo E9050L Keyboard**: Recognized and handled without device collision.
### Supported Linux Distributions & Systems
- **Debian / Ubuntu** (Ubuntu 22.04 LTS, 24.04 LTS, Linux Mint, Pop!_OS)
- **Arch Linux / Manjaro** (Kernel 5.x / 6.x)
- **Fedora / RHEL** (with `evdev` & `uinput` kernel modules)
- **openSUSE & General Linux Distros** supporting `udev`, `evdev`, `uinput`, `UPower`, and `BlueZ`.
- **Architectures**: `x86_64` (`amd64`).

### Tested Operating Systems
- Ubuntu 22.04 LTS (Jammy Jellyfish)
- Ubuntu 24.04 LTS (Noble Numbat)
- Arch Linux / Manjaro (Linux Kernel 6.x)

---

## 🚦 Feature Availability Matrix

| Feature | Available | Experimental | Planned |
| :--- | :---: | :---: | :---: |
| **Device Detection (USB / Dongle / BT)** | ✅ | | |
| **GPUI Graphical Interface** | ✅ | | |
| **Button Remapping via uinput Daemon** | ✅ | | |
| **Profile Storage & Persistence** | ✅ | | |
| **Bilingual i18n (English & Portuguese)** | ✅ | | |
| **Battery Monitoring (UPower / BlueZ / HID)** | ✅ | | |
| **.deb & AppImage Packaging** | ✅ | | |
| **Direct HID 0x07 Report Hardware Query** | | 🧪 | |
| **Bluetooth GATT 0x180F Battery Probe** | | 🧪 | |
| **Dongle Receiver Idle Sub-interface Filter** | | 🧪 | |
| **EEPROM Hardware Onboard Flashing** | | | 📋 |
| **Proprietary Vendor HID Protocol Reverse Engineering** | | | 📋 |
| **Firmware Flashing Tool** | | | 📋 |

---

## 📊 Hardware & Transport Matrix

| Transport | Connection Detection | Button Remapping | Battery Telemetry | Hardware DPI / Polling Rate |
| :--- | :---: | :---: | :---: | :---: |
| **USB Cable** | ✅ Confirmed | ✅ Confirmed | ⚠️ N/A (Bus Powered) | 🧪 Experimental (HID Report) |
| **2.4 GHz USB Dongle** | ✅ Confirmed | ✅ Confirmed | ✅ Confirmed (HID/UPower) | 🧪 Experimental (HID Report) |
| **Bluetooth** | ✅ Confirmed | ✅ Confirmed | ✅ Confirmed (BlueZ D-Bus) | ⚠️ Kernel Managed (~90-133 Hz) |

---

## 📦 Installation

### Debian / Ubuntu (.deb)

Download the `.deb` package from [GitHub Releases](https://github.com/openrapoo/openrapoo/releases):
Download the `.deb` package from [GitHub Releases](https://github.com/waltenne/openrapoo/releases):

```bash
sudo dpkg -i openrapoo_0.1.0_amd64.deb
sudo apt install -f
```

### Standalone AppImage

Download the AppImage from [GitHub Releases](https://github.com/openrapoo/openrapoo/releases):
Download the AppImage from [GitHub Releases](https://github.com/waltenne/openrapoo/releases):

```bash
chmod +x OpenRapoo-0.1.0-x86_64.AppImage
./OpenRapoo-0.1.0-x86_64.AppImage
```

### Building from Source

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt install build-essential pkg-config libhidapi-dev libudev-dev libevdev-dev libdbus-1-dev desktop-file-utils
sudo apt install build-essential pkg-config libhidapi-dev libudev-dev libevdev-dev libdbus-1-dev desktop-file-utils libx11-dev libx11-xcb-dev libxcb-xfixes0-dev libxcb-shape0-dev libxcb-render0-dev libxcb1-dev libfontconfig1-dev libxkbcommon-dev libxkbcommon-x11-dev libgl1-mesa-dev

# Compile workspace
cargo build --release --workspace

# Run master packaging suite
./packaging/build-all.sh
```

---

## 🔑 Permissions & Security

OpenRapoo runs the GUI as a standard user without requiring `sudo`. Hardware access to `/dev/hidraw*` and `/dev/input/event*` is granted cleanly via UDev rules and membership in the `input` group.

```bash
# Add user to input group
sudo usermod -aG input $USER

# Install UDev rule
sudo cp udev/99-openrapoo.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

*Note: Log out and log back in for group changes to take effect.*

---

## 🤖 AI-Assisted Development Transparency
## 🤖 Development Disclosure (Google Antigravity & AI)

OpenRapoo is developed with the assistance of artificial intelligence tools. AI is used as a supporting tool for research, implementation, architecture, code generation and review, testing, troubleshooting, and documentation. Technical decisions, code reviews, hardware validations, and ultimate code responsibility remain with the project maintainers.
OpenRapoo was developed with the assistance of **Google Antigravity** and AI pairing tools. AI was used for research lookups, architecture design, code generation, testing automation, internationalization, and documentation. Technical decisions, code reviews, physical hardware testing, and final validation are the responsibility of the project maintainer.

For details, read [AI-Assisted Development Guide](docs/en/ai-assisted-development.md).
For full disclosure, read the [AI-Assisted Development Guide](docs/en/ai-assisted-development.md).

---

## 📚 Documentation Index

- [Architecture Guide](docs/en/architecture.md)
- [Installation Guide](docs/en/installation.md)
- [Development Guide](docs/en/development.md)
- [Troubleshooting Guide](docs/en/troubleshooting.md)
- [Hardware Support Matrix](docs/en/hardware-support.md)
- [AI-Assisted Development Disclosure](docs/en/ai-assisted-development.md)

---

## 📜 License

Distributed under the **GPL-3.0-or-later** license.

