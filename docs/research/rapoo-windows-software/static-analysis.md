# Architecture & Static Analysis Report

## Application Architecture Overview

The official Rapoo Windows application ("A HUB") utilizes a classic modular architecture separating presentation logic from hardware communication:

```
[ A HUB.exe ] (32-bit x86 PE GUI Application - 330 MB)
      │
      ├── QML / Qt Quick UI Layer (476 QML files)
      │     └── Uses Qt 5.15.2 (Qt5Core.dll, Qt5Quick.dll, Qt5Qml.dll)
      │
      └── Native Hardware Backend Layer
            └── RapooDevice.dll (3.3 MB MinGW C++ DLL)
                  ├── Direct Windows HID Subsystem API (hid.dll / setupapi.dll)
                  ├── HIDAPI wrapper calls (HidD_GetFeature, HidD_SetFeature, etc.)
                  ├── Qt Bluetooth API (Qt5Bluetooth.dll)
                  └── Qt SerialPort API (Qt5SerialPort.dll)
```

## Binary Inspection & Runtime Dependencies

### 1. Main Executable (`A HUB.exe`)
- **Target Architecture**: PE32 executable (GUI) Intel 80386 (32-bit x86).
- **Compiler / Toolchain**: MinGW-w64 / GCC for Windows x86.
- **Framework**: Qt 5.15.2 (built with MinGW 8.1.0 32-bit).
- **Key Imported DLLs**:
  - `Qt5Core.dll`, `Qt5Gui.dll`, `Qt5Widgets.dll`, `Qt5Quick.dll`, `Qt5Qml.dll`, `Qt5Network.dll`
  - `Qt5Bluetooth.dll` (used for Bluetooth LE discovery and battery querying)
  - `Qt5SerialPort.dll` (used for serial communication over RF dongles/UART mode)
  - `KERNEL32.dll`, `USER32.dll`, `ADVAPI32.dll`, `SHELL32.dll`

### 2. Core C++ Communication Engine (`RapooDevice.dll`)
- **Target Architecture**: PE32 DLL Intel 80386 (32-bit x86).
- **Exported Entry Points**:
  - `RapooDevice` class instantiation interface
  - Qt plugin entry function `qt_plugin_instance`
  - C-linkage entry points for HID enumeration and device dispatch
- **Low-Level Subsystem Linkage**:
  - `hid.dll` (Windows HID API functions: `HidD_GetFeature`, `HidD_SetFeature`, `HidD_GetInputReport`, `HidD_SetOutputReport`, `HidP_GetCaps`)
  - `setupapi.dll` (Device information sets and udev-equivalent enumeration: `SetupDiGetClassDevsA`, `SetupDiEnumDeviceInterfaces`)
  - `winmm.dll` (High-resolution multimedia timers for low-latency polling)

### 3. Static Library Archive (`libRapooDevice.a`)
- **Format**: Current `ar` archive, 604,160 bytes.
- **Contents**: Contains object files used to compile `RapooDevice.dll`.
- **Symbol Analysis**: Yielded **3,787 demangled C++ symbol signatures** covering **18 distinct hardware management classes**.

## Core C++ Hardware Classes Identified

From the demangled symbol analysis of `libRapooDevice.a`, the following key hardware classes were identified:

| Class Name | Function / Purpose |
| :--- | :--- |
| `BaseDevice` | Fundamental base class for all Rapoo peripherals. Manages DPI, light modes, battery level queries, macro loading, button bindings, and signal handling. |
| `DeviceFactory` | Factory design pattern implementation to instantiate model-specific device handlers based on VID/PID or Bluetooth MAC. |
| `QueryDevice` | Handles real-time device scanning, enumeration over USB HID / Bluetooth, and returning device connection maps. |
| `DevicePairing` | Base class for 2.4GHz wireless dongle pairing procedures and RF access address generation (`generate_access_address`). |
| `DevicePairingForCh585` | Specialized pairing handler for WCH CH585 RISC-V wireless chips. |
| `DevicePairingForNordicL15` | Specialized pairing handler for Nordic Semiconductor nRF52/L15 wireless platforms. |
| `DevicePairingForTelinkMouse` | Specialized pairing handler for Telink TLSR825x wireless mouse controllers. |
| `DevicePairingNearLink` | Specialized pairing handler for Huawei/SparkLink NearLink low-latency wireless protocol. |
| `FactoryPairingForNearLink` | Low-level factory pairing engine for NearLink dongles and devices. |
| `FactoryPairingForRealtek` | Factory pairing engine for Realtek wireless controller chips. |

## Key Static Strings & Function Signatures Extracted

Inspection of strings in `RapooDevice.dll` revealed the exact method signatures used for peripheral communication:
- `BaseDevice::getBatteryLife(int, int, int, int, int)`
- `BaseDevice::bleBatteryValueChanged(int)`
- `BaseDevice::setDPIListInfo(QList<int>)`
- `BaseDevice::setCurDPIGear(int)`
- `BaseDevice::emitReturnRateChange(int)`
- `QueryDevice::getCorrespondingDevice(unsigned short vid, unsigned short pid, QString name)`
- `DevicePairingForTelinkMouse::writeAuthCodeToMouse(QString)`
- `DevicePairingForTelinkMouse::readAuthCodeToMouse()`

## Subsystem Architectural Breakdown

```
                       +-----------------------------------+
                       |         Qt QML Frontend           |
                       |       (A HUB.exe / QML)           |
                       +-----------------------------------+
                                         |
                                         v
                       +-----------------------------------+
                       |        DeviceFactory /            |
                       |          BaseDevice               |
                       |     (RapooDevice.dll)             |
                       +-----------------------------------+
                                         |
                +------------------------+------------------------+
                |                        |                        |
                v                        v                        v
     +---------------------+  +---------------------+  +---------------------+
     |   USB HID Layer     |  |   Bluetooth LE      |  |  Serial / Dongle    |
     | (hid.dll / HIDAPI)  |  |  (Qt5Bluetooth)     |  |  (Qt5SerialPort)    |
     +---------------------+  +---------------------+  +---------------------+
                |                        |                        |
                +------------------------+------------------------+
                                         |
                                         v
                       +-----------------------------------+
                       |     Rapoo Physical Peripherals    |
                       | (MT760 Pro / NL / E9050L Keyboard)|
                       +-----------------------------------+
```

> [!NOTE]
> Detailed demangled symbol logs are archived in `docs/research/rapoo-windows-software/all_symbols.txt`.

