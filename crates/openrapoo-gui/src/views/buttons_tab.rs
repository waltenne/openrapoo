//! "Botões" (Buttons) Tab View Component.

use crate::widgets::mouse_view::{get_button_action_label, RAPOO_MT760_BUTTONS};
use openrapoo_core::config::{Profile, ProfileStore};

#[derive(Debug, Clone)]
pub struct ButtonsTabModel {
    pub selected_button_code: Option<String>,
}

impl Default for ButtonsTabModel {
    fn default() -> Self {
        ButtonsTabModel {
            selected_button_code: None,
        }
    }
}
