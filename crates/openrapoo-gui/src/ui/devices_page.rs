//! "Dispositivos" home view page rendering the list of detected Rapoo devices.

use crate::device_model::RapooDevice;
use crate::i18n::{self, Language};
use crate::theme::current_theme;
use crate::ui::device_card::render_device_card;
use gpui::{
    div, px, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex};
use std::rc::Rc;

use gpui::ElementId;

pub fn render_devices_page(
    devices: &[RapooDevice],
    selected_index: Option<usize>,
    language: Language,
    on_highlight_device: Rc<dyn Fn(usize, &mut gpui::App)>,
    on_confirm_device: Rc<dyn Fn(usize, &mut gpui::App)>,
    on_prev_device: Rc<dyn Fn(&mut gpui::App)>,
    on_next_device: Rc<dyn Fn(&mut gpui::App)>,
    on_refresh: impl Fn(&mut gpui::App) + 'static,
) -> impl IntoElement {
    let theme = current_theme();

    if devices.is_empty() {
        return v_flex()
            .flex_1()
            .w_full()
            .h_full()
            .items_center()
            .justify_center()
            .gap_4()
            .p_8()
            .child(
                div()
                    .text_size(px(48.0))
                    .child("🖱️"),
            )
            .child(
                div()
                    .text_size(px(20.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme.text_primary)
                    .child(i18n::no_device_title(language)),
            )
            .child(
                div()
                    .max_w(px(480.0))
                    .text_size(px(13.0))
                    .text_color(theme.text_muted)
                    .text_align(gpui::TextAlign::Center)
                    .child(i18n::no_device_desc(language)),
            )
            .child(
                div()
                    .id("btn-empty-refresh")
                    .px_4()
                    .py_2()
                    .rounded_md()
                    .bg(theme.accent_blue)
                    .text_color(theme.text_primary)
                    .text_size(px(13.0))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.panel_elevated))
                    .on_click(move |_, _, cx| on_refresh(cx))
                    .child(i18n::try_detect_again(language)),
            )
            .into_any_element();
    }

    let sel_idx = selected_index.unwrap_or(0);

    let visible_indices: Vec<usize> = if devices.len() <= 2 {
        (0..devices.len()).collect()
    } else {
        let next_idx = (sel_idx + 1) % devices.len();
        vec![sel_idx, next_idx]
    };

    let mut carousel_items = h_flex()
        .gap_6()
        .justify_center()
        .items_center()
        .py_2();

    for &idx in &visible_indices {
        let dev = &devices[idx];
        let is_sel = selected_index == Some(idx);
        let cb_hl = on_highlight_device.clone();
        let cb_cf = on_confirm_device.clone();
        carousel_items = carousel_items.child(render_device_card(
            idx, dev, is_sel, language, cb_hl, cb_cf,
        ));
    }

    let cb_prev_btn = on_prev_device.clone();
    let cb_next_btn = on_next_device.clone();

    let carousel_container = h_flex()
        .w_full()
        .justify_center()
        .items_center()
        .gap_4()
        .child(if devices.len() > 1 {
            div()
                .id("btn-carousel-left-arrow")
                .px_3()
                .py_6()
                .rounded_lg()
                .bg(theme.panel_elevated)
                .text_color(theme.text_primary)
                .text_size(px(22.0))
                .font_weight(gpui::FontWeight::BOLD)
                .cursor_pointer()
                .hover(|s| s.bg(theme.accent_blue).text_color(theme.text_primary))
                .on_click(move |_, _, cx| cb_prev_btn(cx))
                .child("❮")
                .into_any_element()
        } else {
            div().into_any_element()
        })
        .child(carousel_items)
        .child(if devices.len() > 1 {
            div()
                .id("btn-carousel-right-arrow")
                .px_3()
                .py_6()
                .rounded_lg()
                .bg(theme.panel_elevated)
                .text_color(theme.text_primary)
                .text_size(px(22.0))
                .font_weight(gpui::FontWeight::BOLD)
                .cursor_pointer()
                .hover(|s| s.bg(theme.accent_blue).text_color(theme.text_primary))
                .on_click(move |_, _, cx| cb_next_btn(cx))
                .child("❯")
                .into_any_element()
        } else {
            div().into_any_element()
        });

    let carousel_indicators = if devices.len() > 1 {
        let mut dots = h_flex().gap_3().justify_center().items_center().pt_2();
        for idx in 0..devices.len() {
            let is_sel = selected_index == Some(idx);
            let dot_color = if is_sel {
                theme.accent_blue
            } else {
                theme.border
            };
            let dot_size = if is_sel { px(12.0) } else { px(8.0) };
            let cb_hl = on_highlight_device.clone();
            dots = dots.child(
                div()
                    .id(ElementId::from(("dot", idx)))
                    .w(dot_size)
                    .h(dot_size)
                    .rounded_full()
                    .bg(dot_color)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.accent_blue))
                    .on_click(move |_, _, cx| cb_hl(idx, cx)),
            );
        }
        Some(dots)
    } else {
        None
    };

    let navigation_hint = h_flex()
        .w_full()
        .justify_center()
        .items_center()
        .pt_3()
        .child(
            div()
                .px_3()
                .py_1p5()
                .rounded_md()
                .bg(theme.panel_elevated)
                .text_color(theme.text_muted)
                .text_size(px(12.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(i18n::navigation_hint(language)),
        );

    let on_next_scroll = on_next_device.clone();
    let on_prev_scroll = on_prev_device.clone();

    let on_next_key = on_next_device.clone();
    let on_prev_key = on_prev_device.clone();
    let on_confirm_key = on_confirm_device.clone();
    let current_idx = selected_index.unwrap_or(0);

    div()
        .id("devices-page-root")
        .flex_1()
        .w_full()
        .h_full()
        .on_scroll_wheel(move |event, _window, cx| {
            let (dx, dy) = match event.delta {
                gpui::ScrollDelta::Pixels(pt) => (f32::from(pt.x), f32::from(pt.y)),
                gpui::ScrollDelta::Lines(pt) => (pt.x, pt.y),
            };
            if dy < -1.0 || dx > 1.0 {
                on_next_scroll(cx);
            } else if dy > 1.0 || dx < -1.0 {
                on_prev_scroll(cx);
            }
        })
        .on_key_down(move |event, _window, cx| {
            let key = event.keystroke.key.to_lowercase();
            match key.as_str() {
                "arrowleft" | "arrowup" | "left" | "up" | "<" | "," | "h" => {
                    on_prev_key(cx);
                }
                "arrowright" | "arrowdown" | "right" | "down" | ">" | "." | "l" => {
                    on_next_key(cx);
                }
                "enter" | "return" | "space" => {
                    on_confirm_key(current_idx, cx);
                }
                _ => {}
            }
        })
        .child(
            v_flex()
                .flex_1()
                .w_full()
                .h_full()
                .p_6()
                .gap_4()
                .overflow_y_scrollbar()
                .child(
                    h_flex().w_full().justify_center().child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text_muted)
                            .child(format!("Dispositivos Conectados ({})", devices.len())),
                    ),
                )
                .child(carousel_container)
                .children(carousel_indicators)
                .child(navigation_hint),
        )
        .into_any_element()
}
