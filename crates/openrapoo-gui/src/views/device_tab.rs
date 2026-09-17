//! "Dispositivo" (Device) Tab View Component.

use openrapoo_core::device::{detect_rapoo_devices, RapooDevice};
use openrapoo_core::permissions::{check_input_group_status, GroupMembershipState};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DeviceTabInfo {
    pub devices: Vec<RapooDevice>,
    pub manufacturer: &'static str,
    pub model: &'static str,
    pub vendor_id_hex: &'static str,
    pub product_id_hex: &'static str,
    pub udev_rule_path: &'static str,
    pub udev_rule_installed: bool,
    pub input_group_state: GroupMembershipState,
    pub app_version: &'static str,
}

impl Default for DeviceTabInfo {
    fn default() -> Self {
        let devices = detect_rapoo_devices().unwrap_or_default();
        let udev_rule_installed = Path::new("/etc/udev/rules.d/99-openrapoo.rules").exists();
        let input_group_state = check_input_group_status();

        DeviceTabInfo {
            devices,
            manufacturer: "ITON Corp. / Rapoo",
            model: "Rapoo MT760 Pro",
            vendor_id_hex: "0x24AE",
            product_id_hex: "0x186A",
            udev_rule_path: "/etc/udev/rules.d/99-openrapoo.rules",
            udev_rule_installed,
            input_group_state,
            app_version: env!("CARGO_PKG_VERSION"),
        }
    }
}

impl DeviceTabInfo {
    pub fn refresh(&mut self) {
        self.devices = detect_rapoo_devices().unwrap_or_default();
        self.udev_rule_installed = Path::new("/etc/udev/rules.d/99-openrapoo.rules").exists();
        self.input_group_state = check_input_group_status();
    }
}
