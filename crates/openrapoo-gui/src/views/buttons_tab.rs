//! "Botões" (Buttons) Tab View Component — OpenLogi Interactive SVG Hotspot Layout.

use crate::widgets::mouse_view::{
    get_button_action_label, get_svg_asset_path, ALL_HOTSPOTS, BackendButton, CardSide,
    HotspotControl,
};
use openrapoo_core::config::Profile;

#[derive(Debug, Clone)]
pub struct ButtonsTabModel {
    pub selected_button: Option<BackendButton>,
}

impl Default for ButtonsTabModel {
    fn default() -> Self {
        ButtonsTabModel {
            selected_button: None,
        }
    }
}

#[cfg(feature = "gtk")]
pub mod gtk_ui {
    use super::*;
    use gtk4::glib;
    use gtk4::prelude::*;
    use gtk4::{Box, Button, DrawingArea, Label, Orientation, Overlay, Picture};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    /// Build the main interactive SVG canvas view for the "Botões" tab.
    pub fn build_buttons_tab_ui<F>(
        profile: Rc<RefCell<Profile>>,
        on_select_button: F,
    ) -> (Box, Rc<RefCell<Option<BackendButton>>>, Rc<dyn Fn(BackendButton, &str)>)
    where
        F: Fn(&HotspotControl) + 'static + Clone,
    {
        let selected_button = Rc::new(RefCell::new(None::<BackendButton>));

        let main_box = Box::new(Orientation::Horizontal, 16);
        main_box.add_css_class("main-canvas");
        main_box.set_hexpand(true);
        main_box.set_vexpand(true);

        // 1. Left Action Cards Column
        let left_col = Box::new(Orientation::Vertical, 8);
        left_col.set_valign(gtk4::Align::Center);

        // 2. Right Action Cards Column
        let right_col = Box::new(Orientation::Vertical, 8);
        right_col.set_valign(gtk4::Align::Center);

        let card_map: Rc<RefCell<HashMap<BackendButton, Button>>> =
            Rc::new(RefCell::new(HashMap::new()));

        let profile_ref = profile.borrow();

        for hotspot in ALL_HOTSPOTS.iter() {
            let card = build_action_card(hotspot, &profile_ref, {
                let hotspot = hotspot.clone();
                let cb = on_select_button.clone();
                let sel_ref = selected_button.clone();
                let cmap_ref = card_map.clone();
                move |_| {
                    sel_ref.replace(Some(hotspot.backend_button));
                    update_card_selections(&cmap_ref.borrow(), Some(hotspot.backend_button));
                    cb(&hotspot);
                }
            });

            card_map.borrow_mut().insert(hotspot.backend_button, card.clone());

            match hotspot.card_side {
                CardSide::Left => left_col.append(&card),
                CardSide::Right => right_col.append(&card),
            }
        }

        // 3. Center SVG Overlay with Interactive Hotspots
        let overlay = Overlay::new();
        overlay.set_hexpand(true);
        overlay.set_vexpand(true);

        let svg_path = get_svg_asset_path();
        let picture = Picture::new();
        picture.set_file(Some(&gtk4::gio::File::for_path(&svg_path)));
        picture.set_content_fit(gtk4::ContentFit::Contain);
        picture.set_size_request(600, 480);
        picture.set_hexpand(true);
        picture.set_vexpand(true);

        overlay.set_child(Some(&picture));

        // DrawingArea over SVG for Leader Lines, Selected Pulse, and Hotspot Rendering
        let drawing_area = DrawingArea::new();
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);

        let sel_for_draw = selected_button.clone();

        let anim_state: Rc<RefCell<HashMap<BackendButton, (String, u64)>>> =
            Rc::new(RefCell::new(HashMap::new()));

        let anim_for_draw = anim_state.clone();

