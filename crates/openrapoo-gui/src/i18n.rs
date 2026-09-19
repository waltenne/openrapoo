//! Internationalization (i18n) module for OpenRapoo.
//! Supports English (en-US) and Portuguese (pt-BR) with automatic system locale detection,
//! persistent user preferences, and structured domain catalogs.
#![allow(dead_code)]

/// Supported application languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    Portuguese,
    English,
}

impl Language {
    /// Detects default language from system environment variables (LC_ALL, LC_MESSAGES, LANG).
    /// Defaults to `Portuguese` (pt-BR) if locale is Brazilian Portuguese or system is configured in Portuguese,
    /// otherwise defaults to `English` (en-US).
    pub fn from_env() -> Self {
        let locale = std::env::var("LC_ALL")
            .or_else(|_| std::env::var("LC_MESSAGES"))
            .or_else(|_| std::env::var("LANG"))
            .unwrap_or_default()
            .to_lowercase();

        if locale.starts_with("pt") {
            Language::Portuguese
        } else if !locale.is_empty()
            && (locale.starts_with("en") || locale != "c" && locale != "posix")
        {
            Language::English
        } else {
            Language::Portuguese
        }
    }

    /// Parses a locale code string (e.g. "pt-BR", "pt_BR", "pt", "en-US", "en").
    /// Falls back safely to `Language::Portuguese` if unrecognized.
    pub fn from_locale_code(code: &str) -> Self {
        let clean = code.trim().to_lowercase().replace('_', "-");
        if clean.starts_with("pt") {
            Language::Portuguese
        } else if clean.starts_with("en") {
            Language::English
        } else {
            Language::Portuguese
        }
    }

    /// Returns standard BCP 47 locale code representation (e.g. "pt-BR" or "en-US").
    pub fn locale_code(&self) -> &'static str {
        match self {
            Language::English => "en-US",
            Language::Portuguese => "pt-BR",
        }
    }

    /// Returns short uppercase badge label for UI toggle buttons (e.g. "EN" or "PT").
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "EN",
            Language::Portuguese => "PT",
        }
    }

    /// Human readable name of the language in its native script.
    pub fn label(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Portuguese => "Português",
        }
    }

    /// Toggles between available languages.
    pub fn toggle(&self) -> Self {
        match self {
            Language::English => Language::Portuguese,
            Language::Portuguese => Language::English,
        }
    }
}

// ============================================================================
// DOMAIN CATALOG: NAVIGATION
// ============================================================================

pub fn back_to_devices(lang: Language) -> &'static str {
    match lang {
        Language::English => "‹ Devices",
        Language::Portuguese => "‹ Dispositivos",
    }
}

pub fn tab_buttons(lang: Language) -> &'static str {
    match lang {
        Language::English => "Buttons",
        Language::Portuguese => "Botões",
    }
}

pub fn tab_pointer(lang: Language) -> &'static str {
    match lang {
        Language::English => "Pointer",
        Language::Portuguese => "Ponteiro",
    }
}

pub fn tab_device(lang: Language) -> &'static str {
    match lang {
        Language::English => "Device",
        Language::Portuguese => "Dispositivo",
    }
}

pub fn tab_diagnostics(lang: Language) -> &'static str {
    match lang {
        Language::English => "Diagnostics",
        Language::Portuguese => "Diagnóstico",
    }
}

// ============================================================================
// DOMAIN CATALOG: STATUS BAR / FOOTER
// ============================================================================

pub fn refresh(lang: Language) -> &'static str {
    match lang {
        Language::English => "Refresh",
        Language::Portuguese => "Atualizar",
    }
}

pub fn input_group_ok(lang: Language) -> &'static str {
    match lang {
        Language::English => "Input group OK",
        Language::Portuguese => "Grupo input OK",
    }
}

pub fn input_group_no_access(lang: Language) -> &'static str {
    match lang {
        Language::English => "No input group access",
        Language::Portuguese => "Sem acesso ao grupo input",
    }
}

pub fn daemon_active(lang: Language) -> &'static str {
    match lang {
        Language::English => "Daemon Active",
        Language::Portuguese => "Daemon Ativo",
    }
}

pub fn daemon_inactive(lang: Language) -> &'static str {
    match lang {
        Language::English => "Daemon Inactive",
        Language::Portuguese => "Daemon Inativo",
    }
}

pub fn profile_prefix(lang: Language) -> &'static str {
    match lang {
        Language::English => "Profile: ",
        Language::Portuguese => "Perfil: ",
    }
}

pub fn format_profile_name(name: &str, lang: Language) -> String {
    if name.starts_with("Padrão") || name.starts_with("Default") {
        match lang {
            Language::English => "Default (Passthrough)".to_string(),
            Language::Portuguese => "Padrão (Passthrough)".to_string(),
        }
    } else {
        name.to_string()
    }
}

