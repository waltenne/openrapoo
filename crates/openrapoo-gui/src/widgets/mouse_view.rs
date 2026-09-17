//! Rapoo MT760 Pro interactive mouse view widget, hotspot data model, and SVG asset loader.

use openrapoo_core::config::{ButtonAction, Profile};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackendButton {
    Left,
    Right,
    Middle,
    Forward,
    Back,
    Dpi,
    DeviceSwitch,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
}

impl BackendButton {
    pub fn hotspot_id(&self) -> &'static str {
        match self {
            BackendButton::Left => "hotspot-left",
            BackendButton::Right => "hotspot-right",
            BackendButton::Middle => "hotspot-middle",
            BackendButton::Forward => "hotspot-forward",
            BackendButton::Back => "hotspot-back",
            BackendButton::Dpi => "hotspot-dpi",
            BackendButton::DeviceSwitch => "hotspot-device-switch",
            BackendButton::WheelUp => "hotspot-roller-forward",
            BackendButton::WheelDown => "hotspot-roller-back",
            BackendButton::WheelLeft => "hotspot-roller-left",
            BackendButton::WheelRight => "hotspot-roller-right",
        }
    }

    pub fn hex_code(&self) -> &'static str {
        match self {
            BackendButton::Left => "0x110",
            BackendButton::Right => "0x111",
            BackendButton::Middle => "0x112",
            BackendButton::Forward => "0x114",
            BackendButton::Back => "0x113",
            BackendButton::Dpi => "0x115",
            BackendButton::DeviceSwitch => "0x116",
            BackendButton::WheelUp => "REL_WHEEL_UP",
            BackendButton::WheelDown => "REL_WHEEL_DOWN",
            BackendButton::WheelLeft => "REL_HWHEEL_LEFT",
            BackendButton::WheelRight => "REL_HWHEEL_RIGHT",
        }
    }

    pub fn display_name_pt(&self) -> &'static str {
        match self {
            BackendButton::Left => "Clique Esquerdo",
            BackendButton::Right => "Clique Direito",
            BackendButton::Middle => "Clique do Meio",
            BackendButton::Forward => "Botão Avançar",
            BackendButton::Back => "Botão Voltar",
            BackendButton::Dpi => "Botão de DPI",
            BackendButton::DeviceSwitch => "Troca de Dispositivo",
            BackendButton::WheelUp => "Rolar para Cima",
            BackendButton::WheelDown => "Rolar para Baixo",
            BackendButton::WheelLeft => "Roda Lateral Esquerda",
            BackendButton::WheelRight => "Roda Lateral Direita",
        }
    }
}

#[derive(Debug, Clone)]
pub struct HotspotControl {
    pub backend_button: BackendButton,
    pub hotspot_id: &'static str,
    pub name_pt: &'static str,
    pub name_en: &'static str,
    pub hex_code: &'static str,
    pub svg_cx: f64,
    pub svg_cy: f64,
    pub card_side: CardSide,
    pub is_supported_on_linux: bool,
    pub tooltip_pt: &'static str,
}

