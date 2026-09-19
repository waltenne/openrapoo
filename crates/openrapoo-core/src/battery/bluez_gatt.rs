//! GATT Battery Service (0x180F / 0x2A19) telemetry provider via BlueZ D-Bus (`zbus::blocking`).

use std::collections::HashMap;
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::OwnedValue;

use super::provider::{current_epoch_seconds, BatteryProvider, DeviceIdentity};
use super::types::{
    BatteryConfidence, BatteryReading, BatterySource, BatteryState, BatteryValidity,
    RawProviderData,
};

pub struct BluezGattProvider;

impl Default for BluezGattProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl BluezGattProvider {
    pub fn new() -> Self {
        Self
    }
}

impl BatteryProvider for BluezGattProvider {
    fn name(&self) -> &str {
        "BlueZ GATT Battery Service Provider (UUID 0x180F / 0x2A19)"
    }

    fn source(&self) -> BatterySource {
        BatterySource::BluezGatt
    }

    fn priority(&self) -> u8 {
        2
    }

    fn is_available(&self) -> bool {
        Connection::system().is_ok()
    }

    fn query(&self, device: &DeviceIdentity, log: &mut Vec<String>) -> Option<BatteryReading> {
        log.push("Procurando serviços GATT de bateria (UUID 0x180F)...".to_string());

        let conn = match Connection::system() {
            Ok(c) => c,
            Err(_) => return None,
        };

        let manager_proxy = match Proxy::new(
            &conn,
            "org.bluez",
            "/",
            "org.freedesktop.DBus.ObjectManager",
        ) {
            Ok(p) => p,
            Err(_) => return None,
        };

        type ManagedObjects =
            HashMap<zbus::zvariant::OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>;
        let objects: ManagedObjects = match manager_proxy.call("GetManagedObjects", &()) {
            Ok(objs) => objs,
            Err(_) => return None,
        };

        for (path, interfaces) in objects {
            if !interfaces.contains_key("org.bluez.GattCharacteristic1") {
                continue;
            }

            let path_str = path.as_str();

            if let Some(char_props) = interfaces.get("org.bluez.GattCharacteristic1") {
                let uuid = char_props
                    .get("UUID")
                    .map(extract_string_value)
                    .unwrap_or_default();

                // Check for Battery Level Characteristic 00002a19-0000-1000-8000-00805f9b34fb
                if !uuid.to_lowercase().contains("2a19") {
                    continue;
                }

                // Match parent device path to target device
                let dev_match = if let Some(ref phys) = device.phys {
                    let clean_p = phys.to_lowercase().replace([':', '_', '-'], "");
                    let clean_path = path_str.to_lowercase().replace([':', '_', '-'], "");
                    !clean_p.is_empty() && clean_path.contains(&clean_p)
                } else {
                    path_str.to_lowercase().contains("dev_")
                };

                if !dev_match {
                    continue;
                }

                log.push(format!(
                    "Característica GATT Battery Level (0x2A19) encontrada em '{path_str}'"
                ));

                let char_proxy = match Proxy::new(
                    &conn,
                    "org.bluez",
                    path_str,
                    "org.bluez.GattCharacteristic1",
                ) {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                let empty_options: HashMap<String, OwnedValue> = HashMap::new();
                let bytes: Vec<u8> = match char_proxy.call("ReadValue", &(empty_options,)) {
                    Ok(b) => b,
                    Err(e) => {
                        log.push(format!(
                            "Leitura GATT ReadValue falhou em '{path_str}': {e}"
                        ));
                        continue;
                    }
                };

                if let Some(&pct) = bytes.first() {
                    if pct <= 100 {
                        log.push(format!("Leitura GATT realizada com sucesso: {pct}%"));

                        let raw_data = RawProviderData {
                            bluez_path: Some(path_str.to_string()),
                            ..Default::default()
                        };

                        return Some(BatteryReading {
                            percentage: Some(pct),
                            state: BatteryState::Available,
                            is_present: true,
                            charging: None,
                            source: BatterySource::BluezGatt,
                            timestamp: current_epoch_seconds(),
                            device_identifier: device.name.clone(),
                            connection: "bluetooth_gatt".to_string(),
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
                    }
                }
            }
        }

        None
    }
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
