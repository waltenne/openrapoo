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

Inspired by [OpenLogi](https://github.com/AprilNEA/OpenLogi).

---

## 🎯 Problem OpenRapoo Solves

Rapoo does not provide official software for Linux operating systems. Users connecting Rapoo mice (such as the MT760 Pro) to Linux face several issues:
1. Inability to remap extra side buttons or thumb scroll wheels.
2. Inability to adjust hardware DPI sensitivity levels or USB polling rates.
3. Lack of reliable battery status monitoring across USB, 2.4 GHz Dongle, and Bluetooth connections.
4. Reliance on proprietary Windows executables.

OpenRapoo bridges this gap by providing a native, lightweight, non-root Linux GUI and user daemon that interfaces cleanly with `evdev`, `uinput`, `hidraw`, `UPower`, and `BlueZ` D-Bus APIs.

---

## 🔍 Current Project Status (v0.1.0)

OpenRapoo is currently at **Version 0.1.0** with a functional GPUI desktop application, background uinput daemon, multi-provider battery detection, and native Linux packaging (`.deb` and `AppImage`).

### Tested Hardware
- **Rapoo MT760 Pro Mouse**: Vendor ID `0x24AE`, Product ID `0x186A` (USB/Dongle) / `0x4510` (Bluetooth).
- **Rapoo E9050L Keyboard**: Recognized and handled without device collision.

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

```bash
sudo dpkg -i openrapoo_0.1.0_amd64.deb
sudo apt install -f
```

### Standalone AppImage

Download the AppImage from [GitHub Releases](https://github.com/openrapoo/openrapoo/releases):

```bash
chmod +x OpenRapoo-0.1.0-x86_64.AppImage
./OpenRapoo-0.1.0-x86_64.AppImage
```

### Building from Source

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt install build-essential pkg-config libhidapi-dev libudev-dev libevdev-dev libdbus-1-dev desktop-file-utils

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

OpenRapoo is developed with the assistance of artificial intelligence tools. AI is used as a supporting tool for research, implementation, architecture, code generation and review, testing, troubleshooting, and documentation. Technical decisions, code reviews, hardware validations, and ultimate code responsibility remain with the project maintainers.

For details, read [AI-Assisted Development Guide](docs/en/ai-assisted-development.md).

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
