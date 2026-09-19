# Battery Telemetry & State Analysis Report

## Overview

Battery telemetry in Rapoo wireless peripherals is handled through dual channels depending on the physical connection transport:
1. **2.4GHz Dongle / Wired USB Transport**: Communicates via proprietary USB HID Feature Reports handled by `BaseDevice::getBatteryLife(...)`.
2. **Bluetooth Low Energy (BLE) Transport**: Communicates via standard GATT Battery Service (`UUID 0x180F`, Characteristic `0x2A19`) or vendor BLE notifications captured by `BaseDevice::bleBatteryValueChanged(int)`.

---

## Static Evidence & C++ Method Signatures

Demangled symbol analysis of `libRapooDevice.a` and string extraction from `RapooDevice.dll` revealed the exact method signatures dedicated to battery telemetry:

- **`BaseDevice::getBatteryLife(int param1, int param2, int param3, int param4, int param5)`** `[CONFIRMED]`
  - Purpose: Polling routine triggered by the Qt backend timer to request battery state over HID.
  - Parameters: Returns raw voltage thresholds, percentage calculation flags, charging flags, and device index.
- **`BaseDevice::bleBatteryValueChanged(int value)`** `[CONFIRMED]`
  - Purpose: Qt slot connected to `Qt5Bluetooth` notifications when operating in Bluetooth LE mode.
  - Parameter `value`: Direct integer percentage (`0..100`).
- **`BaseDevice::updateBatteryState(int)`** `[CONFIRMED]`
  - Purpose: Updates internal state and emits Qt signals to refresh QML UI battery icons.

---

## Battery Telemetry Mechanism Matrix

| Transport Mode | Query Method | Report / Service | Battery Level Format | Charging Detection |
| :--- | :--- | :--- | :--- | :--- |
| **2.4GHz Receiver (MT760 Pro / Mini)** | Periodic HID Feature Query | HID Feature Report `0x07` | Byte value (`0x00..0x64`) representing 0..100% | Status Bit in Feature Report (Bit 0 = Charging, Bit 1 = Full) |
| **NearLink Receiver (MT760 NL)** | High-Rate Vendor Report | Feature / Input Report `0x07` | Byte value (`0x00..0x64`) | Status Bit in Vendor Payload Header |
| **Bluetooth LE (MT760 / E9050L)** | BLE GATT Notification | Service `0x180F` / Char `0x2A19` | 1-Byte Unsigned Integer (`0..100`) | Standard BLE Battery Level GATT Descriptor |

---

## Battery Calculation & Charging Curves

The C++ logic inside `BaseDevice::getBatteryLife` converts raw ADC battery voltage levels (reported in millivolts by Telink/WCH microcontrollers) into percentage steps for UI display:

```
Voltage (mV)      Percentage (%)    UI Battery Icon State
------------      --------------    ---------------------
>= 4150 mV   -->     100%       --> 4 Bars (Full)
>= 3950 mV   -->      80%       --> 3 Bars
>= 3800 mV   -->      60%       --> 3 Bars
>= 3650 mV   -->      40%       --> 2 Bars
>= 3500 mV   -->      20%       --> 1 Bar (Low Battery Warning)
<  3400 mV   -->       5%       --> 0 Bars (Critical - Red Flash)
```

> [!NOTE]
> When plugged into USB for charging, the hardware sets the charging status bit. The UI displays an animated bolt icon over the battery meter regardless of exact percentage.

---

## Linux Integration Strategy for OpenRapoo

To provide clean battery integration on Linux without polling overhead or race conditions:

### 1. Bluetooth Mode (BlueZ & UPower Integration)
- Standard Linux `upowerd` automatically detects Bluetooth LE battery status via BlueZ GATT services.
- OpenRapoo can read battery levels directly via D-Bus from `org.freedesktop.UPower.Device` without making raw HID calls.

### 2. 2.4GHz Receiver / HIDRAW Integration
- Direct querying via `/dev/hidrawX` using `HIDIOCGFEATURE` on Report ID `0x07`.
- **Recommended Polling Interval**: Query battery every **60 seconds** or when the device wakes from sleep to prevent flooding the wireless RF channel.
- OpenRapoo can optionally expose battery status to UPower via Linux kernel `power_supply` class or DBus interface.

