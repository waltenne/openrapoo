# Device Identifiers & Hardware Taxonomy Report

## Vendor & Product ID Catalog

Static analysis of `RapooDevice.dll`, `libRapooDevice.a`, and QML device assets reveals that Rapoo peripherals use specific Vendor IDs (VIDs) and Product IDs (PIDs) depending on whether they are connected via Direct USB cable, 2.4GHz Wireless Dongle, NearLink Dongle, or Bluetooth LE.

### 1. Primary Vendor Identifiers (VIDs)
- **`0x24AE`** `[CONFIRMED]`: Primary Rapoo Corporation USB Vendor ID used across 2.4GHz dongles and wired USB modes.
- **`0x1C4F`** `[CONFIRMED]`: Legacy / Secondary Rapoo (SiGma Micro OEM) Vendor ID used for budget keyboards and older receivers.
- **`0x1E7D`** `[INDICATED]`: OEM Receiver Vendor ID variant found in secondary string table references.

### 2. Extracted Product Identifiers (PIDs)

The following PID catalog was extracted from static string patterns, QML model maps, and `QueryDevice::getCorrespondingDevice(unsigned short, unsigned short, QString)` symbol references:

| Device Model Name | Connection Mode | Vendor ID (VID) | Product ID (PID) | Chipset / Platform Architecture |
| :--- | :--- | :--- | :--- | :--- |
| **Rapoo MT760 Pro** | 2.4GHz Dongle Mode | `0x24AE` | `0x186A` | Telink TLSR / WCH CH585 Dual-Mode |
| **Rapoo MT760 Pro** | Wired USB Mode | `0x24AE` | `0x4510` | Telink TLSR Direct HID |
| **Rapoo MT760 NearLink (NL)** | NearLink Dongle Mode | `0x24AE` | `0x2014` | SparkLink / NearLink Transceiver |
| **Rapoo MT760 Mini** | 2.4GHz Dongle Mode | `0x24AE` | `0x186B` | Telink TLSR825x Compact |
| **Rapoo MT760AI** | 2.4GHz / USB Mode | `0x24AE` | `0x186C` | WCH CH585 AI Smart Mouse |
| **Rapoo E9050L Keyboard** | 2.4GHz Dongle Mode | `0x24AE` | `0x1008` | Telink Wireless Keyboard Controller |
| **Rapoo E9050L Keyboard** | Bluetooth LE Mode | `0x24AE` (or BLE UUID) | `0x1009` / BLE | Standard GATT / HID-over-GATT |

> [!IMPORTANT]
> **PID `0x1008` Ambiguity Resolution**: In Linux kernel log analysis (`dmesg` / `openrapoo` logs), PID `0x1008` occasionally presents as a dual-node HID interface (Interface 0: Keyboard, Interface 1: Consumer Control / Vendor Feature collection). The official software handles this by separating interface queries via `QueryDevice::getDeviceList(unsigned short, unsigned short, int interface_index)`.

---

## Chipset Family Categorization

The static C++ library `libRapooDevice.a` contains explicit pairing and control classes for 5 distinct chipset hardware platforms:

```
                                  +-----------------------------+
                                  |     Rapoo Hardware Chips    |
                                  +-----------------------------+
                                                 |
         +------------------+--------------------+--------------------+------------------+
         |                  |                    |                    |                  |
         v                  v                    v                    v                  v
+------------------+ +------------------+ +------------------+ +------------------+ +------------------+
|  Telink Platform | |  WCH CH585/CH584 | |  Nordic L15/nRF  | | NearLink (Spark) | | Realtek Platform |
| (DevicePairing   | | (DevicePairing   | | (DevicePairing   | | (DevicePairing   | | (FactoryPairing  |
|  ForTelinkMouse) | |  ForCh585)       | |  ForNordicL15)   | |  NearLink)       | |  ForRealtek)     |
+------------------+ +------------------+ +------------------+ +------------------+ +------------------+
```

### 1. Telink Platform (`DevicePairingForTelinkMouse`)
- **Supported Models**: standard MT760, MT760 Mini, VT9 series, E9050L Keyboard.
- **Key Methods**: `extractTelinkMouseModels()`, `readAuthCodeToMouse()`, `writeAuthCodeToMouse()`.
- **Features**: Authentication code challenge-response over vendor HID report ID `0x07`.

### 2. WCH CH585 / CH584 Platform (`DevicePairingForCh585`)
- **Supported Models**: MT760 Pro, MT760AI.
- **Features**: RISC-V BLE 5.3 + 2.4GHz proprietary transceiver. High-speed feature report updates.

### 3. Nordic Semiconductor L15 (`DevicePairingForNordicL15`)
- **Supported Models**: High-end gaming mice (VT9PRO 4K, VT9 Air).
- **Features**: 4000Hz / 8000Hz polling rates, `ChangePid(int)` dynamic product ID switching during firmware update.

### 4. NearLink Platform (`FactoryPairingForNearLink`)
- **Supported Models**: MT760 NL (NearLink edition).
- **Features**: Low-latency Wireless RF Pairing via MAC address transmission (`setDongleRFAddress`, `setDeviceRF`).

---

## Device Name & String Identification Rules

The C++ method `DevicePairingForTelinkMouse::getChipTypeByDeviceName(QString)` matches physical device string descriptors returned by USB/BLE enumeration:

- Devices containing `"MT760"` -> Handled as MT760 Series (Mouse features enabled: DPI, Polling, Remap, Macro).
- Devices containing `"E9050L"` or `"E9050"` -> Handled as Wireless Keyboard (Keyboard status only: Device info, Battery telemetry, Diagnostics; NO software remapping/DPI).
- Devices containing `"NearLink"` or `"NL"` -> Uses NearLink protocol layer (`DevicePairingNearLink`).

