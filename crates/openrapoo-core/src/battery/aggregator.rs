use super::bluez::BluezBatteryProvider;
use super::bluez_gatt::BluezGattProvider;
use super::hid_standard::StandardHidProvider;
use super::hid_vendor::RapooVendorHidProvider;
use super::provider::{current_epoch_seconds, BatteryProvider, DeviceIdentity};
use super::sysfs::SysfsProvider;

use super::types::{
    BatteryConfidence, BatteryReading, BatterySource, BatteryState, BatteryValidity, ConflictInfo,
};
use super::upower::UPowerProvider;

use crate::device::ConnectionType;

pub struct AggregatedResult {
    pub chosen: BatteryReading,
    pub all_readings: Vec<BatteryReading>,
    pub discarded: Vec<(BatteryReading, String)>,
    pub conflict: Option<ConflictInfo>,
}

pub struct BatteryAggregator {
    providers: Vec<Box<dyn BatteryProvider>>,
}

impl Default for BatteryAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl BatteryAggregator {
    pub fn new() -> Self {
        let mut providers: Vec<Box<dyn BatteryProvider>> = vec![
            Box::new(BluezBatteryProvider::new()),
            Box::new(BluezGattProvider::new()),
            Box::new(UPowerProvider::new()),
            Box::new(StandardHidProvider::new()),
            Box::new(RapooVendorHidProvider::new()),
            Box::new(SysfsProvider::new()),
        ];

        // Sort by priority (lowest number = highest priority)
        providers.sort_by_key(|p| p.priority());

        Self { providers }
    }

    pub fn query_all(&self, device: &DeviceIdentity, log: &mut Vec<String>) -> AggregatedResult {
        let transport = device.resolved_transport();
        let now = current_epoch_seconds();
        let ttl = get_ttl_for_transport(&transport);

        log.push(format!(
            "--- Iniciando Varredura Multi-Fonte de Bateria para '{}' (Transporte Resolvido: {:?}, TTL: {}s) ---",
            device.name, transport, ttl
        ));

        if transport == ConnectionType::Unknown {
            log.push(format!(
                "ERRO DE DIAGNÓSTICO: Transporte do dispositivo '{}' é 'Unknown'. Seleção automática de bateria interrompida.",
                device.name
            ));

            let fallback_reason = "Transporte do dispositivo não pôde ser determinado".to_string();
            let fallback = BatteryReading {
                percentage: None,
                state: BatteryState::Unavailable,
                is_present: false,
                charging: None,
                source: BatterySource::Unavailable,
                timestamp: now,
                device_identifier: device.name.clone(),
                connection: "Unknown".to_string(),
                confidence: BatteryConfidence::None,
                device_match_confidence: BatteryConfidence::None,
                reading_confidence: BatteryConfidence::None,
                reading_valid: false,
                is_stale: false,
                validity: BatteryValidity::Invalid {
                    reason: fallback_reason.clone(),
                },
                invalidation_reason: Some(fallback_reason),
                alternative_source: None,
                conflict_status: None,
                raw_data: None,
            };

            return AggregatedResult {
                chosen: fallback,
                all_readings: Vec::new(),
                discarded: Vec::new(),
                conflict: None,
            };
        }

        let mut valid_readings = Vec::new();
        let mut discarded = Vec::new();

        for provider in &self.providers {
            if !provider.is_available() {
                log.push(format!(
                    "Provedor '{}' não disponível no ambiente.",
                    provider.name()
                ));
                continue;
            }

            // Verify transport compatibility for provider
            if !is_provider_compatible_with_transport(provider.source(), &transport) {
                log.push(format!(
                    "Provedor '{}' incompatível com transporte ativo ({:?}). Pulando.",
                    provider.name(),
                    transport
                ));
                continue;
            }

            log.push(format!("Executando consulta via '{}'...", provider.name()));
            if let Some(mut reading) = provider.query(device, log) {
                // Validate percentage range
                if let Some(pct) = reading.percentage {
                    if pct > 100 {
                        let reason = format!("Percentual fora da faixa válida: {pct}%");
                        log.push(format!(
                            "Leitura do provedor '{}' descartada: {reason}",
                            provider.name()
                        ));
                        reading.validity = BatteryValidity::Invalid {
                            reason: reason.clone(),
                        };
                        discarded.push((reading, reason));
                        continue;
                    }
                }

                // Validate timestamp (future check)
                if reading.timestamp > now + 5 {
                    let reason = "Timestamp no futuro rejeitado".to_string();
                    log.push(format!(
                        "Leitura do provedor '{}' descartada: {reason}",
                        provider.name()
                    ));
                    reading.validity = BatteryValidity::Invalid {
                        reason: reason.clone(),
                    };
                    discarded.push((reading, reason));
                    continue;
                }

                // Validate TTL (expiry check)
                if now > reading.timestamp && (now - reading.timestamp) > ttl {
                    let age_seconds = now - reading.timestamp;
                    let reason =
                        format!("Leitura expirada (idade: {age_seconds}s, max TTL: {ttl}s)");
                    log.push(format!(
                        "Leitura do provedor '{}' expirada: {reason}",
                        provider.name()
                    ));
                    reading.validity = BatteryValidity::StaleReading { age_seconds };
                    discarded.push((reading, reason));
                    continue;
                }

                if reading.validity == BatteryValidity::Valid {
                    valid_readings.push(reading);
                } else {
                    let reason = reading
                        .invalidation_reason
                        .clone()
                        .unwrap_or_else(|| format!("{:?}", reading.validity));
                    log.push(format!(
                        "Leitura do provedor '{}' descartada: {reason}",
                        provider.name()
                    ));
                    discarded.push((reading, reason));
                }
            }
        }

        if valid_readings.is_empty() {
            log.push("Nenhuma leitura válida obtida de nenhum provedor.".to_string());

            // If we have a discarded unconfirmed 0% reading, return it clearly marked
            if let Some((unconfirmed, _reason)) = discarded
                .iter()
                .find(|(r, _)| r.validity == BatteryValidity::ZeroUnconfirmed)
            {
                return AggregatedResult {
                    chosen: unconfirmed.clone(),
                    all_readings: Vec::new(),
                    discarded: Vec::new(),
                    conflict: None,
                };
            }

            // Transport-specific fallback message
            let fallback_reason = match device.transport {
                ConnectionType::TwoPointFourGhz => "Bateria não exposta pelo dongle".to_string(),
                ConnectionType::UsbCable | ConnectionType::UsbWired => {
                    "Bateria indisponível".to_string()
                }
                ConnectionType::Bluetooth => "Bateria indisponível".to_string(),
                ConnectionType::Dock => "Dock detectado, bateria não exposta".to_string(),
                _ => "Bateria indisponível".to_string(),
            };

            let fallback = BatteryReading {
                percentage: None,
                state: BatteryState::Unavailable,
                is_present: false,
                charging: None,
                source: BatterySource::Unavailable,
                timestamp: current_epoch_seconds(),
                device_identifier: device.name.clone(),
                connection: format!("{:?}", device.transport),
                confidence: BatteryConfidence::None,
                device_match_confidence: BatteryConfidence::None,
                reading_confidence: BatteryConfidence::None,
                reading_valid: false,
                is_stale: false,
                validity: BatteryValidity::Invalid {
                    reason: fallback_reason.clone(),
                },
                invalidation_reason: Some(fallback_reason),
                alternative_source: None,
                conflict_status: None,
                raw_data: None,
            };

            return AggregatedResult {
                chosen: fallback,
                all_readings: Vec::new(),
                discarded,
                conflict: None,
            };
        }

        // Check for cross-validation agreement or conflict between multiple valid readings
        let mut conflict: Option<ConflictInfo> = None;
        if valid_readings.len() >= 2 {
            let r1 = &valid_readings[0];
            let r2 = &valid_readings[1];

            if let (Some(pct1), Some(pct2)) = (r1.percentage, r2.percentage) {
                let diff = (pct1 as i16 - pct2 as i16).abs();
                if diff <= 5 {
                    log.push(format!("Validação Cruzada: Fonte '{}' ({pct1}%) e '{}' ({pct2}%) concordam (diferença <= 5%). Confiança ALTA atribuída.", r1.source, r2.source));
                } else {
                    log.push(format!("AVISO DE CONFLITO: Fonte '{}' ({pct1}%) e '{}' ({pct2}%) divergem por {diff}%. Marcando conflito.", r1.source, r2.source));
                    conflict = Some(ConflictInfo {
                        source_a: r1.source.clone(),
                        value_a: pct1,
                        source_b: r2.source.clone(),
                        value_b: pct2,
                    });
                }
            }
        }

        let mut chosen = valid_readings[0].clone();

        if let Some(ref c) = conflict {
            chosen.validity = BatteryValidity::ConflictingReadings {
                sources: vec![c.source_a.clone(), c.source_b.clone()],
            };
            chosen.confidence = BatteryConfidence::Unconfirmed;
            chosen.conflict_status = Some(c.clone());
        } else if valid_readings.len() >= 2 {
            chosen.confidence = BatteryConfidence::High;
        }

        log.push(format!(
            "Fonte escolhida para '{}': {} ({:?}) com confiança {:?}",
            device.name, chosen.source, chosen.percentage, chosen.confidence
        ));

        AggregatedResult {
            chosen,
            all_readings: valid_readings,
            discarded,
            conflict,
        }
    }
}

