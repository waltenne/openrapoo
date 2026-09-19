//! BlueZ Bluetooth Battery telemetry provider using native D-Bus (`zbus::blocking`).

use std::collections::HashMap;
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::OwnedValue;

use super::provider::{current_epoch_seconds, BatteryProvider, DeviceIdentity};
use super::types::{
    BatteryConfidence, BatteryReading, BatterySource, BatteryState, BatteryValidity,
    RawProviderData,
};

pub struct BluezBatteryProvider;

impl Default for BluezBatteryProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl BluezBatteryProvider {
    pub fn new() -> Self {
        Self
    }
}

impl BatteryProvider for BluezBatteryProvider {
    fn name(&self) -> &str {
        "BlueZ D-Bus Battery Provider"
    }

    fn source(&self) -> BatterySource {
        BatterySource::BlueZ
    }

    fn priority(&self) -> u8 {
        1 // Highest priority for Bluetooth devices
    }

    fn is_available(&self) -> bool {
        Connection::system().is_ok()
    }

    fn query(&self, device: &DeviceIdentity, log: &mut Vec<String>) -> Option<BatteryReading> {
        log.push(
            "Consultando BlueZ D-Bus para interface org.bluez.Battery1 e org.bluez.Device1..."
                .to_string(),
        );

        let conn = match Connection::system() {
            Ok(c) => c,
            Err(e) => {
                log.push(format!(
                    "Falha ao conectar ao D-Bus de sistema para BlueZ: {e}"
                ));
                return None;
            }
        };

        let manager_proxy = match Proxy::new(
            &conn,
            "org.bluez",
            "/",
            "org.freedesktop.DBus.ObjectManager",
        ) {
            Ok(p) => p,
            Err(e) => {
                log.push(format!("Falha ao obter ObjectManager do BlueZ: {e}"));
                return None;
            }
        };

        type ManagedObjects =
            HashMap<zbus::zvariant::OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>;
        let objects: ManagedObjects = match manager_proxy.call("GetManagedObjects", &()) {
            Ok(objs) => objs,
            Err(e) => {
                log.push(format!("Falha ao chamar GetManagedObjects no BlueZ: {e}"));
                HashMap::new()
            }
        };

        log.push(format!(
            "Objetos gerenciados pelo BlueZ encontrados: {}",
            objects.len()
        ));

        for (path, interfaces) in objects {
            if !interfaces.contains_key("org.bluez.Battery1")
                && !interfaces.contains_key("org.bluez.Device1")
            {
                continue;
            }

            let path_str = path.as_str();

            // Read org.bluez.Device1 properties for matching
            let mut address = String::new();
            let mut alias = String::new();
            let mut name = String::new();
            let mut is_connected = false;

            if let Ok(dev_proxy) = Proxy::new(&conn, "org.bluez", path_str, "org.bluez.Device1") {
                if let Ok(addr) = dev_proxy.get_property::<String>("Address") {
                    address = addr;
                }
                if let Ok(al) = dev_proxy.get_property::<String>("Alias") {
                    alias = al;
                }
                if let Ok(nm) = dev_proxy.get_property::<String>("Name") {
                    name = nm;
                }
                if let Ok(conn_state) = dev_proxy.get_property::<bool>("Connected") {
                    is_connected = conn_state;
                }
            }

            if address.is_empty() {
                if let Some(dev_props) = interfaces.get("org.bluez.Device1") {
                    if let Some(val) = dev_props.get("Address") {
                        address = extract_string_value(val);
                    }
                    if let Some(val) = dev_props.get("Alias") {
                        alias = extract_string_value(val);
                    }
                    if let Some(val) = dev_props.get("Name") {
                        name = extract_string_value(val);
                    }
                    if let Some(val) = dev_props.get("Connected") {
                        is_connected = extract_bool_value(val);
                    }
                }
            }

            let combined_device_info = format!(
                "{} {} {} {}",
                alias.to_lowercase(),
                name.to_lowercase(),
                address.to_lowercase(),
                path_str.to_lowercase()
            );

            // Match against device identity
            let is_match = is_bluez_match(device, &combined_device_info, &address);
            if !is_match {
                continue;
            }

            log.push(format!("Dispositivo BlueZ correspondente encontrado em '{path_str}' (MAC: {address}, Conectado: {is_connected})"));

            if !interfaces.contains_key("org.bluez.Battery1") {
                log.push(format!(
                    "Interface org.bluez.Battery1 ausente no objeto BlueZ '{path_str}'"
                ));
                continue;
            }

            log.push(format!(
                "Interface org.bluez.Battery1 encontrada em '{path_str}'"
            ));

            if !is_connected {
                log.push(format!(
                    "Dispositivo BlueZ '{path_str}' desconectado (Connected=false)."
                ));
                return Some(BatteryReading {
                    percentage: None,
                    state: BatteryState::DeviceDisconnected,
                    is_present: false,
                    charging: None,
                    source: BatterySource::BlueZ,
                    timestamp: current_epoch_seconds(),
                    device_identifier: device.name.clone(),
                    connection: "bluetooth".to_string(),
                    confidence: BatteryConfidence::Low,
                    device_match_confidence: BatteryConfidence::High,
                    reading_confidence: BatteryConfidence::None,
                    reading_valid: false,
                    is_stale: false,
                    validity: BatteryValidity::DeviceDisconnected,
                    invalidation_reason: Some("Dispositivo Bluetooth desconectado".to_string()),
                    alternative_source: None,
                    conflict_status: None,
                    raw_data: Some(RawProviderData {
                        bluez_path: Some(path_str.to_string()),
                        bluetooth_address: Some(address),
                        model: Some(alias),
                        ..Default::default()
                    }),
                });
            }

            // Direct active D-Bus property query on org.bluez.Battery1
            let mut percentage: Option<u8> = None;
            let mut bt_source = String::new();

            if let Ok(bat_proxy) = Proxy::new(&conn, "org.bluez", path_str, "org.bluez.Battery1") {
                if let Ok(pct) = bat_proxy.get_property::<u8>("Percentage") {
                    percentage = Some(pct);
                } else if let Ok(val) = bat_proxy.get_property::<OwnedValue>("Percentage") {
                    percentage = extract_u8_value(&val);
                }

                if let Ok(src) = bat_proxy.get_property::<String>("Source") {
                    bt_source = src;
                }
            }

            // Fallback to GetManagedObjects properties dictionary if direct property read returned None
            if percentage.is_none() {
                if let Some(bat_props) = interfaces.get("org.bluez.Battery1") {
                    if let Some(val) = bat_props.get("Percentage") {
                        percentage = extract_u8_value(val);
                    }
                    if let Some(val) = bat_props.get("Source") {
                        bt_source = extract_string_value(val);
                    }
                }
            }

            if let Some(pct) = percentage {
                if pct > 100 {
                    log.push(format!(
                        "Percentage de {pct}% rejeitado por valor inválido (>100%)"
                    ));
                    return Some(BatteryReading {
                        percentage: Some(pct),
                        state: BatteryState::Unknown,
                        is_present: true,
                        charging: None,
                        source: BatterySource::BlueZ,
                        timestamp: current_epoch_seconds(),
                        device_identifier: device.name.clone(),
                        connection: "bluetooth".to_string(),
                        confidence: BatteryConfidence::None,
                        device_match_confidence: BatteryConfidence::High,
                        reading_confidence: BatteryConfidence::Rejected,
                        reading_valid: false,
                        is_stale: false,
                        validity: BatteryValidity::Invalid {
                            reason: format!("Percentage de {pct}% acima de 100%"),
                        },
                        invalidation_reason: Some(format!(
                            "Percentage de {pct}% rejeitado por valor inválido"
                        )),
                        alternative_source: None,
                        conflict_status: None,
                        raw_data: Some(RawProviderData {
                            bluez_path: Some(path_str.to_string()),
                            bluetooth_address: Some(address),
                            model: Some(alias),
                            ..Default::default()
                        }),
                    });
                }

                log.push(format!("Percentage obtido via BlueZ: {pct}%"));

                // If 0% on BlueZ without confirmation, mark as unconfirmed
                if pct == 0 {
                    log.push(format!(
                        "BlueZ reportou 0% para {path_str}. Marcando como não confirmada."
                    ));
                    return Some(BatteryReading {
                        percentage: Some(0),
                        state: BatteryState::Unknown,
                        is_present: true,
                        charging: None,
                        source: BatterySource::BlueZ,
                        timestamp: current_epoch_seconds(),
                        device_identifier: device.name.clone(),
                        connection: "bluetooth".to_string(),
                        confidence: BatteryConfidence::Unconfirmed,
                        device_match_confidence: BatteryConfidence::High,
                        reading_confidence: BatteryConfidence::Unconfirmed,
                        reading_valid: false,
                        is_stale: false,
                        validity: BatteryValidity::ZeroUnconfirmed,
                        invalidation_reason: Some("Bateria 0% recebida via BlueZ sem confirmação secundária (estado desconhecido)".to_string()),
                        alternative_source: None,
                        conflict_status: None,
                        raw_data: Some(RawProviderData {
                            bluez_path: Some(path_str.to_string()),
                            bluetooth_address: Some(address),
                            model: Some(alias),
                            ..Default::default()
                        }),
                    });
                }

                let state = if pct <= 5 {
                    BatteryState::Critical
                } else if pct <= 20 {
                    BatteryState::Low
                } else {
                    BatteryState::Available
                };

                let raw_data = RawProviderData {
                    bluez_path: Some(path_str.to_string()),
                    bluetooth_address: Some(address),
                    model: Some(alias),
                    ..Default::default()
                };

                return Some(BatteryReading {
                    percentage: Some(pct.min(100)),
                    state,
                    is_present: true,
                    charging: None,
                    source: BatterySource::BlueZ,
                    timestamp: current_epoch_seconds(),
                    device_identifier: device.name.clone(),
                    connection: if bt_source.is_empty() {
                        "bluetooth".to_string()
                    } else {
                        format!("bluetooth_{bt_source}")
                    },
                    confidence: BatteryConfidence::High,
                    device_match_confidence: BatteryConfidence::High,
                    reading_confidence: BatteryConfidence::High,
                    reading_valid: true,
                    is_stale: false,
                    validity: BatteryValidity::Valid,
                    invalidation_reason: None,
                    alternative_source: None,
                    conflict_status: None,
                    raw_data: Some(raw_data),
                });
            } else {
                log.push(format!("Battery1 encontrada em '{path_str}', mas propriedade Percentage ausente ou não pôde ser lida"));
                return Some(BatteryReading {
                    percentage: None,
                    state: BatteryState::Unknown,
                    is_present: true,
                    charging: None,
                    source: BatterySource::BlueZ,
                    timestamp: current_epoch_seconds(),
                    device_identifier: device.name.clone(),
                    connection: "bluetooth".to_string(),
                    confidence: BatteryConfidence::None,
                    device_match_confidence: BatteryConfidence::High,
                    reading_confidence: BatteryConfidence::None,
                    reading_valid: false,
                    is_stale: false,
                    validity: BatteryValidity::Invalid {
                        reason: "Propriedade Percentage ausente na interface org.bluez.Battery1"
                            .to_string(),
                    },
                    invalidation_reason: Some(format!(
                        "Battery1 encontrada em {path_str}, mas propriedade Percentage ausente"
                    )),
                    alternative_source: None,
                    conflict_status: None,
                    raw_data: Some(RawProviderData {
                        bluez_path: Some(path_str.to_string()),
                        bluetooth_address: Some(address),
                        model: Some(alias),
                        ..Default::default()
                    }),
                });
            }
        }

        None
    }
}

