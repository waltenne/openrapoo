//! Action editor dialog with automatic key capture.

use openrapoo_core::config::{ButtonAction, Profile};

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

#[cfg(feature = "gtk")]
pub fn build_action_dialog_popover<F>(
    button_code: &str,
    button_name: &str,
    profile: &Profile,
    on_save: F,
) -> gtk4::Popover
where
    F: Fn(ButtonAction) + 'static,
{
    use gtk4::prelude::*;
    use gtk4::{Box, Button, Label, Orientation, Popover};

    let popover = Popover::new();
    popover.set_autohide(true);

    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_start(16);
    main_box.set_margin_end(16);
    main_box.set_margin_top(16);
    main_box.set_margin_bottom(16);

    let title_lbl = Label::new(Some(&format!("Editar Ação — {}", button_name)));
    title_lbl.add_css_class("card-label-title");

    let current_act = profile.get_action(button_code).cloned().unwrap_or(ButtonAction::PassThrough);
    let desc_lbl = Label::new(Some(&format!("Ação Atual: {:?}", current_act)));

    let btn_reset = Button::with_label("Restaurar Padrão do Sistema");
    let pop_ref = popover.clone();
    let cb_reset = on_save;
    btn_reset.connect_clicked(move |_| {
        cb_reset(ButtonAction::PassThrough);
        pop_ref.popdown();
    });

    main_box.append(&title_lbl);
    main_box.append(&desc_lbl);
    main_box.append(&btn_reset);

    popover.set_child(Some(&main_box));
    popover
}