// ============================================================================
// DOMAIN CATALOG: DEVICES LIST PAGE
// ============================================================================

pub fn no_device_title(lang: Language) -> &'static str {
    match lang {
        Language::English => "No Rapoo Devices Detected",
        Language::Portuguese => "Nenhum Dispositivo Rapoo Detectado",
    }
}

pub fn no_device_desc(lang: Language) -> &'static str {
    match lang {
        Language::English => "Ensure that the 2.4 GHz USB receiver or Bluetooth connection for the Rapoo MT760 Pro is active.",
        Language::Portuguese => "Certifique-se de que o receptor 2.4 GHz USB ou a conexão Bluetooth do mouse Rapoo MT760 Pro esteja ativa.",
    }
}

pub fn try_detect_again(lang: Language) -> &'static str {
    match lang {
        Language::English => "↻ Try Detecting Again",
        Language::Portuguese => "↻ Tentar Detectar Novamente",
    }
}

pub fn navigation_hint(lang: Language) -> &'static str {
    match lang {
        Language::English => "💡 Navigation: Use ❮ ❯ on screen, arrows ⬅️ ➡️ or Scroll to cycle • Click card or press Enter to configure",
        Language::Portuguese => "💡 Navegação: Use ❮ ❯ na tela, setas ⬅️ ➡️ ou Scroll para alternar no carrossel • Clique no card ou Enter para configurar",
    }
}

pub fn configure_device(lang: Language) -> &'static str {
    match lang {
        Language::English => "Configure Device",
        Language::Portuguese => "Configurar Dispositivo",
    }
}

pub fn connected_devices_count(lang: Language, count: usize) -> String {
    match lang {
        Language::English => format!("Connected Devices ({count})"),
        Language::Portuguese => format!("Dispositivos Conectados ({count})"),
    }
}

// Connection badges
pub fn conn_24ghz(lang: Language) -> &'static str {
    match lang {
        Language::English => "2.4 GHz USB",
        Language::Portuguese => "2.4 GHz USB",
    }
}

pub fn conn_bluetooth(lang: Language) -> &'static str {
    match lang {
        Language::English => "Bluetooth",
        Language::Portuguese => "Bluetooth",
    }
}

pub fn conn_disconnected(lang: Language) -> &'static str {
    match lang {
        Language::English => "Disconnected",
        Language::Portuguese => "Desconectado",
    }
}

// ============================================================================
// DOMAIN CATALOG: POINTER & PERFORMANCE PAGE
// ============================================================================

pub fn pointer_page_title(lang: Language) -> &'static str {
    match lang {
        Language::English => "Pointer & Performance",
        Language::Portuguese => "Ponteiro & Desempenho",
    }
}

pub fn pointer_page_subtitle(lang: Language) -> &'static str {
    match lang {
        Language::English => "Adjust sensitivity (DPI) and polling rate for your mouse.",
        Language::Portuguese => {
            "Ajuste a sensibilidade (DPI) e taxa de resposta (Polling Rate) do mouse."
        }
    }
}

pub fn dpi_sensitivity(lang: Language) -> &'static str {
    match lang {
        Language::English => "DPI Sensitivity",
        Language::Portuguese => "Sensibilidade DPI",
    }
}

pub fn polling_rate(lang: Language) -> &'static str {
    match lang {
        Language::English => "Polling Rate",
        Language::Portuguese => "Taxa de Resposta (Polling Rate)",
    }
}

pub fn active_profile_passthrough(lang: Language) -> &'static str {
    match lang {
        Language::English => "Active profile: Default (Passthrough)",
        Language::Portuguese => "Perfil ativo: Padrão (Passthrough)",
    }
}

pub fn apply_to_mouse(lang: Language) -> &'static str {
    match lang {
        Language::English => "⚡ Apply to Mouse",
        Language::Portuguese => "⚡ Aplicar ao Mouse",
    }
}

pub fn restore_original(lang: Language) -> &'static str {
    match lang {
        Language::English => "🔄 Restore Original Settings",
        Language::Portuguese => "🔄 Restaurar Configuração Original",
    }
}

pub fn read_from_mouse(lang: Language) -> &'static str {
    match lang {
        Language::English => "📡 Read from Mouse",
        Language::Portuguese => "📡 Ler do Mouse",
    }
}

// ============================================================================
// DOMAIN CATALOG: BUTTON MAPPING PAGE
// ============================================================================

pub fn button_mapping_title(lang: Language) -> &'static str {
    match lang {
        Language::English => "Button Mapping",
        Language::Portuguese => "Mapeamento de Botões",
    }
}