fn is_bluez_match(device: &DeviceIdentity, combined: &str, address: &str) -> bool {
    use crate::device::DeviceType;

    let dev_lower = device.name.to_lowercase();
    let comb_lower = combined.to_lowercase();
    let comb_clean = comb_lower.replace([':', '_', '-'], "");
    let addr_clean = address.to_lowercase().replace([':', '_', '-'], "");

    // Strict DeviceType exclusion
    if device.device_type == DeviceType::Mouse {
        if comb_lower.contains("keyboard") {
            return false;
        }
    } else if device.device_type == DeviceType::Keyboard && comb_lower.contains("mouse") {
        return false;
    }

    // Strict MAC address matching
    if let Some(ref target_bt) = device.bluetooth_address {
        let target_clean = target_bt.to_lowercase().replace([':', '_', '-'], "");
        if !target_clean.is_empty() {
            let matches_mac = addr_clean == target_clean || comb_clean.contains(&target_clean);
            if matches_mac {
                return true;
            } else {
                // If target MAC address is specified and does not match, REJECT immediately!
                return false;
            }
        }
    }

    // Match by physical path / phys
    if let Some(ref phys) = device.phys {
        let p_clean = phys.to_lowercase().replace([':', '_', '-'], "");
        if !p_clean.is_empty()
            && (comb_clean.contains(&p_clean)
                || (!addr_clean.is_empty() && p_clean.contains(&addr_clean)))
        {
            return true;
        }
    }

    // Model specific matching
    if dev_lower.contains("e9050") || device.device_type == DeviceType::Keyboard {
        return (comb_lower.contains("e9050")
            || comb_lower.contains("keyboard")
            || comb_lower.contains("kbd"))
            && !comb_lower.contains("mouse");
    }

    if dev_lower.contains("mt760") || device.device_type == DeviceType::Mouse {
        return (comb_lower.contains("mt760")
            || comb_lower.contains("mouse")
            || comb_lower.contains("bt mouse"))
            && !comb_lower.contains("keyboard");
    }

    false
}

