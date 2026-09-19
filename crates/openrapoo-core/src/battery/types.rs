//! Data types and models for battery readings, sources, confidence levels, and compatibility wrappers.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Source provider of the battery reading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatterySource {
    UPower,
    BlueZ,
    BluezGatt,
    Sysfs,
    HidStandard,
    HidVendorSpecific,
    Unavailable,
    Unknown,
}

impl fmt::Display for BatterySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatterySource::UPower => f.write_str("UPower"),
            BatterySource::BlueZ => f.write_str("BlueZ"),
            BatterySource::BluezGatt => f.write_str("BlueZ GATT"),
            BatterySource::Sysfs => f.write_str("sysfs"),
            BatterySource::HidStandard => f.write_str("HID Padrão"),
            BatterySource::HidVendorSpecific => f.write_str("Rapoo Vendor HID"),
            BatterySource::Unavailable => f.write_str("Indisponível"),
            BatterySource::Unknown => f.write_str("Desconhecida"),
        }
    }
}

/// Confidence level assigned to a battery reading after validation and cross-referencing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BatteryConfidence {
    /// Two or more independent, consistent sources agree
    High,
    /// A single trusted provider with valid, fresh data
    Medium,
    /// A single provider with partial or unconfirmed data
    Low,
    /// Reading is suspicious (e.g. 0% without confirmation)
    Unconfirmed,
    /// Reading was rejected during validation
    Rejected,
    /// No valid reading available
    #[default]
    None,
}

impl fmt::Display for BatteryConfidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatteryConfidence::High => f.write_str("Alta (Confirmada)"),
            BatteryConfidence::Medium => f.write_str("Média (Confiável)"),
            BatteryConfidence::Low => f.write_str("Baixa"),
            BatteryConfidence::Unconfirmed => f.write_str("Não confirmada"),
            BatteryConfidence::Rejected => f.write_str("Rejeitada (Inválida)"),
            BatteryConfidence::None => f.write_str("Nenhuma"),
        }
    }
}

/// Validity status explaining why a reading was accepted, flagged, or rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatteryValidity {
    Valid,
    StaleReading { age_seconds: u64 },
    ZeroUnconfirmed,
    ConflictingReadings { sources: Vec<BatterySource> },
    DeviceDisconnected,
    PermissionDenied,
    ServiceUnavailable,
    Invalid { reason: String },
}

/// Operational state of the device battery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatteryState {
    Available,
    Charging,
    Full,
    Low,
    Critical,
    Unknown,
    Unavailable,
    StaleReading,
    ConflictingReading,
    DeviceDisconnected,
}

/// Information describing a conflict between two battery telemetry sources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub source_a: BatterySource,
    pub value_a: u8,
    pub source_b: BatterySource,
    pub value_b: u8,
}

/// Raw low-level hardware or DBus details associated with a reading.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawProviderData {
    pub upower_path: Option<String>,
    pub bluez_path: Option<String>,
    pub hidraw_path: Option<String>,
    pub sysfs_path: Option<String>,
    pub hid_uniq: Option<String>,
    pub parent_usb: Option<String>,
    pub report_id: Option<u8>,
    pub bluetooth_address: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
}

/// A comprehensive battery reading returned by a provider or aggregator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatteryReading {
    pub percentage: Option<u8>,
    pub state: BatteryState,
    pub is_present: bool,
    pub charging: Option<bool>,
    pub source: BatterySource,
    pub timestamp: u64,
    pub device_identifier: String,
    pub connection: String,
    /// Overall composite confidence level
    pub confidence: BatteryConfidence,
    /// Confidence level of matching the physical device (MAC, Serial, USB phys)
    pub device_match_confidence: BatteryConfidence,
    /// Confidence level of the percentage value itself
    pub reading_confidence: BatteryConfidence,
    /// True if percentage value passed validation checks
    pub reading_valid: bool,
    /// True if reading age exceeds transport TTL
    pub is_stale: bool,
    pub validity: BatteryValidity,
    pub invalidation_reason: Option<String>,
    pub alternative_source: Option<BatterySource>,
    pub conflict_status: Option<ConflictInfo>,
    pub raw_data: Option<RawProviderData>,
}

