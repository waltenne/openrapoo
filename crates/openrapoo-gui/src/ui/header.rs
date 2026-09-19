//! Contextual header bar component for OpenRapoo GPUI window.

use crate::device_model::{DeviceUiExt, RapooDevice};
use crate::i18n::{self, Language};
use crate::theme::{current_theme, HEADER_HEIGHT};
use gpui::{
    div, px, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    StyledImage,
};
use gpui_component::h_flex;
use openrapoo_core::battery::BatteryStatus;

pub fn render_header(
    device: Option<&RapooDevice>,
    active_profile_name: &str,
    language: Language,
    on_back: impl Fn(&mut gpui::App) + 'static,
) -> impl IntoElement {
    let theme = current_theme();

    let is_device_mode = device.is_some();

    let left_section = if is_device_mode {
        h_flex().items_center().child(
            div()
                .id("btn-back")
                .px_3()
                .py_1p5()
                .rounded_md()
                .bg(theme.panel_elevated)
                .text_color(theme.text_primary)
                .text_size(px(13.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .cursor_pointer()
                .hover(move |s| s.bg(theme.border))
                .on_click(move |_, _, cx| on_back(cx))
                .child(i18n::back_to_devices(language)),
        )
    } else {
        let app_icon_path = crate::assets::get_app_icon_path();
        let logo_element = if let Some(path) = app_icon_path {
            div().w(px(20.0)).h(px(20.0)).child(
                gpui::img(path)
                    .w(px(20.0))
                    .h(px(20.0))
                    .object_fit(gpui::ObjectFit::Contain),
            )
        } else {
            div().text_size(px(18.0)).child("⚡")
        };

        h_flex().gap_2().items_center().child(logo_element).child(
            div()
                .text_size(px(15.0))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(theme.text_primary)
                .child("OpenRapoo"),
        )
    };

    let center_section = if let Some(dev) = device {
        let dev_icon = match dev.device_type {
            openrapoo_core::device::DeviceType::Keyboard => "⌨️",
            _ => "🖱️",
        };

        let conn_badge = dev.connection_badge(language);

        let battery_element = match &dev.battery_status {
            BatteryStatus::Available {
                percentage,
                charging,
                ..
            } => {
                let text = if *charging {
                    format!("{percentage}% ⚡")
                } else {
                    format!("{percentage}%")
                };
                Some(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_full()
                        .bg(theme.accent_green)
                        .text_color(theme.panel)
                        .text_size(px(11.0))
                        .font_weight(gpui::FontWeight::BOLD)
                        .child(text),
                )
            }
            BatteryStatus::Charging {
                percentage: Some(pct),
                ..
            } => Some(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded_full()
                    .bg(theme.accent_blue)
                    .text_color(theme.text_primary)
                    .text_size(px(11.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(format!("{pct}% ⚡")),
            ),
            BatteryStatus::Full => Some(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded_full()
                    .bg(theme.accent_green)
                    .text_color(theme.panel)
                    .text_size(px(11.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("100% 🟢"),
            ),
            _ => None,
        };

        let mut center = h_flex()
            .gap_3()
            .items_center()
            .child(div().text_size(px(16.0)).child(dev_icon))
            .child(
                div()
                    .text_size(px(15.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme.text_primary)
                    .child(dev.name.clone()),
            )
            .child(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .bg(theme.panel_elevated)
                    .text_color(theme.text_muted)
                    .text_size(px(11.0))
                    .child(conn_badge),
            );

        if let Some(bat_pill) = battery_element {
            center = center.child(bat_pill);
        }

        center = center.child(
            div()
                .px_2p5()
                .py_0p5()
                .rounded_md()
                .bg(theme.panel_elevated)
                .text_color(theme.accent_blue)
                .text_size(px(11.0))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(format!(
                    "{}{}",
                    i18n::profile_prefix(language),
                    i18n::format_profile_name(active_profile_name, language)
                )),
        );

        center
    } else {
        h_flex().items_center().child(
            div()
                .text_size(px(15.0))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.text_muted)
                .child("Central de Dispositivos Rapoo"),
        )
    };

    h_flex()
        .h(px(HEADER_HEIGHT))
        .w_full()
        .bg(theme.panel)
        .px_6()
        .justify_between()
        .items_center()
        .border_b_1()
        .border_color(theme.border)
        .child(left_section)
        .child(center_section)
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .w(px(80.0))
                .justify_end()
                .child(
                    div()
                        .id("btn-minimize")
                        .w(px(28.0))
                        .h(px(28.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_md()
                        .bg(theme.panel_elevated)
                        .text_color(theme.text_muted)
                        .text_size(px(13.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.border).text_color(theme.text_primary))
                        .on_click(|_, window, _| {
                            window.minimize_window();
                        })
                        .child("─"),
                )
                .child(
                    div()
                        .id("btn-close")
                        .w(px(28.0))
                        .h(px(28.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_md()
                        .bg(theme.panel_elevated)
                        .text_color(theme.text_muted)
                        .text_size(px(13.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.error).text_color(theme.text_primary))
                        .on_click(|_, _, cx| {
                            cx.quit();
                        })
                        .child("✕"),
                ),
        )
}
