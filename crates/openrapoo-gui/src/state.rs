//! Application state management for OpenRapoo GPUI.

use crate::device_model::RapooDevice;
use crate::services::device_service::DeviceService;
use crate::services::{DaemonClient, ProfileService, SystemDeviceService};
use openrapoo_core::config::{ButtonAction, Profile};
use openrapoo_core::ipc::{ApplyProfileRequest, SetDpiRequest, SetPollingRateRequest};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use uuid::Uuid;

use crate::i18n::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    Buttons,
    Pointer,
    Device,
    Diagnostics,
}

impl DetailTab {
    pub fn label(&self, lang: Language) -> &'static str {
        match lang {
            Language::English => match self {
                DetailTab::Buttons => "Buttons",
                DetailTab::Pointer => "Pointer",
                DetailTab::Device => "Device",
                DetailTab::Diagnostics => "Diagnostics",
            },
            Language::Portuguese => match self {
                DetailTab::Buttons => "Botões",
                DetailTab::Pointer => "Ponteiro",
                DetailTab::Device => "Dispositivo",
                DetailTab::Diagnostics => "Diagnóstico",
            },
        }
    }

    #[allow(dead_code)]
    pub fn label_pt(&self) -> &'static str {
        self.label(Language::Portuguese)
    }
}

pub struct AppState {
    pub language: Language,
    pub settings: crate::settings::AppSettings,
    pub devices: Vec<RapooDevice>,
    pub selected_device_index: Option<usize>,
    pub active_tab: DetailTab,
    pub selected_hotspot_hex: Option<String>,
    pub active_profile: Profile,
    pub has_unsaved_changes: bool,
    #[allow(dead_code)]
    pub pressed_button_hex: Option<String>,
    pub show_diag_modal: bool,
    pub export_toast_message: Option<String>,
    pub device_service: Arc<SystemDeviceService>,
    pub profile_service: Arc<ProfileService>,
    pub daemon_client: Arc<DaemonClient>,
    pub daemon_running_cached: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        let settings_path = crate::settings::AppSettings::default_settings_path();
        let settings = crate::settings::AppSettings::load_from_file(&settings_path);

        let language = if settings_path.exists() {
            Language::from_locale_code(&settings.language)
        } else {
            Language::from_env()
        };

        let device_service = Arc::new(SystemDeviceService::new());
        let profile_service = Arc::new(ProfileService::new());
        let daemon_client = Arc::new(DaemonClient::new());
        let daemon_running_cached = Arc::new(AtomicBool::new(false));

        // Spawn daemon check non-blockingly in background thread so UI thread initializes instantly
        let client_clone = daemon_client.clone();
        let running_flag = daemon_running_cached.clone();
        std::thread::spawn(move || {
            let res = client_clone.ensure_daemon_running();
            if res.is_ok() {
                running_flag.store(true, Ordering::Relaxed);
            }
        });

        let devices = device_service.scan_devices();
        let active_profile = profile_service.get_active_profile();

        let selected_device_index = if !devices.is_empty() { Some(0) } else { None };