impl Default for BatteryReading {
    fn default() -> Self {
        Self {
            percentage: None,
            state: BatteryState::Unknown,
            is_present: false,
            charging: None,
            source: BatterySource::Unknown,
            timestamp: 0,
            device_identifier: String::new(),
            connection: String::new(),
            confidence: BatteryConfidence::None,
            device_match_confidence: BatteryConfidence::None,
            reading_confidence: BatteryConfidence::None,
            reading_valid: false,
            is_stale: false,
            validity: BatteryValidity::Invalid {
                reason: "Default".to_string(),
            },
            invalidation_reason: None,
            alternative_source: None,
            conflict_status: None,
            raw_data: None,
        }
    }
}

impl BatteryReading {
    pub fn display_text_pt(&self) -> String {
        match &self.validity {
            BatteryValidity::ZeroUnconfirmed => "Bateria: Leitura não confirmada".to_string(),
            BatteryValidity::ConflictingReadings { .. } => "Bateria: Leitura conflitante".to_string(),
            BatteryValidity::StaleReading { age_seconds } => {
                let mins = age_seconds / 60;
                if let Some(pct) = self.percentage {
                    if mins > 0 {
                        format!("Bateria: Última leitura {pct}%, há {mins} min")
                    } else {
                        format!("Bateria: Última leitura {pct}%")
                    }
                } else {
                    "Bateria: Leitura antiga".to_string()
                }
            }
            BatteryValidity::DeviceDisconnected => "Bateria: Dispositivo desconectado".to_string(),
            BatteryValidity::Invalid { reason } => format!("Bateria: Indisponível ({reason})"),
            _ => {
                if let Some(pct) = self.percentage {
                    if self.charging.unwrap_or(false) {
                        format!("Bateria: {pct}% (Carregando ⚡)")
                    } else {
                        format!("Bateria: {pct}%")
                    }
                } else if let Some(ref reason) = self.invalidation_reason {
                    format!("Bateria: Indisponível ({reason})")
                } else {
                    "Bateria: Indisponível".to_string()
                }
            }
        }
    }

    pub fn tooltip_pt(&self) -> String {
        let mut lines = Vec::new();

        if let Some(pct) = self.percentage {
            lines.push(format!("Nível de Bateria: {pct}%"));
        } else {
            lines.push("Nível de Bateria: Não disponível".to_string());
        }

        lines.push(format!("Origem: {}", self.source));
        lines.push(format!("Confiança: {}", self.confidence));

        if let Some(chg) = self.charging {
            lines.push(format!("Estado de Carga: {}", if chg { "Carregando ⚡" } else { "Descarregando" }));
        }

        if let Some(ref reason) = self.invalidation_reason {
            lines.push(format!("Nota: {reason}"));
        }

        lines.join("\n")
    }
}

/// External or internal power source supplying energy to the device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerSource {
    UsbCable,
    Dock,
    Bluetooth,
    Dongle,
    Unknown,
}

impl fmt::Display for PowerSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PowerSource::UsbCable => f.write_str("cabo USB"),
            PowerSource::Dock => f.write_str("dock"),
            PowerSource::Bluetooth => f.write_str("Bluetooth"),
            PowerSource::Dongle => f.write_str("dongle 2.4 GHz"),
            PowerSource::Unknown => f.write_str("fonte desconhecida"),
        }
    }
}

/// Detailed charging information including active power source and evidence string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChargingInfo {
    pub is_charging: bool,
    pub power_source: PowerSource,
    pub evidence: String,
}

/// Status wrapper representing validated device battery states.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatteryStatus {
    Available {
        percentage: u8,
        charging: bool,
        #[serde(default)]
        source: Option<BatterySource>,
        #[serde(default)]
        timestamp: Option<u64>,
        #[serde(default)]
        diagnostic_message: Option<String>,
    },
    Charging {
        percentage: Option<u8>,
        source: PowerSource,
    },
    Discharging {
        percentage: u8,
    },
    Full,
    Unavailable {
        reason: String,
        #[serde(default)]
        source: Option<BatterySource>,
    },
    Stale {
        last_percentage: Option<u8>,
        age_seconds: u64,
    },
    Invalid {
        reason: String,
    },
    #[default]
    Unknown,
}

