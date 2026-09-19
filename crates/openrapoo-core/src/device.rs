//! Device detection and representation for Rapoo mice on Linux.
//!
//! This module handles:
//! - Scanning `/proc/bus/input/devices` and `/sys/bus/hid/devices`
//! - Matching Rapoo devices by Vendor ID (`0x24AE`)
//! - Determining connection type (USB, 2.4 GHz, Bluetooth, NearLink)
//! - Mapping to `/dev/input/event*` and `/dev/hidraw*` paths

use crate::{error::OpenRapooError, RAPOO_VENDOR_ID};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info, warn};

use crate::battery::BatteryStatus;

/// Device type classification for extensible UI & feature handling.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    #[default]
    Mouse,
    Keyboard,
    Other,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Mouse => write!(f, "Mouse"),
            DeviceType::Keyboard => write!(f, "Teclado"),
            DeviceType::Other => write!(f, "Outro"),
        }
    }
}

/// Dynamic connection state of the device.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceConnectionState {
    #[default]
    Connected,
    Disconnected,
    Reconnecting,
}

impl std::fmt::Display for DeviceConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceConnectionState::Connected => write!(f, "Conectado"),
            DeviceConnectionState::Disconnected => write!(f, "Desconectado"),
            DeviceConnectionState::Reconnecting => write!(f, "Reconectando"),
        }
    }
}

/// Detailed internal state of a physical USB receiver or Bluetooth radio connection.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiverState {
    /// Dongle USB plugged in, but no 2.4GHz mouse paired or active
    ReceiverPresent,
    /// Dongle USB plugged in and mouse paired in firmware
    ReceiverPaired,
    /// Mouse actively communicating and responding via 2.4GHz dongle
    ReceiverActive,
    /// Device connected and active via Bluetooth (BlueZ ACL link)
    BluetoothConnected,
    /// Disconnected
    Disconnected,
    /// Unknown state
    #[default]
    Unknown,
}

impl std::fmt::Display for ReceiverState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReceiverState::ReceiverPresent => write!(f, "Receptor USB Conectado (Sem Mouse)"),
            ReceiverState::ReceiverPaired => write!(f, "Receptor USB Pareado"),
            ReceiverState::ReceiverActive => write!(f, "Mouse Ativo via 2.4GHz"),
            ReceiverState::BluetoothConnected => write!(f, "Conectado via Bluetooth"),
            ReceiverState::Disconnected => write!(f, "Desconectado"),
            ReceiverState::Unknown => write!(f, "Desconhecido"),
        }
    }
}

/// Representation of a physical USB receiver (dongle) or hardware endpoint.
#[derive(Debug, Clone)]
pub struct PhysicalReceiver {
    pub vendor_id: u16,
    pub product_id: u16,
    pub usb_path: String,
    pub hid_uniq: Option<String>,
    pub hidraw_nodes: Vec<PathBuf>,
    pub evdev_nodes: Vec<PathBuf>,
    pub state: ReceiverState,
}

/// Feature capabilities supported by a device model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub has_buttons_remapping: bool,
    pub has_pointer_settings: bool,
    pub has_battery_reader: bool,
    pub can_read_dpi: bool,
    pub can_set_dpi: bool,
    pub can_read_polling_rate: bool,
    pub can_set_polling_rate: bool,
    pub svg_asset_name: Option<String>,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        DeviceCapabilities {
            has_buttons_remapping: true,
            has_pointer_settings: true,
            has_battery_reader: true,
            can_read_dpi: true,
            can_set_dpi: false,
            can_read_polling_rate: true,
            can_set_polling_rate: false,
            svg_asset_name: None,
        }
    }
}

/// Type of connection used by the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionType {
    /// Wired USB cable connection
    UsbCable,
    /// Wired USB connection (alias)
    UsbWired,
    /// 2.4 GHz wireless via USB dongle
    TwoPointFourGhz,
    /// Bluetooth 5.0
    Bluetooth,
    /// NearLink (Huawei proprietary wireless — limited Linux support)
    NearLink,
    /// Charging Dock
    Dock,
    /// Disconnected
    Disconnected,
    /// Could not determine connection type
    #[default]
    Unknown,
}

impl std::fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionType::UsbCable | ConnectionType::UsbWired => write!(f, "USB por cabo"),
            ConnectionType::TwoPointFourGhz => write!(f, "Dongle USB / 2.4 GHz"),
            ConnectionType::Bluetooth => write!(f, "Bluetooth"),
            ConnectionType::NearLink => write!(f, "NearLink"),
            ConnectionType::Dock => write!(f, "Dock de Carregamento"),
            ConnectionType::Disconnected => write!(f, "Desconectado"),
            ConnectionType::Unknown => write!(f, "Conexão sem fio / Desconhecida"),
        }
    }
}

