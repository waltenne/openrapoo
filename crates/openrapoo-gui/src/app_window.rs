//! Main Window Controller and GTK4 Layout Builder for OpenRapoo GUI.

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
    use crate::theme::load_custom_css;
    use crate::widgets::mouse_view::{get_button_action_label, RAPOO_MT760_BUTTONS};

    load_custom_css();

    let controller = AppWindowController::default();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("OpenRapoo — Rapoo MT760 Pro Configurator")
        .default_width(960)
        .default_height(680)
        .build();

    // --- 1. TOP HEADER BAR ---
    let header_bar = HeaderBar::new();

    let back_btn = Button::from_icon_name("go-previous-symbolic");
    back_btn.set_tooltip_text(Some("Voltar para lista de dispositivos"));
    header_bar.pack_start(&back_btn);

    let title_box = Box::new(Orientation::Vertical, 0);
    title_box.set_valign(Align::Center);

    let title_label = Label::new(Some("Rapoo MT760 Pro"));
    title_label.add_css_class("openrapoo-title");

    let conn_text = controller
        .home_state
        .detected_device
        .as_ref()
        .map(|d| format!("Conectado ({})", d.connection))
        .unwrap_or_else(|| "Desconectado".to_string());
    let subtitle_label = Label::new(Some(&conn_text));
    subtitle_label.add_css_class("action-subtitle");

    title_box.append(&title_label);
    title_box.append(&subtitle_label);
    header_bar.set_title_widget(Some(&title_box));

    let conn_badge = Label::new(Some("2.4 GHz / NearLink"));
    conn_badge.add_css_class("connection-badge");
    header_bar.pack_end(&conn_badge);

    let manage_dev_btn = Button::with_label("Dispositivos");
    header_bar.pack_end(&manage_dev_btn);

    // --- 2. STACK & STACK SWITCHER (TABS) ---
    let stack = Stack::new();
    stack.set_transition_type(StackTransitionType::SlideLeftRight);

    let stack_switcher = StackSwitcher::new();
    stack_switcher.set_stack(Some(&stack));
    stack_switcher.set_halign(Align::Center);

    // --- TAB 1: BOTÕES ---
    let tab_buttons = Box::new(Orientation::Vertical, 12);
    tab_buttons.set_margin_start(16);
    tab_buttons.set_margin_end(16);
    tab_buttons.set_margin_top(16);
    tab_buttons.set_margin_bottom(16);

    // Profile Bar
    let profile_bar = Box::new(Orientation::Horizontal, 12);
    let active_prof = controller.profiles_state.store.active_profile();
    let prof_label = Label::new(Some(&format!("Perfil Ativo: {}", active_prof.name)));
    prof_label.add_css_class("openrapoo-title");

    let new_prof_btn = Button::with_label("+ Novo Perfil");
    profile_bar.append(&prof_label);
    profile_bar.append(&new_prof_btn);
    tab_buttons.append(&profile_bar);

    // Mouse Canvas + Side Cards Layout
    let mouse_layout = Box::new(Orientation::Horizontal, 20);
    mouse_layout.set_homogeneous(true);

    // Left Side Cards (Buttons 1..3)
    let left_cards = Box::new(Orientation::Vertical, 12);
    for btn_info in &RAPOO_MT760_BUTTONS[0..3] {
        let card = Frame::new(Some(&format!("[{}] {}", btn_info.id, btn_info.name_pt)));
        card.add_css_class("action-card");
        let card_box = Box::new(Orientation::Vertical, 4);

        let action_text = get_button_action_label(active_prof, btn_info.hex_code);
        let lbl_action = Label::new(Some(&action_text));
        lbl_action.add_css_class("action-title");
        lbl_action.set_halign(Align::Start);

        let lbl_desc = Label::new(Some(btn_info.description));
        lbl_desc.add_css_class("action-subtitle");
        lbl_desc.set_halign(Align::Start);

        let edit_btn = Button::with_label("Editar Ação");
        edit_btn.set_halign(Align::End);

        card_box.append(&lbl_action);
        card_box.append(&lbl_desc);
        card_box.append(&edit_btn);
        card.set_child(Some(&card_box));
        left_cards.append(&card);
    }

    // Center Illustration Box
    let center_mouse = Frame::new(Some("Rapoo MT760 Pro — Visão Geral"));
    let center_box = Box::new(Orientation::Vertical, 8);
    center_box.set_margin_start(16);
    center_box.set_margin_end(16);
    center_box.set_margin_top(16);
    center_box.set_margin_bottom(16);

    let mouse_img_label = Label::new(Some("🖱️\n\nRapoo MT760 Pro\n\n[1] Clique Esquerdo   [2] Clique Direito\n[3] Scroll / Meio     [4] Lateral Traseiro\n[5] Lateral Dianteiro [6] Scroll Lateral"));
    mouse_img_label.set_justify(Justification::Center);
    center_box.append(&mouse_img_label);
    center_mouse.set_child(Some(&center_box));

    // Right Side Cards (Buttons 4..6)
    let right_cards = Box::new(Orientation::Vertical, 12);
    for btn_info in &RAPOO_MT760_BUTTONS[3..6] {
        let card = Frame::new(Some(&format!("[{}] {}", btn_info.id, btn_info.name_pt)));
        card.add_css_class("action-card");
        let card_box = Box::new(Orientation::Vertical, 4);

        let action_text = get_button_action_label(active_prof, btn_info.hex_code);
        let lbl_action = Label::new(Some(&action_text));
        lbl_action.add_css_class("action-title");
        lbl_action.set_halign(Align::Start);

        let lbl_desc = Label::new(Some(btn_info.description));
        lbl_desc.add_css_class("action-subtitle");
        lbl_desc.set_halign(Align::Start);

        let edit_btn = Button::with_label("Editar Ação");
        edit_btn.set_halign(Align::End);

        card_box.append(&lbl_action);
        card_box.append(&lbl_desc);
        card_box.append(&edit_btn);
        card.set_child(Some(&card_box));
        right_cards.append(&card);
    }

    mouse_layout.append(&left_cards);
    mouse_layout.append(&center_mouse);
    mouse_layout.append(&right_cards);
    tab_buttons.append(&mouse_layout);

    stack.add_titled(&tab_buttons, Some("buttons"), "Botões");

    // --- TAB 2: PONTEIRO ---
    let tab_pointer = Box::new(Orientation::Vertical, 16);
    tab_pointer.set_margin_start(20);
    tab_pointer.set_margin_end(20);
    tab_pointer.set_margin_top(20);
    tab_pointer.set_margin_bottom(20);

    let ptr_frame = Frame::new(Some("Configurações do Ponteiro e Scroll"));
    let ptr_box = Box::new(Orientation::Vertical, 12);
    ptr_box.set_margin_start(16);
    ptr_box.set_margin_end(16);
    ptr_box.set_margin_top(16);
    ptr_box.set_margin_bottom(16);

    let speed_label = Label::new(Some("Velocidade do Ponteiro:"));
    speed_label.set_halign(Align::Start);
    let speed_scale = Scale::with_range(Orientation::Horizontal, 1.0, 10.0, 1.0);
    speed_scale.set_value(5.0);

    let accel_check = CheckButton::with_label("Aceleração do Ponteiro");
    accel_check.set_active(true);

    let natural_check = CheckButton::with_label("Rolagem Natural (Inverter direção do scroll)");

    ptr_box.append(&speed_label);
    ptr_box.append(&speed_scale);
    ptr_box.append(&accel_check);
    ptr_box.append(&natural_check);
    ptr_frame.set_child(Some(&ptr_box));
    tab_pointer.append(&ptr_frame);

    // Unsupported Hardware Features Banner (DPI & Polling Rate)
    let unsupported_banner = Frame::new(Some("Recursos de Hardware (DPI / Polling Rate)"));
    let unsup_box = Box::new(Orientation::Vertical, 8);
    unsup_box.add_css_class("unsupported-banner");
    unsup_box.set_margin_start(12);
    unsup_box.set_margin_end(12);
    unsup_box.set_margin_top(12);
    unsup_box.set_margin_bottom(12);

    let unsup_title = Label::new(Some("⚠️ Ajuste de DPI e Polling Rate via Software"));
    unsup_title.add_css_class("action-title");
    unsup_title.set_halign(Align::Start);

    let unsup_desc = Label::new(Some(controller.pointer_tab.dpi_unsupported_reason_pt));
    unsup_desc.set_wrap(true);
    unsup_desc.set_halign(Align::Start);

    unsup_box.append(&unsup_title);
    unsup_box.append(&unsup_desc);
    unsupported_banner.set_child(Some(&unsup_box));
    tab_pointer.append(&unsupported_banner);

    stack.add_titled(&tab_pointer, Some("pointer"), "Ponteiro");

    // --- TAB 3: DISPOSITIVO ---
    let tab_device = Box::new(Orientation::Vertical, 16);
    tab_device.set_margin_start(20);
    tab_device.set_margin_end(20);
    tab_device.set_margin_top(20);
    tab_device.set_margin_bottom(20);

    let dev_info_frame = Frame::new(Some("Informações Técnicas do Hardware"));
    let dev_info_box = Box::new(Orientation::Vertical, 10);
    dev_info_box.set_margin_start(16);
    dev_info_box.set_margin_end(16);
    dev_info_box.set_margin_top(16);
    dev_info_box.set_margin_bottom(16);

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
    let lbl_udev_st = Label::new(Some(udev_msg));
    lbl_udev_st.set_halign(Align::Start);

    let group_msg = format!("Grupo input: {}", controller.permissions_status.input_group_state.display_message_pt());
    let lbl_grp_st = Label::new(Some(&group_msg));
    lbl_grp_st.set_halign(Align::Start);

    let refresh_btn = Button::with_label("Atualizar Informações de Dispositivo");
    refresh_btn.set_halign(Align::Start);

    dev_info_box.append(&lbl_mfg);
    dev_info_box.append(&lbl_mdl);
    dev_info_box.append(&lbl_vid_pid);
    dev_info_box.append(&lbl_udev_st);
    dev_info_box.append(&lbl_grp_st);
    dev_info_box.append(&refresh_btn);
    dev_info_frame.set_child(Some(&dev_info_box));
    tab_device.append(&dev_info_frame);

    stack.add_titled(&tab_device, Some("device"), "Dispositivo");

    // --- TAB 4: DIAGNÓSTICO ---
    let tab_diag = Box::new(Orientation::Vertical, 16);
    tab_diag.set_margin_start(20);
    tab_diag.set_margin_end(20);
    tab_diag.set_margin_top(20);
    tab_diag.set_margin_bottom(20);

    let diag_frame = Frame::new(Some("Diagnóstico e Testador de Eventos"));
    let diag_box = Box::new(Orientation::Vertical, 12);
    diag_box.set_margin_start(16);
    diag_box.set_margin_end(16);
    diag_box.set_margin_top(16);
    diag_box.set_margin_bottom(16);

    let diag_desc = Label::new(Some("O OpenRapoo intercepta eventos evdev para remapeamento. Use o botão abaixo para exportar um relatório técnico de diagnósticos sem expor dados pessoais."));
    diag_desc.set_wrap(true);
    diag_desc.set_halign(Align::Start);

    let btn_report = Button::with_label("Gerar Relatório Técnico em Markdown");
    btn_report.set_halign(Align::Start);

    diag_box.append(&diag_desc);
    diag_box.append(&btn_report);
    diag_frame.set_child(Some(&diag_box));
    tab_diag.append(&diag_frame);

    stack.add_titled(&tab_diag, Some("diag"), "Diagnóstico");

    // --- 3. BOTTOM STATUS BAR ---
    let status_bar = Box::new(Orientation::Horizontal, 16);
    status_bar.add_css_class("status-bar");

    let status_permissions = Label::new(Some(&format!(
        "Permissões: udev {} | input {}",
        if controller.permissions_status.udev_rule_exists { "✓" } else { "✗" },
        if controller.permissions_status.input_group_state.is_active() { "✓" } else { "⚠️" }
    )));
    status_permissions.set_halign(Align::Start);
    status_permissions.set_hexpand(true);

    let status_version = Label::new(Some(&format!("OpenRapoo v{}", env!("CARGO_PKG_VERSION"))));
    status_version.set_halign(Align::End);

    status_bar.append(&status_permissions);
    status_bar.append(&status_version);

    // Assemble Main Window
    let window_box = Box::new(Orientation::Vertical, 0);
    window_box.append(&header_bar);
    window_box.append(&stack_switcher);
    window_box.append(&stack);
    window_box.append(&status_bar);

    window.set_child(Some(&window_box));
    window.present();
}