impl BatteryStatus {
    pub fn percentage(&self) -> Option<u8> {
        match self {
            BatteryStatus::Available { percentage, .. } => Some(*percentage),
            BatteryStatus::Charging { percentage, .. } => *percentage,
            BatteryStatus::Discharging { percentage } => Some(*percentage),
            BatteryStatus::Full => Some(100),
            BatteryStatus::Stale { last_percentage, .. } => *last_percentage,
            _ => None,
        }
    }

    pub fn is_charging(&self) -> bool {
        match self {
            BatteryStatus::Available { charging, .. } => *charging,
            BatteryStatus::Charging { .. } => true,
            BatteryStatus::Full => true,
            _ => false,
        }
    }

    pub fn source(&self) -> BatterySource {
        match self {
            BatteryStatus::Available { source, .. } => source.clone().unwrap_or(BatterySource::Unknown),
            BatteryStatus::Unavailable { source, .. } => source.clone().unwrap_or(BatterySource::Unavailable),
            _ => BatterySource::Unknown,
        }
    }

    pub fn display_text_pt(&self) -> String {
        match self {
            BatteryStatus::Available { percentage, charging, .. } => {
                if *charging {
                    format!("{percentage}% · Carregando ⚡")
                } else {
                    format!("{percentage}%")
                }
            }
            BatteryStatus::Charging { percentage: Some(pct), source: PowerSource::Unknown } => {
                format!("{pct}% · Carregando ⚡")
            }
            BatteryStatus::Charging { percentage: Some(pct), source } => {
                format!("{pct}% · Carregando via {source} ⚡")
            }
            BatteryStatus::Charging { percentage: None, source: PowerSource::Unknown } => {
                "Carregamento detectado · percentual indisponível".to_string()
            }
            BatteryStatus::Charging { percentage: None, source } => {
                format!("Carregamento detectado via {source} · percentual indisponível")
            }
            BatteryStatus::Discharging { percentage } => {
                format!("{percentage}% · Em uso")
            }
            BatteryStatus::Full => "100% · Completa".to_string(),
            BatteryStatus::Stale { last_percentage: Some(pct), age_seconds } => {
                format!("Leitura antiga ({pct}%, há {age_seconds}s)")
            }
            BatteryStatus::Stale { last_percentage: None, .. } => {
                "Leitura antiga (expirada)".to_string()
            }
            BatteryStatus::Unavailable { reason, .. } => format!("Bateria indisponível ({reason})"),
            BatteryStatus::Invalid { reason } => format!("Bateria indisponível ({reason})"),
            BatteryStatus::Unknown => "Bateria indisponível".to_string(),
        }
    }

    pub fn display_text_en(&self) -> String {
        match self {
            BatteryStatus::Available { percentage, charging, .. } => {
                if *charging {
                    format!("{percentage}% (Charging ⚡)")
                } else {
                    format!("{percentage}%")
                }
            }
            BatteryStatus::Charging { percentage: Some(pct), .. } => {
                format!("{pct}% (Charging ⚡)")
            }
            BatteryStatus::Charging { percentage: None, .. } => {
                "Charging detected (percentage unavailable)".to_string()
            }
            BatteryStatus::Discharging { percentage } => {
                format!("{percentage}% (In use)")
            }
            BatteryStatus::Full => "100% (Full)".to_string(),
            BatteryStatus::Stale { .. } => "Battery: Stale reading".to_string(),
            BatteryStatus::Unavailable { reason, .. } => format!("Battery: Unavailable ({reason})"),
            BatteryStatus::Invalid { reason } => format!("Battery: Invalid ({reason})"),
            BatteryStatus::Unknown => "Battery: Not Available".to_string(),
        }
    }