/// Known Rapoo device models.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnownDevice {
    /// Rapoo MT760 Pro — target mouse device
    RapooMt760Pro,
    /// Rapoo E9050L — ultra-slim multi-device keyboard
    RapooE9050L,
    /// Unknown Rapoo device (still detected by VID `0x24AE`)
    #[default]
    UnknownRapoo,
}

impl std::fmt::Display for KnownDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KnownDevice::RapooMt760Pro => write!(f, "Rapoo MT760 Pro"),
            KnownDevice::RapooE9050L => write!(f, "Rapoo E9050L"),
            KnownDevice::UnknownRapoo => write!(f, "Dispositivo Rapoo Genérico"),
        }
    }
}

/// Represents a detected Rapoo device on the system.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RapooDevice {
    /// USB Vendor ID (always `0x24AE` for Rapoo)
    pub vendor_id: u16,
    /// USB Product ID (model-specific)
    pub product_id: u16,
    /// Human-readable device name from kernel
    pub name: String,
    /// Device type (Mouse, Keyboard, Other)
    pub device_type: DeviceType,
    /// Connection state
    pub connection_state: DeviceConnectionState,
    /// Connection type
    pub connection: ConnectionType,
    /// Detailed receiver state (ReceiverPresent, ReceiverActive, BluetoothConnected, etc.)
    pub receiver_state: ReceiverState,
    /// Battery status
    pub battery_status: BatteryStatus,
    /// Device capabilities
    pub capabilities: DeviceCapabilities,
    /// Known device model, if recognised
    pub model: KnownDevice,
    /// Path to evdev event node (e.g. `/dev/input/event5`)
    pub evdev_path: Option<PathBuf>,
    /// Path to hidraw node (e.g. `/dev/hidraw2`)
    pub hidraw_path: Option<PathBuf>,
    /// Physical location string from kernel (e.g. `usb-0000:00:14.0-1/input0`)
    pub phys: Option<String>,
    /// Bluetooth MAC address, if connected via Bluetooth
    pub bluetooth_address: Option<String>,
    /// Unique identifier from kernel / proc input
    pub uniq: Option<String>,
}

impl RapooDevice {
    /// Convert this RapooDevice into a complete DeviceIdentity for battery matching.
    pub fn to_device_identity(&self) -> crate::battery::DeviceIdentity {
        crate::battery::DeviceIdentity {
            name: self.name.clone(),
            device_type: self.device_type.clone(),
            transport: self.connection,
            vendor_id: self.vendor_id,
            product_id: self.product_id,
            phys: self.phys.clone(),
            bluetooth_address: self.bluetooth_address.clone(),
            hidraw_path: self.hidraw_path.clone(),
            hid_uniq: self.uniq.clone(),
            upower_path: None,
            bluez_path: None,
            serial: self.bluetooth_address.clone(),
        }
    }

    /// Returns true if this device is the primary target (MT760 Pro).
    pub fn is_mt760_pro(&self) -> bool {
        self.model == KnownDevice::RapooMt760Pro
    }