fn extract_string_value(val: &OwnedValue) -> String {
    if let Ok(s) = val.downcast_ref::<String>() {
        s.clone()
    } else if let Ok(s) = val.downcast_ref::<&str>() {
        s.to_string()
    } else if let Ok(s) = val.downcast_ref::<zbus::zvariant::Str>() {
        s.as_str().to_string()
    } else {
        val.to_string().trim_matches('"').to_string()
    }
}

fn extract_bool_value(val: &OwnedValue) -> bool {
    bool::try_from(val).unwrap_or_default()
}

pub fn extract_u8_value(val: &OwnedValue) -> Option<u8> {
    if let Ok(v) = u8::try_from(val) {
        return Some(v);
    }
    if let Ok(v) = u16::try_from(val) {
        return Some(v as u8);
    }
    if let Ok(v) = u32::try_from(val) {
        return Some(v as u8);
    }
    if let Ok(v) = i32::try_from(val) {
        if (0..=255).contains(&v) {
            return Some(v as u8);
        }
    }
    if let Ok(v) = u64::try_from(val) {
        return Some(v as u8);
    }
    let s = val.to_string();
    let trimmed = s.trim_matches(['"', '\'', ' ', '(', ')']);
    if let Ok(num) = trimmed.parse::<u8>() {
        return Some(num);
    }
    None
}
