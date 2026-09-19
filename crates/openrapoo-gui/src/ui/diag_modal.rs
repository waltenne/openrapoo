//! Modal Popup component for viewing, copying, and exporting Diagnostic Reports.

use crate::i18n::Language;
use crate::theme::current_theme;
use gpui::{
    div, px, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex};
use std::rc::Rc;

pub fn render_diag_modal(
    report_markdown: &str,
    toast_msg: Option<&str>,
    language: Language,
    on_copy: Rc<dyn Fn(String, &mut gpui::App)>,
    on_export: Rc<dyn Fn(String, &mut gpui::App)>,
    on_close: Rc<dyn Fn(&mut gpui::App)>,
) -> impl IntoElement {
    let theme = current_theme();
    let is_en = matches!(language, Language::English);

    let text_copy = report_markdown.to_string();
    let text_export = report_markdown.to_string();
    let cb_copy = on_copy.clone();
    let cb_export = on_export.clone();
    let cb_close = on_close.clone();
    let cb_close_bg = on_close;

    div()
        .id("diag-modal-backdrop")
        .absolute()
        .inset_0()
        .bg(gpui::hsla(0.0, 0.0, 0.0, 0.75))
        .flex()
        .items_center()
        .justify_center()
        .p_6()
        .on_click(move |_, _, cx| cb_close_bg(cx))
        .child(
            v_flex()
                .id("diag-modal-container")
                .w(px(720.0))
                .max_w_full()
                .max_h(px(560.0))
                .rounded_2xl()
                .bg(theme.panel)
                .border_1()
                .border_color(theme.border)
                .gap_0()
                .overflow_hidden()
                .on_click(|_, _, _| {}) // prevent click propagation to backdrop
                .child(
                    // Header
                    h_flex()
                        .w_full()
                        .justify_between()
                        .items_center()
                        .px_6()
                        .py_4()
                        .bg(theme.panel_elevated)
                        .border_b_1()
                        .border_color(theme.border)
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(div().text_size(px(18.0)).child("📄"))
                                .child(
                                    div()
                                        .text_size(px(15.0))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(theme.text_primary)
                                        .child(if is_en {
                                            "Technical Diagnostic Report"
                                        } else {
                                            "Relatório Técnico de Diagnóstico"
                                        }),
                                ),
                        )
                        .child(
                            div()
                                .id("btn-modal-close-icon")
                                .w(px(28.0))
                                .h(px(28.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded_md()
                                .bg(theme.panel)
                                .text_color(theme.text_muted)
                                .cursor_pointer()
                                .hover(|s| s.bg(theme.error).text_color(theme.text_primary))
                                .on_click(move |_, _, cx| cb_close(cx))
                                .child("✕"),
                        ),
                )
                .child(
                    // Scrollable monospaced body
                    v_flex()
                        .flex_1()
                        .w_full()
                        .p_5()
                        .overflow_y_scrollbar()
                        .child(
                            div()
                                .w_full()
                                .p_4()
                                .rounded_lg()
                                .bg(theme.bg)
                                .border_1()
                                .border_color(theme.border)
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .font_family("Monospace")
                                        .text_color(theme.text_primary)
                                        .child(report_markdown.to_string()),
                                ),
                        ),
                )
                .child(
                    // Footer with Toast & Actions
                    v_flex()
                        .w_full()
                        .px_6()
                        .py_4()
                        .border_t_1()
                        .border_color(theme.border)
                        .bg(theme.panel_elevated)
                        .gap_3()
                        .child(if let Some(msg) = toast_msg {
                            div()
                                .w_full()
                                .p_2()
                                .rounded_md()
                                .bg(theme.accent_green)
                                .text_color(theme.panel)
                                .text_size(px(12.0))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_align(gpui::TextAlign::Center)
                                .child(msg.to_string())
                        } else {
                            div()
                        })
                        .child(
                            h_flex().w_full().justify_between().items_center().child(
                                h_flex()
                                    .gap_3()
                                    .child(
                                        div()
                                            .id("btn-modal-copy")
                                            .px_4()
                                            .py_2()
                                            .rounded_md()
                                            .bg(theme.accent_blue)
                                            .text_color(theme.text_primary)
                                            .text_size(px(13.0))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .cursor_pointer()
                                            .hover(|s| s.bg(theme.border))
                                            .on_click(move |_, _, cx| {
                                                cb_copy(text_copy.clone(), cx)
                                            })
                                            .child(if is_en {
                                                "📋 Copy Content"
                                            } else {
                                                "📋 Copiar Conteúdo"
                                            }),
                                    )
                                    .child(
                                        div()
                                            .id("btn-modal-export")
                                            .px_4()
                                            .py_2()
                                            .rounded_md()
                                            .bg(theme.panel)
                                            .border_1()
                                            .border_color(theme.border)
                                            .text_color(theme.text_primary)
                                            .text_size(px(13.0))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .cursor_pointer()
                                            .hover(|s| s.bg(theme.border))
                                            .on_click(move |_, _, cx| {
                                                cb_export(text_export.clone(), cx)
                                            })
                                            .child(if is_en {
                                                "💾 Export File (.md)"
                                            } else {
                                                "💾 Exportar Arquivo (.md)"
                                            }),
                                    ),
                            ),
                        ),
                ),
        )
}
