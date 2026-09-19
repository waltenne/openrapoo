# OpenRapoo Architectural Opportunities & Implementation Roadmap

## Architectural Comparison Matrix

| Feature Domain | Official Rapoo Windows Software ("A HUB") | OpenRapoo Linux Architecture | Improvement Opportunity for OpenRapoo |
| :--- | :--- | :--- | :--- |
| **GUI Framework** | Qt 5.15.2 / QML (Heavy, 330 MB binary footprint) | GPUI (Rust Native, Lightweight, GPU-accelerated) | Fast startup, low CPU/RAM usage (~15MB RAM vs 300MB+) |
| **Daemon & Subsystem** | Embedded C++ DLL loaded synchronously inside GUI process | Decoupled System Daemon (`openrapoo-daemon`) via IPC / D-Bus | Prevents GUI freezes, isolates udev permissions, stable background service |
| **Multi-Device Handling** | Single active device focus in QML, background polling | Async Rust `tokio` multi-device engine monitoring `/dev/hidraw*` | Parallel device status tracking (MT760 Pro + E9050L Keyboard concurrently) |
| **Device Filtering** | Rigid PID lookup list in `QueryDevice` | Dynamic udev rules + vendor VID `0x24AE` discovery | Auto-detects new Rapoo receivers and Bluetooth LE peripherals automatically |
| **Keyboard Support** | Hidden / Minimal support for non-programmable keyboards | Explicit diagnostic-only view for **E9050L** (Image asset, telemetry, diagnostic log) | Clear UI distinction: Mice show full config; Keyboards show device info & battery |

---

## Technical Recommendations for Current OpenRapoo Issues

### 1. Resolving GUI Freeze & Multi-Device Contention
- **Problem**: When multiple Rapoo devices (e.g. MT760 Pro mouse + E9050L keyboard) are connected, the OS experiences lag and the GUI freezes.
- **Root Cause**: Both GUI and daemon attempt to scan and grab `/dev/input/event*` and `/dev/hidraw*` simultaneously, causing `EBUSY (os error 16)` locks on dual-interface nodes (like PID `0x1008`).
- **Solution Identified**:
  - `openrapoo-daemon` MUST be the sole owner of `/dev/hidraw*` device descriptors.
  - `openrapoo-gui` must NEVER open `/dev/hidraw*` directly when daemon is active; it must query status over Unix Domain Socket / IPC.
  - Non-remappable devices (like the E9050L keyboard) must NOT be grabbed in exclusive mode (`evdev::GrabMode::Exclusive`) because grabbing the keyboard node blocks system input typing!

### 2. Device View Customization (Rapoo E9050L Keyboard)
- As confirmed by reverse-engineering, Rapoo keyboards do not support software DPI or button remapping.
- **UI Policy for E9050L**:
  - Render target image: `E9050L.png`.
  - Display Sections: **Device Overview**, **Battery Telemetry**, **Connection Mode**, **Scan Diagnostic Modal**.
  - Hide DPI, Lighting, and Macro control panels.

---

## Proposed OpenRapoo Rust Implementation Roadmap

```
Phase 1: Udev & Daemon Isolation
├── Refactor openrapoo-daemon to use non-blocking hidapi / tokio tasks.
├── Implement device classification (Mouse vs Keyboard vs Receiver).
└── Enforce read-only inspection for Keyboard nodes (no evdev grab).

Phase 2: Protocol Layer Enhancement
├── Implement Report ID 0x07 Feature Query for Battery Telemetry (MT760 & E9050L).
├── Add 7-gear DPI & Polling Rate configuration commands for MT760 Pro.
└── Integrate UPower DBus notification listener for BLE battery status.

Phase 3: GUI Polish & Modular Views
├── Render dedicated E9050L Keyboard Diagnostic View with asset image.
├── Move raw scan logs into a collapsible modal dialog (Button-triggered).
└── Implement smooth IPC status stream between daemon and GPUI frontend.
```

