//! UPower battery telemetry provider using native D-Bus (`zbus::blocking`).

use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::OwnedObjectPath;

use super::provider::{current_epoch_seconds, BatteryProvider, DeviceIdentity};
use super::types::{
    BatteryConfidence, BatteryReading, BatterySource, BatteryState, BatteryValidity, RawProviderData,
};

pub struct UPowerProvider;

impl UPowerProvider {
    pub fn new() -> Self {
        Self
    }
}

impl BatteryProvider for UPowerProvider {
    fn name(&self) -> &str {
        "UPower D-Bus Provider"
    }

    fn source(&self) -> BatterySource {
        BatterySource::UPower
    }

    fn priority(&self) -> u8 {
        3
    }

    fn is_available(&self) -> bool {
        Connection::system().is_ok()
    }

    fn query(&self, device: &DeviceIdentity, log: &mut Vec<String>) -> Option<BatteryReading> {
        log.push("Conectando ao barramento de sistema D-Bus (UPower)...".to_string());

        let conn = match Connection::system() {
            Ok(c) => c,
            Err(e) => {
                log.push(format!("Falha ao conectar ao D-Bus de sistema: {e}"));
                return None;
            }
        };

        let upower_proxy = match Proxy::new(
            &conn,
            "org.freedesktop.UPower",
            "/org/freedesktop/UPower",
            "org.freedesktop.UPower",
        ) {
            Ok(p) => p,
            Err(e) => {
                log.push(format!("Falha ao criar proxy para org.freedesktop.UPower: {e}"));
                return None;
            }
        };

        let device_paths: Vec<OwnedObjectPath> = match upower_proxy.call("EnumerateDevices", &()) {
            Ok(paths) => paths,
            Err(e) => {
                log.push(format!("Falha ao chamar EnumerateDevices no UPower: {e}"));
                return None;
            }
        };

        log.push(format!("Dispositivos UPower enumerados: {}", device_paths.len()));

        for dev_path in device_paths {
            let path_str = dev_path.as_str();
            let path_lower = path_str.to_lowercase();

            if !path_lower.contains("mouse")
                && !path_lower.contains("keyboard")
                && !path_lower.contains("dev_")
                && !path_lower.contains("battery")
            {
                continue;
            }

            let dev_proxy = match Proxy::new(
                &conn,
                "org.freedesktop.UPower",
                path_str,
                "org.freedesktop.UPower.Device",
            ) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let is_present: bool = dev_proxy.get_property("IsPresent").unwrap_or(false);
            if !is_present {
                log.push(format!("Dispositivo UPower '{path_str}' ignorado: IsPresent=false"));
                continue;
            }

            let model: String = dev_proxy.get_property("Model").unwrap_or_default();
            let serial: String = dev_proxy.get_property("Serial").unwrap_or_default();
            let native_path: String = dev_proxy.get_property("NativePath").unwrap_or_default();
            let state_code: u32 = dev_proxy.get_property("State").unwrap_or(0);
            let dev_type_code: u32 = dev_proxy.get_property("Type").unwrap_or(0);
            let update_time: u64 = dev_proxy.get_property("UpdateTime").unwrap_or(0);
            let battery_level_code: u32 = dev_proxy.get_property("BatteryLevel").unwrap_or(0);
            let raw_pct: f64 = dev_proxy.get_property("Percentage").unwrap_or(-1.0);

            // Match device to identity
            if !is_upower_device_match(device, &model, &serial, &native_path, path_str, dev_type_code) {
                log.push(format!("Objeto UPower '{path_str}' descartado para '{}' (Model: '{model}')", device.name));
                continue;
            }

            log.push(format!("Dispositivo UPower correspondido para '{}': {path_str}", device.name));

            // Determine percentage
            let percentage_opt = if raw_pct >= 0.0 {
                Some(raw_pct.round() as u8)
            } else {
                match battery_level_code {
                    8 => Some(100),
                    7 => Some(80),
                    6 => Some(50),
                    3 => Some(20),
                    4 => Some(5),
                    _ => None,
                }
            };

            let now = current_epoch_seconds();
            let is_stale = update_time > 0 && now > update_time && (now - update_time) > 3600;

            // Handle suspicious 0% reading with state unknown
            if percentage_opt == Some(0) && state_code == 0 && battery_level_code == 0 {
                log.push(format!("Ignorando {path_str}: bateria 0% com State=0 (Unknown) e BatteryLevel=0 (BlueZ não sincronizou)"));
                return Some(BatteryReading {
                    percentage: None,
                    state: BatteryState::Unknown,
                    is_present: true,
                    charging: None,
                    source: BatterySource::UPower,
                    timestamp: if update_time > 0 { update_time } else { now },
                    device_identifier: device.name.clone(),
                    connection: "upower_dbus".to_string(),
                    confidence: BatteryConfidence::Unconfirmed,
                    device_match_confidence: BatteryConfidence::High,
                    reading_confidence: BatteryConfidence::Unconfirmed,
                    reading_valid: false,
                    is_stale,
                    validity: BatteryValidity::ZeroUnconfirmed,
                    invalidation_reason: Some("Leitura de 0% não confirmada pelo UPower/BlueZ (estado desconhecido)".to_string()),
                    alternative_source: None,
                    conflict_status: None,
                    raw_data: Some(RawProviderData {
                        upower_path: Some(path_str.to_string()),
                        model: Some(model),
                        serial: Some(serial),
                        ..Default::default()
                    }),
                });
            }

            if let Some(pct) = percentage_opt {
                let is_charging = state_code == 1; // 1 = Charging
                let _is_discharging = state_code == 2 || state_code == 3;

                let validity = if is_stale {
                    BatteryValidity::StaleReading { age_seconds: now - update_time }
                } else {
                    BatteryValidity::Valid
                };

                let state = if is_charging {
                    BatteryState::Charging
                } else if state_code == 4 {
                    BatteryState::Full
                } else if pct <= 5 {
                    BatteryState::Critical
                } else if pct <= 20 {
                    BatteryState::Low
                } else {
                    BatteryState::Available
                };

                let dev_match_conf = if native_path.contains("hci") || !serial.is_empty() {
                    BatteryConfidence::High
                } else {
                    BatteryConfidence::Medium
                };

                let reading_conf = if pct == 0 && state_code == 0 {
                    BatteryConfidence::Unconfirmed
                } else {
                    dev_match_conf.clone()
                };

                let raw_data = RawProviderData {
                    upower_path: Some(path_str.to_string()),
                    bluetooth_address: if serial.contains(':') { Some(serial.clone()) } else { None },
                    model: Some(model),
                    serial: Some(serial),
                    ..Default::default()
                };

                return Some(BatteryReading {
                    percentage: Some(pct.min(100)),
                    state,
                    is_present,
                    charging: Some(is_charging),
                    source: BatterySource::UPower,
                    timestamp: if update_time > 0 { update_time } else { now },
                    device_identifier: device.name.clone(),
                    connection: "upower_dbus".to_string(),
                    confidence: reading_conf.clone(),
                    device_match_confidence: dev_match_conf,
                    reading_confidence: reading_conf,
                    reading_valid: !is_stale && (pct > 0 || state_code != 0),
                    is_stale,
                    validity,
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

fn is_upower_device_match(
    device: &DeviceIdentity,
    model: &str,
    serial: &str,
    native_path: &str,
    path_str: &str,
    dev_type_code: u32,
) -> bool {
    use crate::device::DeviceType;

    let dev_lower = device.name.to_lowercase();
    let mod_lower = model.to_lowercase();
    let path_lower = path_str.to_lowercase();
    let native_lower = native_path.to_lowercase();

    let combined = format!("{mod_lower} {path_lower} {native_lower}");
    let combined_clean = combined.replace([':', '_', '-'], "").to_lowercase();
    let serial_clean = serial.replace([':', '_', '-'], "").to_lowercase();

    // Check vendor exclusion
    for ex in ["logitech", "razer", "corsair", "apple", "steelseries", "dell", "hp", "lenovo"] {
        if mod_lower.contains(ex) || native_lower.contains(ex) {
            return false;
        }
    }

    // Strict DeviceType exclusion
    if device.device_type == DeviceType::Mouse {
        if dev_type_code == 6 || mod_lower.contains("keyboard") || path_lower.contains("keyboard_dev") {
            return false;
        }
    } else if device.device_type == DeviceType::Keyboard {
        if dev_type_code == 5 || mod_lower.contains("mouse") || path_lower.contains("mouse_dev") {
            return false;
        }
    }

    // Strict MAC address matching if target has bluetooth_address
    if let Some(ref target_bt) = device.bluetooth_address {
        let target_clean = target_bt.replace([':', '_', '-'], "").to_lowercase();
        if !target_clean.is_empty() {
            let matches_mac = combined_clean.contains(&target_clean) || serial_clean.contains(&target_clean);
            if matches_mac {
                return true;
            } else {
                // Target MAC specified but does not match this UPower device — reject immediately!
                return false;
            }
        }
    }

    // Match by physical path / phys
    if let Some(ref phys) = device.phys {
        let p_clean = phys.to_lowercase().replace([':', '_', '-'], "");
        if !p_clean.is_empty() && (combined_clean.contains(&p_clean) || (!serial_clean.is_empty() && p_clean.contains(&serial_clean))) {
            return true;
        }
    }

    // Model specific matching (only when no MAC specified)
    if dev_lower.contains("e9050") || device.device_type == DeviceType::Keyboard {
        if combined.contains("e9050") || dev_type_code == 6 || mod_lower.contains("keyboard") {
            return !combined.contains("mouse");
        }
        return false;
    }

    if dev_lower.contains("mt760") || device.device_type == DeviceType::Mouse {
        if combined.contains("mt760") || dev_type_code == 5 || mod_lower.contains("mouse") {
            return !combined.contains("keyboard");
        }
        return false;
    }

    false
}
