//! "Botões" (Buttons) Tab View Component — OpenLogi Minimalist Style.

use crate::widgets::mouse_view::{
    get_button_action_label, RapooButtonInfo, CardSide, RAPOO_MT760_BUTTONS,
};
use openrapoo_core::config::Profile;

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

#[cfg(feature = "gtk")]
pub mod gtk_ui {
    use super::*;
    use gtk4::prelude::*;
    use gtk4::{Box, Button, DrawingArea, Label, Orientation, Overlay, Picture};
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Build the main OpenLogi-style canvas view for the "Botões" tab.
    pub fn build_buttons_tab_ui<F>(profile: Rc<RefCell<Profile>>, on_select_button: F) -> Box
    where
        F: Fn(&RapooButtonInfo) + 'static + Clone,
    {
        let main_box = Box::new(Orientation::Horizontal, 24);
        main_box.add_css_class("main-canvas");
        main_box.set_hexpand(true);
        main_box.set_vexpand(true);
        main_box.set_halign(gtk4::Align::Center);
        main_box.set_valign(gtk4::Align::Center);

        // 1. Left Action Cards Column
        let left_col = Box::new(Orientation::Vertical, 10);
        left_col.set_valign(gtk4::Align::Center);

        // 2. Right Action Cards Column
        let right_col = Box::new(Orientation::Vertical, 10);
        right_col.set_valign(gtk4::Align::Center);

        let profile_ref = profile.borrow();

        for btn in RAPOO_MT760_BUTTONS.iter() {
            let card = build_action_card(btn, &profile_ref, {
                let btn = btn.clone();
                let cb = on_select_button.clone();
                move |_| cb(&btn)
            });

            match btn.card_side {
                CardSide::Left => left_col.append(&card),
                CardSide::Right => right_col.append(&card),
            }
        }

        // 3. Center Mouse Image Overlay & Drawing Area for Leader Lines
        let overlay = Overlay::new();
        overlay.set_hexpand(true);
        overlay.set_vexpand(true);
        overlay.set_halign(gtk4::Align::Center);
        overlay.set_valign(gtk4::Align::Center);

        // Load Rapoo MT760 Pro SVG asset
        let picture = Picture::new();
        picture.set_filename(Some("crates/openrapoo-gui/assets/rapoo_mt760_pro.svg"));
        picture.set_content_fit(gtk4::ContentFit::Contain);
        picture.set_size_request(340, 460);

        overlay.set_child(Some(&picture));

        // DrawingArea over SVG for Leader Lines
        let drawing_area = DrawingArea::new();
        drawing_area.set_size_request(340, 460);

        // Paint polyline leader lines and glowing hotspot dots
        drawing_area.set_draw_func(move |_, cr, width, height| {
            let w = width as f64;
            let h = height as f64;

            for btn in RAPOO_MT760_BUTTONS.iter() {
                let hx = btn.hotspot_x_pct * w;
                let hy = btn.hotspot_y_pct * h;

                // Determine line start near card
                let (start_x, stub_x) = match btn.card_side {
                    CardSide::Left => (hx - 40.0, hx - 20.0),
                    CardSide::Right => (hx + 40.0, hx + 20.0),
                };

                // Paint subtle polyline
                cr.set_source_rgba(0.35, 0.38, 0.48, 0.45);
                cr.set_line_width(1.5);

                cr.move_to(start_x, hy);
                cr.line_to(stub_x, hy);
                cr.line_to(hx, hy);
                let _ = cr.stroke();

                // Paint outer glow ring
                cr.set_source_rgba(0.23, 0.51, 0.96, 0.35); // Accent blue glow
                cr.arc(hx, hy, 7.0, 0.0, 2.0 * std::f64::consts::PI);
                let _ = cr.fill();

                // Paint inner hotspot dot
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.95);
                cr.arc(hx, hy, 3.0, 0.0, 2.0 * std::f64::consts::PI);
                let _ = cr.fill();
            }
        });

        overlay.add_overlay(&drawing_area);

        // Assembly: Left Column | Center Overlay | Right Column
        main_box.append(&left_col);
        main_box.append(&overlay);
        main_box.append(&right_col);

        main_box
    }

    /// Build an individual OpenLogi-style Action Card widget
    fn build_action_card<F>(btn: &RapooButtonInfo, profile: &Profile, on_click: F) -> Button
    where
        F: Fn(&Button) + 'static,
    {
        let card_btn = Button::new();
        card_btn.add_css_class("openlogi-card");

        let inner_box = Box::new(Orientation::Vertical, 2);

        // Sublabel (e.g. "Clique do Meio")
        let sub_lbl = Label::new(Some(btn.name_pt));
        sub_lbl.add_css_class("card-label-sub");
        sub_lbl.set_halign(gtk4::Align::Start);

        // Title row (e.g. "Padrão do Sistema >")
        let title_row = Box::new(Orientation::Horizontal, 8);

        let current_action = get_button_action_label(profile, btn.hex_code);
        let title_lbl = Label::new(Some(&current_action));
        title_lbl.add_css_class("card-label-title");
        title_lbl.set_halign(gtk4::Align::Start);
        title_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        title_lbl.set_max_width_chars(18);

        let chevron_lbl = Label::new(Some("›"));
        chevron_lbl.add_css_class("card-chevron");
        chevron_lbl.set_halign(gtk4::Align::End);
        chevron_lbl.set_hexpand(true);

        title_row.append(&title_lbl);
        title_row.append(&chevron_lbl);

        inner_box.append(&sub_lbl);
        inner_box.append(&title_row);

        card_btn.set_child(Some(&inner_box));
        card_btn.connect_clicked(on_click);

        card_btn
    }
}