pub const ALL_HOTSPOTS: &[HotspotControl] = &[
    // Left side controls
    HotspotControl {
        backend_button: BackendButton::Left,
        hotspot_id: "hotspot-left",
        name_pt: "Clique Esquerdo",
        name_en: "Left Click",
        hex_code: "0x110",
        svg_cx: 55.0,
        svg_cy: 112.0,
        card_side: CardSide::Left,
        is_supported_on_linux: true,
        tooltip_pt: "Clique esquerdo principal (BTN_LEFT)",
    },
    HotspotControl {
        backend_button: BackendButton::WheelLeft,
        hotspot_id: "hotspot-roller-left",
        name_pt: "Roda Lateral Esquerda",
        name_en: "Roller Left",
        hex_code: "REL_HWHEEL_LEFT",
        svg_cx: 55.0,
        svg_cy: 220.0,
        card_side: CardSide::Left,
        is_supported_on_linux: true,
        tooltip_pt: "Roda de rolagem lateral (para esquerda)",
    },
    HotspotControl {
        backend_button: BackendButton::WheelRight,
        hotspot_id: "hotspot-roller-right",
        name_pt: "Roda Lateral Direita",
        name_en: "Roller Right",
        hex_code: "REL_HWHEEL_RIGHT",
        svg_cx: 55.0,
        svg_cy: 328.0,
        card_side: CardSide::Left,
        is_supported_on_linux: true,
        tooltip_pt: "Roda de rolagem lateral (para direita)",
    },
    HotspotControl {
        backend_button: BackendButton::Forward,
        hotspot_id: "hotspot-forward",
        name_pt: "Botão Avançar",
        name_en: "Forward Button",
        hex_code: "0x114",
        svg_cx: 55.0,
        svg_cy: 465.0,
        card_side: CardSide::Left,
        is_supported_on_linux: true,
        tooltip_pt: "Botão lateral dianteiro de navegação (BTN_EXTRA)",
    },
    HotspotControl {
        backend_button: BackendButton::Back,
        hotspot_id: "hotspot-back",
        name_pt: "Botão Voltar",
        name_en: "Back Button",
        hex_code: "0x113",
        svg_cx: 55.0,
        svg_cy: 573.0,
        card_side: CardSide::Left,
        is_supported_on_linux: true,
        tooltip_pt: "Botão lateral traseiro de navegação (BTN_SIDE)",
    },
    // Right side controls
    HotspotControl {
        backend_button: BackendButton::WheelUp,
        hotspot_id: "hotspot-roller-forward",
        name_pt: "Rolar para Cima",
        name_en: "Roller Forward",
        hex_code: "REL_WHEEL_UP",
        svg_cx: 1145.0,
        svg_cy: 76.0,
        card_side: CardSide::Right,
        is_supported_on_linux: true,
        tooltip_pt: "Rolagem vertical para cima (REL_WHEEL)",
    },
    HotspotControl {
        backend_button: BackendButton::Middle,
        hotspot_id: "hotspot-middle",
        name_pt: "Clique do Meio",
        name_en: "Middle Click",
        hex_code: "0x112",
        svg_cx: 1145.0,
        svg_cy: 184.0,
        card_side: CardSide::Right,
        is_supported_on_linux: true,
        tooltip_pt: "Clique da roda de rolagem (BTN_MIDDLE)",
    },
    HotspotControl {
        backend_button: BackendButton::WheelDown,
        hotspot_id: "hotspot-roller-back",
        name_pt: "Rolar para Baixo",
        name_en: "Roller Back",
        hex_code: "REL_WHEEL_DOWN",
        svg_cx: 1145.0,
        svg_cy: 292.0,
        card_side: CardSide::Right,
        is_supported_on_linux: true,
        tooltip_pt: "Rolagem vertical para baixo (REL_WHEEL)",
    },
    HotspotControl {
        backend_button: BackendButton::Right,
        hotspot_id: "hotspot-right",
        name_pt: "Clique Direito",
        name_en: "Right Click",
        hex_code: "0x111",
        svg_cx: 1145.0,
        svg_cy: 420.0,
        card_side: CardSide::Right,
        is_supported_on_linux: true,
        tooltip_pt: "Clique secundário de menu de contexto (BTN_RIGHT)",
    },
    HotspotControl {
        backend_button: BackendButton::Dpi,
        hotspot_id: "hotspot-dpi",
        name_pt: "Botão de DPI",
        name_en: "DPI Toggle",
        hex_code: "0x115",
        svg_cx: 1145.0,
        svg_cy: 548.0,
        card_side: CardSide::Right,
        is_supported_on_linux: false,
        tooltip_pt: "⚠️ Ajuste de DPI via software não suportado sem protocolo HID proprietário",
    },
    HotspotControl {
        backend_button: BackendButton::DeviceSwitch,
        hotspot_id: "hotspot-device-switch",
        name_pt: "Troca de Dispositivo",
        name_en: "Device Switch",
        hex_code: "0x116",
        svg_cx: 1145.0,
        svg_cy: 676.0,
        card_side: CardSide::Right,
        is_supported_on_linux: false,
        tooltip_pt: "⚠️ Troca de dispositivo é processada internamente pelo hardware no firmware",
    },
];