        Self {
            language,
            settings,
            devices,
            selected_device_index,
            active_tab: DetailTab::Buttons,
            selected_hotspot_hex: None,
            active_profile,
            has_unsaved_changes: false,
            pressed_button_hex: None,
            show_diag_modal: false,
            export_toast_message: None,
            device_service,
            profile_service,
            daemon_client,
            daemon_running_cached,
        }
    }

    pub fn toggle_language(&mut self) {
        self.set_language(self.language.toggle());
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = language;
        self.settings.language = self.language.locale_code().to_string();
        let settings_path = crate::settings::AppSettings::default_settings_path();
        if let Err(err) = self.settings.save_to_file(&settings_path) {
            tracing::warn!("Failed to persist settings: {err}");
        }
    }

    pub fn is_daemon_running(&self) -> bool {
        self.daemon_running_cached.load(Ordering::Relaxed)
    }

    pub fn refresh_devices(&mut self) {
        self.devices = self.device_service.scan_devices();
        if self.selected_device_index.is_some() && self.devices.is_empty() {
            self.selected_device_index = None;
        } else if self.selected_device_index.is_none() && !self.devices.is_empty() {
            self.selected_device_index = Some(0);
        }
    }

    pub fn selected_device(&self) -> Option<&RapooDevice> {
        self.selected_device_index.and_then(|i| self.devices.get(i))
    }

    pub fn select_device(&mut self, index: usize) {
        if index < self.devices.len() {
            self.selected_device_index = Some(index);
        }
    }

    pub fn select_hotspot(&mut self, hex_code: Option<String>) {
        self.selected_hotspot_hex = hex_code;
    }

    pub fn update_button_action(&mut self, hex_code: &str, action: ButtonAction) {
        if let Ok(()) = self.profile_service.update_action(hex_code, action) {
            self.active_profile = self.profile_service.get_active_profile();
            self.has_unsaved_changes = true;
            self.sync_profile_to_daemon();
        }
    }

    #[allow(dead_code)]
    pub fn switch_profile(&mut self, profile_id: Uuid) {
        if self.profile_service.set_active_profile(profile_id).is_ok() {
            self.active_profile = self.profile_service.get_active_profile();
            self.has_unsaved_changes = false;
            self.sync_profile_to_daemon();
        }
    }

    #[allow(dead_code)]
    pub fn create_profile(&mut self, name: &str) {
        if let Ok(prof) = self.profile_service.create_profile(name) {
            let _ = self.profile_service.set_active_profile(prof.id);
            self.active_profile = self.profile_service.get_active_profile();
            self.has_unsaved_changes = false;
            self.sync_profile_to_daemon();
        }
    }

    #[allow(dead_code)]
    pub fn duplicate_current_profile(&mut self, new_name: &str) {
        let current_id = self.active_profile.id;
        if let Ok(dup) = self.profile_service.duplicate_profile(current_id, new_name) {
            let _ = self.profile_service.set_active_profile(dup.id);
            self.active_profile = self.profile_service.get_active_profile();
            self.has_unsaved_changes = false;
            self.sync_profile_to_daemon();
        }
    }

    #[allow(dead_code)]
    pub fn delete_current_profile(&mut self) {
        let current_id = self.active_profile.id;
        if self.profile_service.delete_profile(current_id).is_ok() {
            self.active_profile = self.profile_service.get_active_profile();
            self.has_unsaved_changes = false;
            self.sync_profile_to_daemon();
        }
    }

    #[allow(dead_code)]
    pub fn refresh_selected_battery(&mut self) {
        if let Some(i) = self.selected_device_index {
            if let Some(dev) = self.devices.get_mut(i) {
                crate::services::BatteryService::refresh_battery(dev);
            }
        }
    }

    pub fn reset_button_action(&mut self, hex_code: &str) {
        if let Ok(()) = self.profile_service.reset_action(hex_code) {
            self.active_profile = self.profile_service.get_active_profile();
            self.has_unsaved_changes = true;
            self.sync_profile_to_daemon();
        }
    }

    pub fn update_dpi(&mut self, dpi: u32) {
        if let Ok(()) = self.profile_service.update_dpi(dpi) {
            self.active_profile = self.profile_service.get_active_profile();
            self.has_unsaved_changes = true;
            self.apply_hardware_dpi(dpi);
        }
    }

    pub fn update_polling_rate(&mut self, rate: u32) {
        if let Ok(()) = self.profile_service.update_polling_rate(rate) {
            self.active_profile = self.profile_service.get_active_profile();
            self.has_unsaved_changes = true;
            self.apply_hardware_polling_rate(rate);
        }
    }

    pub fn apply_hardware_dpi(&mut self, dpi: u32) {
        let device_id = self
            .selected_device()
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "Rapoo MT760 Pro".to_string());

        let req = SetDpiRequest {
            transaction_id: format!("dpi-{}", Uuid::new_v4()),
            device_id,
            dpi,
            gear: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        let daemon_client = self.daemon_client.clone();
        std::thread::spawn(move || {
            let _ = daemon_client.ensure_daemon_running();
            if let Ok(resp) = daemon_client.set_dpi_transaction(req) {
                tracing::info!("Hardware DPI response: {:?}", resp);
            }
        });
    }

    pub fn apply_hardware_polling_rate(&mut self, rate_hz: u32) {
        let device_id = self
            .selected_device()
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "Rapoo MT760 Pro".to_string());

        let req = SetPollingRateRequest {
            transaction_id: format!("polling-{}", Uuid::new_v4()),
            device_id,
            rate_hz,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        let daemon_client = self.daemon_client.clone();
        std::thread::spawn(move || {
            let _ = daemon_client.ensure_daemon_running();
            if let Ok(resp) = daemon_client.set_polling_rate_transaction(req) {
                tracing::info!("Hardware Polling Rate response: {:?}", resp);
            }
        });
    }

    pub fn apply_hardware_settings(&mut self) {
        let dpi = self.active_profile.dpi;
        let polling_rate = self.active_profile.polling_rate;
        self.apply_hardware_dpi(dpi);
        self.apply_hardware_polling_rate(polling_rate);
        self.export_toast_message =
            Some("⚡ Configurações de DPI e Polling enviadas ao mouse!".to_string());
    }

    pub fn restore_hardware_snapshot(&mut self) {
        let device_id = self
            .selected_device()
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "Rapoo MT760 Pro".to_string());

        let daemon_client = self.daemon_client.clone();
        std::thread::spawn(move || {
            let _ = daemon_client.ensure_daemon_running();
            if let Ok(resp) = daemon_client.restore_snapshot(&device_id) {
                tracing::info!("Hardware Snapshot Restore response: {:?}", resp);
            }
        });
        self.export_toast_message = Some("🔄 Snapshot de hardware restaurado!".to_string());
    }

    pub fn read_hardware_state(&mut self) {
        let device_id = self
            .selected_device()
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "Rapoo MT760 Pro".to_string());

        let daemon_client = self.daemon_client.clone();
        std::thread::spawn(move || {
            let _ = daemon_client.ensure_daemon_running();
            if let Ok((dpi, rate, _transport)) = daemon_client.read_hardware_state(&device_id) {
                tracing::info!("Hardware State Read: DPI={dpi} Rate={rate}");
            }
        });
        self.export_toast_message = Some("📡 Leitura do hardware efetuada!".to_string());
    }

    /// Dispatch active profile updates to the daemon asynchronously in a background thread.
    /// Ensures the GPUI main UI thread never blocks or freezes on socket IPC / sleeps.
    pub fn sync_profile_to_daemon(&mut self) {
        self.export_toast_message =
            Some("✓ Configuração aplicada pelo OpenRapoo no Linux (evdev/uinput)".to_string());
        self.has_unsaved_changes = false;

        let req = ApplyProfileRequest {
            transaction_id: format!("tx-{}", Uuid::new_v4()),
            device_name: self
                .selected_device()
                .map(|d| d.name.clone())
                .unwrap_or_else(|| "Rapoo MT760 Pro".to_string()),
            profile_id: self.active_profile.id,
            profile_version: 1,
            actions: self.active_profile.mappings.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        let daemon_client = self.daemon_client.clone();

        std::thread::spawn(move || {
            let mut res = daemon_client.apply_profile_transaction(req.clone());
            if res.is_err() {
                if let Ok(()) = daemon_client.ensure_daemon_running() {
                    res = daemon_client.apply_profile_transaction(req);
                }
            }

            match res {
                Ok(resp) => {
                    tracing::info!("Daemon IPC transaction applied: {:?}", resp.status);
                }
                Err(e) => {
                    tracing::warn!("Daemon IPC transaction warning: {e}");
                }
            }
        });
    }

    pub fn open_diag_modal(&mut self) {
        self.show_diag_modal = true;
        self.export_toast_message = None;
    }

    pub fn close_diag_modal(&mut self) {
        self.show_diag_modal = false;
        self.export_toast_message = None;
    }

    pub fn export_diag_report(&mut self, markdown_content: &str) {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        let target_path = home.join("openrapoo-diagnostic-report.md");
        if std::fs::write(&target_path, markdown_content).is_ok() {
            self.export_toast_message =
                Some(format!("✓ Relatório salvo em {}", target_path.display()));
        } else {
            self.export_toast_message = Some("Erro ao salvar relatório.".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_toggle_language() {
        let mut app_state = AppState::new();
        app_state.set_language(Language::English);
        assert_eq!(app_state.language, Language::English);
        assert_eq!(app_state.settings.language, "en-US");

        app_state.toggle_language();
        assert_eq!(app_state.language, Language::Portuguese);
        assert_eq!(app_state.settings.language, "pt-BR");
    }
}
