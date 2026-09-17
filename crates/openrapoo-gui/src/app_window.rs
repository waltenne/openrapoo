//! Main Window Controller and GTK4 Layout Builder for OpenRapoo GUI — OpenLogi Minimalist Redesign.

#![allow(dead_code)]

use crate::views::{
    buttons_tab::ButtonsTabModel,
    device_tab::DeviceTabInfo,
    diag_tab::DiagTabState,
    home_view::HomeViewState,
    logs_view::LogsViewState,
    permissions_view::PermissionsCheckStatus,
    pointer_tab::PointerTabSettings,
    profiles_view::ProfilesViewState,
};

pub struct AppWindowController {
    pub home_state: HomeViewState,
    pub buttons_tab: ButtonsTabModel,
    pub pointer_tab: PointerTabSettings,
    pub device_tab: DeviceTabInfo,
    pub diag_tab: DiagTabState,
    pub profiles_state: ProfilesViewState,
    pub permissions_status: PermissionsCheckStatus,
    pub logs_state: LogsViewState,
}

impl Default for AppWindowController {
    fn default() -> Self {
        AppWindowController {
            home_state: HomeViewState::default(),
            buttons_tab: ButtonsTabModel::default(),
            pointer_tab: PointerTabSettings::default(),
            device_tab: DeviceTabInfo::default(),
            diag_tab: DiagTabState::default(),
            profiles_state: ProfilesViewState::default(),
            permissions_status: PermissionsCheckStatus::check(),
            logs_state: LogsViewState::default(),
        }
    }
}

