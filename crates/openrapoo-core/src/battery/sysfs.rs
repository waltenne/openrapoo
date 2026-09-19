//! Sysfs power_supply battery telemetry provider.

use std::fs;
use std::path::Path;

use super::provider::{current_epoch_seconds, BatteryProvider, DeviceIdentity};
use super::types::{
    BatteryConfidence, BatteryReading, BatterySource, BatteryState, BatteryValidity, RawProviderData,
};

pub struct SysfsProvider;

impl SysfsProvider {
    pub fn new() -> Self {
        Self
    }
}

impl BatteryProvider for SysfsProvider {
    fn name(&self) -> &str {
        "Sysfs Power Supply Provider"
    }

    fn source(&self) -> BatterySource {
        BatterySource::Sysfs
    }

    fn priority(&self) -> u8 {
        6
    }

    fn is_available(&self) -> bool {
        Path::new("/sys/class/power_supply").exists()
    }

    fn query(&self, device: &DeviceIdentity, log: &mut Vec<String>) -> Option<BatteryReading> {
        let sysfs_base = Path::new("/sys/class/power_supply");
        if !sysfs_base.exists() {
            log.push("Diretório /sys/class/power_supply não encontrado.".to_string());
            return None;
        }

        let entries = match fs::read_dir(sysfs_base) {
            Ok(e) => e,
            Err(e) => {
                log.push(format!("Falha ao ler /sys/class/power_supply: {e}"));
                return None;
            }
        };

        for entry in entries.flatten() {
            let entry_name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();

            let supply_type = fs::read_to_string(path.join("type"))
                .unwrap_or_default()
                .trim()
                .to_string();

            if supply_type != "Mouse" && supply_type != "Keyboard" && supply_type != "Battery" && !entry_name.contains("hid") {
                continue;
            }

            let uevent = fs::read_to_string(path.join("uevent")).unwrap_or_default();
            let mut model_name = fs::read_to_string(path.join("model_name"))
                .or_else(|_| fs::read_to_string(path.join("device/name")))
                .unwrap_or_default();

            if model_name.trim().is_empty() {
                for line in uevent.lines() {
                    if let Some(val) = line.strip_prefix("POWER_SUPPLY_MODEL_NAME=") {
                        model_name = val.trim().to_string();
                        break;
                    }
                }
            }

            if is_vendor_excluded(&uevent, &model_name) {
                log.push(format!("Descartada entrada sysfs '{entry_name}': fabricante excluído (Logitech/Razer/etc.)"));
                continue;
            }

            if !is_device_match(&device.name, &model_name, &uevent, &entry_name, device.phys.as_deref()) {
                log.push(format!("Descartada entrada sysfs '{entry_name}': modelo '{model_name}' não corresponde a '{}'", device.name));
                continue;
            }

            log.push(format!("Entrada sysfs validada para '{}': {entry_name}", device.name));

            let cap_str = fs::read_to_string(path.join("capacity")).unwrap_or_default();
            let pct_opt = cap_str.trim().parse::<u8>().ok().or_else(|| {
                fs::read_to_string(path.join("capacity_level")).ok().and_then(|lvl| match lvl.trim().to_lowercase().as_str() {
                    "full" => Some(100),
                    "high" => Some(80),
                    "normal" | "medium" => Some(50),
                    "low" => Some(20),
                    "critical" => Some(5),
                    _ => None,
                })
            });

            if let Some(pct) = pct_opt {
                let status_str = fs::read_to_string(path.join("status")).unwrap_or_default().to_lowercase();
                let is_charging = status_str.contains("charging") && !status_str.contains("discharging");

                let raw_data = RawProviderData {
                    sysfs_path: Some(path.to_string_lossy().to_string()),
                    model: Some(model_name.clone()),
                    ..Default::default()
                };

                return Some(BatteryReading {
                    percentage: Some(pct.min(100)),
                    state: if is_charging { BatteryState::Charging } else { BatteryState::Available },
                    is_present: true,
                    charging: Some(is_charging),
                    source: BatterySource::Sysfs,
                    timestamp: current_epoch_seconds(),
                    device_identifier: device.name.clone(),
                    connection: "sysfs".to_string(),
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

        None
    }
}

fn is_vendor_excluded(uevent: &str, model: &str) -> bool {
    let u_lower = uevent.to_lowercase();
    let m_lower = model.to_lowercase();
    for ex in ["logitech", "razer", "corsair", "apple", "steelseries", "dell", "hp", "lenovo"] {
        if u_lower.contains(ex) || m_lower.contains(ex) {
            return true;
        }
    }
    false
}

fn is_device_match(device_name: &str, model_name: &str, uevent: &str, entry_name: &str, phys: Option<&str>) -> bool {
    let dev_lower = device_name.to_lowercase();
    let mod_lower = model_name.to_lowercase();
    let combined = format!("{mod_lower} {} {}", uevent.to_lowercase(), entry_name.to_lowercase());

    if let Some(p) = phys {
        let p_clean = p.to_lowercase().replace([':', '_', '-'], "");
        let combined_clean = combined.replace([':', '_', '-'], "");
        if !p_clean.is_empty() && combined_clean.contains(&p_clean) {
            return true;
        }
    }

    if dev_lower.contains("e9050") {
        return combined.contains("e9050") || (combined.contains("keyboard") && !combined.contains("mouse"));
    }

    if dev_lower.contains("mt760") {
        return combined.contains("mt760") || (combined.contains("mouse") && !combined.contains("keyboard"));
    }

    combined.contains("rapoo") || combined.contains("24ae")
}