pub fn button_mapping_subtitle(lang: Language) -> &'static str {
    match lang {
        Language::English => "Select a button on the image or list to customize its action.",
        Language::Portuguese => {
            "Selecione um botão na imagem ou na lista para personalizar sua ação."
        }
    }
}

pub fn reset_action(lang: Language) -> &'static str {
    match lang {
        Language::English => "Reset to Default",
        Language::Portuguese => "Restaurar Padrão",
    }
}

// ============================================================================
// DOMAIN CATALOG: DEVICE INFO PAGE
// ============================================================================

pub fn device_info_title(lang: Language) -> &'static str {
    match lang {
        Language::English => "Device Details",
        Language::Portuguese => "Detalhes do Dispositivo",
    }
}

pub fn open_diag_modal(lang: Language) -> &'static str {
    match lang {
        Language::English => "🔍 Open Detailed Battery Diagnostics",
        Language::Portuguese => "🔍 Abrir Diagnóstico Detalhado de Bateria",
    }
}

// ============================================================================
// DOMAIN CATALOG: DIAGNOSTICS & BATTERY
// ============================================================================

pub fn battery_diag_title(lang: Language) -> &'static str {
    match lang {
        Language::English => "Battery Diagnostic Report",
        Language::Portuguese => "Relatório de Diagnóstico de Bateria",
    }
}

pub fn export_log(lang: Language) -> &'static str {
    match lang {
        Language::English => "📥 Export Log",
        Language::Portuguese => "📥 Exportar Log",
    }
}

pub fn close_modal(lang: Language) -> &'static str {
    match lang {
        Language::English => "Close",
        Language::Portuguese => "Fechar",
    }
}

// ============================================================================
// DOMAIN CATALOG: DISCONNECTED & ERRORS
// ============================================================================

pub fn disconnected_title(lang: Language) -> &'static str {
    match lang {
        Language::English => "Device Disconnected",
        Language::Portuguese => "Dispositivo Desconectado",
    }
}

pub fn disconnected_desc(lang: Language) -> &'static str {
    match lang {
        Language::English => "The selected mouse or keyboard was not detected on the system bus.",
        Language::Portuguese => {
            "O mouse ou teclado selecionado não foi detectado no barramento do sistema."
        }
    }
}

pub fn reconnect_button(lang: Language) -> &'static str {
    match lang {
        Language::English => "Try Reconnecting",
        Language::Portuguese => "Tentar Reconectar",
    }
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_toggle() {
        assert_eq!(Language::English.toggle(), Language::Portuguese);
        assert_eq!(Language::Portuguese.toggle(), Language::English);
    }

    #[test]
    fn test_language_codes_and_labels() {
        assert_eq!(Language::English.code(), "EN");
        assert_eq!(Language::Portuguese.code(), "PT");
        assert_eq!(Language::English.label(), "English");
        assert_eq!(Language::Portuguese.label(), "Português");
        assert_eq!(Language::English.locale_code(), "en-US");
        assert_eq!(Language::Portuguese.locale_code(), "pt-BR");
    }

    #[test]
    fn test_from_locale_code_parsing_and_fallbacks() {
        assert_eq!(Language::from_locale_code("pt-BR"), Language::Portuguese);
        assert_eq!(Language::from_locale_code("pt_BR"), Language::Portuguese);
        assert_eq!(Language::from_locale_code("pt"), Language::Portuguese);

        assert_eq!(Language::from_locale_code("en-US"), Language::English);
        assert_eq!(Language::from_locale_code("en_US"), Language::English);
        assert_eq!(Language::from_locale_code("en"), Language::English);

        // Fallback for invalid/unrecognized locale codes defaults to Portuguese
        assert_eq!(Language::from_locale_code("fr-FR"), Language::Portuguese);
        assert_eq!(Language::from_locale_code("es-ES"), Language::Portuguese);
        assert_eq!(Language::from_locale_code(""), Language::Portuguese);
    }

    #[test]
    fn test_translations_coverage() {
        assert_eq!(back_to_devices(Language::English), "‹ Devices");
        assert_eq!(back_to_devices(Language::Portuguese), "‹ Dispositivos");
        assert_eq!(
            no_device_title(Language::English),
            "No Rapoo Devices Detected"
        );
        assert_eq!(
            no_device_title(Language::Portuguese),
            "Nenhum Dispositivo Rapoo Detectado"
        );
        assert_eq!(apply_to_mouse(Language::English), "⚡ Apply to Mouse");
        assert_eq!(apply_to_mouse(Language::Portuguese), "⚡ Aplicar ao Mouse");
        assert_eq!(
            connected_devices_count(Language::English, 2),
            "Connected Devices (2)"
        );
        assert_eq!(
            connected_devices_count(Language::Portuguese, 2),
            "Dispositivos Conectados (2)"
        );
    }
}
