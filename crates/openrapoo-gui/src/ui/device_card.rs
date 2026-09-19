//! Device card widget for the "Dispositivos" home view.

use crate::device_model::{DeviceUiExt, RapooDevice};
use crate::i18n::Language;
use crate::theme::current_theme;
use gpui::{
    div, px, ElementId, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, StyledImage,
};
use gpui_component::{h_flex, v_flex};
use openrapoo_core::battery::BatteryStatus;

use std::rc::Rc;

pub fn render_device_card(
    index: usize,
    device: &RapooDevice,
    is_selected: bool,
    language: Language,
    _on_highlight: Rc<dyn Fn(usize, &mut gpui::App)>,
    on_confirm: Rc<dyn Fn(usize, &mut gpui::App)>,
) -> impl IntoElement {
    let theme = current_theme();

    let border_color = if is_selected {
        theme.accent_blue
    } else {
        theme.border
    };

    let bg_color = if is_selected {
        theme.panel_elevated
    } else {
        theme.panel
    };

    let conn_badge = device.connection_badge(language);
    let type_badge = device.type_badge(language);

    let (conn_text, conn_color) = if device.is_connected() {
        (
            match language {
                Language::English => "Connected 🟢",
                Language::Portuguese => "Conectado 🟢",
            },
            theme.accent_green,
        )
    } else {
        (
            match language {
                Language::English => "Disconnected 🔴",
                Language::Portuguese => "Desconectado 🔴",
            },
            theme.warning,
        )
    };

    let state_indicator = div()
        .px_2p5()
        .py_0p5()
        .rounded_full()
        .bg(conn_color)
        .text_color(theme.panel)
        .text_size(px(11.0))
        .font_weight(gpui::FontWeight::BOLD)
        .child(conn_text);

    use openrapoo_core::device::DeviceType;

    let image_section = if device.device_type == DeviceType::Keyboard {
        if let Some(png_path) = crate::assets::get_e9050l_png_path() {
            div()
                .w_full()
                .h(px(130.0))
                .py_2()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    gpui::img(png_path)
                        .w(px(250.0))
                        .h(px(115.0))
                        .object_fit(gpui::ObjectFit::Contain),
                )
        } else {
            div()
                .w_full()
                .h(px(100.0))
                .py_2()
                .flex()
                .items_center()
                .justify_center()
                .child(div().text_size(px(42.0)).child("⌨️"))
        }
    } else if let Some(png_path) = crate::assets::get_mt760_pro_png_path() {
        div()
            .w_full()
            .h(px(130.0))
            .py_2()
            .flex()
            .items_center()
            .justify_center()
            .child(
                gpui::img(png_path)
                    .w(px(125.0))
                    .h(px(120.0))
                    .object_fit(gpui::ObjectFit::Contain),
            )
    } else {
        div()
            .w_full()
            .h(px(100.0))
            .py_2()
            .flex()
            .items_center()
            .justify_center()
            .child(div().text_size(px(42.0)).child("🖱️"))
    };

    let battery_label = match &device.battery_status {
        BatteryStatus::Available {
            percentage,
            charging,
            ..
        } => {
            if *charging {
                format!("Bateria: {percentage}% (Carregando ⚡)")
            } else {
                format!("Bateria: {percentage}%")
            }
        }
        BatteryStatus::Charging {
            percentage: Some(pct),
            ..
        } => format!("Bateria: {pct}% (Carregando ⚡)"),
        BatteryStatus::Charging {
            percentage: None, ..
        } => "Bateria: Carregando ⚡".to_string(),
        BatteryStatus::Full => "Bateria: 100% (Completa) 🟢".to_string(),
        BatteryStatus::Discharging { percentage } => format!("Bateria: {percentage}%"),
        _ => "Bateria: Indisponível".to_string(),
    };

    let cb_card_click = on_confirm.clone();

    v_flex()
        .id(ElementId::from(index))
        .w(px(320.0))
        .max_w(px(340.0))
        .p_5()
        .rounded_xl()
        .bg(bg_color)
        .border_2()
        .border_color(border_color)
        .cursor_pointer()
        .hover(|s| s.bg(theme.panel_elevated).border_color(theme.accent_blue))
        .on_click(move |_, _, cx| cb_card_click(index, cx))
        .gap_3()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .px_2p5()
                        .py_0p5()
                        .rounded_md()
                        .bg(theme.panel_elevated)
                        .text_color(theme.text_muted)
                        .text_size(px(11.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .child(type_badge),
                )
                .child(state_indicator),
        )
        .child(image_section)
        .child(
            v_flex()
                .gap_1()
                .items_center()
                .w_full()
                .child(
                    div()
                        .w_full()
                        .text_align(gpui::TextAlign::Center)
                        .text_size(px(17.0))
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.text_primary)
                        .truncate()
                        .child(device.name.clone()),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(theme.text_muted)
                        .child("Fabricante: Rapoo"),
                ),
        )
        .child(
            v_flex()
                .w_full()
                .gap_1p5()
                .pt_3()
                .border_t_1()
                .border_color(theme.border)
                .child(
                    div()
                        .w_full()
                        .text_size(px(12.0))
                        .text_color(theme.text_muted)
                        .truncate()
                        .child(format!("Conexão: {conn_badge}")),
                )
                .child(
                    div()
                        .w_full()
                        .text_size(px(12.0))
                        .text_color(
                            if matches!(
                                device.battery_status,
                                BatteryStatus::Available { .. }
                                    | BatteryStatus::Full
                                    | BatteryStatus::Charging { .. }
                            ) {
                                theme.text_primary
                            } else {
                                theme.text_muted
                            },
                        )
                        .truncate()
                        .child(battery_label),
                ),
        )
}
