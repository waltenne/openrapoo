//! Trait and identity types for battery providers.

use super::types::{BatteryReading, BatterySource};
use crate::device::{ConnectionType, DeviceType};
use std::path::PathBuf;

/// Identity information passed to battery providers for matching devices.
#[derive(Debug, Clone, Default)]
pub struct DeviceIdentity {
    pub name: String,
    pub device_type: DeviceType,
    pub transport: ConnectionType,
    pub vendor_id: u16,
    pub product_id: u16,
    pub phys: Option<String>,
    pub bluetooth_address: Option<String>,
    pub hidraw_path: Option<PathBuf>,
    pub hid_uniq: Option<String>,
    pub upower_path: Option<String>,
    pub bluez_path: Option<String>,
    pub serial: Option<String>,
}

impl DeviceIdentity {
    /// Attempts to deduce the active connection transport from device attributes if Unknown.
    pub fn resolved_transport(&self) -> ConnectionType {
        if self.transport != ConnectionType::Unknown {
            return self.transport;
        }

        let name_lower = self.name.to_lowercase();
        let phys_lower = self.phys.as_deref().unwrap_or_default().to_lowercase();

        if phys_lower.contains("bluetooth")
            || phys_lower.contains("hci")
            || self.bluetooth_address.is_some()
            || name_lower.contains("(bluetooth)")
            || name_lower.contains("bt mouse")
        {
            return ConnectionType::Bluetooth;
        }

        if phys_lower.contains("usb-") || phys_lower.starts_with("usb") {
            if name_lower.contains("composite device") || name_lower.contains("wired") {
                return ConnectionType::UsbCable;
            }
            if name_lower.contains("nearlink")
                || name_lower.contains("dongle")
                || name_lower.contains("2.4g")
            {
                return ConnectionType::TwoPointFourGhz;
            }
        }

        ConnectionType::Unknown
    }
}

/// Abstract battery provider trait implemented by UPower, BlueZ, GATT, sysfs, and HID readers.
pub trait BatteryProvider: Send + Sync {
    /// Human-readable provider name for logging and diagnostics.
    fn name(&self) -> &str;

    /// Telemetry source type.
    fn source(&self) -> BatterySource;

    /// Provider priority: lower numbers represent higher priority (1=highest, 7=lowest).
    fn priority(&self) -> u8;

    /// Query battery status for the specified device identity.
    fn query(&self, device: &DeviceIdentity, log: &mut Vec<String>) -> Option<BatteryReading>;

    /// Returns true if this provider is supported and accessible in the current environment.
    fn is_available(&self) -> bool;
}

/// Get current epoch time in seconds.
pub fn current_epoch_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
