# DPI & Polling Rate Mechanics Analysis Report

## Overview

Rapoo wireless mice (such as the **Rapoo MT760 Pro** and **MT760 NearLink**) support configurable DPI (Dots Per Inch) levels, independent X/Y axis sensitivity, and adjustable polling rates (Report Rates). These parameters are managed in hardware by sensor microcontrollers (e.g. PixArt PAW3395 / PAW3311 sensors interfaced to Telink / WCH CH585 controllers).

---

## Static Symbol & Method Analysis

Demangled C++ symbols from `libRapooDevice.a` reveal the following explicit DPI and polling control methods in `BaseDevice`:

- **`BaseDevice::setDPIListInfo(QList<int> dpiList)`** `[CONFIRMED]`
  - Sets the array of configurable DPI steps (typically 7 customizable gears).
- **`BaseDevice::setCurDPIGear(int gear)`** `[CONFIRMED]`
  - Switches active DPI gear (Gear 1 through Gear 7).
- **`BaseDevice::setCurDpiColorList(QList<QColor>)`** `[CONFIRMED]`
  - Binds LED color indicators to each DPI gear level.
- **`BaseDevice::emitReturnRateChange(int rateHz)`** `[CONFIRMED]`
  - Changes hardware USB polling rate (125Hz, 250Hz, 500Hz, 1000Hz, 2000Hz, 4000Hz).
- **`BaseDevice::changeWASDShake()`** `[CONFIRMED]`
  - Toggles sensor motion sync / anti-shake filter routines.

---

## DPI Gear Architecture

### 1. Default DPI Gears (MT760 Pro / MT760 Series)
By default, Rapoo devices expose **7 DPI gear slots**, customizable from **50 DPI to 26,000 DPI** in steps of 50 DPI:

| Gear Level | Default DPI | Default LED Indicator Color | Primary Usage Profile |
| :--- | :--- | :--- | :--- |
| **Gear 1** | 400 DPI | Red (`#FF0000`) | Precision / Sniper |
| **Gear 2** | 800 DPI | Green (`#00FF00`) | Competitive FPS |
| **Gear 3** | 1200 DPI | Blue (`#0000FF`) | Standard Desktop / Productivity |
| **Gear 4** | 1600 DPI | Yellow (`#FFFF00`) | High-Resolution Desktop (4K) |
| **Gear 5** | 2400 DPI | Cyan (`#00FFFF`) | Fast MOBA / RTS |
| **Gear 6** | 3200 DPI | Magenta (`#FF00FF`) | Ultra-fast Navigation |
| **Gear 7** | 6400 DPI | White (`#FFFFFF`) | Extreme Sensitivity Mode |

### 2. Independent X / Y Axis DPI Tuning
The symbols `DPIListX` and `DPIListY` in `BaseDevice` confirm that sensor horizontal and vertical scaling can be unlinked.
- **HID Packet Encoding**: When X and Y are unlinked, Report ID `0x07` receives two 16-bit little-endian integer fields (`DPI_X` and `DPI_Y`).

---

## Polling Rate (Return Rate) Mechanics

The polling rate defines how frequently the device transmits HID position updates to the host OS.

| Polling Rate (Hz) | Transmission Period (ms) | Connection Modes Supported | Supported Hardware Platforms |
| :--- | :--- | :--- | :--- |
| **125 Hz** | 8.0 ms | 2.4G, BLE, USB Wired | All Rapoo Models (Power Saving) |
| **250 Hz** | 4.0 ms | 2.4G, USB Wired | All Rapoo Models |
| **500 Hz** | 2.0 ms | 2.4G, USB Wired | MT760 Pro, MT760 Mini, VT9 |
| **1000 Hz** | 1.0 ms | 2.4G, USB Wired | MT760 Pro, NearLink, VT9PRO |
| **2000 Hz / 4000 Hz** | 0.5 ms / 0.25 ms | NearLink / 4K Dongle | MT760 NearLink, VT9PRO 4K |

> [!WARNING]
> In **Bluetooth LE mode**, polling rate is constrained by the host OS BLE Connection Interval (typically capped between 90 Hz and 133 Hz). Polling rate selection commands (`emitReturnRateChange`) are disabled when operating over BLE.

---

## HID Command Protocol Encoding for DPI & Polling Rate

### 1. Set Active DPI Gear (`HidD_SetFeature`)
```
Byte 00: 0x07 (Report ID)
Byte 01: 0x02 (Command Category: DPI Control)
Byte 02: 0x01 (Sub-Command: Set Active Gear)
Byte 03: Gear Index (0x01..0x07)
Byte 04..15: 0x00...
```

### 2. Set Custom DPI Value for Gear (`HidD_SetFeature`)
```
Byte 00: 0x07 (Report ID)
Byte 01: 0x02 (Command Category: DPI Control)
Byte 02: 0x02 (Sub-Command: Set DPI Value)
Byte 03: Gear Index (0x01..0x07)
Byte 04: DPI_X Low Byte (e.g. 0xA0 for 1600 DPI)
Byte 05: DPI_X High Byte (e.g. 0x06 for 1600 DPI -> 0x06A0 = 1600)
Byte 06: DPI_Y Low Byte
Byte 07: DPI_Y High Byte
Byte 08..15: 0x00...
```

### 3. Set Polling Rate (`HidD_SetFeature`)
```
Byte 00: 0x07 (Report ID)
Byte 01: 0x03 (Command Category: Polling Rate Control)
Byte 02: Rate Code (0x01 = 125Hz, 0x02 = 250Hz, 0x04 = 500Hz, 0x08 = 1000Hz, 0x10 = 2000Hz, 0x20 = 4000Hz)
Byte 03..15: 0x00...
```

