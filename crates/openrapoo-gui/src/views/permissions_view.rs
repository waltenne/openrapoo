#![allow(dead_code)]
//! Permissions and udev status checker view model for OpenRapoo GUI.

use openrapoo_core::permissions::{check_input_group_status, GroupMembershipState};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PermissionsCheckStatus {
    pub udev_rule_exists: bool,
    pub input_group_state: GroupMembershipState,
    pub dev_uinput_accessible: bool,
    pub evdev_nodes_accessible: bool,
}

impl PermissionsCheckStatus {
    pub fn check() -> Self {
        let udev_rule_exists = Path::new("/etc/udev/rules.d/99-openrapoo.rules").exists();
        let dev_uinput_accessible = Path::new("/dev/uinput").exists();
        let input_group_state = check_input_group_status();

        let evdev_nodes_accessible = Path::new("/dev/input")
            .read_dir()
            .map(|entries| {
                entries.flatten().any(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    name.starts_with("event")
                        && e.metadata()
                            .map(|m| !m.permissions().readonly())
                            .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        PermissionsCheckStatus {
            udev_rule_exists,
            input_group_state,
            dev_uinput_accessible,
            evdev_nodes_accessible,
        }
    }
}
