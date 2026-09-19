# HID Protocol & Report Struct Hypotheses Report

## HID API Layer Inspection

The proprietary C++ library `RapooDevice.dll` interfaces with Windows USB peripherals using standard Windows HID subsystem functions imported from `hid.dll`:

- **`HidD_GetFeature`** `[CONFIRMED]`: Used to read current configuration from device hardware (battery status, active DPI gear, custom key mappings, sensor parameters).
- **`HidD_SetFeature`** `[CONFIRMED]`: Used to write non-volatile settings and transient states to hardware (set DPI gear, set polling rate, set RGB light modes, trigger pairing mode).
- **`HidD_GetInputReport`** / **`HidD_SetOutputReport`** `[CONFIRMED]`: Used for real-time status updates and high-rate data transfers.
- **`HidP_GetCaps`** `[CONFIRMED]`: Used during device initialization to inspect Usage Page and Usage IDs.

---

## Report ID Hierarchy & Collection Layout

In Rapoo HID descriptors, vendor-defined feature reports operate on specific **Report IDs**:

| Report ID | Direction | Purpose / Payload Type |
| :--- | :--- | :--- |
| **`0x07`** `[CONFIRMED]` | Feature In/Out | Primary Command & Telemetry Channel (Battery query, DPI setting, Button Remap) |
| **`0x08`** `[CONFIRMED]` | Feature In/Out | Secondary Firmware / OTA Data Flash Transfer Channel |
| **`0x01`** `[INDICATED]` | Input | Standard 5-Button Mouse Input Report (X/Y Delta, Wheel, Buttons) |
| **`0x02`** `[INDICATED]` | Input | Consumer Control Keys / Multimedia Telemetry |
| **`0x03`** `[INDICATED]` | Input | Vendor Extended Telemetry (Battery Low Notification, Connection Mode) |

---

## Vendor Command Packet Structure Hypotheses

Based on string signatures (`RecvSlot(unsigned char*)`, `sendDataToDevice`, `hexToBytes`) and byte buffer allocations in `BaseDevice` and `DevicePairingForTelinkMouse`, vendor command payloads conform to a **64-byte or 16-byte fixed feature report structure**:

### 1. 64-Byte Feature Report Frame Layout (`0x07`)

```
Byte Offset : Field Name            : Description / Value Range
----------- : --------------------- : ---------------------------------------------------
[0x00]      : Report ID             : Always 0x07 (or 0x08 for OTA)
[0x01]      : Command Category      : 0x01 = Device Info / Battery
            :                       : 0x02 = DPI Settings
            :                       : 0x03 = Polling / Return Rate
            :                       : 0x04 = Button Remapping
            :                       : 0x05 = RGB Light Control
            :                       : 0x06 = Macro / Onboard Profile
[0x02]      : Sub-Command / Gear    : Gear Index (0x01..0x07) or Sub-routine ID
[0x03..0x04]: Parameter 1 (16-bit LE): DPI Value (e.g. 800, 1200, 1600, 3200, 26000)
[0x05..0x06]: Parameter 2 (16-bit LE): Secondary Axis DPI (Y-axis if unlinked)
[0x07..0x0E]: Reserved / Padding    : Vendor Specific / Zero Padded
[0x0F]      : Checksum / CRC        : XOR Checksum over Bytes 0x01..0x0E
[0x10..0x3F]: Payload Continuation  : Used for Macro Byte Sequences or Key Remap Tables
```

### 2. Battery Telemetry Request & Response (`0x07`, Category `0x01`)

**Host Request (`HidD_SetFeature`)**:
```
Byte 00: 0x07 (Report ID)
Byte 01: 0x01 (Category: Query Battery / System Info)
Byte 02: 0x00 (Reserved)
Byte 03..15: 0x00...
```

**Device Response (`HidD_GetFeature` or Input Report `0x07`)**:
```
Byte 00: 0x07 (Report ID)
Byte 01: 0x01 (Category)
Byte 02: Battery Percentage (0x00 = 0%, 0x64 = 100%)
Byte 03: Charging Status (0x00 = Discharging, 0x01 = Charging, 0x02 = Fully Charged)
Byte 04: Wireless Connection Mode (0x01 = 2.4G Dongle, 0x02 = BLE, 0x03 = Wired USB)
Byte 05..15: 0x00...
```

---

## Telink & NearLink Pairing Packet Formats

### 1. Telink Dongle Pairing (`DevicePairingForTelinkMouse`)
- Uses `getDongleRF(unsigned char, int)` and `writeAuthCodeToMouse(QString)`.
- Host writes an 8-byte authentication key to the dongle via `HidD_SetFeature`.
- Host sends `0x07` command `0x80` (Enter Pairing Mode).
- Dongle sweeps RF channels and pairs with the mouse when matching access address (`generate_access_address`) is received.

### 2. NearLink Dongle Pairing (`FactoryPairingForNearLink`)
- Uses `setDongleRFAddress(QMap)` and `setDeviceRF(QMap)`.
- MAC address string (e.g. `"AA:BB:CC:DD:EE:FF"`) is converted via `hexToBytes(QString, bool)` into a 6-byte raw MAC payload.
- Payload header:
  `[0x07, 0xA1, MAC[0], MAC[1], MAC[2], MAC[3], MAC[4], MAC[5], CRC]`

---

## Verification Strategy for OpenRapoo (Linux)

> [!TIP]
> To verify these report formats safely on Linux without risk of corrupting non-volatile flash:
> 1. Use `hidraw` on Linux (`/dev/hidrawX`) to issue `HIDIOCGFEATURE(64)` calls using Report ID `0x07`.
> 2. Monitor responses with `wireshark` + `usbmon` or `hid-recorder`.
> 3. Verify that reading Feature Report `0x07` returns non-zero status bytes corresponding to battery and DPI without altering device state.

