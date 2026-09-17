//! Rapoo MT760 Pro interactive mouse view widget.

use openrapoo_core::config::{ButtonAction, Profile};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RapooButtonInfo {
    pub id: usize,
    pub name_pt: &'static str,
    pub name_en: &'static str,
    pub hex_code: &'static str,
    pub description: &'static str,
}

pub const RAPOO_MT760_BUTTONS: &[RapooButtonInfo] = &[
    RapooButtonInfo {
        id: 1,
        name_pt: "Clique Esquerdo",
        name_en: "Left Click",
        hex_code: "0x110",
        description: "Botão principal do mouse (BTN_LEFT)",
    },
    RapooButtonInfo {
        id: 2,
        name_pt: "Clique Direito",
        name_en: "Right Click",
        hex_code: "0x111",
        description: "Botão secundário de menu de contexto (BTN_RIGHT)",
    },
    RapooButtonInfo {
        id: 3,
        name_pt: "Clique do Meio",
        name_en: "Middle Click / Scroll Wheel",
        hex_code: "0x112",
        description: "Clique da roda de rolagem vertical (BTN_MIDDLE)",
    },
    RapooButtonInfo {
        id: 4,
        name_pt: "Botão Lateral Traseiro",
        name_en: "Side Back Button",
        hex_code: "0x113",
        description: "Navegação Voltar (BTN_SIDE)",
    },
    RapooButtonInfo {
        id: 5,
        name_pt: "Botão Lateral Dianteiro",
        name_en: "Side Forward Button",
        hex_code: "0x114",
        description: "Navegação Avançar (BTN_EXTRA)",
    },
    RapooButtonInfo {
        id: 6,
        name_pt: "Roda de Scroll Lateral",
        name_en: "Horizontal Scroll Wheel",
        hex_code: "0x006",
        description: "Roda de rolagem horizontal (REL_HWHEEL)",
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
