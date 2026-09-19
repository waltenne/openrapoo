# Complete File & Hash Inventory Report

## Overview

This document provides a detailed breakdown of all files discovered in the official Rapoo Windows software distribution directory (`/media/user/Games 2/A HUB`). A total of **938 files** totaling **485.04 MB** were cataloged and analyzed.

## File Breakdown by Extension

| Extension | Count | Description / Role in Architecture |
| :--- | :--- | :--- |
| `.qml` | 476 | Qt Quick UI components (Views, Controls, Modals, Device Skins) |
| `.qmlc` | 162 | Compiled Qt QML Bytecode cache files |
| `.dll` | 93 | Dynamic Link Libraries (Qt5 plugins, MinGW runtimes, C++ `RapooDevice.dll`) |
| `.png` | 75 | UI Graphical Assets & Device Renderings |
| `.qmltypes` | 29 | QML Type Definition descriptor files |
| `no_ext` | 25 | Configuration, metadata, and binary blob files |
| `.txt` | 22 | License descriptors, translation maps, and symbol tables |
| `.qm` | 9 | Compiled Qt Translation Binaries (Localization) |
| `.exe` | 4 | PE Executables (`A HUB.exe`, `AHUBMaintenanceTool.exe`, Audio Installers) |
| `.icns` / `.ico` | 6 | Icon resources for Windows application framing |
| `.cat` / `.inf` | 6 | Windows Device Driver Setup Information and Catalogs |
| `.bin` | 2 | Firmware Binary Images (`code_2M.bin`, `param_128K.bin`) |
| **Total** | **938** | **Total Disk Footprint: 485.04 MB** |

## Critical Executable Files & Core Libraries

Below is the verified inventory of primary executable binaries and low-level device management libraries:

| Path | Size (Bytes) | SHA-256 Hash | File Type / Architecture |
| :--- | :--- | :--- | :--- |
| `A HUB.exe` | 330,464,496 | `9e86e1c4a3bf8468995df4fe73645d2a67c9bf1e27dd066e42db5c11595c8b6a` | PE32 executable (GUI) Intel 80386 (stripped to external PDB) |
| `AHUBMaintenanceTool.exe` | 25,525,248 | `dcd483fc0654cd430cae7225ba95b02d39847f35ed9d3a7f3154467a777d3eb1` | PE32+ executable (console) x86-64 (Qt MaintenanceTool) |
| `RapooDevice.dll` | 3,460,096 | `7f1e948a3194098481eb081c78e47ec05fbdf9845210214a1e9569234b6e5114` | PE32 executable (DLL) Intel 80386, MinGW C++ build |
| `libRapooDevice.a` | 604,160 | `3a5796b291c944bf83df7819bfd021ab07e155097df5a2c4e12e347781a91e54` | Current ar archive, static symbol export object library |
| `code_2M.bin` | 2,097,152 | `e619fa1905fb559779df3df130a10996fa15d0234a7138b0561bdff60fb00bb8` | Raw Binary Blob (2MB MCU Firmware Image) |
| `param_128K.bin` | 131,072 | `f312891bf6054817d1217e91d0ca5970c657df7a3891456bf1a349b1a1362e55` | Raw Binary Blob (128KB NVM Parameter Block) |

## Directory Hierarchy & Structural Organization

```
/media/waltenne/Games 2/A HUB/
├── A HUB.exe                          # Main Qt5 Desktop GUI Application (330 MB)
├── AHUBMaintenanceTool.exe            # Application Updater / Installer Framework
├── RapooDevice.dll                    # Core Hardware Communication DLL (HIDAPI integration)
├── libRapooDevice.a                   # Exported C++ Archive with demangled symbols
├── code_2M.bin                        # MCU Firmware Image (2 MB)
├── param_128K.bin                     # Non-Volatile Flash Configuration Image (128 KB)
├── Qt5Core.dll, Qt5Quick.dll, ...     # Qt 5.15.2 Runtime Shared Libraries (32-bit x86)
├── audio/                             # Audio Virtualization Extensions & Driver Installers
│   └── default/AudioDriver/VE5/...
├── bearer/, iconengines/, imageformats/ # Qt Plugin Modules
├── platforms/
│   └── qwindows.dll                   # Qt Windows Platform Abstraction Layer
├── qml/                               # Extensive QML UI Component Tree (476 files)
│   ├── QtQuick/
│   ├── QtGraphicalEffects/
│   └── Rapoo/ Custom Widgets & Dialogs
└── styles/                            # Theme and Color Scheme Definitions
```

> [!NOTE]
> The full line-by-line file listing and complete cryptographic hash dictionary is saved in machine-readable JSON format at `docs/research/rapoo-windows-software/raw_inventory.json`.

