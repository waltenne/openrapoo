#[allow(dead_code)]
use crate::views::{
    diag_view::DiagViewState,
    home_view::HomeViewState,
    logs_view::LogsViewState,
    permissions_view::PermissionsCheckStatus,
    profiles_view::ProfilesViewState,
};

#[allow(dead_code)]
pub struct AppWindowController {
    pub home_state: HomeViewState,
    pub profiles_state: ProfilesViewState,
    pub diag_state: DiagViewState,
    pub permissions_status: PermissionsCheckStatus,
    pub logs_state: LogsViewState,
}

impl Default for AppWindowController {
    fn default() -> Self {
        AppWindowController {
            home_state: HomeViewState::default(),
            profiles_state: ProfilesViewState::default(),
            diag_state: DiagViewState::default(),
            permissions_status: PermissionsCheckStatus::check(),
            logs_state: LogsViewState::default(),
        }
    }
}

#[cfg(feature = "gtk")]
pub fn build_gtk_ui(app: &gtk4::Application) {
    use gtk4::prelude::*;
    use gtk4::*;

    let controller = AppWindowController::default();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("OpenRapoo — Configurador de Mouse Rapoo MT760 Pro")
        .default_width(720)
        .default_height(540)
        .build();

    let header_bar = HeaderBar::new();
    let title_label = Label::new(Some("OpenRapoo — Rapoo MT760 Pro"));
    title_label.add_css_class("title");
    header_bar.set_title_widget(Some(&title_label));

    let main_box = Box::new(Orientation::Vertical, 16);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);

    // Header Card
    let status_card = Frame::new(Some("Status do Dispositivo"));
    let status_box = Box::new(Orientation::Vertical, 8);
    status_box.set_margin_start(16);
    status_box.set_margin_end(16);
    status_box.set_margin_top(16);
    status_box.set_margin_bottom(16);

    let dev_name = controller
        .home_state
        .detected_device
        .as_ref()
        .map(|d| format!("Dispositivo: {} ({})", d.name, d.connection))
        .unwrap_or_else(|| "Dispositivo: Nenhum mouse Rapoo detectado".to_string());

    let label_dev = Label::new(Some(&dev_name));
    label_dev.set_halign(Align::Start);

    let udev_str = if controller.permissions_status.udev_rule_exists {
        "Regras udev: Instaladas em /etc/udev/rules.d/99-openrapoo.rules ✓"
    } else {
        "Regras udev: Ausentes ✗ (Execute: sudo openrapoo-gui --install-udev)"
    };
    let label_udev = Label::new(Some(udev_str));
    label_udev.set_halign(Align::Start);

    let group_str = format!(
        "Grupo input: {}",
        controller
            .permissions_status
            .input_group_state
            .display_message_pt()
    );
    let label_group = Label::new(Some(&group_str));
    label_group.set_halign(Align::Start);

    status_box.append(&label_dev);
    status_box.append(&label_udev);
    status_box.append(&label_group);
    status_card.set_child(Some(&status_box));

    // Profile Card
    let profile_card = Frame::new(Some("Perfis de Remapeamento"));
    let profile_box = Box::new(Orientation::Vertical, 10);
    profile_box.set_margin_start(16);
    profile_box.set_margin_end(16);
    profile_box.set_margin_top(16);
    profile_box.set_margin_bottom(16);

    let active_prof = controller.profiles_state.store.active_profile();
    let prof_info = format!(
        "Perfil Ativo: {} ({} botões mapeados)",
        active_prof.name,
        active_prof.mappings.len()
    );
    let label_prof = Label::new(Some(&prof_info));
    label_prof.set_halign(Align::Start);

    let btn_editor = Button::with_label("Editar Mapeamento de Botões");
    btn_editor.set_halign(Align::Start);

    profile_box.append(&label_prof);
    profile_box.append(&btn_editor);
    profile_card.set_child(Some(&profile_box));

    main_box.append(&status_card);
    main_box.append(&profile_card);

    let outer_box = Box::new(Orientation::Vertical, 0);
    outer_box.append(&header_bar);
    outer_box.append(&main_box);

    window.set_child(Some(&outer_box));
    window.present();
}
