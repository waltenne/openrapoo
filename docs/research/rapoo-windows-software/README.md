# Rapoo Windows Software Reverse-Engineering & Technical Research Report

## Executive Summary

This directory contains a comprehensive, read-only static analysis of the official proprietary Rapoo Windows software ("A HUB" / Rapoo Driver Suite) located at `/media/user/Games 2/A HUB`.

The goal of this research is to dissect the internal architecture, hardware chipset support, HID protocols, battery telemetry, DPI/polling rates, macro engine, and firmware update mechanisms of Rapoo peripherals (specifically focusing on **Rapoo MT760 Pro**, **MT760 NearLink/Mini**, and Rapoo wireless keyboards like the **E9050L**). This empirical evidence will inform future developments of **OpenRapoo** on Linux without requiring proprietary software or Windows drivers.

## Safety & Compliance Statement

> [!IMPORTANT]
> All analyses conducted in this study were strictly read-only and performed on static files located at `/media/user/Games 2/A HUB`. 
> - No proprietary code was executed on physical devices during this investigation.
> - No binary files or directories in the source location were modified, moved, or deleted.
> - No hardware registers or firmware write operations were dispatched.
> - All conclusions are based on binary header analysis, demangled C++ symbol inspection, string extraction, XML/INI configuration schemas, and QML resource inspection.

## Evidence Confidence Levels

To maintain technical integrity, all findings across these documents are categorized into three explicit confidence levels:
1. **[CONFIRMED] Static Evidence**: Directly verified from exported C++ symbols (`libRapooDevice.a`), PE string tables (`RapooDevice.dll`), XML schemas (`App.xml`, `Device.xml`), or QML source files.
2. **[INDICATED] Strong Indication**: Standard patterns inferred from method parameters, Qt signal/slot names, HIDAPI wrapper calls, and chip vendor pairing flows (e.g. Nordic L15 vs Telink vs WCH CH585).
3. **[HYPOTHESIS] Technical Hypothesis**: Proposed HID packet layouts, feature report IDs, or state machines derived from data structure offsets and standard USB HID specifications.

## Research Documentation Directory Map

| Document | Description | Key Topics |
| :--- | :--- | :--- |
| [inventory.md](file:///home/waltenne/Documents/projects/openrapoo/docs/research/rapoo-windows-software/inventory.md) | Complete File & Hash Inventory | File counts, extensions, file types, SHA-256 hashes, size breakdowns |
| [static-analysis.md](file:///home/waltenne/Documents/projects/openrapoo/docs/research/rapoo-windows-software/static-analysis.md) | Architectural & DLL Inspection | Qt5 framework, C++ backend (`RapooDevice.dll`), static library `libRapooDevice.a`, platform components |
| [device-identifiers.md](file:///home/waltenne/Documents/projects/openrapoo/docs/research/rapoo-windows-software/device-identifiers.md) | VID/PID & Hardware Taxonomy | Rapoo USB Vendor IDs (`0x24AE`), PIDs, chipset families (Telink, Nordic L15, CH585, NearLink, Realtek) |
| [hid-protocol-hypotheses.md](file:///home/waltenne/Documents/projects/openrapoo/docs/research/rapoo-windows-software/hid-protocol-hypotheses.md) | HID Protocol & Report Structs | HIDAPI calls, Feature Reports (`0x07`, `0x08`), Output/Input Reports, RF Pairing packets |
| [battery-analysis.md](file:///home/waltenne/Documents/projects/openrapoo/docs/research/rapoo-windows-software/battery-analysis.md) | Battery Telemetry & State | Telemetry queries (`getBatteryLife`), BLE battery service, charging status decoding, UPower integration |
| [dpi-polling-analysis.md](file:///home/waltenne/Documents/projects/openrapoo/docs/research/rapoo-windows-software/dpi-polling-analysis.md) | DPI & Polling Rate Mechanics | 7-gear DPI switching, X/Y independent DPI, polling rates (125Hz-1000Hz), NearLink high-rate support |
| [profiles-and-macros.md](file:///home/waltenne/Documents/projects/openrapoo/docs/research/rapoo-windows-software/profiles-and-macros.md) | Profiles, Remapping & Macros | QML macro engine, XML key bindings, on-board profile storage vs driver software profiles |
| [firmware-analysis.md](file:///home/waltenne/Documents/projects/openrapoo/docs/research/rapoo-windows-software/firmware-analysis.md) | Firmware & OTA Architecture | Analysis of `code_2M.bin` and `param_128K.bin`, OTA update slots, flashing safety checks |
| [openrapoo-opportunities.md](file:///home/waltenne/Documents/projects/openrapoo/docs/research/rapoo-windows-software/openrapoo-opportunities.md) | OpenRapoo Integration Roadmap | Architectural opportunities, daemon separation, kernel udev rules, proposed Rust implementation plan |
| [evidence-index.json](file:///home/waltenne/Documents/projects/openrapoo/docs/research/rapoo-windows-software/evidence-index.json) | Structured Evidence Index | JSON mapping of evidence keys to binary paths, symbols, strings, and line references |

---

## Key Hardware Findings Summary

1. **Multi-Chip Architecture Support**: Rapoo's official software handles 5 primary wireless/USB chipset platforms:
   - **Telink Semiconductor** (e.g. `TLSR825x` series): Handled by `DevicePairingForTelinkMouse`.
   - **WCH / WinChipHead** (e.g. `CH584` / `CH585` RISC-V BLE/2.4G chips): Handled by `DevicePairingForCh585`.
   - **Nordic Semiconductor** (e.g. `nRF52840` / `L15` platform): Handled by `DevicePairingForNordicL15`.
   - **NearLink (SparkLink / 星闪)**: Handled by `FactoryPairingForNearLink` & `DevicePairingNearLink` (used in modern low-latency mice like MT760 NL).
   - **Realtek Semiconductor**: Handled by `FactoryPairingForRealtek`.

2. **Unified Core Engine**: The entire peripheral communication stack rests inside a C++ Qt plugin dynamic library: `RapooDevice.dll` (backed by demangled archive `libRapooDevice.a`).
3. **HID Report Structure**: Devices communicate primarily via HID Feature Reports (`HidD_GetFeature` / `HidD_SetFeature`) and vendor-defined HID collections operating on Report IDs such as `0x07` and `0x08`.

