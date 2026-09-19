//! Main device configuration page containing centered tab navigation and layout sections.

use crate::device_model::RapooDevice;
use crate::i18n::Language;
use crate::state::DetailTab;
use crate::theme::current_theme;
use crate::ui::action_editor::render_action_editor;
use crate::ui::button_list::render_button_list;
use crate::ui::device_info_page::render_device_info_page;
use crate::ui::mouse_view::render_mouse_view;
use crate::ui::pointer_page::render_pointer_page;
use gpui::{
    div, px, ElementId, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex};
use openrapoo_core::config::{ButtonAction, Profile};
use std::rc::Rc;

pub fn render_device_page(
    device: &RapooDevice,
    active_tab: DetailTab,
    active_profile: &Profile,
    selected_hotspot_hex: Option<&str>,
    language: Language,
    on_select_tab: Rc<dyn Fn(DetailTab, &mut gpui::App)>,
    on_select_hotspot: Rc<dyn Fn(String, &mut gpui::App)>,
    on_update_action: Rc<dyn Fn(String, ButtonAction, &mut gpui::App)>,
    on_reset_action: Rc<dyn Fn(String, &mut gpui::App)>,
    on_update_dpi: Rc<dyn Fn(u32, &mut gpui::App)>,
    on_update_polling_rate: Rc<dyn Fn(u32, &mut gpui::App)>,
    on_apply_hardware: Rc<dyn Fn(&mut gpui::App)>,
    on_restore_hardware: Rc<dyn Fn(&mut gpui::App)>,
    on_read_hardware: Rc<dyn Fn(&mut gpui::App)>,
    on_refresh_battery: Rc<dyn Fn(&mut gpui::App)>,
    on_open_diag_modal: Rc<dyn Fn(&mut gpui::App)>,
) -> impl IntoElement {
    let theme = current_theme();

    use openrapoo_core::device::DeviceType;

    let available_tabs: Vec<DetailTab> = if device.device_type == DeviceType::Keyboard
        || !device.capabilities.has_buttons_remapping
    {
        vec![DetailTab::Device, DetailTab::Diagnostics]
    } else {
        vec![
            DetailTab::Buttons,
            DetailTab::Pointer,
            DetailTab::Device,
            DetailTab::Diagnostics,
        ]
    };

    let effective_tab = if available_tabs.contains(&active_tab) {
        active_tab
    } else {
        DetailTab::Device
    };

    let mut tab_pills = h_flex()
        .p_1()
        .gap_1()
        .rounded_xl()
        .bg(theme.panel_elevated)
        .border_1()
        .border_color(theme.border);

    for (idx, tab) in available_tabs.iter().enumerate() {
        let is_active = *tab == effective_tab;
        let bg = if is_active {
            theme.accent_blue
        } else {
            theme.panel_elevated
        };
        let fg = if is_active {
            theme.text_primary
        } else {
            theme.text_muted
        };

        let icon = match tab {
            DetailTab::Buttons => "🎛️",
            DetailTab::Pointer => "🎯",
            DetailTab::Device => "ℹ️",
            DetailTab::Diagnostics => "🔬",
        };

        let cb = on_select_tab.clone();
        let target_tab = *tab;

        tab_pills = tab_pills.child(
            div()
                .id(ElementId::from(idx + 500))
                .px_4()
                .py_1p5()
                .rounded_lg()
                .bg(bg)
                .text_color(fg)
                .text_size(px(13.0))
                .font_weight(if is_active {
                    gpui::FontWeight::BOLD
                } else {
                    gpui::FontWeight::MEDIUM
                })
                .cursor_pointer()
                .hover(move |s| {
                    if is_active {
                        s
                    } else {
                        s.bg(theme.border).text_color(theme.text_primary)
                    }
                })
                .on_click(move |_, _, cx| cb(target_tab, cx))
                .child(
                    h_flex()
                        .gap_1p5()
                        .items_center()
                        .child(div().text_size(px(12.0)).child(icon))
                        .child(tab.label(language)),
                ),
        );
    }

    let tab_bar = h_flex()
        .w_full()
        .py_2p5()
        .px_6()
        .justify_center()
        .items_center()
        .bg(theme.panel)
        .border_b_1()
        .border_color(theme.border)
        .child(tab_pills);

    let tab_content = match effective_tab {
        DetailTab::Buttons => h_flex()
            .flex_1()
            .w_full()
            .overflow_x_scrollbar()
            .child(render_button_list(
                active_profile,
                selected_hotspot_hex,
                language,
                on_select_hotspot.clone(),
            ))
            .child(render_mouse_view(selected_hotspot_hex, on_select_hotspot))
            .child(render_action_editor(
                active_profile,
                selected_hotspot_hex,
                language,
                on_update_action,
                on_reset_action,
            ))
            .into_any_element(),
        DetailTab::Pointer => render_pointer_page(
            device,
            active_profile,
            language,
            on_update_dpi,
            on_update_polling_rate,
            on_apply_hardware,
            on_restore_hardware,
            on_read_hardware,
        )
        .into_any_element(),
        DetailTab::Device => render_device_info_page(Some(device), language).into_any_element(),
        DetailTab::Diagnostics => {
            let report = crate::services::BatteryService::get_diagnostic_report(device);
            let cb_ref = on_refresh_battery.clone();
            crate::ui::battery_diag_tab::render_battery_diag_tab(
                device,
                &report,
                language,
                move |cx| cb_ref(cx),
                on_open_diag_modal,
            )
            .into_any_element()
        }
    };

    v_flex()
        .flex_1()
        .w_full()
        .overflow_hidden()
        .child(tab_bar)
        .child(tab_content)
}