/// Resolve the path for openrapoo-mt760-pro.svg dynamically in dev, test, or packaging environment
pub fn get_svg_asset_path() -> PathBuf {
    let filename = "openrapoo-mt760-pro.svg";

    // 1. Check CARGO_MANIFEST_DIR
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p1 = PathBuf::from(&manifest_dir).join("assets").join(filename);
        if p1.exists() {
            return p1;
        }
        let p2 = PathBuf::from(&manifest_dir).join("..").join("..").join(filename);
        if p2.exists() {
            return p2;
        }
    }

    // 2. Check current working directory
    let p_cwd = PathBuf::from(filename);
    if p_cwd.exists() {
        return p_cwd;
    }

    let p_assets_cwd = PathBuf::from("crates/openrapoo-gui/assets").join(filename);
    if p_assets_cwd.exists() {
        return p_assets_cwd;
    }

    // 3. Check executable relative path
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let p_exe = exe_dir.join("assets").join(filename);
            if p_exe.exists() {
                return p_exe;
            }
            let p_share = exe_dir.join("../share/openrapoo/assets").join(filename);
            if p_share.exists() {
                return p_share;
            }
        }
    }

    PathBuf::from(filename)
}

pub fn get_button_action_label(profile: &Profile, hex_code: &str) -> String {
    match profile.get_action(hex_code) {
        Some(ButtonAction::PassThrough) | None => "Padrão do Sistema".to_string(),
        Some(ButtonAction::Disabled) => "Desativado".to_string(),
        Some(ButtonAction::MouseButton { button }) => format!("Botão {:?}", button),
        Some(ButtonAction::Key { key }) => format!("Tecla: {}", key),
        Some(ButtonAction::KeyCombo { keys }) => format!("Combo: {}", keys.join("+")),
        Some(ButtonAction::Copy) => "Copiar (Ctrl+C)".to_string(),
        Some(ButtonAction::Paste) => "Colar (Ctrl+V)".to_string(),
        Some(ButtonAction::OpenTerminal) => "Abrir Terminal".to_string(),
        Some(ButtonAction::CloseWindow) => "Fechar Janela (Alt+F4)".to_string(),
        Some(ButtonAction::MediaPlayPause) => "Mídia: Play/Pause".to_string(),
        Some(ButtonAction::MediaNext) => "Mídia: Próximo".to_string(),
        Some(ButtonAction::MediaPrev) => "Mídia: Anterior".to_string(),
        Some(ButtonAction::MediaMute) => "Mídia: Mute".to_string(),
        Some(ButtonAction::SwitchWorkspace { number }) => format!("Área de Trabalho {}", number),
        Some(ButtonAction::RunCommand { argv }) => format!("Executar: {}", argv.join(" ")),
        Some(ButtonAction::TypeText { text }) => format!("Digitar: \"{}\"", text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_hotspots_count_and_mapping() {
        assert_eq!(ALL_HOTSPOTS.len(), 11);
        let left = ALL_HOTSPOTS.iter().find(|h| h.backend_button == BackendButton::Left).unwrap();
        assert_eq!(left.hotspot_id, "hotspot-left");
        assert_eq!(left.hex_code, "0x110");
        assert!(left.is_supported_on_linux);

        let dpi = ALL_HOTSPOTS.iter().find(|h| h.backend_button == BackendButton::Dpi).unwrap();
        assert_eq!(dpi.hotspot_id, "hotspot-dpi");
        assert!(!dpi.is_supported_on_linux);
    }

    #[test]
    fn test_svg_asset_path_resolution() {
        let path = get_svg_asset_path();
        assert!(path.to_string_lossy().contains("openrapoo-mt760-pro.svg"));
    }
}
