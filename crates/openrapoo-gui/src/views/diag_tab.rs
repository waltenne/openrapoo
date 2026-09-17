//! "Diagnóstico" (Diagnostics) Tab View Component.

use openrapoo_core::device::{detect_rapoo_devices, RapooDevice};

#[derive(Debug, Clone)]
pub struct DiagTabState {
    pub devices: Vec<RapooDevice>,
    pub live_event_log: Vec<String>,
}

impl Default for DiagTabState {
    fn default() -> Self {
        DiagTabState {
            devices: detect_rapoo_devices().unwrap_or_default(),
            live_event_log: Vec::new(),
        }
    }
}

impl DiagTabState {
    pub fn add_log_event(&mut self, event_str: impl Into<String>) {
        self.live_event_log.push(event_str.into());
        if self.live_event_log.len() > 100 {
            self.live_event_log.remove(0);
        }
    }

    pub fn generate_summary_text(&self) -> String {
        let status = openrapoo_core::permissions::check_input_group_status();
        let config_path = openrapoo_core::config::ProfileStore::default_config_path();

        let mut out = String::new();
        out.push_str("=== Relatório de Diagnóstico OpenRapoo ===\n");
        out.push_str(&format!("Versão: v{}\n", env!("CARGO_PKG_VERSION")));
        out.push_str(&format!("Grupo input: {}\n", status.display_message_pt()));
        out.push_str(&format!("Caminho config: {}\n", config_path.display()));
        out.push_str(&format!("Dispositivos detectados: {}\n", self.devices.len()));

        for (i, dev) in self.devices.iter().enumerate() {
            out.push_str(&format!(
                "  [{}] {} (0x{:04X}:0x{:04X}) - {}\n",
                i + 1,
                dev.name,
                dev.vendor_id,
                dev.product_id,
                dev.connection
            ));
            if let Some(ref ev) = dev.evdev_path {
                out.push_str(&format!("      evdev: {}\n", ev.display()));
            }
        }
        out
    }
}