        drawing_area.set_draw_func(move |_, cr, width, height| {
            let w = width as f64;
            let h = height as f64;

            // SVG native viewBox: 1200 x 820
            let svg_native_w = 1200.0;
            let svg_native_h = 820.0;

            // Calculate uniform scale and offset for ContentFit::Contain
            let scale = (w / svg_native_w).min(h / svg_native_h);
            let offset_x = (w - svg_native_w * scale) / 2.0;
            let offset_y = (h - svg_native_h * scale) / 2.0;

            let cur_selected = *sel_for_draw.borrow();
            let cur_anims = anim_for_draw.borrow();

            for hotspot in ALL_HOTSPOTS.iter() {
                let hx = offset_x + hotspot.svg_cx * scale;
                let hy = offset_y + hotspot.svg_cy * scale;

                let is_sel = cur_selected == Some(hotspot.backend_button);
                let anim_class = cur_anims.get(&hotspot.backend_button).map(|(c, _)| c.as_str());

                // Paint outer ring / glow
                if is_sel {
                    cr.set_source_rgba(0.13, 0.77, 0.37, 0.4); // #22c55e glow
                    cr.arc(hx, hy, 12.0 * scale.max(0.6), 0.0, 2.0 * std::f64::consts::PI);
                    let _ = cr.fill();

                    cr.set_source_rgba(0.13, 0.77, 0.37, 0.9);
                    cr.arc(hx, hy, 7.0 * scale.max(0.6), 0.0, 2.0 * std::f64::consts::PI);
                    let _ = cr.fill();
                } else if anim_class == Some("pressed") {
                    cr.set_source_rgba(0.23, 0.51, 0.96, 0.9); // #3b82f6 pressed
                    cr.arc(hx, hy, 8.0 * scale.max(0.6), 0.0, 2.0 * std::f64::consts::PI);
                    let _ = cr.fill();
                } else if !hotspot.is_supported_on_linux {
                    cr.set_source_rgba(0.5, 0.5, 0.5, 0.3); // Gray disabled
                    cr.arc(hx, hy, 5.0 * scale.max(0.6), 0.0, 2.0 * std::f64::consts::PI);
                    let _ = cr.fill();
                } else {
                    cr.set_source_rgba(0.72, 0.73, 0.92, 0.85); // SVG default dot
                    cr.arc(hx, hy, 5.5 * scale.max(0.6), 0.0, 2.0 * std::f64::consts::PI);
                    let _ = cr.fill();
                }

                // Inner white center
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.95);
                cr.arc(hx, hy, 2.5 * scale.max(0.6), 0.0, 2.0 * std::f64::consts::PI);
                let _ = cr.fill();
            }
        });

        overlay.add_overlay(&drawing_area);

        // Function to trigger evdev event feedback animations on hotspots
        let da_ref = drawing_area.clone();
        let anim_trigger = Rc::new(move |btn: BackendButton, event_class: &str| {
            let class_str = event_class.to_string();
            anim_state.borrow_mut().insert(btn, (class_str, 300));
            da_ref.queue_draw();

            let da_queue = da_ref.clone();
            let anim_clean = anim_state.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
                anim_clean.borrow_mut().remove(&btn);
                da_queue.queue_draw();
                glib::ControlFlow::Break
            });
        });

        // Assembly: Left Cards | SVG Canvas Overlay | Right Cards
        main_box.append(&left_col);
        main_box.append(&overlay);
        main_box.append(&right_col);

        (main_box, selected_button, anim_trigger)
    }

    fn update_card_selections(map: &HashMap<BackendButton, Button>, selected: Option<BackendButton>) {
        for (btn, card) in map.iter() {
            if Some(*btn) == selected {
                card.add_css_class("selected");
            } else {
                card.remove_css_class("selected");
            }
        }
    }

    /// Build an individual OpenLogi Action Card widget
    fn build_action_card<F>(hotspot: &HotspotControl, profile: &Profile, on_click: F) -> Button
    where
        F: Fn(&Button) + 'static,
    {
        let card_btn = Button::new();
        card_btn.add_css_class("openlogi-card");
        if !hotspot.is_supported_on_linux {
            card_btn.add_css_class("disabled");
        }
        card_btn.set_tooltip_text(Some(hotspot.tooltip_pt));

        let inner_box = Box::new(Orientation::Vertical, 2);

        // Sublabel (e.g. "Clique do Meio")
        let sub_lbl = Label::new(Some(hotspot.name_pt));
        sub_lbl.add_css_class("card-label-sub");
        sub_lbl.set_halign(gtk4::Align::Start);

        // Title row (e.g. "Padrão do Sistema ›")
        let title_row = Box::new(Orientation::Horizontal, 8);

        let current_action = if hotspot.is_supported_on_linux {
            get_button_action_label(profile, hotspot.hex_code)
        } else {
            "Não suportado".to_string()
        };

        let title_lbl = Label::new(Some(&current_action));
        title_lbl.add_css_class("card-label-title");
        title_lbl.set_halign(gtk4::Align::Start);
        title_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        title_lbl.set_max_width_chars(16);

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
