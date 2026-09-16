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

/// Type of connection used by the device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionType {
    /// Wired USB-C connection
    UsbWired,
    /// 2.4 GHz wireless via USB dongle
    TwoPointFourGhz,
    /// Bluetooth 5.0
    Bluetooth,
    /// NearLink (Huawei proprietary wireless — limited Linux support)
    NearLink,
    /// Could not determine connection type
    Unknown,
}

impl std::fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionType::UsbWired => write!(f, "USB (wired)"),
            ConnectionType::TwoPointFourGhz => write!(f, "2.4 GHz wireless"),
            ConnectionType::Bluetooth => write!(f, "Bluetooth 5.0"),
            ConnectionType::NearLink => write!(f, "NearLink"),
            ConnectionType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Known Rapoo device models.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnownDevice {
    /// Rapoo MT760 Pro — target device for this project
    RapooMt760Pro,
    /// Unknown Rapoo device (still detected by VID `0x24AE`)
    UnknownRapoo,
}

impl std::fmt::Display for KnownDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KnownDevice::RapooMt760Pro => write!(f, "Rapoo MT760 Pro"),
            KnownDevice::UnknownRapoo => write!(f, "Unknown Rapoo device"),
        }
    }
}

/// Represents a detected Rapoo device on the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RapooDevice {
    /// USB Vendor ID (always `0x24AE` for Rapoo)
    pub vendor_id: u16,
    /// USB Product ID (model-specific)
    pub product_id: u16,
    /// Human-readable device name from kernel
    pub name: String,
    /// Connection type
    pub connection: ConnectionType,
    /// Known device model, if recognised
    pub model: KnownDevice,
    /// Path to evdev event node (e.g. `/dev/input/event5`)
    pub evdev_path: Option<PathBuf>,
    /// Path to hidraw node (e.g. `/dev/hidraw2`)
    pub hidraw_path: Option<PathBuf>,
    /// Physical location string from kernel (e.g. `usb-0000:00:14.0-1/input0`)
    pub phys: Option<String>,
}

impl RapooDevice {
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
}

