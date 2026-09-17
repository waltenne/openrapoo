//! Action editor dialog with automatic key capture.

use openrapoo_core::config::{ButtonAction, MouseButton};

#[derive(Debug, Clone)]
pub struct ActionEditorState {
    pub button_code: String,
    pub button_name: String,
    pub current_action: ButtonAction,
    pub captured_key: Option<String>,
}

impl ActionEditorState {
    pub fn new(button_code: impl Into<String>, button_name: impl Into<String>, current_action: ButtonAction) -> Self {
        ActionEditorState {
            button_code: button_code.into(),
            button_name: button_name.into(),
            current_action,
            captured_key: None,
        }
    }

    pub fn set_key(&mut self, key_str: impl Into<String>) {
        let key = key_str.into();
        self.captured_key = Some(key.clone());
        self.current_action = ButtonAction::Key { key };
    }

    pub fn set_key_combo(&mut self, keys: Vec<String>) {
        self.current_action = ButtonAction::KeyCombo { keys };
    }

    pub fn set_run_command(&mut self, argv: Vec<String>) {
        self.current_action = ButtonAction::RunCommand { argv };
    }
}