#[cfg(feature = "gtk")]
pub fn build_gtk_ui(app: &gtk4::Application) {
    use gtk4::prelude::*;
    use gtk4::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::theme::load_custom_css;
    use crate::views::buttons_tab::gtk_ui::build_buttons_tab_ui;
    use crate::widgets::action_dialog::build_action_dialog_popover;

    load_custom_css();

    let controller = AppWindowController::default();
    let profile_store = Rc::new(RefCell::new(controller.profiles_state.store.clone()));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("OpenRapoo — Rapoo MT760 Pro")
        .default_width(1040)
        .default_height(720)
        .build();

    // --- 1. OPENLOGI MINIMAL HEADERBAR ---
    let header_bar = HeaderBar::new();
    header_bar.set_show_title_buttons(true);

    // Left: "< Back" Button + Device Name
    let left_header = Box::new(Orientation::Horizontal, 12);
    left_header.set_valign(Align::Center);

    let back_btn = Button::with_label("‹ Back");
    back_btn.add_css_class("back-button");

    let device_title = Label::new(Some("Rapoo MT760 Pro"));
    device_title.add_css_class("device-title");

    left_header.append(&back_btn);
    left_header.append(&device_title);
    header_bar.pack_start(&left_header);

    // Center: Floating Pill Tab Switcher (Buttons | Pointer | Device | Diagnostics)
    let pill_switcher = Box::new(Orientation::Horizontal, 4);
    pill_switcher.add_css_class("pill-switcher");

    let tab_buttons_btn = Button::with_label("Buttons");
    tab_buttons_btn.add_css_class("pill-tab");
    tab_buttons_btn.add_css_class("active");

    let tab_pointer_btn = Button::with_label("Pointer");
    tab_pointer_btn.add_css_class("pill-tab");

    let tab_device_btn = Button::with_label("Device");
    tab_device_btn.add_css_class("pill-tab");

    let tab_diag_btn = Button::with_label("Diagnostics");
    tab_diag_btn.add_css_class("pill-tab");

    pill_switcher.append(&tab_buttons_btn);
    pill_switcher.append(&tab_pointer_btn);
    pill_switcher.append(&tab_device_btn);
    pill_switcher.append(&tab_diag_btn);
    header_bar.set_title_widget(Some(&pill_switcher));

    // Right: Status Pill ("• Connected") + "+" Add Profile Button
    let right_header = Box::new(Orientation::Horizontal, 8);
    right_header.set_valign(Align::Center);

    let status_pill = Box::new(Orientation::Horizontal, 6);
    status_pill.add_css_class("status-pill");

    let status_dot = Box::new(Orientation::Horizontal, 0);
    status_dot.add_css_class("status-dot");

    let is_connected = controller.home_state.detected_device.is_some();
    if !is_connected {
        status_dot.add_css_class("disconnected");
    }

    let status_label = Label::new(Some(if is_connected { "Connected" } else { "Disconnected" }));
    status_label.add_css_class("status-text");

    status_pill.append(&status_dot);
    status_pill.append(&status_label);

    let add_prof_btn = Button::with_label("+");
    add_prof_btn.add_css_class("back-button");

    right_header.append(&status_pill);
    right_header.append(&add_prof_btn);
    header_bar.pack_end(&right_header);

    // --- 2. STACK FOR TAB PAGES ---
    let stack = Stack::new();
    stack.set_transition_type(StackTransitionType::Crossfade);

    // Tab 1: Buttons Page
    let current_profile = Rc::new(RefCell::new(profile_store.borrow().active_profile().clone()));
    
    let window_ref = window.clone();
    let profile_for_cb = current_profile.clone();
    
    let buttons_page = build_buttons_tab_ui(current_profile, move |btn_info| {
        let popover = build_action_dialog_popover(
            btn_info.hex_code,
            btn_info.name_pt,
            &profile_for_cb.borrow(),
            |_new_action| {
                // Action saved
            },
        );
        popover.set_parent(&window_ref);
        popover.popup();
    });

    stack.add_named(&buttons_page, Some("buttons"));

    // Tab 2: Pointer Page
    let pointer_page = Box::new(Orientation::Vertical, 16);
    pointer_page.add_css_class("main-canvas");
    pointer_page.set_margin_start(32);
    pointer_page.set_margin_end(32);
    pointer_page.set_margin_top(32);

    let ptr_title = Label::new(Some("Configurações do Ponteiro e Scroll"));
    ptr_title.add_css_class("device-title");
    ptr_title.set_halign(Align::Start);

    let speed_label = Label::new(Some("Velocidade do Ponteiro:"));
    speed_label.set_halign(Align::Start);
    let speed_scale = Scale::with_range(Orientation::Horizontal, 1.0, 10.0, 1.0);
    speed_scale.set_value(5.0);

    let accel_check = CheckButton::with_label("Aceleração do Ponteiro");
    accel_check.set_active(true);

    let natural_check = CheckButton::with_label("Rolagem Natural (Inverter direção do scroll)");

    let unsup_banner = Box::new(Orientation::Vertical, 6);
    unsup_banner.add_css_class("unsupported-banner");
    let unsup_title = Label::new(Some("⚠️ Ajuste de DPI e Polling Rate via Software"));
    unsup_title.add_css_class("card-label-title");
    unsup_title.set_halign(Align::Start);

    let unsup_desc = Label::new(Some(controller.pointer_tab.dpi_unsupported_reason_pt));
    unsup_desc.set_wrap(true);
    unsup_desc.set_halign(Align::Start);

    unsup_banner.append(&unsup_title);
    unsup_banner.append(&unsup_desc);

    pointer_page.append(&ptr_title);
    pointer_page.append(&speed_label);
    pointer_page.append(&speed_scale);
    pointer_page.append(&accel_check);
    pointer_page.append(&natural_check);
    pointer_page.append(&unsup_banner);

    stack.add_named(&pointer_page, Some("pointer"));

    // Tab 3: Device Page
    let device_page = Box::new(Orientation::Vertical, 12);
    device_page.add_css_class("main-canvas");
    device_page.set_margin_start(32);
    device_page.set_margin_end(32);
    device_page.set_margin_top(32);

    let dev_title = Label::new(Some("Informações do Dispositivo & Permissões"));
    dev_title.add_css_class("device-title");
    dev_title.set_halign(Align::Start);

    let lbl_mfg = Label::new(Some(&format!("Fabricante: {}", controller.device_tab.manufacturer)));
    lbl_mfg.set_halign(Align::Start);
    let lbl_mdl = Label::new(Some(&format!("Modelo: {}", controller.device_tab.model)));
    lbl_mdl.set_halign(Align::Start);
    let lbl_vid_pid = Label::new(Some(&format!("VID / PID: {} : {}", controller.device_tab.vendor_id_hex, controller.device_tab.product_id_hex)));
    lbl_vid_pid.set_halign(Align::Start);

    let udev_msg = if controller.permissions_status.udev_rule_exists {
        "Regras udev: Instaladas em /etc/udev/rules.d/99-openrapoo.rules ✓"
    } else {
        "Regras udev: Ausentes ✗ (Execute: sudo openrapoo-gui --install-udev)"
    };
    let lbl_udev = Label::new(Some(udev_msg));
    lbl_udev.set_halign(Align::Start);

    let group_msg = format!("Grupo input: {}", controller.permissions_status.input_group_state.display_message_pt());
    let lbl_grp = Label::new(Some(&group_msg));
    lbl_grp.set_halign(Align::Start);

    device_page.append(&dev_title);
    device_page.append(&lbl_mfg);
    device_page.append(&lbl_mdl);
    device_page.append(&lbl_vid_pid);
    device_page.append(&lbl_udev);
    device_page.append(&lbl_grp);

    stack.add_named(&device_page, Some("device"));

    // Tab 4: Diagnostics Page
    let diag_page = Box::new(Orientation::Vertical, 12);
    diag_page.add_css_class("main-canvas");
    diag_page.set_margin_start(32);
    diag_page.set_margin_end(32);
    diag_page.set_margin_top(32);

    let diag_title = Label::new(Some("Diagnóstico e Eventos evdev"));
    diag_title.add_css_class("device-title");
    diag_title.set_halign(Align::Start);

    let diag_desc = Label::new(Some("O OpenRapoo intercepta e reemite eventos através do nó /dev/uinput. Utilize o relatório para suporte técnico."));
    diag_desc.set_wrap(true);
    diag_desc.set_halign(Align::Start);

    let btn_diag_report = Button::with_label("Gerar Relatório de Diagnóstico");
    btn_diag_report.set_halign(Align::Start);

    diag_page.append(&diag_title);
    diag_page.append(&diag_desc);
    diag_page.append(&btn_diag_report);

    stack.add_named(&diag_page, Some("diag"));

    // --- TAB SWITCHER LOGIC ---
    let stack_ref = stack.clone();

    let p1 = stack_ref.clone();
    let b1 = tab_buttons_btn.clone();
    let b2 = tab_pointer_btn.clone();
    let b3 = tab_device_btn.clone();
    let b4 = tab_diag_btn.clone();

    tab_buttons_btn.connect_clicked(move |_| {
        p1.set_visible_child_name("buttons");
        b1.add_css_class("active");
        b2.remove_css_class("active");
        b3.remove_css_class("active");
        b4.remove_css_class("active");
    });

    let p2 = stack_ref.clone();
    let b1 = tab_buttons_btn.clone();
    let b2 = tab_pointer_btn.clone();
    let b3 = tab_device_btn.clone();
    let b4 = tab_diag_btn.clone();

    tab_pointer_btn.connect_clicked(move |_| {
        p2.set_visible_child_name("pointer");
        b2.add_css_class("active");
        b1.remove_css_class("active");
        b3.remove_css_class("active");
        b4.remove_css_class("active");
    });

    let p3 = stack_ref.clone();
    let b1 = tab_buttons_btn.clone();
    let b2 = tab_pointer_btn.clone();
    let b3 = tab_device_btn.clone();
    let b4 = tab_diag_btn.clone();

    tab_device_btn.connect_clicked(move |_| {
        p3.set_visible_child_name("device");
        b3.add_css_class("active");
        b1.remove_css_class("active");
        b2.remove_css_class("active");
        b4.remove_css_class("active");
    });

    let p4 = stack_ref;
    let b1 = tab_buttons_btn;
    let b2 = tab_pointer_btn;
    let b3 = tab_device_btn;
    let b4 = tab_diag_btn.clone();

    tab_diag_btn.connect_clicked(move |_| {
        p4.set_visible_child_name("diag");
        b4.add_css_class("active");
        b1.remove_css_class("active");
        b2.remove_css_class("active");
        b3.remove_css_class("active");
    });

    // --- 3. BOTTOM FOOTER ---
    let status_footer = Box::new(Orientation::Horizontal, 16);
    status_footer.add_css_class("status-footer");

    let udev_ok = controller.permissions_status.udev_rule_exists;
    let input_ok = controller.permissions_status.input_group_state.is_active();
    let foot_status = Label::new(Some(&format!(
        "• Regras udev {} | Grupo input {}",
        if udev_ok { "instaladas ✓" } else { "ausentes ✗" },
        if input_ok { "ativo ✓" } else { "pendente ⚠️" }
    )));
    foot_status.set_halign(Align::Start);
    foot_status.set_hexpand(true);

    let foot_ver = Label::new(Some(&format!("v{}", env!("CARGO_PKG_VERSION"))));
    foot_ver.set_halign(Align::End);

    status_footer.append(&foot_status);
    status_footer.append(&foot_ver);

    // Assemble Window
    let window_box = Box::new(Orientation::Vertical, 0);
    window_box.append(&header_bar);
    window_box.append(&stack);
    window_box.append(&status_footer);

    window.set_child(Some(&window_box));
    window.present();
}
