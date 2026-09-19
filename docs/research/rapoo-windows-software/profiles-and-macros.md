# Profiles, Remapping & Macro Engine Analysis Report

## Overview

Rapoo's official Windows software incorporates a flexible QML-driven macro editor and button remapping engine. Configurations can be stored locally on disk as XML/INI profiles or written to on-board MCU EEPROM/Flash memory for driverless persistence across machines.

---

## Static Symbol & QML Architecture Analysis

Demangled C++ symbols from `libRapooDevice.a` and QML files in `qml/Rapoo/` reveal the structure of the macro and button remapping engine:

### 1. Macro & Configuration Methods (`BaseDevice`)
- **`BaseDevice::addConfigInfo(QMap<QString, QVariant>)`** `[CONFIRMED]`
  - Adds or updates a profile configuration map.
- **`BaseDevice::deleteConfigInfo(int profileIndex)`** `[CONFIRMED]`
  - Removes a saved user profile.
- **`BaseDevice::deleteMacro(int macroId)`** `[CONFIRMED]`
  - Deletes a single macro sequence.
- **`BaseDevice::deleteMultMacro(int macroGroupId)`** `[CONFIRMED]`
  - Purges a group of macros.
- **`BaseDevice::importConfigFile(QString path)` / `exportConfigFile(QString path)`** `[CONFIRMED]`
  - Serializes profiles to/from XML configuration files.

### 2. QML Front-End Widgets
Inspection of the QML asset tree (`476 QML files`) identified key macro UI components:
- `MacroEditView.qml`: Macro recorder interface (keypress capture, down/up delay timing, loop counts).
- `KeyMappingDialog.qml`: Interactive button remapper for mouse buttons (Left Click, Right Click, Middle Click, Forward, Back, DPI Loop, Macro Trigger, Media Key).
- `ProfileManager.qml`: Switcher between Mode 1 (Office Mode), Mode 2 (Gaming Profile 1), and Mode 3 (Gaming Profile 2).

---

## On-Board Flash Storage vs Driver-Side Profiles

```
                             +-----------------------------------+
                             |     User Action / Selection       |
                             +-----------------------------------+
                                               |
                     +-------------------------+-------------------------+
                     |                                                   |
                     v                                                   v
      +-----------------------------+                     +-----------------------------+
      |    Driver-Side Profile      |                     |      On-Board Hardware      |
      |   (Stored in XML/INI Disk)  |                     |    (Written to MCU Flash)   |
      +-----------------------------+                     +-----------------------------+
      | - Unlimited Macro Storage   |                     | - Limited to 3 Profiles     |
      | - Requires Software Running |                     | - Driverless (Works on Linux|
      | - Complex Delays & Scripts  |                     |   or Mac without App)       |
      +-----------------------------+                     +-----------------------------+
```

> [!IMPORTANT]
> **Keyboard vs Mouse Remapping Policy**:
> Static analysis confirms that Rapoo's driver suite **does NOT enable software key remapping for wireless keyboards like the Rapoo E9050L**. Keyboards run fixed standard HID scancode tables. Remapping and Macro UI options are exclusively conditionally rendered for gaming mice (MT760 Pro, VT9 series).

---

## Profile XML Schema Structure

User configuration profiles exported by `BaseDevice::exportConfigFile` follow a structured XML format:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<RapooProfile version="1.0">
    <DeviceInfo model="MT760 PRO" pid="0x186A" vid="0x24AE"/>
    <ActiveProfile index="0"/>
    <DpiSettings currentGear="3">
        <Gear index="1" value="400" color="#FF0000"/>
        <Gear index="2" value="800" color="#00FF00"/>
        <Gear index="3" value="1200" color="#0000FF"/>
        <Gear index="4" value="1600" color="#FFFF00"/>
        <Gear index="5" value="2400" color="#00FFFF"/>
        <Gear index="6" value="3200" color="#FF00FF"/>
        <Gear index="7" value="6400" color="#FFFFFF"/>
    </DpiSettings>
    <PollingRate value="1000"/>
    <ButtonBindings>
        <Button index="1" action="LeftClick"/>
        <Button index="2" action="RightClick"/>
        <Button index="3" action="MiddleClick"/>
        <Button index="4" action="BrowserForward"/>
        <Button index="5" action="BrowserBack"/>
        <Button index="6" action="DpiSwitch"/>
        <Button index="7" action="Macro" macroId="101"/>
    </ButtonBindings>
    <Macros>
        <Macro id="101" name="Rapid Fire">
            <Event type="KeyDown" code="0x01" delayMs="10"/>
            <Event type="KeyUp" code="0x01" delayMs="50"/>
        </Macro>
    </Macros>
</RapooProfile>
```

---

## Macro Binary Frame Layout (On-Board Upload)

When writing a macro to mouse hardware via `HidD_SetFeature` Report ID `0x07` (Category `0x06`):

```
Byte Offset : Field Description
----------- : ---------------------------------------------------
[0x00]      : Report ID (0x07)
[0x01]      : Category: Macro Data Upload (0x06)
[0x02]      : Macro Slot Index (0x01..0x0A)
[0x03]      : Total Action Sequence Length
[0x04..0x07]: Event 1: [HID Scancode, Action Type (Down/Up), Delay Low Byte, Delay High Byte]
[0x08..0x0B]: Event 2: [HID Scancode, Action Type (Down/Up), Delay Low Byte, Delay High Byte]
...         : ...
[0x3F]      : Frame XOR Checksum
```