/// Scan the system and return all connected Rapoo devices.
///
/// This function is **read-only**: it never writes to any device.
/// It works by:
/// 1. Iterating `/proc/bus/input/devices` for input nodes
/// 2. Cross-referencing with `/sys/bus/hid/devices` for HID info
/// 3. Matching by Vendor ID `0x24AE`
pub fn detect_rapoo_devices() -> Result<Vec<RapooDevice>, OpenRapooError> {
    let mut devices = Vec::new();

    // Primary path: scan /proc/bus/input/devices
    let proc_devices = std::fs::read_to_string("/proc/bus/input/devices")
        .map_err(|e| OpenRapooError::DeviceAccess(format!("Cannot read /proc/bus/input/devices: {e}")))?;

    let mut current: Option<ProcInputEntry> = None;

    for line in proc_devices.lines() {
        if line.is_empty() {
            // Blank line = end of a device block
            if let Some(entry) = current.take() {
                if entry.vendor_id == RAPOO_VENDOR_ID {
                    debug!("Found Rapoo device: {:04X}:{:04X} — {}", entry.vendor_id, entry.product_id, entry.name);
                    if let Some(device) = build_rapoo_device(entry) {
                        devices.push(device);
                    }
                }
            }
            continue;
        }

        // Parse /proc/bus/input/devices format
        // Lines look like:
        //   I: Bus=0003 Vendor=24ae Product=2018 Version=0111
        //   N: Name="Rapoo MT760 Pro"
        //   P: Phys=usb-0000:00:14.0-1/input0
        //   H: Handlers=mouse0 event5
        match line.split_once(": ") {
            Some(("I", info)) => {
                current = Some(ProcInputEntry::from_info_line(info));
            }
            Some(("N", name)) => {
                if let Some(ref mut e) = current {
                    e.name = name.trim_start_matches("Name=").trim_matches('"').to_string();
                }
            }
            Some(("P", phys)) => {
                if let Some(ref mut e) = current {
                    e.phys = Some(phys.trim_start_matches("Phys=").to_string());
                }
            }
            Some(("H", handlers)) => {
                if let Some(ref mut e) = current {
                    let handlers_str = handlers.trim_start_matches("Handlers=");
                    for handler in handlers_str.split_whitespace() {
                        if let Some(num) = handler.strip_prefix("event") {
                            if num.parse::<u32>().is_ok() {
                                e.event_node = Some(PathBuf::from(format!("/dev/input/event{num}")));
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
            if let Some(device) = build_rapoo_device(entry) {
                devices.push(device);
            }
        }
    }

    // Enrich with hidraw paths from sysfs
    enrich_with_hidraw(&mut devices);

    if devices.is_empty() {
        info!("No Rapoo devices found (VID 0x{RAPOO_VENDOR_ID:04X})");
    } else {
        info!("Detected {} Rapoo device(s)", devices.len());
    }

    Ok(devices)
}

/// Intermediate structure while parsing /proc/bus/input/devices
#[derive(Debug, Default)]
struct ProcInputEntry {
    vendor_id: u16,
    product_id: u16,
    name: String,
    phys: Option<String>,
    event_node: Option<PathBuf>,
}

impl ProcInputEntry {
    fn from_info_line(line: &str) -> Self {
        let mut entry = ProcInputEntry::default();
        for part in line.split_whitespace() {
            if let Some(vendor) = part.strip_prefix("Vendor=") {
                entry.vendor_id = u16::from_str_radix(vendor, 16).unwrap_or(0);
            } else if let Some(product) = part.strip_prefix("Product=") {
                entry.product_id = u16::from_str_radix(product, 16).unwrap_or(0);
            }
        }
        entry
    }
}

/// Build a `RapooDevice` from a parsed `/proc/bus/input/devices` entry.
fn build_rapoo_device(entry: ProcInputEntry) -> Option<RapooDevice> {
    let model = classify_pid(entry.product_id);
    let connection = guess_connection_type(&entry.phys, &model);

    Some(RapooDevice {
        vendor_id: entry.vendor_id,
        product_id: entry.product_id,
        name: if entry.name.is_empty() {
            format!("Rapoo Device {:04X}", entry.product_id)
        } else {
            entry.name
        },
        connection,
        model,
        evdev_path: entry.event_node,
        hidraw_path: None, // filled by enrich_with_hidraw
        phys: entry.phys,
    })
}

/// Classify a product ID into a known device model.
///
/// PID 0x186A confirmed on real hardware: NearLink/USB receiver exposes
/// 3 HID interfaces (Mouse + Keyboard + Mouse) all under the same PID.
fn classify_pid(pid: u16) -> KnownDevice {
    use crate::known_pids::*;
    match pid {
        MT760_PRO_NEARLINK | MT760_PRO_BT | MT760_PRO_WIRED => KnownDevice::RapooMt760Pro,
        _ => {
            warn!("Unknown Rapoo PID 0x{pid:04X} — treating as generic Rapoo device. Please report this PID to the OpenRapoo project.");
            KnownDevice::UnknownRapoo
        }
    }
}

/// Attempt to determine the connection type from the physical location string.
fn guess_connection_type(phys: &Option<String>, _model: &KnownDevice) -> ConnectionType {
    match phys.as_deref() {
        Some(p) if p.contains("bluetooth") || p.contains("hci") => ConnectionType::Bluetooth,
        Some(p) if p.contains("usb") => {
            // Can't reliably distinguish USB-C wired from 2.4 GHz USB dongle
            // without reading the HID descriptor. Default to 2.4 GHz as it's
            // more common for wireless Rapoo mice.
            ConnectionType::TwoPointFourGhz
        }
        _ => ConnectionType::Unknown,
    }
}

/// Scan `/sys/bus/hid/devices` and match hidraw nodes to detected devices.
fn enrich_with_hidraw(devices: &mut Vec<RapooDevice>) {
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
                    debug!("Matched hidraw {} to device {:04X}:{:04X}", hidraw.display(), vendor, product);
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
        if name_str.starts_with("hidraw") {
            // The entry is a symlink/directory; the actual device is /dev/hidraw<N>
            if let Some(num) = name_str.strip_prefix("hidraw") {
                return Some(PathBuf::from(format!("/dev/hidraw{num}")));
            }
        }
        // Also check one level deeper for hidraw subdirectory
        if name_str.starts_with("hidraw") {
            let inner = entry.path();
            if inner.is_dir() {
                if let Ok(sub) = std::fs::read_dir(&inner) {
                    for sub_entry in sub.flatten() {
                        let sub_name = sub_entry.file_name();
                        let sub_str = sub_name.to_string_lossy();
                        if sub_str.starts_with("hidraw") {
                            if let Some(num) = sub_str.strip_prefix("hidraw") {
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
        assert_eq!(ConnectionType::UsbWired.to_string(), "USB (wired)");
        assert_eq!(ConnectionType::TwoPointFourGhz.to_string(), "2.4 GHz wireless");
        assert_eq!(ConnectionType::Bluetooth.to_string(), "Bluetooth 5.0");
        assert_eq!(ConnectionType::NearLink.to_string(), "NearLink");
        assert_eq!(ConnectionType::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_known_device_display() {
        assert_eq!(KnownDevice::RapooMt760Pro.to_string(), "Rapoo MT760 Pro");
        assert_eq!(KnownDevice::UnknownRapoo.to_string(), "Unknown Rapoo device");
    }

    #[test]
    fn test_guess_connection_bluetooth() {
        let conn = guess_connection_type(
            &Some("bluetooth/hci0:1a2b3c".to_string()),
            &KnownDevice::RapooMt760Pro,
        );
        assert_eq!(conn, ConnectionType::Bluetooth);
    }

    #[test]
    fn test_guess_connection_usb() {
        let conn = guess_connection_type(
            &Some("usb-0000:00:14.0-1/input0".to_string()),
            &KnownDevice::RapooMt760Pro,
        );
        assert_eq!(conn, ConnectionType::TwoPointFourGhz);
    }

    #[test]
    fn test_proc_input_entry_parse() {
        let entry = ProcInputEntry::from_info_line("Bus=0003 Vendor=24ae Product=2018 Version=0111");
        assert_eq!(entry.vendor_id, 0x24AE);
        assert_eq!(entry.product_id, 0x2018);
    }
}
