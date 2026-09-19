//! Diagnostic report generator for battery telemetry, environment analysis, and source verification.

use serde::{Deserialize, Serialize};

use super::environment::{collect_system_environment, SystemEnvironment};
use super::provider::current_epoch_seconds;
use super::types::{BatteryReading, BatterySource, BatteryStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryDiagnosticReport {
    pub device_name: String,
    pub phys_path: Option<String>,
    pub hidraw_path: Option<String>,
    pub environment: SystemEnvironment,
    pub consulted_sources: Vec<String>,
    pub active_source: BatterySource,
    pub raw_status: BatteryStatus,
    pub chosen_reading: BatteryReading,
    pub all_readings: Vec<BatteryReading>,
    pub discarded_readings: Vec<(BatteryReading, String)>,
    pub diagnostic_log: Vec<String>,
    pub timestamp: u64,
}

impl BatteryDiagnosticReport {
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        let pct_raw = self
            .chosen_reading
            .percentage
            .map(|p| format!("{p}%"))
            .unwrap_or_else(|| "N/A / Ausente".to_string());
        let validation_str = if self.chosen_reading.reading_valid {
            "Válida / Aceita"
        } else {
            "Rejeitada / Invalidade"
        };
        let rej_reason = self
            .chosen_reading
            .invalidation_reason
            .as_deref()
            .unwrap_or("Nenhum motivo específico registrado");

        md.push_str("# 🔋 Relatório Técnico de Diagnóstico de Bateria — OpenRapoo\n\n");
        md.push_str(&format!("- **Dispositivo Target**: {}\n", self.device_name));
        md.push_str(&format!("- **Data/Hora**: Unix Epoch {}\n", self.timestamp));
        md.push_str(&format!(
            "- **Transporte Ativo**: {}\n",
            self.chosen_reading.connection
        ));
        md.push_str(&format!(
            "- **Fonte Encontrada**: {}\n",
            self.chosen_reading.source
        ));
        md.push_str(&format!(
            "- **Confiança da Associação**: {}\n",
            self.chosen_reading.device_match_confidence
        ));
        md.push_str(&format!("- **Percentual Bruto**: {}\n", pct_raw));
        md.push_str(&format!(
            "- **Validação do Percentual**: {}\n",
            validation_str
        ));
        md.push_str(&format!(
            "- **Confiança da Leitura**: {}\n",
            self.chosen_reading.reading_confidence
        ));
        if !self.chosen_reading.reading_valid {
            md.push_str(&format!("- **Motivo da Rejeição**: {}\n", rej_reason));
        }
        md.push_str(&format!(
            "- **Resultado Final Exibido**: {}\n\n",
            self.chosen_reading.display_text_pt()
        ));

        md.push_str("---\n\n");
        md.push_str("## 🖥️ 1. Ambiente do Sistema e Hardware\n\n");
        md.push_str(&format!(
            "- **Distribuição**: {}\n",
            self.environment.distro
        ));
        md.push_str(&format!(
            "- **Kernel**: {}\n",
            self.environment.kernel_version
        ));
        md.push_str(&format!(
            "- **Ambiente Gráfico**: {} ({})\n",
            self.environment.desktop_environment, self.environment.session_type
        ));
        md.push_str(&format!(
            "- **UPower Versão**: {}\n",
            self.environment
                .upower_version
                .as_deref()
                .unwrap_or("Não disponível")
        ));
        md.push_str(&format!(
            "- **BlueZ Versão**: {}\n",
            self.environment
                .bluez_version
                .as_deref()
                .unwrap_or("Não disponível")
        ));
        md.push_str(&format!(
            "- **Controladores Bluetooth**: {:?}\n",
            self.environment.bluetooth_controllers
        ));
        md.push_str(&format!(
            "- **Grupos do Usuário**: {:?}\n\n",
            self.environment.user_groups
        ));

        md.push_str("### Dispositivos HID e Interfaces Detectadas\n");
        for hid in &self.environment.hidraw_interfaces {
            md.push_str(&format!("- {}\n", hid));
        }
        md.push('\n');

        md.push_str("---\n\n");
        md.push_str("## 🔍 2. Provedores Consultados e Validação Cruzada\n\n");

        if self.all_readings.is_empty() {
            md.push_str("> [!WARNING]\n> Nenhuma leitura válida foi obtida de provedores primários ou secundários.\n\n");
        } else {
            for r in &self.all_readings {
                let pct_str = r
                    .percentage
                    .map(|p| format!("{p}%"))
                    .unwrap_or_else(|| "N/A".to_string());
                md.push_str(&format!(
                    "- **{}**: {} | Confiança: {} | Conexão: {}\n",
                    r.source, pct_str, r.confidence, r.connection
                ));
            }
            md.push('\n');
        }

        if !self.discarded_readings.is_empty() {
            md.push_str("### Leituras Descartadas ou Invalidadas\n\n");
            for (r, reason) in &self.discarded_readings {
                md.push_str(&format!(
                    "- **{}**: {}\n  - *Motivo*: {}\n",
                    r.source,
                    r.percentage
                        .map(|p| format!("{p}%"))
                        .unwrap_or_else(|| "N/A".to_string()),
                    reason
                ));
            }
            md.push('\n');
        }

        if let Some(ref conflict) = self.chosen_reading.conflict_status {
            md.push_str("> [!CAUTION]\n");
            md.push_str(&format!(
                "> **Conflito entre Fontes Detectado**: Fonte {} ({}%) vs Fonte {} ({}%)\n",
                conflict.source_a, conflict.value_a, conflict.source_b, conflict.value_b
            ));
            md.push_str("> A aplicação exibiu a leitura como não confirmada para evitar apresentar dados incorretos ao usuário.\n\n");
        }

        md.push_str("---\n\n");
        md.push_str("## 📋 3. Log de Associação e Justificativas das Regras\n\n");
        md.push_str("```text\n");
        for line in &self.diagnostic_log {
            md.push_str(line);
            md.push('\n');
        }
        md.push_str("```\n\n");

        md.push_str("---\n\n");
        md.push_str("## 💡 4. Explicações das Decisões de Regra\n\n");
        md.push_str("- **Aceitação de 80% (Teclado)**: Aceito se `IsPresent == true`, leitura recente, e associado corretamente por MAC/modelo.\n");
        md.push_str("- **Rejeição de 0% Suspeito (Mouse)**: Valor 0% isolado com `state: unknown` no UPower/BlueZ é desconsiderado até confirmação secundária.\n");
        md.push_str("- **Desconsideração do IconName**: O campo `icon-name` do freedesktop (ex: `battery-missing-symbolic`) é ignorado como fonte principal de verdade.\n");
        md.push_str("- **Dongle 2.4GHz**: Varre hidraw dinamicamente e consulta via Report ID `0x07` (Categoria `0x01`).\n");

        md
    }
}

