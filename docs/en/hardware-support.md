# Rapoo Hardware & Transport Support Matrix

This document provides technical details on supported Rapoo devices, Vendor/Product IDs, connection transports, and features.

---

## 🖲️ Tested Devices

### 1. Rapoo MT760 Pro Mouse
- **Vendor ID (VID)**: `0x24AE` (ITON Corp. / Rapoo)
- **Product ID (PID)**:
  - `0x186A` (USB Cable / 2.4 GHz USB Dongle Receiver)
  - `0x4510` (Bluetooth 5.0 Mode)
- **Features**: 10 Buttons, Side Thumb Scroll Wheel, Multi-device switching (up to 4 devices), 800-4000 DPI optical sensor.

### 2. Rapoo E9050L Keyboard
- **Vendor ID (VID)**: `0x24AE`
- **Product ID (PID)**: `0x1831`
- **Features**: Isolated from mouse remapping logic to prevent input device collision.

---

## 📊 Feature Matrix by Transport

| Feature | USB Cable | 2.4 GHz USB Dongle | Bluetooth |
| :--- | :---: | :---: | :---: |
| **Device Detection** | ✅ Confirmed | ✅ Confirmed | ✅ Confirmed |
| **Button Remapping (`uinput`)** | ✅ Confirmed | ✅ Confirmed | ✅ Confirmed |
| **Battery Telemetry** | ⚠️ N/A (Bus Powered) | ✅ Confirmed (HID/UPower) | ✅ Confirmed (BlueZ D-Bus) |
| **Software Profiles** | ✅ Confirmed | ✅ Confirmed | ✅ Confirmed |
| **Polling Rate Config** | 🧪 Experimental | 🧪 Experimental | ⚠️ Kernel Managed (~90-133 Hz) |
| **DPI Adjustment** | 🧪 Experimental | 🧪 Experimental | 🧪 Experimental |

---

## 🔍 De-duplication Criteria

When the Rapoo MT760 Pro is connected via both USB Cable and Bluetooth simultaneously, OpenRapoo's aggregator isolates interfaces based on system connection type hierarchy (USB Cable > Dongle 2.4G > Bluetooth) to prevent duplicate device cards in the user interface.

