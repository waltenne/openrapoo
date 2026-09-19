//! Status bar component for OpenRapoo GPUI window footer.

use crate::i18n::{self, Language};
use crate::theme::{current_theme, STATUS_BAR_HEIGHT};
use gpui::{
    div, px, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
};
use gpui_component::h_flex;

pub fn render_status_bar(
    group_ok: bool,
    daemon_running: bool,
    active_profile_name: &str,
    language: Language,
    on_refresh: impl Fn(&mut gpui::App) + 'static,
    on_toggle_language: impl Fn(&mut gpui::App) + 'static,
) -> impl IntoElement {
    let theme = current_theme();

    let group_dot = if group_ok {
        theme.accent_green
    } else {
        theme.warning
    };

    let daemon_dot = if daemon_running {
        theme.accent_green
    } else {
        theme.text_muted
    };

    let group_label = if group_ok {
        i18n::input_group_ok(language)
    } else {
        i18n::input_group_no_access(language)
    };

    let daemon_label = if daemon_running {
        i18n::daemon_active(language)
    } else {
        i18n::daemon_inactive(language)
    };

    let profile_label = format!(
        "{}{}",
        i18n::profile_prefix(language),
        i18n::format_profile_name(active_profile_name, language)
    );

    h_flex()
        .h(px(STATUS_BAR_HEIGHT))
        .w_full()
        .px_4()
        .justify_center()
        .items_center()
        .gap_3()
        .bg(theme.panel)
        .border_t_1()
        .border_color(theme.border)
        .text_size(px(11.0))
        .child(
            div()
                .id("btn-footer-refresh")
                .px_2p5()
                .py_1()
                .rounded_md()
                .bg(theme.panel_elevated)
                .border_1()
                .border_color(theme.border)
                .text_color(theme.text_primary)
                .text_size(px(11.0))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .cursor_pointer()
                .hover(|s| {
                    s.bg(theme.border)
                        .border_color(theme.accent_blue)
                        .text_color(theme.accent_blue)
                })
                .on_click(move |_, _, cx| on_refresh(cx))
                .child(
                    h_flex()
                        .gap_1p5()
                        .items_center()
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(gpui::FontWeight::BOLD)
                                .child("↻"),
                        )
                        .child(i18n::refresh(language)),
                ),
        )
        .child(
            h_flex()
                .gap_1p5()
                .items_center()
                .px_2p5()
                .py_1()
                .rounded_md()
                .bg(theme.panel_elevated)
                .child(div().w(px(7.0)).h(px(7.0)).rounded_full().bg(group_dot))
                .child(
                    div()
                        .text_color(theme.text_muted)
                        .text_size(px(11.0))
                        .child(group_label),
                ),
        )
        .child(
            h_flex()
                .gap_1p5()
                .items_center()
                .px_2p5()
                .py_1()
                .rounded_md()
                .bg(theme.panel_elevated)
                .child(div().w(px(7.0)).h(px(7.0)).rounded_full().bg(daemon_dot))
                .child(
                    div()
                        .text_color(theme.text_muted)
                        .text_size(px(11.0))
                        .child(daemon_label),
                ),
        )
        .child(
            div()
                .px_2p5()
                .py_1()
                .rounded_md()
                .bg(theme.panel_elevated)
                .text_color(theme.text_muted)
                .text_size(px(11.0))
                .truncate()
                .child(profile_label),
        )
        .child(
            div()
                .id("btn-footer-language")
                .px_2p5()
                .py_1()
                .rounded_md()
                .bg(theme.panel_elevated)
                .border_1()
                .border_color(theme.border)
                .text_color(theme.accent_blue)
                .text_size(px(11.0))
                .font_weight(gpui::FontWeight::BOLD)
                .cursor_pointer()
                .hover(|s| {
                    s.bg(theme.accent_blue)
                        .border_color(theme.accent_blue)
                        .text_color(theme.panel)
                })
                .on_click(move |_, _, cx| on_toggle_language(cx))
                .child(format!("🌐 {}", language.code())),
        )
}
