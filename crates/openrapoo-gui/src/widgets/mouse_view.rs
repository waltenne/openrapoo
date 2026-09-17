//! Rapoo MT760 Pro interactive mouse view widget and leader lines painter.

use openrapoo_core::config::{ButtonAction, Profile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardSide {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct RapooButtonInfo {
    pub id: usize,
    pub name_pt: &'static str,
    pub name_en: &'static str,
    pub hex_code: &'static str,
    pub description: &'static str,
    pub icon_name: &'static str,
    pub default_action_pt: &'static str,
    pub card_side: CardSide,
    pub hotspot_x_pct: f64,
    pub hotspot_y_pct: f64,
}

pub const RAPOO_MT760_BUTTONS: &[RapooButtonInfo] = &[
    RapooButtonInfo {
        id: 1,
        name_pt: "Clique do Meio",
        name_en: "Middle Click",
        hex_code: "0x112",
        description: "Clique da roda de rolagem (BTN_MIDDLE)",
        icon_name: "mouse-middle-click",
        default_action_pt: "Clique do Meio",
        card_side: CardSide::Left,
        hotspot_x_pct: 0.50,
        hotspot_y_pct: 0.20,
    },
    RapooButtonInfo {
        id: 2,
        name_pt: "Clique Esquerdo",
        name_en: "Left Click",
        hex_code: "0x110",
        description: "Botão principal do mouse (BTN_LEFT)",
        icon_name: "mouse-left-click",
        default_action_pt: "Clique Esquerdo",
        card_side: CardSide::Left,
        hotspot_x_pct: 0.44,
        hotspot_y_pct: 0.18,
    },
    RapooButtonInfo {
        id: 3,
        name_pt: "Roda de Scroll Lateral",
        name_en: "Thumb Wheel",
        hex_code: "0x006",
        description: "Roda de rolagem horizontal (REL_HWHEEL)",
        icon_name: "horizontal-scroll",
        default_action_pt: "Rolagem Horizontal",
        card_side: CardSide::Left,
        hotspot_x_pct: 0.31,
        hotspot_y_pct: 0.48,
    },
    RapooButtonInfo {
        id: 4,
        name_pt: "Botão Lateral Dianteiro",
        name_en: "Forward Button",
        hex_code: "0x114",
        description: "Navegação Avançar (BTN_EXTRA)",
        icon_name: "go-next",
        default_action_pt: "Avançar Página",
        card_side: CardSide::Left,
        hotspot_x_pct: 0.31,
        hotspot_y_pct: 0.58,
    },
    RapooButtonInfo {
        id: 5,
        name_pt: "Botão Lateral Traseiro",
        name_en: "Back Button",
        hex_code: "0x113",
        description: "Navegação Voltar (BTN_SIDE)",
        icon_name: "go-previous",
        default_action_pt: "Voltar Página",
        card_side: CardSide::Left,
        hotspot_x_pct: 0.30,
        hotspot_y_pct: 0.65,
    },
    RapooButtonInfo {
        id: 6,
        name_pt: "Clique Direito",
        name_en: "Right Click",
        hex_code: "0x111",
        description: "Botão secundário de menu (BTN_RIGHT)",
        icon_name: "mouse-right-click",
        default_action_pt: "Clique Direito",
        card_side: CardSide::Right,
        hotspot_x_pct: 0.56,
        hotspot_y_pct: 0.18,
    },
    RapooButtonInfo {
        id: 7,
        name_pt: "Botão de DPI",
        name_en: "DPI Toggle",
        hex_code: "0x115",
        description: "Alternar resolução DPI",
        icon_name: "display-brightness",
        default_action_pt: "Alternar DPI",
        card_side: CardSide::Right,
        hotspot_x_pct: 0.50,
        hotspot_y_pct: 0.30,
    },
];

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

pub fn get_button_action_icon(profile: &Profile, hex_code: &str, default_icon: &'static str) -> &'static str {
    match profile.get_action(hex_code) {
        Some(ButtonAction::Copy) | Some(ButtonAction::Paste) => "edit-copy",
        Some(ButtonAction::OpenTerminal) => "utilities-terminal",
        Some(ButtonAction::CloseWindow) => "window-close",
        Some(ButtonAction::MediaPlayPause) | Some(ButtonAction::MediaNext) | Some(ButtonAction::MediaPrev) => "media-playback-start",
        Some(ButtonAction::RunCommand { .. }) => "system-run",
        _ => default_icon,
    }
}