pub fn generate_battery_diagnostic_report(
    device_name: &str,
    transport: crate::device::ConnectionType,
    phys: Option<&str>,
    hidraw_path: Option<&std::path::Path>,
) -> BatteryDiagnosticReport {
    use super::aggregator::BatteryAggregator;
    use super::provider::DeviceIdentity;

    let mut log = Vec::new();
    let env = collect_system_environment();

    let identity = DeviceIdentity {
        name: device_name.to_string(),
        transport,
        phys: phys.map(|s| s.to_string()),
        hidraw_path: hidraw_path.map(|p| p.to_path_buf()),
        ..Default::default()
    };

    let aggregator = BatteryAggregator::new();
    let result = aggregator.query_all(&identity, &mut log);

    let raw_status: BatteryStatus = result.chosen.clone().into();

    BatteryDiagnosticReport {
        device_name: device_name.to_string(),
        phys_path: phys.map(|s| s.to_string()),
        hidraw_path: hidraw_path.map(|p| p.to_string_lossy().to_string()),
        environment: env,
        consulted_sources: vec![
            "BlueZ D-Bus".into(),
            "BlueZ GATT".into(),
            "UPower D-Bus".into(),
            "HID Standard".into(),
            "Rapoo Vendor HID (0x07)".into(),
            "Sysfs".into(),
        ],
        active_source: result.chosen.source.clone(),
        raw_status,
        chosen_reading: result.chosen,
        all_readings: result.all_readings,
        discarded_readings: result.discarded,
        diagnostic_log: log,
        timestamp: current_epoch_seconds(),
    }
}
