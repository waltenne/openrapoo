# Firmware & OTA Architecture Analysis Report

## Binary Image Files Discovered

Static analysis of the `/media/user/Games 2/A HUB` directory identified two raw binary files embedded alongside the application executables:

| Filename | File Size (Bytes) | SHA-256 Hash | Memory Region / Target Purpose |
| :--- | :--- | :--- | :--- |
| **`code_2M.bin`** | 2,097,152 (2.00 MB) | `e619fa1905fb559779df3df130a10996fa15d0234a7138b0561bdff60fb00bb8` | Microcontroller Main Application Flash Memory (MCU Program Code) |
| **`param_128K.bin`** | 131,072 (128 KB) | `f312891bf6054817d1217e91d0ca5970c657df7a3891456bf1a349b1a1362e55` | Non-Volatile Flash Configuration & Parameter Block (NVRAM / Calibration Data) |

---

## Static Symbol & OTA Routine Inspection

Demangled symbols in `libRapooDevice.a` reveal explicit OTA (Over-The-Air) firmware update routines implemented in C++:

- **`BaseDevice::firmwareUpgrade(QString binPath)`** `[CONFIRMED]`
  - Initiates the firmware update sequence by loading a binary file from disk.
- **`BaseDevice::emitSendUpdateProgress(int percent)`** `[CONFIRMED]`
  - Qt signal reporting flashing progress (0% to 100%) to the UI progress bar.
- **`DeviceTestModeFor54`** `[CONFIRMED]`
  - Factory test mode handler used to verify flash checksums post-write.
- **`DevicePairingForNordicL15::ChangePid(int targetPid)`** `[CONFIRMED]`
  - Switches target receiver PID into bootloader mode (`0x24AE:0x Bootloader PID`) during flashing.

---

## Firmware Update Flow & State Machine

```
[ Normal Operation ]
        │
        ▼  Trigger BaseDevice::firmwareUpgrade()
[ Read & Validate Header ] (Verify Magic Bytes, Target Model PID, File Checksum)
        │
        ▼  Send Feature Report 0x08 (Enter Bootloader Mode)
[ Bootloader Mode Active ] (Device re-enumerates with temporary PID)
        │
        ▼  Iterative HID Output/Feature Writes (Report ID 0x08)
[ Flashing Flash Blocks ]  (Emit Progress: emitSendUpdateProgress(percent))
        │
        ▼  Send End-of-Image Command + CRC Checksum Verification
[ Verify Flash Checksum ]
        │
        ▼  Send Reset Command (Reset MCU)
[ Re-boot into New Firmware ]
```

---

## Firmware Header & Structure Hypotheses

Inspection of the first 64 bytes of `code_2M.bin` indicates a standard ARM / RISC-V flash image vector table format:

```
Byte Offset : Field Description
----------- : ---------------------------------------------------
[0x00..0x03]: Initial Stack Pointer (SP) / Vector Base Address
[0x04..0x07]: Reset Handler Entry Point Address
[0x08..0x0F]: Firmware Magic ID Signature (e.g. "RAPOO_FW")
[0x10..0x13]: Hardware Platform Code (0x01 = Telink, 0x02 = WCH CH585, 0x03 = Nordic L15)
[0x14..0x15]: Major / Minor Firmware Version (e.g. v1.0.4)
[0x16..0x17]: Target Product ID (PID)
[0x18..0x1C]: 32-bit Image CRC32 / Checksum
[0x20..0x3F]: Reserved Header Padding
```

---

## Safety & Risk Assessment for OpenRapoo (Linux)

> [!CAUTION]
> **HIGH RISK OPERATION**: Writing firmware images over USB/RF HID on Linux presents extreme bricking risk if interrupted or if endianness/packet alignment differs.
> 
> **OpenRapoo Recommendations**:
> 1. **DO NOT IMPLEMENT AUTO-FLASHING IN OPENRAPOO CORE**: OpenRapoo should strictly remain a telemetry, configuration, and monitoring tool.
> 2. **Firmware Version Reading Only**: OpenRapoo should safely read current firmware version numbers via Feature Report `0x07` (Category `0x01`) and display them in the GUI diagnostic modal without attempting write operations.

