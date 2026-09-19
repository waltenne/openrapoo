//! Standard Kernel HID battery telemetry provider.

use super::provider::{current_epoch_seconds, BatteryProvider, DeviceIdentity};
use super::types::{
    BatteryConfidence, BatteryReading, BatterySource, BatteryState, BatteryValidity,
    RawProviderData,
};
use std::fs;
use std::path::Path;

pub struct StandardHidProvider;

impl Default for StandardHidProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl StandardHidProvider {
    pub fn new() -> Self {
        Self
    }
}

impl BatteryProvider for StandardHidProvider {
    fn name(&self) -> &str {
        "Standard Kernel HID Battery Provider"
    }

    fn source(&self) -> BatterySource {
        BatterySource::HidStandard
    }

    fn priority(&self) -> u8 {
        4
    }

    fn is_available(&self) -> bool {
        Path::new("/sys/class/power_supply").exists()
    }

    fn query(&self, device: &DeviceIdentity, log: &mut Vec<String>) -> Option<BatteryReading> {
        let sysfs_base = Path::new("/sys/class/power_supply");
        if !sysfs_base.exists() {
            return None;
        }

        let entries = match fs::read_dir(sysfs_base) {
            Ok(e) => e,
            Err(_) => return None,
        };

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("hid-") || !name.ends_with("-battery") {
                continue;
            }

            let path = entry.path();
            let _uevent = fs::read_to_string(path.join("uevent")).unwrap_or_default();
            let model_name = fs::read_to_string(path.join("model_name"))
                .or_else(|_| fs::read_to_string(path.join("device/name")))
                .unwrap_or_default();

            // Match physical address/MAC or model name
            let is_match = if let Some(ref phys) = device.phys {
                let phys_clean = phys.to_lowercase().replace([':', '_', '-'], "");
                let name_clean = name.to_lowercase().replace([':', '_', '-'], "");
                !phys_clean.is_empty() && name_clean.contains(&phys_clean)
            } else {
                model_name.to_lowercase().contains("rapoo") || name.contains("24ae")
            };

            if is_match {
                if let Ok(cap_str) = fs::read_to_string(path.join("capacity")) {
                    if let Ok(pct) = cap_str.trim().parse::<u8>() {
                        log.push(format!(
                            "Leitura capturada via Standard HID battery em {name}: {pct}%"
                        ));
                        let status_str = fs::read_to_string(path.join("status"))
                            .unwrap_or_default()
                            .to_lowercase();
                        let is_charging =
                            status_str.contains("charging") && !status_str.contains("discharging");

                        let raw_data = RawProviderData {
                            sysfs_path: Some(path.to_string_lossy().to_string()),
                            model: Some(model_name),
                            ..Default::default()
                        };

                        return Some(BatteryReading {
                            percentage: Some(pct.min(100)),
                            state: if is_charging {
                                BatteryState::Charging
                            } else {
                                BatteryState::Available
                            },
                            is_present: true,
                            charging: Some(is_charging),
                            source: BatterySource::HidStandard,
                            timestamp: current_epoch_seconds(),
                            device_identifier: device.name.clone(),
                            connection: "hid_standard".to_string(),
                            confidence: BatteryConfidence::Medium,
                            device_match_confidence: BatteryConfidence::Medium,
                            reading_confidence: BatteryConfidence::Medium,
                            reading_valid: true,
                            is_stale: false,
                            validity: BatteryValidity::Valid,
                            invalidation_reason: None,
                            alternative_source: None,
                            conflict_status: None,
                            raw_data: Some(raw_data),
                        });
                    }
                }
            }
        }

        None
    }
}