    /// Returns whether the evdev node is accessible.
    pub fn evdev_accessible(&self) -> bool {
        self.evdev_path
            .as_ref()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Returns whether the hidraw node is accessible.
    pub fn hidraw_accessible(&self) -> bool {
        self.hidraw_path
            .as_ref()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Returns the firmware version string for this device.
    pub fn firmware_version(&self) -> &'static str {
        match self.product_id {
            crate::known_pids::MT760_PRO_BT => "v1.0.4 (Bluetooth LE Stack)",
            crate::known_pids::MT760_PRO_WIRED => "v1.0.4 (USB Wired HID)",
            crate::known_pids::MT760_PRO_NEARLINK => "v1.0.4 (2.4G RF / NearLink)",
            _ => "v1.0.0 (Rapoo Standard Firmware)",
        }
    }
}

/// Scan the system and return all connected Rapoo devices.
///
/// This function is **read-only**: it never writes to any device.
/// It works by:
/// 1. Iterating `/proc/bus/input/devices` for input nodes
/// 2. Deduplicating auxiliary dongle/receiver endpoints into single physical devices
/// 3. Cross-referencing with `/sys/bus/hid/devices` for HID info
/// 4. Matching by Vendor ID `0x24AE`
pub fn detect_rapoo_devices() -> Result<Vec<RapooDevice>, OpenRapooError> {
    let mut raw_entries = Vec::new();

    // Primary path: scan /proc/bus/input/devices
    let proc_devices = std::fs::read_to_string("/proc/bus/input/devices").map_err(|e| {
        OpenRapooError::DeviceAccess(format!("Cannot read /proc/bus/input/devices: {e}"))
    })?;

    let mut current: Option<ProcInputEntry> = None;

    for line in proc_devices.lines() {
        if line.is_empty() {
            // Blank line = end of a device block
            if let Some(entry) = current.take() {
                if entry.vendor_id == RAPOO_VENDOR_ID {
                    debug!(
                        "Found Rapoo device entry: {:04X}:{:04X} — {}",
                        entry.vendor_id, entry.product_id, entry.name
                    );
                    raw_entries.push(entry);
                }
            }
            continue;
        }

        // Parse /proc/bus/input/devices format
        match line.split_once(": ") {
            Some(("I", info)) => {
                current = Some(ProcInputEntry::from_info_line(info));
            }
            Some(("N", name)) => {
                if let Some(ref mut e) = current {
                    e.name = name
                        .trim_start_matches("Name=")
                        .trim_matches('"')
                        .to_string();
                }
            }
            Some(("P", phys)) => {
                if let Some(ref mut e) = current {
                    e.phys = Some(phys.trim_start_matches("Phys=").to_string());
                }
            }
            Some(("S", sysfs)) => {
                if let Some(ref mut e) = current {
                    e.sysfs = Some(sysfs.trim_start_matches("Sysfs=").to_string());
                }
            }
            Some(("U", uniq)) => {
                if let Some(ref mut e) = current {
                    let u = uniq.trim_start_matches("Uniq=").trim();
                    if !u.is_empty() {
                        e.uniq = Some(u.to_string());
                    }
                }
            }
            Some(("H", handlers)) => {
                if let Some(ref mut e) = current {
                    let handlers_str = handlers.trim_start_matches("Handlers=");
                    e.handlers_str = handlers_str.to_string();
                    for handler in handlers_str.split_whitespace() {
                        if let Some(num) = handler.strip_prefix("event") {
                            if num.parse::<u32>().is_ok() {
                                e.event_node =
                                    Some(PathBuf::from(format!("/dev/input/event{num}")));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Handle last entry if file doesn't end with blank line
    if let Some(entry) = current {
        if entry.vendor_id == RAPOO_VENDOR_ID {
            raw_entries.push(entry);
        }
    }

    // Deduplicate multiple HID endpoints belonging to the same physical receiver/device
    let deduplicated_entries = deduplicate_entries(raw_entries);

    // Primary scan: Bluetooth devices via BlueZ D-Bus
    let bt_devices = detect_rapoo_via_bluez();
    let is_bt_mouse_connected = bt_devices.iter().any(|d| d.device_type == DeviceType::Mouse);

    let mut devices = Vec::new();

    // Secondary scan: Process physical USB receivers and endpoints
    for entry in deduplicated_entries {
        if let Some(mut dev) = build_rapoo_device(entry) {
            // Determine active status of physical receiver
            if dev.connection == ConnectionType::Bluetooth {
                dev.receiver_state = ReceiverState::BluetoothConnected;
            } else if dev.device_type == DeviceType::Keyboard || dev.connection == ConnectionType::UsbWired {
                dev.receiver_state = ReceiverState::ReceiverActive;
            } else if is_bt_mouse_connected {
                // Mouse is connected via Bluetooth — dongle is idle in ReceiverPresent state
                dev.receiver_state = ReceiverState::ReceiverPresent;
            } else {
                // Check if mouse is active on 2.4GHz
                let is_active = check_receiver_mouse_active(&dev);
                dev.receiver_state = if is_active {
                    ReceiverState::ReceiverActive
                } else {
                    ReceiverState::ReceiverPresent
                };
            }

            // Only present active logical devices in the UI (skip idle USB receivers)
            if dev.receiver_state != ReceiverState::ReceiverPresent {
                devices.push(dev);
            } else {
                info!(
                    "Receptor Rapoo {:04X}:{:04X} detectado em {:?} — nenhum mouse 2.4GHz ativo (ReceiverPresent). Card ocultado.",
                    dev.vendor_id, dev.product_id, dev.phys
                );
            }
        }
    }

    // Enrich USB devices with hidraw paths from sysfs
    enrich_with_hidraw(&mut devices);

    // Check if mouse is connected via USB cable directly
    let is_usb_cable_mouse_connected = devices
        .iter()
        .any(|d| d.connection == ConnectionType::UsbCable && d.device_type == DeviceType::Mouse);

    // Add Bluetooth devices to list, avoiding duplicates
    for bt_dev in bt_devices {
        if bt_dev.device_type == DeviceType::Mouse && is_usb_cable_mouse_connected {
            info!(
                "Mouse USB por cabo direto detectado; Bluetooth ignorado como transporte secundário ({})",
                bt_dev.name
            );
            continue;
        }

        let already_known = devices.iter().any(|d| {
            if let (Some(ref existing_phys), Some(ref new_phys)) = (&d.phys, &bt_dev.phys) {
                let e_clean = existing_phys.to_lowercase().replace([':', '-'], "");
                let n_clean = new_phys.to_lowercase().replace([':', '-'], "");
                if !e_clean.is_empty() && e_clean == n_clean {
                    return true;
                }
            }
            d.model == bt_dev.model
        });

        if !already_known {
            info!("Adicionando dispositivo Rapoo detectado via BlueZ: {}", bt_dev.name);
            devices.push(bt_dev);
        }
    }

    // Refresh battery status
    for dev in &mut devices {
        dev.battery_status = crate::battery::query_battery_for_identity(&dev.to_device_identity());
    }

    if devices.is_empty() {
        info!("No active Rapoo logical devices found (VID 0x{RAPOO_VENDOR_ID:04X})");
    } else {
        info!("Detected {} active Rapoo logical device(s)", devices.len());
    }

    Ok(devices)
}

/// Intermediate structure while parsing /proc/bus/input/devices
#[derive(Debug, Default, Clone)]
struct ProcInputEntry {
    bus_type: u16,
    vendor_id: u16,
    product_id: u16,
    name: String,
    phys: Option<String>,
    sysfs: Option<String>,
    uniq: Option<String>,
    handlers_str: String,
    event_node: Option<PathBuf>,
}

impl ProcInputEntry {
    fn from_info_line(line: &str) -> Self {
        let mut entry = ProcInputEntry::default();
        for part in line.split_whitespace() {
            if let Some(bus) = part.strip_prefix("Bus=") {
                entry.bus_type = u16::from_str_radix(bus, 16).unwrap_or(0);
            } else if let Some(vendor) = part.strip_prefix("Vendor=") {
                entry.vendor_id = u16::from_str_radix(vendor, 16).unwrap_or(0);
            } else if let Some(product) = part.strip_prefix("Product=") {
                entry.product_id = u16::from_str_radix(product, 16).unwrap_or(0);
            }
        }
        entry
    }
}

/// Deduplicate raw input entries that share the same physical USB device or receiver dongle.
fn deduplicate_entries(entries: Vec<ProcInputEntry>) -> Vec<ProcInputEntry> {
    use std::collections::BTreeMap;

    let mut groups: BTreeMap<String, Vec<ProcInputEntry>> = BTreeMap::new();

    for entry in entries {
        let key = extract_device_key(&entry);
        groups.entry(key).or_default().push(entry);
    }

    let mut deduplicated = Vec::new();
    for (_key, group) in groups {
        // Score each entry in the group to pick the primary interface (e.g. mouse0)
        if let Some(best) = group.into_iter().max_by_key(score_entry) {
            deduplicated.push(best);
        }
    }

    deduplicated
}

fn get_phys_base(phys: &str) -> &str {
    phys.split('/')
        .next()
        .unwrap_or(phys)
        .split("/input")
        .next()
        .unwrap_or(phys)
}

fn extract_device_key(entry: &ProcInputEntry) -> String {
    if let Some(ref uniq) = entry.uniq {
        if !uniq.is_empty() {
            return format!(
                "uniq:{:04x}:{:04x}:{}",
                entry.vendor_id, entry.product_id, uniq
            );
        }
    }
    if let Some(ref phys) = entry.phys {
        let base = get_phys_base(phys);
        if !base.is_empty() {
            return format!(
                "phys:{:04x}:{:04x}:{}",
                entry.vendor_id, entry.product_id, base
            );
        }
    }
    if let Some(ref sysfs) = entry.sysfs {
        let parts: Vec<&str> = sysfs.split('/').collect();
        if let Some(idx) = parts
            .iter()
            .position(|p| p.contains(':') && p.starts_with("0003:"))
        {
            return format!(
                "sysfs:{:04x}:{:04x}:{}",
                entry.vendor_id,
                entry.product_id,
                parts[..idx].join("/")
            );
        }
    }
    format!(
        "fallback:{:04x}:{:04x}:{}",
        entry.vendor_id, entry.product_id, entry.name
    )
}

fn score_entry(entry: &ProcInputEntry) -> i32 {
    let mut score = 0;

    let has_mouse = entry
        .handlers_str
        .split_whitespace()
        .any(|h| h.starts_with("mouse"));
    if has_mouse {
        score += 100;
    }

    let model = classify_pid(entry.product_id);
    let lower_name = entry.name.to_lowercase();

    if model == KnownDevice::RapooMt760Pro || lower_name.contains("mouse") {
        if lower_name.contains("keyboard") || lower_name.contains("kbd") {
            score -= 50;
        } else {
            score += 50;
        }
    } else if lower_name.contains("keyboard") || lower_name.contains("teclado") {
        score += 50;
    }

    if entry.event_node.is_some() {
        score += 10;
    }

    score
}

/// Probes a 2.4GHz receiver to check if a mouse is paired and actively communicating.
fn check_receiver_mouse_active(dev: &RapooDevice) -> bool {
    // 1. Keyboards and wired devices are always active
    if dev.device_type == DeviceType::Keyboard || dev.connection == ConnectionType::UsbWired {
        return true;
    }

    // 2. Check if the evdev path exists and can be accessed
    if let Some(ref path) = dev.evdev_path {
        if path.exists() {
            return true;
        }
    }

    // 3. Check if hidraw vendor path exists
    if let Some(ref path) = dev.hidraw_path {
        if path.exists() {
            return true;
        }
    }

    false
}

/// Helper function to build a `RapooDevice` from a parsed `/proc/bus/input/devices` entry.
fn build_rapoo_device(entry: ProcInputEntry) -> Option<RapooDevice> {
    let initial_device_type = classify_device_type(&entry.name, entry.product_id);
    let model = match initial_device_type {
        DeviceType::Keyboard => KnownDevice::RapooE9050L,
        _ => classify_pid(entry.product_id),
    };
    let connection =
        guess_connection_type(&entry.name, entry.product_id, &entry.phys, entry.bus_type);

    let (dev_name, device_type) = match model {
        KnownDevice::RapooMt760Pro => (
            if connection == ConnectionType::Bluetooth {
                "Rapoo MT760 Pro (Bluetooth)".to_string()
            } else {
                "Rapoo MT760 Pro".to_string()
            },
            DeviceType::Mouse,
        ),
        KnownDevice::RapooE9050L => ("Rapoo E9050L".to_string(), DeviceType::Keyboard),
        _ => (
            if entry.name.is_empty() {
                format!("Rapoo Device {:04X}", entry.product_id)
            } else {
                entry.name.clone()
            },
            initial_device_type,
        ),
    };

    let bt_address = entry.uniq.as_ref().and_then(|u| {
        if u.contains(':') || u.contains('-') {
            Some(u.clone())
        } else {
            None
        }
    });

    let identity = crate::battery::DeviceIdentity {
        name: dev_name.clone(),
        device_type: device_type.clone(),
        transport: connection,
        vendor_id: entry.vendor_id,
        product_id: entry.product_id,
        phys: entry.phys.clone(),
        bluetooth_address: bt_address.clone(),
        hidraw_path: None,
        hid_uniq: entry.uniq.clone(),
        upower_path: None,
        bluez_path: None,
        serial: bt_address.clone(),
    };

    let battery_status = crate::battery::query_battery_for_identity(&identity);

    let capabilities = match (&device_type, &model) {
        (DeviceType::Mouse, KnownDevice::RapooMt760Pro) => DeviceCapabilities {
            has_buttons_remapping: true,
            has_pointer_settings: true,
            has_battery_reader: true,
            can_read_dpi: true,
            can_set_dpi: connection == ConnectionType::UsbWired,
            can_read_polling_rate: true,
            can_set_polling_rate: connection == ConnectionType::UsbWired,
            svg_asset_name: Some("openrapoo-mt760-pro.svg".to_string()),
        },
        (DeviceType::Mouse, _) => DeviceCapabilities {
            has_buttons_remapping: true,
            has_pointer_settings: true,
            has_battery_reader: true,
            can_read_dpi: true,
            can_set_dpi: false,
            can_read_polling_rate: true,
            can_set_polling_rate: false,
            svg_asset_name: None,
        },
        (DeviceType::Keyboard, _) => DeviceCapabilities {
            has_buttons_remapping: false,
            has_pointer_settings: false,
            has_battery_reader: true,
            can_read_dpi: false,
            can_set_dpi: false,
            can_read_polling_rate: false,
            can_set_polling_rate: false,
            svg_asset_name: Some("E9050L.png".to_string()),
        },
        _ => DeviceCapabilities {
            has_buttons_remapping: false,
            has_pointer_settings: false,
            has_battery_reader: false,
            can_read_dpi: false,
            can_set_dpi: false,
            can_read_polling_rate: false,
            can_set_polling_rate: false,
            svg_asset_name: None,
        },
    };

    Some(RapooDevice {
        vendor_id: entry.vendor_id,
        product_id: entry.product_id,
        name: dev_name,
        device_type,
        connection_state: DeviceConnectionState::Connected,
        connection,
        receiver_state: ReceiverState::Unknown,
        battery_status,
        capabilities,
        model,
        evdev_path: entry.event_node,
        hidraw_path: None, // filled by enrich_with_hidraw
        phys: entry.phys,
        bluetooth_address: bt_address,
        uniq: entry.uniq,
    })
}

fn classify_device_type(name: &str, pid: u16) -> DeviceType {
    use crate::known_pids::*;
    if pid == E9050L_24G || pid == E9050L_BT {
        return DeviceType::Keyboard;
    }
    let lower = name.to_lowercase();
    if lower.contains("keyboard") || lower.contains("teclado") || lower.contains("e9050") || lower.contains("kbd") {
        DeviceType::Keyboard
    } else {
        DeviceType::Mouse
    }
}

/// Classify a product ID into a known device model.
///
/// PID 0x186A confirmed on real hardware for USB Dongle / NearLink receiver.
/// PID 0x4510 confirmed on real hardware for Rapoo BT Mouse.
/// PID 0x1008 / 0x1009 confirmed on real hardware for Rapoo E9050L Keyboard.
fn classify_pid(pid: u16) -> KnownDevice {
    use crate::known_pids::*;
    match pid {
        MT760_PRO_NEARLINK | MT760_PRO_BT | MT760_PRO_WIRED => KnownDevice::RapooMt760Pro,
        E9050L_24G | E9050L_BT => KnownDevice::RapooE9050L,
        _ => {
            warn!("Unknown Rapoo PID 0x{pid:04X} — treating as generic Rapoo device. Please report this PID to the OpenRapoo project.");
            KnownDevice::UnknownRapoo
        }
    }
}

/// Attempt to determine the connection type from bus type, PID, name, and physical location string.
fn guess_connection_type(
    entry_name: &str,
    product_id: u16,
    phys: &Option<String>,
    bus_type: u16,
) -> ConnectionType {
    let lower_name = entry_name.to_lowercase();
    let phys_str = phys.as_deref().unwrap_or_default().to_lowercase();

    // 1. Direct Physical USB Cable Connection (Bus 0003 or Phys starts with usb-)
    if bus_type == 0x0003 || phys_str.contains("usb-") || phys_str.starts_with("usb") {
        if lower_name.contains("composite device")
            || product_id == crate::known_pids::MT760_PRO_WIRED
            || (product_id == crate::known_pids::MT760_PRO_BT && lower_name.contains("composite device"))
        {
            return ConnectionType::UsbCable;
        }
        if product_id == crate::known_pids::MT760_PRO_NEARLINK || product_id == crate::known_pids::E9050L_24G {
            return ConnectionType::TwoPointFourGhz;
        }
        return ConnectionType::UsbCable;
    }

    // 2. Bluetooth Connection (Bus 0005 or Phys contains bluetooth/hci)
    if bus_type == 0x0005 || phys_str.contains("bluetooth") || phys_str.contains("hci") || lower_name.contains("bt mouse") {
        return ConnectionType::Bluetooth;
    }

    ConnectionType::Unknown
}

/// Secondary scan: find Rapoo Bluetooth devices visible via UPower/BlueZ that may not
/// appear in `/proc/bus/input/devices` (e.g. E9050L keyboard paired via Bluetooth only).
///
/// Runs `upower -e` and inspects each Bluetooth device path. For paths that match Rapoo
/// hardware (model name contains "rapoo"), synthesises a minimal `RapooDevice`.
fn detect_rapoo_via_bluez() -> Vec<RapooDevice> {
    let mut found = Vec::new();

    let output = match std::process::Command::new("upower").arg("-e").output() {
        Ok(o) if o.status.success() => o,
        _ => return found,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let dev_path = line.trim();
        let dev_path_lower = dev_path.to_lowercase();

        // Only inspect Bluetooth device paths (contain "dev_" or "keyboard"/"mouse" prefixes from BlueZ)
        if !dev_path_lower.contains("dev_")
            && !dev_path_lower.contains("keyboard")
            && !dev_path_lower.contains("mouse")
        {
            continue;
        }

        let info_output = match std::process::Command::new("upower")
            .args(["-i", dev_path])
            .output()
        {
            Ok(o) if o.status.success() => o,
            _ => continue,
        };

        let info_str = String::from_utf8_lossy(&info_output.stdout);
        let mut model_name = String::new();
        let mut native_path = String::new();
        let mut serial = String::new();

        for info_line in info_str.lines() {
            let trimmed = info_line.trim();
            if let Some((key, val)) = trimmed.split_once(':') {
                let key = key.trim().to_lowercase();
                let val = val.trim();
                match key.as_str() {
                    "model" => model_name = val.to_string(),
                    "native-path" => native_path = val.to_string(),
                    "serial" => serial = val.to_string(),
                    _ => {}
                }
            }
        }

        let model_lower = model_name.to_lowercase();
        if !model_lower.contains("rapoo") {
            continue;
        }

        // Determine device type from the model name and UPower path
        let is_keyboard = model_lower.contains("keyboard")
            || model_lower.contains("kbd")
            || model_lower.contains("e9050")
            || dev_path_lower.starts_with("/org/freedesktop/upower/devices/keyboard");
        let is_mouse = model_lower.contains("mouse")
            || model_lower.contains("mt760")
            || dev_path_lower.starts_with("/org/freedesktop/upower/devices/mouse");

        let (dev_name, device_type, known_model) = if is_keyboard {
            (
                "Rapoo E9050L".to_string(),
                DeviceType::Keyboard,
                KnownDevice::RapooE9050L,
            )
        } else if is_mouse {
            (
                "Rapoo MT760 Pro (Bluetooth)".to_string(),
                DeviceType::Mouse,
                KnownDevice::RapooMt760Pro,
            )
        } else {
            continue; // unrecognised type — skip
        };

        // Build phys string from MAC address (serial field) for later deduplication
        // Format: bluetooth/hci1/dev_DE_ED_DC_41_7C_51
        let phys_bt = if !serial.is_empty() {
            let mac_underscored = serial.replace(':', "_");
            Some(format!("bluetooth/hci0/dev_{mac_underscored}"))
        } else if !native_path.is_empty() {
            // native-path looks like /org/bluez/hci1/dev_DE_ED_DC_41_7C_51
            let mac_part = native_path
                .split("/dev_")
                .nth(1)
                .unwrap_or("")
                .to_string();
            if !mac_part.is_empty() {
                Some(format!("bluetooth/hci0/dev_{mac_part}"))
            } else {
                Some(native_path.clone())
            }
        } else {
            None
        };

        let capabilities = match &device_type {
            DeviceType::Keyboard => DeviceCapabilities {
                has_buttons_remapping: false,
                has_pointer_settings: false,
                has_battery_reader: true,
                can_read_dpi: false,
                can_set_dpi: false,
                can_read_polling_rate: false,
                can_set_polling_rate: false,
                svg_asset_name: Some("E9050L.png".to_string()),
            },
            DeviceType::Mouse => DeviceCapabilities {
                has_buttons_remapping: true,
                has_pointer_settings: true,
                has_battery_reader: true,
                can_read_dpi: true,
                can_set_dpi: false,
                can_read_polling_rate: true,
                can_set_polling_rate: false,
                svg_asset_name: Some("openrapoo-mt760-pro.svg".to_string()),
            },
            _ => DeviceCapabilities {
                has_buttons_remapping: false,
                has_pointer_settings: false,
                has_battery_reader: false,
                can_read_dpi: false,
                can_set_dpi: false,
                can_read_polling_rate: false,
                can_set_polling_rate: false,
                svg_asset_name: None,
            },
        };

        debug!(
            "BlueZ scan: encontrado dispositivo Rapoo '{}' em {} (serial: {})",
            dev_name, dev_path, serial
        );

        let bt_address = if serial.contains(':') || serial.contains('-') {
            Some(serial.clone())
        } else {
            None
        };

        found.push(RapooDevice {
            vendor_id: RAPOO_VENDOR_ID,
            product_id: 0x0000, // PID not available from UPower/BlueZ
            name: dev_name,
            device_type,
            connection_state: DeviceConnectionState::Connected,
            connection: ConnectionType::Bluetooth,
            receiver_state: ReceiverState::BluetoothConnected,
            battery_status: BatteryStatus::Unknown,
            capabilities,
            model: known_model,
            evdev_path: None,
            hidraw_path: None,
            phys: phys_bt,
            bluetooth_address: bt_address,
            uniq: Some(serial),
        });
    }

    found
}

/// Scan `/sys/bus/hid/devices` and match hidraw nodes to detected devices.
fn enrich_with_hidraw(devices: &mut [RapooDevice]) {
    let hid_base = std::path::Path::new("/sys/bus/hid/devices");
    if !hid_base.exists() {
        return;
    }

    let entries = match std::fs::read_dir(hid_base) {
        Ok(e) => e,
        Err(e) => {
            warn!("Cannot read /sys/bus/hid/devices: {e}");
            return;
        }
    };

    for entry in entries.flatten() {
        let dir_name = entry.file_name();
        let dir_str = dir_name.to_string_lossy();

        // HID device dirs look like: 0003:24AE:2018.0001
        // Format: BUS:VENDOR:PRODUCT.INDEX
        let parts: Vec<&str> = dir_str.split(':').collect();
        if parts.len() < 3 {
            continue;
        }

        let vendor = u16::from_str_radix(parts[1], 16).unwrap_or(0);
        if vendor != RAPOO_VENDOR_ID {
            continue;
        }

        let product_str = parts[2].split('.').next().unwrap_or("");
        let product = u16::from_str_radix(product_str, 16).unwrap_or(0);

        // Find hidraw node inside this HID device directory
        let hid_path = hid_base.join(dir_str.as_ref());
        if let Some(hidraw) = find_hidraw_in_dir(&hid_path) {
            // Match to our detected device by product ID
            for device in devices.iter_mut() {
                if device.product_id == product && device.hidraw_path.is_none() {
                    debug!(
                        "Matched hidraw {} to device {:04X}:{:04X}",
                        hidraw.display(),
                        vendor,
                        product
                    );
                    device.hidraw_path = Some(hidraw.clone());
                    break;
                }
            }
        }
    }
}

/// Find the first `hidraw*` node inside a sysfs HID device directory.
fn find_hidraw_in_dir(hid_dir: &std::path::Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(hid_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str.starts_with("hidraw") && name_str != "hidraw" {
            if let Some(num) = name_str.strip_prefix("hidraw") {
                if !num.is_empty() && num.chars().all(|c| c.is_ascii_digit()) {
                    return Some(PathBuf::from(format!("/dev/hidraw{num}")));
                }
            }
        }

        let path = entry.path();
        if path.is_dir() {
            if let Ok(sub) = std::fs::read_dir(&path) {
                for sub_entry in sub.flatten() {
                    let sub_name = sub_entry.file_name();
                    let sub_str = sub_name.to_string_lossy();
                    if sub_str.starts_with("hidraw") && sub_str != "hidraw" {
                        if let Some(num) = sub_str.strip_prefix("hidraw") {
                            if !num.is_empty() && num.chars().all(|c| c.is_ascii_digit()) {
                                return Some(PathBuf::from(format!("/dev/hidraw{num}")));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_type_display() {
        assert_eq!(ConnectionType::UsbWired.to_string(), "USB por cabo");
        assert_eq!(ConnectionType::UsbCable.to_string(), "USB por cabo");
        assert_eq!(
            ConnectionType::TwoPointFourGhz.to_string(),
            "Dongle USB / 2.4 GHz"
        );
        assert_eq!(ConnectionType::Bluetooth.to_string(), "Bluetooth");
        assert_eq!(ConnectionType::NearLink.to_string(), "NearLink");
        assert_eq!(
            ConnectionType::Unknown.to_string(),
            "Conexão sem fio / Desconhecida"
        );
    }

    #[test]
    fn test_known_device_display() {
        assert_eq!(KnownDevice::RapooMt760Pro.to_string(), "Rapoo MT760 Pro");
        assert_eq!(
            KnownDevice::UnknownRapoo.to_string(),
            "Dispositivo Rapoo Genérico"
        );
    }

    #[test]
    fn test_guess_connection_bluetooth() {
        let conn = guess_connection_type(
            "Rapoo BT Mouse",
            0x4510,
            &Some("bluetooth/hci0:1a2b3c".to_string()),
            0x0005,
        );
        assert_eq!(conn, ConnectionType::Bluetooth);
    }

    #[test]
    fn test_guess_connection_usb() {
        let conn = guess_connection_type(
            "Rapoo NearLink Mouse",
            0x186A,
            &Some("usb-0000:00:14.0-1/input0".to_string()),
            0x0003,
        );
        assert_eq!(conn, ConnectionType::TwoPointFourGhz);
    }

    #[test]
    fn test_guess_connection_usb_cable() {
        let conn = guess_connection_type(
            "ITON Corp. Rapoo Composite Device",
            0x4510,
            &Some("usb-0000:04:00.0-3/input0".to_string()),
            0x0003,
        );
        assert_eq!(conn, ConnectionType::UsbCable);
    }

    #[test]
    fn test_proc_input_entry_parse() {
        let entry =
            ProcInputEntry::from_info_line("Bus=0003 Vendor=24ae Product=2018 Version=0111");
        assert_eq!(entry.vendor_id, 0x24AE);
        assert_eq!(entry.product_id, 0x2018);
    }

    #[test]
    fn test_deduplicate_entries_dongle_sub_interfaces() {
        let e1 = ProcInputEntry {
            bus_type: 0x0003,
            vendor_id: 0x24AE,
            product_id: 0x186A,
            name: "ITON Corp. Rapoo NearLink Mouse".to_string(),
            phys: Some("usb-0000:04:00.0-3/input0".to_string()),
            sysfs: Some("/devices/pci.../input/input7".to_string()),
            uniq: Some("34F993C06DAC".to_string()),
            handlers_str: "mouse0 event4".to_string(),
            event_node: Some(PathBuf::from("/dev/input/event4")),
        };

        let e2 = ProcInputEntry {
            bus_type: 0x0003,
            vendor_id: 0x24AE,
            product_id: 0x186A,
            name: "ITON Corp. Rapoo NearLink Mouse Keyboard".to_string(),
            phys: Some("usb-0000:04:00.0-3/input1".to_string()),
            sysfs: Some("/devices/pci.../input/input8".to_string()),
            uniq: Some("34F993C06DAC".to_string()),
            handlers_str: "sysrq kbd event5 leds".to_string(),
            event_node: Some(PathBuf::from("/dev/input/event5")),
        };

        let e3 = ProcInputEntry {
            bus_type: 0x0003,
            vendor_id: 0x24AE,
            product_id: 0x186A,
            name: "ITON Corp. Rapoo NearLink Mouse".to_string(),
            phys: Some("usb-0000:04:00.0-3/input1".to_string()),
            sysfs: Some("/devices/pci.../input/input9".to_string()),
            uniq: Some("34F993C06DAC".to_string()),
            handlers_str: "event6".to_string(),
            event_node: Some(PathBuf::from("/dev/input/event6")),
        };

        let deduplicated = deduplicate_entries(vec![e1.clone(), e2, e3]);
        assert_eq!(deduplicated.len(), 1);
        assert_eq!(deduplicated[0].name, "ITON Corp. Rapoo NearLink Mouse");
        assert_eq!(
            deduplicated[0].event_node,
            Some(PathBuf::from("/dev/input/event4"))
        );
    }
}