    pub fn tooltip_pt(&self) -> String {
        match self {
            BatteryStatus::Available { percentage, charging, source, diagnostic_message, .. } => {
                let src_str = source.as_ref().map(|s| s.to_string()).unwrap_or_else(|| "Desconhecida".to_string());
                let state_str = if *charging { "Carregando ⚡" } else { "Em uso" };
                let msg = diagnostic_message.as_deref().unwrap_or("");

                if msg.is_empty() {
                    format!("Nível: {percentage}%\nEstado: {state_str}\nOrigem: {src_str}")
                } else {
                    format!("Nível: {percentage}%\nEstado: {state_str}\nOrigem: {src_str}\nInfo: {msg}")
                }
            }
            BatteryStatus::Charging { percentage, source } => {
                let pct_str = percentage.map(|p| format!("{p}%")).unwrap_or_else(|| "N/A".to_string());
                format!("Nível: {pct_str}\nEstado: Carregando ⚡\nFonte de Energia: {source}")
            }
            BatteryStatus::Discharging { percentage } => {
                format!("Nível: {percentage}%\nEstado: Em uso")
            }
            BatteryStatus::Full => "Nível: 100%\nEstado: Bateria Completa".to_string(),
            BatteryStatus::Stale { last_percentage, age_seconds } => {
                let pct_str = last_percentage.map(|p| format!("{p}%")).unwrap_or_else(|| "N/A".to_string());
                format!("Bateria Antiga\nÚltimo Nível: {pct_str}\nIdade: {age_seconds} segundos")
            }
            BatteryStatus::Unavailable { reason, source } => {
                let src_str = source.as_ref().map(|s| s.to_string()).unwrap_or_else(|| "N/A".to_string());
                format!("Bateria Indisponível\nMotivo: {reason}\nOrigem: {src_str}")
            }
            BatteryStatus::Invalid { reason } => {
                format!("Leitura Inválida\nMotivo: {reason}")
            }
            BatteryStatus::Unknown => "Status de bateria desconhecido".to_string(),
        }
    }
}

impl From<BatteryReading> for BatteryStatus {
    fn from(reading: BatteryReading) -> Self {
        match reading.validity {
            BatteryValidity::Valid => {
                if reading.charging == Some(true) {
                    BatteryStatus::Charging {
                        percentage: reading.percentage,
                        source: PowerSource::Unknown,
                    }
                } else if let Some(pct) = reading.percentage {
                    BatteryStatus::Available {
                        percentage: pct,
                        charging: false,
                        source: Some(reading.source),
                        timestamp: Some(reading.timestamp),
                        diagnostic_message: reading.invalidation_reason,
                    }
                } else {
                    BatteryStatus::Unavailable {
                        reason: reading.invalidation_reason.unwrap_or_else(|| "Bateria sem porcentagem informada".to_string()),
                        source: Some(reading.source),
                    }
                }
            }
            BatteryValidity::ZeroUnconfirmed => BatteryStatus::Unavailable {
                reason: "Leitura de 0% não confirmada por fonte secundária".to_string(),
                source: Some(reading.source),
            },
            BatteryValidity::ConflictingReadings { .. } => BatteryStatus::Unavailable {
                reason: "Conflito entre fontes de bateria".to_string(),
                source: Some(reading.source),
            },
            BatteryValidity::StaleReading { age_seconds } => BatteryStatus::Stale {
                last_percentage: reading.percentage,
                age_seconds,
            },
            BatteryValidity::DeviceDisconnected => BatteryStatus::Unavailable {
                reason: "Dispositivo desconectado".to_string(),
                source: Some(reading.source),
            },
            BatteryValidity::PermissionDenied => BatteryStatus::Unavailable {
                reason: "Permissão negada ao acessar dispositivo".to_string(),
                source: Some(reading.source),
            },
            BatteryValidity::ServiceUnavailable => BatteryStatus::Unavailable {
                reason: "Serviço de bateria indisponível".to_string(),
                source: Some(reading.source),
            },
            BatteryValidity::Invalid { reason } => BatteryStatus::Invalid { reason },
        }
    }
}
