//! Mouse View component rendering transparent mt760-pro.png image, overlay hotspots layer, and canvas leader lines.

use crate::assets::get_mt760_pro_png_path;
use crate::theme::{current_theme, ACCENT_GREEN};
use crate::ui::hotspot::{Side, ALL_HOTSPOTS};
use gpui::{
    canvas, div, img, point, px, ElementId, InteractiveElement, IntoElement, ParentElement,
    PathBuilder, StatefulInteractiveElement, Styled, StyledImage,
};
use std::rc::Rc;

pub fn render_mouse_view(
    selected_hex: Option<&str>,
    on_select_hotspot: Rc<dyn Fn(String, &mut gpui::App)>,
) -> impl IntoElement {
    let theme = current_theme();

    let png_path = get_mt760_pro_png_path();

    // Render dimensions matching mt760-pro.png aspect ratio
    let render_w = 330.0f32;
    let render_h = 440.0f32;

    let mouse_image = if let Some(ref path) = png_path {
        div().w_full().h_full().child(
            img(path.clone())
                .w(px(render_w))
                .h(px(render_h))
                .object_fit(gpui::ObjectFit::Contain),
        )
    } else {
        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2()
            .text_color(theme.text_muted)
            .child(div().text_size(px(32.0)).child("🖼️"))
            .child(
                div()
                    .text_size(px(14.0))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("Imagem mt760-pro.png Não Encontrada"),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .child("Certifique-se de que o arquivo esteja em crates/openrapoo-gui/assets/mt760-pro.png"),
            )
    };

    let mut hotspots_container = div().absolute().top_0().left_0().w_full().h_full();

    for (idx, hotspot) in ALL_HOTSPOTS.iter().enumerate() {
        let (cx, cy) = hotspot.scaled_center(render_w, render_h);
        // Center 14px circular dot precisely on hotspot (cx, cy)
        let abs_x = cx - 7.0;
        let abs_y = cy - 7.0;

        let is_selected = selected_hex == Some(hotspot.hex_code);
        let is_supported = hotspot.is_supported_on_linux;

        let cb = on_select_hotspot.clone();
        let hex = hotspot.hex_code.to_string();

        let mut dot = div()
            .id(ElementId::from(idx + 100))
            .absolute()
            .left(px(abs_x))
            .top(px(abs_y))
            .w(px(14.0))
            .h(px(14.0))
            .rounded_full();

        if !is_supported {
            dot = dot
                .bg(theme.panel_elevated)
                .border_1()
                .border_color(theme.border)
                .opacity(0.4)
                .cursor_not_allowed()
                .child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(8.0))
                        .text_color(theme.text_muted)
                        .child("🔒"),
                );
        } else if is_selected {
            dot = dot
                .bg(ACCENT_GREEN)
                .border_2()
                .border_color(theme.text_primary)
                .cursor_pointer()
                .child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(9.0))
                        .text_color(theme.panel)
                        .font_weight(gpui::FontWeight::BOLD)
                        .child("✓"),
                );
        } else {
            dot = dot
                .bg(theme.accent_blue)
                .border_2()
                .border_color(theme.text_primary)
                .cursor_pointer()
                .hover(|s| s.bg(ACCENT_GREEN).border_color(theme.text_primary))
                .on_click(move |_, _, cx| cb(hex.clone(), cx));
        }

        hotspots_container = hotspots_container.child(dot);
    }

    // Canvas connecting lines between hotspot centers and side cards
    let selected_code = selected_hex.map(str::to_string);
    let leader_canvas = canvas(
        move |_, _, _| selected_code,
        move |bounds, selected_code, window, _| {
            let canvas_origin = bounds.origin;
            for hotspot in ALL_HOTSPOTS {
                let is_selected = selected_code.as_deref() == Some(hotspot.hex_code);
                if !is_selected {
                    continue;
                }

                let (cx, cy) = hotspot.scaled_center(render_w, render_h);
                let start_point = point(canvas_origin.x + px(cx), canvas_origin.y + px(cy));

                let stub_x = match hotspot.side {
                    Side::Left => canvas_origin.x + px((cx - 50.0).max(0.0)),
                    Side::Right => canvas_origin.x + px((cx + 50.0).min(render_w)),
                };

                let stub_point = point(stub_x, canvas_origin.y + px(cy));

                let mut path = PathBuilder::stroke(px(2.5));
                path.move_to(start_point);
                path.line_to(stub_point);

                if let Ok(built) = path.build() {
                    window.paint_path(built, ACCENT_GREEN);
                }
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .w_full()
    .h_full();

    div()
        .flex_1()
        .h_full()
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .relative()
                .w(px(render_w))
                .h(px(render_h))
                .child(mouse_image)
                .child(leader_canvas)
                .child(hotspots_container),
        )
}
