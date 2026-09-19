//! Action editor inspector component for customizing mouse button actions.

use crate::i18n::Language;
use crate::theme::current_theme;
use crate::ui::hotspot::ALL_HOTSPOTS;
use gpui::{
    div, px, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::v_flex;
use openrapoo_core::config::{ButtonAction, Profile};
use std::rc::Rc;

pub fn render_action_editor(
    active_profile: &Profile,
    selected_hex: Option<&str>,
    language: Language,
    on_update_action: Rc<dyn Fn(String, ButtonAction, &mut gpui::App)>,
    on_reset_action: Rc<dyn Fn(String, &mut gpui::App)>,
) -> impl IntoElement {
    let theme = current_theme();
    let is_en = matches!(language, Language::English);

    let hotspot = selected_hex.and_then(|hex| ALL_HOTSPOTS.iter().find(|h| h.hex_code == hex));

    let Some(hotspot) = hotspot else {
        return div().w(px(300.0)).flex_shrink_0().h_full().child(
            v_flex()
                .w_full()
                .h_full()
                .p_5()
                .bg(theme.panel)
                .border_l_1()
                .border_color(theme.border)
                .justify_center()
                .items_center()
                .gap_3()
                .child(div().text_size(px(32.0)).child("🎯"))
                .child(
                    div()
                        .text_size(px(14.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.text_primary)
                        .child(if is_en { "Select a Control" } else { "Selecione um Controle" }),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(theme.text_muted)
                        .text_align(gpui::TextAlign::Center)
                        .child(if is_en { "Click one of the mouse buttons or list items to edit its action." } else { "Clique em um dos botões do mouse ou na lista para editar sua ação." }),
                ),
        );
    };

    let current_action = active_profile
        .get_action(hotspot.hex_code)
        .cloned()
        .unwrap_or(ButtonAction::PassThrough);

    let hex_code = hotspot.hex_code.to_string();
    let hex_code_reset = hex_code.clone();
    let cb_update = on_update_action.clone();
    let cb_reset = on_reset_action.clone();

    let current_action_display = match &current_action {
        ButtonAction::PassThrough => if is_en { "System Default".to_string() } else { "Padrão do Sistema".to_string() },
        ButtonAction::Key { key } => if is_en { format!("Key: {key}") } else { format!("Tecla: {key}") },
        ButtonAction::KeyCombo { keys } => if is_en { format!("Shortcut: {}", keys.join(" + ")) } else { format!("Atalho: {}", keys.join(" + ")) },
        ButtonAction::Copy => if is_en { "Copy (Ctrl + C)".to_string() } else { "Copiar (Ctrl + C)".to_string() },
        ButtonAction::Paste => if is_en { "Paste (Ctrl + V)".to_string() } else { "Colar (Ctrl + V)".to_string() },
        ButtonAction::OpenTerminal => if is_en { "Open Terminal".to_string() } else { "Abrir Terminal".to_string() },
        ButtonAction::CloseWindow => if is_en { "Close Window".to_string() } else { "Fechar Janela".to_string() },
        ButtonAction::Disabled => if is_en { "Disabled".to_string() } else { "Desativado".to_string() },
        ButtonAction::Macro { macro_id } => format!("Macro ({})", &macro_id.to_string()[..8]),
        _ => format!("{current_action:?}"),
    };

    let is_active_action = |act: &ButtonAction| -> bool {
        match (act, &current_action) {
            (ButtonAction::PassThrough, ButtonAction::PassThrough) => true,
            (ButtonAction::Copy, ButtonAction::Copy) => true,
            (ButtonAction::Paste, ButtonAction::Paste) => true,
            (ButtonAction::OpenTerminal, ButtonAction::OpenTerminal) => true,
            (ButtonAction::CloseWindow, ButtonAction::CloseWindow) => true,
            (ButtonAction::Disabled, ButtonAction::Disabled) => true,
            (ButtonAction::Key { key: k1 }, ButtonAction::Key { key: k2 }) => k1 == k2,
            (ButtonAction::KeyCombo { keys: k1 }, ButtonAction::KeyCombo { keys: k2 }) => k1 == k2,
            _ => false,
        }
    };

    div().w(px(320.0)).flex_shrink_0().h_full().child(
        v_flex()
            .w_full()
            .h_full()
            .p_4()
            .bg(theme.panel)
            .border_l_1()
            .border_color(theme.border)
            .gap_4()
            .overflow_y_scrollbar()
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(15.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(if is_en { format!("Edit Action — {}", hotspot.name(language)) } else { format!("Editar Ação — {}", hotspot.name(language)) }),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(theme.text_muted)
                            .child(if is_en { format!("evdev code: {}", hotspot.hex_code) } else { format!("Código evdev: {}", hotspot.hex_code) }),
                    ),
            )
            .child(
                v_flex()
                    .p_3()
                    .rounded_lg()
                    .bg(theme.panel_elevated)
                    .border_1()
                    .border_color(theme.border)
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text_muted)
                            .child(if is_en { "Configured Action" } else { "Ação Configurada" }),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.accent_green)
                            .child(current_action_display),
                    )
                    .child(
                        v_flex()
                            .p_2()
                            .rounded_md()
                            .bg(theme.bg)
                            .border_1()
                            .border_color(theme.border)
                            .gap_0p5()
                            .child(
                                gpui_component::h_flex()
                                    .gap_1p5()
                                    .items_center()
                                    .child(div().text_size(px(10.0)).child("⚡"))
                                    .child(
                                        div()
                                            .text_size(px(10.0))
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(theme.accent_blue)
                                            .child(if is_en { "Real Hardware Status:" } else { "Status de Aplicação Real:" }),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(10.5))
                                    .text_color(theme.text_muted)
                                    .child(if is_en { "Applied by OpenRapoo in this session via evdev/uinput (Linux User-Space Daemon)." } else { "Aplicado pelo OpenRapoo nesta sessão via evdev/uinput (Linux User-Space Daemon)." }),
                            ),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(if is_en { "Shortcuts & Key Combinations" } else { "Atalhos & Combinações de Teclas" }),
                    )
                    .child(action_item(
                        "act-ctrlc",
                        if is_en { "📋 Copy (Ctrl + C)" } else { "📋 Copiar (Ctrl + C)" },
                        is_active_action(&ButtonAction::Copy),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| cb(hex.clone(), ButtonAction::Copy, cx)
                        },
                    ))
                    .child(action_item(
                        "act-ctrlv",
                        if is_en { "📥 Paste (Ctrl + V)" } else { "📥 Colar (Ctrl + V)" },
                        is_active_action(&ButtonAction::Paste),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| cb(hex.clone(), ButtonAction::Paste, cx)
                        },
                    ))
                    .child(action_item(
                        "act-reopentab",
                        if is_en { "🔁 Reopen Closed Tab (Ctrl + Shift + T)" } else { "🔁 Reabrir Aba Fechada (Ctrl + Shift + T)" },
                        is_active_action(&ButtonAction::KeyCombo {
                            keys: vec![
                                "KEY_LEFTCTRL".into(),
                                "KEY_LEFTSHIFT".into(),
                                "KEY_T".into(),
                            ],
                        }),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| {
                                cb(
                                    hex.clone(),
                                    ButtonAction::KeyCombo {
                                        keys: vec![
                                            "KEY_LEFTCTRL".into(),
                                            "KEY_LEFTSHIFT".into(),
                                            "KEY_T".into(),
                                        ],
                                    },
                                    cx,
                                )
                            }
                        },
                    ))
                    .child(action_item(
                        "act-closetab",
                        if is_en { "❌ Close Active Tab (Ctrl + W)" } else { "❌ Fechar Aba Ativa (Ctrl + W)" },
                        is_active_action(&ButtonAction::KeyCombo {
                            keys: vec!["KEY_LEFTCTRL".into(), "KEY_W".into()],
                        }),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| {
                                cb(
                                    hex.clone(),
                                    ButtonAction::KeyCombo {
                                        keys: vec!["KEY_LEFTCTRL".into(), "KEY_W".into()],
                                    },
                                    cx,
                                )
                            }
                        },
                    ))
                    .child(action_item(
                        "act-nexttab",
                        if is_en { "➡️ Next Tab (Ctrl + Tab)" } else { "➡️ Próxima Aba (Ctrl + Tab)" },
                        is_active_action(&ButtonAction::KeyCombo {
                            keys: vec!["KEY_LEFTCTRL".into(), "KEY_TAB".into()],
                        }),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| {
                                cb(
                                    hex.clone(),
                                    ButtonAction::KeyCombo {
                                        keys: vec!["KEY_LEFTCTRL".into(), "KEY_TAB".into()],
                                    },
                                    cx,
                                )
                            }
                        },
                    ))
                    .child(action_item(
                        "act-undo",
                        if is_en { "↩️ Undo (Ctrl + Z)" } else { "↩️ Desfazer (Ctrl + Z)" },
                        is_active_action(&ButtonAction::KeyCombo {
                            keys: vec!["KEY_LEFTCTRL".into(), "KEY_Z".into()],
                        }),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| {
                                cb(
                                    hex.clone(),
                                    ButtonAction::KeyCombo {
                                        keys: vec!["KEY_LEFTCTRL".into(), "KEY_Z".into()],
                                    },
                                    cx,
                                )
                            }
                        },
                    ))
                    .child(action_item(
                        "act-redo",
                        if is_en { "↪️ Redo (Ctrl + Y)" } else { "↪️ Refazer (Ctrl + Y)" },
                        is_active_action(&ButtonAction::KeyCombo {
                            keys: vec!["KEY_LEFTCTRL".into(), "KEY_Y".into()],
                        }),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| {
                                cb(
                                    hex.clone(),
                                    ButtonAction::KeyCombo {
                                        keys: vec!["KEY_LEFTCTRL".into(), "KEY_Y".into()],
                                    },
                                    cx,
                                )
                            }
                        },
                    ))
                    .child(action_item(
                        "act-desktop",
                        if is_en { "🖥️ Show Desktop (Super + D)" } else { "🖥️ Mostrar Área de Trabalho (Super + D)" },
                        is_active_action(&ButtonAction::KeyCombo {
                            keys: vec!["KEY_LEFTMETA".into(), "KEY_D".into()],
                        }),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| {
                                cb(
                                    hex.clone(),
                                    ButtonAction::KeyCombo {
                                        keys: vec!["KEY_LEFTMETA".into(), "KEY_D".into()],
                                    },
                                    cx,
                                )
                            }
                        },
                    )),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(if is_en { "Macros & Automation" } else { "Macros & Automações" }),
                    )
                    .child(action_item(
                        "act-macro-selectall-copy",
                        if is_en { "⚡ Macro: Select All + Copy" } else { "⚡ Macro: Selecionar Tudo + Copiar" },
                        is_active_action(&ButtonAction::KeyCombo {
                            keys: vec![
                                "KEY_LEFTCTRL".into(),
                                "KEY_A".into(),
                                "KEY_C".into(),
                            ],
                        }),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| {
                                cb(
                                    hex.clone(),
                                    ButtonAction::KeyCombo {
                                        keys: vec![
                                            "KEY_LEFTCTRL".into(),
                                            "KEY_A".into(),
                                            "KEY_C".into(),
                                        ],
                                    },
                                    cx,
                                )
                            }
                        },
                    ))
                    .child(action_item(
                        "act-macro-cut-paste",
                        if is_en { "⚡ Macro: Cut + Paste" } else { "⚡ Macro: Recortar + Colar" },
                        is_active_action(&ButtonAction::KeyCombo {
                            keys: vec![
                                "KEY_LEFTCTRL".into(),
                                "KEY_X".into(),
                                "KEY_V".into(),
                            ],
                        }),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| {
                                cb(
                                    hex.clone(),
                                    ButtonAction::KeyCombo {
                                        keys: vec![
                                            "KEY_LEFTCTRL".into(),
                                            "KEY_X".into(),
                                            "KEY_V".into(),
                                        ],
                                    },
                                    cx,
                                )
                            }
                        },
                    ))
                    .child(action_item(
                        "act-macro-newtab-paste",
                        if is_en { "⚡ Macro: New Tab + Paste" } else { "⚡ Macro: Nova Aba + Colar" },
                        is_active_action(&ButtonAction::KeyCombo {
                            keys: vec![
                                "KEY_LEFTCTRL".into(),
                                "KEY_T".into(),
                                "KEY_V".into(),
                            ],
                        }),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| {
                                cb(
                                    hex.clone(),
                                    ButtonAction::KeyCombo {
                                        keys: vec![
                                            "KEY_LEFTCTRL".into(),
                                            "KEY_T".into(),
                                            "KEY_V".into(),
                                        ],
                                    },
                                    cx,
                                )
                            }
                        },
                    )),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(if is_en { "System & Media Actions" } else { "Ações do Sistema & Mídia" }),
                    )
                    .child(action_item(
                        "act-passthrough",
                        if is_en { "↩️ Restore System Default" } else { "↩️ Restaurar Padrão do Sistema" },
                        is_active_action(&ButtonAction::PassThrough),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| cb(hex.clone(), ButtonAction::PassThrough, cx)
                        },
                    ))
                    .child(action_item(
                        "act-playpause",
                        if is_en { "⏯️ Media: Play / Pause" } else { "⏯️ Mídia: Reproduzir / Pausar" },
                        is_active_action(&ButtonAction::Key {
                            key: "KEY_PLAYPAUSE".to_string(),
                        }),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| {
                                cb(
                                    hex.clone(),
                                    ButtonAction::Key {
                                        key: "KEY_PLAYPAUSE".to_string(),
                                    },
                                    cx,
                                )
                            }
                        },
                    ))
                    .child(action_item(
                        "act-volup",
                        if is_en { "🔊 Volume Up" } else { "🔊 Aumentar Volume" },
                        is_active_action(&ButtonAction::Key {
                            key: "KEY_VOLUMEUP".to_string(),
                        }),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| {
                                cb(
                                    hex.clone(),
                                    ButtonAction::Key {
                                        key: "KEY_VOLUMEUP".to_string(),
                                    },
                                    cx,
                                )
                            }
                        },
                    ))
                    .child(action_item(
                        "act-voldown",
                        if is_en { "🔉 Volume Down" } else { "🔉 Diminuir Volume" },
                        is_active_action(&ButtonAction::Key {
                            key: "KEY_VOLUMEDOWN".to_string(),
                        }),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| {
                                cb(
                                    hex.clone(),
                                    ButtonAction::Key {
                                        key: "KEY_VOLUMEDOWN".to_string(),
                                    },
                                    cx,
                                )
                            }
                        },
                    ))
                    .child(action_item(
                        "act-terminal",
                        if is_en { "🖥️ Open Terminal" } else { "🖥️ Abrir Terminal" },
                        is_active_action(&ButtonAction::OpenTerminal),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| cb(hex.clone(), ButtonAction::OpenTerminal, cx)
                        },
                    ))
                    .child(action_item(
                        "act-closewin",
                        if is_en { "❌ Close Window (Alt + F4)" } else { "❌ Fechar Janela (Alt + F4)" },
                        is_active_action(&ButtonAction::CloseWindow),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| cb(hex.clone(), ButtonAction::CloseWindow, cx)
                        },
                    ))
                    .child(action_item(
                        "act-disabled",
                        if is_en { "🚫 Disable Button" } else { "🚫 Desativar Botão" },
                        is_active_action(&ButtonAction::Disabled),
                        language,
                        {
                            let hex = hex_code.clone();
                            let cb = cb_update.clone();
                            move |_, _, cx| cb(hex.clone(), ButtonAction::Disabled, cx)
                        },
                    )),
            )
            .child(
                div()
                    .id("act-reset-all")
                    .w_full()
                    .py_2()
                    .mt_auto()
                    .rounded_md()
                    .bg(theme.border)
                    .text_color(theme.text_primary)
                    .text_size(px(12.0))
                    .text_align(gpui::TextAlign::Center)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.panel_elevated))
                    .on_click(move |_, _, cx| cb_reset(hex_code_reset.clone(), cx))
                    .child(if is_en { "Restore Default Configuration" } else { "Restaurar Configuração Padrão" }),
            ),
    )
}

fn action_item(
    id: &'static str,
    label: &str,
    is_active: bool,
    language: Language,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    let theme = current_theme();

    let bg = if is_active {
        theme.panel_elevated
    } else {
        theme.panel_elevated
    };

    let border_color = if is_active {
        theme.accent_green
    } else {
        theme.border
    };

    gpui_component::h_flex()
        .id(id)
        .w_full()
        .p_2p5()
        .rounded_lg()
        .bg(bg)
        .border_1()
        .border_color(border_color)
        .text_color(theme.text_primary)
        .text_size(px(12.0))
        .justify_between()
        .items_center()
        .cursor_pointer()
        .hover(|s| s.bg(theme.border))
        .on_click(on_click)
        .child(div().child(label.to_string()))
        .child(if is_active {
            div()
                .px_1p5()
                .py_0p5()
                .rounded_full()
                .bg(theme.accent_green)
                .text_color(theme.panel)
                .text_size(px(10.0))
                .font_weight(gpui::FontWeight::BOLD)
                .child(if matches!(language, Language::English) { "✓ Active" } else { "✓ Ativo" })
        } else {
            div()
        })
}