/// Helper function to determine TTL (time-to-live) in seconds per transport.
pub fn get_ttl_for_transport(transport: &ConnectionType) -> u64 {
    match transport {
        ConnectionType::Bluetooth => 60,
        ConnectionType::UsbCable | ConnectionType::UsbWired => 30,
        ConnectionType::TwoPointFourGhz => 30,
        ConnectionType::Dock => 30,
        _ => 60,
    }
}

/// Helper function to check if a battery telemetry provider source is compatible with the active transport.
pub fn is_provider_compatible_with_transport(
    source: BatterySource,
    transport: &ConnectionType,
) -> bool {
    match (source, transport) {
        // Bluetooth transport accepts BlueZ, BlueZ GATT, UPower, or Generic/Sysfs
        (BatterySource::BlueZ | BatterySource::BluezGatt, ConnectionType::Bluetooth) => true,
        (BatterySource::UPower | BatterySource::Sysfs, ConnectionType::Bluetooth) => true,

        // USB Cable transport accepts HID Standard, Sysfs, UPower, Rapoo Vendor HID
        (
            BatterySource::HidStandard
            | BatterySource::Sysfs
            | BatterySource::UPower
            | BatterySource::HidVendorSpecific,
            ConnectionType::UsbCable | ConnectionType::UsbWired,
        ) => true,

        // 2.4GHz Dongle transport accepts Rapoo Vendor HID, HID Standard, UPower
        (
            BatterySource::HidVendorSpecific | BatterySource::HidStandard | BatterySource::UPower,
            ConnectionType::TwoPointFourGhz,
        ) => true,

        // Dock transport accepts Sysfs, HID, UPower
        (
            BatterySource::Sysfs
            | BatterySource::HidStandard
            | BatterySource::UPower
            | BatterySource::HidVendorSpecific,
            ConnectionType::Dock,
        ) => true,

        // Unknown transport accepts all non-unavailable sources
        (_, ConnectionType::Unknown) => true,

        // Otherwise incompatible
        _ => false,
    }
}
