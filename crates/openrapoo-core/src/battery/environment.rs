//! Automatic system environment inspector for telemetry and diagnostics.

use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

use super::hidraw::scan_hidraw_interfaces;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEnvironment {
    pub distro: String,
    pub kernel_version: String,
    pub upower_version: Option<String>,
    pub bluez_version: Option<String>,
    pub desktop_environment: String,
    pub session_type: String,
    pub bluetooth_controllers: Vec<String>,
    pub upower_devices: Vec<String>,
    pub bluez_devices: Vec<String>,
    pub hid_devices: Vec<String>,
    pub evdev_interfaces: Vec<String>,
    pub hidraw_interfaces: Vec<String>,
    pub user_groups: Vec<String>,
    pub udev_rules_found: Vec<String>,
}

pub fn collect_system_environment() -> SystemEnvironment {
    // 1. Distro
    let distro = fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|c| {
            for line in c.lines() {
                if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
                    return Some(val.trim_matches('"').to_string());
                }
            }
            None
        })
        .unwrap_or_else(|| "Linux Genérico".to_string());

    // 2. Kernel
    let kernel_version = Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "Desconhecido".to_string());

    // 3. UPower CLI Version
    let upower_version = Command::new("upower")
        .arg("--version")
        .output()
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .unwrap_or_default()
                .trim()
                .to_string()
        });

    // 4. BlueZ CLI Version
    let bluez_version = Command::new("bluetoothctl")
        .arg("--version")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    // 5. Desktop & Session
    let desktop_environment = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "Desconhecido".to_string());

    let session_type =
        std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "x11/wayland (auto)".to_string());

    // 6. Bluetooth controllers
    let mut bluetooth_controllers = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/bluetooth") {
        for entry in entries.flatten() {
            bluetooth_controllers.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    // 7. HID Devices
    let mut hid_devices = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/bus/hid/devices") {
        for entry in entries.flatten() {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if dir_name.contains("24AE") || dir_name.contains("24ae") {
                hid_devices.push(dir_name);
            }
        }
    }

    // 8. Evdev interfaces
    let mut evdev_interfaces = Vec::new();
    if let Ok(content) = fs::read_to_string("/proc/bus/input/devices") {
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("N: Name=") {
                if rest.to_lowercase().contains("rapoo") {
                    evdev_interfaces.push(rest.trim_matches('"').to_string());
                }
            }
        }
    }

    // 9. Hidraw interfaces
    let hidraw_candidates = scan_hidraw_interfaces();
    let hidraw_interfaces = hidraw_candidates
        .iter()
        .map(|c| {
            format!(
                "{} (VID: {:04X}, PID: {:04X}, Vendor: {})",
                c.hidraw_path.display(),
                c.vendor_id,
                c.product_id,
                c.is_vendor_interface
            )
        })
        .collect();

    // 10. User Groups
    let mut user_groups = Vec::new();
    if let Ok(output) = Command::new("id").arg("-Gn").output() {
        let groups_str = String::from_utf8_lossy(&output.stdout);
        user_groups = groups_str
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
    }

    // 11. Udev Rules
    let mut udev_rules_found = Vec::new();
    for rule_dir in ["/etc/udev/rules.d", "/usr/lib/udev/rules.d"] {
        if let Ok(entries) = fs::read_dir(rule_dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.contains("rapoo") || fname.contains("input") || fname.contains("hid") {
                    udev_rules_found.push(format!("{rule_dir}/{fname}"));
                }
            }
        }
    }

    SystemEnvironment {
        distro,
        kernel_version,
        upower_version,
        bluez_version,
        desktop_environment,
        session_type,
        bluetooth_controllers,
        upower_devices: Vec::new(),
        bluez_devices: Vec::new(),
        hid_devices,
        evdev_interfaces,
        hidraw_interfaces,
        user_groups,
        udev_rules_found,
    }
}
