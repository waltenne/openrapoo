#![allow(dead_code)]
//! Button action editor component for OpenRapoo GUI.

/// Action categories for the button editor UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionCategory {
    Passthrough,
    MouseButton,
    KeyCombo,
    ShortcutAction,
    MediaControl,
    WorkspaceSwitch,
    RunCommand,
    TypeText,
    Disabled,
}

impl ActionCategory {
    pub fn name_pt(&self) -> &'static str {
        match self {
            ActionCategory::Passthrough => "Padrão do Sistema (Passthrough)",
            ActionCategory::MouseButton => "Botão do Mouse",
            ActionCategory::KeyCombo => "Tecla ou Combinação",
            ActionCategory::ShortcutAction => "Ação Rápida (Copiar, Colar, Terminal)",
            ActionCategory::MediaControl => "Controle de Mídia",
            ActionCategory::WorkspaceSwitch => "Mudar Área de Trabalho",
            ActionCategory::RunCommand => "Executar Comando Customizado",
            ActionCategory::TypeText => "Digitar Texto Predefinido",
            ActionCategory::Disabled => "Desativar Botão",
        }
    }

    pub fn name_en(&self) -> &'static str {
        match self {
            ActionCategory::Passthrough => "OS Default (Passthrough)",
            ActionCategory::MouseButton => "Mouse Button",
            ActionCategory::KeyCombo => "Key or Shortcut Combo",
            ActionCategory::ShortcutAction => "Quick Shortcut (Copy, Paste, Terminal)",
            ActionCategory::MediaControl => "Media Controls",
            ActionCategory::WorkspaceSwitch => "Switch Virtual Workspace",
            ActionCategory::RunCommand => "Run Custom Command",
            ActionCategory::TypeText => "Type Preset Text",
            ActionCategory::Disabled => "Disable Button",
        }
    }
}
