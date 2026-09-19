//! Proprietary Rapoo Vendor HID telemetry provider via `/dev/hidraw*`.

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use crate::RAPOO_VENDOR_ID;
use super::hidraw::find_vendor_hidraw_candidate;
use super::provider::{current_epoch_seconds, BatteryProvider, DeviceIdentity};
use super::types::{
    BatteryConfidence, BatteryReading, BatterySource, BatteryState, BatteryValidity, RawProviderData,
};

pub struct RapooVendorHidProvider;

impl RapooVendorHidProvider {
    pub fn new() -> Self {
        Self
    }
}

impl BatteryProvider for RapooVendorHidProvider {
    fn name(&self) -> &str {
        "Rapoo Vendor HID Provider (Report ID 0x07 / 0xA0)"
    }

    fn source(&self) -> BatterySource {
        BatterySource::HidVendorSpecific
    }

    fn priority(&self) -> u8 {
        5
    }

    fn is_available(&self) -> bool {
        // True if any hidraw node exists
        Path::new("/dev").exists()
    }

    fn query(&self, device: &DeviceIdentity, log: &mut Vec<String>) -> Option<BatteryReading> {
        let transport = device.resolved_transport();
        if transport == crate::device::ConnectionType::Bluetooth {
            log.push("Provedor Rapoo Vendor HID ignorado para transporte Bluetooth (dispositivo conectado via Bluetooth).".to_string());
            return None;
        }

        let vid = if device.vendor_id != 0 { device.vendor_id } else { RAPOO_VENDOR_ID };

        // 1. Locate vendor hidraw candidate dynamically using descriptor scanner
        let candidate = find_vendor_hidraw_candidate(
            vid,
            device.product_id,
            device.hid_uniq.as_deref().or(device.phys.as_deref()),
        );

        let target_hidraw_path = match candidate {
            Some(ref c) => {
                log.push(format!("Candidato hidraw proprietário encontrado: {} (Report IDs: {:?})", c.hidraw_path.display(), c.report_ids));
                c.hidraw_path.clone()
            }
            None => {
                if let Some(ref path) = device.hidraw_path {
                    log.push(format!("Usando hidraw associado ao dispositivo: {}", path.display()));
                    path.clone()
                } else {
                    log.push("Nenhuma interface hidraw proprietária Rapoo encontrada.".to_string());
                    return None;
                }
            }
        };

        if !target_hidraw_path.exists() {
            log.push(format!("Caminho hidraw {} não existe.", target_hidraw_path.display()));
            return None;
        }

        // 2. Query vendor battery report using Report ID 0x07 (Category 0x01) with fallback to 0xA0
        query_single_hidraw_vendor_battery(&target_hidraw_path, device, log)
    }
}

fn query_single_hidraw_vendor_battery(
    hidraw_path: &Path,
    device: &DeviceIdentity,
    log: &mut Vec<String>,
) -> Option<BatteryReading> {
    log.push(format!("Abrindo nó hidraw '{}' para consulta proprietária...", hidraw_path.display()));

    let mut file = match OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(hidraw_path)
    {
        Ok(f) => f,
        Err(e) => {
            log.push(format!("Falha ao abrir '{}': {e} (permissão ou dispositivo ocupado)", hidraw_path.display()));
            return None;
        }
    };

    // --- Probe 1: Confirmed Windows Protocol — Report ID 0x07, Category 0x01 ---
    let mut req_packet = [0u8; 16];
    req_packet[0] = 0x07; // Report ID
    req_packet[1] = 0x01; // Category: Device Info / Battery Query

    if let Err(e) = file.write_all(&req_packet) {
        log.push(format!("Envio da sonda Report ID 0x07 falhou em '{}': {e}", hidraw_path.display()));
    } else {
        log.push("Sonda Report ID 0x07 enviada. Aguardando resposta...".to_string());
        std::thread::sleep(std::time::Duration::from_millis(15));

        let mut buf = [0u8; 64];
        if let Ok(n) = file.read(&mut buf) {
            if n >= 4 && buf[0] == 0x07 && buf[1] == 0x01 {
                let pct = buf[2];
                let chg_state = buf[3];
                if pct <= 100 {
                    let is_charging = chg_state == 1 || chg_state == 2;
                    log.push(format!("Resposta válida recebida via Report ID 0x07 em '{}': {pct}% (estado carga: {chg_state})", hidraw_path.display()));

                    let raw_data = RawProviderData {
                        hidraw_path: Some(hidraw_path.to_string_lossy().to_string()),
                        report_id: Some(0x07),
                        ..Default::default()
                    };

                    return Some(BatteryReading {
                        percentage: Some(pct),
                        state: if is_charging { BatteryState::Charging } else { BatteryState::Available },
                        is_present: true,
                        charging: Some(is_charging),
                        source: BatterySource::HidVendorSpecific,
                        timestamp: current_epoch_seconds(),
                        device_identifier: device.name.clone(),
                        connection: "hidraw_0x07".to_string(),
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

    // --- Probe 2: Legacy / Experimental Fallback Probe (Report ID 0xA0) ---
    let mut req_a0 = [0u8; 16];
    req_a0[0] = 0xA0;
    req_a0[1] = 0x08;

    if let Err(e) = file.write_all(&req_a0) {
        log.push(format!("Envio da sonda 0xA0 falhou em '{}': {e}", hidraw_path.display()));
    } else {
        std::thread::sleep(std::time::Duration::from_millis(15));
        let mut buf = [0u8; 64];
        if let Ok(n) = file.read(&mut buf) {
            if n >= 3 {
                if let Some((pct, chg)) = parse_rapoo_battery_response(&buf[..n]) {
                    log.push(format!("Resposta 0xA0 convertida com sucesso: {pct}%"));

                    let raw_data = RawProviderData {
                        hidraw_path: Some(hidraw_path.to_string_lossy().to_string()),
                        report_id: Some(0xA0),
                        ..Default::default()
                    };

                    return Some(BatteryReading {
                        percentage: Some(pct),
                        state: if chg { BatteryState::Charging } else { BatteryState::Available },
                        is_present: true,
                        charging: Some(chg),
                        source: BatterySource::HidVendorSpecific,
                        timestamp: current_epoch_seconds(),
                        device_identifier: device.name.clone(),
                        connection: "hidraw_0xA0".to_string(),
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

    log.push("Bateria não exposta pelo dongle ou protocolo ainda não respondeu neste modo de conexão.".to_string());
    None
}

/// Helper function to parse legacy Rapoo raw bytes buffer response.
pub fn parse_rapoo_battery_response(resp: &[u8]) -> Option<(u8, bool)> {
    if resp.len() < 3 {
        return None;
    }

    if (resp[0] == 0xA0 || resp[0] == 0xA1) && resp[1] == 0x08 {
        let raw_val = resp[2];
        let charging = (resp[2] & 0x80) != 0;
        let pct = (raw_val & 0x7F).min(100);
        return Some((pct, charging));
    }

    if resp[0] == 0xBA || resp[0] == 0x09 {
        let val = resp[1];
        if val <= 100 {
            return Some((val, false));
        } else {
            let pct = match val {
                0..=4 => (val as u16 * 25) as u8,
                5..=100 => val,
                _ => 100,
            };
            return Some((pct, false));
        }
    }

    None
}

