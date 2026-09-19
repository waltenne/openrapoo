use crate::i18n::Language;
use crate::theme::current_theme;
use crate::ui::hotspot::{ControlCategory, ALL_HOTSPOTS};
use gpui::{
    div, px, ElementId, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex};
use openrapoo_core::config::{ButtonAction, Profile};
use std::rc::Rc;

pub fn render_button_list(
    active_profile: &Profile,
    selected_hex: Option<&str>,
    language: Language,
    on_select_button: Rc<dyn Fn(String, &mut gpui::App)>,
) -> impl IntoElement {
    let theme = current_theme();
    let is_en = matches!(language, Language::English);

    let categories = [
        ControlCategory::MainButtons,
        ControlCategory::SideButtons,
        ControlCategory::Wheels,
        ControlCategory::HardwareButtons,
        ControlCategory::Undetected,
    ];

    let mut main_container = v_flex().gap_4().w_full();

    for (cat_idx, category) in categories.iter().enumerate() {
        let items_in_cat: Vec<_> = ALL_HOTSPOTS
            .iter()
            .enumerate()
            .filter(|(_, h)| h.category == *category)
            .collect();

        if items_in_cat.is_empty() {
            continue;
        }

        let section_header = h_flex()
            .gap_2()
            .items_center()
            .justify_between()
            .pb_1p5()
            .border_b_1()
            .border_color(theme.border)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(div().text_size(px(13.0)).child(category.icon()))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(category.label(language)),
                    ),
            )
            .child(
                div()
                    .px_1p5()
                    .py_0p5()
                    .rounded_full()
                    .bg(theme.panel_elevated)
                    .text_size(px(10.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(theme.text_muted)
                    .child(format!("{}", items_in_cat.len())),
            );

        let mut group_list = v_flex().gap_2().w_full();

        for (idx, hotspot) in items_in_cat {
            let is_selected = selected_hex == Some(hotspot.hex_code);
            let is_supported = hotspot.is_supported_on_linux;

            let action = active_profile
                .get_action(hotspot.hex_code)
                .cloned()
                .unwrap_or(ButtonAction::PassThrough);

            let is_custom_mapped = !matches!(action, ButtonAction::PassThrough);

            let action_label = match &action {
                ButtonAction::PassThrough => {
                    if is_en {
                        "System Default".to_string()
                    } else {
                        "Padrão do Sistema".to_string()
                    }
                }
                ButtonAction::Key { key } => {
                    if is_en {
                        format!("Key: {key}")
                    } else {
                        format!("Tecla: {key}")
                    }
                }
                ButtonAction::KeyCombo { keys } => {
                    if is_en {
                        format!("Shortcut: {}", keys.join("+"))
                    } else {
                        format!("Atalho: {}", keys.join("+"))
                    }
                }
                ButtonAction::RunCommand { argv } => {
                    if is_en {
                        format!("Command: {}", argv.join(" "))
                    } else {
                        format!("Comando: {}", argv.join(" "))
                    }
                }
                ButtonAction::Copy => {
                    if is_en {
                        "Copy (Ctrl+C)".to_string()
                    } else {
                        "Copiar (Ctrl+C)".to_string()
                    }
                }
                ButtonAction::Paste => {
                    if is_en {
                        "Paste (Ctrl+V)".to_string()
                    } else {
                        "Colar (Ctrl+V)".to_string()
                    }
                }
                ButtonAction::OpenTerminal => {
                    if is_en {
                        "Open Terminal".to_string()
                    } else {
                        "Abrir Terminal".to_string()
                    }
                }
                ButtonAction::CloseWindow => {
                    if is_en {
                        "Close Window".to_string()
                    } else {
                        "Fechar Janela".to_string()
                    }
                }
                ButtonAction::Disabled => {
                    if is_en {
                        "Disabled".to_string()
                    } else {
                        "Desativado".to_string()
                    }
                }
                _ => format!("{action:?}"),
            };

            let card_bg = if is_selected {
                theme.panel_elevated
            } else {
                theme.panel
            };

            let border_color = if is_selected {
                theme.accent_blue
            } else {
                theme.border
            };

            let cb = on_select_button.clone();
            let hex = hotspot.hex_code.to_string();

            let mut card = h_flex()
                .id(ElementId::from(cat_idx * 100 + idx))
                .w_full()
                .p_2p5()
                .rounded_lg()
                .bg(card_bg)
                .border_1()
                .border_color(border_color)
                .cursor_pointer()
                .items_center()
                .justify_between()
                .hover(|s| s.bg(theme.panel_elevated))
                .on_click(move |_, _, cx| cb(hex.clone(), cx))
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(if is_selected {
                            div()
                                .w(px(3.0))
                                .h(px(22.0))
                                .rounded_full()
                                .bg(theme.accent_blue)
                        } else {
                            div().w(px(3.0)).h(px(22.0))
                        })
                        .child(
                            v_flex()
                                .gap_0p5()
                                .child(
                                    h_flex().gap_1p5().items_center().child(
                                        div()
                                            .text_size(px(13.0))
                                            .font_weight(if is_selected {
                                                gpui::FontWeight::BOLD
                                            } else {
                                                gpui::FontWeight::SEMIBOLD
                                            })
                                            .text_color(theme.text_primary)
                                            .child(hotspot.name(language)),
                                    ),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(if !is_supported {
                                            theme.warning
                                        } else if is_custom_mapped {
                                            theme.accent_green
                                        } else {
                                            theme.text_muted
                                        })
                                        .child(if is_supported {
                                            action_label
                                        } else {
                                            if is_en {
                                                "Not detected in current mode".to_string()
                                            } else {
                                                "Não detectado no modo atual".to_string()
                                            }
                                        }),
                                ),
                        ),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(theme.bg)
                        .border_1()
                        .border_color(theme.border)
                        .text_size(px(10.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.text_muted)
                        .child(hotspot.hex_code),
                );

            if !is_supported {
                card = card.opacity(0.65);
            }

            group_list = group_list.child(card);
        }

        main_container = main_container.child(
            v_flex()
                .gap_2()
                .w_full()
                .child(section_header)
                .child(group_list),
        );
    }

    v_flex()
        .w(px(290.0))
        .flex_shrink_0()
        .h_full()
        .p_3p5()
        .bg(theme.panel)
        .border_r_1()
        .border_color(theme.border)
        .overflow_y_scrollbar()
        .gap_4()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .text_size(px(14.0))
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.text_primary)
                        .child("Mapeamento de Botões"),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(theme.text_muted)
                        .child(format!("{} controles", ALL_HOTSPOTS.len())),
                ),
        )
        .child(main_container)
}
